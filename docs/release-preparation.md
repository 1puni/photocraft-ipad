# Public source release

The maintainer authorized publication on 8 October 2026. The v0.1 source release
includes the iPad workspace, five reproducible PhotoCraft patches, a local preview
and a standalone segmentation evaluation. Hosted demos remain a separate project.

## Release checks

- [x] **Licensing:** MIT OR Apache-2.0 code, preserved upstream notices, independent
  visual identity and complete asset attribution. [License review](licensing.md).
- [x] **History:** full reachable history scanned with Gitleaks and reviewed for
  private material; original authorship and development history are preserved.
- [x] **Presentation:** public-facing copy reviewed, research separated from
  shipped features, and superseded custom voice promotion removed.
- [x] **Baseline verification:** fresh-clone setup/build, editor tests, native and
  Wasm lint, server boundaries and license packaging passed. [Test record](verification.md).
- [x] **Publication:** opened on 8 October 2026 at revision `f08c26a`. Anonymous
  GitHub API and README access verified; private vulnerability reporting enabled.

## Verified development snapshot — 8 October 2026

Public source at
[`872fac9`](https://github.com/1puni/photocraft-ipad/commit/872fac9ca3e62ef57021425215a96c30e29ee1d9)
includes the native-keyboard correction, Colour studio, docked mask controls,
readable nested layer rows, welcome attribution, and single-layer/group Arrange
with edge autoscroll. The README through `080d724` presents these workflows with
attributed screenshots; [visual provenance](visuals.md) identifies their build.

The merged source, build configuration and pins match tested implementation
`9d60555`. All **62 tests**, formatting, native Clippy with warnings denied and
Wasm check passed. The publication checkout independently passed the 62 tests.
The optimized build and locked dependency notices completed at 18:20 UTC.
Chrome acceptance on a synthetic 29-layer PSD covered normal row scrolling,
handle-based movement at a scrolled position and Undo, with no console errors.
Continuous edge holds, offscreen targeting, cancellation and short portrait docks
have synthetic UI coverage. Physical Pencil acceptance remains on the
[roadmap](ipad-port.md), along with multi-layer dragging and the remaining port.

The engine tree, five patches and dependency lockfile are unchanged. Detailed
feature and browser evidence stays in the [test record](verification.md).

## First public development batch

The first public development batch landed on 8 October 2026 in
[`66c5572`](https://github.com/1puni/photocraft-ipad/commit/66c5572b733c9a01d14684c249ecc2789522c3cd),
preserving the release and research history. On-demand top-bar search replaces
the permanent shelf and custom speech recognizer; text entry uses the native
keyboard. The batch also adds pinned foreground/background colour chips,
distinct blur choices, one-shot Stamp/Healing source selection, Layer via Copy/Cut
and explicit panel drag scrolling.

The implementation passed 32 tests, formatting, native Clippy, Wasm checks,
the optimized build and locked dependency notice generation. Chrome acceptance
covered stable search, cloned pixels and undo, selection copying, and a saved
three-layer PSD reopened with its contents intact. The engine pin, five patches
and dependency lockfile are unchanged. The [test record](verification.md#native-command-entry-source-picking-and-pencil-scrolling-correction)
contains the evidence; physical Pencil and native keyboard dictation acceptance
remain on the roadmap.

## Development in the open

Use `main` for verified batches and topic branches for ongoing work. Fetch before
pushing; preserve other contributors' work and merge diverged histories normally.
Each checkout uses its own `target/` directory. Keep `upstream.env` and the five
engine patches synchronized, and regenerate dependency notices when dependencies
change. The engine baseline is `5896f0b`; the patched tree is
`7545cfe69924ec4f48fee63aa61ebf3ff82c48ac` (equivalent to `af949b1`).

[The roadmap](ipad-port.md) owns upcoming workflow and physical-device acceptance.
[Security reports](../SECURITY.md) have a private reporting route. Upstream
contributions follow the [collaboration notes](upstream.md).
