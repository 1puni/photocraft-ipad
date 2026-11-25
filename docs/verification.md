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

Delivered on public `main` at `66c5572`, including implementation `cc83215` and
browser evidence `15baa97`. The publication checkout independently reran
`cargo test --locked`: all 32 tests and binary/doc-test targets passed. Its
source comparison confirmed the integrated Rust source matches `cc83215`, with
the engine pin, five patches and lockfile unchanged from `1083549`.

Search now opens from the top magnifier in a modal without changing the canvas or inspector geometry. Text comes from a normal input and the native keyboard; there is no application microphone or recognizer. Results separate names from menu paths and offer category filters. Bare “blur” remains ambiguous; “blur tool” selects the tool, while Box, Gaussian, Motion and Lens Blur dispatch their distinct shared dialogs. The foreground/background colours remain visible in the pinned rail, with separate editing targets.

The selection context menu and Layer properties expose Layer via Copy/Cut, inverse and deselect. Copy/Cut operate on selected pixels of the active layer. Clone Stamp and Healing have a one-shot Set source action: tap a source on the canvas, then the next Pencil stroke paints. Source picking accepts a finger as well; ordinary single-finger canvas input remains navigation-only. Engine errors are now visible in the workspace.

Extension scroll areas explicitly accept pointer drags, including the first interaction after reload. Pencil contacts starting over UI also report force-less touch lifetimes, allowing the shared dropdowns and dialogs to enable their existing kinetic scrolling. UI ownership is fixed at press-down and ends on release/cancellation; those contacts do not feed brush pressure samples. Canvas Pencil input retains its original pressure/tilt route. Inspector and sheet scrollbars use a solid 14-point track with a gap from controls.

Fresh extension checks pass: 32 tests, formatting, native Clippy with warnings denied, and Wasm check (three unchanged upstream dead-code warnings). Regression evidence includes fresh-session layer dragging without clicks/edits, shared scrolling after a Pencil UI tap, cancellation and canvas/UI ownership classification, source pick followed by cloned pixels and undo, selection copy/cut pixels and undo, four distinct blur dialogs, visible colour targets, and unchanged canvas bounds when search opens at portrait/Split View/keyboard-sized viewports. Typing results also leaves the search field and sheet position fixed. These are synthetic tests, not measured physical Pencil or native-dictation acceptance.

The optimized build and locked dependency notice generation completed at 16:04 UTC. The engine pin, five patches and dependency lockfile remain unchanged. Integration with canonical `4965c29` retained the publishing copy and segmentation research without changing the tested Rust source. Port 4876 returned HTTP 200 for the resulting distribution.

Mac Chrome acceptance: the bottom command shelf/microphone are absent; both pinned colour chips are visible and the background chip opens Background editing. Dragging search results scrolls without selecting a command. Enter on bare “blur” keeps choices open; the exact Motion Blur result opens angle/distance controls. The first broad-list click after a fling landed on Shape Blur as inertia continued, so the subsequent check narrowed the query before choosing Motion Blur; this is not evidence of a wrong engine dispatch. Pencil inertia and precise selection after flings remain part of physical acceptance. Set source visibly arms, a canvas tap returns to Stamp painting, the next stroke copies source pixels, and Undo removes that stroke. Marquee selection → Layer via Copy creates a separate third layer; saving and reopening the 302,417-byte PSD retains all three layers and the existing image/mask content. The final build was reloaded and its fixed search position checked before/after typing. No console errors were captured; the existing upstream SetTheme(SystemDefault) warning remains. Physical iPad scrolling, source picking, cancellation/boundary gestures and keyboard dictation still await acceptance.

## Tablet colour studio

The colour inspector has pinned foreground/background targets and Picker, Values and Swatches views. The picker uses interpolated saturation/value and hue fields, retaining independent hue/saturation intent when either target becomes black. Wide bottom docks show quick swatches beside a bounded picker and arrange all three numeric channels across one row. Narrow landscape inspectors use aligned vertical values. RGB, HSB, Lab D50 and hex controls set the shared sRGB tool colour; Lab controls retain in-progress component values instead of feeding clipped RGB conversions back into subsequent edits. This is not profile proofing or a CMYK workflow.

Named swatches retain exact floating-point RGB values in browser storage, with explicit save/remove, validation, a 64-colour limit and a pinned failure/retry message. Startup reads never rewrite malformed stored data. Foreground edits notify the shared type tool so selected text recolours even when the chosen swatch equals the existing foreground; background edits leave text alone. Ordinary view/model changes never recolour a document.

All 38 extension tests, formatting, native Clippy with warnings denied and Wasm check pass. New regressions cover exact swatch roundtrips and rejected data, independent achromatic hue memory, idle numeric-model changes near the hue boundary, reachable storage retry above a full palette, pinned controls and visible numeric channels at landscape/portrait/Split View sizes, and selected-text recolouring with Undo. Read-only independent review found no remaining source defects after these fixes. The engine pin, five patches, dependency lockfile and release presentation are unchanged.

The final optimized build with locked dependency notice generation completed at 16:45 UTC. Mac Chrome acceptance used separate tabs, preserving the earlier review document: opened the saved three-layer PSD, dragged both picker fields, switched RGB/HSB/Lab without changing the colour, edited Lab lightness, saved named swatch “Harbour blue” (#497da7), changed Background without changing Foreground, and restored the saved swatch in a fresh browser workspace. The final build visibly retains all three numeric channels in the shallow bottom dock at approximately 520 pixels wide; controls below them scroll while target/view switches stay pinned. Landscape inspection used a 1272×846 viewport. Dragging a numeric field edits its value; scrolling was checked from unused panel space and with the wheel. These are mouse/browser checks, not physical Pencil acceptance.

The browser viewport-emulation override distorted canvas scaling, so it was reset and native window sizing was used for visual acceptance; exact 1194×834, 834×1194 and 507×768 geometry remains covered by the UI tests. The final browser console captured no errors and only the existing upstream SetTheme(SystemDefault) warning. The new colour surface still needs real iPad keyboard/Pencil precision and orientation checks.

## Mask controls and explicit layer targeting

The Layers mask button now opens controls in the existing dock when a pixel or vector mask is present. Back, layer identity and target tabs remain pinned while the body scrolls. Pixel-mask Composite, Mask only and Overlay are explicit view states; Enabled and Linked display their current states. Load selection and Invert use the shared commands, and an active selection changes the label to Invert selected area. Apply/delete sit under Mask actions; Apply is disabled for unsupported layer kinds. Vector masks expose Edit path, their own enable/link/invert state, conversion to a pixel mask, and pixel-mask creation without removing the vector mask. Vector targeting is labelled as a path target; brushes still paint the layer image. Density/feather controls remain separate work because pixel masks have no corresponding engine property setter yet.

Explicit image/mask thumbnail targeting now finishes Quick Mask through its normal command, preserving its edits as a selection and retaining Undo. Add mask resolves Quick Mask before reading that restored selection, so the new mask keeps the edited shape. Switching to the same pixel mask preserves Mask only/Overlay view; selecting Image exits it. Returning to the layer stack does not change the paint target. External alpha-channel/Quick Mask selection dismisses mask details rather than showing a misleading selected mask tab. Multiselect thumbnail taps toggle membership consistently with layer-name taps; disabled vector masks now use the same crossed-out treatment as pixel masks.

All 47 extension tests, formatting, native Clippy with warnings denied and Wasm check pass. Regressions cover mask view changes without document/history edits, same-mask view preservation through inversion and reopening controls, inversion isolation and Undo, pixel/vector state isolation, group Apply enablement, mask delete/Undo, vector-only pixel-mask creation, external target changes, full-size primary controls and pinned navigation across three viewport sizes, and multiselect thumbnail membership. Painted Quick Mask data survives explicit target changes, mask creation and Undo. Disposable UI fixtures also verify that Apply changes the expected image alpha and Undo restores image plus mask; vector conversion opens pixel controls without changing image pixels, and Undo restores the original vector mask. Independent read-only review found no remaining concrete source defects. No engine pin, patch, dependency or release-presentation change is needed for this slice.

The final optimized build with locked dependency notices completed at 17:12 UTC. Mac Chrome acceptance opened the earlier three-layer PSD, edited the pixel mask while Overlay controls remained beside the canvas, inverted in Mask only view, and checked enable/Undo and the resulting mask pixels. Undo can restore an earlier active layer along with the document; this is the shared engine's recorded-history behavior. The browser caught an unsupported Back icon, replaced by a full-size text button. Saving and reopening the 308,123-byte PSD in a fresh final-build workspace retained all three layers and both mask strokes. Back and reopening mask controls retained the mask target and Overlay view. Final console capture contains no errors and only the existing SetTheme(SystemDefault) warning.

Landscape browser inspection used 1272×846/903 viewports. Compact browser resizing was inconclusive in this pass; exact 1194×834, 834×1194 and 507×768 layout, 44-point controls and pinned navigation are covered by the UI tests. Physical iPad orientation, Pencil mask painting and scrolling still need acceptance. Existing review documents were preserved, and the Mac/LAN preview remains on port 4876.

## Readable nested layer rows

Layer rows now budget for at least 88 points of name space after indentation, disclosure and image/mask targets. Crowded rows place the targets on a second line; wider rows retain the compact layout. Visibility and group disclosure have 44-point targets, thumbnail targets remain 44×48, and names are left aligned. Name and thumbnail identities remain stable when a resize switches the row layout. Selection, renaming and paint-target changes do not change row geometry.

All 49 extension tests, formatting, native Clippy with warnings denied and Wasm check pass. Layout regressions cover leaf/group rows, zero/one/two masks, depths 0/2/8, long names and four widths with the actual nonfloating scrollbar gutter. They verify readable names, disjoint full-size targets and containment within the scrolling content area. A real nested-group workflow verifies automatic reveal of both lines, independent image/pixel/vector targeting, unchanged row geometry on rename, and visibility without changing selection. Existing panel-drag scrolling and portrait footer tests still pass. Independent design review caught the name-control identity issue; resize tests verify stable control IDs. The final regression also checks actual text-shape intersection with the visible name rectangle through a portrait/landscape cycle.

The final optimized build with locked notices completed at 17:41 UTC. Mac Chrome opened a synthetic 104,218-byte PSD containing three nested groups, both mask types and two original colour rectangles. Browser acceptance verified readable crowded rows, independent image/pixel/vector targeting, collapse/expand retaining the target, and name-click selection. The first browser resize cycle exposed disappearing name text; the final implementation uses explicit bounded, clipped text drawing and a stable click target. Repeating portrait emulation → normal landscape on the final build retained the long name and its click behavior. The emulation scales this canvas differently from a physical device, so it establishes transition behavior rather than exact iPad target sizes; those remain covered by the UI geometry tests. The override was reset. Final console capture contains no errors and only the existing SetTheme(SystemDefault) warning. LAN preview returned HTTP 200; existing review documents were preserved. Physical Pencil/orientation acceptance remains open. Engine pin, five patches, dependencies and release presentation are unchanged.

## Welcome attribution

The empty workspace credits “iPad workspace by 1puni”, linked to this repository, with “Built on PhotoCraft” beneath it. It uses extension-owned chrome, retains the upstream welcome and attribution, and disappears when a document opens. All 49 tests, formatting, native Clippy and Wasm check pass. The optimized build with locked notices completed at 17:53 UTC; Chrome visibly showed both credit lines, then removed the credit strip after opening the synthetic nested-group PSD. No console errors were captured. Engine pin, patches and dependencies remain unchanged.

## Layer arrangement

Implementation `7939307` adds an explicit Arrange toggle and dedicated 44-point
drag handles. One layer or group can move above, below or into a group using
`layer.moveTo`. Ordinary name/thumbnail drags remain available for scrolling.
Multi-selection and Quick Mask disable Arrange. Moving the active masked layer
preserves its paint target; moving a different layer selects its image target.

All 59 tests, formatting, native Clippy with warnings denied and Wasm check pass.
Regressions cover nesting and one-step Undo, self/descendant rejection,
multi-selection, mask targeting, Quick Mask gating, outside/Escape/touch-cancel
drops, hidden-panel cancellation and a clipped row behind the pinned footer.
The row geometry matrix also runs with drag handles present. Review caught and
fixed unclipped drop hit-testing and Quick Mask's extra history edit before the
final build. Engine pin, patches and dependencies are unchanged.

The optimized build with locked notices completed at 18:07 UTC. Chrome opened
the synthetic nested-group PSD and moved its masked group above the outer
group; one Undo restored the original hierarchy. Moving the image layer into
another group worked, and dropping outside Layers left the hierarchy intact.
The artwork remained unchanged. Console capture has no errors and only the
existing SetTheme warning. The preview remains on port 4876; other review
documents were preserved. Physical Pencil drag/scroll acceptance, multi-layer
dragging and long-stack edge autoscroll remain open.

## Long-stack arrangement

Implementation `9d60555` adds edge autoscroll during a dedicated-handle drag.
Only the visible stack accepts it: a band up to 36 points at each edge controls
direction and speed, with a neutral middle even in a short portrait dock.
Elapsed time is capped to avoid a jump after a stalled frame, and offsets clamp
to the content bounds. Leaving the band or ending/cancelling the drag clears the
timer; ordinary row scrolling retains the existing ScrollArea behavior.

All 62 tests, formatting, native Clippy with warnings denied and Wasm check pass.
Held-pointer tests scroll until a named, initially hidden layer is visible,
drop on that same layer and verify Undo. Further checks cover reversing at the
top edge, stopping outside the stack, no idle drift, short portrait content and
Touch Cancel during active autoscroll. No ScrollArea state reset, engine patch,
pin or dependency change was needed.

The optimized build with locked notices completed at 18:20 UTC. Chrome opened a
synthetic 109,066-byte PSD with 29 layers, scrolled it by dragging ordinary rows,
then moved a layer by its handle at the scrolled position. Undo restored the
order and the previously active layer; the artwork remained unchanged. Console
capture has no errors and only the existing SetTheme warning. The browser tool
performs complete drags rather than sustained holds, so continuous edge-hold
acceptance here is synthetic UI evidence, not browser or physical Pencil
measurement. The updated preview is served on port 4876, and previous review
documents remain open. Physical Pencil acceptance and multi-layer dragging
remain follow-ups.

## Moving selected layers together

UI implementation `2a2305f` allows Arrange on a selected set. The grip snapshots
the set when the gesture starts; an unselected grip still moves only its own
layer. Selected rows are excluded from drop feedback. Successful set moves keep
the active layer and its mask/vector/channel target. Cancelled and in-place
drops preserve document, history and selection.

Engine patch six, `21a573cb94c31a862afc2f201976639b758631c2`, extends `layer.moveTo`
with an explicit `layers` parameter while retaining the single-layer/active
fallback. It normalizes roots in document order, carries selected descendants
with their selected group once, validates IDs/positions/cycles/depth before
commit, and records one history step. Explicit set moves capture the live
selection for Undo. No-op moves create no history entry. Applying all six mail
patches to the unchanged public baseline reproduced tree
`6823804243a55b9bd987f15ba7c8b56cf71a4c3f`, matching `upstream.env`.

All 66 extension tests, formatting, native Clippy with warnings denied and Wasm
check pass. Engine checks pass: all 779 non-ignored unit tests, integration/doc
targets, architecture layering, the workspace Wasm check and the opt-in
adversarial-command test. The six new engine tests cover ordered moves across
parents, selected ancestor/child normalization, malformed inputs, cycles, depth
overflow, in-place moves, legacy fallback and selection-preserving Undo/Redo.
The first run caught a stale-history-selection defect, fixed before acceptance.

Unqualified engine Clippy fails on existing `nonminimal_bool` warnings in
`text/src/cjk.rs`, `engine/src/color_cmds.rs`, `engine/src/fill_cmds.rs`, and
`manual_range_contains` in the existing fill tests. Those paths are unchanged.
After fixing the new patch's own warning, engine Clippy passes with only those
two baseline lint categories allowed. This is a qualified result, not a clean
unqualified engine Clippy run. Dependency and lockfile contents are unchanged.

The optimized build with locked notices completed at 18:36 UTC. Chrome opened
the synthetic 29-layer PSD, selected two separated layers and moved the pair
below another layer. Their relative order, both selected highlights and the
active layer were preserved. One Undo restored the original order and selection;
Redo reapplied the move. A subsequent outside drop changed neither order nor
selection. The artwork stayed unchanged. Console capture has no errors and only
the existing SetTheme warning. Existing review documents were preserved; the
Mac/LAN preview remains on port 4876. Physical Pencil acceptance remains open.

## Docked layer properties

Implementation `8a38a53` replaces the layer-properties modal with an inspector
mode beside the canvas. Back and the active-layer/paint-target caption remain
pinned while controls scroll. Rename, blend, layer opacity, visibility and a
two-column lock grid precede clearly scoped active-layer and selected-set
actions. Mask controls link back to properties without retargeting the canvas.
The existing properties commands also open this dock.

All 73 tests, formatting, native Clippy with warnings denied and Wasm check pass.
New coverage includes image/pixel-mask/vector/alpha/Quick Mask target preservation,
rename Undo, a held opacity drag coalescing into one Undo step, portrait pinned
navigation, Background gating and transitions between mask controls/properties.
The opacity control has a distinct accessible label from brush opacity. The
engine pin, six patches and dependencies are unchanged.

The optimized build with locked notices completed at 18:54 UTC. Chrome opened
the synthetic nested-group PSD. Dragging layer opacity from 100% to 39% visibly
faded the artwork; one Undo restored its opacity. Renaming a group appeared in
the stack. Opening properties on a pixel mask, scrolling its controls and
switching to mask controls and back preserved the mask target. Back remained
visible while scrolling. No console errors were captured; the existing SetTheme
warning remains. Previous review documents were preserved and the preview stays
on port 4876. Physical Pencil acceptance remains open.

The browser check also reproduced an existing shared-engine history behavior:
after selecting a different layer and editing its opacity, Undo restores the
previous history state's selected layer rather than the live pre-edit selection.
Opacity itself restores correctly. The dock does not introduce this behavior;
the shared engine explicitly tests it. Improving selection capture for property
edits is a separate history follow-up, and target preservation on Undo is not
claimed for this batch.

## Undo keeps the pre-edit layer selection

Implementation `28afe7a` packages engine patch seven, `a28993e`. A successful
new `Session::edit` transaction records the live pre-edit active layer and
selected set with its Undo snapshot. Selecting a layer alone remains outside
history. Subsequent coalesced updates preserve the gesture's original Undo
target, while Redo restores its final target. Failed edits do not alter history
target metadata. The earlier Arrange-only capture is now handled by this shared
path. No history representation, dependency or lockfile changed.

The held-opacity UI regression failed against the previous engine: select an
older layer, drag opacity, then Undo incorrectly targets the newer layer. It
now passes and checks both Undo and Redo selection. All 73 extension tests,
formatting, native Clippy with warnings denied and Wasm check pass. All 783
non-ignored engine unit tests, integration/doc targets, seven history tests,
layering, workspace Wasm and the opt-in adversarial-command check pass. The
existing target-restoration regression remains unchanged. The Merge Layers
regression now verifies the complete pre-merge selection on Undo and the merged
layer on Redo, across 8/16/32-bit documents.

Engine Clippy passes with the same two existing lint categories allowed as in
the selected-layer Arrange batch (`nonminimal_bool`, `manual_range_contains`);
this remains a qualified engine result. A fresh local clone applying all seven
mail patches to the unchanged public baseline reproduced tree
`62cc2633db6d09647e80dfa2e053539bfcdaff05`, matching `upstream.env`.

The optimized build with locked notices completed at 19:05 UTC. In a fresh
Chrome document, the same synthetic PSD opened with its outer group selected.
Selecting the pixel layer and dragging its docked opacity from 100% to 39%
faded the artwork. One Undo restored 100% and kept the pixel layer active; Redo
restored 39% on that same layer. A final Undo and return to the stack visibly
retained the selected image thumbnail and original artwork. No console errors
were captured; only the existing SetTheme warning remains. This supersedes the
Undo selection limitation recorded in the docked-properties batch. The preview
is served on port 4876, previous review documents remain open, and physical
Pencil acceptance remains separate.

## Mask density and feather beside the canvas

UI implementation `2d7c167` adds density (0–100%) and feather (0–1000 px) to
the existing pixel/vector mask dock. Each has a 44-point slider and a separately
labelled numeric field. Feather uses a logarithmic slider for precision near
zero; direct numeric entry accepts fractional pixels. Values come from the
current mask, disabled masks remain editable, and Back/target tabs stay pinned
while the controls scroll. Gesture keys separate documents, layers, mask types
and properties so each drag has its own Undo step.

Patch/pin commit `fb12080` packages engine `9fe4e48` as patch eight. The new
`layer.layerMask.edit` command uses the existing compositor and history path,
validates finite ranges before editing, handles explicit or active targets, and
skips no-op history entries. Vector controls reuse `layer.vectorMask.edit`.
Applying all eight patches in a fresh local clone reproduced engine tree
`0314c46e7585c5e313dded3b95ac2fd9bacd7e76`. The public baseline, first seven
patches, dependencies and lockfile are unchanged.

All 77 extension tests, formatting, native Clippy with warnings denied and Wasm
check pass. Held-drag tests cover both controls on both mask types, one-step
Undo, unchanged image/other-mask content and paint targets, disabled masks,
fresh values when switching targets and portrait scrolling. A numeric-entry
test types 12.5 px and verifies the stored value and one-step Undo. It caught an
unlabelled built-in slider value field; separate labelled fields fix that
accessibility gap.

All 789 non-ignored engine unit tests, integration/doc targets, layering,
workspace Wasm, command parity and the adversarial-command check pass. Six new
engine tests cover validation, no-ops, coalescing, targeting, density/feather
composition at 8/16/32-bit depth and a serialized PSD roundtrip (including PSD's
byte quantization of density). Engine Clippy passes with the same two documented
baseline lint categories allowed; it remains a qualified engine result.

The optimized build with locked notices completed at 19:20 UTC. Chrome opened
a synthetic 109,546-byte PSD with a sharp pixel-mask edge and an independent
vector mask. Reducing each mask's density from 100% to 49% visibly revealed its
hidden area; Undo restored full masking. Typing 40 px into each feather field
softened its respective vertical/horizontal edge, and one Undo restored the
sharp edge without switching the active layer or controls. Switching mask tabs
showed their independent values.

The portrait viewport override showed the bottom-dock breakpoint; it also
changed rendering/input scale, so it is not physical target-size or Pencil
scrolling evidence. Portrait pinned navigation and scrolling are covered by
synthetic UI tests. The override was reset and the artwork fitted again.

Chrome saved a 251,514-byte PSD and reopened it in a fresh tab. The pixel mask
still read 100% density and 40 px feather, with its soft edge intact and the
disabled vector mask retained. Both browser tabs logged no errors and only the
existing SetTheme warning. The reopened review document remains available;
previous review documents were preserved. Port 4876 serves this build. Physical
Pencil acceptance remains open.

## Opt-in live browser control — 24 November 2026

Prepared on an isolated topic branch. Existing browser documents and the original
local preview were not reloaded or replaced. The new relay remains local; no
hosted editor deployment or upstream submission was made.

Verified:

- `cargo fmt --check`.
- `cargo test --locked`: all 77 extension tests passed.
- `cargo clippy --locked --all-targets --no-deps -- -D warnings` passed.
- `cargo check --target wasm32-unknown-unknown` passed; subsequent optimized
  `./build.sh` used the lockfile and passed, including notices/package generation.
- `uv run --no-project --python 3.12 python -m unittest test_server test_control_relay`:
  five tests passed, including auth rejection, exclusive browser ownership,
  exact request/reply forwarding and expiry without replay.
- Shared UI regression `attaching_transport_preserves_live_document_and_dispatches_requests`
  passed (one selected test, 837 filtered). The engine patch preserves the live
  session while attaching a transport and runs an inspect request through its queue.
- Existing upstream `photocraft-cli` built. Real MCP stdio initialize and
  `tools/call ui_inspect` reached the relay and correctly reported no active
  browser before a page opted in.

Native builds used `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0` to limit disk
usage. A pre-existing unused `OPENABLE` engine constant warning appeared in the
WASM build. No engine commands or format behavior changed.

**Live acceptance remains unverified:** native browser reads worked, but New Tab,
New Window and File-menu input attempts timed out. No live page has yet attached;
inspect/render against a real browser document, browser save/download and
physical iPad testing must still be performed. Automated transport simulation and
successful compilation do not establish those results. Open a new opted-in tab
using [the setup guide](browser-control.md), preserving any unsaved old tab, then
verify document dimensions/layers through MCP and save/reopen a layered document.

### Byte transfers and explicit browser handoff

The initial protocol1 page subsequently attached: real upstream MCP `ui_inspect`
returned an empty live browser session and advancing frame counters. A viewport
preview request timed out while later inspect requests continued to succeed;
viewport capture is not claimed working on this tested browser path.

Protocol2 adds bounded chunk transfers into the browser's existing automation
file services. Import/export use the same services as the shared editor, with no
picker and no host filesystem root. `browser_file.py` implements explicit local
upload, atomic download, and export-copy through upstream MCP. `relay.release`
retires a page's lease before a deliberate desktop/iPad handoff.

Fresh checks passed: all **79** extension tests, **8** Python server/transport/
file-client tests, formatting and native Clippy with warnings denied. The
optimized browser build passed with `CARGO_PROFILE_WASM_RELEASE_LTO=false` for
this local development build (source release profile unchanged). The shared
native importer also read a four-layer PSD successfully.

**Live protocol2 file I/O verified after the new empty page refreshed:** real
upstream MCP uploaded a 9,412,396-byte RGBA-layered PSD without a picker. Live
`doc_inspect` reported 1516 × 1037 and all four independent layers. MCP save used
the browser exporter and atomically downloaded a 9,174,444-byte PSD with no
warnings. Both the shared native reader and an independent PSD parser confirmed
the dimensions and four layers; the independent comparison found every alpha
value in the transparent artwork layer unchanged. The original local source and
older browser document were not replaced.

MCP export-copy produced a 1,894,512-byte PNG, visually inspected with all four
layers composed. Its normal flattened-output warning applies to the PNG copy;
the live document and saved PSD retained their layers. Export-copy did not change
the document save path/revision. These are actual browser import/export results,
not transport simulations. The viewport screenshot path still timed out; use
PNG export-copy for document previews. Physical-iPad file I/O and shared live
canvas synchronization are not claimed; the documented lease handoff transfers
saved document contents between independent browser sessions.


## Named browser sessions — 2026-11-24

Implementation: `1954f9e`, local release build, Zen on macOS, WebGL renderer.
Verified in a fresh empty editor window: expand Device/Tab badge, edit tab label,
read matching browser title and MCP session metadata, select exact tab through
upstream MCP and successfully inspect UI. Reload retained tab ID and label while
changing page generation; reselection and UI inspection succeeded. A second tab
registered a distinct ID and independently editable label. The badge was visually
checked at the desktop window's portrait-shaped viewport. No artwork was loaded
or altered for these checks.

Automated checks: 79 native tests, 22 server/relay tests, JavaScript identity
checks (reload, separate tabs, copied-session conflict, device rename propagation,
storage-denied fallback), cargo fmt, Clippy with warnings denied for this crate,
WASM check, and full packaged release build passed. Existing dead-code warnings
remain in upstream engine/CMS code. Relay tests cover independent controller
selection, wrong-page replies, offline state, stale selection, cancellation on
reload and timeout without replay.

Physical iPad, landscape/Split View badge layout, document autosave/recovery and
live browser duplicate-tab handling were not validated in this batch. Duplicate
tab conflict handling is covered by automated identity tests. This is explicit
routing and reconnect identity, not document synchronization or recovery.

## Browser autosave and reload recovery — 2026-11-24

The named-session batch above has been extended with local document recovery;
its earlier “no document autosave” limitation is superseded by this batch.

A fresh Zen/macOS WebGL tab loaded a private six-layer 1505 × 1045 PSD through
MCP. One layer was renamed, moved by (7, 3), and set to 0.73 opacity. After the
badge reported a committed local autosave, the page was reloaded. The same tab
ID reconnected with a new page generation. MCP readback confirmed all six layer
records, active layer, selection and dimensions matched the pre-reload state.
PNG exports before/after were byte-identical (SHA-256
`c0ee3f3e93b0f7527ac228aed058b47fb119d5c313495744ab3d18114393226a`).
The source PSD and other artwork tabs were not modified.

All 81 native tests passed. The strengthened native recovery round-trip also
passed separately with nonempty pixels, hidden/transparent translated layers,
two documents, active selection, exact native-bundle comparison, and corrupt
snapshot rejection. JavaScript tests with fake-indexeddb 6.2.8 passed for atomic
reload, independent tabs, duplicate-tab fork, stale-page write rejection,
transaction abort retaining the old snapshot, empty workspace persistence,
blocked session identity and the pending-change reload guard. Existing identity
tests passed. Clippy, formatting, WASM check and packaged release build passed;
existing upstream dead-code warnings remain. Served JS modules were compared
byte-for-byte with the reviewed source.

Physical iPad reload/suspension, landscape/Split View badge layout, browser
storage eviction and large-document performance remain unverified. The browser
check used a normal reload, not an OS crash or force-killed process. Recovery is
local to the browser origin and tab, and does not preserve undo history.
See [browser recovery](browser-recovery.md) for behavior and checks.

## Studio-bound Save — 2026-11-24

Engine patch 0010 adds deferred save acknowledgement; pinned tree
`1873a6a6dc17329b56a73b4ea468ca8a15da4924`. Final optimized build completed
at 14:47:46 UTC. Served studio JS matches source byte-for-byte.

A fresh macOS Chrome WebGL browser used a disposable synthetic pixel-layer
document through the real local studio endpoint and same-origin proxy. Actual
Save button and Cmd-S persisted the moved layer in the exact PSD. A delayed PUT
proved subsequent edits stay dirty and only the captured revision reaches disk.
An aborted request preserved disk state and a normal retry succeeded. A changed
server version rejected the stale save visibly. Save As downloaded an independent
PSD and persisted its detached binding. No page errors were captured. Screenshots
were inspected at 1024×768, 768×1024 and 512×768; the status badge remains visible.
Physical iPad/Safari acceptance of this new path is not claimed.

81 extension tests, formatting, native Clippy, Wasm check, packaged build, three
Python server/proxy tests and JS binding tests passed. The deferred engine-save
regression, touched UI crate Clippy and architecture layering checks passed. Full
UI tests: 835 passed, three ignored, one pre-existing failure in
`layer_reveal::tests::a_new_active_layer_opens_its_groups_and_scrolls_into_view`.
The same undo-selection assertion fails on the parent revision without patch 0010.
It is unrelated to studio persistence and remains unresolved. Existing upstream
Wasm dead-code warnings are unchanged. Private studio storage, HTTP, library,
viewer and linked-layer tests also passed; their fixtures and browser evidence
remain with that studio, not this public repository.

## Discoverable layer transforms — 2026-11-25

Move now starts with visible transform handles in the browser workspace. The
Move context bar and layer properties expose Resize / Rotate and Warp. Active
transforms have a touch-sized mode menu for resize/rotation, skew, distortion,
perspective and warp, plus gesture hints and the existing Apply/Cancel actions.
Returning from warp explicitly says Discard warp; Apply preserves it instead.
The controls reuse the engine's sessions and commands without an engine patch.

All 82 extension tests, formatting and native Clippy passed. The new UI regression
runs at 1194×834, 834×1194 and 507×768: it enters through the visible control,
drags a resize corner, applies, undoes to the exact original document, switches
to Distort and Warp, cancels without document changes, and starts Warp directly.
It checks mode/Apply/Cancel bounds and 44-point heights at every size. Pointer
drags are synthetic, not physical Pencil evidence. Browser visual inspection
was blocked by the unavailable computer-use browser host; physical iPad/Safari
acceptance remains pending.

Wasm check and the optimized packaged build passed (10:17 UTC); existing upstream
dead-code warnings remain. The existing LAN listener on port 4878 returns HTTP
200, with index HTML and compiled WASM byte-identical to the new local package.
This proves distribution, not browser execution. Save before reloading existing
tabs to adopt the build; no artwork was modified during verification.
