# iPad workspace roadmap

The goal is the complete PhotoCraft workflow adapted for iPad and Apple Pencil. Shared engine commands power a dedicated touch workspace.

## Architecture

PhotoCraft owns the engine, renderer and shared commands. This repository owns the Rust iPad workspace, touch/Pencil input adapter, local preview and acceptance evidence. Reusable browser-host and input changes are kept as reviewable patches. See [release checks](release-preparation.md).

## Design direction

The workspace keeps familiar editor navigation and precise Pencil targets: a scrollable, grouped icon rail, compact navigation and contextual controls. Command browsing follows the actual menu hierarchy, with search available on demand. PhotoCraft's neutral theme tokens (canvas #282828, chrome #323232, dock #1e1e1e, text #dedede, accent #378ef0 in Pro), shared sans-serif type and restrained selected-state accents carry through the interface. The two-column 40-point tool rail exposes every tool; panel tabs use icons and the selected panel's title. Form controls use 44-point targets where finger use matters. Portrait keeps the canvas above its inspector.

Review each surface as an editing workflow, not as proof that a command can be found in a list. The hierarchy and rail are the first design pass; effects, selections, colours, presets and file workflows still require individual refinement.

The next command-search pass opens from the top bar when needed. It will use normal text input, including native iPad keyboard dictation, and the same live tool/menu catalogue. Ambiguous targets show labelled choices; parameters use existing dialogs. This replaces the current permanent bottom command shelf and custom browser recognizer.

The next layout pass keeps one unrestricted Studio, with Photography and Illustration as optional saved arrangements. These change visible panels while retaining the same tools, documents and settings. A single edge dock connects layer targets, properties and deeper brush/colour/adjustment controls to the canvas, with collapsed, working and expanded states. The intended working sizes are roughly 304 points in landscape and a 300-point bottom dock in portrait. The context strip stays within one row of two to four relevant controls.

The Layers stack now scrolls independently with pinned actions, separate image/mask/vector targets, meaningful layer-kind thumbnails and a properties/action sheet. A minimum portrait dock height preserves two usable layer rows. Remaining refinements include drag reorder, inline properties for wider docks, mask parameters and contextual actions such as Set source or Constrain instead of bare Alt/Shift latches. Typography, presets, masks and adjustments should expand the same dock instead of requiring unrelated modal lists. The reference workflows are photograph → crop → adjustment → mask → retouch → PSD/export, and canvas → brush → colour → clipped layer → brush dynamics → type/vector → save.

Search and keyboard dictation share resolution and action handling. An unambiguous exact tool name can switch tools with a visible Revert action. Menu/document commands retain their labelled choices and shared controls.

## Acceptance matrix

| Surface | Implemented in v0.1 | Next |
|---|---|---|
| Hosting | Mac serves only public files on LAN, original and preview separated | Re-run scoped-server tests after changes |
| Workspace | Touch rail, adaptive bottom/right inspector, contextual strip | Portrait, landscape, Split View, safe areas, keyboard checks |
| Tools | All 49 tools in a grouped, scrollable icon rail | Per-tool options, actual gestures, apply/cancel controls |
| Commands | Menu hierarchy and shared tool/menu resolution | On-demand top-bar search correction; native keyboard dictation acceptance; responsive dialogs for every family |
| Layers | Independent stack, pinned actions, image/mask targeting, properties sheet, multiselect, visibility, opacity, blend, locks, order, groups, masks, rename, thumbnails; channel/path sheets; adaptive Layer Style dialog | Physical channel/path/effects workflows; adjustment surfaces |
| Brush | Tip basics, pressure, tilt influence, smoothing, presets; all 13 shared dynamics sections and live stroke preview | Physical dynamics/texture/mixer workflows and complete preset management |
| Colour | Saturation/value pad, hue, hex, foreground/background, eyedropper | Swatches, precise multi-model values, profile proofing |
| History | Large history rows, undo/redo and state traversal | Integration test with active transforms and grouped settings |
| Documents | Open/new/save/export access, touch new-document and Export As dialogs, document switcher | Close/recovery workflow, remaining file dialog families, repeated PSD roundtrips |
| Pencil | Per-point pressure/tilt feed, optional coalesced samples with fallback, palm suppression, cancellation release | Physical device evidence; browser sample fidelity, source/constraint gestures |
| Navigation | Two-finger pan/pinch; finger canvas does not paint | Real-device stress, orientation changes, accidental touch cases |
| Reliability | Preserve last served preview when build fails | Browser memory loss/context recovery, large PSD performance |

## Preview evidence before workspace replacement

Chrome: create, paint, undo/redo, panel toggle and PSD download verified. Original editor PSD saved and reopened with stroke intact. The preview has also been tried on an iPad; iPadOS 26.3 Web Inspector connects over USB. See [verification](verification.md) for build and device test results.
