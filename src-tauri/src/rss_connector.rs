//! A single-request, user-configured RSS adapter. It does not schedule or persist feeds.
use crate::connectors::{
    AccessMethod, ConnectorDefinition, ConnectorHealth, ConnectorKind, TermsReview,
};
use crate::intelligence_identity::{canonicalize_item_url, content_fingerprint};
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
const BACKOFF_BASE_SECONDS: u64 = 60;
const MAX_BACKOFF_SECONDS: u64 = 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RssFetchError {
    Disabled,
    InvalidConnector,
    RateLimited,
    BackoffActive,
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

#[derive(Clone, Copy)]
struct PollState {
    last_attempt: Instant,
    consecutive_failures: u8,
    retry_after: Option<Instant>,
}

#[derive(Default)]
struct PollLimiter {
    states: HashMap<String, PollState>,
}

impl PollLimiter {
    fn claim(&mut self, id: &str, interval: Duration, now: Instant) -> Result<(), RssFetchError> {
        if let Some(state) = self.states.get_mut(id) {
            if now.saturating_duration_since(state.last_attempt) < interval {
                return Err(RssFetchError::RateLimited);
            }
            if state
                .retry_after
                .is_some_and(|retry_after| now < retry_after)
            {
                return Err(RssFetchError::BackoffActive);
            }
            state.last_attempt = now;
        } else if self.states.len() >= MAX_TRACKED_CONNECTORS {
            return Err(RssFetchError::RateLimited);
        } else {
            self.states.insert(
                id.to_owned(),
                PollState {
                    last_attempt: now,
                    consecutive_failures: 0,
                    retry_after: None,
                },
            );
        }
        Ok(())
    }

    fn record_failure(&mut self, id: &str, interval: Duration, now: Instant) {
        if let Some(state) = self.states.get_mut(id) {
            state.consecutive_failures = state.consecutive_failures.saturating_add(1);
            let exponent = u32::from(state.consecutive_failures.saturating_sub(1)).min(6);
            let exponential = BACKOFF_BASE_SECONDS
                .saturating_mul(2u64.saturating_pow(exponent))
                .min(MAX_BACKOFF_SECONDS);
            let delay = interval.max(Duration::from_secs(exponential));
            state.retry_after = now.checked_add(delay);
        }
    }

    fn record_success(&mut self, id: &str) {
        if let Some(state) = self.states.get_mut(id) {
            state.consecutive_failures = 0;
            state.retry_after = None;
        }
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
        let host = url
            .host_str()
            .ok_or(RssFetchError::InvalidConnector)?
            .to_owned();
        let timeout = Duration::from_millis(u64::from(connector.request_timeout_ms));
        let started_at = Instant::now();
        let addresses = resolve_public_addresses(&host, timeout)?;
        let remaining_timeout = timeout.saturating_sub(started_at.elapsed());
        if remaining_timeout.is_zero() {
            return Err(RssFetchError::RequestTimedOut);
        }
        let client = Client::builder()
            .https_only(true)
            .no_proxy()
            .redirect(Policy::none())
            .timeout(remaining_timeout)
            .resolve_to_addrs(&host, &addresses)
            .build()
            .map_err(|_| RssFetchError::NetworkFailure)?;
        let response = client
            .get(url)
            .header(
                reqwest::header::ACCEPT,
                "application/atom+xml, application/rss+xml, application/xml, text/xml;q=0.9",
            )
            .header(
                reqwest::header::USER_AGENT,
                "Moonships Workbench/0.1.0 (https://github.com/bjo163/moonships)",
            )
            .send()
            .map_err(map_request_error)?;
        if !response.status().is_success() {
            return Err(map_http_status(&host, response.status()));
        }
        let limit = connector.maximum_response_bytes;
        if response
            .content_length()
            .is_some_and(|length| length > limit)
        {
            return Err(RssFetchError::ResponseTooLarge);
        }
        read_bounded(response, limit)
    }
}

fn map_http_status(host: &str, status: reqwest::StatusCode) -> RssFetchError {
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS
        || (host.eq_ignore_ascii_case("api.weather.gov")
            && status == reqwest::StatusCode::FORBIDDEN)
    {
        RssFetchError::RateLimited
    } else {
        RssFetchError::HttpStatus
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

fn read_bounded(reader: impl Read, limit: u64) -> Result<Vec<u8>, RssFetchError> {
    let mut body = Vec::with_capacity(limit.min(64 * 1024) as usize);
    reader
        .take(limit.saturating_add(1))
        .read_to_end(&mut body)
        .map_err(map_io_error)?;
    if body.len() as u64 > limit {
        return Err(RssFetchError::ResponseTooLarge);
    }
    Ok(body)
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
pub fn fetch_rss(
    connector: &mut ConnectorDefinition,
) -> Result<Vec<IntelligenceItem>, RssFetchError> {
    fetch_rss_with_transport(connector, &HttpRssTransport)
}

fn fetch_rss_with_transport(
    connector: &mut ConnectorDefinition,
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
    let interval = Duration::from_secs(connector.minimum_poll_interval_seconds);
    if let Err(error) = limiter
        .lock()
        .map_err(|_| RssFetchError::RateLimited)?
        .claim(&connector.id, interval, Instant::now())
    {
        if matches!(
            error,
            RssFetchError::RateLimited | RssFetchError::BackoffActive
        ) {
            connector.health = ConnectorHealth::RateLimited;
        }
        return Err(error);
    }

    let attempted_at = timestamp_now();
    connector.last_attempt_at = Some(attempted_at);
    let result = transport.fetch(connector).and_then(|bytes| {
        let observed_at = timestamp_now();
        parse_rss(&bytes, connector, &observed_at)
    });
    let mut state = limiter.lock().map_err(|_| RssFetchError::BackoffActive)?;
    match result {
        Ok(items) => {
            state.record_success(&connector.id);
            connector.health = ConnectorHealth::Healthy;
            connector.last_success_at = Some(timestamp_now());
            Ok(items)
        }
        Err(error) => {
            state.record_failure(&connector.id, interval, Instant::now());
            connector.health = match error {
                RssFetchError::RequestTimedOut
                | RssFetchError::NetworkFailure
                | RssFetchError::ResolutionFailed
                | RssFetchError::ResolverBusy => ConnectorHealth::Unavailable,
                RssFetchError::UnsafeAddress | RssFetchError::InvalidConnector => {
                    ConnectorHealth::Blocked
                }
                RssFetchError::RateLimited | RssFetchError::BackoffActive => {
                    ConnectorHealth::RateLimited
                }
                RssFetchError::Disabled => ConnectorHealth::Disabled,
                RssFetchError::HttpStatus
                | RssFetchError::ResponseTooLarge
                | RssFetchError::MalformedFeed
                | RssFetchError::InvalidItem => ConnectorHealth::Error,
            };
            Err(error)
        }
    }
}

fn timestamp_now() -> String {
    DateTime::<Utc>::from(std::time::SystemTime::now()).to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[derive(Default)]
struct RssEntry {
    title: Option<String>,
    link: Option<String>,
    description: Option<String>,
    guid: Option<String>,
    pub_date: Option<String>,
    updated_date: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RootFormat {
    Rss,
    Atom,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Title,
    Link,
    Description,
    Guid,
    PubDate,
    UpdatedDate,
}

fn item_field(name: &[u8]) -> Option<Field> {
    match name {
        b"title" => Some(Field::Title),
        b"link" => Some(Field::Link),
        b"description" | b"content:encoded" | b"summary" => Some(Field::Description),
        b"guid" | b"id" => Some(Field::Guid),
        b"pubDate" | b"published" => Some(Field::PubDate),
        b"updated" => Some(Field::UpdatedDate),
        _ => None,
    }
}

fn atom_link_href(
    element: &quick_xml::events::BytesStart<'_>,
) -> Result<Option<String>, RssFetchError> {
    let mut href = None;
    let mut relation = None;
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|_| RssFetchError::MalformedFeed)?;
        let value = attribute
            .unescape_value()
            .map_err(|_| RssFetchError::MalformedFeed)?
            .into_owned();
        match attribute.key.as_ref() {
            b"href" => href = Some(value),
            b"rel" => relation = Some(value),
            _ => {}
        }
    }
    if relation.as_deref().is_none_or(|value| value == "alternate") {
        Ok(href)
    } else {
        Ok(None)
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
    let mut root_format = None;
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
                if root_format.is_none() {
                    match name.as_ref() {
                        b"rss" => {
                            let mut version = None;
                            for attribute in element.attributes() {
                                let attribute =
                                    attribute.map_err(|_| RssFetchError::MalformedFeed)?;
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
                            root_format = Some(RootFormat::Rss);
                        }
                        b"feed" => root_format = Some(RootFormat::Atom),
                        _ => return Err(RssFetchError::MalformedFeed),
                    }
                } else if (root_format == Some(RootFormat::Rss) && name.as_ref() == b"item")
                    || (root_format == Some(RootFormat::Atom) && name.as_ref() == b"entry")
                {
                    if current_entry.is_some() {
                        return Err(RssFetchError::MalformedFeed);
                    }
                    current_entry = Some(RssEntry::default());
                } else if current_entry.is_some() {
                    if root_format == Some(RootFormat::Atom) && name.as_ref() == b"link" {
                        if let Some(href) = atom_link_href(&element)? {
                            if let Some(entry) = current_entry.as_mut() {
                                entry.link = Some(href);
                            }
                        }
                    } else if let Some(field) = item_field(name.as_ref()) {
                        current_field = Some((field, String::new()));
                    }
                }
            }
            Event::Empty(element) => {
                if root_format.is_none() {
                    return Err(RssFetchError::MalformedFeed);
                }
                if element.name().as_ref() == b"item" {
                    return Err(RssFetchError::InvalidItem);
                }
                if element.name().as_ref() == b"entry" {
                    return Err(RssFetchError::MalformedFeed);
                }
                if root_format == Some(RootFormat::Atom) && element.name().as_ref() == b"link" {
                    if let (Some(href), Some(entry)) =
                        (atom_link_href(&element)?, current_entry.as_mut())
                    {
                        entry.link = Some(href);
                    }
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
                if (root_format == Some(RootFormat::Rss) && name.as_ref() == b"item")
                    || (root_format == Some(RootFormat::Atom) && name.as_ref() == b"entry")
                {
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
                            Field::UpdatedDate => entry.updated_date = Some(value),
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
    if root_format.is_none() || current_entry.is_some() {
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
    let url_identity = canonicalize_item_url(&entry.link.ok_or(RssFetchError::InvalidItem)?)
        .map_err(|_| RssFetchError::InvalidItem)?;
    let published_at = match entry
        .pub_date
        .or(entry.updated_date)
        .filter(|date| !date.is_empty())
    {
        Some(date) => Some(
            DateTime::parse_from_rfc2822(&date)
                .or_else(|_| DateTime::parse_from_rfc3339(&date))
                .map_err(|_| RssFetchError::InvalidItem)?
                .with_timezone(&Utc)
                .to_rfc3339_opts(SecondsFormat::Millis, true),
        ),
        None => None,
    };
    let summary = entry.description.filter(|value| !value.is_empty());
    let content_hash =
        content_fingerprint(&title, summary.as_deref()).map_err(|_| RssFetchError::InvalidItem)?;
    let external_id = entry.guid.filter(|value| !value.is_empty());
    let stable_identity = external_id
        .as_deref()
        .unwrap_or(url_identity.canonical.as_str());
    let id_name = format!(
        "{}:{}:{stable_identity}",
        connector.id, connector.source_url
    );
    let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, id_name.as_bytes()).to_string();
    let category = reqwest::Url::parse(&connector.source_url)
        .ok()
        .filter(|url| {
            url.host_str() == Some("api.weather.gov") && url.path().starts_with("/alerts/")
        })
        .map_or("general", |_| "weather_alert");
    let mut item = IntelligenceItem {
        id,
        source_id: connector.id.clone(),
        source_url: connector.source_url.clone(),
        external_id,
        original_url: url_identity.original,
        canonical_url: url_identity.canonical,
        title,
        summary,
        published_at,
        observed_at: observed_at.to_owned(),
        event_time: None,
        category: category.to_owned(),
        content_hash: Some(content_hash),
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
<item><title>Example &amp; fixture</title><link>https://CNEOS.JPL.NASA.GOV/news/./example?utm_source=test&amp;id=4#section</link>
<guid>cneos-example-1</guid><pubDate>Mon, 26 Oct 2026 04:00:00 +0000</pubDate>
<description><![CDATA[A bounded local test fixture.]]></description></item></channel></rss>"#;
    const NWS_ATOM_FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom"><title>Active Alerts for California</title>
<entry><id>urn:oid:2.49.0.1.840.0.fixture-alert</id><title>Flood Warning</title>
<link rel="alternate" href="https://api.weather.gov/alerts/urn%3Aoid%3Afixture-alert"/>
<updated>2026-10-26T04:00:00Z</updated><summary>A bounded weather alert fixture.</summary></entry></feed>"#;

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
        assert_eq!(
            items[0].original_url,
            "https://CNEOS.JPL.NASA.GOV/news/./example?utm_source=test&id=4#section"
        );
        assert_eq!(
            items[0].canonical_url,
            "https://cneos.jpl.nasa.gov/news/example?id=4"
        );
        assert!(items[0].content_hash.is_some());
        assert_eq!(items[0].external_id.as_deref(), Some("cneos-example-1"));
        assert_eq!(
            items[0].published_at.as_deref(),
            Some("2026-10-26T04:00:00.000Z")
        );
        assert_eq!(items[0].observed_at, "2026-10-26T04:35:00.000Z");
        assert_eq!(items[0].event_time, None);
    }

    #[test]
    fn parses_nws_atom_alerts_with_stable_ids_and_source_category() {
        let mut connector = connector("fixture.nws.atom");
        connector.display_name = "NWS Active Alerts (CA)".to_owned();
        connector.source_url = "https://api.weather.gov/alerts/active.atom?area=CA".to_owned();
        let first = parse_rss(
            NWS_ATOM_FIXTURE.as_bytes(),
            &connector,
            "2026-10-26T04:35:00Z",
        )
        .unwrap();
        let second = parse_rss(
            NWS_ATOM_FIXTURE.as_bytes(),
            &connector,
            "2026-10-26T04:36:00Z",
        )
        .unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].title, "Flood Warning");
        assert_eq!(first[0].category, "weather_alert");
        assert_eq!(
            first[0].external_id.as_deref(),
            Some("urn:oid:2.49.0.1.840.0.fixture-alert")
        );
        assert_eq!(
            first[0].original_url,
            "https://api.weather.gov/alerts/urn%3Aoid%3Afixture-alert"
        );
        assert_eq!(
            first[0].published_at.as_deref(),
            Some("2026-10-26T04:00:00.000Z")
        );
        assert_eq!(first[0].id, second[0].id);
        assert_ne!(first[0].observed_at, second[0].observed_at);
    }

    #[test]
    fn valid_empty_atom_feed_is_a_successful_empty_result() {
        let mut connector = connector("fixture.nws.empty");
        connector.source_url = "https://api.weather.gov/alerts/active.atom?area=CA".to_owned();
        assert_eq!(
            parse_rss(
                br#"<feed xmlns="http://www.w3.org/2005/Atom"><title>No active alerts</title></feed>"#,
                &connector,
                "2026-10-26T04:35:00Z",
            ),
            Ok(Vec::new())
        );
    }

    #[test]
    fn provider_rate_limit_statuses_map_to_backoff_without_hiding_other_http_errors() {
        assert_eq!(
            map_http_status("api.weather.gov", reqwest::StatusCode::TOO_MANY_REQUESTS),
            RssFetchError::RateLimited
        );
        assert_eq!(
            map_http_status("api.weather.gov", reqwest::StatusCode::FORBIDDEN),
            RssFetchError::RateLimited
        );
        assert_eq!(
            map_http_status("api.weather.gov", reqwest::StatusCode::SERVICE_UNAVAILABLE),
            RssFetchError::HttpStatus
        );
        assert_eq!(
            map_http_status("example.test", reqwest::StatusCode::FORBIDDEN),
            RssFetchError::HttpStatus
        );
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
    fn bounded_http_body_reader_maps_oversize_and_slow_reads() {
        use std::io::{Cursor, Read};

        assert_eq!(
            read_bounded(Cursor::new(vec![b'x'; 9]), 8),
            Err(RssFetchError::ResponseTooLarge)
        );

        struct TimedOutReader;
        impl Read for TimedOutReader {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "fixture timeout",
                ))
            }
        }
        assert_eq!(
            read_bounded(TimedOutReader, 8),
            Err(RssFetchError::RequestTimedOut)
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
            limiter.states.insert(
                format!("id-{index}"),
                PollState {
                    last_attempt: start,
                    consecutive_failures: 0,
                    retry_after: None,
                },
            );
        }
        assert_eq!(
            limiter.claim("another", Duration::from_secs(60), start),
            Err(RssFetchError::RateLimited)
        );
    }

    #[test]
    fn repeated_failures_apply_capped_exponential_backoff() {
        let start = Instant::now();
        let mut limiter = PollLimiter::default();
        let interval = Duration::from_secs(60);
        limiter.claim("backoff", interval, start).unwrap();
        let mut now = start;
        for _ in 0..10 {
            limiter.record_failure("backoff", interval, now);
            now = limiter.states["backoff"].retry_after.unwrap();
            limiter.claim("backoff", interval, now).unwrap();
        }
        limiter.record_failure("backoff", interval, now);
        assert_eq!(
            limiter.states["backoff"]
                .retry_after
                .unwrap()
                .duration_since(now),
            Duration::from_secs(MAX_BACKOFF_SECONDS)
        );
        limiter.record_success("backoff");
        assert_eq!(limiter.states["backoff"].consecutive_failures, 0);

        let mut limiter = PollLimiter::default();
        limiter.claim("backoff", interval, start).unwrap();
        limiter.record_failure("backoff", interval, start);
        assert_eq!(
            limiter.claim("backoff", interval, start + Duration::from_secs(59)),
            Err(RssFetchError::RateLimited)
        );
        limiter
            .claim("backoff", interval, start + Duration::from_secs(60))
            .unwrap();
        limiter.record_failure("backoff", interval, start + Duration::from_secs(60));
        assert_eq!(
            limiter.claim("backoff", interval, start + Duration::from_secs(120)),
            Err(RssFetchError::BackoffActive)
        );
        limiter
            .claim("backoff", interval, start + Duration::from_secs(180))
            .unwrap();
        limiter.record_success("backoff");
        limiter
            .claim("backoff", interval, start + Duration::from_secs(240))
            .unwrap();
        assert_eq!(limiter.states["backoff"].consecutive_failures, 0);
    }

    #[test]
    fn timeout_and_rate_limit_failures_are_returned_as_stable_errors() {
        let mut timeout_connector = connector("fixture.rss.timeout");
        timeout_connector.last_success_at = Some("2026-10-26T04:00:00Z".to_owned());
        assert_eq!(
            fetch_rss_with_transport(
                &mut timeout_connector,
                &FixtureTransport(Err(RssFetchError::RequestTimedOut))
            ),
            Err(RssFetchError::RequestTimedOut)
        );
        assert_eq!(timeout_connector.health, ConnectorHealth::Unavailable);
        assert!(timeout_connector.last_attempt_at.is_some());
        assert_eq!(
            timeout_connector.last_success_at.as_deref(),
            Some("2026-10-26T04:00:00Z")
        );
        assert_eq!(
            fetch_rss_with_transport(
                &mut timeout_connector,
                &FixtureTransport(Ok(NASA_CNEOS_RSS_FIXTURE.as_bytes().to_vec()))
            ),
            Err(RssFetchError::RateLimited)
        );
        assert_eq!(timeout_connector.health, ConnectorHealth::RateLimited);
    }

    #[test]
    fn successful_fetch_updates_the_in_memory_last_success_state() {
        let mut connector = connector("fixture.rss.freshness-success");
        let items = fetch_rss_with_transport(
            &mut connector,
            &FixtureTransport(Ok(NASA_CNEOS_RSS_FIXTURE.as_bytes().to_vec())),
        )
        .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(connector.health, ConnectorHealth::Healthy);
        assert!(connector.last_attempt_at.is_some());
        let last_success_at = connector.last_success_at.clone().unwrap();
        let now = DateTime::parse_from_rfc3339(&last_success_at)
            .unwrap()
            .with_timezone(&Utc);
        let view = crate::connector_freshness::ConnectorFreshnessView::evaluate(&connector, &now);
        assert_eq!(
            view.status,
            crate::connector_freshness::ConnectorFreshness::Fresh
        );
        assert!(view.is_live);
        assert_eq!(
            view.last_success_at.as_deref(),
            Some(last_success_at.as_str())
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
                &mut disabled,
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
