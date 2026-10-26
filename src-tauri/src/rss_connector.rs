//! A single-request, user-configured RSS adapter. It does not schedule or persist feeds.
use crate::connectors::{
    AccessMethod, ConnectorDefinition, ConnectorHealth, ConnectorKind, TermsReview,
};
use crate::intelligence_items::{
    IntelligenceItem, ItemFreshness, ItemVerification, MAX_EXTERNAL_ID_CHARS,
    MAX_ITEM_SUMMARY_CHARS, MAX_ITEM_TITLE_CHARS,
};
use chrono::{DateTime, SecondsFormat, Utc};
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use std::collections::HashMap;
use std::io::Read;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use uuid::Uuid;

const MAX_XML_EVENTS: usize = 100_000;
const MAX_TRACKED_CONNECTORS: usize = 256;
const MAX_RESOLVED_ADDRESSES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RssFetchError {
    Disabled,
    InvalidConnector,
    RateLimited,
    ResolverBusy,
    ResolutionFailed,
    UnsafeAddress,
    RequestTimedOut,
    NetworkFailure,
    HttpStatus,
    ResponseTooLarge,
    MalformedFeed,
    InvalidItem,
}

#[derive(Default)]
struct PollLimiter {
    last_attempt: HashMap<String, Instant>,
}

impl PollLimiter {
    fn claim(&mut self, id: &str, interval: Duration, now: Instant) -> Result<(), RssFetchError> {
        if let Some(previous) = self.last_attempt.get(id) {
            if now.saturating_duration_since(*previous) < interval {
                return Err(RssFetchError::RateLimited);
            }
        } else if self.last_attempt.len() >= MAX_TRACKED_CONNECTORS {
            return Err(RssFetchError::RateLimited);
        }
        self.last_attempt.insert(id.to_owned(), now);
        Ok(())
    }
}

static POLL_LIMITER: OnceLock<Mutex<PollLimiter>> = OnceLock::new();
static DNS_WORKER_BUSY: AtomicBool = AtomicBool::new(false);

trait RssTransport {
    fn fetch(&self, connector: &ConnectorDefinition) -> Result<Vec<u8>, RssFetchError>;
}

struct HttpRssTransport;

impl RssTransport for HttpRssTransport {
    fn fetch(&self, connector: &ConnectorDefinition) -> Result<Vec<u8>, RssFetchError> {
        let url = reqwest::Url::parse(&connector.source_url)
            .map_err(|_| RssFetchError::InvalidConnector)?;
        if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
            return Err(RssFetchError::InvalidConnector);
        }
        let host = url.host_str().ok_or(RssFetchError::InvalidConnector)?;
        let timeout = Duration::from_millis(u64::from(connector.request_timeout_ms));
        let started_at = Instant::now();
        let addresses = resolve_public_addresses(host, timeout)?;
        let remaining_timeout = timeout.saturating_sub(started_at.elapsed());
        if remaining_timeout.is_zero() {
            return Err(RssFetchError::RequestTimedOut);
        }
        let client = Client::builder()
            .https_only(true)
            .no_proxy()
            .redirect(Policy::none())
            .timeout(remaining_timeout)
            .resolve_to_addrs(host, &addresses)
            .build()
            .map_err(|_| RssFetchError::NetworkFailure)?;
        let mut response = client
            .get(url)
            .header(
                reqwest::header::ACCEPT,
                "application/rss+xml, application/xml, text/xml;q=0.9",
            )
            .header(reqwest::header::USER_AGENT, "Moonships/0.1 RSS reader")
            .send()
            .map_err(map_request_error)?;
        if !response.status().is_success() {
            return Err(RssFetchError::HttpStatus);
        }
        let limit = connector.maximum_response_bytes;
        if response
            .content_length()
            .is_some_and(|length| length > limit)
        {
            return Err(RssFetchError::ResponseTooLarge);
        }
        let mut body = Vec::with_capacity(limit.min(64 * 1024) as usize);
        response
            .by_ref()
            .take(limit.saturating_add(1))
            .read_to_end(&mut body)
            .map_err(map_io_error)?;
        if body.len() as u64 > limit {
            return Err(RssFetchError::ResponseTooLarge);
        }
        Ok(body)
    }
}

fn map_request_error(error: reqwest::Error) -> RssFetchError {
    if error.is_timeout() {
        RssFetchError::RequestTimedOut
    } else {
        RssFetchError::NetworkFailure
    }
}

fn map_io_error(error: std::io::Error) -> RssFetchError {
    if error.kind() == std::io::ErrorKind::TimedOut {
        RssFetchError::RequestTimedOut
    } else {
        RssFetchError::NetworkFailure
    }
}

fn resolve_public_addresses(
    host: &str,
    timeout: Duration,
) -> Result<Vec<SocketAddr>, RssFetchError> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        if !is_public_address(ip) {
            return Err(RssFetchError::UnsafeAddress);
        }
        return Ok(vec![SocketAddr::new(ip, 443)]);
    }

    if DNS_WORKER_BUSY.swap(true, Ordering::AcqRel) {
        return Err(RssFetchError::ResolverBusy);
    }
    let hostname = host.to_owned();
    let (sender, receiver) = mpsc::sync_channel(1);
    let worker = std::thread::Builder::new()
        .name("rss-dns-resolution".to_owned())
        .spawn(move || {
            let addresses = (hostname.as_str(), 443)
                .to_socket_addrs()
                .map(|results| results.take(MAX_RESOLVED_ADDRESSES + 1).collect::<Vec<_>>())
                .map_err(|_| RssFetchError::ResolutionFailed);
            DNS_WORKER_BUSY.store(false, Ordering::Release);
            let _ = sender.send(addresses);
        });
    if worker.is_err() {
        DNS_WORKER_BUSY.store(false, Ordering::Release);
        return Err(RssFetchError::ResolutionFailed);
    }
    let resolved = receiver
        .recv_timeout(timeout)
        .map_err(|_| RssFetchError::RequestTimedOut)??;
    if resolved.is_empty() || resolved.len() > MAX_RESOLVED_ADDRESSES {
        return Err(RssFetchError::ResolutionFailed);
    }
    if resolved
        .iter()
        .any(|address| !is_public_address(address.ip()))
    {
        return Err(RssFetchError::UnsafeAddress);
    }
    Ok(resolved)
}

fn is_public_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(address) => {
            let octets = address.octets();
            let [a, b, c, _] = octets;
            !(a == 0
                || a == 10
                || a == 127
                || a >= 224
                || (a == 100 && (64..=127).contains(&b))
                || (a == 169 && b == 254)
                || (a == 172 && (16..=31).contains(&b))
                || (a == 192 && b == 0)
                || (a == 192 && b == 0 && c == 2)
                || (a == 192 && b == 88 && c == 99)
                || (a == 192 && b == 168)
                || (a == 198 && (b == 18 || b == 19))
                || (a == 198 && b == 51 && c == 100)
                || (a == 203 && b == 0 && c == 113))
        }
        IpAddr::V6(address) => {
            let segments = address.segments();
            (segments[0] & 0xe000) == 0x2000
                && !(segments[0] == 0x2001
                    && (segments[1] == 0
                        || segments[1] == 2
                        || segments[1] == 0x0010
                        || segments[1] == 0x0020
                        || segments[1] == 0x0db8))
        }
    }
}

/// Fetches and normalizes one configured feed after an explicit caller action.
/// The feed must have reviewed terms and be enabled; no automatic schedule exists.
pub fn fetch_rss(connector: &ConnectorDefinition) -> Result<Vec<IntelligenceItem>, RssFetchError> {
    fetch_rss_with_transport(connector, &HttpRssTransport)
}

fn fetch_rss_with_transport(
    connector: &ConnectorDefinition,
    transport: &impl RssTransport,
) -> Result<Vec<IntelligenceItem>, RssFetchError> {
    if !connector.is_enabled()
        || connector.validate().is_err()
        || connector.kind != ConnectorKind::Rss
        || connector.access_method != AccessMethod::PublicRss
        || connector.terms_review != TermsReview::Reviewed
        || connector.health == ConnectorHealth::Blocked
    {
        return Err(if connector.is_enabled() {
            RssFetchError::InvalidConnector
        } else {
            RssFetchError::Disabled
        });
    }
    let limiter = POLL_LIMITER.get_or_init(|| Mutex::new(PollLimiter::default()));
    limiter
        .lock()
        .map_err(|_| RssFetchError::RateLimited)?
        .claim(
            &connector.id,
            Duration::from_secs(connector.minimum_poll_interval_seconds),
            Instant::now(),
        )?;

    let bytes = transport.fetch(connector)?;
    let observed_at = DateTime::<Utc>::from(std::time::SystemTime::now())
        .to_rfc3339_opts(SecondsFormat::Millis, true);
    parse_rss(&bytes, connector, &observed_at)
}

#[derive(Default)]
struct RssEntry {
    title: Option<String>,
    link: Option<String>,
    description: Option<String>,
    guid: Option<String>,
    pub_date: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Title,
    Link,
    Description,
    Guid,
    PubDate,
}

fn item_field(name: &[u8]) -> Option<Field> {
    match name {
        b"title" => Some(Field::Title),
        b"link" => Some(Field::Link),
        b"description" | b"content:encoded" => Some(Field::Description),
        b"guid" => Some(Field::Guid),
        b"pubDate" => Some(Field::PubDate),
        _ => None,
    }
}

fn parse_rss(
    bytes: &[u8],
    connector: &ConnectorDefinition,
    observed_at: &str,
) -> Result<Vec<IntelligenceItem>, RssFetchError> {
    if bytes.is_empty() || bytes.len() as u64 > connector.maximum_response_bytes {
        return Err(RssFetchError::ResponseTooLarge);
    }
    let mut reader = Reader::from_reader(bytes);
    // Preserve whitespace split around XML entities; individual field values are trimmed later.
    reader.config_mut().trim_text(false);
    let mut buffer = Vec::new();
    let mut root_seen = false;
    let mut current_entry: Option<RssEntry> = None;
    let mut current_field: Option<(Field, String)> = None;
    let mut items = Vec::new();
    let mut event_count = 0usize;

    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|_| RssFetchError::MalformedFeed)?;
        event_count += 1;
        if event_count > MAX_XML_EVENTS {
            return Err(RssFetchError::MalformedFeed);
        }
        match event {
            Event::Start(element) => {
                let name = element.name();
                if !root_seen {
                    if name.as_ref() != b"rss" {
                        return Err(RssFetchError::MalformedFeed);
                    }
                    let mut version = None;
                    for attribute in element.attributes() {
                        let attribute = attribute.map_err(|_| RssFetchError::MalformedFeed)?;
                        if attribute.key.as_ref() == b"version" {
                            version = Some(
                                attribute
                                    .unescape_value()
                                    .map_err(|_| RssFetchError::MalformedFeed)?
                                    .into_owned(),
                            );
                        }
                    }
                    if version.as_deref() != Some("2.0") {
                        return Err(RssFetchError::MalformedFeed);
                    }
                    root_seen = true;
                } else if name.as_ref() == b"item" {
                    if current_entry.is_some() {
                        return Err(RssFetchError::MalformedFeed);
                    }
                    current_entry = Some(RssEntry::default());
                } else if current_entry.is_some() {
                    if let Some(field) = item_field(name.as_ref()) {
                        current_field = Some((field, String::new()));
                    }
                }
            }
            Event::Empty(element) => {
                if !root_seen {
                    return Err(RssFetchError::MalformedFeed);
                }
                if element.name().as_ref() == b"item" {
                    return Err(RssFetchError::InvalidItem);
                }
            }
            Event::Text(text) => {
                if let Some((_, value)) = current_field.as_mut() {
                    let decoded = text.decode().map_err(|_| RssFetchError::MalformedFeed)?;
                    let decoded = quick_xml::escape::unescape(&decoded)
                        .map_err(|_| RssFetchError::MalformedFeed)?;
                    append_bounded(value, &decoded)?;
                }
            }
            Event::CData(text) => {
                if let Some((_, value)) = current_field.as_mut() {
                    let decoded = text.decode().map_err(|_| RssFetchError::MalformedFeed)?;
                    append_bounded(value, &decoded)?;
                }
            }
            Event::GeneralRef(reference) => {
                if let Some((_, value)) = current_field.as_mut() {
                    let decoded = if let Some(character) = reference
                        .resolve_char_ref()
                        .map_err(|_| RssFetchError::MalformedFeed)?
                    {
                        character.to_string()
                    } else {
                        match reference
                            .decode()
                            .map_err(|_| RssFetchError::MalformedFeed)?
                            .as_ref()
                        {
                            "amp" => "&".to_owned(),
                            "lt" => "<".to_owned(),
                            "gt" => ">".to_owned(),
                            "apos" => "'".to_owned(),
                            "quot" => "\"".to_owned(),
                            _ => return Err(RssFetchError::MalformedFeed),
                        }
                    };
                    append_bounded(value, &decoded)?;
                }
            }
            Event::End(element) => {
                let name = element.name();
                if name.as_ref() == b"item" {
                    let entry = current_entry.take().ok_or(RssFetchError::MalformedFeed)?;
                    items.push(normalize_entry(entry, connector, observed_at)?);
                    if items.len() > usize::from(connector.maximum_items_per_run) {
                        return Err(RssFetchError::MalformedFeed);
                    }
                } else if let (Some((field, value)), Some(entry)) =
                    (current_field.take(), current_entry.as_mut())
                {
                    if item_field(name.as_ref()) == Some(field) {
                        let value = value.trim().to_owned();
                        match field {
                            Field::Title => entry.title = Some(value),
                            Field::Link => entry.link = Some(value),
                            Field::Description => entry.description = Some(value),
                            Field::Guid => entry.guid = Some(value),
                            Field::PubDate => entry.pub_date = Some(value),
                        }
                    } else {
                        current_field = Some((field, value));
                    }
                }
            }
            Event::DocType(_) => return Err(RssFetchError::MalformedFeed),
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if !root_seen || current_entry.is_some() || items.is_empty() {
        return Err(RssFetchError::MalformedFeed);
    }
    Ok(items)
}

fn append_bounded(target: &mut String, value: &str) -> Result<(), RssFetchError> {
    if target.len().saturating_add(value.len())
        > MAX_ITEM_SUMMARY_CHARS
            .max(MAX_ITEM_TITLE_CHARS)
            .max(MAX_EXTERNAL_ID_CHARS)
            * 4
    {
        return Err(RssFetchError::InvalidItem);
    }
    target.push_str(value);
    Ok(())
}

fn normalize_entry(
    entry: RssEntry,
    connector: &ConnectorDefinition,
    observed_at: &str,
) -> Result<IntelligenceItem, RssFetchError> {
    let title = entry.title.ok_or(RssFetchError::InvalidItem)?;
    let canonical_url = entry.link.ok_or(RssFetchError::InvalidItem)?;
    let published_at = match entry.pub_date.filter(|date| !date.is_empty()) {
        Some(date) => Some(
            DateTime::parse_from_rfc2822(&date)
                .map_err(|_| RssFetchError::InvalidItem)?
                .with_timezone(&Utc)
                .to_rfc3339_opts(SecondsFormat::Millis, true),
        ),
        None => None,
    };
    let mut item = IntelligenceItem {
        id: Uuid::new_v4().to_string(),
        source_id: connector.id.clone(),
        source_url: connector.source_url.clone(),
        external_id: entry.guid.filter(|value| !value.is_empty()),
        canonical_url,
        title,
        summary: entry.description.filter(|value| !value.is_empty()),
        published_at,
        observed_at: observed_at.to_owned(),
        event_time: None,
        category: "general".to_owned(),
        content_hash: None,
        freshness_state: ItemFreshness::Fresh,
        verification_state: ItemVerification::SourceAttributed,
    };
    item.validate().map_err(|_| RssFetchError::InvalidItem)?;
    Ok(item)
}

#[cfg(test)]
mod tests {
    use super::*;
    const NASA_CNEOS_RSS_FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0"><channel><title>Recent CNEOS News Stories</title>
<item><title>Example &amp; fixture</title><link>https://cneos.jpl.nasa.gov/news/example</link>
<guid>cneos-example-1</guid><pubDate>Mon, 26 Oct 2026 04:00:00 +0000</pubDate>
<description><![CDATA[A bounded local test fixture.]]></description></item></channel></rss>"#;

    fn connector(id: &str) -> ConnectorDefinition {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "kind": "rss",
            "displayName": "NASA CNEOS RSS example",
            "sourceUrl": "https://cneos.jpl.nasa.gov/feed/news.xml",
            "accessMethod": "public_rss",
            "termsReview": "reviewed",
            "termsUrl": "https://cneos.jpl.nasa.gov/feed/",
            "termsNotes": "Officially documented public RSS feed; respect published terms and cadence; attribute NASA as source.",
            "minimumPollIntervalSeconds": 900,
            "requestTimeoutMs": 5000,
            "maximumResponseBytes": 512000,
            "maximumItemsPerRun": 100,
            "freshnessThresholdSeconds": 3600,
            "enabled": true,
            "health": "ready",
            "lastAttemptAt": null,
            "lastSuccessAt": null
        }))
        .unwrap()
    }

    struct FixtureTransport(Result<Vec<u8>, RssFetchError>);

    impl RssTransport for FixtureTransport {
        fn fetch(&self, _: &ConnectorDefinition) -> Result<Vec<u8>, RssFetchError> {
            self.0.clone()
        }
    }

    #[test]
    fn parses_rss_fields_and_keeps_source_and_observation_attribution() {
        let connector = connector("fixture.rss.parse");
        let items = parse_rss(
            NASA_CNEOS_RSS_FIXTURE.as_bytes(),
            &connector,
            "2026-10-26T04:35:00Z",
        )
        .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Example & fixture");
        assert_eq!(items[0].source_url, connector.source_url);
        assert_eq!(items[0].external_id.as_deref(), Some("cneos-example-1"));
        assert_eq!(
            items[0].published_at.as_deref(),
            Some("2026-10-26T04:00:00.000Z")
        );
        assert_eq!(items[0].observed_at, "2026-10-26T04:35:00.000Z");
        assert_eq!(items[0].event_time, None);
    }

    #[test]
    fn malformed_or_unsupported_xml_is_a_controlled_error() {
        let connector = connector("fixture.rss.malformed");
        assert_eq!(
            parse_rss(b"<rss><channel><item>", &connector, "2026-10-26T04:35:00Z"),
            Err(RssFetchError::MalformedFeed)
        );
        assert_eq!(
            parse_rss(b"<feed><entry/></feed>", &connector, "2026-10-26T04:35:00Z"),
            Err(RssFetchError::MalformedFeed)
        );
        let doctype = b"<!DOCTYPE rss [<!ENTITY x SYSTEM 'file:///etc/passwd'>]><rss><channel></channel></rss>";
        assert_eq!(
            parse_rss(doctype, &connector, "2026-10-26T04:35:00Z"),
            Err(RssFetchError::MalformedFeed)
        );
    }

    #[test]
    fn invalid_item_urls_and_dates_are_rejected_without_partial_results() {
        let connector = connector("fixture.rss.item-error");
        let invalid = br#"<rss version="2.0"><channel><item><title>Bad URL</title><link>http://example.com/</link></item></channel></rss>"#;
        assert_eq!(
            parse_rss(invalid, &connector, "2026-10-26T04:35:00Z"),
            Err(RssFetchError::InvalidItem)
        );
        let invalid_date = br#"<rss version="2.0"><channel><item><title>Bad date</title><link>https://example.com/</link><pubDate>soon</pubDate></item></channel></rss>"#;
        assert_eq!(
            parse_rss(invalid_date, &connector, "2026-10-26T04:35:00Z"),
            Err(RssFetchError::InvalidItem)
        );
    }

    #[test]
    fn response_size_cap_is_enforced_before_xml_parsing() {
        let mut connector = connector("fixture.rss.size-limit");
        connector.maximum_response_bytes = 8;
        assert_eq!(
            parse_rss(
                NASA_CNEOS_RSS_FIXTURE.as_bytes(),
                &connector,
                "2026-10-26T04:35:00Z"
            ),
            Err(RssFetchError::ResponseTooLarge)
        );
    }

    #[test]
    fn rate_limiter_applies_minimum_interval_and_bounded_connector_count() {
        let start = Instant::now();
        let mut limiter = PollLimiter::default();
        assert_eq!(
            limiter.claim("source", Duration::from_secs(60), start),
            Ok(())
        );
        assert_eq!(
            limiter.claim(
                "source",
                Duration::from_secs(60),
                start + Duration::from_secs(59)
            ),
            Err(RssFetchError::RateLimited)
        );
        assert_eq!(
            limiter.claim(
                "source",
                Duration::from_secs(60),
                start + Duration::from_secs(60)
            ),
            Ok(())
        );
        for index in 0..MAX_TRACKED_CONNECTORS {
            limiter.last_attempt.insert(format!("id-{index}"), start);
        }
        assert_eq!(
            limiter.claim("another", Duration::from_secs(60), start),
            Err(RssFetchError::RateLimited)
        );
    }

    #[test]
    fn timeout_and_rate_limit_failures_are_returned_as_stable_errors() {
        let timeout_connector = connector("fixture.rss.timeout");
        assert_eq!(
            fetch_rss_with_transport(
                &timeout_connector,
                &FixtureTransport(Err(RssFetchError::RequestTimedOut))
            ),
            Err(RssFetchError::RequestTimedOut)
        );
        assert_eq!(
            fetch_rss_with_transport(
                &timeout_connector,
                &FixtureTransport(Ok(NASA_CNEOS_RSS_FIXTURE.as_bytes().to_vec()))
            ),
            Err(RssFetchError::RateLimited)
        );
    }

    #[test]
    fn fetch_rejects_disabled_sources_before_transport() {
        let mut disabled = connector("fixture.rss.disabled");
        let mut disabled_json = serde_json::to_value(disabled).unwrap();
        disabled_json["enabled"] = serde_json::json!(false);
        disabled_json["health"] = serde_json::json!("disabled");
        disabled = serde_json::from_value(disabled_json).unwrap();
        assert_eq!(
            fetch_rss_with_transport(
                &disabled,
                &FixtureTransport(Err(RssFetchError::NetworkFailure))
            ),
            Err(RssFetchError::Disabled)
        );
    }

    #[test]
    fn network_targets_must_resolve_only_to_public_addresses() {
        for address in [
            "127.0.0.1",
            "10.0.0.1",
            "169.254.1.1",
            "192.168.1.1",
            "::1",
            "fc00::1",
            "fe80::1",
            "2001:db8::1",
        ] {
            assert!(!is_public_address(address.parse().unwrap()), "{address}");
        }
        assert!(is_public_address("8.8.8.8".parse().unwrap()));
        assert!(is_public_address("2001:4860:4860::8888".parse().unwrap()));
    }
}
