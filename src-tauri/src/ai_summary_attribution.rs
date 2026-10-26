//! Strict evidence attribution for AI-generated factual summary claims.

use crate::intelligence_items::{IntelligenceItem, ItemValidationError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const AI_SUMMARY_SCHEMA_VERSION: u32 = 1;
pub const MAX_AI_SUMMARY_BYTES: usize = 64 * 1024;
const MAX_EVIDENCE_ITEMS: usize = 100;
const MAX_CLAIMS: usize = 64;
const MAX_CITATIONS_PER_CLAIM: usize = 8;
const MAX_CLAIM_CHARS: usize = 2_000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimSupport {
    Supported,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AiSummaryDraft {
    schema_version: u32,
    claims: Vec<AiSummaryClaimDraft>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AiSummaryClaimDraft {
    text: String,
    support: ClaimSupport,
    evidence_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiSummaryEvidence {
    pub evidence_id: String,
    pub source_id: String,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttributedClaim {
    pub text: String,
    pub support: ClaimSupport,
    pub evidence: Vec<AiSummaryEvidence>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttributedSummary {
    pub schema_version: u32,
    pub claims: Vec<AttributedClaim>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiSummaryAttributionError {
    TooLarge,
    InvalidJson,
    UnsupportedSchemaVersion,
    InvalidEvidence,
    InvalidClaim,
    UnknownEvidence,
}

pub fn attribute_summary(
    response_json: &str,
    evidence_items: &[IntelligenceItem],
) -> Result<AttributedSummary, AiSummaryAttributionError> {
    if response_json.len() > MAX_AI_SUMMARY_BYTES {
        return Err(AiSummaryAttributionError::TooLarge);
    }
    let draft: AiSummaryDraft =
        serde_json::from_str(response_json).map_err(|_| AiSummaryAttributionError::InvalidJson)?;
    if draft.schema_version != AI_SUMMARY_SCHEMA_VERSION {
        return Err(AiSummaryAttributionError::UnsupportedSchemaVersion);
    }
    if evidence_items.is_empty() || evidence_items.len() > MAX_EVIDENCE_ITEMS {
        return Err(AiSummaryAttributionError::InvalidEvidence);
    }
    let mut evidence = BTreeMap::new();
    for item in evidence_items {
        let mut validated = item.clone();
        validated
            .validate()
            .map_err(|_: ItemValidationError| AiSummaryAttributionError::InvalidEvidence)?;
        if evidence
            .insert(
                validated.id.clone(),
                AiSummaryEvidence {
                    evidence_id: validated.id,
                    source_id: validated.source_id,
                    url: validated.original_url,
                },
            )
            .is_some()
        {
            return Err(AiSummaryAttributionError::InvalidEvidence);
        }
    }
    if draft.claims.is_empty() || draft.claims.len() > MAX_CLAIMS {
        return Err(AiSummaryAttributionError::InvalidClaim);
    }
    let mut claims = Vec::with_capacity(draft.claims.len());
    for claim in draft.claims {
        if claim.text.trim().is_empty()
            || claim.text.chars().count() > MAX_CLAIM_CHARS
            || claim.text.chars().any(char::is_control)
            || claim.evidence_ids.len() > MAX_CITATIONS_PER_CLAIM
        {
            return Err(AiSummaryAttributionError::InvalidClaim);
        }
        let mut references = Vec::with_capacity(claim.evidence_ids.len());
        let mut seen = BTreeSet::new();
        for evidence_id in claim.evidence_ids {
            if !seen.insert(evidence_id.clone()) {
                return Err(AiSummaryAttributionError::InvalidClaim);
            }
            references.push(
                evidence
                    .get(&evidence_id)
                    .cloned()
                    .ok_or(AiSummaryAttributionError::UnknownEvidence)?,
            );
        }
        match claim.support {
            ClaimSupport::Supported if references.is_empty() => {
                return Err(AiSummaryAttributionError::InvalidClaim);
            }
            ClaimSupport::Unknown if !references.is_empty() => {
                return Err(AiSummaryAttributionError::InvalidClaim);
            }
            _ => {}
        }
        claims.push(AttributedClaim {
            text: claim.text,
            support: claim.support,
            evidence: references,
        });
    }
    Ok(AttributedSummary {
        schema_version: AI_SUMMARY_SCHEMA_VERSION,
        claims,
    })
}

#[cfg(test)]
mod tests {
    use super::{attribute_summary, AiSummaryAttributionError, ClaimSupport};
    use crate::intelligence_items::{IntelligenceItem, ItemFreshness, ItemVerification};
    use uuid::Uuid;

    fn evidence() -> IntelligenceItem {
        let url = "https://example.com/news/item-1".to_owned();
        IntelligenceItem {
            id: Uuid::from_u128(42).to_string(),
            source_id: "fixture-source".to_owned(),
            source_url: "https://example.com/feed.xml".to_owned(),
            external_id: Some("item-1".to_owned()),
            original_url: url.clone(),
            canonical_url: url,
            title: "A source title".to_owned(),
            summary: Some("A source summary".to_owned()),
            published_at: None,
            observed_at: "2026-10-26T12:00:00Z".to_owned(),
            event_time: None,
            category: "news".to_owned(),
            content_hash: None,
            freshness_state: ItemFreshness::Fresh,
            verification_state: ItemVerification::SourceAttributed,
        }
    }

    #[test]
    fn supported_claims_receive_only_application_resolved_evidence_ids_and_urls() {
        let id = Uuid::from_u128(42);
        let input = format!(
            r#"{{"schemaVersion":1,"claims":[{{"text":"The report states a fact.","support":"supported","evidenceIds":["{id}"]}}]}}"#
        );
        let summary = attribute_summary(&input, &[evidence()]).unwrap();
        assert_eq!(summary.claims[0].support, ClaimSupport::Supported);
        assert_eq!(summary.claims[0].evidence[0].evidence_id, id.to_string());
        assert_eq!(
            summary.claims[0].evidence[0].url,
            "https://example.com/news/item-1"
        );
    }

    #[test]
    fn unsupported_claims_are_explicitly_unknown_and_cannot_carry_evidence() {
        let id = Uuid::from_u128(42);
        let cited_unknown = format!(
            r#"{{"schemaVersion":1,"claims":[{{"text":"Unknown claim","support":"unknown","evidenceIds":["{id}"]}}]}}"#
        );
        let summary = attribute_summary(
            r#"{"schemaVersion":1,"claims":[{"text":"The cause is unclear.","support":"unknown","evidenceIds":[]}]}"#,
            &[evidence()],
        )
        .unwrap();
        assert_eq!(summary.claims[0].support, ClaimSupport::Unknown);
        assert!(summary.claims[0].evidence.is_empty());
        assert_eq!(
            attribute_summary(&cited_unknown, &[evidence()]),
            Err(AiSummaryAttributionError::InvalidClaim)
        );
    }

    #[test]
    fn invented_evidence_ids_and_uncited_supported_claims_fail_closed() {
        assert_eq!(
            attribute_summary(
                r#"{"schemaVersion":1,"claims":[{"text":"A claim","support":"supported","evidenceIds":["00000000-0000-4000-8000-000000000099"]}]}"#,
                &[evidence()]
            ),
            Err(AiSummaryAttributionError::UnknownEvidence)
        );
        assert_eq!(
            attribute_summary(
                r#"{"schemaVersion":1,"claims":[{"text":"A claim","support":"supported","evidenceIds":[]}]}"#,
                &[evidence()]
            ),
            Err(AiSummaryAttributionError::InvalidClaim)
        );
    }
}
