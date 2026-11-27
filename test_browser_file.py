import base64
import tempfile
import unittest
from pathlib import Path
import browser_file as bf


class MemoryMCP:
    def __init__(self, data=b"", corrupt=False):
        self.data = data
        self.uploaded = bytearray()
        self.corrupt = corrupt
        self.opened = False

    def transfer(self, operation, **params):
        if operation == "write":
            assert params["offset"] == len(self.uploaded)
            self.uploaded.extend(base64.b64decode(params["base64"]))
        if operation == "info":
            return {"complete": True, "size": len(self.data)}
        if operation == "read":
            data = self.data[params["offset"]:params["offset"] + params["length"]]
            return {"offset": params["offset"] + int(self.corrupt), "base64": base64.b64encode(data).decode()}
        return {}

    def tool(self, tool, arguments):
        if tool == "doc_open":
            self.opened = True
        return {}


class BrowserFiles(unittest.TestCase):
    def test_large_upload_and_atomic_download_are_exact(self):
        data = bytes(range(256)) * 3000
        with tempfile.TemporaryDirectory() as folder:
            source = Path(folder) / "input.psd"
            target = Path(folder) / "output.psd"
            source.write_bytes(data)
            mcp = MemoryMCP(data)
            self.assertEqual(bf.upload(mcp, source)["bytes"], len(data))
            self.assertEqual(mcp.uploaded, data)
            self.assertTrue(mcp.opened)
            self.assertEqual(bf.download(mcp, target)["bytes"], len(data))
            self.assertEqual(target.read_bytes(), data)
            self.assertEqual(list(Path(folder).glob(".photocraft-*")), [])

    def test_bad_export_cannot_replace_existing_file(self):
        with tempfile.TemporaryDirectory() as folder:
            target = Path(folder) / "existing.psd"
            target.write_bytes(b"keep")
            with self.assertRaises(RuntimeError):
                bf.download(MemoryMCP(b"wrong", corrupt=True), target)
            self.assertEqual(target.read_bytes(), b"keep")
            self.assertEqual(list(Path(folder).glob(".photocraft-*")), [])


if __name__ == "__main__":
    unittest.main()
