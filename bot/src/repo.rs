use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use tokio::{process::Command, sync::Mutex};

use crate::{
    config::Config,
    entry::{Draft, LinkEntry, same_link, unique_id},
};

/// A working clone of the website repository.
///
/// Every mutation goes through `publish`, which is serialised behind a mutex -
/// two `/link` invocations at once would otherwise race on the same worktree.
pub struct Repo {
    config: Arc<Config>,
    /// Absolute. `config.workdir` may be relative.
    workdir: PathBuf,
    /// Absolute, always `workdir/repo`.
    clone_dir: PathBuf,
    askpass: PathBuf,
    lock: Mutex<()>,
}

#[derive(Debug)]
pub struct Published {
    pub entry: LinkEntry,
    pub commit: String,
    pub commit_url: Option<String>,
}

/// How many times a push may lose a race against another commit before we
/// give up and tell the user.
const PUSH_ATTEMPTS: usize = 3;

impl Repo {
    pub async fn open(config: Arc<Config>) -> Result<Self> {
        tokio::fs::create_dir_all(&config.workdir)
            .await
            .with_context(|| format!("could not create {}", config.workdir.display()))?;

        // Resolved up front, because every git call below either runs *in* one
        // of these directories or passes one as an argument. A relative
        // `WORKDIR` would otherwise be interpreted against the git process's
        // own working directory and nest the clone inside itself.
        let workdir = tokio::fs::canonicalize(&config.workdir)
            .await
            .with_context(|| format!("could not resolve {}", config.workdir.display()))?;

        let repo = Self {
            clone_dir: workdir.join("repo"),
            askpass: write_askpass(&workdir).await?,
            workdir,
            config,
            lock: Mutex::new(()),
        };

        let _guard = repo.lock.lock().await;
        repo.ensure_clone().await?;
        drop(_guard);

        Ok(repo)
    }

    /// Reads the links file as it currently is on the remote.
    pub async fn read_links(&self) -> Result<Vec<LinkEntry>> {
        let _guard = self.lock.lock().await;
        self.sync().await?;
        self.load_links().await
    }

    /// Appends `draft` to the links file and pushes the result.
    pub async fn publish(&self, draft: Draft) -> Result<Published> {
        let _guard = self.lock.lock().await;
        let mut last_error = None;

        for attempt in 1..=PUSH_ATTEMPTS {
            self.sync().await?;
            let mut links = self.load_links().await?;

            if let Some(existing) = links
                .iter()
                .find(|entry| same_link(&entry.url, draft.url.as_str()))
            {
                bail!("that link is already on the site as `{}`", existing.id);
            }

            let added = Utc::now();
            let entry = draft
                .clone()
                .into_entry(unique_id(added, &draft.title, &links), added);

            // Newest first, matching how the page renders it.
            links.insert(0, entry.clone());
            self.write_links(&links).await?;

            self.git(&["add", "--", &self.config.links_path]).await?;
            self.git(&[
                "-c",
                &format!("user.name={}", self.config.commit_name),
                "-c",
                &format!("user.email={}", self.config.commit_email),
                "commit",
                "--quiet",
                "-m",
                &commit_message(&entry),
            ])
            .await
            .context("could not create the commit")?;

            let push = self
                .git(&[
                    "push",
                    "origin",
                    &format!("HEAD:refs/heads/{}", self.config.git_branch),
                ])
                .await;

            match push {
                Ok(_) => {
                    let commit = self.git(&["rev-parse", "HEAD"]).await?.trim().to_owned();
                    let commit_url = self
                        .config
                        .web_url()
                        .map(|base| format!("{base}/commit/{commit}"));

                    return Ok(Published {
                        entry,
                        commit,
                        commit_url,
                    });
                }
                Err(error) => {
                    tracing::warn!(attempt, %error, "push was rejected, retrying");
                    last_error = Some(error);
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| anyhow::anyhow!("push failed"))
            .context(format!("push still failing after {PUSH_ATTEMPTS} attempts")))
    }

    async fn ensure_clone(&self) -> Result<()> {
        if self.clone_dir.join(".git").is_dir() {
            return Ok(());
        }

        tracing::info!(dir = %self.clone_dir.display(), "cloning the website repository");
        if self.clone_dir.exists() {
            tokio::fs::remove_dir_all(&self.clone_dir).await.ok();
        }

        let remote = self.config.authenticated_remote()?;
        let clone_dir = self.clone_dir.display().to_string();

        // Shallow: we only ever add a commit on top of the tip, so the history
        // is dead weight (and this repo carries a lot of images).
        self.git_in(
            &self.workdir,
            &[
                "clone",
                "--quiet",
                "--depth=1",
                "--single-branch",
                "--branch",
                &self.config.git_branch,
                &remote,
                &clone_dir,
            ],
        )
        .await
        .context("could not clone the repository - check GIT_REMOTE and GIT_TOKEN")?;

        Ok(())
    }

    /// Throws away whatever is in the worktree and matches the remote exactly.
    /// Cheaper and far more predictable than trying to merge.
    async fn sync(&self) -> Result<()> {
        self.ensure_clone().await?;

        self.git(&[
            "fetch",
            "--quiet",
            "--depth=1",
            "origin",
            &self.config.git_branch,
        ])
        .await
        .context("could not fetch from the remote")?;
        self.git(&["reset", "--quiet", "--hard", "FETCH_HEAD"])
            .await?;
        self.git(&["clean", "--quiet", "-fd"]).await?;

        Ok(())
    }

    fn links_file(&self) -> PathBuf {
        self.clone_dir.join(&self.config.links_path)
    }

    async fn load_links(&self) -> Result<Vec<LinkEntry>> {
        let path = self.links_file();
        let raw = match tokio::fs::read_to_string(&path).await {
            Ok(raw) => raw,
            // A missing file is fine - the first link creates it.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                return Err(error).with_context(|| format!("could not read {}", path.display()));
            }
        };

        if raw.trim().is_empty() {
            return Ok(Vec::new());
        }

        serde_json::from_str(&raw)
            .with_context(|| format!("{} is not a valid list of links", self.config.links_path))
    }

    async fn write_links(&self, links: &[LinkEntry]) -> Result<()> {
        let path = self.links_file();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut json = serde_json::to_string_pretty(links)?;
        json.push('\n');

        tokio::fs::write(&path, json)
            .await
            .with_context(|| format!("could not write {}", path.display()))
    }

    async fn git(&self, args: &[&str]) -> Result<String> {
        self.git_in(&self.clone_dir, args).await
    }

    async fn git_in(&self, dir: &Path, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            // Never wait on a terminal we do not have; hand the token over via
            // the environment rather than the command line, where it would be
            // visible to anything that can read /proc.
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_ASKPASS", &self.askpass)
            .env("BORT_GIT_TOKEN", &self.config.git_token)
            // Ignore whatever git config the host happens to have.
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .stdin(Stdio::null())
            .output()
            .await
            .context("could not run git - is it installed?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "git {} failed: {}",
                args.first().copied().unwrap_or_default(),
                redact(stderr.trim(), &self.config.git_token)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

/// git asks for the password by running this; it just prints the token.
async fn write_askpass(workdir: &Path) -> Result<PathBuf> {
    let path = workdir.join("askpass.sh");
    tokio::fs::write(&path, "#!/bin/sh\nprintf '%s\\n' \"$BORT_GIT_TOKEN\"\n")
        .await
        .with_context(|| format!("could not write {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).await?;
    }

    path.canonicalize()
        .with_context(|| format!("could not resolve {}", path.display()))
}

fn commit_message(entry: &LinkEntry) -> String {
    // Single line, no quotes to escape - the title goes in as-is since git is
    // invoked without a shell.
    format!("links: add \"{}\"", entry.title.replace('\n', " "))
}

/// Belt and braces: make sure a token never lands in a Discord message.
fn redact(text: &str, token: &str) -> String {
    if token.is_empty() {
        return text.to_owned();
    }
    text.replace(token, "***")
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::process::Command as SyncCommand;

    use url::Url;

    /// Drives the real git binary against a local bare repository, so the
    /// clone / sync / commit / push path is exercised for real rather than
    /// mocked.
    fn test_setup(dir: &Path) -> Config {
        let origin = dir.join("origin.git");
        let seed = dir.join("seed");

        git(
            dir,
            &[
                "init",
                "--quiet",
                "--bare",
                "-b",
                "main",
                origin.to_str().unwrap(),
            ],
        );
        git(
            dir,
            &["init", "--quiet", "-b", "main", seed.to_str().unwrap()],
        );
        std::fs::create_dir_all(seed.join("src/content")).unwrap();
        std::fs::write(seed.join("src/content/links.json"), "[]\n").unwrap();
        git(&seed, &["add", "."]);
        git(
            &seed,
            &[
                "-c",
                "user.name=Seed",
                "-c",
                "user.email=seed@example.com",
                "commit",
                "--quiet",
                "-m",
                "initial",
            ],
        );
        git(
            &seed,
            &["remote", "add", "origin", origin.to_str().unwrap()],
        );
        git(&seed, &["push", "--quiet", "origin", "main"]);

        Config {
            discord_token: "unused".to_owned(),
            allowed_users: vec![1],
            guild_id: None,
            git_remote: Url::from_directory_path(&origin).unwrap().to_string(),
            git_branch: "main".to_owned(),
            git_token: "unused".to_owned(),
            git_username: "x-access-token".to_owned(),
            workdir: dir.join("work"),
            links_path: "src/content/links.json".to_owned(),
            commit_name: "Test Bot".to_owned(),
            commit_email: "bot@example.com".to_owned(),
            user_agent: "test".to_owned(),
            fetch_timeout: std::time::Duration::from_secs(5),
            max_page_bytes: 1024,
        }
    }

    fn git(dir: &Path, args: &[&str]) {
        let output = SyncCommand::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn draft(url: &str, title: &str) -> Draft {
        Draft {
            url: Url::parse(url).unwrap(),
            title: title.to_owned(),
            site: "example.com".to_owned(),
            author: Some("Ann".to_owned()),
            description: Some("A description".to_owned()),
            comment: Some("Worth a read".to_owned()),
            tags: vec!["rust".to_owned()],
            pub_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()),
        }
    }

    #[tokio::test]
    async fn publishing_commits_and_pushes_to_the_remote() {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repo::open(Arc::new(test_setup(dir.path()))).await.unwrap();

        assert!(repo.read_links().await.unwrap().is_empty());

        let published = repo
            .publish(draft("https://example.com/post", "A Cool Post"))
            .await
            .unwrap();
        assert!(
            published.entry.id.ends_with("-a-cool-post"),
            "{}",
            published.entry.id
        );
        assert!(!published.commit.is_empty());

        // Newest first, and the file shape is what the Astro schema expects.
        let raw = String::from_utf8(
            SyncCommand::new("git")
                .current_dir(dir.path().join("origin.git"))
                .args(["show", "main:src/content/links.json"])
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();

        assert!(raw.contains("\"addedDate\""), "{raw}");
        assert!(raw.contains("\"pubDate\": \"2026-07-01\""), "{raw}");
        assert!(raw.ends_with("]\n"), "file should end with a newline");

        let pushed: Vec<LinkEntry> = serde_json::from_str(&raw).unwrap();
        assert_eq!(pushed.len(), 1);
        assert_eq!(pushed[0].url, "https://example.com/post");
        assert_eq!(
            pushed[0].tags.as_deref(),
            Some(["rust".to_owned()].as_slice())
        );
    }

    #[tokio::test]
    async fn the_same_link_is_not_added_twice() {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repo::open(Arc::new(test_setup(dir.path()))).await.unwrap();

        repo.publish(draft("https://example.com/post", "A Cool Post"))
            .await
            .unwrap();

        // Same article, shared with tracking junk and a trailing slash.
        let error = repo
            .publish(draft("https://www.example.com/post/", "A Cool Post"))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("already on the site"), "{error}");

        assert_eq!(repo.read_links().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn a_second_link_keeps_the_first_one() {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repo::open(Arc::new(test_setup(dir.path()))).await.unwrap();

        repo.publish(draft("https://example.com/one", "First"))
            .await
            .unwrap();
        repo.publish(draft("https://example.com/two", "Second"))
            .await
            .unwrap();

        let links = repo.read_links().await.unwrap();
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].title, "Second", "newest entry should be first");
        assert_eq!(links[1].title, "First");
    }

    /// Someone else pushing between our fetch and our push must not lose their
    /// commit or ours.
    #[tokio::test]
    async fn a_concurrent_push_is_retried_on_top() {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repo::open(Arc::new(test_setup(dir.path()))).await.unwrap();

        repo.publish(draft("https://example.com/one", "First"))
            .await
            .unwrap();

        // Simulate an unrelated commit landing on main from elsewhere.
        let seed = dir.path().join("seed");
        git(&seed, &["pull", "--quiet", "origin", "main"]);
        std::fs::write(seed.join("README.md"), "hello\n").unwrap();
        git(&seed, &["add", "."]);
        git(
            &seed,
            &[
                "-c",
                "user.name=Someone",
                "-c",
                "user.email=someone@example.com",
                "commit",
                "--quiet",
                "-m",
                "unrelated",
            ],
        );
        git(&seed, &["push", "--quiet", "origin", "main"]);

        repo.publish(draft("https://example.com/two", "Second"))
            .await
            .unwrap();

        let links = repo.read_links().await.unwrap();
        assert_eq!(links.len(), 2);

        // The unrelated commit survived.
        let files = String::from_utf8(
            SyncCommand::new("git")
                .current_dir(dir.path().join("origin.git"))
                .args(["ls-tree", "--name-only", "main"])
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();
        assert!(files.contains("README.md"), "{files}");
    }

    #[test]
    fn tokens_never_make_it_into_error_messages() {
        let text = "fatal: could not read from https://x:ghp_secret@github.com/a/b.git";
        assert_eq!(
            redact(text, "ghp_secret"),
            "fatal: could not read from https://x:***@github.com/a/b.git"
        );
    }

    #[test]
    fn commit_messages_stay_on_one_line() {
        let entry = LinkEntry {
            id: "id".to_owned(),
            url: "https://example.com".to_owned(),
            title: "A title\nwith a newline".to_owned(),
            site: "example.com".to_owned(),
            author: None,
            description: None,
            comment: None,
            tags: None,
            pub_date: None,
            added_date: "2026-08-08T00:00:00Z".to_owned(),
        };

        assert_eq!(
            commit_message(&entry),
            "links: add \"A title with a newline\""
        );
    }
}
