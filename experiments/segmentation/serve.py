"""Serve only the disposable lab public directory; retain bounded benchmark receipts."""
import argparse
import json
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

p=argparse.ArgumentParser()
p.add_argument('--lab',type=Path,required=True)
p.add_argument('--port',type=int,default=4879)
a=p.parse_args()
root=(a.lab/'public').resolve()

class Handler(SimpleHTTPRequestHandler):
    def list_directory(self,path):
        self.send_error(404)

    def send_head(self):
        path=Path(self.translate_path(self.path))
        if not path.resolve().is_relative_to(root) or any(p.startswith('.') for p in path.relative_to(root).parts):
            self.send_error(404)
            return None
        return super().send_head()

    def do_POST(self):
        n=int(self.headers.get('Content-Length','0'))
        if self.path!='/receipt' or not 0<n<=65536:
            self.send_error(400)
            return
        try:
            value=json.loads(self.rfile.read(n))
        except (ValueError,UnicodeDecodeError):
            self.send_error(400)
            return
        with (a.lab/'browser-receipts.jsonl').open('a') as f:
            f.write(json.dumps(value)+'\n')
        self.send_response(204)
        self.end_headers()

ThreadingHTTPServer(('0.0.0.0',a.port),partial(Handler,directory=str(root))).serve_forever()
