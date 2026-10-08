# Verification record — 2026-10-08

Entries identify the revision and environment tested. Historical implementation
snapshots are retained below; the roadmap describes the current direction.

## Command-shelf integration checks

Implementation commit `2ba5edf` is integrated as `989c3ae`, with the publishing
README, licensing and build scripts preserved. Fresh checks in the publishing
checkout passed formatting, all 26 tests, and native plus WebAssembly Clippy
with crate warnings denied. The three existing upstream Wasm dead-code warnings
are unchanged. The full-history Gitleaks scan found no secrets.
The optimized Wasm build and locked license packaging also passed, completing
at 15:26 UTC. No dependency versions, license policy or engine patch changed.

The command/voice browser acceptance below was performed by the implementation
session on the same source batch. It is distinct from microphone acceptance on
the physical iPad. Engine tree and the five-patch series are unchanged.

## Public readiness checks

Fresh GitHub clone of `82be479`, including the Layers stack and navigation work
from `5eae0db`, with a new, isolated Cargo target directory:

- `setup.sh` fetched public upstream and applied all five patches. The resulting
  tree matched `7545cfe69924ec4f48fee63aa61ebf3ff82c48ac`.
- `cargo fmt --check`, `cargo test --locked` (19 passed), and
  `cargo clippy --locked --all-targets --no-deps -- -D warnings` passed.
- `build.sh` produced the optimized WebAssembly distribution at 14:46 UTC,
  with three existing upstream dead-code warnings. Shell syntax checks passed.
- `cargo-about 0.9.2 --locked --fail` resolved all 240 runtime/build packages.
  Every package is represented in the generated license page. Required upstream,
  asset and translation notices are present and served over HTTP; embedded font
  terms and the IJG acknowledgement are included. No local manifest paths or
  restricted ArtCraft logo SVGs appear in the distribution.
- A deliberately missing license tool caused the build to fail while preserving
  all 16 previously served files byte-for-byte.
- `uv run --no-project --python 3.12 python -m unittest test_server` passed.
- Chrome imported the public-domain Great Wave, displayed the new Layers stack
  and grouped tool rail, and opened Brush studio. README screenshots were
  refreshed from this build; provenance is in [visuals](visuals.md).
- Gitleaks 8.30.1 scanned all refs and full reachable history with zero findings.
  File inventory and screenshot metadata were also reviewed for private content.
- Focused review of setup, engine pinning, notice generation and build publication
  found no unresolved release-preparation defects.

## Initial repository preparation checks

Fresh checks on the `06495ac` extension snapshot plus repository preparation:

- `setup.sh` cloned public upstream at the pinned revision and applied all four
  patches. The resulting Git tree matched `9905aaa3fcc69c678788d24132010d2f1c08037e`.
  A changed sibling checkout was refused without modifying its work.
- `cargo test --locked`: 14 passed. `cargo clippy --locked --all-targets --no-deps
  -- -D warnings` and `cargo fmt --check` passed.
- `build.sh`: optimized WebAssembly build passed (Rust 1.96.0, Trunk 0.21.14),
  with three existing upstream dead-code warnings. Output includes licenses,
  NOTICE, upstream attribution and referenced asset license texts.
- `uv run --no-project --python 3.12 python -m unittest test_server`: passed.
- Fresh Chrome launch imported the public-domain Great Wave, displayed the layer
  inspector, and opened the brush studio. The initial README captures came from this run.
  The launch page and SVG cover were inspected. A browser viewport override did
  not take effect, so this run adds no portrait-device claim; existing Rust layout
  tests cover portrait and Split View dimensions.
- Local Markdown/HTML links resolved. Existing history: 48 blobs scanned for
  high-confidence credential patterns, no matches. Source and patch review found
  no credentials or private logs to publish; ordinary commit authorship remains.

## Earlier engineering checks

Tests exercise the Rust workspace, shared touch controls, input policies and browser workflows. Physical Pencil measurements are the next test pass.

- Base fork: upstream v0.3.0 / `5896f0b`, local host, input and touch-dialog changes through `af949b1`.
- Base UI/browser test run: 834 UI unit tests passed, 3 upstream tests ignored; integration tests and doc tests also passed. A GPU render test required running outside the filesystem sandbox. Batched strokes preserve individual pressure/tilt samples, including pen taps.
- Portrait/Split View tests cover New Document, Export and all ten Layer Style effect types without horizontal overflow. A real-widget test covers effect add, enable/disable, cancel, apply and undo. It caught and fixed dialog movement between pointer press/release as the scroll viewport grew.
- Base Clippy with `--no-deps --all-targets -- -D warnings` passed. The broader dependency lint invocation reports an existing `clippy::nonminimal_bool` in the unmodified text crate.
- `cargo xtask layers`: 28 crates, no layering violations.
- Extension: 19 tests passed, including every tool inspector in Split View, all 13 brush sections, portrait/landscape canvas space, channel targeting/duplication/undo through real widgets, stable sheet switching, layer-name synchronization after unlock/undo, image/mask stroke targeting and independent undo, mask-only view exit, selection-to-mask creation after alpha-channel editing, fixed stack/footer geometry with 25 layers across three viewport sizes, tool-rail coverage, hierarchical menu navigation into Levels, touch modifiers, palm suppression, cancellation and the touch signal needed by kinetic scrolling. Extension Clippy passed with warnings denied; wasm check passed with three existing upstream dead-code warnings.
- `cargo xtask wasm` passed. `cargo xtask perf --quick` completed outside the sandbox with GPU access; its explicitly unmeasurable scenarios remain unverified, and this is not an iPad latency measurement.
- Scoped HTTP server boundary test passed for hidden files, symlink escape, traversal, GET/HEAD and wasm MIME. The server was restarted with this boundary active; editor returned 200 and repository metadata returned 404.
- Served separated workspace: Mac Chrome exercised New Document, brush painting, all-tool chooser, Crop settings, alpha-channel creation/targeting, Brush studio and PSD download. Test documents were saved before reload. The optimized build served at 13:49 UTC includes the input work, new sheets and touch dialogs. Physical pen fidelity remains unverified.
- Browser verification of touch dialogs: unlocked Background, added a Stroke effect, applied it, exported PNG through the stacked Export dialog. The downloaded 1920×1080 image was inspected and contains the painted stroke. A stale layer rename field observed during this workflow is fixed and regression-tested; the fix was built at 13:56 UTC.
- V's design review prompted a compact document bar, icon inspector tabs, all-tool scroll rail and hierarchical menu with optional search. Mac browser acceptance verified New Layer from the pinned footer, image painting, mask creation and painting, separate thumbnail targeting, the properties sheet, mask-only viewing and return to composite, then PSD save (296,746 bytes). The final optimized build at 14:33 UTC was reloaded: active tool/panel indicators are visible, and the saved PSD reopened with separate image/mask thumbnails and the masked strokes intact. Physical iPad acceptance of this redesign remains open. The voice command layer is planned, not implemented.
- iPadOS 26.3: USB Web Inspector connects to the actual local preview. Console opened without messages; this does not establish absence of earlier errors. V's earlier physical preview test was positive. Pressure/tilt ranges and end-to-end Pencil workflows still need measured acceptance.

A shared Cargo target collision with the OSS review copy initially linked stale UI despite a new output hash. Build output is now isolated under this checkout’s target directory; the browser visibly shows the new rail and Layers stack. The existing layer-reveal helper is exposed by the base fork; its two tests and base lint/layer checks pass.

No upstream PR was submitted as part of these checks.

## Historical command shelf and browser speech adapter (superseded)

The historical `2ba5edf` implementation added a bottom command shelf and browser speech adapter, since replaced by on-demand search and native keyboard transcription. That revision passed 26 tests and native/Wasm Clippy; it never established physical iPad microphone acceptance. Current behavior is described below.

Optimized preview built and staged at 15:09 UTC. Mac Chrome reopened the saved image/mask PSD, searched "switch to eraser tool" without switching, switched on Enter, restored Brush with the tool-revert action, and searched Levels. Choosing Image / Adjustments opened the shared Levels dialog; Cancel returned to the unchanged document. The native microphone overlay disappeared while the dialog was open and returned afterward. This validates command dispatch and overlay placement, not microphone recognition on either device.

## Canonical integration check

Merge `5afa515` retains the release preparation through `29bd591` and command implementation `2ba5edf`, preserving both histories. Engine `af949b1` remains tree `7545cfe69924ec4f48fee63aa61ebf3ff82c48ac`; the five-patch series, `upstream.env`, lockfile, presentation and notice-generation scripts are unchanged from the release candidate.

Fresh checks in the implementation checkout's isolated target passed: formatting, all 26 tests plus binary/doc-test targets, native Clippy with warnings denied, Wasm check, server boundary test, and the canonical optimized build. Dependency notice generation passed with `cargo-about 0.9.2 --locked --fail`; all 240 runtime/build packages remain represented. The combined distribution was staged at 15:20 UTC. The running port 4876 served its editor and dependency license page; the served license page matched the generated file byte-for-byte. Repository visibility remained private. Device speech and measured Pencil acceptance remain open.

## Native command entry, source picking and Pencil scrolling correction

Search now opens from the top magnifier in a modal without changing the canvas or inspector geometry. Text comes from a normal input and the native keyboard; there is no application microphone or recognizer. Results separate names from menu paths and offer category filters. Bare “blur” remains ambiguous; “blur tool” selects the tool, while Box, Gaussian, Motion and Lens Blur dispatch their distinct shared dialogs. The foreground/background colours remain visible in the pinned rail, with separate editing targets.

The selection context menu and Layer properties expose Layer via Copy/Cut, inverse and deselect. Copy/Cut operate on selected pixels of the active layer. Clone Stamp and Healing have a one-shot Set source action: tap a source on the canvas, then the next Pencil stroke paints. Source picking accepts a finger as well; ordinary single-finger canvas input remains navigation-only. Engine errors are now visible in the workspace.

Extension scroll areas explicitly accept pointer drags, including the first interaction after reload. Pencil contacts starting over UI also report force-less touch lifetimes, allowing the shared dropdowns and dialogs to enable their existing kinetic scrolling. UI ownership is fixed at press-down and ends on release/cancellation; those contacts do not feed brush pressure samples. Canvas Pencil input retains its original pressure/tilt route. Inspector and sheet scrollbars use a solid 14-point track with a gap from controls.

Fresh extension checks pass: 32 tests, formatting, native Clippy with warnings denied, and Wasm check (three unchanged upstream dead-code warnings). Regression evidence includes fresh-session layer dragging without clicks/edits, shared scrolling after a Pencil UI tap, cancellation and canvas/UI ownership classification, source pick followed by cloned pixels and undo, selection copy/cut pixels and undo, four distinct blur dialogs, visible colour targets, and unchanged canvas bounds when search opens at portrait/Split View/keyboard-sized viewports. These are synthetic tests, not measured physical Pencil or native-dictation acceptance. Optimized browser acceptance for this correction is recorded after the build below.
