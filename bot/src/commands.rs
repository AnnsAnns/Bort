use std::time::Duration;

use poise::serenity_prelude as serenity;
use url::Url;

use crate::{
    Context, Error,
    entry::{Draft, clean_url, parse_tags, tidy},
    metadata,
};

/// How long the preview keeps responding to button presses. Discord's
/// interaction tokens die after 15 minutes, so this has to stay below that.
const PREVIEW_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const MODAL_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// Add a link to the Cool Links page on annsann.eu.
#[poise::command(slash_command, rename = "link", check = "may_publish")]
pub async fn link(
    ctx: Context<'_>,
    #[description = "Link to the article"] url: String,
    #[description = "Why it is worth reading"] comment: Option<String>,
    #[description = "Comma separated, e.g. rust, embedded"] tags: Option<String>,
    #[description = "Override the source name (defaults to the domain)"] site: Option<String>,
) -> Result<(), Error> {
    // Scraping takes longer than Discord's three second budget.
    ctx.defer_ephemeral().await?;

    let requested = parse_url(&url)?;
    let data = ctx.data();
    let scraped = metadata::scrape(&data.http, &requested, data.config.max_page_bytes).await?;

    // Prefer where we ended up over where we were pointed - shortener and
    // tracking links would otherwise be stored as-is.
    let final_url = clean_url(scraped.final_url.as_ref().unwrap_or(&requested));

    let mut draft = Draft {
        title: scraped
            .title
            .clone()
            .unwrap_or_else(|| metadata::title_from_url(&final_url)),
        site: site
            .as_deref()
            .and_then(|value| tidy(value, 100))
            .unwrap_or_else(|| metadata::site_name(&scraped, &final_url)),
        author: scraped.author.clone(),
        description: scraped.description.clone(),
        comment: comment.as_deref().and_then(|value| tidy(value, 500)),
        tags: tags.as_deref().map(parse_tags).unwrap_or_default(),
        pub_date: scraped.published,
        url: final_url,
    };

    let buttons = ButtonIds::new(ctx.id());
    let handle = ctx.send(preview(&draft, &buttons, None)).await?;

    loop {
        let press = serenity::ComponentInteractionCollector::new(ctx)
            .custom_ids(buttons.all())
            .timeout(PREVIEW_TIMEOUT)
            .await;

        let Some(press) = press else {
            handle
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content("🕒 Preview expired, nothing was published.")
                        .components(Vec::new()),
                )
                .await?;
            return Ok(());
        };

        if press.data.custom_id == buttons.edit {
            // Responds to the interaction with the modal, waits for the submit
            // and acknowledges it, so nothing else to do for this press.
            let submitted = poise::execute_modal_on_component_interaction(
                ctx,
                press,
                Some(EditModal::from(&draft)),
                Some(MODAL_TIMEOUT),
            )
            .await?;

            let warning = submitted.and_then(|values| values.apply(&mut draft));
            handle
                .edit(ctx, preview(&draft, &buttons, warning.as_deref()))
                .await?;
            continue;
        }

        press
            .create_response(ctx, serenity::CreateInteractionResponse::Acknowledge)
            .await?;

        if press.data.custom_id == buttons.cancel {
            handle
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content("🗑️ Discarded, nothing was published.")
                        .components(Vec::new()),
                )
                .await?;
            return Ok(());
        }

        handle
            .edit(
                ctx,
                poise::CreateReply::default()
                    .content("⏳ Committing and pushing…")
                    .components(Vec::new()),
            )
            .await?;

        let reply = match data.repo.publish(draft.clone()).await {
            Ok(published) => {
                tracing::info!(id = %published.entry.id, commit = %published.commit, "published link");

                let commit = match &published.commit_url {
                    Some(url) => format!(
                        "[`{}`]({url})",
                        &published.commit[..7.min(published.commit.len())]
                    ),
                    None => format!("`{}`", published.commit),
                };

                poise::CreateReply::default().content(format!(
                    "✅ Published **{}**\n{} — live in a minute or two at <https://annsann.eu/links>",
                    published.entry.title, commit
                ))
            }
            Err(error) => {
                tracing::error!(%error, "publishing failed");
                poise::CreateReply::default().content(format!("❌ Could not publish: {error:#}"))
            }
        };

        handle.edit(ctx, reply.components(Vec::new())).await?;
        return Ok(());
    }
}

/// Only the people listed in `ALLOWED_USERS` get to write to the website.
async fn may_publish(ctx: Context<'_>) -> Result<bool, Error> {
    if ctx
        .data()
        .config
        .allowed_users
        .contains(&ctx.author().id.get())
    {
        return Ok(true);
    }

    tracing::warn!(user = %ctx.author().id, "rejected an unauthorised /link");
    ctx.send(
        poise::CreateReply::default()
            .ephemeral(true)
            .content("❌ You are not allowed to publish links."),
    )
    .await?;

    Ok(false)
}

fn parse_url(raw: &str) -> Result<Url, Error> {
    let raw = raw.trim();
    let url = Url::parse(raw)
        .or_else(|_| Url::parse(&format!("https://{raw}")))
        .map_err(|_| "that does not look like a link")?;

    if !matches!(url.scheme(), "http" | "https") {
        return Err("only http and https links can be added".into());
    }

    Ok(url)
}

/// Custom IDs are namespaced by the interaction ID so two people running the
/// command at the same time do not collect each other's button presses.
struct ButtonIds {
    publish: String,
    edit: String,
    cancel: String,
}

impl ButtonIds {
    fn new(interaction: u64) -> Self {
        Self {
            publish: format!("{interaction}:publish"),
            edit: format!("{interaction}:edit"),
            cancel: format!("{interaction}:cancel"),
        }
    }

    fn all(&self) -> Vec<String> {
        vec![self.publish.clone(), self.edit.clone(), self.cancel.clone()]
    }

    fn row(&self) -> serenity::CreateActionRow {
        serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(self.publish.clone())
                .emoji('🚀')
                .label("Publish")
                .style(serenity::ButtonStyle::Success),
            serenity::CreateButton::new(self.edit.clone())
                .emoji('✏')
                .label("Edit")
                .style(serenity::ButtonStyle::Secondary),
            serenity::CreateButton::new(self.cancel.clone())
                .emoji('🗑')
                .label("Discard")
                .style(serenity::ButtonStyle::Danger),
        ])
    }
}

fn preview(draft: &Draft, buttons: &ButtonIds, warning: Option<&str>) -> poise::CreateReply {
    let mut embed = serenity::CreateEmbed::new()
        .title(draft.title.as_str())
        .url(draft.url.as_str())
        .description(
            draft
                .description
                .as_deref()
                .unwrap_or("*no description found*"),
        )
        .field("Source", draft.site.as_str(), true)
        .field(
            "Published",
            draft
                .pub_date
                .map(|date| date.to_string())
                .unwrap_or_else(|| "unknown".to_owned()),
            true,
        )
        .footer(serenity::CreateEmbedFooter::new(
            "Nothing has been committed yet",
        ));

    if let Some(author) = &draft.author {
        embed = embed.field("Author", author.as_str(), true);
    }
    if !draft.tags.is_empty() {
        embed = embed.field(
            "Tags",
            draft
                .tags
                .iter()
                .map(|tag| format!("#{tag}"))
                .collect::<Vec<_>>()
                .join(" "),
            false,
        );
    }
    if let Some(comment) = &draft.comment {
        embed = embed.field("Your comment", comment.as_str(), false);
    }

    let content = match warning {
        Some(warning) => format!("⚠️ {warning}"),
        None => "Check it over, then publish:".to_owned(),
    };

    poise::CreateReply::default()
        .ephemeral(true)
        .content(content)
        .embed(embed)
        .components(vec![buttons.row()])
}

/// Discord allows at most five inputs per modal, which is exactly what we need.
/// The source name is not in here - it is derived from the domain and can be
/// overridden with the command's `site` option in the rare case it is wrong.
#[derive(Debug, poise::Modal)]
#[name = "Edit link"]
struct EditModal {
    #[name = "Title"]
    #[max_length = 200]
    title: String,
    #[name = "Description"]
    #[paragraph]
    #[max_length = 500]
    description: Option<String>,
    #[name = "Your comment"]
    #[paragraph]
    #[max_length = 500]
    comment: Option<String>,
    #[name = "Tags"]
    #[placeholder = "rust, embedded"]
    #[max_length = 200]
    tags: Option<String>,
    #[name = "Published"]
    #[placeholder = "YYYY-MM-DD, leave empty for unknown"]
    #[max_length = 10]
    pub_date: Option<String>,
}

impl From<&Draft> for EditModal {
    fn from(draft: &Draft) -> Self {
        Self {
            title: draft.title.clone(),
            description: draft.description.clone(),
            comment: draft.comment.clone(),
            tags: (!draft.tags.is_empty()).then(|| draft.tags.join(", ")),
            pub_date: draft.pub_date.map(|date| date.to_string()),
        }
    }
}

impl EditModal {
    /// Writes the submitted values back onto the draft, returning a warning if
    /// something could not be used.
    fn apply(self, draft: &mut Draft) -> Option<String> {
        let mut warning = None;

        if let Some(title) = tidy(&self.title, 200) {
            draft.title = title;
        }
        draft.description = self
            .description
            .as_deref()
            .and_then(|value| tidy(value, 500));
        draft.comment = self.comment.as_deref().and_then(|value| tidy(value, 500));
        draft.tags = self.tags.as_deref().map(parse_tags).unwrap_or_default();

        draft.pub_date = match self.pub_date.as_deref().map(str::trim) {
            None | Some("") => None,
            Some(raw) => {
                let parsed = metadata::parse_date(raw);
                if parsed.is_none() {
                    warning = Some(format!(
                        "Could not read `{raw}` as a date, left it as unknown."
                    ));
                }
                parsed
            }
        };

        warning
    }
}
