# Local verification — 2026-10-08

This is an in-progress port. Passing these checks does not establish complete iPad or Adobe Photoshop parity.

- Base fork: upstream v0.3.0 / `5896f0b`, local host, touch and per-point pen changes through `953dce7`.
- Base UI/browser test run: 832 UI unit tests passed, 3 upstream tests ignored; integration tests and doc tests also passed. A GPU render test required running outside the filesystem sandbox. Batched strokes preserve individual pressure/tilt samples, including pen taps.
- New portrait/Split View dialog regression passed after fixing its font setup.
- Base Clippy with `--no-deps --all-targets -- -D warnings` passed. The broader dependency lint invocation reports an existing `clippy::nonminimal_bool` in the unmodified text crate.
- `cargo xtask layers`: 28 crates, no layering violations.
- Extension: 12 tests passed, including every tool inspector in Split View, all 13 brush sections, portrait/landscape canvas space, channel targeting/duplication/undo through real widgets, touch modifiers, palm suppression, cancellation and the touch signal needed by kinetic scrolling. Extension Clippy passed with warnings denied; wasm check passed with three existing upstream dead-code warnings.
- `cargo xtask wasm` passed. `cargo xtask perf --quick` completed outside the sandbox with GPU access; its explicitly unmeasurable scenarios remain unverified, and this is not an iPad latency measurement.
- Scoped HTTP server boundary test passed for hidden files, symlink escape, traversal, GET/HEAD and wasm MIME.
- Served separated workspace: Mac Chrome exercised New Document, brush painting, Brush inspector, all-tool chooser, Crop settings and PSD download. The test document was saved before reload. Per-point/coalesced pen changes and the latest brush studio, channels, paths, layer controls and scrolling still await an optimized build and physical browser acceptance.
- iPadOS 26.3: USB Web Inspector connects to the actual local preview. Console opened without messages; this does not establish absence of earlier errors. V's earlier physical preview test was positive. Pressure/tilt ranges and end-to-end Pencil workflows still need measured acceptance.

Builds stay local; no pushes or PRs until V reviews repository copy. Implementation/validation logs are local temporary operational evidence, not publication artifacts.
