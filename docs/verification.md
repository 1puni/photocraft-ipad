# Local verification — 2026-10-08

This is an in-progress port. Passing these checks does not establish complete iPad or Adobe Photoshop parity.

- Base fork: upstream v0.3.0 / `5896f0b`, local host and touch changes through `cf6c45e`.
- Base UI/browser test run: 828 UI unit tests passed, 3 upstream tests ignored; integration tests and doc tests also passed. A GPU render test required running outside the filesystem sandbox.
- New portrait/Split View dialog regression passed after fixing its font setup.
- Base Clippy with `--no-deps --all-targets -- -D warnings` passed. The broader dependency lint invocation reports an existing `clippy::nonminimal_bool` in the unmodified text crate.
- `cargo xtask layers`: 28 crates, no layering violations.
- Extension: 9 tests passed, including every tool inspector in Split View, portrait/landscape canvas space, commands/undo, touch modifiers, palm suppression and cancellation. Extension Clippy passed with warnings denied.
- Scoped HTTP server boundary test passed for hidden files, symlink escape, traversal, GET/HEAD and wasm MIME.
- First separated workspace build: Mac Chrome loaded it; New Document, brush painting and the Brush inspector were exercised visually. The current editor remains open with a test stroke. Subsequent Tool inspector changes await the next optimized build and browser test.
- iPadOS 26.3: USB Web Inspector connects to the actual local preview. Console opened without messages; this does not establish absence of earlier errors. V's earlier physical preview test was positive. Pressure/tilt ranges and end-to-end Pencil workflows still need measured acceptance.

Builds stay local; no pushes or PRs until V reviews repository copy. Implementation/validation logs are local temporary operational evidence, not publication artifacts.
