import functools
import http.client
import tempfile
import threading
import unittest
from pathlib import Path
from http.server import ThreadingHTTPServer
import server


class PreviewBoundary(unittest.TestCase):
    def test_only_public_files_are_served_including_head_requests(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            public = root / "public"
            public.mkdir()
            (public / "index.html").write_text("PhotoCraft preview")
            (public / "module.wasm").write_bytes(b"\x00asm")
            (root / "private.txt").write_text("not an artifact")
            (public / "escape").symlink_to(root / "private.txt")
            (public / ".hidden").write_text("hidden")
            (public / "folder").mkdir()
            previous = server.PUBLIC
            server.PUBLIC = public
            httpd = ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(server.Preview, directory=str(public)))
            thread = threading.Thread(target=httpd.serve_forever, daemon=True)
            thread.start()
            try:
                for method in ("GET", "HEAD"):
                    for path in ("/escape", "/.hidden", "/folder/", "/../private.txt", "/%2e%2e/private.txt"):
                        conn = http.client.HTTPConnection("127.0.0.1", httpd.server_port)
                        conn.request(method, path)
                        response = conn.getresponse()
                        self.assertEqual(response.status, 404, (method, path))
                        response.read()
                        conn.close()
                conn = http.client.HTTPConnection("127.0.0.1", httpd.server_port)
                conn.request("GET", "/module.wasm")
                response = conn.getresponse()
                self.assertEqual(response.status, 200)
                self.assertEqual(response.getheader("Content-Type"), "application/wasm")
                self.assertEqual(response.getheader("Cache-Control"), "no-cache")
                response.read()
                conn.close()
            finally:
                httpd.shutdown()
                httpd.server_close()
                thread.join()
                server.PUBLIC = previous


if __name__ == "__main__":
    unittest.main()
