"""Opt-in local TCP-to-browser transport for the existing PhotoCraft MCP bridge.

No document engine or filesystem authority lives here. The browser owns its session.
"""
import argparse
from collections import deque
from functools import partial
import hmac
import json
from pathlib import Path
import queue
import secrets
import socketserver
import threading
import time
from http.server import ThreadingHTTPServer
from server import Preview, PUBLIC
from named_sessions import Sessions

MAX_REQUEST = 1 << 20
MAX_RESPONSE = 8 << 20


class Relay:
    def __init__(self, token):
        self.token = token
        self.named = Sessions()
        self.lock = threading.Lock()
        self.session = None
        self.retired = set()
        self.seen = 0
        self.pending = {}
        self.queued = deque()

    def poll(self, session):
        with self.lock:
            if session in self.retired:
                raise ValueError("this browser lease was released; use the chosen new page")
            if self.session not in (None, session):
                raise ValueError("another browser owns this relay; restart to switch")
            self.session = session
            self.seen = time.monotonic()
            while self.queued:
                request = self.queued.popleft()
                if request["id"] in self.pending:
                    return request
            return None

    def release(self):
        with self.lock:
            if self.pending:
                return {"ok": False, "error": "wait for pending requests before releasing the browser"}
            if len(self.retired) >= 32:
                return {"ok": False, "error": "restart relay after 32 browser handoffs"}
            if self.session is not None:
                self.retired.add(self.session)
            self.session = None
            self.seen = 0
            return {"ok": True, "result": {"released": True, "nextBrowserMayAttach": True}}

    def reply(self, session, request_id, reply):
        with self.lock:
            if session != self.session:
                raise ValueError("wrong browser session")
            waiter = self.pending.pop(request_id, None)
            if waiter is not None:
                waiter.put_nowait(reply)

    def call(self, method, params, timeout=30):
        waiter = queue.Queue(maxsize=1)
        request_id = secrets.token_hex(16)
        with self.lock:
            if self.session is None or time.monotonic() - self.seen > 5:
                return {"ok": False, "error": "no active browser; keep the opted-in editor tab visible"}
            if len(self.pending) >= 32:
                return {"ok": False, "error": "control queue is full"}
            self.pending[request_id] = waiter
            self.queued.append({"id": request_id, "method": method, "params": params})
        try:
            return waiter.get(timeout=timeout)
        except queue.Empty:
            return {"ok": False, "error": "browser timeout; operation may have completed; inspect before retrying"}
        finally:
            with self.lock:
                self.pending.pop(request_id, None)
                # Remove expired requests so reconnecting cannot run a stale edit.
                self.queued = deque(r for r in self.queued if r["id"] != request_id)


class RelayHTTP(Preview):
    def authorized_session(self):
        supplied = self.headers.get("Authorization", "")
        if not hmac.compare_digest(supplied.encode(), ("Bearer " + self.server.relay.token).encode()):
            self.send_error(403)
            return None
        if self.headers.get("X-PhotoCraft-Protocol") not in ("2", "3"):
            self.send_error(426, "reload the editor to use the current control protocol")
            return None
        session = self.headers.get("X-PhotoCraft-Session", "")
        if not session or len(session) > 128:
            self.send_error(400)
            return None
        return session

    def json_response(self, value, status=200):
        body = json.dumps(value).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        if self.path != "/control/next":
            return super().do_GET()
        session = self.authorized_session()
        if session is None:
            return
        try:
            self.json_response(self.server.relay.poll(session))
        except ValueError as error:
            self.json_response({"error": str(error)}, 409)

    def do_POST(self):
        if self.path == "/control/poll":
            return self.named_poll()
        if self.path != "/control/reply":
            self.send_error(404)
            return
        session = self.authorized_session()
        if session is None:
            return
        try:
            size = int(self.headers.get("Content-Length", "0"))
            if not 0 < size <= MAX_RESPONSE:
                raise ValueError("invalid response size")
            value = json.loads(self.rfile.read(size))
            if not isinstance(value, dict) or not isinstance(value.get("id"), str) or not isinstance(value.get("reply"), dict):
                raise ValueError("invalid response envelope")
            if self.headers.get("X-PhotoCraft-Protocol") == "3":
                self.server.relay.named.reply(session, self.headers.get("X-PhotoCraft-Page", ""), value["id"], value["reply"])
            else:
                self.server.relay.reply(session, value["id"], value["reply"])
            self.json_response({"ok": True})
        except (ValueError, TypeError) as error:
            self.json_response({"error": str(error)}, 400)

    def named_poll(self):
        session = self.authorized_session()
        if session is None:
            return
        try:
            size = int(self.headers.get("Content-Length", "0"))
            if not 0 < size <= 4096 or self.headers.get("X-PhotoCraft-Protocol") != "3":
                raise ValueError("invalid session registration")
            value = json.loads(self.rfile.read(size))
            if not isinstance(value, dict) or value.get("id") != session or value.get("page") != self.headers.get("X-PhotoCraft-Page"):
                raise ValueError("identity/header mismatch")
            self.json_response(self.server.relay.named.poll(value))
        except (ValueError, TypeError) as error:
            self.json_response({"error": str(error)}, 400)

    def setup(self):
        super().setup()
        self.connection.settimeout(35)

    def log_message(self, fmt, *args):
        # Polling is frequent. Never log credentials or document/control contents.
        if self.path not in ("/control/next", "/control/reply", "/control/poll"):
            super().log_message(fmt, *args)


class ControlTCP(socketserver.StreamRequestHandler):
    def handle(self):
        self.connection.settimeout(40)
        authenticated = False
        selected = None
        try:
            while True:
                raw = self.rfile.readline(MAX_REQUEST + 1)
                if not raw:
                    return
                if len(raw) > MAX_REQUEST:
                    self.write_reply({"id": None, "ok": False, "error": "request exceeds budget"})
                    return
                try:
                    msg = json.loads(raw)
                    if not isinstance(msg, dict):
                        raise ValueError("request must be an object")
                    request_id = msg.get("id")
                    if not authenticated:
                        params = msg.get("params", {})
                        token = params.get("token", "") if isinstance(params, dict) else ""
                        authenticated = msg.get("method") == "auth" and isinstance(token, str) and hmac.compare_digest(token.encode(), self.server.relay.token.encode())
                        self.write_reply({"id": request_id, "ok": authenticated, **({"result": {"authenticated": True}} if authenticated else {"error": "authentication failed"})})
                        if not authenticated:
                            return
                        continue
                    method = msg.get("method")
                    params = msg.get("params", {})
                    if not isinstance(method, str) or not isinstance(params, dict):
                        raise ValueError("expected method string and params object")
                    if method == "session.list":
                        reply = {"ok": True, "result": {"sessions": self.server.relay.named.list()}}
                    elif method == "session.select":
                        try:
                            selected = self.server.relay.named.select(params.get("id"))
                            reply = {"ok": True, "result": {"id": selected[0], "page": selected[1]}}
                        except (ValueError, TypeError) as error:
                            selected = None
                            reply = {"ok": False, "error": str(error)}
                    elif selected is not None or self.server.relay.named.list():
                        reply = self.server.relay.named.call(selected, method, params)
                    else:
                        reply = self.server.relay.release() if method == "relay.release" else self.server.relay.call(method, params)
                    self.write_reply({**reply, "id": request_id})
                except (ValueError, TypeError):
                    self.write_reply({"id": None, "ok": False, "error": "invalid JSON request"})
        except (OSError, TimeoutError):
            return

    def write_reply(self, value):
        raw = json.dumps(value).encode() + b"\n"
        if len(raw) > MAX_RESPONSE:
            raw = json.dumps({"id": value.get("id"), "ok": False, "error": "response exceeds budget; operation may have completed"}).encode() + b"\n"
        self.wfile.write(raw)
        self.wfile.flush()


class TCPServer(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True

    def __init__(self, *args, **kwargs):
        self.slots = threading.BoundedSemaphore(16)
        super().__init__(*args, **kwargs)

    def process_request(self, request, client_address):
        if not self.slots.acquire(blocking=False):
            self.shutdown_request(request)
            return
        try:
            super().process_request(request, client_address)
        except BaseException:
            self.slots.release()
            raise

    def process_request_thread(self, request, client_address):
        try:
            super().process_request_thread(request, client_address)
        finally:
            self.slots.release()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=4877)
    parser.add_argument("--control-port", type=int, default=7879)
    parser.add_argument("--token-file", type=Path, required=True)
    args = parser.parse_args()
    token = args.token_file.read_text().strip().lower()
    if len(token) != 64 or any(c not in "0123456789abcdef" for c in token):
        parser.error("token file must contain 64 hexadecimal characters")
    relay = Relay(token)
    with TCPServer(("127.0.0.1", args.control_port), ControlTCP) as tcp, ThreadingHTTPServer((args.host, args.port), partial(RelayHTTP, directory=str(PUBLIC))) as http:
        tcp.relay = http.relay = relay
        threading.Thread(target=tcp.serve_forever, daemon=True).start()
        print(f"PhotoCraft browser relay: http://{args.host}:{args.port}/ipad/ ; MCP bridge 127.0.0.1:{args.control_port}", flush=True)
        http.serve_forever()


if __name__ == "__main__":
    main()
