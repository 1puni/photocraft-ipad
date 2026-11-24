import json
import socket
import threading
import time
import unittest
from named_sessions import Sessions
from test_control_relay import BrowserRelay, TOKEN


def identity(tab='a', page='a1', name='Studio'):
    return dict(id=tab, page=page, deviceId='device', deviceName='iPad', tabName=name, document='panel.psd')


class NamedSessionTests(unittest.TestCase):
    def test_reload_retains_identity_but_invalidates_selection(self):
        s=Sessions();s.poll(identity());target=s.select('a')
        self.assertEqual(s.poll(identity(page='a2')), {'sessionConflict': True})
        s.tabs['a']['seen']-=6
        self.assertIsNone(s.poll(identity(page='a2')))
        self.assertFalse(s.call(target,'edit.undo',{})['ok'])
        self.assertEqual(s.select('a'),('a','a2'))
        self.assertEqual(s.list()[0]['deviceName'],'iPad')

    def test_no_implicit_target_and_no_delivery_to_other_tab(self):
        s=Sessions();s.poll(identity());s.poll(identity('b','b1'))
        self.assertFalse(s.call(None,'edit.undo',{})['ok'])
        result=[]
        t=threading.Thread(target=lambda:result.append(s.call(s.select('a'),'edit.undo',{},timeout=1)));t.start()
        for _ in range(100):
            if s.pending:break
            time.sleep(.001)
        self.assertIsNone(s.poll(identity('b','b1')))
        msg=s.poll(identity())
        with self.assertRaises(ValueError):s.reply('b','b1',msg['id'],{'ok':True})
        with self.assertRaises(ValueError):s.reply('a','old',msg['id'],{'ok':True})
        s.reply('a','a1',msg['id'],{'ok':True,'result':'a'})
        t.join();self.assertEqual(result,[{'ok':True,'result':'a'}])

    def test_reload_cancels_pending_and_timeout_never_replays(self):
        s=Sessions();s.poll(identity());result=[]
        t=threading.Thread(target=lambda:result.append(s.call(s.select('a'),'edit.undo',{},timeout=1)));t.start()
        for _ in range(100):
            if s.pending:break
            time.sleep(.001)
        s.tabs['a']['seen']-=6
        self.assertIsNone(s.poll(identity(page='a2')))
        t.join();self.assertIn('reloaded',result[0]['error'])
        self.assertFalse(s.call(s.select('a'),'edit.undo',{},timeout=.001)['ok'])
        self.assertIsNone(s.poll(identity(page='a2')))
        self.assertFalse(s.pending)

    def test_metadata_validation_and_offline_state(self):
        s=Sessions()
        with self.assertRaises(ValueError):s.poll({'id':'x'})
        s.poll(identity());s.tabs['a']['seen']-=6
        self.assertFalse(s.list()[0]['connected'])
        with self.assertRaises(ValueError):s.select('a')


class NamedHTTPTests(BrowserRelay):
    def poll3(self, ident, token=TOKEN):
        import http.client
        c=http.client.HTTPConnection('127.0.0.1',self.http.server_port,timeout=3)
        c.request('POST','/control/poll',json.dumps(ident),{'Authorization':'Bearer '+token,'X-PhotoCraft-Protocol':'3','X-PhotoCraft-Session':ident['id'],'X-PhotoCraft-Page':ident['page']})
        r=c.getresponse();raw=r.read();value=r.status,json.loads(raw) if r.status==200 else raw;c.close();return value

    def test_named_http_registration_and_tcp_selection(self):
        self.assertEqual(self.poll3(identity(),token='wrong')[0],403)
        self.assertEqual(self.poll3(identity()),(200,None))
        self.assertEqual(self.poll3(identity('b','b1')),(200,None))
        with socket.create_connection(self.tcp.server_address,timeout=3) as sock:
            stream=sock.makefile('rwb')
            self.addCleanup(stream.close)
            def rpc(method,params):
                stream.write(json.dumps({'id':1,'method':method,'params':params}).encode()+b'\n');stream.flush();return json.loads(stream.readline())
            self.assertTrue(rpc('auth',{'token':TOKEN})['ok'])
            self.assertEqual(len(rpc('session.list',{})['result']['sessions']),2)
            self.assertFalse(rpc('ui.inspect',{})['ok'])
            self.assertEqual(rpc('session.select',{'id':'a'})['result'],{'id':'a','page':'a1'})
            self.assertFalse(rpc('session.select',{'id':'absent'})['ok'])
            self.assertFalse(rpc('ui.inspect',{})['ok'])

    def test_controller_selections_are_independent(self):
        self.poll3(identity());self.poll3(identity('b','b1'))
        streams=[]
        for tab in ('a','b'):
            sock=socket.create_connection(self.tcp.server_address,timeout=3)
            self.addCleanup(sock.close)
            stream=sock.makefile('rwb');self.addCleanup(stream.close);streams.append(stream)
            for method,params in [('auth',{'token':TOKEN}),('session.select',{'id':tab})]:
                stream.write(json.dumps({'id':1,'method':method,'params':params}).encode()+b'\n');stream.flush()
                self.assertTrue(json.loads(stream.readline())['ok'])
        streams[0].write(b'{"id":2,"method":"ui.inspect","params":{}}\n');streams[0].flush()
        for _ in range(100):
            if self.http.relay.named.pending:break
            time.sleep(.001)
        self.assertEqual(self.poll3(identity('b','b1')),(200,None))
        status,msg=self.poll3(identity());self.assertEqual(status,200)
        self.http.relay.named.reply('a','a1',msg['id'],{'ok':True,'result':'only a'})
        self.assertEqual(json.loads(streams[0].readline())['result'],'only a')
