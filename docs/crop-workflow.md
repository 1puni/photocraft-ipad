# Crop beside the canvas

Choose Crop, then use **Crop: Free** to select an aspect ratio: Original, Square,
4:3, 3:4, 16:9 or 9:16. The frame previews a preset immediately, centred inside
the current crop region. Trying several presets keeps the same basis until you
drag the frame, so exploring ratios does not progressively shrink the result.
Free releases the ratio constraint without resetting the frame.

**Outside: Keep** retains pixels beyond the new canvas boundary; a later canvas
expansion can reveal them. **Outside: Delete** removes those pixels when applied.
This is the existing engine preference, now visible beside the crop. The workspace
does not silently change the chosen policy.

**Apply** commits the crop through the shared command and Undo restores it.
Apply is disabled on the untouched default frame; returning to the original
frame after trying presets restores that protection. **Cancel** discards the
pending frame and its gesture state without editing the document. The Crop tool
then displays its normal default frame again. Both controls stay at the right
edge when choosing a different ratio or retention policy.

This bar replaces the generic tool strip while cropping, including virtual
Shift/Alt latches; those latches are restored when leaving direct manipulation.
Physical keyboard modifiers retain their engine behavior. The inspector's crop
ratio controls share the same immediate-preview helper. Pencil/mouse manipulates
the frame; fingers retain canvas navigation. Crop does not add rotation/straighten.

The interface uses the existing neutral theme, selected-state accent and
44-point controls. There are two visible choices and two completion actions,
fitting a single row at the tested 507-point Split View width. Its purpose is to
make the crop result and pixel-retention decision inspectable before committing.

[Split View preview](images/crop/split-crop.png) and
[landscape preview](images/crop/landscape-crop.png) and
[light-theme Split View](images/crop/split-crop-light.png) show the real offscreen-rendered
workspace with original synthetic artwork. Reproduce with
`cargo run --locked --example transform_preview -- .cache/manipulation-visuals`.
An optional second argument selects `pro`, `proMedium`, `studio`, `studioLight`
or `classic` for theme review. See the
[verification record](verification.md#rotation-and-crop-workflows--2026-11-25)
for behavioral checks and the still-open device acceptance.
