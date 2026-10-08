# Contributing

This is an independent iPad browser workspace built on PhotoCraft. Credit the
upstream engine and keep changes small, understandable, and easy to move upstream.
The repository is currently private while its first public release is prepared.

Start with [setup](README.md#build-and-try-locally) and the
[acceptance matrix](docs/ipad-port.md). Report extension issues here. Before
reporting an engine bug upstream, reproduce it in an unmodified PhotoCraft build
and search their existing issues. Do not send maintainers an iPad-specific support
request disguised as an upstream bug.

The editor and input adapter are Rust. Document changes, add focused regressions
for behavior changes, run `cargo fmt --check`, `cargo test --locked`,
`cargo clippy --locked --all-targets --no-deps -- -D warnings`, and
`cargo check --locked --target wasm32-unknown-unknown`. Keep the default local
`target/` directory: sharing a target with another extension checkout previously
linked stale UI. Server
changes also require `uv run --no-project --python 3.12 python -m unittest test_server`.
Inspect changed UI at portrait, landscape and Split View widths. Distinguish
synthetic input from physical Pencil evidence; include device/browser versions.

Contributions use MIT OR Apache-2.0. Keep existing notices. Use original or
permissively licensed assets with sources in ATTRIBUTION.md; never personal photos
in fixtures or screenshots. Do not copy ArtCraft's restricted brand marks.
AI assistance is welcome when disclosed; contributors remain responsible for
review, provenance and evidence.

See [upstream collaboration](docs/upstream.md) before proposing a patch to
PhotoCraft. No upstream contact or PR has been made for this workspace yet.
