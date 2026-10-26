# Integration source and license review

Complete this review for every PR that adds, changes or removes an external adapter, integration, provider, source feed, imported third-party code, dependency, data set, or visual/audio asset. The author records evidence; a reviewer checks the evidence and records an outcome. A declaration that terms were reviewed in product settings is not a substitute for this engineering review.

## Review record

- Task / PR:
- Adapter, integration, package, source or asset (include exact version/commit where applicable):
- Reviewer and review date (UTC):
- Review outcome: `approved` / `blocked` / `not applicable` (choose exactly one):
- Evidence links or repository paths:

`not applicable` requires a short reason. `blocked` means the adapter or affected content is not ready to merge/release; identify the missing evidence or required change below. Do not mark a blocked change approved by leaving the field blank.

## Code, package and asset provenance

- [ ] Origin is identified and pinned to a release, commit or immutable artifact when possible; no copied code has unknown provenance.
- [ ] License and applicable notices/attribution are verified against the upstream source and recorded in `docs/THIRD_PARTY_NOTICES.md` when required.
- [ ] License compatibility and redistribution obligations for source and binary distributions are considered; no legal conclusion is inferred from a package registry label alone.
- [ ] Names, logos, screenshots, fonts, sound and other assets have documented rights and are not presented as affiliation or endorsement.
- [ ] Dependency version, transitive dependency/license exposure and supported update path are recorded where applicable.

## Data access and provider terms

- [ ] Official documentation and current terms/privacy policy are linked, with review date and applicable account/region or API plan noted where material.
- [ ] Intended access method, permitted purpose, authentication requirements, attribution, rate limits, caching/retention and redistribution are documented.
- [ ] The design does not bypass authentication, paywalls, access controls, robots or provider rate limits; restrictions and uncertainty are recorded.
- [ ] Data fields collected, user-visible source attribution, sensitive data handling, deletion/retention and any onward sharing are described.
- [ ] Failure, revocation, terms change and source disable behavior are addressed; no unapproved polling or background access is introduced.

## Decision

- [ ] The review result and evidence above are complete and the outcome is approved.
- [ ] Any required notice, documentation, attribution or restriction is included in this change.
- [ ] If blocked or not applicable, the rationale and follow-up are recorded; a blocked adapter must remain unavailable.

An approved review records the evidence and reviewer decision at that time. It is not legal advice or a permanent grant of rights; re-review when the upstream terms, access method, dependency version, data use or distribution changes.
