# Windows Smoke Test — Moonships v0.1.0-alpha.2

**Test target:** Windows desktop installer, Moonships Workbench v0.1.0-alpha.2  
**Execution:** user-reported manual smoke test  
**Date recorded:** 2026-10-25  
**Evidence type:** user-provided observations; not independently re-executed by CI

## Results

| Check | Result | Observation |
|---|---|---|
| First launch and Overview | PASS | Application opens and Overview displays without an error. |
| Command Palette | PASS | Ctrl+K opens it; a command can be used; Escape closes it. |
| Workspace Profile | PASS | Dialog opens and profile selection can be saved. |
| Theme preference | PASS | Dark/light toggle works and preference persists after restart. |
| Resize and responsive shell | PASS | Sidebar and content remain readable when the window is resized smaller and larger. |
| Keyboard and focus | PASS | Keyboard navigation works and visible focus indicators are clear. |

## Scope not yet verified

- Tray hide/restore/quit behavior was not included in the reported run.
- The complete M1-039 accessibility matrix remains pending, including the exact 960×640, 1280×720 and 1440×900 viewports, settings-dialog edge cases, OS reduced-motion preference, and full computed-color contrast checks.
- This report records user-observed behavior; it is not an automated test result or a claim of complete WCAG 2.2 AA conformance.

## Follow-up

- Keep M1-040 open until tray hide/restore/quit behavior is tested and the remaining acceptance criteria are recorded.
- Use this report as evidence for the six smoke-test flows above.
