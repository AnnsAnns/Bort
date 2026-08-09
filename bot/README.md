# Bort Linkbot

Discord bot that adds entries to the [Cool Links](https://annsann.eu/links)
page. Post a link with `/link`, check the preview, press publish — it commits
to this repository and the existing deploy workflow puts it on the site.

```
/link url:<url> [comment:<text>] [tags:<a, b>] [site:<override>]
```

The reply is ephemeral, so nobody else in the channel sees the back and forth.

## How it works

1. Fetches the URL and pulls out the title, description, author and publication
   date from OpenGraph tags, then `<meta>` tags, then JSON-LD, then whatever
   the `<title>` and `<time>` elements say.
2. Shows the result as an embed with **Publish** / **Edit** / **Discard**.
   *Edit* opens a modal for the title, description, comment, tags and date —
   scraped metadata is frequently wrong or clickbaity, so nothing is committed
   until you say so.
3. On publish: fetches the branch, resets the working clone to it, prepends the
   entry to `src/content/links.json`, commits and pushes. If someone else
   pushed in the meantime the push is retried on top of their commit, up to
   three times.

Duplicate links are rejected — comparison ignores `www.`, trailing slashes,
scheme, and `utm_*`-style tracking parameters, which are also stripped before
the URL is stored.

## Setup

Create an application at <https://discord.com/developers/applications>, add a
bot, and invite it with the `applications.commands` scope. It needs no
gateway intents and no message permissions.

Create a fine-grained personal access token scoped to this repository with
**Contents: Read and write**.

```sh
cp .env.example .env
$EDITOR .env
cargo run --release
```

`git` must be on `PATH`. The first start clones the repository into `WORKDIR`
and fails loudly if the token or remote is wrong, rather than waiting until
someone runs the command.

## Running it as a service

```ini
# /etc/systemd/system/bort-linkbot.service
[Unit]
Description=Bort Linkbot
After=network-online.target

[Service]
Type=simple
User=bort
WorkingDirectory=/Bort/bot
EnvironmentFile=/Bort/bot/.env
ExecStart=/Bort/bot/target/release/bort-linkbot
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

## Tests

```sh
cargo test              # includes a real clone/commit/push against a temporary local repo
cargo test -- --ignored # additionally scrapes a live page
```

## Notes

- `WORKDIR` gets `git reset --hard` and `git clean -fd` on every publish. Point
  it at a disposable directory, never at a checkout you work in.
- The token is handed to git through `GIT_ASKPASS` and the environment, so it
  never appears in a command line or in `.git/config`, and it is redacted from
  any error message that reaches Discord.
- The entry format is validated by the website build (`src/content.config.ts`).
  If you change the shape of an entry, change it in both places — otherwise the
  bot pushes happily and the deploy fails.
