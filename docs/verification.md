# Local verification — 2026-10-08

## Repository preparation checks

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
  inspector, and opened the brush studio. The README captures come from this run.
  The launch page and SVG cover were inspected. A browser viewport override did
  not take effect, so this run adds no portrait-device claim; existing Rust layout
  tests cover portrait and Split View dimensions.
- Local Markdown/HTML links resolved. Existing history: 48 blobs scanned for
  high-confidence credential patterns, no matches. Source and patch review found
  no credentials or private logs to publish; ordinary commit authorship remains.

## Earlier engineering checks

Tests exercise the Rust workspace, shared touch controls, input policies and browser workflows. Physical Pencil measurements are the next test pass.

- Base fork: upstream v0.3.0 / `5896f0b`, local host, input and touch-dialog changes through `2b4b87d`.
- Base UI/browser test run: 834 UI unit tests passed, 3 upstream tests ignored; integration tests and doc tests also passed. A GPU render test required running outside the filesystem sandbox. Batched strokes preserve individual pressure/tilt samples, including pen taps.
- Portrait/Split View tests cover New Document, Export and all ten Layer Style effect types without horizontal overflow. A real-widget test covers effect add, enable/disable, cancel, apply and undo. It caught and fixed dialog movement between pointer press/release as the scroll viewport grew.
- Base Clippy with `--no-deps --all-targets -- -D warnings` passed. The broader dependency lint invocation reports an existing `clippy::nonminimal_bool` in the unmodified text crate.
- `cargo xtask layers`: 28 crates, no layering violations.
- Extension: 14 tests passed, including every tool inspector in Split View, all 13 brush sections, portrait/landscape canvas space, channel targeting/duplication/undo through real widgets, stable sheet switching, layer-name synchronization after unlock/undo, touch modifiers, palm suppression, cancellation and the touch signal needed by kinetic scrolling. Extension Clippy passed with warnings denied; wasm check passed with three existing upstream dead-code warnings.
- `cargo xtask wasm` passed. `cargo xtask perf --quick` completed outside the sandbox with GPU access; its explicitly unmeasurable scenarios remain unverified, and this is not an iPad latency measurement.
- Scoped HTTP server boundary test passed for hidden files, symlink escape, traversal, GET/HEAD and wasm MIME. The server was restarted with this boundary active; editor returned 200 and repository metadata returned 404.
- Served separated workspace: Mac Chrome exercised New Document, brush painting, all-tool chooser, Crop settings, alpha-channel creation/targeting, Brush studio and PSD download. Test documents were saved before reload. The optimized build served at 13:49 UTC includes the input work, new sheets and touch dialogs. Physical pen fidelity remains unverified.
- Browser verification of touch dialogs: unlocked Background, added a Stroke effect, applied it, exported PNG through the stacked Export dialog. The downloaded 1920×1080 image was inspected and contains the painted stroke. A stale layer rename field observed during this workflow is fixed and regression-tested locally; its next optimized build is pending.
- iPadOS 26.3: USB Web Inspector connects to the actual local preview. Console opened without messages; this does not establish absence of earlier errors. V's earlier physical preview test was positive. Pressure/tilt ranges and end-to-end Pencil workflows still need measured acceptance.

Release preparation and source may be pushed privately; public visibility awaits V’s go-ahead. No upstream PR has been submitted.
