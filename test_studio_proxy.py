import unittest
from unittest.mock import Mock
from studio_proxy import proxy, ROUTE

class StudioBoundary(unittest.TestCase):
    def test_only_document_routes(self):
        for path in ['/studio-api/../private','/studio-api/documents/a/b/../../secret','/studio-api/documents/a/b?url=http://other','/studio-api/selection']:
            self.assertIsNone(ROUTE.fullmatch(path))
        self.assertIsNotNone(ROUTE.fullmatch('/studio-api/documents/a/b/psd'))
    def test_cross_origin_put_and_rebinding_host_rejected(self):
        for origin,host in [('https://elsewhere','v.local:4878'),(None,'v.local:4878'),('http://evil:4878','evil:4878')]:
            h=Mock();h.path='/studio-api/documents/a/b/psd';h.command='PUT';h.headers={'Host':host}
            if origin:h.headers['Origin']=origin
            proxy(h);h.send_error.assert_called_once_with(403)
if __name__=='__main__':unittest.main()
