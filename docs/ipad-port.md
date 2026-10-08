# iPad workspace roadmap

The goal is the complete PhotoCraft workflow adapted for iPad and Apple Pencil. Shared engine commands power a dedicated touch workspace.

## Architecture

PhotoCraft owns the engine, renderer and shared commands. This repository owns the Rust iPad workspace, touch/Pencil input adapter, local preview and acceptance evidence. Reusable browser-host and input changes are kept as reviewable patches. See [release checks](release-preparation.md).

## Design direction

The workspace keeps familiar editor navigation and precise Pencil targets: a scrollable, grouped icon rail, compact navigation and contextual controls. Command browsing follows the actual menu hierarchy, with search available on demand. PhotoCraft's neutral theme tokens (canvas #282828, chrome #323232, dock #1e1e1e, text #dedede, accent #378ef0 in Pro), shared sans-serif type and restrained selected-state accents carry through the interface. The two-column 40-point tool rail exposes every tool; panel tabs use icons and the selected panel's title. Form controls use 44-point targets where finger use matters. Portrait keeps the canvas above its inspector.

Review each surface as an editing workflow, not as proof that a command can be found in a list. The hierarchy and rail are the first design pass; effects, selections, colours, presets and file workflows still require individual refinement.

Command search opens from the top bar when needed. It uses normal text input, including native iPad keyboard dictation, and the same live tool/menu catalogue. Ambiguous targets show labelled choices; parameters use existing dialogs. There is no permanent bottom command shelf or application speech recognizer.

The next layout pass keeps one unrestricted Studio, with Photography and Illustration as optional saved arrangements. These change visible panels while retaining the same tools, documents and settings. A single edge dock connects layer targets, properties and deeper brush/colour/adjustment controls to the canvas, with collapsed, working and expanded states. The intended working sizes are roughly 304 points in landscape and a 300-point bottom dock in portrait. The context strip stays within one row of two to four relevant controls.

The Layers stack now scrolls independently with pinned actions, separate image/mask/vector targets, meaningful layer-kind thumbnails and a properties/action sheet. Rows stay compact when their names have room; crowded masked/group rows move the thumbnails to a second line, preserving readable names and full-size targets. Mask controls open inside the dock, leaving the canvas drawable: explicit composite/mask/overlay views, enabled/link state, selection loading and inversion, with apply/delete separated from routine editing. A minimum portrait dock height preserves two usable compact layer rows. Remaining refinements include inline image properties for wider docks, mask density/feather and contextual constraints instead of bare Alt/Shift latches. Typography, presets and adjustments should expand the same dock instead of requiring unrelated modal lists. The reference workflows are photograph → crop → adjustment → mask → retouch → PSD/export, and canvas → brush → colour → clipped layer → brush dynamics → type/vector → save.

Search and keyboard dictation share resolution and action handling. An unambiguous exact tool name switches tools on Enter; the search sheet offers Restore for the previous tool. Transcribing text alone does not execute it. Menu/document commands retain their labelled choices and shared controls. Native iPad keyboard/dictation acceptance remains open.

Arrange mode adds dedicated drag handles for moving layers above, below, or
into a group through the shared undoable command. Grabbing a selected handle
moves the selected set in its existing order; a selected group carries its
children once. Grabbing an unselected handle moves that layer alone. Set moves
preserve the active paint target and restore the selection with Undo. Normal
row drags remain available for scrolling. Quick Mask disables Arrange;
cancelled, in-place and clipped-out drops leave the document unchanged.
Holding a handle near a visible stack edge scrolls toward offscreen layers;
leaving the edge or ending the drag stops it.

## Acceptance matrix

| Surface | Implemented in v0.1 | Next |
|---|---|---|
| Hosting | Mac serves only public files on LAN, original and preview separated | Re-run scoped-server tests after changes |
| Workspace | Touch rail, adaptive bottom/right inspector, contextual strip | Portrait, landscape, Split View, safe areas, keyboard checks |
| Tools | All 49 tools in a grouped, scrollable icon rail | Per-tool options, actual gestures, apply/cancel controls |
| Commands | Menu hierarchy, on-demand top-bar search, shared tool/menu resolution; normal text entry supports native keyboard dictation | Real iPad keyboard/dictation acceptance; responsive dialogs for every family; catalogue access alone is not completion |
| Layers | Independent stack, readable nested rows, pinned actions, image/mask targeting, docked mask controls, properties sheet, selected-set Arrange handles with edge autoscroll, consistent multiselect, visibility, opacity, blend, locks, order, groups, masks, rename, thumbnails; channel/path sheets; adaptive Layer Style dialog | Mask density/feather, physical channel/path/effects workflows; adjustment surfaces |
| Brush | Tip basics, pressure, tilt influence, smoothing, presets; all 13 shared dynamics sections and live stroke preview | Physical dynamics/texture/mixer workflows and complete preset management |
| Colour | Pinned foreground/background targets; smooth saturation/value and hue picker; RGB, HSB, Lab D50 and hex values; browser-saved named swatches; foreground sampling; selected-type recolouring | CMYK/profile proofing, palette import/export, physical Pencil precision |
| Selection / retouch | Selection menu with Layer via Copy/Cut, inverse and deselect; one-shot Set source for Clone Stamp/Healing | Physical Pencil source-pick acceptance; complete selection/refine/mask workflows |
| History | Large history rows, undo/redo and state traversal | Integration test with active transforms and grouped settings |
| Documents | Open/new/save/export access, touch new-document and Export As dialogs, document switcher | Close/recovery workflow, remaining file dialog families, repeated PSD roundtrips |
| Pencil | Per-point pressure/tilt feed, optional coalesced samples with fallback, palm suppression, cancellation release | Physical device evidence; browser sample fidelity, source/constraint gestures |
| Navigation | Two-finger pan/pinch; finger canvas does not paint; direct panel drag scrolling, UI-only Pencil touch lifetimes for shared controls, wider inspector/sheet scrollbars | Physical Pencil scrolling, cross-boundary/cancel stress, orientation changes |
| Reliability | Preserve last served preview when build fails | Browser memory loss/context recovery, large PSD performance |

## Preview evidence before workspace replacement

Chrome: create, paint, undo/redo, panel toggle and PSD download verified. Original editor PSD saved and reopened with stroke intact. The preview has also been tried on an iPad; iPadOS 26.3 Web Inspector connects over USB. See [verification](verification.md) for build and device test results.
