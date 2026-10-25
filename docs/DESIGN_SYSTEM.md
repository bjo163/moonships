# Moonships Design System

**Status:** Canonical design contract — proposed; implementation and validation pending  
**Scope:** UI structure, visual identity, design tokens, components, motion, accessibility, asset governance and visual QA.  
**Companion:** [Audio & SFX Specification](AUDIO_SFX_SPEC.md) is authoritative for sound behavior and audio assets.  
**Information architecture:** [UX, Navigation & Design Specification](UX_AND_DESIGN.md) owns menus, user journeys and low-fidelity page layouts. It must reference this document rather than redefine visual tokens.

## 1. Product design direction

Moonships Workbench should feel like a **precision-built desktop command workbench**: calm, capable, tactile, technical and slightly futuristic. It must not look like a generic admin template, a neon hacker dashboard, a game HUD or a marketing website.

The visual signature is **Orbit / Signal / Instrument**:
- **Orbit:** stable spatial frame, clear navigation and connected work objects.
- **Signal:** restrained accent colors and explicit state cues.
- **Instrument:** precise typography, useful tables, timestamps, measurements, evidence and predictable controls.

Use graphite/ink surfaces, a moon-lilac primary accent, cool cyan secondary accent, warm amber warnings and restrained red for danger. Original orbital or lunar line motifs belong in identity, onboarding and empty states—not behind dense data or on every page.

This is a workbench, not a poster. **Legibility, speed, evidence, user control and truthful status outrank visual effects.**

## 2. Normative design principles

1. Truth over theatre: no fake metrics, simulated activity, invented progress or decorative charts that imply real data.
2. One action, one outcome: every primary action has a clear label, pending state and honest terminal state.
3. Shared system, distinct modules: pages reuse the same shell, semantic tokens, component patterns and state model.
4. Dense but breathable: support technical information density without sacrificing scanability.
5. Keyboard is first-class: important operations never require a mouse alone.
6. Safe by design: target, scope, permissions, risk and consequences are visible before consequential actions.
7. Progressive disclosure: show the next useful decision first; advanced configuration stays out of the way.
8. Local-first honesty: distinguish local, cached, stale, remote, model-generated and verified information where relevant.
9. Quiet by default: no background music, surprise audio, flashing or attention-seeking motion.
10. Accessibility is behavior, not a theme. Color, icons, sound and animation are never the only status channel.
11. Performance is part of design. No permanent particle fields, heavy blur or unnecessary polling.
12. Reuse before invention. Add a token or component only when existing semantics cannot represent the need.

The words MUST, MUST NOT, SHOULD and MAY are normative requirements for implementation and review.

## 3. Brand and visual identity

### Personality
- Precise, capable, calm, resilient and quietly unconventional.
- Technical without intimidating non-specialists.
- Futuristic through composition and detail, not sci-fi clichés.
- Friendly through useful feedback, not mascot clutter.
- High information density in working areas; minimal ornament in operational views.

### Brand mark and motifs
Create an original Moonships mark from simple lunar/orbital geometry: a stable circular or crescent form with one trajectory/signal detail. It must work in monochrome, at 16×16 tray/icon size, in both themes and without glow.

### Asset rules
- Do not copy another product's logo, icon set, illustrations, sound files or distinctive trade dress.
- Do not use emoji as production navigation icons.
- Use one consistent, licensed line-icon family; Lucide is the initial candidate, subject to implementation review.
- Create original lightweight vector illustrations only for onboarding, empty states and About. Avoid large illustrations in dense work screens.
- Start with platform UI fonts. Bundle a brand font only after license, performance and language coverage review.
- Keep source files editable and record creator, source, license, version and intended use in an asset manifest.
- Do not require remote fonts or images for core app rendering.
- A missing asset must never break layout or hide an action.

### Icon rules
- Standard UI icon: 16–18 px; prominent action/page icon: 20 px; navigation icon: 18 px.
- Keep one consistent stroke weight and optical alignment.
- Icon-only controls need accessible names and pointer tooltips; ambiguous or destructive actions must not be icon-only.
- A status icon must be paired with text or accessible status text.
- Never use warning, lock, shield or network symbols decoratively where they could imply a real state.

## 4. Desktop layout and responsive behavior

### Initial targets
- Review viewports: 1280×720 and 1440×900.
- Proposed minimum client-area size: 1024×640, pending verification in a real Windows build.
- Expanded sidebar: 248 px; collapsed rail: 68 px.
- Top bar: 56 px.
- Page padding: 24 px wide, 16 px compact.
- Main content reading width: typically 1200 px; tables may use more width.
- Spacing base: 4 px; common rhythm 4/8/12/16/24/32/40/48 px.
- Control heights: compact 28–32 px, default 36 px, large 40–44 px, subject to accessibility review.
- Prefer grid/flex layouts and min-width: 0; never depend on absolute positioning for primary page structure.

These are proposed implementation targets, not verified constraints until tested on Windows.

### Resize rules
- At compact widths, collapse the sidebar to an icon rail with an explicit expand control and accessible labels.
- Tables scroll within their own region; the whole window must not gain accidental horizontal overflow.
- Detail inspectors may become drawers or stack below primary content.
- Toolbars may wrap or move secondary actions into a labelled overflow menu; keep the primary action visible.
- Dialogs scroll internally when tall; footer actions remain reachable.
- At minimum size, no primary action, risk detail, focus ring or error message may be clipped.
- Essential content must never be revealed only on hover.

### Page composition
A typical page uses scope/breadcrumb when useful, title and one primary action, supporting description or freshness/risk context, justified filters, the main work area, and secondary inspector/history when needed. Do not put every status indicator in a permanent global bar.

## 5. Design token contract

Tokens are the only approved route for shared colors, typography, spacing, radii, borders, focus, elevation and motion. Component CSS must not introduce arbitrary color values for normal UI states.

Required semantic token groups:
- Canvas/surfaces: canvas, sidebar, surface, raised surface, overlay.
- Text: primary, muted, subtle, inverse.
- Borders: default and strong.
- Actions: accent, hover, pressed, focus ring and link.
- Status: success, warning, danger and info, with paired surface/text tokens.
- Risk: read-only, change, destructive, privileged and remote.
- Data visualization: categorical series colors separate from semantic status colors.
- Typography: UI/mono families, size, line-height and weight scales.
- Spacing: numbered tokens mapped to a 4 px base.
- Shape: control, card, dialog and pill radii.
- Elevation: menu and dialog shadows; shadows never act as the only boundary signal.
- Motion: fast/normal/slow duration and standard/enter/exit easing.
- Density: compact and comfortable without hiding content or reducing keyboard usability.

### Proposed initial palette (not contrast-certified)

| Semantic role | Dark proposal | Light proposal | Rule |
|---|---|---|---|
| Canvas | #0B1017 | #F6F8FB | Root background |
| Sidebar | #0E1520 | #EDF1F6 | Navigation |
| Surface | #121C28 | #FFFFFF | Cards/panels |
| Raised surface | #182536 | #F2F5FA | Hover/raised surfaces |
| Border | #2A394B | #D4DCE7 | Structure |
| Primary text | #F1F5F9 | #172033 | Main copy |
| Muted text | #A8B5C5 | #536176 | Secondary copy; measure contrast |
| Primary accent | #A69BFF | #6356D9 | Primary action and selected navigation |
| Secondary accent | #38BDF8 | #007DA8 | Links and secondary signals |
| Success | #22C55E | #167A3E | Verified success |
| Warning | #F59E0B | #8A5200 | Warning/stale; pair with icon/text |
| Danger | #F87171 | #B4232F | Error/destructive; pair with icon/text |
| Info | #60A5FA | #245FC0 | Informational state |

These values are hypotheses for the first implementation. Calculate actual text and non-text contrast in each supported theme, adjust as necessary, and record the results. Never claim WCAG conformance from proposed values alone. Do not use status colors as decorative glow.

### Typography
- Use a platform UI sans-serif stack and platform monospace for code, paths, hashes and diagnostic values.
- Initial scale targets: 12 px metadata, 13–14 px dense data, 14 px default body/control, 16 px section headings, 20–24 px page titles.
- Avoid all-caps UI copy and oversized marketing typography.
- Use tabular numerals for measurements, timestamps, sizes and counters.
- Critical risk, source, permission and destructive-target information must remain inspectable and accessible even when visually truncated.

## 6. Component and interaction contract

The implementation may use a component library as a foundation, but Moonships owns semantic tokens, variants and behavior. Do not create a second component system for individual modules.

### Core inventory
AppShell, Sidebar, TopBar, Workspace/ProfileSelector, Breadcrumbs, PageHeader; Button, IconButton, Link, Input, Textarea, Select, Checkbox, Radio, Switch, SegmentedControl; Tooltip, Popover, DropdownMenu, ContextMenu, CommandPalette; Dialog, Drawer, ConfirmDialog, PreviewDialog; Tabs, Accordion, Card, Separator, Table, DataGrid, List, Board, Timeline; StatusBadge, RiskBadge, SourceBadge, FreshnessIndicator, PermissionSummary; EmptyState, LoadingState, Skeleton, ProgressPanel, ErrorState, OfflineState, StaleState, PartialResultState; Toast, NotificationCenter, ActivityFeed, JobRow, ResultSummary, ArtifactList, SourceCitation; SettingsGroup, SettingsRow, ThemePicker, DensityPicker, SoundPreferences.

Every core component documents default, hover, focus-visible, pressed, disabled, loading and invalid/error states where applicable.

### Actions
- One visually dominant primary action per page/dialog section unless the workflow genuinely has parallel primary actions.
- Secondary actions are quieter. Destructive actions name the verb and target, not just a color.
- Disabled actions explain why when the reason is not obvious.
- Async actions prevent duplicate submission, show truthful progress and expose a terminal result.
- Show cancellation only if cancellation is genuinely supported.
- Never report success before persistence/provider/native operation confirms it.
- Risky-action previews identify target, scope, expected changes, skipped items, permissions and reversibility. Consequential actions require separate confirmation.

### Forms and data
- Persistent labels; placeholders are examples, not labels.
- Validation appears next to fields and in a summary for long forms. Preserve entered values after recoverable failures.
- Tables have named empty states, loading/error states, headers, keyboard behavior and explicit overflow rules.
- Board and List views share one canonical domain model; they must not maintain duplicate status state.
- Tool results show source, timestamp, scope, freshness and uncertainty where relevant.
- Do not show charts unless data supports the comparison and underlying values are inspectable.

### Dialogs and keyboard
- Dialog open focuses a meaningful element; modal Tab remains inside; Escape closes only when safe; close returns focus to the trigger.
- Destructive confirmation initially focuses the safe/cancel action.
- Command Palette supports arrow-key selection, Enter, Escape and focus restoration.
- Shortcuts are discoverable and must not break text editing or OS-standard behavior.
- Focus indicators remain visible and are not clipped.
- Status updates are announced politely; assertive announcements are reserved for urgent failures.
- A toast is not the only record of a consequential result; link it to history/activity.

## 7. Shared state semantics

Every screen uses this common vocabulary where applicable: not configured, idle, loading, running, waiting for approval, succeeded, partial, empty, stale, offline, permission denied, invalid, cancelled, timed out, failed, unsupported and degraded.

- Distinguish business-record state, job execution state and diagnostic log severity.
- Succeeded means the operation was verified; partial remains partial.
- Empty data is not a failed fetch.
- Stale data is labelled and includes last-updated time when known.
- Permission denied never triggers automatic escalation.
- Unknown states fail visibly and safely, never silently become success.
- Pair color with text/icon and accessible announcements.

## 8. Motion and micro-interaction

Motion communicates hierarchy, entry/exit, progress or a state transition. It is not ambient decoration.

- Initial targets: 100–160 ms for small feedback, 140–220 ms for menus/dialogs, 160–240 ms for page transitions; tune after testing.
- Use ease-out for entry and ease-in for exit; avoid spring/bounce by default.
- Page navigation uses a quick fade or minimal transition, not a long sliding scene.
- Progress indicators represent actual known progress; use indeterminate progress when the total is unknown.
- No continuous background animation, parallax, animated gradients, particle fields or flashing alerts in normal work views.
- Honor the OS reduced-motion preference by default and provide an app-level reduced/no-motion setting.
- Reduced motion removes nonessential movement but preserves state feedback.
- Never convey progress, danger or completion solely through animation.

## 9. Sound integration

[Audio & SFX Specification](AUDIO_SFX_SPEC.md) is the only source of truth for sound behavior, event IDs, defaults, throttling and the audio asset catalog. UI components request semantic events through one central sound service; components must not play arbitrary files directly.

High-level invariant: SFX is optional, event-driven, brief, nonverbal and never the sole status channel. No music, ambient loop, surprise startup sound or audio from untrusted remote content.

## 10. Asset manifest and governance

Each nontrivial asset record includes asset ID, path/type, description, intended use, creator/source URL, license/SPDX identifier, attribution requirement, version/checksum where practical, dimensions/duration, theme variants, status and fallback.

Use asset statuses proposed, approved, implemented and deprecated. Unknown rights block release. Missing assets must not break layout. Prefer local vector UI icons and lightweight optional illustrations. No remote fonts/images in the core shell. App/tray icons need small-size inspection and light/dark contrast review.

## 11. Settings and user preferences

At minimum:
- Theme: system/dark/light.
- Density: comfortable/compact.
- Motion: follow OS/reduced/off.
- Sound: off/essential-only/all supported, with master and volume controls.
- Notification categories and quiet behavior when notifications are implemented.

Preferences persist locally and are validated; invalid values fall back safely. Onboarding explains preferences but never silently grants permissions or enables background/network behavior.

## 12. Accessibility and quality gate

Use WCAG 2.2 AA as the baseline for web-rendered UI where criteria apply, plus Windows-specific checks. This document does not certify conformance.

Required checks:
- Measure text contrast (4.5:1 for normal text; 3:1 for large text) and relevant non-text boundaries/focus indicators against applicable criteria.
- Core flows work with keyboard alone; focus is logical, visible and not obscured.
- Controls have programmatic names, roles, values and validation messages.
- Status/error/progress updates are exposed to assistive technology without noisy repeated announcements.
- Color is never the only status indicator.
- All SFX can be disabled; no important information exists only in audio.
- Reduced-motion behavior is tested; no flashing or unnecessary motion.
- Test 1280×720, 1440×900 and confirmed minimum window size, keyboard-only operation and Windows scaling.
- Verify forced-colors/high-contrast behavior where supported.

## 13. Visual QA evidence

For each release candidate, capture screenshots from a reproducible real Windows build and record app version/commit, OS build, display scaling, window size, theme, density, flow, test-data type, expected/observed result, keyboard/focus results, contrast output and reviewed visual diffs. Never silently bless a known defect in a baseline.

Minimum screen matrix:
1. First run/onboarding.
2. Home with genuine empty and populated local states.
3. Project list/detail and create/edit.
4. Task List and Board, including empty, filtered and error states.
5. Command Palette and focus restoration.
6. Settings: theme, density, motion and sound.
7. Tray hide/restore/quit.
8. Preview/confirmation for a simulated destructive operation.
9. Loading, partial, offline, permission-denied and failed job results.

A screenshot proves visual rendering only; behavior tests and accessibility checks are separate gates.

## 14. Implementation checklist

- [ ] Shared token file is implemented and components consume semantic tokens.
- [ ] Dark/light contrast is measured; no conformance claim before evidence.
- [ ] Resize rules pass minimum, 1280×720 and 1440×900 checks.
- [ ] Core component states and keyboard behavior are tested.
- [ ] Reduced-motion and sound preferences persist and are honored.
- [ ] Sound requests use one service and obey settings/throttling.
- [ ] Every shipped asset has provenance/license and fallback recorded.
- [ ] Visual captures come from the real app build.
- [ ] Empty/loading/error/partial/offline/stale/permission-denied states are truthful.
- [ ] No fake live data, surprise sound, background music or decorative activity loop exists.
- [ ] Accessibility and Windows lifecycle checks have recorded evidence.

## 15. Primary references

- [W3C WCAG 2.2](https://www.w3.org/TR/wcag/)
- [Microsoft Fluent 2 — Design](https://fluent2.microsoft.design/get-started/design)
- [Microsoft Fluent 2 — Layout](https://fluent2.microsoft.design/layout)
- [Microsoft Fluent 2 — Motion](https://fluent2.microsoft.design/motion)
- [Tauri 2 — Capabilities](https://v2.tauri.app/security/capabilities/)
- [MDN — Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API)
- [MDN — Using Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Using_Web_Audio_API)

These references inform design; they do not grant permission to copy assets or certify Moonships.
