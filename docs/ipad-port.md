# iPad port acceptance

V's goal is the complete PhotoCraft workflow adapted for iPad and Apple Pencil, not a shortcut toolbar. Upstream engine limitations remain explicit; command availability is not Adobe feature parity.

## Repository and publication boundary

The PhotoCraft fork owns reusable browser-host hooks and shared input fixes. This repository owns the Rust iPad workspace, touch/Pencil input adapter, local preview, and acceptance evidence. V explicitly wants local commits as work proceeds. Do not push, open PRs, or publish before reviewing repository copy with V. The public fork and empty private extension repository were created before this hold.

## Acceptance matrix

| Surface | Implemented locally | Still required |
|---|---|---|
| Hosting | Mac serves only public files on LAN, original and preview separated | Re-run scoped-server tests after changes |
| Workspace | Touch rail, adaptive bottom/right inspector, contextual strip | Portrait, landscape, Split View, safe areas, keyboard checks |
| Tools | All 49 tools in searchable touch grid | Per-tool options, actual gestures, apply/cancel controls |
| Commands | Searchable catalogue, categories, disabled unsupported actions | Responsive dialogs for every family; catalogue access alone is not completion |
| Layers | Selection, multiselect, visibility, opacity, blend, locks, order, groups, masks, rename, thumbnails; channel/path sheets; adaptive Layer Style dialog | Physical channel/path/effects workflows; adjustment surfaces |
| Brush | Tip basics, pressure, tilt influence, smoothing, presets; all 13 shared dynamics sections and live stroke preview | Physical dynamics/texture/mixer workflows and complete preset management |
| Colour | Saturation/value pad, hue, hex, foreground/background, eyedropper | Swatches, precise multi-model values, profile proofing |
| History | Large history rows, undo/redo and state traversal | Integration test with active transforms and grouped settings |
| Documents | Open/new/save/export access, touch new-document and Export As dialogs, document switcher | Close/recovery workflow, remaining file dialog families, repeated PSD roundtrips |
| Pencil | Per-point pressure/tilt feed, optional coalesced samples with fallback, palm suppression, cancellation release | Physical device evidence; browser sample fidelity, source/constraint gestures |
| Navigation | Two-finger pan/pinch; finger canvas does not paint | Real-device stress, orientation changes, accidental touch cases |
| Reliability | Preserve last served preview when build fails | Browser memory loss/context recovery, large PSD performance |

## Preview evidence before workspace replacement

Chrome: create, paint, undo/redo, panel toggle and PSD download verified. Original editor PSD saved and reopened with stroke intact. V tested the iPad preview and called it a good start. This is not evidence of all Pencil signals or complete iPad parity. iPadOS 26.3 Web Inspector connects over USB. The local implementation column includes changes awaiting an optimized preview build; see verification.md for the served/verified boundary.
