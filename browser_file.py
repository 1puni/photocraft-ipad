"""Transfer a file through upstream MCP to/from the live browser's memory store.

The caller explicitly chooses local paths. The relay has no filesystem access.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent
CHUNK = 512 << 10


class MCP:
    def __init__(self, cli, bridge, token_file):
        self.process = subprocess.Popen([str(cli), "mcp", "--bridge", bridge, "--control-token-file", str(token_file)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        self.sequence = 0
        self.request("initialize", {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "photocraft-browser-file", "version": "1"}})
        self.send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def send(self, value):
        self.process.stdin.write(json.dumps(value) + "\n")
        self.process.stdin.flush()

    def request(self, method, params):
        self.sequence += 1
        self.send({"jsonrpc": "2.0", "id": self.sequence, "method": method, "params": params})
        for line in self.process.stdout:
            reply = json.loads(line)
            if reply.get("id") == self.sequence:
                if "error" in reply:
                    raise RuntimeError(str(reply["error"]))
                return reply["result"]
        raise RuntimeError("MCP process disconnected")

    def tool(self, tool, arguments):
        result = self.request("tools/call", {"name": tool, "arguments": arguments})
        texts = [c["text"] for c in result.get("content", []) if c.get("type") == "text"]
        if result.get("isError"):
            raise RuntimeError("; ".join(texts))
        if not texts:
            return None
        try:
            return json.loads(texts[0])
        except ValueError:
            return texts[0]

    def transfer(self, operation, **params):
        return self.tool("control_call", {"method": "transfer." + operation, "params": params})

    def close(self):
        self.process.terminate()
        self.process.wait(timeout=5)


def upload(mcp, path):
    size = path.stat().st_size
    if size > 64 << 20:
        raise ValueError("file exceeds the browser's 64 MiB transfer limit")
    mcp.transfer("begin", name=path.name, size=size)
    offset = 0
    digest = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(CHUNK):
            digest.update(chunk)
            mcp.transfer("write", name=path.name, offset=offset, base64=base64.b64encode(chunk).decode())
            offset += len(chunk)
    result = mcp.tool("doc_open", {"path": path.name})
    mcp.transfer("remove", name=path.name)
    return {"opened": str(path), "bytes": offset, "sha256": digest.hexdigest(), "result": result, "document": mcp.tool("doc_inspect", {})}


def download(mcp, path, copy=False):
    result = mcp.transfer("export", name=path.name) if copy else mcp.tool("doc_save", {"path": path.name})
    info = mcp.transfer("info", name=path.name)
    if not info["complete"]:
        raise RuntimeError("browser export is incomplete")
    path.parent.mkdir(parents=True, exist_ok=True)
    pending = None
    try:
        digest = hashlib.sha256()
        with tempfile.NamedTemporaryFile(mode="wb", prefix=".photocraft-", dir=path.parent, delete=False) as target:
            pending = Path(target.name)
            offset = 0
            while offset < info["size"]:
                value = mcp.transfer("read", name=path.name, offset=offset, length=min(CHUNK, info["size"] - offset))
                chunk = base64.b64decode(value["base64"], validate=True)
                if value["offset"] != offset or not chunk or len(chunk) > info["size"] - offset:
                    raise RuntimeError("invalid browser export chunk")
                target.write(chunk)
                digest.update(chunk)
                offset += len(chunk)
            target.flush()
            os.fsync(target.fileno())
        os.replace(pending, path)
        pending = None
        mcp.transfer("remove", name=path.name)
        return {"saved": str(path), "bytes": offset, "sha256": digest.hexdigest(), "result": result}
    finally:
        if pending is not None:
            pending.unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, default=ROOT.parent / "photocraft/target/debug/photocraft-cli")
    parser.add_argument("--bridge", default="127.0.0.1:7879")
    parser.add_argument("--token-file", type=Path, default=ROOT / ".runtime/control.token")
    parser.add_argument("operation", choices=["open", "save", "export"])
    parser.add_argument("path", type=Path)
    args = parser.parse_args()
    mcp = MCP(args.cli, args.bridge, args.token_file)
    try:
        result = upload(mcp, args.path.resolve()) if args.operation == "open" else download(mcp, args.path.resolve(), copy=args.operation == "export")
        print(json.dumps(result, ensure_ascii=False))
    finally:
        mcp.close()


if __name__ == "__main__":
    main()
