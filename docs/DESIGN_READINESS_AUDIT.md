# Moonships Product & Design Readiness Audit

**Review date:** 2026-10-25  
**Repository:** `bjo163/moonships`  
**Status:** Design-spec audit; not proof of a running application or user-tested UI  
**Scope:** Product definition, UX/design completeness, implementation readiness, security boundaries, validation and task coverage.

## Executive conclusion

Moonships already has a substantial written product contract. The repository should not be described as having “no design”: `PRODUCT_BLUEPRINT.md` defines product scope and release stages, while `UX_AND_DESIGN.md` includes navigation, key flows, wireframes, initial design tokens, states and a UI checklist. The gap is that the design is still **documentary and unvalidated**: there is no implemented desktop shell, no running UI, no visual regression evidence, no approved shipped asset pack, and no user test evidence. The canonical UI design contract is now docs/DESIGN_SYSTEM.md and the canonical sound contract is docs/AUDIO_SFX_SPEC.md; these are specifications, not implemented product features.

The next step is not to write another competing mega-blueprint. Keep the existing documents canonical, close the small implementation-readiness gaps listed here, then build the smallest real vertical slice and validate it on Windows.

## Current evidence vs. assumptions

| Area | Evidence in repository | Honest status |
|---|---|---|
| Product proposition and audiences | `PRODUCT_BLUEPRINT.md` | Specified; needs prioritization validation |
| Navigation, workflows and low-fi layouts | `UX_AND_DESIGN.md` | Specified; not implemented or user-tested |\n| Visual tokens, components, motion and asset governance | `DESIGN_SYSTEM.md` | Canonical proposal; contrast and behavior not yet validated in app |\n| SFX event catalog and playback policy | `AUDIO_SFX_SPEC.md` | Canonical proposal; default off; no approved/shipped SFX yet |
| Initial dark-theme tokens | `UX_AND_DESIGN.md` | Proposed values; contrast not yet measured |
| Architecture and technology | `ARCHITECTURE.md` | Proposed baseline; no app source yet |
| Data and security contracts | `DATA_MODEL.md`, `SECURITY_MODEL.md` | Documented baseline; runtime enforcement absent |
| Milestones and atomic tasks | `MILESTONES.md`, `TASK_BACKLOG.md` | Planned; implementation tasks remain open until evidenced |
| Backlog-to-Issues automation | Workflow and `TASK_ISSUE_SYNC.md` | Automation exists; continue validating reconciliation edge cases |
| Desktop behavior | No desktop app source/build/tests currently present in repository map | Not implemented |
| Visual fidelity / usability | No rendered application or test evidence | Unknown; do not claim complete |

## Product decisions to preserve

1. Windows-first desktop product using Rust + Tauri 2 and React + TypeScript + Vite.
2. SQLite and local-first core behavior; no mandatory account, backend, AI provider, or cloud sync.
3. Modular monolith before service decomposition.
4. Projects and Tasks are first-class core functionality, not a secondary feature.
5. One registered tool contract and one policy boundary across UI, tray, workflows and future AI/CLI surfaces.
6. Planned, experimental, unsupported, degraded, stale, failed and verified states must remain distinguishable.
7. No autonomous destructive remediation, unrestricted shell, unauthorized security testing, silent privilege escalation or background polling without explicit configuration.
8. Do not copy World Monitor source code or branding without a deliberate license/terms review. Treat it as a product reference only for now.
9. Do not select or imply a project license until the owner makes that decision.

## Design readiness findings

### Canonical design sources (do not create parallel specifications)\n\n- UI tokens, component behavior, visual identity, motion, accessibility, asset provenance and visual QA: [Design System](DESIGN_SYSTEM.md).\n- SFX defaults, semantic event IDs, playback/throttling, asset manifest and tests: [Audio & SFX Specification](AUDIO_SFX_SPEC.md).\n- Navigation, user flows and low-fidelity layouts only: [UX & Design](UX_AND_DESIGN.md).

- Product promise, audiences, domains, V0.1–V1.0 scope and post-V1 tracks: `PRODUCT_BLUEPRINT.md`.
- Main navigation, home, task/project flows, command palette, tray, onboarding, tool flows, dark/light theme direction, component inventory, accessibility intent, error states and low-fidelity layouts: `UX_AND_DESIGN.md`.
- Internal runtime boundaries, tool contracts, policy, persistence and adapters: `ARCHITECTURE.md`.
- Entities and persistence expectations: `DATA_MODEL.md`.
- Threat model, capabilities, risk classes and approval rules: `SECURITY_MODEL.md`.
- Feature ownership and non-overlap invariants: `OVERLAP_AUDIT.md`.
- Small implementation tasks and issue sync: `TASK_BACKLOG.md`, `TASK_ISSUE_SYNC.md`.

### Gaps to close during implementation

1. **No implemented reference screens.** Build the shell and a real Home/Projects/Tasks vertical slice before polishing specialist screens. A markdown wireframe is not a running prototype.
2. **Window-size behavior needs a hard implementation contract.** The UX spec gives review viewports but implementation must also define a tested minimum window size, resizing behavior, sidebar collapse behavior, and overflow rules for tables/detail panels. Do not make the interface unusable when the window is smaller.
3. **Tokens need measured verification.** Implement semantic tokens for both themes and verify actual foreground/background, focus, borders and status combinations against WCAG 2.2 AA where applicable. Proposed hex values are not evidence of passing contrast.
4. **Components need behavior, not only names.** Implement consistent loading/empty/error/offline/stale/permission-denied/partial/cancelled states; accessible dialogs; focus return; and clear destructive-action previews. Avoid inventing a second status or risk enum in the UI.
5. **Visual QA is not yet operationalized.** Add a repeatable Windows screenshot/manual review matrix for the key shell, Home, Project, Task, Command Palette, and destructive preview flows. Use screenshots as review evidence, not as a substitute for behavior tests.
6. **First-run needs a real exit path.** Onboarding must be skippable or resumable, persist profile/layout choices, never grant permissions, and never require account/API-key setup for core use.
7. **Navigation needs honest unavailable states.** Routes must distinguish implemented pages from Planned/Not configured/Unsupported; no dead links and no fabricated live data.
8. **Data and job states need consistency.** The UI must distinguish a job's lifecycle from business activity history and diagnostic logs; no fake progress, fake cancellation, or success on partial/provider failure.
9. **No product validation evidence exists yet.** After the first vertical slice, run a small task-based usability review with representative workflows and record friction before expanding the menu.
10. **Sound and visual assets need provenance.** No sound file, logo, illustration or font ships until license/source, fallback and review status are recorded; SFX remains off by default.\n11. **License and external-source review remain release gates.** UI inspiration does not grant permission to copy code, assets, branding or data.

## Recommended screen build order

1. **Desktop shell:** app frame, sidebar, top bar, route registry, theme, focus/keyboard behavior, status area.
2. **Home:** useful empty state, New Project/New Task, recent work only from real persisted records.
3. **Projects:** list/create/detail and empty/loading/error states.
4. **Tasks:** create/edit, List and Board views backed by one canonical model, filters and history.
5. **Command Palette:** local deterministic search, navigation/create commands, keyboard selection, Escape/focus restoration.
6. **Settings and tray:** theme, data/privacy basics, close/hide/quit semantics, no startup-at-login unless a later explicit decision changes scope.
7. **Tool surfaces:** only after registry, permissions, progress, error, cancellation and result contracts exist.
8. **Specialist domains:** follow the product milestones; do not build all menu items as empty pages.

## Design acceptance gates

A screen is ready for implementation review only when:

- The user goal and primary action are explicit.
- Inputs, outputs, empty/loading/success/partial/error/offline/stale/permission-denied/cancelled states are defined as applicable.
- Keyboard and focus behavior are defined and testable.
- Data freshness, source, permission and risk are visible when relevant.
- Destructive actions show exact targets and changes before confirmation, then revalidate before execution.
- The page uses shared tokens/components and does not introduce duplicate domain state.
- At least one behavior test covers the primary path and one covers a failure/edge path.
- The UI is reviewed at 1280×720 and 1440×900, plus the implemented minimum window size.
- Accessibility checks include visible focus, logical headings/labels, keyboard operation, status announcements and measured contrast.
- The implementation matches the documented feature status; screenshots and badges must not imply unbuilt capabilities.

## Research references

These are primary references for implementation checks, not endorsements to copy another product's assets:

- Tauri capabilities and webview permissions: https://v2.tauri.app/security/capabilities/
- Tauri system tray: https://v2.tauri.app/learn/system-tray/
- Tauri architecture/project: https://github.com/tauri-apps/tauri
- W3C WCAG 2.2: https://www.w3.org/TR/wcag/
- Microsoft Fluent accessibility guidance: https://fluent2.microsoft.design/accessibility
- Microsoft Fluent layout and 4px spacing guidance: https://fluent2.microsoft.design/layout

Tauri capabilities constrain what a frontend webview can invoke, but native command handlers must still validate arguments and enforce policy. WCAG 2.2 is a useful testable baseline for the web-rendered UI; desktop-specific lifecycle, security and Windows behavior require additional native tests.

## Final recommendation

**Do not expand the feature catalogue yet.** Treat M1 as the design-to-product bridge: implement the smallest real desktop shell, verify the shared design system and keyboard/accessibility behavior, then ship Projects & Tasks as the first complete vertical slice. Only after that should Moonships invest in the broad specialist menus. This minimizes overlapping specs, prevents fake-complete screens, and gives future AI coding agents a concrete UI and test pattern to replicate.
