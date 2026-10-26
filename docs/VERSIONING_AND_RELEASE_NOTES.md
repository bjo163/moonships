# Versioning and Release Notes

## Version policy

Moonships uses `MAJOR.MINOR.PATCH` for stable releases. Before 1.0, the major number stays `0`: increment MINOR for a user-visible feature set and PATCH for a compatible correction. A release candidate uses a human-facing tag such as `v0.1.0-alpha.3`; prerelease identifiers are ordered and incremented for each published candidate. Use `beta.N` and `rc.N` only when the release stage changes. A stable release drops the prerelease suffix.

Windows package tooling requires numeric prerelease identifiers for the embedded application version. Map a human-facing tag such as `v0.1.0-alpha.3` to embedded version `0.1.0-3`. The embedded numeric counter increases across alpha, beta and release candidates for the same stable version, so a later stage never sorts below an earlier candidate. Keep `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` aligned on the embedded version. The tag is the public release identity; the embedded version is the package identity. Do not infer that a version has been released just because it appears in documentation or a local build.

Every published release must be built from an immutable, reviewed Git tag pointing to the exact commit that passed the release gates. Release notes must describe only behavior verified on that commit. Distinguish implemented behavior from planned, unavailable, unsupported, and unverified behavior. Never claim a test, installer, signature, migration, security review, or support target passed without recorded evidence.

## Release notes format

Create one Markdown file per release, named `RELEASE_NOTES_<tag>.md`, and use these sections in this order:

1. **Release identity** — public tag, embedded version, commit, publication date, and release stage.
2. **Highlights** — the main user-visible changes, if any.
3. **Changes and fixes** — concise, user-facing descriptions; omit internal-only changes unless they affect users.
4. **Data and migration notes** — records changed, compatibility behavior, backup/recovery implications, and whether migration is automatic. State “None” when there is no data change.
5. **Known limitations** — unavailable or unverified features and relevant workarounds.
6. **Security and privacy** — network, credential, permission, and data-retention changes; state “None” only after reviewing the release diff.
7. **Verification and compatibility** — exact CI run or local checks, actual OS/package formats exercised, and limitations of that evidence.
8. **Downloads and integrity** — artifact names and SHA-256 values, added only after the release workflow produces and verifies them.

Use plain language and avoid promises about future dates. Do not include secrets, user data, unpublished security details, or claims the release tests do not establish. Template and reviewed sample are in this document below; copy the structure when preparing an actual release and remove all example-only content.

## Sample release notes — documentation example only

> This is a review sample, not a published release announcement. The example tag, commit, date, CI run, installers, and digests are deliberately omitted; fill them only from a real release workflow result.

### Release identity

- Public tag: `v0.1.0-alpha.3` (example)
- Embedded version: `0.1.0-3` (example)
- Commit: not assigned
- Publication date: not published
- Stage: alpha preview

### Highlights

- Manage local Projects and Tasks in the desktop workspace.
- Control the optional AI preference from Settings. AI is off by default.

### Changes and fixes

- AI settings now report that this build has no connected production provider. Enabling the preference does not send prompts or data to another provider.
- Existing version-1 app preferences migrate to version 2 with AI kept off.

### Data and migration notes

- The local app configuration record advances from schema version 1 to 2. Existing theme, profile, log-level and tray preferences are retained; the new AI preference is initialized as disabled. The migration runs locally and does not change Project or Task records.

### Known limitations

- No production AI provider is connected. AI-assisted generation is unavailable even when its preference is enabled.
- Direct, task-based usability observation of the Projects and Tasks workflows has not been completed.
- This sample does not establish stable-release readiness, supported Windows versions, installer availability, or successful installation and upgrade behavior.

### Security and privacy

- No AI prompt or workspace data is sent by the AI preference control. No alternate provider fallback is configured.
- No other security or privacy change is asserted by this sample; an actual release requires review of its exact diff.

### Verification and compatibility

- Example checks for the documented M7-015 change: Rust library tests, Rust formatting and Clippy, frontend tests, typecheck, and production build.
- This sample does not record a release CI run or manual Windows installation result. Add actual run links and tested OS/package details before publication.

### Downloads and integrity

- Not published. Add each workflow-produced installer filename and its verified SHA-256 digest after a real release build.
