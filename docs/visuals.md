# Visual sources

The cover is original SVG in `public/images/cover.svg`: a pencil gesture, lavender
paper (`#e8e5f2`), graphite (`#282535`) and muted violet (`#8273ab`), with a small
winking horn for 1puni. Georgia carries the headline; system sans-serif carries
supporting text. No brand assets or fonts are extracted from upstream.

Design brief: match the care and visual richness of PhotoCraft's presentation,
show our actual iPad workspace, and give 1puni a small wink. Lead with the work.
The cover's source is editable; it needs no image-generation service to reproduce.

The workspace image is captured from the v0.1 browser build while displaying
Hokusai's *The Great Wave off Kanagawa*, c. 1831, public domain.
[Wikimedia source](https://commons.wikimedia.org/wiki/File:The_Great_Wave_off_Kanagawa.jpg).

To reproduce: build and serve this repo, open `/ipad/?webgl`, import the artwork
through Open, choose Fit, leave Layers visible and capture the page viewport.
For the brush capture, choose Brush, then “All brush dynamics & preview”. Captures
were made in Chrome at 1272×846 CSS pixels on 8 October 2026, from the optimized
extension snapshot `06495ac` and patched engine tree in `upstream.env`.
Keep browser chrome, private files and unrelated tabs out of the image.
