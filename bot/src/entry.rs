use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

/// One entry of `src/content/links.json`.
///
/// Keep this in sync with the `links` collection schema in
/// `src/content.config.ts` - the website validates every field on build, so a
/// mismatch here breaks the deploy rather than failing quietly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkEntry {
    pub id: String,
    pub url: String,
    pub title: String,
    pub site: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// `YYYY-MM-DD` - when the linked article was published.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pub_date: Option<String>,
    /// RFC 3339 in UTC - when it was added here. Sorts the feed.
    pub added_date: String,
}

/// A pending entry: everything except the id, which can only be assigned once
/// we know what is already in the file.
#[derive(Debug, Clone)]
pub struct Draft {
    pub url: Url,
    pub title: String,
    pub site: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub comment: Option<String>,
    pub tags: Vec<String>,
    pub pub_date: Option<NaiveDate>,
}

impl Draft {
    pub fn into_entry(self, id: String, added: DateTime<Utc>) -> LinkEntry {
        LinkEntry {
            id,
            url: self.url.to_string(),
            title: self.title,
            site: self.site,
            author: self.author,
            description: self.description,
            comment: self.comment,
            tags: (!self.tags.is_empty()).then_some(self.tags),
            pub_date: self.pub_date.map(|date| date.to_string()),
            added_date: added.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        }
    }
}

/// Builds `2026-08-08-some-article-title`, suffixed with `-2`, `-3`, ... if
/// that is already taken.
pub fn unique_id(added: DateTime<Utc>, title: &str, existing: &[LinkEntry]) -> String {
    let date = added.date_naive();
    let slug = slugify(title);
    let slug = if slug.is_empty() {
        "link".to_owned()
    } else {
        slug
    };

    let base = format!("{date}-{slug}");
    if !existing.iter().any(|entry| entry.id == base) {
        return base;
    }

    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|candidate| !existing.iter().any(|entry| &entry.id == candidate))
        .expect("the range is unbounded")
}

const MAX_SLUG_LEN: usize = 60;

/// Lowercase ASCII words joined by dashes. Common German and Romance
/// characters get transliterated instead of dropped, so "Größe" becomes
/// "groesse" rather than "gr-e".
pub fn slugify(title: &str) -> String {
    let mut slug = String::new();

    for ch in title.chars() {
        match ch {
            'a'..='z' | '0'..='9' => slug.push(ch),
            'A'..='Z' => slug.push(ch.to_ascii_lowercase()),
            'ä' | 'Ä' => slug.push_str("ae"),
            'ö' | 'Ö' => slug.push_str("oe"),
            'ü' | 'Ü' => slug.push_str("ue"),
            'ß' => slug.push_str("ss"),
            // Deliberately spelled out rather than using ranges: the ranges
            // would swallow the umlauts handled above.
            'á' | 'â' | 'ã' | 'å' | 'à' | 'Á' | 'Â' | 'Ã' | 'Å' | 'À' => slug.push('a'),
            'é' | 'ê' | 'ë' | 'è' | 'É' | 'Ê' | 'Ë' | 'È' => slug.push('e'),
            'í' | 'î' | 'ï' | 'ì' | 'Í' | 'Î' | 'Ï' | 'Ì' => slug.push('i'),
            'ó' | 'ô' | 'õ' | 'ò' | 'Ó' | 'Ô' | 'Õ' | 'Ò' => slug.push('o'),
            'ú' | 'û' | 'ù' | 'Ú' | 'Û' | 'Ù' => slug.push('u'),
            'ñ' | 'Ñ' => slug.push('n'),
            'ç' | 'Ç' => slug.push('c'),
            'ł' | 'Ł' => slug.push('l'),
            _ if !slug.ends_with('-') => slug.push('-'),
            _ => {}
        }
    }

    if slug.len() > MAX_SLUG_LEN {
        // Cut on a word boundary if there is one nearby, otherwise hard cut.
        let cut = slug[..MAX_SLUG_LEN]
            .rfind('-')
            .filter(|idx| *idx > MAX_SLUG_LEN / 2)
            .unwrap_or(MAX_SLUG_LEN);
        slug.truncate(cut);
    }

    slug.trim_matches('-').to_owned()
}

/// `"Rust, #embedded ,,web"` -> `["rust", "embedded", "web"]`
pub fn parse_tags(raw: &str) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();

    for tag in raw.split([',', ' ']) {
        let tag = tag.trim().trim_start_matches('#').to_lowercase();
        if !tag.is_empty() && !tags.contains(&tag) {
            tags.push(tag);
        }
    }

    tags
}

/// Drops the tracking junk that gets copied along with a link, so the same
/// article shared from two places is recognised as a duplicate.
const TRACKING_PARAMS: &[&str] = &[
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "utm_id",
    "fbclid",
    "gclid",
    "msclkid",
    "mc_cid",
    "mc_eid",
    "igshid",
    "si",
];

pub fn clean_url(url: &Url) -> Url {
    let mut cleaned = url.clone();

    let kept: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(key, _)| !TRACKING_PARAMS.contains(&key.as_ref()))
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();

    if kept.is_empty() {
        cleaned.set_query(None);
    } else {
        cleaned.query_pairs_mut().clear().extend_pairs(kept);
    }

    cleaned.set_fragment(None);
    cleaned
}

/// Loose equality for duplicate detection: ignores scheme, `www.`, a trailing
/// slash and case in the host.
pub fn same_link(a: &str, b: &str) -> bool {
    fn key(raw: &str) -> String {
        match Url::parse(raw) {
            Ok(url) => format!(
                "{}{}{}",
                url.host_str()
                    .unwrap_or_default()
                    .trim_start_matches("www."),
                url.path().trim_end_matches('/'),
                url.query().map(|q| format!("?{q}")).unwrap_or_default()
            )
            .to_lowercase(),
            Err(_) => raw.to_lowercase(),
        }
    }

    key(a) == key(b)
}

/// Collapses whitespace and cuts overly long scraped strings down to size.
pub fn tidy(raw: &str, max_len: usize) -> Option<String> {
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return None;
    }

    if collapsed.chars().count() <= max_len {
        return Some(collapsed);
    }

    let mut truncated: String = collapsed.chars().take(max_len.saturating_sub(1)).collect();
    while truncated.ends_with(|c: char| c.is_whitespace() || c.is_ascii_punctuation()) {
        truncated.pop();
    }
    truncated.push('…');
    Some(truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_are_url_safe() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("Die Größe der Bäume"), "die-groesse-der-baeume");
        assert_eq!(slugify("  --Rust 2024--  "), "rust-2024");
        assert_eq!(slugify("日本語"), "");
    }

    #[test]
    fn long_slugs_are_cut_on_a_word_boundary() {
        let slug = slugify(
            "a very long title that keeps going and going well past the sixty character limit",
        );
        assert!(slug.len() <= MAX_SLUG_LEN, "{slug}");
        assert!(!slug.ends_with('-'), "{slug}");
    }

    #[test]
    fn ids_do_not_collide() {
        let added = "2026-08-08T21:15:00Z".parse::<DateTime<Utc>>().unwrap();
        let existing = vec![LinkEntry {
            id: "2026-08-08-test".to_owned(),
            url: "https://example.com".to_owned(),
            title: "Test".to_owned(),
            site: "example.com".to_owned(),
            author: None,
            description: None,
            comment: None,
            tags: None,
            pub_date: None,
            added_date: "2026-08-08T21:15:00Z".to_owned(),
        }];

        assert_eq!(unique_id(added, "Test", &existing), "2026-08-08-test-2");
        assert_eq!(unique_id(added, "Other", &existing), "2026-08-08-other");
    }

    #[test]
    fn tags_are_normalised() {
        assert_eq!(
            parse_tags("Rust, #embedded ,,rust"),
            vec!["rust", "embedded"]
        );
        assert!(parse_tags("  ,, ").is_empty());
    }

    #[test]
    fn tracking_params_are_stripped() {
        let url = Url::parse("https://example.com/post?utm_source=discord&id=7#section").unwrap();
        assert_eq!(clean_url(&url).as_str(), "https://example.com/post?id=7");

        let url = Url::parse("https://example.com/post?utm_source=discord").unwrap();
        assert_eq!(clean_url(&url).as_str(), "https://example.com/post");
    }

    #[test]
    fn duplicates_are_recognised_across_spellings() {
        assert!(same_link(
            "https://www.Example.com/post/",
            "http://example.com/post"
        ));
        assert!(!same_link(
            "https://example.com/post",
            "https://example.com/other"
        ));
    }

    #[test]
    fn tidy_collapses_and_truncates() {
        assert_eq!(tidy("  a \n b  ", 50).as_deref(), Some("a b"));
        assert_eq!(tidy("   ", 50), None);
        assert_eq!(tidy("abcdefghij", 5).as_deref(), Some("abcd…"));
    }
}
