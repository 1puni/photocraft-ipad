# Layer transformation on iPad

Select an image layer, then use its visible Move handles or tap **Transform layer**
in the pinned Layers footer. This entry also works while another tool is selected.
**Transform** remains available in Move's context bar and layer properties.

Drag inside the box to move. Corners resize proportionally, edges stretch one
axis, and dragging just outside rotates. **Resize**, **Distort** and **Warp** are
visible choices in a dedicated context bar. **More** contains Skew, Perspective
and gesture guidance; an active advanced mode names itself on that button.
**Cancel** and **Done** stay at the right edge in every mode. Done applies the
shared transform command; Cancel leaves the document unchanged.

During a transform the bar replaces the generic tool strip. Tool settings,
Shift/Alt latches and persistent instructions do not compete with the editing
controls. Virtual modifier latches are suspended and restored on exit; physical
keyboard modifiers keep their shared-engine behavior. Entering transform also
ends an armed clone-source pick. Move settings can hide idle transform handles.
New group lives in layer properties/actions, making room for the labelled
Transform entry without increasing the Layers footer's height.

## Why this interface

The November 25 first pass exposed two commands and put the transform modes
inside another menu. That solved availability, but still reflected the desktop
command catalogue more than the task of manipulating an object. The maintainer
asked for a reassessment against excellent tablet experiences.

Two primary references informed the revision:

- [Procreate's transform interface](https://help.procreate.com/procreate/handbook/transform/transform-interface-gestures)
  combines on-object controls with visible transformation methods. Its example
  supports putting the common choices in front of the user instead of inside a
  Mode menu. [Apple's 2022 award](https://www.apple.com/newsroom/2022/06/apple-announces-winners-of-the-2022-apple-design-awards/)
  recognized Procreate for inclusivity; it is not evidence that this particular
  transformation pattern, or our implementation, has won an interaction award.
- [Affinity Photo 2's iPad Move tool](https://affinity.help/photo2ipad/en-US.lproj/pages/Tools/tools_move.html)
  combines selecting, moving, resizing and rotating. This supports keeping
  resize handles available with Move rather than requiring a separate mode just
  to discover them.

The design decision is direct manipulation with visible common modes and stable
completion controls. It is an adaptation to the existing engine, not a claim to
replicate those products. The established neutral theme tokens and shared
sans-serif typography remain: artwork carries colour; selected mode and Done
carry the accent. Controls have at least 44-point heights. The transform bar
fits one row in the tested 507-point Split View, portrait and landscape layouts.
No new palette, decorative frame, floating card or animation is needed here.

## Limits and acceptance

Warp is currently an exclusive mesh session in the shared engine. Resize and
Distort are disabled while it is active. Choose Done to retain the warp before
starting another transform; More → Discard warp explicitly removes its mesh.
The workspace does not silently throw away a warp when another mode is tapped.

Pencil or a mouse manipulates the object. Fingers keep the existing canvas
navigation policy. Rotation still uses the engine's outside-box drag, rather
than a dedicated visible rotation node. Those are remaining differences from
reference tablet experiences, and should be assessed with actual iPad use.
An accessibility or design award is not a substitute for that acceptance.

The [verification record](verification.md#focused-transform-interface--2026-11-25)
separates synthetic interaction, real rendered layout and browser/device checks.
To reproduce the original synthetic-art previews without opening a window:

```sh
cargo run --locked --example transform_preview -- .cache/transform-visuals
```

The example uses the actual engine, canvas, theme and workspace; it creates its
own disposable document and never opens personal artwork. Representative renders:
[landscape selection](images/transforms/landscape-selected.png),
[Split View resizing](images/transforms/split-resize.png), and
[portrait warp](images/transforms/portrait-warp.png).
