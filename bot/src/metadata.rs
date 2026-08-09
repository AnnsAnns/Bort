use std::{collections::HashMap, sync::LazyLock};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, NaiveDate, NaiveDateTime};
use encoding_rs::Encoding;
use reqwest::{Client, header::CONTENT_TYPE};
use scraper::{Html, Selector};
use url::Url;

use crate::entry::tidy;

/// What we managed to scrape off a page. Everything is optional because a
/// depressing number of pages ship no metadata at all.
#[derive(Debug, Default, Clone)]
pub struct PageMetadata {
    /// The URL we ended up at, after redirects.
    pub final_url: Option<Url>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub site_name: Option<String>,
    pub author: Option<String>,
    pub published: Option<NaiveDate>,
}

static META: LazyLock<Selector> = LazyLock::new(|| Selector::parse("meta").unwrap());
static TITLE: LazyLock<Selector> = LazyLock::new(|| Selector::parse("title").unwrap());
static TIME: LazyLock<Selector> = LazyLock::new(|| Selector::parse("time[datetime]").unwrap());
static LD_JSON: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(r#"script[type="application/ld+json"]"#).unwrap());

/// Downloads a page and pulls the interesting bits out of it.
///
/// Never returns an error for "the page had no metadata" - only for the URL
/// being unusable or unreachable. A page we could not parse still yields a
/// `PageMetadata` with the fields we do know.
pub async fn scrape(client: &Client, url: &Url, max_bytes: usize) -> Result<PageMetadata> {
    if !matches!(url.scheme(), "http" | "https") {
        bail!("only http and https links are supported");
    }

    let mut response = client
        .get(url.clone())
        .send()
        .await
        .context("could not reach the page")?
        .error_for_status()
        .context("the page returned an error status")?;

    let final_url = response.url().clone();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    // Not `.text()`: a link to a 2 GB file should not take the bot down with it.
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .context("the download was cut short")?
    {
        let remaining = max_bytes.saturating_sub(body.len());
        if remaining == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
    }

    let mut metadata = PageMetadata {
        final_url: Some(final_url),
        ..Default::default()
    };

    let is_html = content_type
        .as_deref()
        .is_none_or(|value| value.contains("html") || value.contains("xml"));
    if is_html {
        let html = decode(&body, content_type.as_deref());
        extract(&html, &mut metadata);
    }

    Ok(metadata)
}

/// Honours the charset from the `Content-Type` header, falling back to the
/// `<meta charset>` in the document and then to UTF-8.
fn decode(body: &[u8], content_type: Option<&str>) -> String {
    let from_header = content_type
        .and_then(|value| {
            value
                .split(';')
                .filter_map(|part| part.trim().strip_prefix("charset="))
                .next()
        })
        .map(|charset| charset.trim_matches('"'))
        .and_then(|charset| Encoding::for_label(charset.as_bytes()));

    let encoding = from_header
        .or_else(|| sniff_charset(body))
        .unwrap_or(encoding_rs::UTF_8);

    encoding.decode(body).0.into_owned()
}

fn sniff_charset(body: &[u8]) -> Option<&'static Encoding> {
    let head = String::from_utf8_lossy(&body[..body.len().min(4096)]).to_ascii_lowercase();
    let rest = &head[head.find("charset=")? + "charset=".len()..];

    let label: String = rest
        .trim_start_matches(['"', '\''])
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':'))
        .collect();

    Encoding::for_label(label.as_bytes())
}

fn extract(html: &str, metadata: &mut PageMetadata) {
    let document = Html::parse_document(html);
    let meta = collect_meta(&document);
    let json_ld = collect_json_ld(&document);

    metadata.title = pick(&meta, &["og:title", "twitter:title"])
        .or_else(|| json_ld_string(&json_ld, "headline"))
        .or_else(|| {
            document
                .select(&TITLE)
                .next()
                .and_then(|element| tidy(&element.text().collect::<String>(), 200))
        });

    metadata.description = pick(
        &meta,
        &["og:description", "twitter:description", "description"],
    )
    .or_else(|| json_ld_string(&json_ld, "description"));

    metadata.site_name = pick(&meta, &["og:site_name", "application-name"]);

    metadata.author = pick(&meta, &["author", "article:author", "twitter:creator"])
        // `article:author` is very often a profile URL rather than a name.
        .filter(|author| Url::parse(author).is_err() && !author.starts_with('@'))
        .or_else(|| json_ld_author(&json_ld));

    metadata.published = pick(
        &meta,
        &[
            "article:published_time",
            "og:article:published_time",
            "date",
            "datepublished",
            "dc.date",
            "dcterms.date",
            "citation_publication_date",
            "parsely-pub-date",
            "sailthru.date",
        ],
    )
    .as_deref()
    .and_then(parse_date)
    .or_else(|| {
        json_ld_string(&json_ld, "datePublished")
            .as_deref()
            .and_then(parse_date)
    })
    .or_else(|| {
        document
            .select(&TIME)
            .filter_map(|element| element.value().attr("datetime"))
            .find_map(parse_date)
    });
}

/// All `<meta>` tags keyed by their `property` or `name`, lowercased. First
/// occurrence wins, which is what pages with duplicate tags usually intend.
fn collect_meta(document: &Html) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for element in document.select(&META) {
        let attributes = element.value();
        let Some(key) = attributes
            .attr("property")
            .or_else(|| attributes.attr("name"))
            .or_else(|| attributes.attr("itemprop"))
        else {
            continue;
        };
        let Some(content) = attributes.attr("content").and_then(|raw| tidy(raw, 500)) else {
            continue;
        };

        map.entry(key.trim().to_lowercase()).or_insert(content);
    }

    map
}

fn pick(meta: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| meta.get(*key).cloned())
}

fn collect_json_ld(document: &Html) -> Vec<serde_json::Value> {
    document
        .select(&LD_JSON)
        .filter_map(|element| serde_json::from_str(&element.text().collect::<String>()).ok())
        .collect()
}

/// JSON-LD blocks nest arbitrarily (`@graph`, arrays of things, ...), so just
/// walk the whole tree and take the first string under the wanted key.
fn json_ld_string(blocks: &[serde_json::Value], key: &str) -> Option<String> {
    fn walk(value: &serde_json::Value, key: &str) -> Option<String> {
        match value {
            serde_json::Value::Object(map) => map
                .get(key)
                .and_then(serde_json::Value::as_str)
                .and_then(|found| tidy(found, 500))
                .or_else(|| map.values().find_map(|nested| walk(nested, key))),
            serde_json::Value::Array(items) => items.iter().find_map(|item| walk(item, key)),
            _ => None,
        }
    }

    blocks.iter().find_map(|block| walk(block, key))
}

/// `author` is either a string or an object with a `name`.
fn json_ld_author(blocks: &[serde_json::Value]) -> Option<String> {
    fn walk(value: &serde_json::Value) -> Option<String> {
        match value {
            serde_json::Value::Object(map) => map
                .get("author")
                .and_then(|author| match author {
                    serde_json::Value::String(name) => tidy(name, 100),
                    serde_json::Value::Object(person) => person
                        .get("name")?
                        .as_str()
                        .and_then(|name| tidy(name, 100)),
                    serde_json::Value::Array(people) => {
                        people.first().and_then(|first| match first {
                            serde_json::Value::String(name) => tidy(name, 100),
                            other => other.get("name")?.as_str().and_then(|name| tidy(name, 100)),
                        })
                    }
                    _ => None,
                })
                .or_else(|| map.values().find_map(walk)),
            serde_json::Value::Array(items) => items.iter().find_map(walk),
            _ => None,
        }
    }

    blocks.iter().find_map(walk)
}

/// Every date format the web has thrown at this so far.
pub fn parse_date(raw: &str) -> Option<NaiveDate> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    // The overwhelmingly common case: something ISO-8601 shaped, where the
    // first ten characters are already the date we want.
    if let Some(head) = raw.get(..10)
        && let Ok(date) = NaiveDate::parse_from_str(head, "%Y-%m-%d")
    {
        return Some(date);
    }

    if let Ok(parsed) = DateTime::parse_from_rfc3339(raw) {
        return Some(parsed.date_naive());
    }
    if let Ok(parsed) = DateTime::parse_from_rfc2822(raw) {
        return Some(parsed.date_naive());
    }

    for format in [
        "%Y/%m/%d",
        "%d.%m.%Y",
        "%d/%m/%Y",
        "%B %d, %Y",
        "%d %B %Y",
        "%b %d, %Y",
    ] {
        if let Ok(date) = NaiveDate::parse_from_str(raw, format) {
            return Some(date);
        }
    }

    for format in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M:%S"] {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(raw, format) {
            return Some(parsed.date());
        }
    }

    None
}

/// Falls back to the hostname when a page does not name itself.
pub fn site_name(metadata: &PageMetadata, url: &Url) -> String {
    metadata
        .site_name
        .clone()
        .or_else(|| {
            url.host_str()
                .map(|host| host.trim_start_matches("www.").to_owned())
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

/// Last resort title for pages with no metadata at all: the last path segment,
/// tidied up into something readable.
pub fn title_from_url(url: &Url) -> String {
    url.path_segments()
        .and_then(|segments| {
            segments
                .filter(|segment| !segment.is_empty())
                .next_back()
                .map(|segment| {
                    segment
                        .trim_end_matches(".html")
                        .trim_end_matches(".htm")
                        .replace(['-', '_'], " ")
                })
        })
        .and_then(|title| tidy(&title, 200))
        .unwrap_or_else(|| url.host_str().unwrap_or("Untitled").to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_parse_in_the_formats_pages_actually_use() {
        let expected = NaiveDate::from_ymd_opt(2026, 8, 8).unwrap();

        for raw in [
            "2026-08-08",
            "2026-08-08T21:15:00Z",
            "2026-08-08T21:15:00+02:00",
            "2026-08-08 21:15:00",
            "2026/08/08",
            "08.08.2026",
            "August 8, 2026",
            "8 August 2026",
        ] {
            assert_eq!(parse_date(raw), Some(expected), "failed on {raw}");
        }

        assert_eq!(parse_date("Sat, 08 Aug 2026 21:15:00 GMT"), Some(expected));
        assert_eq!(parse_date("not a date"), None);
        assert_eq!(parse_date(""), None);
    }

    #[test]
    fn open_graph_wins_over_the_title_tag() {
        let mut metadata = PageMetadata::default();
        extract(
            r#"<html><head>
                <title>Post Title | Some Blog</title>
                <meta property="og:title" content="Post Title">
                <meta property="og:description" content="  A   description. ">
                <meta property="og:site_name" content="Some Blog">
                <meta property="article:published_time" content="2026-08-08T10:00:00Z">
                <meta name="author" content="Ann">
            </head><body></body></html>"#,
            &mut metadata,
        );

        assert_eq!(metadata.title.as_deref(), Some("Post Title"));
        assert_eq!(metadata.description.as_deref(), Some("A description."));
        assert_eq!(metadata.site_name.as_deref(), Some("Some Blog"));
        assert_eq!(metadata.author.as_deref(), Some("Ann"));
        assert_eq!(
            metadata.published,
            Some(NaiveDate::from_ymd_opt(2026, 8, 8).unwrap())
        );
    }

    #[test]
    fn falls_back_to_json_ld_and_the_title_tag() {
        let mut metadata = PageMetadata::default();
        extract(
            r#"<html><head>
                <title>  Bare   Title </title>
                <script type="application/ld+json">
                {"@graph":[{"@type":"Article","datePublished":"2026-07-27T09:00:00+00:00",
                 "author":{"@type":"Person","name":"Someone Else"}}]}
                </script>
            </head><body></body></html>"#,
            &mut metadata,
        );

        assert_eq!(metadata.title.as_deref(), Some("Bare Title"));
        assert_eq!(metadata.author.as_deref(), Some("Someone Else"));
        assert_eq!(
            metadata.published,
            Some(NaiveDate::from_ymd_opt(2026, 7, 27).unwrap())
        );
    }

    #[test]
    fn a_profile_url_is_not_an_author_name() {
        let mut metadata = PageMetadata::default();
        extract(
            r#"<html><head>
                <meta property="article:author" content="https://example.com/authors/ann">
            </head><body><time datetime="2026-01-02">Jan 2</time></body></html>"#,
            &mut metadata,
        );

        assert_eq!(metadata.author, None);
        assert_eq!(
            metadata.published,
            Some(NaiveDate::from_ymd_opt(2026, 1, 2).unwrap())
        );
    }

    #[test]
    fn latin1_pages_decode_correctly() {
        let body =
            b"<html><head><meta charset=\"iso-8859-1\"><title>Gr\xf6\xdfe</title></head></html>";
        assert!(decode(body, None).contains("Größe"));
        assert!(decode(body, Some("text/html; charset=iso-8859-1")).contains("Größe"));
    }

    /// Hits the network, so it is not part of the normal run.
    /// `cargo test -- --ignored --nocapture` to try the scraper against a real page.
    #[tokio::test]
    #[ignore = "requires network access"]
    async fn scrapes_a_real_page() {
        let client = Client::builder()
            .user_agent("BortLinkbot/test")
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .unwrap();
        let url = Url::parse("https://annsann.eu/blog/the_internet_does_forget/").unwrap();

        let metadata = scrape(&client, &url, 4 * 1024 * 1024).await.unwrap();
        println!("{metadata:#?}");

        assert!(metadata.title.is_some(), "no title found");
        assert!(metadata.description.is_some(), "no description found");
    }

    #[test]
    fn urls_give_a_usable_last_resort_title() {
        let url = Url::parse("https://example.com/blog/some-cool-post.html").unwrap();
        assert_eq!(title_from_url(&url), "some cool post");

        let url = Url::parse("https://example.com/").unwrap();
        assert_eq!(title_from_url(&url), "example.com");
    }
}
