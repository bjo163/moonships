//! Stable but conservative identity helpers for attributed intelligence items.
use crate::connectors::valid_https_url;
use crate::hash_tools::sha256_text;
use crate::intelligence_items::{MAX_ITEM_SUMMARY_CHARS, MAX_ITEM_TITLE_CHARS};

const FINGERPRINT_DOMAIN: &str = "moonships-intelligence-content-v1\0";
const TRACKING_QUERY_KEYS: [&str; 6] = ["fbclid", "gclid", "dclid", "msclkid", "mc_cid", "mc_eid"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalUrl {
    /// Exact URL supplied by the source. Retained for attribution and display.
    pub original: String,
    /// Normalized identity URL, with fragment and known tracking parameters removed.
    pub canonical: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityError {
    InvalidUrl,
    InvalidContent,
    ContentTooLarge,
}

/// Normalize URL syntax while preserving the original source URL separately.
/// Only HTTPS URLs are accepted; path, non-tracking query fields and their order stay intact.
pub fn canonicalize_item_url(original: &str) -> Result<CanonicalUrl, IdentityError> {
    if !valid_https_url(original) {
        return Err(IdentityError::InvalidUrl);
    }
    let mut url = reqwest::Url::parse(original).map_err(|_| IdentityError::InvalidUrl)?;
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
        return Err(IdentityError::InvalidUrl);
    }

    url.set_fragment(None);
    if url.query().is_some() {
        let retained: Vec<(String, String)> = url
            .query_pairs()
            .filter(|(key, _)| !is_tracking_parameter(key))
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect();
        url.set_query(None);
        if !retained.is_empty() {
            url.query_pairs_mut().extend_pairs(retained);
        }
    }
    let canonical = url.to_string();
    if canonical.len() > 2_048 {
        return Err(IdentityError::InvalidUrl);
    }
    Ok(CanonicalUrl {
        original: original.to_owned(),
        canonical,
    })
}

fn is_tracking_parameter(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.starts_with("utm_") || TRACKING_QUERY_KEYS.contains(&key.as_str())
}

/// Create a versioned SHA-256 fingerprint from title and optional summary.
/// Leading/trailing and repeated Unicode whitespace are collapsed; letter case and punctuation stay intact.
pub fn content_fingerprint(title: &str, summary: Option<&str>) -> Result<String, IdentityError> {
    if title.trim().is_empty()
        || title.chars().count() > MAX_ITEM_TITLE_CHARS
        || summary.is_some_and(|value| value.chars().count() > MAX_ITEM_SUMMARY_CHARS)
    {
        return Err(IdentityError::InvalidContent);
    }
    let title = normalize_whitespace(title);
    let summary = summary.map(normalize_whitespace);
    let mut input = String::with_capacity(FINGERPRINT_DOMAIN.len() + title.len() + 32);
    input.push_str(FINGERPRINT_DOMAIN);
    append_fingerprint_field(&mut input, Some(&title));
    append_fingerprint_field(&mut input, summary.as_deref());
    sha256_text(&input).map_err(|_| IdentityError::ContentTooLarge)
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn append_fingerprint_field(target: &mut String, value: Option<&str>) {
    match value {
        Some(value) => {
            target.push('\u{1}');
            target.push_str(&value.len().to_string());
            target.push(':');
            target.push_str(value);
        }
        None => target.push('\u{0}'),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_urls_are_stable_but_keep_the_exact_original_reference() {
        let first = canonicalize_item_url("https://EXAMPLE.com/news?id=42&utm_source=mail#section")
            .unwrap();
        let second =
            canonicalize_item_url("https://example.com/news?utm_source=feed&id=42#other").unwrap();
        assert_eq!(first.canonical, "https://example.com/news?id=42");
        assert_eq!(first.canonical, second.canonical);
        assert_eq!(
            first.original,
            "https://EXAMPLE.com/news?id=42&utm_source=mail#section"
        );
        assert_ne!(first.original, first.canonical);
    }

    #[test]
    fn canonicalization_only_removes_known_tracking_parameters() {
        let url =
            canonicalize_item_url("https://example.com/path?ref=partner&FBCLID=abc&x=1").unwrap();
        assert_eq!(url.canonical, "https://example.com/path?ref=partner&x=1");
    }

    #[test]
    fn canonicalization_rejects_unsafe_or_invalid_urls() {
        for url in [
            "http://example.com/story",
            "https://user@example.com/story",
            "file:///etc/passwd",
            "not a url",
        ] {
            assert_eq!(canonicalize_item_url(url), Err(IdentityError::InvalidUrl));
        }
    }

    #[test]
    fn fingerprints_normalize_whitespace_and_keep_fields_unambiguous() {
        let first = content_fingerprint("  Crew   update ", Some("Mission\n status")).unwrap();
        let second = content_fingerprint("Crew update", Some("Mission status")).unwrap();
        assert_eq!(first, second);
        assert_ne!(
            content_fingerprint("ab", Some("c")).unwrap(),
            content_fingerprint("a", Some("bc")).unwrap()
        );
        assert_ne!(
            content_fingerprint("Title", None).unwrap(),
            content_fingerprint("Title", Some("")).unwrap()
        );
    }

    #[test]
    fn fingerprints_reject_unbounded_or_empty_titles() {
        assert_eq!(
            content_fingerprint(" ", None),
            Err(IdentityError::InvalidContent)
        );
        assert_eq!(
            content_fingerprint(&"x".repeat(MAX_ITEM_TITLE_CHARS + 1), None),
            Err(IdentityError::InvalidContent)
        );
    }
}
