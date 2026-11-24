# Studio-bound documents

A local studio can open `/ipad/?webgl&studioPanel=PANEL&studioComposition=COMPOSITION`.
The link contains document identifiers, not a control credential. The browser opens
the PSD through the same-origin `/studio-api/documents/PANEL/COMPOSITION` adapter.
The local studio owns the library and persistence; this extension owns the browser
binding. Ordinary documents still save as downloads.

File → Save, the workspace Save button and Cmd-S use the shared editor save path.
Upstream patch 0010 adds an optional deferred writer: it queues the document ID,
revision and encoded bytes without setting `saved_revision`. The browser host marks
that revision saved only after the studio acknowledges the write. Edits made during
the request stay dirty. Export remains a download. Save As downloads an independent copy and detaches its studio binding, including across reload.

The binding and last acknowledged PSD hash are stored in session storage; local
recovery restores the document independently. A recovered bound document keeps its
base hash, so a different device's save is detected rather than overwritten. Opening
a fresh studio link fetches the current saved document. This is file-based editing,
not live multi-device canvas synchronization. A conflict leaves the local work intact:
save a download before reopening the newer studio version. A failed save does not
mark the document saved, and retrying identical bytes after a lost acknowledgement
is idempotent.

`studio_proxy.py` exposes only document GET/PUT routes through the already-local
server, forwarding to the studio on loopback port 4881. Writes require a matching
Origin and an allowed local host; neither links nor requests accept filesystem paths.
The studio validates the PSD and supported layer model before replacing anything,
keeps the exact PSD, extracts layers and preview, and retains prior checkpoints.
Current Nisse persistence supports 8-bit RGB plain pixel layers; unsupported groups,
masks, adjustments/effects or other formats return an explicit error, preserving the
previous file and allowing a download. The exact PSD is never flattened on save.

Older open tabs use their loaded application code. Keep their work, then use the
new studio link in another tab; don't reload an unsaved legacy tab to upgrade it.
The initial implementation does not attach an arbitrary old imported document to a
studio composition by filename. Binding always starts from an explicit studio link.

Tests: `node test_studio_store.mjs`, `uv run --no-project --python 3.12 python -m
unittest test_server test_studio_proxy`, the extension Rust checks, and the deferred
save regression in the engine patch. Nisse's server/storage tests belong to that
private studio, not this repository. Browser validation is recorded in verification.md.
