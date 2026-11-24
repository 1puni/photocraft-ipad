# Live browser control

An opt-in local relay connects the existing PhotoCraft MCP bridge to the editor
session visible in the browser. It sends requests into the shared editor's control
queue; command dispatch, Undo, delayed input and background-job replies stay in
PhotoCraft. There is no second headless document and no browser-click automation.

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
the tab visible: background-tab throttling can delay control. The first browser
session owns the relay until an explicit `relay.release` or server restart; a
second tab cannot silently steal its requests. Ordinary URLs without the fragment do not enable control.
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
lines protocol: first `auth` with `params.token`, then `ui.inspect`, or
`engine.execute` with `params.command` and `params.params`. Replies preserve IDs.
The browser polls `/control/next` and posts `/control/reply`; both require the
credential and its page-lifetime session identifier.

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
- No automatic recovery or reconnect-to-another-tab is added. File chunks have
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
uv run --no-project --python 3.12 python browser_file.py open /explicit/input.psd
uv run --no-project --python 3.12 python browser_file.py save /explicit/saved.psd
uv run --no-project --python 3.12 python browser_file.py export /explicit/preview.png
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
| `relay.release` | `{}` | Retire current lease; next opted-in page may attach |

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

1. Complete `browser_file.py save /explicit/handoff.psd` on the current controller.
2. Invoke MCP `control_call` with `{"method":"relay.release","params":{}}`.
   Pending requests must finish first; the retired page cannot reclaim the lease.
3. Open the secret opt-in URL on the intended new browser. Keep old pages intact.
4. Run `browser_file.py open /explicit/handoff.psd`, then inspect layers there.

The same release step permits a planned refresh of a saved page. A refresh makes
a new page-lifetime identity. A new protocol build may require restarting the
relay; protocol2 rejects old clients rather than quietly attaching stale code.
Preserve the old document until the new page's layer state and saved output are
verified. PSD transfer preserves document contents, not in-memory undo history.
