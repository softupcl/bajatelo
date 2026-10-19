# AGENTS.md — REA ClipGrab Desktop Clone

## 1. Project mission

Build an independent desktop application inspired by the observable user experience of ClipGrab: a graphical tool for analyzing supported media URLs, downloading authorized content, and converting local media files.

Use **REA (Reverse Engineering Approach)** as a disciplined process:

1. Investigate the reference and collect evidence.
2. Separate verified facts from hypotheses and unknowns.
3. Produce a functional and technical specification.
4. Implement the application incrementally in a separate project.
5. Compare the implementation with documented evidence.
6. Fix discrepancies and run regression tests.

The reference is for behavioral and usability analysis. Do not copy proprietary source code, trademarks, logos, artwork, or other assets without permission. Create an original product identity and independent implementation.

## 2. Mandatory agent behavior

- Read this file before making changes and inspect the repository before proposing an implementation.
- Follow the existing architecture and conventions where they are already established. Do not replace the stack or restructure the project without explaining why and obtaining approval when the change is substantial.
- Work in small, reviewable steps. Prefer one feature or coherent task per change.
- Before coding, state the intended change, relevant files, assumptions, and acceptance criteria.
- Do not claim a command, build, test, download, conversion, or comparison succeeded unless it was actually run and its result checked.
- Never invent REA findings, screenshots, URLs tested, platform support, format availability, or application behavior.
- Keep documentation synchronized with behavior and implementation.
- Preserve user changes. Do not overwrite, delete, reset, or reformat unrelated files.
- Ask before destructive actions, major dependency changes, publishing, signing, or releasing artifacts.
- If the repository is empty or its stack differs from the proposed stack, inspect it first and explain the proposed next step rather than blindly scaffolding over it.
- When blocked by missing software, permissions, access, or unavailable reference evidence, document the blocker and continue only with independent work that remains valid.

## 3. Proposed technology baseline

The intended baseline, subject to verification during the specification phase:

- **Desktop shell:** Tauri 2
- **Frontend:** Vue 3 + TypeScript
- **Desktop/backend services:** Rust
- **Media URL analysis and download:** yt-dlp, where compatible and permitted
- **Media inspection and conversion:** FFmpeg and FFprobe
- **Unit/component tests:** Vitest and the project’s chosen Vue testing tools
- **End-to-end tests:** Playwright where suitable for the selected Tauri setup

Verify current compatibility, installation requirements, licensing, platform support, and packaging implications before locking versions. Use the package manager and lockfile already established by the repository. Do not add a second package manager.

If the repository already uses another approved stack, do not migrate it without explicit approval.

## 4. REA investigation protocol

### 4.1 Allowed inputs

The REA process may inspect, when available and authorized:

- The official ClipGrab website and public documentation.
- A running reference application that the user is authorized to inspect.
- Screenshots and recordings supplied by the user.
- Public source code or packages whose licenses permit the intended inspection.
- Observable screens, navigation, controls, validation messages, loading states, errors, and outputs.

Official reference starting point: https://clipgrab.org/

Do not assume the original application is installed or that source code, internal APIs, Electron bundles, debug interfaces, or symbols are available. Detect what is actually accessible.

### 4.2 Evidence rules

Every meaningful finding must be identified as one of:

- **Verified:** directly observed or supported by a cited source/evidence artifact.
- **Inferred:** a reasonable technical interpretation, with the supporting evidence and uncertainty stated.
- **Unknown:** not verified yet.
- **Not applicable:** irrelevant to the target implementation, with a short reason where useful.

Never present an inference as a fact. Record evidence paths, URLs, test conditions, and date where practical. Do not include secrets, tokens, personal data, browser cookies, or private session material in the evidence.

### 4.3 Investigation must not modify the reference

- Do not alter the original application or source directory during the investigation phase.
- Do not run destructive commands against the reference.
- Do not attempt to bypass DRM, paywalls, authentication, access controls, or platform restrictions.
- Do not extract or reuse proprietary assets as clone assets.
- If REA tools are missing, report that fact and use only available, authorized observation methods. Do not pretend unavailable tools were used.

### 4.4 REA deliverables

Maintain these files as applicable:

- `docs/rea/01-research.md`
- `docs/rea/02-feature-inventory.md`
- `docs/rea/03-user-flows.md`
- `docs/rea/04-ui-inventory.md`
- `docs/rea/05-technical-findings.md`
- `docs/rea/06-open-questions.md`

Each screen or flow entry should describe its trigger, input, visible state, user action, expected observable result, failure states, and evidence source. Mark unknown behavior explicitly.

## 5. Product scope

### 5.1 MVP features

1. URL input with validation and clear error messages.
2. Media metadata inspection for supported URLs.
3. Display of formats and qualities actually reported by the source.
4. User selection of available video/audio options.
5. Download task queue with distinct task IDs.
6. Download progress, status, and errors where the underlying engine exposes them.
7. Cancel and retry operations with correct process cleanup.
8. Local file conversion through FFmpeg/FFprobe.
9. Initial target formats: MP4 (H.264/AAC where available), MP3, and keeping the original file.
10. Native file/folder selection, conflict handling, and open-file/open-folder actions.
11. Local operation history and basic preferences.
12. Dependency diagnostics for missing or incompatible media tools.
13. Spanish-language UI for the first release.
14. Tests and user-facing documentation.

### 5.2 Later features (do not assume they are MVP)

- Additional formats and advanced conversion profiles.
- Subtitle handling, playlists, batch imports, or scheduled tasks.
- Additional platform-specific integrations.
- Automatic updates, telemetry, cloud synchronization, or account systems.

Add later features only when supported by a documented requirement and an approved backlog item.

### 5.3 Compatibility expectations

Recognizing a URL does not guarantee that its content can be downloaded. Platform support, formats, access requirements, rate limits, and extractor behavior can change. The UI must communicate the actual result of each operation rather than promise universal support.

Only support content the user is authorized to download. Respect applicable laws, platform terms, and the licenses of all dependencies. Do not implement DRM circumvention or bypass access controls.

## 6. Architecture principles

Keep responsibilities separate:

- **UI:** screens, components, user input, status display, accessible interactions.
- **Application layer:** task lifecycle, validation, queue coordination, preferences, and user-visible errors.
- **Media services:** isolated adapters for metadata inspection, downloading, probing, and conversion.
- **Desktop integration:** file dialogs, filesystem access, process lifecycle, notifications if required, and permitted shell actions.
- **Persistence:** local preferences and history, using a minimal documented schema.

Do not put download or conversion logic directly inside Vue components. Keep the UI independent from the implementation details of yt-dlp and FFmpeg.

Suggested layout, to be adapted after repository inspection:

```text
src/
  components/
  features/
    downloads/
    conversion/
    history/
    settings/
  layouts/
  pages/
  services/
  types/
src-tauri/
  src/
    commands/
    models/
    services/
  capabilities/
docs/
  rea/
  spec/
  qa/
tests/
```

Do not create unnecessary abstractions. Prefer typed contracts and small services with clear responsibilities.

## 7. Security and process execution

- Never construct shell command strings by concatenating user-controlled input.
- Invoke external tools through process APIs with separate, validated arguments.
- Validate protocols and URL inputs; reject unsupported schemes such as `file:`, `javascript:`, and arbitrary local paths when a remote URL is expected.
- Apply least privilege to Tauri capabilities and filesystem access.
- Ask the user to select destination directories through permitted native dialogs; validate the destination before writing.
- Prevent silent overwrites. Use an explicit conflict policy and communicate it to the user.
- Do not execute commands, scripts, or binaries downloaded from media URLs.
- Do not store browser cookies, credentials, or access tokens by default. Do not add authenticated-download support without a separately reviewed security design.
- Do not log secrets, full credential-bearing URLs, private local paths unnecessarily, or sensitive process arguments.
- Handle child-process exit, cancellation, timeouts, and cleanup explicitly.
- Do not expose arbitrary command execution to the frontend.
- Treat filenames and metadata returned by remote sources as untrusted input. Sanitize names and guard against path traversal.
- Avoid writing partial files as if they were completed outputs. Use temporary files or a clear incomplete-state policy where practical.

## 8. Download task lifecycle

Use a documented state model, such as:

- `queued`
- `inspecting`
- `ready`
- `downloading`
- `completed`
- `failed`
- `cancelled`

Adapt states only when justified by the implementation. Define allowed transitions and test them. A task must not be marked completed merely because the process started or produced partial output.

For each task, track only necessary fields, such as ID, source URL (redacted where needed in logs), title if available, selected format, destination, progress if available, timestamps, and error details. Do not persist secrets.

Ensure cancellation actually terminates the associated process and cleans up listeners/resources. Retrying must not create duplicate active processes for the same task.

## 9. Conversion requirements

- Use FFprobe to inspect media when needed and FFmpeg for conversions.
- Validate input existence and output destination before starting.
- Report progress only when it can be derived reliably; otherwise show an indeterminate progress state.
- Check the actual process exit code and verify that the output exists and is non-empty before reporting success.
- Preserve the original input unless the user explicitly requests deletion in a separately confirmed operation.
- Handle missing codecs, corrupt inputs, disk-space errors, cancellation, and output-name conflicts.
- Never delete source files automatically after conversion.
- Use user-owned or otherwise authorized test media.

## 10. UI and accessibility

- Use a clear desktop layout with a primary download workflow and separate conversion, history, and settings views.
- Include loading, empty, success, partial, cancelled, and error states where relevant.
- Do not use fake progress or hard-coded metadata to represent real operations.
- Provide actionable error messages and avoid exposing raw stack traces to ordinary users.
- Ensure keyboard navigation, visible focus, sensible labels, adequate contrast, and accessible status announcements.
- Use original branding and assets. Treat the reference as evidence for interaction patterns, not a source of assets to copy.
- Keep language consistent in Spanish; centralize user-facing strings if practical.

## 11. Testing policy

Add tests at the same time as each feature. Use fakes/mocks for unit tests and authorized fixtures for integration tests.

Minimum coverage areas:

- URL validation and unsupported protocols.
- Metadata parsing and missing fields.
- Queue behavior and legal state transitions.
- Progress event handling.
- Cancellation, retries, and process cleanup.
- Missing media tools and tool-version diagnostics.
- Conversion success and failure.
- Output conflicts, invalid paths, and sanitized filenames.
- History persistence and preferences.
- UI empty/loading/error/completed states.

Do not make automated tests depend on a third-party platform being available unless the test is explicitly an optional integration test. Avoid downloading real social-media content in the default test suite.

Run relevant tests after each coherent change. Before release, run the complete available test/build suite and report the exact commands and results.

## 12. REA differential validation

After implementing a feature:

1. Find the relevant requirement and REA evidence.
2. Execute the same observable user flow in the clone.
3. Compare behavior and visible states, not internal implementation details.
4. Record differences and classify their severity.
5. Fix reproducible defects and add regression tests.
6. Update the report with evidence.

Maintain:

- `docs/qa/differential-report.md`
- `docs/qa/regression-results.md`

Use statuses such as `passed`, `failed`, `blocked`, and `not-tested`. Do not mark a test passed without running it. Do not force a visual match where the evidence is missing; record the unknown instead.

## 13. Specification and backlog

Before substantial implementation, maintain:

- `docs/spec/requirements.md`
- `docs/spec/architecture.md`
- `docs/spec/data-model.md`
- `docs/spec/error-catalog.md`
- `docs/spec/test-plan.md`
- `docs/spec/mvp-backlog.md`

Each backlog item should include:

- ID and concise title.
- User value.
- Dependencies.
- Acceptance criteria that can be verified.
- Tests required.
- Status and blockers.

Implement in dependency order. Prefer thin vertical slices that connect UI, service, and tests rather than building every screen before any real functionality exists.

## 14. Dependency and licensing policy

- Verify the licenses and redistribution conditions for Tauri, Rust crates, frontend packages, yt-dlp, FFmpeg, and any bundled binaries.
- Do not assume FFmpeg has one universal license configuration; inspect the actual build and codec configuration being distributed.
- Document whether dependencies are installed by the user, downloaded during setup, or bundled with the app.
- Do not silently bundle external executables or change their source.
- Pin compatible versions where appropriate and preserve lockfiles.
- Review security advisories and update compatibility before preparing a release.

## 15. Definition of done

A feature is complete only when all applicable points are true:

- Requirements and acceptance criteria are documented.
- Implementation follows the approved architecture.
- Loading, success, cancellation, and failure states are handled.
- Input validation and security checks are present.
- Automated tests were run and their results recorded.
- No fake success state or hard-coded operational result remains.
- Documentation and backlog are updated.
- Any limitations or untested cases are stated explicitly.

A release is not complete until the relevant production build and packaging steps have actually been run and verified on the claimed platform.

## 16. Suggested execution sequence

Follow this sequence unless repository evidence justifies a documented change:

1. Inspect the repository, environment, available REA skill/tools, and reference evidence.
2. Complete `docs/rea/` without modifying the reference.
3. Approve requirements, architecture, error catalog, test plan, and backlog.
4. Scaffold or adapt the project without overwriting existing work.
5. Build the app shell and navigation.
6. Implement URL validation and metadata inspection.
7. Implement real downloads and queue lifecycle.
8. Implement local conversion.
9. Add history, preferences, and diagnostics.
10. Run differential validation and regression tests.
11. Review licensing, security, packaging, and release readiness.

At the end of each phase, summarize: files changed, commands run, test results, unresolved questions, and the next recommended task. Do not automatically proceed into a materially different phase without reviewing the current phase’s results.
