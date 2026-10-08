# Selection comparison — 8 October 2026

**Pick: full-precision EdgeTAM for a learned selection prototype, with cached image
embeddings and remembered positive/negative prompts.** Keep PhotoCraft's existing
colour/graph-cut tools and edge refinement. This is a measured candidate decision,
not a shipped editor feature or a general model leaderboard.

V requested trying the variants and choosing the most sensible one. V also asked
why inference was local rather than on the server. Local processing was our initial
assumption, not V's requirement. V is receptive to it. Server inference remains a
valid alternative; no server inference service was deployed or rented.

## What ran

- The actual Rust PhotoCraft Magic Wand, Quick Selection and rectangle GrabCut.
- Existing independent Quick Selection add/subtract corrections, starting from the
  rectangle result, and an experimental retained-constraint GrabCut solve.
- MobileSAM FP32, EdgeTAM FP32, EdgeTAM quantized encoder/decoder, and EdgeTAM FP32
  encoder plus quantized decoder. Each learned model received a point, box plus
  point, and box plus the same complete positive/negative prompt history.
- Seven fixtures: public-domain astronaut, cat and coffee photos; four synthetic
  cases covering contrasting colour, similar colour, a multicolour object touching
  a same-colour distractor, and a hole plus a thin line. The coffee target is the
  cup including its handle, excluding saucer and spoon. Fixed marks are in
  `compare.py`; they are not chosen separately to favour each model.

The [contact sheet](evidence/comparison.jpg) shows corrected results. Darkening
indicates excluded pixels. It is a binary selection comparison, not alpha matting.
The [raw results](evidence/native-results.json) contain 119 case/variant records.

## Results and decision

| Variant | Observation | Decision |
|---|---|---|
| Existing algorithms | Strong synthetic boundaries, but the cat loses much of its head; subtracting a similar-colour region can wipe out the target | Retain for colour selection; improve correction semantics |
| Remembered constraints | Avoids the similar-colour wipeout and thin-line/background flood; does not provide learned object recognition | Useful complementary change, not a replacement for ML |
| MobileSAM FP32 | Good photo masks, but failed to remove the synthetic hole even with the negative prompt; slow on the iPad | Not the default candidate |
| EdgeTAM FP32 | Better synthetic hole handling, much faster on the iPad; cat mask still has eye/edge defects | **Selected baseline** |
| EdgeTAM fully quantized | Severe mask deterioration, including empty masks | Reject this particular export |
| EdgeTAM quantized decoder only | Nearly unchanged synthetic score natively, modest WASM speed benefit, catastrophic WebGPU mismatch in the tested stack | Reject for now; don't trade correctness for 12 MB |

Mean binary IoU across **only the four synthetic fixtures**: existing rectangle
0.9758; existing correction sequence 0.4418; remembered constraints 0.9759;
MobileSAM corrected 0.9446; EdgeTAM corrected 0.9718; fully quantized EdgeTAM 0.0518;
quantized-decoder EdgeTAM 0.9718. This deliberately small, constructed set tests
specific behaviour. It does not rank photo segmentation quality. Photos were
inspected visually and have no ground-truth scores. Model-reported mask-quality
scores are predictions, not measured accuracy.

### Actual connected iPad: Safari 26.3, current HTTP LAN preview

The benchmark ran in a separate frame/worker without replacing the Rust editor or
reading its document. The device reported `isSecureContext=false`, no `navigator.gpu`,
and eight logical cores. ONNX Runtime Web 1.23.2 used **single-threaded WASM CPU**.

| Model | Image encoder | First prompt decoder | Three further decoder runs |
|---|---:|---:|---:|
| MobileSAM FP32 | 9.680 s | 1.182 s | 1.389 / 1.441 / 1.527 s |
| EdgeTAM FP32 | 3.650 s | 0.308 s | 0.473 / 0.366 / 0.389 s |

These timings exclude model loading (encoder session creation was 2.967 s and
0.907 s respectively), image preparation, UI drawing, and downloads of prompt
fixtures. Encoder output was actually produced on the device and fed to its
decoder. Scores matched the native reference to approximately 1e-6. The first
device run used the earlier coffee prompt set; the later visual comparison uses
the corrected cup-only set. The complete raw device receipts preserve both the
input reference scores and outputs. A second run with pixel-mask parity and
quantized variants was requested through Web Inspector but did not return; those
extra iPad checks are **not verified**.

### Mac Chrome: secure localhost, M3 Max, GPU path

Full-precision EdgeTAM produced **identical thresholded reference-mask pixels** on
the cup fixture. Image encoder: 106.8 ms; first decoder: 192.9 ms; repeated decoder:
25.0 / 72.2 / 27.8 ms. Loading/compilation and input preparation are separate.
This proves a functioning browser GPU path on this Mac, **not iPad GPU speed**.

The mixed-precision model's WebGPU mask IoU against its own native reference was
only 0.00767 on two attempts. On Chrome WASM it was 0.99877, while FP32 was exactly
1.0. The GPU failure has not been isolated to an ONNX operator or export/runtime
defect; it does not prove quantization is intrinsically unsuitable. It is sufficient
to exclude this variant from the current choice. Older receipts use `complete`
to mean execution completed, not parity passed; the harness now reports parity
separately and flags these mismatches.

## Browser, Rust and server boundary

PhotoCraft remains Rust compiled to WebAssembly. These Python, Rust and JavaScript
files are a **standalone evaluation harness**, outside the editor build. Model
inference ran locally through ONNX Runtime; the lab server only served public
fixtures/models and accepted bounded benchmark receipts. No private image was
uploaded and no production runtime dependency was added.

The selected FP32 ONNX pair is 40,896,123 bytes (~39 MiB); MobileSAM is 44,653,652;
mixed precision is 28,677,232; fully quantized is 14,214,211. Cache models once and
image embeddings per image revision. Corrections should re-run only the decoder.
An image change must invalidate the cached embedding; a view zoom must not.

For integration, keep selection/history/refinement in the Rust engine. Choose a
browser inference adapter deliberately; this experiment does not establish a
pure-Rust inference implementation. Use a worker so segmentation cannot freeze
Pencil input. Keep explicit positive/negative marks, cancellation and stale-result
rejection. Reuse the current edge refinement first; hair/transparency quality
needs its own matting evaluation. The retained-constraint prototype here also
needs full-resolution hard-seed enforcement before production use.

Local inference permits offline use after assets are cached and avoids image
uploads. A server can run larger models and offload device work, but requires an
explicit image-transfer/session design and separate end-to-end network benchmarks.
Do not turn local execution into a product requirement merely because this test
used it. iPad WebGPU needs a trusted secure origin; current LAN HTTP cannot provide
it. HTTPS/iPad GPU acceptance and product integration are still future work.

Peak browser memory and large 24–36 MP workflows were not measured. The fixtures
are 300–600 pixels across before the learned models resize their inputs. No live
Pencil selection UX, hair alpha-matte benchmark or full-resolution correction
acceptance is implied by these results.

## Reproduce

PhotoCraft engine revision tested: `af949b10f9ebaa206993659d25834a1930623b98`.
The standalone Rust crate uses the same sibling `../photocraft` layout as the app.
Model URLs, pinned revisions, byte sizes and SHA-256 hashes are in
[models.json](evidence/models.json). Runtime versions: ONNX Runtime native 1.30.0
(CPU, four intra-op threads); ONNX Runtime Web 1.23.2 (one WASM thread).
Native timings were collected on a working Mac with other activity and are
diagnostic, not a controlled server benchmark.

From this repository's root, with a disposable lab directory:

```sh
LAB=/absolute/path/to/segmentation-lab
uv venv --python 3.11 "$LAB/.venv"
uv pip install --python "$LAB/.venv/bin/python" -r experiments/segmentation/requirements.txt
uv run --no-project --python "$LAB/.venv/bin/python" python experiments/segmentation/prepare.py --lab "$LAB"
cargo build --release --manifest-path experiments/segmentation/classical/Cargo.toml
uv run --no-project --python "$LAB/.venv/bin/python" python experiments/segmentation/compare.py --lab "$LAB" --models mobile edge edge_quantized edge_qdecoder
npm pack onnxruntime-web@1.23.2 --pack-destination "$LAB"
tar -xzf "$LAB/onnxruntime-web-1.23.2.tgz" -C "$LAB"
uv run --no-project --python "$LAB/.venv/bin/python" python experiments/segmentation/prepare.py --lab "$LAB" --stage-browser
uv run --no-project --python "$LAB/.venv/bin/python" python experiments/segmentation/serve.py --lab "$LAB"
```

Open port 4879 on the Mac's LAN address for iPad WASM, or localhost for Mac WebGPU.
`?names=edge` selects the chosen model; `?auto&names=edge` automatically runs WASM.
Avoid overlapping runs. The receipt server exposes only the disposable `public/`
directory; model files/runtime binaries are not committed or added to the editor.
The script's trusted generated fixture inputs are not a public API.

See [asset attribution](ATTRIBUTION.md). This decision and evidence are retained
locally; publication is coordinated separately with the repository release work.

Validation: the Rust evaluation executable built in release mode and passed
Clippy with warnings denied; Python modules compiled; the browser worker passed
Node's syntax check. All 119 records are unique, every output mask has the
expected dimensions, the synthetic correction failure/recovery checks passed,
and the contact sheet was decoded and visually inspected. The browser receipts
provide actual inference checks beyond compilation. Editor tests were not
rerun because this experiment does not change its source or dependencies.
