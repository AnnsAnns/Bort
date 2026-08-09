use std::{env, path::PathBuf, time::Duration};

use anyhow::{Context, Result, bail};

/// Everything the bot needs, all of it read from the environment (or a `.env`
/// file next to the binary). See `.env.example`.
#[derive(Debug)]
pub struct Config {
    pub discord_token: String,
    /// Only these Discord user IDs may run the command. Deliberately has no
    /// "allow everyone" mode - this thing pushes to a live website.
    pub allowed_users: Vec<u64>,
    /// Register slash commands in this guild only, which makes them show up
    /// immediately instead of after Discord's global command propagation.
    pub guild_id: Option<u64>,

    /// Clone URL, e.g. `https://github.com/AnnsAnns/Bort.git`
    pub git_remote: String,
    pub git_branch: String,
    /// Fine grained PAT with `contents: read and write` on the repo.
    pub git_token: String,
    /// Username half of the HTTPS basic auth pair. GitHub ignores it for PATs.
    pub git_username: String,
    /// Where the working clone lives. Created on first start.
    pub workdir: PathBuf,
    /// Path of the links file *inside* the repository.
    pub links_path: String,
    pub commit_name: String,
    pub commit_email: String,

    pub user_agent: String,
    pub fetch_timeout: Duration,
    /// Hard cap on how much of a page we download before giving up on it.
    pub max_page_bytes: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let allowed_users = parse_id_list(&required("ALLOWED_USERS")?)
            .context("ALLOWED_USERS must be a comma separated list of Discord user IDs")?;
        if allowed_users.is_empty() {
            bail!("ALLOWED_USERS is empty - refusing to let anyone publish to the site");
        }

        let guild_id = match env::var("GUILD_ID") {
            Ok(raw) if !raw.trim().is_empty() => {
                Some(raw.trim().parse().context("GUILD_ID must be a number")?)
            }
            _ => None,
        };

        Ok(Self {
            discord_token: required("DISCORD_TOKEN")?,
            allowed_users,
            guild_id,

            git_remote: optional("GIT_REMOTE", "https://github.com/AnnsAnns/Bort.git"),
            git_branch: optional("GIT_BRANCH", "main"),
            git_token: required("GIT_TOKEN")?,
            git_username: optional("GIT_USERNAME", "x-access-token"),
            workdir: PathBuf::from(optional("WORKDIR", "./workdir")),
            links_path: optional("LINKS_PATH", "src/content/links.json"),
            commit_name: optional("COMMIT_NAME", "Bort Linkbot"),
            commit_email: optional("COMMIT_EMAIL", "bort@annsann.eu"),

            user_agent: optional(
                "USER_AGENT",
                concat!(
                    "BortLinkbot/",
                    env!("CARGO_PKG_VERSION"),
                    " (+https://annsann.eu/links)"
                ),
            ),
            fetch_timeout: Duration::from_secs(
                optional("FETCH_TIMEOUT_SECS", "20")
                    .parse()
                    .context("FETCH_TIMEOUT_SECS must be a number")?,
            ),
            max_page_bytes: optional("MAX_PAGE_BYTES", "4194304")
                .parse()
                .context("MAX_PAGE_BYTES must be a number")?,
        })
    }

    /// The remote with the auth username spliced in, so git only ever asks us
    /// for the password (which `GIT_ASKPASS` then answers with the token).
    pub fn authenticated_remote(&self) -> Result<String> {
        let mut url = url::Url::parse(&self.git_remote).context("GIT_REMOTE is not a valid URL")?;

        // Only http(s) carries basic auth; anything else (file://, ssh) is
        // handed to git untouched.
        if matches!(url.scheme(), "http" | "https") && url.username().is_empty() {
            url.set_username(&self.git_username)
                .ok()
                .context("could not add the username to GIT_REMOTE")?;
        }

        Ok(url.to_string())
    }

    /// `https://github.com/owner/repo.git` -> `https://github.com/owner/repo`,
    /// so we can link the commit we just pushed.
    pub fn web_url(&self) -> Option<String> {
        let url = url::Url::parse(&self.git_remote).ok()?;
        let path = url.path().trim_end_matches(".git");
        Some(format!("{}://{}{}", url.scheme(), url.host_str()?, path))
    }
}

fn required(key: &str) -> Result<String> {
    let value = env::var(key).with_context(|| format!("{key} is not set"))?;
    if value.trim().is_empty() {
        bail!("{key} is empty");
    }
    Ok(value)
}

fn optional(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => value,
        _ => default.to_owned(),
    }
}

fn parse_id_list(raw: &str) -> Result<Vec<u64>> {
    raw.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| part.parse::<u64>().map_err(Into::into))
        .collect()
}
