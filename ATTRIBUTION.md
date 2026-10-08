# Attribution

| Material | Author / source | License |
|---|---|---|
| Linked engine, canvas, file services, command catalogue and shared UI | [PhotoCraft](https://github.com/storytold/photocraft), ArtCraft team and contributors; pinned in `upstream.env` | MIT OR Apache-2.0; original notices in NOTICE |
| `patches/photocraft/*.patch` | 1puni changes to PhotoCraft, original authors retained in each patch | MIT OR Apache-2.0 |
| Rust iPad workspace/input adapter, workbench and diagnostics | 1puni and contributors | MIT OR Apache-2.0 |
| `public/images/cover.svg` | Original vector artwork by 1puni, created with AI coding assistance | MIT OR Apache-2.0 |
| `public/images/workspace.jpg`, `public/images/colour-studio.jpg`, `public/images/mask-controls.jpg`, `public/images/brush-studio.jpg` | This project's browser UI displaying Hokusai's Great Wave (c. 1831); [artwork source](https://commons.wikimedia.org/wiki/File:The_Great_Wave_off_Kanagawa.jpg); [capture provenance](docs/visuals.md) | Artwork public domain; UI MIT OR Apache-2.0 |
| `experiments/segmentation/evidence/comparison.jpg` | Public-domain/CC0 scikit-image photographs and original synthetic fixtures; [individual sources](experiments/segmentation/ATTRIBUTION.md) | Source photographs public domain/CC0; original evaluation material MIT OR Apache-2.0 |

The linked UI uses PhotoCraft's icon and font facilities. Its full asset inventory
is [upstream ATTRIBUTION.md](https://github.com/storytold/photocraft/blob/5896f0b63e6f342e0311920b4d2a0ea5a94330fc/ATTRIBUTION.md).
No third-party fonts or ArtCraft brand marks are copied into this repository.
The build includes the upstream NOTICE, asset inventory and license texts beside
the compiled editor, including translation, magnetic-lasso and font notices.
`scripts/package-notices.sh` generates `dependency-licenses.html` using cargo-about
and the locked WebAssembly dependency graph. Generated distributions stay out of Git.

See [visual sources and reproduction](docs/visuals.md) for capture details.
