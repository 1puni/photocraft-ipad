# Browser autosave and reload recovery

Every editor tab saves an automatic local recovery copy, including pages without
an automation token. The Device · Tab badge shows **Unsaved changes** while the
copy is pending and **Recovery ready** after commit. Expand it for the last save
time or an actionable storage error. Changes are saved after a two-second pause,
or after ten seconds of continued changes once the pointer is released. Large
documents may take longer to encode and commit; use the status, not a stopwatch.

Reload restores all open documents, their order, active document, and selected
layers from the last committed copy. Native `.pcraft` encoding preserves the
layered document model, including pixels, masks and layer properties. Recovery
runs before the automation relay attaches, so remote edits cannot race it.
Recovered documents are marked unsaved and have no downloaded-file path. Undo
history, viewport, clipboard, in-progress strokes and live tool previews are not
part of the recovery copy.

This tab-recovery schedule is independent of the native desktop recovery timer
in File Handling preferences.

The copy is stored in IndexedDB on this browser/device and origin. Use the same
hostname and port; `localhost`, a LAN IP and a hostname are separate storage
locations. Reload and the browser’s Restore Closed Tab can retain tab identity;
a new independent tab is a new workspace. This is not cloud sync or a backup
across devices. Use Save/download for durable files and device handoff. Clearing
site data, private-browsing cleanup or browser storage eviction can remove local
recovery. Physical iPad suspension/crash behavior remains to be verified.

Snapshots replace the complete workspace in one transaction. Closing a document
and allowing the next autosave removes it from this tab’s recovery copy. A
quota/write failure retains the previous copy and reports failure; it does not
claim success. Corrupt snapshots are retained and automatic writes disabled for
that page rather than overwriting recoverable evidence. Save any new work as a
download. The editor requests a normal browser reload/close warning while changes
are pending, but mobile browsers do not always deliver this event. It does not
try to encode or save the document during unload.

A separate IndexedDB lease isolates copied tabs. A reload waits up to 6.5 seconds
for the previous page’s lease to expire. If another page still holds it, the new
tab takes a new identity and copies the recovery snapshot to that identity.
Every write checks ownership in the same transaction. A suspended page that lost
its lease cannot overwrite the new owner; save a download and reload that page.

## Verification commands

```sh
cargo test --locked
cargo check --locked --target wasm32-unknown-unknown
cargo clippy --locked --all-targets --no-deps -- -D warnings
cargo fmt --check
node test_session_identity.cjs
npm install --prefix .cache/recovery-tests --save-exact fake-indexeddb@6.2.8
node test_recovery_store.cjs
./build.sh
```

The native tests compare full encoded document bundles before and after recovery,
and reject corrupt snapshots without partially loading them. Storage tests cover
atomic replacement, independent tabs, duplicated tabs, stale ownership, quota
rollback, closing documents and the pending-change unload guard. The live-browser
record belongs in [verification](verification.md).

References: IndexedDB reports success at [transaction completion](https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction/complete_event),
not an individual write request. The [beforeunload event](https://developer.mozilla.org/en-US/docs/Web/API/Window/beforeunload_event)
is not reliable on all mobile lifecycle paths.
