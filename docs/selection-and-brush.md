# From selection to mask painting

The selection bar keeps construction, actions and the next editing step in one
row. **Mode: New** opens New, Add, Subtract and Intersect. Choosing a mode clears
virtual Shift/Alt overrides so the visible choice controls the next gesture;
physical keyboard modifiers retain their usual behavior. **Selection** retains
Layer via Copy/Cut, Invert selection and Deselect. **Options** opens tool settings
and the virtual modifiers.

**Make mask** offers **Keep selected area** and **Hide selected area**, using the
shared layer-mask commands on the active layer. It is disabled without a selection
or active layer and during Quick Mask editing. It opens the mask inspector after
success. An existing pixel mask changes this action to **Edit mask**, preventing
this shortcut from replacing it. Existing vector masks remain independent.

The mask inspector exposes **Paint mask**, which explicitly targets that mask and
switches to Brush. Creating a mask preserves the layer image. Undo restores the
original document and selection. The engine's Keep command leaves the selection
active; Hide consumes it. When a selection remains, the inspector explains that
it limits painting and exposes **Deselect**. Clearing it is an explicit, separately
undoable edit; changing tools or opening controls does not discard it.

Brush-like tools now show live **Size** in pixels and **Opacity** as a percentage.
Each opens a labelled, 44-point-high slider rather than wrapping inline sliders
into multiple rows. **Brush** opens the brush inspector; **Options** opens per-tool
settings and virtual modifiers. Clone Stamp and Healing retain Set source and
Cancel source. Changes use the shared brush gesture/coalescing path.

This keeps the existing theme and the canvas-first layout. Selection modes cost
one extra tap in exchange for a stable bar at narrow widths. Brush values stay
visible while their larger sliders appear only when editing them. It is an alpha
interaction choice to validate on a physical iPad, not a claim of benchmark parity.

Real offscreen renders with synthetic artwork:

- [Selection in Split View](images/selection-brush/split-selection.png)
- [Mask controls in portrait](images/selection-brush/portrait-mask.png)
- [Brush opacity in Split View](images/selection-brush/split-brush-opacity.png)

Reproduce with `cargo run --locked --example transform_preview -- .cache/selection-visuals`.
Behavior and delivery checks belong in [verification](verification.md#selection-to-mask-and-brush-controls--2026-11-25).
