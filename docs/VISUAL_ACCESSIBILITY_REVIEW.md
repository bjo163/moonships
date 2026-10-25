# Desktop Shell Visual & Accessibility Review

**Task:** M1-039  
**Review type:** source-level review and reproducible manual checklist  
**Status:** partial; Windows visual/keyboard execution still required  
**Scope:** app shell, onboarding/profile dialog, Command Palette, settings and shared overlays

## Environment and evidence

- Source reviewed on the current `main` branch; review is not a claim that a packaged Windows build was manually exercised.
- The native window contract is `minWidth: 960`, `minHeight: 640`; responsive behavior additionally targets 1280×720 and 1440×900.
- Automated regression assertions are added in `src/App.test.tsx`. They have not yet been executed in this authoring environment.
- Contrast ratios below are calculated from the literal foreground/background token pairs using the WCAG relative-luminance formula. These are targeted token pairs, not a complete audit of every rendered component/state.

## Contrast spot checks

| Theme | Foreground token | Background token | Ratio | WCAG AA normal text (4.5:1) |
|---|---|---|---:|---|
| Dark | `#f1f5f9` primary | `#0b1017` canvas | 17.42:1 | Pass |
| Dark | `#a8b5c5` muted | `#0b1017` canvas | 9.16:1 | Pass |
| Dark | `#8190a3` subtle | `#0b1017` canvas | 5.86:1 | Pass |
| Dark | `#a69bff` accent | `#121c28` surface | 7.16:1 | Pass |
| Light | `#172033` primary | `#f6f8fb` canvas | 15.29:1 | Pass |
| Light | `#536176` muted | `#f6f8fb` canvas | 5.91:1 | Pass |
| Light | `#58677b` subtle (adjusted in M1-039) | `#f6f8fb` canvas | 5.42:1 | Pass |
| Light | `#6356d9` accent | `#ffffff` surface | 5.44:1 | Pass |
| Light | `#007da8` link | `#ffffff` surface | 4.67:1 | Pass |

Large text and non-text UI components have different WCAG thresholds. The table does not cover hover/pressed/disabled states, gradients, overlays, focus-ring contrast, opacity compositing, or every component pairing. Do not treat this spot check as a full WCAG 2.2 AA conformance claim.

## Source-level findings addressed

- Responsive layout now collapses the sidebar at 960px and retains a compact top rail at narrow widths.
- Main content and cards use `min-width: 0` and wrapping to avoid grid/flex overflow from long labels.
- Settings and Command Palette overlays can scroll vertically on short windows; dialog content is constrained to the viewport.
- Reduced-motion preference and visible keyboard focus styling remain present.
- Native window minimum remains 960×640; this is the supported minimum, not a guarantee that every responsive breakpoint is equivalent to a phone-sized app.
- Light-theme subtle text token was darkened to improve normal-text contrast against the canvas.

## Manual review matrix (must be run against a Windows build)

| Surface / flow | Viewports | Keyboard/accessibility checks | Result |
|---|---|---|---|
| Main shell and navigation | 960×640, 1280×720, 1440×900 | Tab order, skip link, visible focus, no clipped primary controls | Pending |
| First-run profile selector | 960×640 and 1280×720 | Dialog name, Escape/close, profile selection, focus return | Pending |
| Command Palette | 960×640 and 1280×720 | Ctrl+K/Cmd+K, type-to-filter, arrows, Enter, Escape, focus return | Pending |
| Settings dialog | 960×640 and short 720px height | Toggle labels, scrollability, Escape/close, focus return | Pending |
| Dark and light themes | All three target viewports | Contrast spot checks plus hover/focus/disabled states | Pending |
| Reduced motion | OS preference enabled | No essential information conveyed by animation | Pending |

## Known gaps / follow-up

- Execute `npm run check`, `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `cargo test --manifest-path src-tauri/Cargo.toml --all-targets`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` with Node 24 LTS/npm 11 and Rust stable.
- Inspect real Windows MSI and NSIS installations at 100% and 150% display scaling.
- Run a complete contrast audit on actual computed colors, including focus indicators, disabled controls, hover states and overlay compositing.
- Record actual pass/fail results and any follow-up issue IDs after manual execution; do not replace Pending with Pass based on source inspection alone.
