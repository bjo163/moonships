# Testing and Release Gates

## Local checks

Run the supported toolchain from the repository root:

```sh
npm install
npm run format:check
npm run lint
npm run test:repo
npm test
npm run typecheck
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Windows installer generation must be validated on Windows because Tauri uses the Windows MSVC toolchain and Windows bundlers.

## CI expectations

- Frontend component smoke tests run separately from repository backlog-sync regression tests.
- Rust unit tests run as a separate job from Rust formatting and Clippy.
- The Windows packaging job must produce both an MSI and an NSIS setup executable. A successful bare executable build alone is not an installer release.
- Release artifacts are previews until core application features and manual installation/runtime checks are completed.

## Alpha release scope

The initial desktop shell is a foundation preview. Do not describe planned or disconnected features as operational. A release must identify its version, platform, installer formats, and known limitations.

## M1-041 — shared-component interaction evidence

The shared Button, Input, Badge, Dialog, Toast, EmptyState, LoadingState, ErrorState and Progress implementations live in `src/ui-primitives.tsx`. Tests are intentionally split:

- `src/ui-primitives.test.tsx`: static markup/semantic contract checks with the existing Vitest Node environment.
- `src/ui-primitives.behavior.test.tsx`: jsdom-scoped React Testing Library + user-event tests for actual DOM focus, Escape, focus cycling, backdrop dismissal, focus restoration, typed input/invalid descriptions, disabled/loading actions, live status announcements, retry and bounded native progress values.
- `npm test`: runs **both** suites (plus existing App tests); `npm run check` enforces format/lint/tests/typecheck/build.

A DOM simulation is not a real WebView2 or a complete WCAG 2.2 AA audit. Do not infer Windows visual, forced-colors, contrast or tray-runtime PASS from green unit tests.

### Windows manual follow-up (where shared primitives are exposed in a built UI)

Record the exact commit/installer, Windows version, text/display scaling, theme, expected result, observed result and reproduction steps. Mark a component `NOT EXPOSED` rather than claiming PASS if the current desktop shell does not yet render it.

| Action | Expected behavior |
|---|---|
| Open and close a component Dialog using keyboard | Initial focus lands inside; Tab/Shift+Tab do not escape; Escape closes and focus returns to the launcher |
| Click on Dialog content, then outside/backdrop | Content click keeps it open; backdrop click closes it without activating content |
| Submit a form with an invalid Input | Persistent label remains visible, error is announced/associated and entered value is retained |
| Activate enabled, disabled and loading Buttons | Only enabled action fires; busy/disabled state and focus indication remain intelligible |
| Trigger success/error Toast and progress states | Status text and roles are readable to assistive technology; progress represents actual data |
| Repeat with OS reduced motion, dark/light and forced colors | Spinner movement stops under reduced motion; focus/error/status remains distinguishable without color alone |

These checks complement M1-041 interaction tests. They do **not** replace the independent Windows tray/usability gate **M1-040** or computed-contrast/visual review **M1-039**. Do not capture personal workspace data, credentials or private documents in evidence.

## M1-032/033 — registered commands and shortcut lifecycle

- `src/command-registry.behavior.test.tsx` runs six jsdom App-level interaction cases for the exact allowlisted palette entries, rejected unknown registry IDs, safe route navigation, Settings opening, and project/task/note create previews that make no persistence/execution claims.
- `src/command-shortcut.behavior.test.tsx` runs five jsdom App-level cases for Ctrl+K search focus, Escape and opener focus restoration (including Escape from the Close button), editable input/textarea exclusion, blocking nested Settings shortcuts, and focus handoff to Settings and command preview.
- The native window's Tab-cycle, OS scaling, WebView2 focus behavior and actual Windows accessibility results remain **NOT TESTED**; M1-039 and M1-040 own the manual and computed-contrast review.
- M1-032 implementation CI [40239724966](https://github.com/bjo163/moonships/actions/runs/40239724966) succeeded across all five jobs. At this edit the M1-033 latest source commit `0d0a7c3` passed the frontend CI job in [40240662226](https://github.com/bjo163/moonships/actions/runs/40240662226); recheck that run and all jobs before asserting complete repository CI.
