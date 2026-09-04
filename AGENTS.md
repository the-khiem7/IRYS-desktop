# IRYS Desktop Agent Guide

IRYS is a Tauri 2 + Vue 3 + TypeScript Windows desktop utility for a quiet 30-30-30 eye-break routine. Treat it as a desktop product, not as a website in a window. The app spends most of its life in the tray, so design for repeated use, low interruption, keyboard and pointer interaction, compact windows, recoverable states, and trustworthy timing.

## Architecture boundaries

- Rust owns the clock, timer state, scheduling, persistence, platform checks, and other behavior that must remain reliable while no window is visible.
- Vue owns presentation, interaction affordances, local visual state, and rendering of data returned by Tauri. Do not move the authoritative timer or business state into a browser timer.
- Tauri 2 owns the IPC boundary, windows, tray integration, capabilities, native commands, and platform lifecycle. Verify these behaviors against `src-tauri` and `src-tauri/tauri.conf.json` before describing them as available.
- The main settings surface is in `src-vue`; the break reminder is a separate compact/overlay surface. Keep their jobs and interruption levels distinct.
- Keep design tokens and shared visual rules in the existing Vue/CSS structure, especially `src-vue/styles/theme.css` when the task concerns the application theme. Avoid scattering unrelated magic values through components.

Do not introduce WinUI XAML, Wails APIs, Electron assumptions, or Tauri v1 APIs into this project. For Tauri v2 frontend calls use `@tauri-apps/api/core`; capabilities must explicitly grant the permissions required by a command or plugin. Commands must be registered in the Tauri handler and errors must cross the IPC boundary in a user-recoverable form.

## Design authority order

Use the installed design skills in this order. A later layer may refine an earlier layer, but it must not silently override the product job, state coverage, Windows idiom, accessibility constraints, or verified technical behavior.

1. `ux-design` is the product-flow authority. It defines the user job, user mode, frequency and risk, information architecture, action hierarchy, chosen usability pattern, recovery path, and required states.
2. `windows-design` is the Windows desktop-idiom authority. It defines the desktop archetype, navigation, commands, density, window behavior, keyboard/focus behavior, and Windows interaction expectations.
3. `fluent-foundations` is the Fluent language authority. It guides semantic color, spacing, radius, typography, tonal hierarchy, focus, contrast, and reduced motion without assuming XAML.
4. `fluent-patterns` is the Fluent structure authority. It guides settings, navigation, forms, lists, command bars, empty states, loading, and errors without dictating a framework implementation.
5. `frontend-design` is the visual-direction authority. It adds a product-specific palette, typography, composition, visual hierarchy, assets, and a distinctive point of view. It must not turn IRYS into a generic dashboard or erase the calm, low-interruption job.
6. Vue, CSS, Rust, and Tauri 2 are the implementation authorities. Translate the approved design into existing components, styles, commands, events, windows, capabilities, and runtime states.

Use `cross-platform-design` when the same product is being deliberately translated to Linux, macOS, mobile, or browser surfaces. IRYS may have Linux packages, but do not start cross-platform design merely because Tauri can build on several platforms. Preserve the product job and state coverage, then translate navigation, window behavior, typography, icons, density, and motion per host idiom rather than pixel-cloning Windows.

Use the repository's `tauri-v2` skill for Tauri commands, IPC, events, channels, capabilities, windows, tray, plugins, configuration, and packaging. Read the relevant baseline pack under `docs/baseline/irys-desktop-app/` when a change touches documented architecture, behavior, or release assumptions. Builds and static checks do not prove tray, window, notification, focus, scheduler, or installed-machine behavior; label those boundaries clearly.

## Brainstorming protocol

Brainstorming means making design decisions before writing production UI code. Do not start by generating Vue components, CSS, or a polished screenshot. First produce a compact decision record, call out assumptions, and identify only questions that materially change the direction.

### Phase 1 — Understand the product job

Inspect the repository and brief. For IRYS, begin with the relevant surface:

- Settings window: help a returning desktop user understand the current routine and adjust it quickly.
- Break window: interrupt only when needed, explain the next action immediately, and make skip/snooze/accept behavior unambiguous.
- Tray/background behavior: keep the routine available without forcing the settings window into the user's work.

State the primary job, user mode, frequency, risk, window, and success measure. If a task concerns a different surface, name its product-specific job instead of assuming it is another settings page. If the job is genuinely ambiguous, ask one focused question; otherwise state an assumption and continue.

Output:

```md
Product frame
- Surface: settings / break / tray / shared
- Job: ...
- User mode: first-time / returning / power user
- Frequency and risk: ...
- Platform: Windows desktop via Tauri 2 + Vue 3 + WebView2
- Success looks like: ...
- Assumptions or open questions: ...
```

### Phase 2 — Define the UX flow and states

Use `ux-design` to map the minimum useful path: entry point, first meaningful action, required decision, feedback, success, and recovery. Choose one usability pattern and explain why it fits a quiet, repeated desktop utility. Define the information architecture, primary and secondary actions, keyboard speed path, error prevention, and recovery behavior before visual styling.

At minimum, consider these IRYS states:

- Settings startup/loading while the Rust snapshot is requested.
- Running routine with the next break and current timing visible.
- Paused routine with an obvious resume path.
- Active break with a clear rest action and any allowed skip/snooze action.
- Saving, saved, invalid input, IPC failure, and persistence failure.
- Away/full-screen deferral, pre-warning, and stale or unavailable platform data when applicable.
- Tray-only or hidden-window behavior when the settings window is not open.

Every production surface must account for empty, loading, partial, error, permission, success, and long-running states where they apply. Long-running or deferred work needs visible progress/status, cancellation or stop behavior, retry, and resume semantics when applicable. Preserve user input across validation failures and recoverable errors.

Output a compact UX decision brief:

```md
UX decision brief
- Job: ...
- User mode: ...
- Frequency/risk: ...
- Pattern: settings / command surface / status utility / focused interruption / other
- Primary action: ...
- Secondary actions: ...
- Core path: entry -> action -> feedback -> success
- Recovery path: ...
- Required states: ...
- Keyboard and repeated-use path: ...
- Handoff constraints: ...
```

### Phase 3 — Shape the Windows desktop experience

Use `windows-design` to choose the desktop archetype and composition before choosing controls. For this repository, explicitly record the selected track as `Tauri 2 + Vue 3 desktop WebView2`; do not redirect the project to WinUI XAML, React, or Electron. State the WebView2 fidelity tradeoff where native chrome or controls matter. If a future task changes the shell or target platform, revisit the track instead of assuming it.

Define the window model, navigation model, command model, resize behavior, pane relationships, focus order, keyboard shortcuts, context menus, and density. The existing settings window and break window have different constraints: settings can be resized and revisited; the break surface is compact, topmost, and interruption-sensitive. A design must respect those actual constraints or propose a verified configuration change separately.

Use Windows idioms deliberately. A NavigationView-like structure may be represented by Vue navigation regions; command bars by semantic toolbars; SettingsCard-like groups by accessible settings sections; InfoBar-like feedback by explicit status regions. Do not copy native control names mechanically when the WebView implementation cannot provide the same behavior. Native title-bar, backdrop, tray, always-on-top, transparent-window, focus, notification, and full-screen behavior must be marked `current`, `planned`, `assumed`, or `runtime-verified`.

Output a Windows UI decision brief and small layout sketch:

```md
Windows UI decision brief
- Track: Tauri 2 + Vue 3 + WebView2
- Desktop archetype: settings utility / tray utility / focused interruption
- Window model: ...
- Navigation model: ...
- Command model: ...
- Main regions: ...
- Resize and compact-window behavior: ...
- Keyboard/focus path: ...
- Native-shell capabilities: current / planned / assumed / runtime-verified
- Required platform evidence: ...
```

```text
Settings window
┌──────────────────────────────────────────────────────────────┐
│ IRYS identity / window actions / appearance                   │
├───────────────┬──────────────────────────────────────────────┤
│ sections      │ current routine + settings + status feedback │
│               │                                              │
└───────────────┴──────────────────────────────────────────────┘

Break window
┌──────────────────────────────────────────────┐
│ rest signal / remaining time                  │
│ short explanation                             │
│ [take a break] [snooze or skip if permitted]  │
└──────────────────────────────────────────────┘
```

Do not use `sidebar + identical cards + table` as an automatic answer. If that structure appears, explain why it serves the IRYS job or rewrite it around a calmer status utility, settings rail, compact command surface, or purpose-specific break composition.

### Phase 4 — Apply Fluent foundations and patterns

Use `fluent-foundations` and `fluent-patterns` after the UX and desktop shape are clear. Treat Fluent as a framework-agnostic design language, not as a request to introduce XAML. Translate it into semantic CSS variables, Vue component states, accessible HTML, and verified Tauri window behavior.

Apply this baseline:

- Use semantic Fluent-style tokens for color, spacing, radius, typography, borders, and motion; avoid scattered magic values.
- Use the 4px spacing rhythm and coherent hierarchy. Keep component spacing smaller than section spacing.
- Let neutral surfaces carry most of the interface. Reserve the IRYS brand color for primary actions, selected navigation, the eye mark, and genuine status emphasis.
- Use tonal surfaces and borders to establish hierarchy before shadows. Keep settings and status surfaces calmer than promotional content.
- Use a Windows-appropriate UI typeface and verify the WebView2 fallback stack. Do not use typography to imitate a different platform.
- Give every interactive element a visible focus treatment and keyboard path. Give icon-only controls an accessible name.
- Meet contrast requirements, do not communicate state by color alone, and respect `prefers-reduced-motion`.
- Keep compact controls visually restrained while preserving usable pointer and touch hit areas.

Use Fluent patterns for settings groups, navigation, command actions, empty/loading/error states, and status feedback. Keep validation close to the field or action, expose a retry path, and announce status changes through appropriate semantics such as live regions. Do not treat a Fluent token name as proof that the corresponding native WinUI control exists in a Vue WebView.

### Phase 5 — Establish visual direction

Use `frontend-design` only after the UX and Windows/Fluent decisions exist. Produce a short plan before implementation and critique it against the IRYS subject: healthy visual habits, a calm reminder, and a utility that stays out of the way. The plan must contain:

- A 4–6 color named palette with semantic roles for light and dark appearance.
- Typography choices and roles, including the Windows/WebView2 fallback behavior.
- A layout concept with alignment guidance and an ASCII wireframe.
- A small set of visual principles tied to IRYS and its user context.
- The one memorable visual decision and the surrounding restraint that keeps it useful.

Reject visual defaults that are not justified by the brief: identical rounded cards, decorative gradient washes, arbitrary all-caps labels, excessive shadows, ornamental motion, vague CTA copy, and a generic SaaS dashboard. Use plain product language. A visual distinction is valuable only when it improves recognition, hierarchy, comprehension, or confidence in the next break action.

Before coding, perform a self-critique: identify which choices are specific to IRYS, which could have been generated for any utility, and what was revised. Keep the visual system small enough to maintain across settings and break surfaces.

### Phase 6 — Translate, implement, and inspect

Only after the design direction is selected should implementation begin. Map the approved decisions to Vue components, semantic CSS tokens, explicit state variants, Tauri commands/events, capabilities, and actual window labels. Keep the authoritative timer and state in Rust; keep Vue as a thin, resilient client.

During implementation:

- Build the current-status and primary task path before decorative polish.
- Keep loading, paused, saving, error, break, skip, snooze, and success states explicit in the component model.
- Preserve keyboard order, focus restoration, compact-window behavior, and user input during errors.
- Use semantic HTML and ARIA only where needed; prefer native semantics over unnecessary custom widgets.
- Use Tauri v2 APIs and capability files verified in this repository. Do not use v1 imports or undocumented permissions.
- Keep light/dark appearance, reduced motion, WebView2 fallback behavior, and narrow-window behavior in the design rather than treating them as follow-up work.

Inspect the rendered result, not only the source. Exercise first-run/loading, returning-user, pause/resume, break acceptance, skip/snooze, save failure, keyboard-only, resize, hidden/tray, and relevant full-screen/away scenarios. Capture screenshots or other concrete evidence when available. Run `npm run typecheck` or `npm run build` for fast feedback and `npm run verify` for the repository's Docker-backed frontend and Rust checks. A green build does not prove tray, window, scheduler, notification, focus, or installed-machine behavior; report those as verified only after the corresponding runtime check.

## Required brainstorming handoff

For an IRYS UI/UX brainstorming task, return the following in order:

1. Product frame and assumptions.
2. UX decision brief, chosen pattern, core path, recovery path, and state matrix.
3. Windows desktop archetype, window/navigation/command model, and Tauri capability boundary.
4. Fluent foundations and pattern decisions, including semantic tokens and accessibility constraints.
5. Visual direction, palette, typography, layout sketch, and self-critique.
6. Vue/CSS/Rust/Tauri implementation map, evidence plan, open questions, and the next decision needed.

Do not present implementation details as design decisions. Mark important items as `current`, `planned`, `assumed`, or `runtime-verified` whenever their status could be misunderstood. For brainstorming-only requests, stop at the selected direction and handoff unless the user explicitly asks for implementation.

## Anti-patterns

- Designing a marketing landing page when the user needs a quiet, repeated desktop routine.
- Starting with cards, gradients, colors, or components before identifying the user's job.
- Moving timer authority into a Vue/browser timer that can be throttled or disappear with the window.
- Treating Tauri 2 + Vue as WinUI XAML or importing Tauri v1 APIs into a v2 project.
- Letting visual novelty override interruption level, navigation, focus, contrast, error recovery, or state coverage.
- Copying a mobile bottom tab bar, Android FAB, or browser-only pattern into a Windows desktop utility without a clear reason.
- Using one generic sidebar/card shell for both the settings window and the break reminder.
- Hiding paused, deferred, failed, stale, permission, or long-running behavior.
- Claiming native window, tray, notification, or focus behavior from a mockup without runtime verification.
- Adding dependencies or Rust/backend changes merely to support a visual preference.

## Release versioning guard

Never package or publish an installer from changed source with a version that has already been released. Before creating a distributable build, increment the version consistently in `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`; the generated installer filename and GitHub release tag must use that new version. Do not overwrite or replace an existing release artifact with different application contents under the same version.

The goal is a calm, distinctive, Fluent-informed Windows desktop experience whose timing and behavior remain trustworthy because the approved design survives the Tauri 2 + Vue implementation boundary.
