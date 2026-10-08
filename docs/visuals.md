# Visual sources

The cover is original SVG in `public/images/cover.svg`: a pencil gesture, lavender
paper (`#e8e5f2`), graphite (`#282535`) and muted violet (`#8273ab`), with a small
winking horn for 1puni. Georgia carries the headline; system sans-serif carries
supporting text. No brand assets or fonts are extracted from upstream.

Design brief: match the care and visual richness of PhotoCraft's presentation,
show our actual iPad workspace, and give 1puni a small wink. Lead with the work.
The cover's source is editable; it needs no image-generation service to reproduce.

The editor screenshots are captured from the v0.1 browser build while displaying
Hokusai's *The Great Wave off Kanagawa*, c. 1831, public domain.
[Wikimedia source](https://commons.wikimedia.org/wiki/File:The_Great_Wave_off_Kanagawa.jpg).

The workspace, Colour studio and mask-controls captures were refreshed on
8 October 2026 from the optimized `d9f9605` build, in Chrome at 1272×846 pixels.
They show the actual page viewport, without browser chrome or image compositing.
The engine tree remains the one recorded in `upstream.env`.

To reproduce: build and serve this repo, open `/ipad/?webgl` in a separate tab,
import the artwork through Open and choose Fit. Unlock the background, name it
“The Great Wave” and add a white pixel mask. Capture Layers for `workspace.jpg`;
open its mask controls in Composite view for `mask-controls.jpg`. Select the image
target and open Colour studio, choosing a blue foreground (`#4578b9` in this
capture), for `colour-studio.jpg`. The artwork pixels are unchanged. No personal
document, saved browser palette or unrelated tab is included.

The linked brush-studio capture remains the earlier `82be479` snapshot, made in
Chrome at 1272×902 pixels on the same date. To reproduce it, choose Brush, then
“All brush dynamics & preview”.
