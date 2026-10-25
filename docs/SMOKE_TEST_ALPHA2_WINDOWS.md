# Windows Smoke Test — Moonships v0.1.0-alpha.2

**Test target:** Windows desktop installer, Moonships Workbench v0.1.0-alpha.2  
**Execution:** user-reported manual smoke test  
**Date recorded:** 2026-10-25  
**Evidence type:** user-provided observations; not independently re-executed by CI

## Results

| Check | Result | Observation |
|---|---|---|
| First launch and Overview | USER-REPORTED PASS | Application opens and Overview displays without an error. |
| Command Palette | USER-REPORTED PASS | Ctrl+K opens it; a command can be used; Escape closes it. |
| Workspace Profile | USER-REPORTED PASS | Dialog opens and profile selection can be saved. |
| Theme preference | USER-REPORTED PASS | Dark/light toggle works and preference persists after restart. |
| Resize and responsive shell | USER-REPORTED PASS | Sidebar and content remain readable when the window is resized smaller and larger. |
| Keyboard and focus | USER-REPORTED PASS | Keyboard navigation works and visible focus indicators are clear. |

## Scope not yet verified

- Tray hide/restore/quit behavior was not included in the reported run.
- The complete M1-039 accessibility matrix remains pending, including the exact 960×640, 1280×720 and 1440×900 viewports, settings-dialog edge cases, OS reduced-motion preference, and full computed-color contrast checks.
- This report records user-observed behavior; it is not an automated test result or a claim of complete WCAG 2.2 AA conformance.

## Follow-up

- Keep M1-040 open until tray hide/restore/quit behavior is tested and the remaining acceptance criteria are recorded.
- Use this report as evidence for the six smoke-test flows above.

## Manual verification protocol for the next Windows candidate

**Target:** the next Windows CI installer built from the exact commit under review, **not** the previously published Alpha.2 binary. Record the commit SHA, installer filename, Windows version, display scale and execution date before testing. Do not label tests below as PASS until actual observations are supplied.

| Test | Procedure | Expected result | Status |
|---|---|---|---|
| Tray hide | Enable "Hide to tray on close" in Settings, then close the main window | Window hides; application process and tray item remain available | NOT TESTED |
| Tray restore | From tray menu select Open | Original main window is shown and focused | NOT TESTED |
| Tray Settings | From tray menu select Settings | Main window becomes visible and Settings opens | NOT TESTED |
| Tray quit | From tray menu select Quit | Main window and process exit cleanly | NOT TESTED |
| Window sizes | Review 960×640, 1280×720 and 1440×900 at 100% and 150% Windows scaling | Navigation, overlays and primary actions remain reachable without clipping | NOT TESTED |
| Keyboard/focus | Open/close Palette, Settings and Profile with keyboard; test Tab, Shift+Tab, arrows, Enter and Escape | Focus is visible, remains within active modal where applicable and returns to the opener | NOT TESTED |
| Motion setting | Enable Reduce animations in Windows accessibility settings and reopen each loading/overlay state | No essential status requires animation; moving effects respect the preference | NOT TESTED |
| Computed contrast | Inspect actual foreground, surface, focus, hover, disabled and error-state colors in both themes | Applicable WCAG 2.2 AA targets are met, or defects are recorded precisely | NOT TESTED |
| Settings edge cases | Toggle theme/tray, restart, verify persistence, and inspect unavailable or invalid-state feedback | Persistence and error feedback match actual device state | NOT TESTED |

For every failed check, include expected versus observed behavior, exact commit/build, reproduction steps and an existing issue ID where possible. Do not include personal workspace data, tokens, or confidential screenshots. Keep M1-040 and M1-039 open until their outstanding checks have evidence.
