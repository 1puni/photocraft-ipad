# License review

Reviewed 8 October 2026 against the pinned PhotoCraft source in `upstream.env`,
its asset inventory, the locked WebAssembly graph and upstream's current terms.

**The source can be published under MIT OR Apache-2.0.** Both licenses permit
modification and redistribution, including commercial use. We retain the same
choice for our contribution, preserve PhotoCraft's notices, and identify our
changes in the patch series and Git history. No additional code-license
permission from upstream is required for this independent workspace.

## Code and identity

| Material | Terms and treatment |
|---|---|
| PhotoCraft code and our extension | [MIT](https://github.com/storytold/photocraft/blob/main/LICENSE-MIT) OR [Apache-2.0](https://github.com/storytold/photocraft/blob/main/LICENSE-APACHE); both texts and copyright notices retained |
| Upstream NOTICE | Retained with our own contribution identified; distributed alongside builds |
| ArtCraft name, wordmark and marks | [Separate brand terms](https://github.com/storytold/photocraft/blob/main/docs/brand/LICENSE-brand.txt); no ArtCraft logo artwork is included in this repo or our generated distribution. Plain-text upstream credit remains |
| PhotoCraft kitsune app icon | Explicitly MIT OR Apache-2.0 under [its own license](https://github.com/storytold/photocraft/blob/main/assets/app-icon/LICENSE.txt); distinct from the restricted ArtCraft marks |
| Our cover and screenshots | Original SVG and actual browser captures; the displayed Hokusai artwork is public domain. Sources in [ATTRIBUTION.md](../ATTRIBUTION.md) |

“PhotoCraft on iPad” describes the independent adaptation. We do not call it
ArtCraft or an official edition. Upstream credit identifies the foundation of
this independent project.

## Embedded assets and dependencies

The [pinned upstream inventory](https://github.com/storytold/photocraft/blob/5896f0b63e6f342e0311920b4d2a0ea5a94330fc/ATTRIBUTION.md)
covers Inter/JetBrains fonts (OFL), Lucide/Feather icons (ISC/MIT), the magnetic
lasso icon (CC BY 3.0, including author and modification attribution), SCOWL,
translations, and original public-domain ICC profiles. Packaging copies their
license texts, including the translation notice previously missing from our
build. Optional craft-fonts are excluded by upstream's WebAssembly build script.

`cargo-about 0.9.2` resolved **240 packages** with development-only dependencies
excluded and build dependencies included. Every package has a satisfiable license
expression under `about.toml`; selected families are MIT, Apache-2.0,
BSD-2-Clause, BSD-3-Clause, IJG, OFL-1.1, Ubuntu-font-1.0, Unicode-3.0 and Zlib.
`self_cell` offers Apache-2.0 OR GPL-2.0-only; we select Apache-2.0.
`epaint_default_fonts` has additional font terms, and `jpeg-encoder` includes IJG
terms. The required IJG acknowledgement is in NOTICE.

The generated `dependency-licenses.html` contains the actual license texts,
copyright notices, crate versions and source links. It contains no local manifest
paths. This report covers the configured Wasm graph, not a claim about every
optional native feature. Re-run the generator when dependencies change.

## Reproduce

```sh
cargo install cargo-about --version 0.9.2 --locked --features cli
./scripts/package-notices.sh .cache/notices-review
```

`build.sh` runs this same step and only replaces the served build after both
compilation and notice generation succeed. `--locked --fail` prevents a changed
lockfile or an unresolved license from silently producing a release artifact.
The launch page links Credits and Dependency licenses.

The standalone segmentation evaluation has its own [asset inventory](../experiments/segmentation/ATTRIBUTION.md).
Its committed illustrations use public-domain/CC0 fixtures. Pinned model weights
and runtime binaries are downloaded separately, outside Git and the editor build;
they are not redistributed as part of this source repository.
