# Desktop Release Guide

## Automated Windows release

Create a release branch such as `release/v0.1.0-alpha.1` from the exact commit to publish. Once this workflow is merged into `main`, pushing that branch starts `.github/workflows/desktop-release.yml`; the workflow creates the matching `v0.1.0-alpha.1` tag on the same commit and publishes the release assets.

The workflow:
1. Verifies Node.js 24 LTS and npm 11.
2. Runs frontend formatting/lint, repository regression tests, frontend tests, TypeScript and production build checks.
3. Runs Rust formatting, unit tests and Clippy with warnings denied.
4. Builds Tauri Windows MSI and NSIS installers.
5. Verifies both installers exist before publishing a GitHub Release with both files attached.

The workflow uses the repository's `GITHUB_TOKEN`; no personal access token should be added.

## Pre-release checklist

- [ ] All task PRs merged to `main` and required CI checks are green.
- [ ] Review the tagged commit and release notes.
- [ ] Confirm the release workflow uploads both MSI and NSIS installers.
- [ ] Install the MSI on a clean supported Windows machine.
- [ ] Install the NSIS package on a clean supported Windows machine.
- [ ] Launch the app, confirm the window renders, close/reopen, and inspect for startup errors.
- [ ] Record OS version, package format, install/uninstall outcome, and known issues.

## Alpha scope and limitations

The first release is a **foundation preview**, not a production-ready feature release. The GitHub tag remains `v0.1.0-alpha.1`; Tauri's Windows MSI packager requires numeric-only prerelease identifiers, so the embedded package version is `0.1.0-1`. The release tag is the human-facing prerelease label. The current interface is a desktop shell; operational tools, persistence, system tray behavior and the complete roadmap are not implemented merely because installers build successfully. Release notes must not claim those capabilities are available.

A successful CI installer build verifies compilation and package generation. It does not replace manual install/launch/uninstall testing on a real Windows device.
