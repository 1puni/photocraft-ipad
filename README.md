# PhotoCraft on iPad

Local review build. Nothing in this repository is approved for publication yet.

PhotoCraft's Rust engine, browser host hooks, and shared touch controls belong in the sibling
[1puni/photocraft](https://github.com/1puni/photocraft) fork (`ipad-browser` branch).
This repository owns the Rust iPad workspace and Pencil/finger input adapter, plus
serving, the launch page, device diagnostics, and integration acceptance. It links
to the sibling engine and canvas instead of keeping a second copy of editor code.
The full-port acceptance matrix is in [docs/ipad-port.md](docs/ipad-port.md).

## Run

Build with `./build.sh` (Rust, the wasm32-unknown-unknown target, and Trunk required).
Both checkouts must be siblings. A failed build preserves the previous served preview.

From this directory:

```sh
uv run --no-project --python 3.12 python server.py
```

Open http://localhost:4876 on the Mac or http://v.local:4876 on the same network.
The preview serves only `public/`, never source, Git metadata, or personal files.
Image editing and file contents remain in the browser. Save downloads a file;
there is currently no automatic document recovery in the upstream web build.

The verified official v0.3.0 archive is staged under
`public/editor/photocraft-web-0.3.0/`. Its SHA256 is
`1825b2beb2b84331124f6cb78e5eb6eb9584612e8ad6a4e817b7f27cb3f93d06`.
The locally built tablet variant goes in `public/ipad/`.

## Browser and pen

Start with Safari on iPad. The LAN HTTP preview uses WebGL2; WebGPU requires a
trusted HTTPS origin (or localhost on the device actually running the browser).
The actual Pencil is needed to verify pressure and tilt. Pencil USB-C has no
pressure sensor. The diagnostic page displays observed values, not inferred support.
Squeeze, double-tap, haptics and barrel roll are not promised by this web build.

USB alone does not grant remote control. Unlock and trust the Mac; enable Safari
Settings → Advanced → Web Inspector for USB inspection. A human still makes the
physical Pencil strokes. No App Store membership or native iPad app is needed.

## Review before publication

- Inspect both repository diffs and user-facing copy with V.
- Verify iPad launch, Pencil strokes, palm rejection, navigation, save and reopen.
- Review the documentation's limits against measured results.
- Do not push, publish, or submit upstream until V approves after that review.
