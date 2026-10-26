# Audio & SFX Specification

**Status:** Canonical sound contract — proposed; no sound assets approved or shipped yet  
**Parent:** [Moonships Design System](DESIGN_SYSTEM.md)  
**Scope:** UI sound effects, event semantics, settings, accessibility, asset production/licensing, playback policy and validation.  
**Non-goal:** background music, ambient loops, voice prompts, notification jingles or audio branding that plays without an explicit user action.

## 1. Audio direction

Moonships audio should feel like a **quiet instrument panel**, not an arcade game. Use subtle, short, clean, low-fatigue, nonverbal sounds with a coherent timbre. Avoid harsh beeps, alarms for ordinary events, loud cinematic impacts, random pitch variation and stock “hacker computer” effects.

Sound is a secondary feedback channel. It never replaces visible text, status icons, screen-reader announcements, logs or history.

## 2. Defaults and user control

- Master SFX default is OFF on first launch. Users explicitly opt in; onboarding never silently enables sound.
- Settings provide master toggle, mode (off, essential-only, all-supported), volume and category controls where practical.
- Provide a preview button that plays only after the user activates it.
- Volume is independent of OS master volume and has a conservative app-level ceiling.
- No music, ambient sound, startup/shutdown sound, idle sound or sound on every keystroke.
- No playback before opt-in/user gesture; respect OS mute/focus/quiet settings where exposed.
- Setting changes apply immediately and persist locally.
- Missing, corrupt or blocked audio never breaks the underlying UI operation; do not show an error toast for SFX failure.
- Bundle approved assets locally. Never fetch or execute audio from remote pages/providers.
- Master-off disables every SFX. Critical status always has non-audio communication.

## 3. Semantic event catalog

The UI requests semantic event IDs through one central SoundService. Components never reference raw audio paths. Events below are proposals, not proof that files exist.

| Event ID | Meaning | Initial mode | Playback rule | Visual/text alternative |
|---|---|---|---|---|
| ui.confirm | Non-destructive action was accepted | all-supported | Once per committed action; none on hover | Visible state/label |
| ui.navigate | User changes major workspace context | off unless enabled | Avoid sound for every route/tab | Active nav and page title |
| ui.open | Command palette/dialog opens | off unless enabled | None for repeated focus changes | Focused dialog/title |
| ui.close | Non-destructive palette/dialog closes | off unless enabled | Do not override system feedback | Focus returns to trigger |
| job.started | A real job begins | essential-only | Only after runtime confirms running; dedupe by job ID | Running state/progress |
| job.succeeded | Job completed and result is verified | essential-only | Once per job ID; not for partial results | Success label and result |
| job.partial | Job completed with partial outcome | essential-only | Distinct restrained cue only if enabled | Partial label/explanation |
| job.failed | Job failed | essential-only | One cue per job ID; throttle repeated failures | Error state/actionable detail |
| approval.required | Consequential action awaits approval | essential-only | Only if notification policy permits; never loop | Approval queue and target |
| connection.lost | Active connection becomes unavailable | all-supported | Once per transition, not every retry | Offline/degraded indicator |
| connection.restored | Previously unavailable connection recovers | all-supported | Once per verified recovery | Connected state/timestamp |
| notification.received | Enabled notification arrives | off unless enabled | Respect quiet hours and category settings | Notification center |
| data.saved | User edit persisted | off unless enabled | Avoid autosave/keystroke repetition | Saved state/time |
| destructive.completed | Approved destructive action has verified outcome | essential-only | Only after actual result, never on click/preview | Result summary/activity |

Essential-only means a small subset of meaningful job/connection/approval events, never an alarm channel. It never overrides master-off. Keep event IDs stable once implemented.

M5-014's opt-in desktop notification is an OS notification, not an app-owned SFX. Its eligibility follows the persisted notification preference; OS quiet hours and presentation settings remain authoritative. Moonships adds no audio asset or component-level playback.

## 4. Playback rules

- One semantic event at most per transition; use an idempotency key for repeated job/notification events.
- Do not overlap duplicate copies. Deduplicate by entity and event transition.
- Apply a conservative global rate limit; coalesce repeated low-priority events and drop stale queued sounds.
- Playback never delays, blocks or changes the underlying operation.
- Failed, cancelled, denied or partial work never plays job.succeeded.
- Hover, focus, typing, scrolling, drag movement and ordinary button presses are silent by default.
- No sound on app launch, hide/restore, theme change or routine polling.
- Notification sound is eligible only when its category is enabled; OS/quiet-hours behavior takes precedence.
- When events coincide, play only the highest-priority permitted event and suppress redundant lower-priority cues.
- Internal job retries do not repeat job.started.
- approval.required never implies approval granted.
- No spatial audio/panning in the first release.
- No event bypasses master-off or the app-level volume ceiling.

## 5. Priority and concurrency

Priority order: approval.required; job.failed/destructive.completed; job.partial; job.succeeded/connection transitions; job.started; UI events/data.saved.

A high-priority event may suppress a low-priority event in the same short window. If the app is busy or audio is unavailable, drop the cue rather than delaying work. No looping alarm or volume escalation.

## 6. Asset specification

### Technical targets
- Bundle local assets only.
- Keep lossless WAV/FLAC as editing masters. Choose a runtime format (OGG Vorbis or WAV) only after testing the packaged Windows WebView2/Tauri target and packaging size.
- UI feedback target: 40–350 ms; meaningful job/approval cue up to 600 ms. These are targets, not rigid requirements.
- Mono is sufficient; no spatial mix.
- Avoid clipping, harsh high-frequency transients, excessive low-frequency energy and long tails.
- Normalize perceived loudness consistently, leave headroom and test on laptop speakers/headphones at low volume. Do not set a LUFS target until the runtime pipeline and listening tests are defined.
- No generated/downloaded sound enters the shipped bundle until its provenance/license and review are recorded.

### Sound identity
Use one small family of related sounds with consistent envelope and tonal palette. Prefer a soft rounded click, brief tonal confirmation and restrained two-part attention cue. Success, failure and partial result must be distinguishable without forcing users to memorize pitch; visible/text alternatives remain primary. Do not copy sounds from operating systems, games, films or other apps. No text-to-speech for ordinary SFX.

### Manifest
When the asset directory is introduced, maintain assets/audio/manifest.json. Each entry includes event ID, asset ID, path, status (proposed/approved/implemented/deprecated), duration, format, sample rate, channels, peak/loudness review, source/creator, license/SPDX identifier, attribution, review version/date, checksum, fallback behavior, accessibility review and playback test status.

Unknown rights block release. Every semantic event has a no-op fallback.

## 7. Runtime integration contract

- One central SoundService maps semantic IDs to approved local assets.
- UI components emit semantic requests only; they cannot play arbitrary paths or remote media.
- The service checks master mode, category, quiet state, rate limit, dedupe key and asset availability.
- Playback failure is logged at low diagnostic level without secrets and never fails the UI action.
- Preferences live in local settings and migrate safely.
- Activate WebView audio only after explicit opt-in/gesture; test the packaged Windows runtime, not only development mode.
- SoundService cannot invoke tools, elevate permissions, trigger approvals or decide operation success. It only reflects state from the canonical runtime.
- For notifications while hidden, use the app notification policy/platform API; do not keep an audible WebView running just to play sound.
- Do not create a second sound state machine. Use canonical job/connection/notification transitions.

## 8. Accessibility and privacy

- All SFX can be disabled immediately and persistently.
- Sound is never the sole indicator of state, risk, approval, error or completion.
- No audio starts without opt-in; no looping or uninterruptible playback.
- All events have visual labels and accessible announcements.
- Motion and sound settings are independent.
- Do not record microphone input, capture system audio, fingerprint audio devices or send audio telemetry.
- Do not store history of sound-preview interactions.
- Do not request OS permissions beyond those required by the selected playback implementation.

## 9. Validation matrix

Before enabling an event in a release, test:
- Master off, essential-only, all-supported and minimum/maximum volume.
- Fresh install, restart, persistence and invalid-settings fallback.
- Rapid repeated events, duplicate job IDs, retries and simultaneous events.
- App hidden/restored, minimized, system muted and OS quiet/focus mode where supported.
- Missing/corrupt asset, unsupported codec and playback denial.
- SFX failure never changes operation result or blocks keyboard interaction.
- Every event has visible/text alternatives.
- Loudness, clipping, tails and intelligibility on laptop speakers/headphones.
- Packaged Windows build/WebView2 runtime, not only dev mode.

## 10. Release checklist

- [ ] Master SFX defaults off on a clean profile.
- [ ] Preview is user-triggered and cannot autoplay on launch.
- [ ] All requests use one service and semantic event catalog.
- [ ] Shipped files have approved provenance/licenses and manifest entries.
- [ ] No music, ambient loops, surprise audio or repeated polling sounds.
- [ ] Dedupe, throttling and failure fallback tests pass.
- [ ] Every audio event has a visual/text alternative.
- [ ] Preferences persist and master-off is respected under every event.
- [ ] Packaged Windows tests pass.
- [ ] Product owner approves any change to default sound behavior.

## 11. Research references

- [MDN — Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API)
- [MDN — Using Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Using_Web_Audio_API)
- [W3C WCAG 2.2](https://www.w3.org/TR/wcag/)
- [Tauri 2 — Capabilities](https://v2.tauri.app/security/capabilities/)

These references inform implementation and accessibility; they do not certify the finished app.
