# iPad workspace roadmap

The goal is the complete PhotoCraft workflow adapted for iPad and Apple Pencil. Shared engine commands power a dedicated touch workspace.

## Repository and publication boundary

The PhotoCraft fork owns reusable browser-host hooks and shared input fixes. This repository owns the Rust iPad workspace, touch/Pencil input adapter, local preview, and acceptance evidence. Source and release preparation are published to the private extension repository. Public visibility awaits V’s go-ahead; see [release preparation](release-preparation.md).

## Design direction after V's hands-on review

V prefers the familiar editor and precise Pencil targets. Replace the catalogue-first interface with a scrollable, grouped icon rail containing every tool, compact navigation, and contextual controls. Command browsing follows the actual menu hierarchy; search is an optional fallback. Keep the existing neutral PhotoCraft theme tokens (canvas #282828, chrome #323232, dock #1e1e1e, text #dedede, accent #378ef0 in Pro), the shared sans-serif UI type, and restrained selected-state accents. A two-column 40-point tool rail exposes more tools without oversized text buttons; panel tabs use icons with the selected panel's title. Preserve 44-point form controls where finger use matters. Portrait keeps the canvas above its inspector.

Review each surface as an editing workflow, not as proof that a command can be found in a list. The hierarchy and rail are the first design pass; effects, selections, colours, presets and file workflows still require individual refinement.

Keep command search available from a top-bar search button. V rejected the permanent bottom shelf because it consumes canvas space and explicitly chose the native keyboard's transcription. Search opens only when requested; dictated and typed text use the same normal text field. The application must not implement its own speech recognition or microphone capture.

The design agent's proposal is one unrestricted Studio, with Photography and Illustration as optional saved arrangements. They change visible panels, not tools, documents, settings or capability. The distinctive interaction is a single edge dock with collapsed, working and expanded states: layer targets, their properties and deeper brush/colour/adjustment controls remain connected to the canvas. In landscape use a roughly 304-point working dock; in portrait use a bottom dock around 300 points, with explicit expansion and collapse. Keep the context strip to one row of two to four relevant controls.

The Layers stack now scrolls independently with pinned actions, separate image/mask/vector targets, meaningful layer-kind thumbnails and a properties/action sheet. A minimum portrait dock height preserves two usable layer rows. Remaining refinements include drag reorder, inline properties for wider docks, mask parameters and contextual actions such as Set source or Constrain instead of bare Alt/Shift latches. Typography, presets, masks and adjustments should expand the same dock instead of requiring unrelated modal lists. The reference workflows are photograph → crop → adjustment → mask → retouch → PSD/export, and canvas → brush → colour → clipped layer → brush dynamics → type/vector → save.

Native keyboard dictation supplies text only. Exact tool names can execute on Enter; search results support a deliberate tap for any command. Ambiguous targets show choices, and parameterized actions open their existing controls. Merely transcribing text does not execute it. Validate native keyboard entry and dictation on the iPad; do not add a custom microphone button or recognition service.

## Acceptance matrix

| Surface | Implemented in v0.1 | Next |
|---|---|---|
| Hosting | Mac serves only public files on LAN, original and preview separated | Re-run scoped-server tests after changes |
| Workspace | Touch rail, adaptive bottom/right inspector, contextual strip | Portrait, landscape, Split View, safe areas, keyboard checks |
| Tools | All 49 tools in a grouped, scrollable icon rail | Per-tool options, actual gestures, apply/cancel controls |
| Commands | Menu hierarchy, on-demand top-bar search, shared tool/menu resolution; normal text entry supports native keyboard dictation | Real iPad keyboard/dictation acceptance; responsive dialogs for every family; catalogue access alone is not completion |
| Layers | Independent stack, pinned actions, image/mask targeting, properties sheet, multiselect, visibility, opacity, blend, locks, order, groups, masks, rename, thumbnails; channel/path sheets; adaptive Layer Style dialog | Physical channel/path/effects workflows; adjustment surfaces |
| Brush | Tip basics, pressure, tilt influence, smoothing, presets; all 13 shared dynamics sections and live stroke preview | Physical dynamics/texture/mixer workflows and complete preset management |
| Colour | Visible foreground/background chips pinned to the rail; saturation/value pad, hue, hex, eyedropper | Swatches, precise multi-model values, profile proofing |
| Selection / retouch | Selection menu with Layer via Copy/Cut, inverse and deselect; one-shot Set source for Clone Stamp/Healing | Physical Pencil source-pick acceptance; complete selection/refine/mask workflows |
| History | Large history rows, undo/redo and state traversal | Integration test with active transforms and grouped settings |
| Documents | Open/new/save/export access, touch new-document and Export As dialogs, document switcher | Close/recovery workflow, remaining file dialog families, repeated PSD roundtrips |
| Pencil | Per-point pressure/tilt feed, optional coalesced samples with fallback, palm suppression, cancellation release | Physical device evidence; browser sample fidelity, source/constraint gestures |
| Navigation | Two-finger pan/pinch; finger canvas does not paint; direct panel drag scrolling, UI-only Pencil touch lifetimes for shared controls, wider inspector/sheet scrollbars | Physical Pencil scrolling, cross-boundary/cancel stress, orientation changes |
| Reliability | Preserve last served preview when build fails | Browser memory loss/context recovery, large PSD performance |

## Preview evidence before workspace replacement

Chrome: create, paint, undo/redo, panel toggle and PSD download verified. Original editor PSD saved and reopened with stroke intact. V tested the iPad preview and called it a good start. iPadOS 26.3 Web Inspector connects over USB. See [verification](verification.md) for build and device test results.
