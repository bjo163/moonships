# Desktop Release Guide

Version numbering and the release-notes structure are defined in [Versioning and Release Notes](VERSIONING_AND_RELEASE_NOTES.md). Use its sample for documentation review only; it is not a published release.

## Automated Windows release

The release workflow is configured to build and publish from a release branch named like `release/v0.1.0-alpha.2`. Push a reviewed release branch based on the exact `main` commit intended for testing; the workflow runs on Windows and publishes the matching `v0.1.0-alpha.2` GitHub prerelease with MSI and NSIS installers.

The workflow:
1. Verifies Node.js 24 LTS and npm 11.
2. Runs frontend formatting/lint, repository regression tests, frontend tests, TypeScript and production build checks.
3. Runs Rust formatting, unit tests and Clippy with warnings denied.
4. Builds Tauri Windows MSI and NSIS installers.
5. Verifies both installers exist before publishing the GitHub Release with both files attached.

The workflow uses the repository's `GITHUB_TOKEN`; no personal access token should be added.

## Pre-release checklist

- [ ] All intended source changes are on the reviewed release commit.
- [ ] Review the release branch diff against `main` and confirm version values match.
- [ ] Confirm Node 24/npm 11 and all frontend/Rust gates are green in GitHub Actions.
- [ ] Confirm the release contains both MSI and NSIS installers and their SHA-256 digests.
- [ ] Install the NSIS `.exe` on a clean supported Windows machine.
- [ ] Install the MSI on a clean supported Windows machine.
- [ ] Launch the app, test first-run profile, Ctrl+K, theme, Settings, and resize/overflow behavior.
- [ ] Close/reopen, inspect startup errors, then uninstall without deleting user data unexpectedly.
- [ ] Record OS version, display scaling, package format, install/uninstall outcome, and known issues.

## Alpha scope and limitations

This is a **foundation preview**, not a production-ready feature release. Tauri's Windows MSI packager requires numeric-only prerelease identifiers, so the embedded app/package version for alpha.2 is `0.1.0-2`; the human-facing GitHub release tag is `v0.1.0-alpha.2`. Keep `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` versions aligned.

The current interface is a desktop shell. Projects, tasks and notes are preview-only; operational tools and persistent project/task storage are not connected. Profile choices change pinned sections only and do not grant permissions. A successful CI installer build verifies compilation and package generation; it does not replace manual install/launch/uninstall testing on a real Windows device.

## Release candidate: v0.1.0-alpha.2

This release candidate includes responsive shell/overlay overflow hardening and a source-level accessibility review baseline. Automated assertions were added for the viewport/keyboard contract, but do not report them as passing until CI completes. The review document lists contrast spot checks and explicitly pending Windows manual tests.

Previous prerelease: [v0.1.0-alpha.1](https://github.com/bjo163/moonships/releases/tag/v0.1.0-alpha.1).
