# iPad port acceptance

V's goal is the complete PhotoCraft workflow adapted for iPad and Apple Pencil, not a shortcut toolbar. Upstream engine limitations remain explicit; command availability is not Adobe feature parity.

## Repository and publication boundary

The PhotoCraft fork owns reusable browser-host hooks and shared input fixes. This repository owns the Rust iPad workspace, touch/Pencil input adapter, local preview, and acceptance evidence. V explicitly wants incremental commits. The initial publication/copy review now belongs to the separate publishing session; send it verified commit handoffs regularly and continue development in the open after it confirms the public repository and branch. Do not race its initial release from this checkout.

## Design direction after V's hands-on review

V prefers the familiar editor and precise Pencil targets. Replace the catalogue-first interface with a scrollable, grouped icon rail containing every tool, compact navigation, and contextual controls. Command browsing follows the actual menu hierarchy; search is an optional fallback. Keep the existing neutral PhotoCraft theme tokens (canvas #282828, chrome #323232, dock #1e1e1e, text #dedede, accent #378ef0 in Pro), the shared sans-serif UI type, and restrained selected-state accents. A two-column 40-point tool rail exposes more tools without oversized text buttons; panel tabs use icons with the selected panel's title. Preserve 44-point form controls where finger use matters. Portrait keeps the canvas above its inspector.

Review each surface as an editing workflow, not as proof that a command can be found in a list. The hierarchy and rail are the first design pass; effects, selections, colours, presets and file workflows still require individual refinement.

Keep command search prominent as a useful power tool. V also requested a bottom dictation control: a short spoken command should select a tool or reach the full command system. Speech and typing must share resolution and action handling; ambiguous targets and missing parameters need explicit choices. Browser-native speech recognition is the first path to verify on iPad Safari. Do not claim speech is implemented or fully on-device until measured.

The design agent's proposal is one unrestricted Studio, with Photography and Illustration as optional saved arrangements. They change visible panels, not tools, documents, settings or capability. The distinctive interaction is a single edge dock with collapsed, working and expanded states: layer targets, their properties and deeper brush/colour/adjustment controls remain connected to the canvas. In landscape use a roughly 304-point working dock; in portrait use a bottom dock around 300 points, with explicit expansion and collapse. Keep the context strip to one row of two to four relevant controls.

The Layers stack now scrolls independently with pinned actions, separate image/mask/vector targets, meaningful layer-kind thumbnails and a properties/action sheet. A minimum portrait dock height preserves two usable layer rows. Remaining refinements include drag reorder, inline properties for wider docks, mask parameters and contextual actions such as Set source or Constrain instead of bare Alt/Shift latches. Typography, presets, masks and adjustments should expand the same dock instead of requiring unrelated modal lists. The reference workflows are photograph → crop → adjustment → mask → retouch → PSD/export, and canvas → brush → colour → clipped layer → brush dynamics → type/vector → save.

Voice is a small bottom-edge microphone beside command search, opening a transcript/action tray. Final unambiguous reversible tool switches can execute immediately with a visible result and Revert. Ambiguity shows choices; required parameters open the same controls as menu/typed commands. Consequential actions show their target and proposed operation first. No always-listening mode, audio retention or document upload is needed. A custom web button cannot be assumed to invoke the native keyboard's dictation; test browser speech recognition and retain normal text/keyboard input as fallback. [WebKit documents Safari's Siri-backed speech recognition](https://webkit.org/blog/11648/new-webkit-features-in-safari-14-1/).

## Acceptance matrix

| Surface | Implemented locally | Still required |
|---|---|---|
| Hosting | Mac serves only public files on LAN, original and preview separated | Re-run scoped-server tests after changes |
| Workspace | Touch rail, adaptive bottom/right inspector, contextual strip | Portrait, landscape, Split View, safe areas, keyboard checks |
| Tools | All 49 tools in a grouped, scrollable icon rail | Per-tool options, actual gestures, apply/cancel controls |
| Commands | Menu hierarchy with optional search and shared command dispatch | Native dictation; responsive dialogs for every family; catalogue access alone is not completion |
| Layers | Independent stack, pinned actions, image/mask targeting, properties sheet, multiselect, visibility, opacity, blend, locks, order, groups, masks, rename, thumbnails; channel/path sheets; adaptive Layer Style dialog | Physical channel/path/effects workflows; adjustment surfaces |
| Brush | Tip basics, pressure, tilt influence, smoothing, presets; all 13 shared dynamics sections and live stroke preview | Physical dynamics/texture/mixer workflows and complete preset management |
| Colour | Saturation/value pad, hue, hex, foreground/background, eyedropper | Swatches, precise multi-model values, profile proofing |
| History | Large history rows, undo/redo and state traversal | Integration test with active transforms and grouped settings |
| Documents | Open/new/save/export access, touch new-document and Export As dialogs, document switcher | Close/recovery workflow, remaining file dialog families, repeated PSD roundtrips |
| Pencil | Per-point pressure/tilt feed, optional coalesced samples with fallback, palm suppression, cancellation release | Physical device evidence; browser sample fidelity, source/constraint gestures |
| Navigation | Two-finger pan/pinch; finger canvas does not paint | Real-device stress, orientation changes, accidental touch cases |
| Reliability | Preserve last served preview when build fails | Browser memory loss/context recovery, large PSD performance |

## Preview evidence before workspace replacement

Chrome: create, paint, undo/redo, panel toggle and PSD download verified. Original editor PSD saved and reopened with stroke intact. V tested the iPad preview and called it a good start. This is not evidence of all Pencil signals or complete iPad parity. iPadOS 26.3 Web Inspector connects over USB. The local implementation column includes changes awaiting an optimized preview build; see verification.md for the served/verified boundary.
