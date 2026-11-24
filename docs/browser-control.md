# Live browser control

An opt-in local relay connects the existing PhotoCraft MCP bridge to the editor
session visible in the browser. It sends requests into the shared editor's control
queue; command dispatch, Undo, delayed input and background-job replies stay in
PhotoCraft. There is no second headless document and no browser-click automation.

Desktop browser import, four-layer PSD save/readback and PNG export-copy have
been verified through real upstream MCP; see [the test record](verification.md).
Viewport screenshots timed out on the tested path; use export-copy for previews.

Build this checkout normally (`./setup.sh`, `./build.sh`). Start a separate local
instance so an existing browser's unsaved document stays untouched:

```sh
mkdir -p .runtime
uv run --no-project --python 3.12 python -c 'import os,secrets; p=".runtime/control.token"; fd=os.open(p,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600); os.write(fd,(secrets.token_hex(32)+"\n").encode()); os.close(fd)'
uv run --no-project --python 3.12 python control_relay.py \
  --host 127.0.0.1 --port 4877 --control-port 7879 \
  --token-file .runtime/control.token
```

Open `http://127.0.0.1:4877/ipad/#control=<token-file contents>` in a **new** tab.
The fragment is opt-in and is not sent in ordinary HTTP requests. The browser
sends the credential in an Authorization header to the same-origin relay. Keep
the tab visible: background-tab throttling can delay control. Each tab registers a named target. List sessions and select an exact tab on
your control connection before sending editor commands (see below). Ordinary URLs without the fragment do not enable control.
Treat the complete opt-in URL as a credential; do not share or commit it.

Use the byte-transfer helper below to open a PSD without a picker. Save the old
tab before switching if it has changes you want to carry forward. Do not reload
an unsaved tab to update its code.

Build the existing CLI in the sibling engine checkout, then start MCP:

```sh
cargo build --locked --manifest-path ../photocraft/Cargo.toml -p photocraft-cli
../photocraft/target/debug/photocraft-cli mcp \
  --bridge 127.0.0.1:7879 --control-token-file "$PWD/.runtime/control.token"
```

The process uses the existing MCP stdio protocol. Configure an MCP client with
that executable and those arguments; no new tool registry is needed. `ui_inspect`
and `doc_inspect` read the visible session. `command_list` discovers commands;
`command_run` executes them. `doc_render_preview` in bridge mode uses the shared
window screenshot path, subject to browser renderer support.

For transport diagnostics, the loopback TCP endpoint accepts the existing JSON
lines protocol: first `auth` with `params.token`, then `session.list` and `session.select` with `params.id`, followed by `ui.inspect`, or
`engine.execute` with `params.command` and `params.params`. Replies preserve IDs.
The browser posts `/control/poll` and `/control/reply`; both require the
credential, stable tab identifier and page-generation identifier. Older protocol-2
builds use the single-owner `/control/next` route and `relay.release`.

## Boundaries and limitations

- The TCP endpoint is loopback only. The HTTP listener also defaults to loopback;
  pass a specific LAN address only when using the editor on the trusted LAN.
  HTTP on a LAN is unencrypted. This is a local development relay, not a public
  deployment service.
- It grants no local filesystem roots. Browser `automation_read` /
  `automation_write` services access only a bounded, page-lifetime memory store.
  Upstream MCP `doc_open` / `doc_save` use plain staged names; no browser picker
  is needed. The local helper alone reads/writes the explicit paths it is given.
- Requests are capped at 1 MiB, responses at 8 MiB, pending requests at 32.
  Disconnected browsers fail before dispatch. Timed-out queued requests expire;
  an already-dispatched command may finish. Inspect before retrying an edit.
- Tab-local recovery is described in [browser recovery](browser-recovery.md).
  Control never reconnects to a different tab automatically. File chunks have
  explicit offsets; identical repeated chunks are idempotent. Document edits
  must be inspected before retrying an unknown outcome.
- The shared UI may expose native-only methods; unsupported browser operations
  remain unsupported. This transport does not claim desktop parity.

Tests: `uv run --no-project --python 3.12 python -m unittest test_server test_control_relay`.
The relay tests exercise authentication, session ownership, exact TCP-to-browser
request/reply forwarding, malformed replies and expiry without replay. WASM
compilation and a real browser session are separate acceptance checks.


## Open, save and preview without a picker

After the page opts in, these commands use the existing MCP stdio client and
browser importer/exporter. They do not click the browser or expose a filesystem
root to it:

```sh
uv run --no-project --python 3.12 python browser_file.py --session <tab-id> open /explicit/input.psd
uv run --no-project --python 3.12 python browser_file.py --session <tab-id> save /explicit/saved.psd
uv run --no-project --python 3.12 python browser_file.py --session <tab-id> export /explicit/preview.png
```

`open` uploads chunks, invokes upstream `doc_open`, removes the staging bytes and
returns `doc_inspect`. `save` invokes upstream `doc_save`, downloads chunks to a
sibling temporary file, fsyncs and atomically replaces the requested output.
`export` exports a copy without changing the document's path or saved revision;
PNG export provides a document preview even where a viewport screenshot stalls.
Each successful operation prints its byte count and SHA-256. `--cli`, `--bridge`
and `--token-file` override the local defaults.

The transfer protocol is exposed through existing MCP `control_call`:

| Method | Params | Result |
| --- | --- | --- |
| `transfer.begin` | `name`, `size` | Allocate bounded staging slot |
| `transfer.write` | `name`, `offset`, `base64` | Received byte count, completeness |
| `transfer.info` | `name` | Expected and received size |
| `transfer.read` | `name`, `offset`, `length` | Base64 bytes from a complete file |
| `transfer.remove` | `name` | Drop staged bytes |
| `transfer.export` | `name` (extension chooses format) | Export copy into staging |
| `relay.release` | `{}` | Legacy protocol 2 only: retire the current exclusive lease |

Chunks are at most 512 KiB (below the upstream 1 MiB request budget after base64),
files at most 64 MiB, and total reserved staging at most 128 MiB / 16 files.
Names are plain filenames, never paths. Incomplete uploads cannot be opened.
Staged bytes disappear on reload; the browser's "Saved" state after `doc_save`
means the encoder wrote to this memory store. Only a successfully completed
helper `save` confirms an actual local file; a failed download must be retried
before closing. Use `export` for a copy that leaves the saved-state indicator alone.

## Explicit desktop / iPad handoff

An ordinary `/ipad/?webgl` URL gives an independent browser document. It does
not opt into control and does not share another device's canvas automatically.
To move the working document without a picker:

1. List sessions and identify the source device/tab by its exact ID.
2. Complete `browser_file.py --session SOURCE_ID save /explicit/handoff.psd`.
3. Open the secret opt-in URL on the intended destination browser; name its tab.
4. List sessions again, then run `browser_file.py --session DESTINATION_ID open
   /explicit/handoff.psd` and inspect the layers there.

A refresh preserves the tab ID but changes the page generation. Existing control
connections must select again; there is no queued-edit replay. Local autosave restores the last committed
recovery copy before the control connection attaches. Keep the old document until the destination layers and saved output are
verified. PSD transfer preserves document contents, not in-memory undo history.
Already-open protocol-2 pages use the legacy exclusive `relay.release` handoff;
save them before upgrading both the relay and page to protocol 3.

## Named device and tab sessions (protocol 3)

Each opted-in editor now shows a **Device · Tab** badge. Expand it to name the
device and tab; the browser title includes those names and the current document.
Device identity/name lives in local storage for this browser origin. Tab identity
and label live in session storage and survive reload; page generation changes on
every load. These are routing metadata, never authentication credentials. Different
hostnames/ports and browser profiles have separate storage identities.

A reload restores this tab’s last committed [local recovery copy](browser-recovery.md).
It does not synchronize devices or restore undo history. Keep using the file-save
workflow for portable backups and cross-device handoff.

Discover and target the actual visible tab through existing MCP `control_call`:

- `session.list`, `{}` returns IDs, device/tab names, document name, current page
  generation, connected status and last-seen age.
- `session.select`, `{"id":"<exact tab id>"}` binds **this MCP connection** to
  that tab and current page generation. Then use normal `doc_inspect`, `doc_open`,
  commands and file transfers. Other MCP clients have independent selections.
- No named session is implicitly selected, even if there is only one. A reload
  invalidates old selections and pending requests; select again and inspect.
  A controller reconnect also requires reselection, preventing automatic replay
  of mutations against another page.

The file helper supports the same target selection:

```sh
uv run --no-project --python 3.12 python browser_file.py sessions
uv run --no-project --python 3.12 python browser_file.py --session TAB_ID open /explicit/panel.psd
uv run --no-project --python 3.12 python browser_file.py --session TAB_ID save /explicit/panel.psd
```

Keep the intended editor tab visible. A tab is offline after five seconds without
a heartbeat. Reload can take about five seconds to reclaim its stable identity;
a concurrent duplicated tab gets a new ID instead of sharing a document target.
The relay retains at most 64 entries and prunes long-offline entries when full.
Labels are descriptive, not trusted routing keys; always select the exact ID.

Protocol 2 remains supported for already-open legacy pages. Its exclusive lease
and `relay.release` are legacy behavior, not needed for named tabs. Once a relay
has named sessions, controllers must select one explicitly. Existing old tabs do
not gain the badge until they load the new build; save their artwork first.
