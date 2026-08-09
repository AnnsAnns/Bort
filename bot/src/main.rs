//! Discord bot that turns `/link <url>` into a commit on the website repo.
//!
//! It scrapes the page for its title, description, author and publication
//! date, shows the result as an ephemeral preview, and - once confirmed -
//! pulls the website repository, appends the entry to `src/content/links.json`
//! and pushes. GitHub Actions takes it from there.

mod commands;
mod config;
mod entry;
mod metadata;
mod repo;

use std::sync::Arc;

use anyhow::{Context as _, Result};
use poise::serenity_prelude as serenity;
use tracing_subscriber::EnvFilter;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Shared state handed to every command invocation.
pub struct Data {
    pub config: Arc<config::Config>,
    pub http: reqwest::Client,
    pub repo: Arc<repo::Repo>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("bort_linkbot=info,warn")),
        )
        .init();

    let config = Arc::new(config::Config::from_env()?);
    let token = config.discord_token.clone();
    let guild_id = config.guild_id;

    let http = reqwest::Client::builder()
        .user_agent(&config.user_agent)
        .timeout(config.fetch_timeout)
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .context("could not build the HTTP client")?;

    // Cloning up front means the first `/link` is not the one paying for it,
    // and a broken token or remote fails loudly at startup instead of in a
    // Discord message.
    let repo = Arc::new(repo::Repo::open(Arc::clone(&config)).await?);
    tracing::info!(remote = %config.git_remote, branch = %config.git_branch, "repository ready");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::link()],
            on_error: |error| Box::pin(report_error(error)),
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                match guild_id {
                    // Guild commands appear immediately, global ones can take
                    // up to an hour to propagate.
                    Some(id) => {
                        poise::builtins::register_in_guild(
                            ctx,
                            &framework.options().commands,
                            serenity::GuildId::new(id),
                        )
                        .await?
                    }
                    None => {
                        poise::builtins::register_globally(ctx, &framework.options().commands)
                            .await?
                    }
                }

                tracing::info!(user = %ready.user.name, "logged in");
                Ok(Data { config, http, repo })
            })
        })
        .build();

    // No intents needed: slash commands arrive as interactions, not messages.
    serenity::ClientBuilder::new(&token, serenity::GatewayIntents::empty())
        .framework(framework)
        .await
        .context("could not create the Discord client")?
        .start()
        .await
        .context("the Discord client stopped")?;

    Ok(())
}

async fn report_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            tracing::error!(%error, command = ctx.command().name, "command failed");

            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .ephemeral(true)
                        .content(format!("❌ {error}")),
                )
                .await;
        }
        other => {
            if let Err(error) = poise::builtins::on_error(other).await {
                tracing::error!(%error, "the error handler itself failed");
            }
        }
    }
}
