"""Serve the review artifacts only; the editor never uploads documents to this server."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import argparse

PUBLIC = Path(__file__).resolve().parent / "public"


class Preview(SimpleHTTPRequestHandler):
    def list_directory(self, path):
        self.send_error(404)

    def end_headers(self):
        self.send_header("Cache-Control", "no-cache")
        self.send_header("X-Content-Type-Options", "nosniff")
        super().end_headers()

    def send_head(self):
        # No hidden files or symlink escapes if additional artifacts are staged later.
        path = Path(self.translate_path(self.path))
        if not path.resolve().is_relative_to(PUBLIC.resolve()) or any(p.startswith(".") for p in path.relative_to(PUBLIC).parts):
            self.send_error(404)
            return
        return super().send_head()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=4876)
    args = parser.parse_args()
    server = ThreadingHTTPServer(("0.0.0.0", args.port), partial(Preview, directory=str(PUBLIC)))
    print(f"PhotoCraft review: http://localhost:{args.port} — serving {PUBLIC}", flush=True)
    server.serve_forever()
