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
session owns the relay until the server restarts; a second tab cannot silently
steal its requests. Ordinary URLs without the fragment do not enable control.
Treat the complete opt-in URL as a credential; do not share or commit it.

Import a saved PSD in the new tab through the editor's Open action. Save the old
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
- It grants no local filesystem roots. MCP `doc_open` / `doc_save` filesystem
  operations return the shared editor's missing-authority errors. Import and
  save through the browser's existing file picker/download actions.
- Requests are capped at 1 MiB, responses at 8 MiB, pending requests at 32.
  Disconnected browsers fail before dispatch. Timed-out queued requests expire;
  an already-dispatched command may finish. Inspect before retrying an edit.
- No automatic document transfer, recovery, reconnect-to-another-tab or retry
  of a browser mutation is added. A server restart permits a fresh owner.
- The shared UI may expose native-only methods; unsupported browser operations
  remain unsupported. This transport does not claim desktop parity.

Tests: `uv run --no-project --python 3.12 python -m unittest test_server test_control_relay`.
The relay tests exercise authentication, session ownership, exact TCP-to-browser
request/reply forwarding, malformed replies and expiry without replay. WASM
compilation and a real browser session are separate acceptance checks.
