"""Optional same-origin adapter to the local studio's document service."""
import http.client
import json
import re

ROUTE = re.compile(r'^/studio-api/documents/[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+(?:/psd)?$')

def proxy(handler):
    if not ROUTE.fullmatch(handler.path):
        handler.send_error(404)
        return
    origin = handler.headers.get('Origin')
    expected = 'http://' + handler.headers.get('Host', '')
    if origin and origin != expected or handler.command == 'PUT' and origin != expected:
        handler.send_error(403)
        return
    if not re.fullmatch(r'(?:127\.0\.0\.1|localhost|v\.local):(?:4877|4878)', handler.headers.get('Host','')):
        handler.send_error(403)
        return
    try:
        size = int(handler.headers.get('Content-Length', '0'))
        if not 0 <= size <= 64 * 1024 * 1024:
            handler.send_error(413)
            return
        body = handler.rfile.read(size) if size else None
        if body is not None and len(body) != size:
            handler.send_error(400)
            return
        headers = {'Origin': expected}
        for name in ('Content-Type','If-Match'):
            if name in handler.headers:
                headers[name] = handler.headers[name]
        conn = http.client.HTTPConnection('127.0.0.1',4881,timeout=60)
        try:
            conn.request(handler.command,handler.path.removeprefix('/studio-api'),body,headers)
            response = conn.getresponse()
            data = response.read()
            handler.send_response(response.status)
            for key in ('Content-Type','ETag'):
                value = response.getheader(key)
                if value: handler.send_header(key,value)
            handler.send_header('Content-Length',str(len(data)))
            handler.end_headers()
            handler.wfile.write(data)
        finally:
            conn.close()
    except (ValueError, OSError, http.client.HTTPException):
        data=json.dumps({'error':'Studio unavailable; keep this tab and retry Save.'}).encode()
        handler.send_response(502)
        handler.send_header('Content-Type','application/json')
        handler.send_header('Content-Length',str(len(data)))
        handler.end_headers()
        handler.wfile.write(data)
