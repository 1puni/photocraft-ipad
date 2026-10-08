# Public source release

The maintainer authorized publication on 8 October 2026. The v0.1 source release
includes the iPad workspace, seven reproducible PhotoCraft patches, a local preview
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
[`9d1b1b0`](https://github.com/1puni/photocraft-ipad/commit/9d1b1b094cd1b60b1f7afe24b22a7d22e5a14816)
includes the native-keyboard correction, Colour studio, docked masks and layer properties,
readable nested layer rows, welcome attribution, and layer/group/selected-set
Arrange with edge autoscroll, and pre-edit layer selection restoration on Undo.
The README presents these workflows with
attributed screenshots; [visual provenance](visuals.md) identifies their build.

The merged source, build configuration and pins match tested implementation
`28afe7a`. All **73 extension tests**, formatting,
extension native Clippy with warnings denied and Wasm check passed.
The publication checkout independently passed all 73 extension tests.
The optimized build and locked dependency notices completed at 19:05 UTC.
Chrome repeated the original opacity fixture: select a different layer, drag
opacity from 100% to 39%, Undo to 100%, then Redo to 39%. The selected layer stayed
correct throughout, with no console errors. This resolves the property-edit Undo
selection limitation in the previous snapshot. Coalesced updates retain the
gesture's initial Undo target; selecting a layer alone creates no history step.
Continuous edge holds, offscreen targeting, cancellation and short portrait docks
have synthetic UI coverage. Physical Pencil acceptance remains on the
[roadmap](ipad-port.md), along with the remaining port.

The seventh patch captures live pre-edit selection in the shared successful-edit
path. A fresh local clone in the publication checkout independently reproduced
the exact engine tree by applying all seven patches to the unchanged upstream
baseline. The first six patches, dependencies, lockfile and build scripts are
unchanged.

Engine verification passed 783 non-ignored unit tests, integration/doc targets,
seven history tests, architecture layering, Wasm and the opt-in adversarial-command test. Strict engine
Clippy still reports pre-existing `nonminimal_bool` and `manual_range_contains`
findings in unchanged code; it passes with only those two categories allowed.
Detailed results are in the [test record](verification.md#undo-keeps-the-pre-edit-layer-selection).

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
three-layer PSD reopened with its contents intact. That batch retained the then-current
engine pin, five patches and dependency lockfile. The [test record](verification.md#native-command-entry-source-picking-and-pencil-scrolling-correction)
contains the evidence; physical Pencil and native keyboard dictation acceptance
remain on the roadmap.

## Development in the open

Use `main` for verified batches and topic branches for ongoing work. Fetch before
pushing; preserve other contributors' work and merge diverged histories normally.
Each checkout uses its own `target/` directory. Keep `upstream.env` and the seven
engine patches synchronized, and regenerate dependency notices when dependencies
change. The engine baseline is `5896f0b`; the patched tree is
`62cc2633db6d09647e80dfa2e053539bfcdaff05` (equivalent to `a28993e`).

[The roadmap](ipad-port.md) owns upcoming workflow and physical-device acceptance.
[Security reports](../SECURITY.md) have a private reporting route. Upstream
contributions follow the [collaboration notes](upstream.md).
