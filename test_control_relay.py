import functools
import http.client
import json
import socket
import threading
import time
import unittest
from http.server import ThreadingHTTPServer
import control_relay as cr

TOKEN = "a" * 64


class BrowserRelay(unittest.TestCase):
    def setUp(self):
        self.relay = cr.Relay(TOKEN)
        self.http = ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(cr.RelayHTTP, directory=str(cr.PUBLIC)))
        self.tcp = cr.TCPServer(("127.0.0.1", 0), cr.ControlTCP)
        self.http.relay = self.tcp.relay = self.relay
        for server in (self.http, self.tcp):
            threading.Thread(target=server.serve_forever, daemon=True).start()
            self.addCleanup(server.server_close)
            self.addCleanup(server.shutdown)

    def request(self, path, body=None, token=TOKEN, session="browser-one"):
        conn = http.client.HTTPConnection("127.0.0.1", self.http.server_port, timeout=3)
        headers = {"Authorization": "Bearer " + token, "X-PhotoCraft-Session": session}
        conn.request("GET" if body is None else "POST", path, None if body is None else json.dumps(body), headers)
        response = conn.getresponse()
        result = response.status, response.read()
        conn.close()
        return result

    def test_auth_single_browser_and_exact_request_response(self):
        self.assertEqual(self.request("/control/next", token="wrong")[0], 403)
        self.assertEqual(self.request("/control/next")[0], 200)
        self.assertEqual(self.request("/control/next", session="other-tab")[0], 409)
        with socket.create_connection(self.tcp.server_address, timeout=3) as sock:
            stream = sock.makefile("rwb")
            self.addCleanup(stream.close)
            stream.write(json.dumps({"id": 1, "method": "auth", "params": {"token": TOKEN}}).encode() + b"\n")
            stream.flush()
            self.assertTrue(json.loads(stream.readline())["ok"])
            stream.write(json.dumps({"id": 2, "method": "engine.execute", "params": {"command": "edit.undo", "params": {}}}).encode() + b"\n")
            stream.flush()
            request = None
            for _ in range(50):
                request = json.loads(self.request("/control/next")[1])
                if request:
                    break
                time.sleep(.01)
            self.assertEqual(request["method"], "engine.execute")
            self.assertEqual(request["params"]["command"], "edit.undo")
            reply = {"ok": True, "result": {"document": "same-browser"}}
            self.assertEqual(self.request("/control/reply", {"id": request["id"], "reply": reply})[0], 200)
            self.assertEqual(json.loads(stream.readline()), {**reply, "id": 2})

    def test_absent_browser_and_expiry_never_replay_edits(self):
        self.assertFalse(self.relay.call("engine.execute", {})["ok"])
        self.relay.poll("browser")
        self.assertIn("timeout", self.relay.call("engine.execute", {}, timeout=.001)["error"])
        self.assertIsNone(self.relay.poll("browser"))
        self.assertFalse(self.relay.pending)

    def test_wrong_session_cannot_forge_reply(self):
        self.relay.poll("browser-one")
        self.assertEqual(self.request("/control/reply", {"id": "x", "reply": {"ok": True}}, session="other")[0], 400)
        self.assertEqual(self.request("/control/reply", ["not", "an", "envelope"])[0], 400)

    def test_tcp_requires_authentication(self):
        with socket.create_connection(self.tcp.server_address, timeout=3) as sock:
            sock.sendall(b'{"id":1,"method":"ui.inspect","params":{}}\n')
            self.assertFalse(json.loads(sock.recv(4096))["ok"])
            self.assertEqual(sock.recv(4096), b"")


if __name__ == "__main__":
    unittest.main()
