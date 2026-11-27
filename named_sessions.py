"""Named browser targets; commands bind to a page generation, never just a label."""
import queue
import secrets
import threading
import time


class Sessions:
    def __init__(self):
        self.lock = threading.Lock()
        self.tabs = {}
        self.pending = {}

    def poll(self, identity):
        keys = ('id', 'page', 'deviceId', 'deviceName', 'tabName', 'document')
        if not isinstance(identity, dict) or any(not isinstance(identity.get(k), str) or len(identity[k]) > 160 for k in keys):
            raise ValueError('invalid session identity')
        if any(not identity[k] for k in keys[:-1]):
            raise ValueError('empty session identity')
        now = time.monotonic()
        with self.lock:
            key = identity['id']
            old = self.tabs.get(key)
            if old and old['page'] != identity['page']:
                if now - old['seen'] < 5:
                    return {'sessionConflict': True}
                # Reload: fail all old requests, including dispatched ones. Never replay.
                for rid, p in list(self.pending.items()):
                    if p['session'] == key:
                        p['waiter'].put_nowait({'ok': False, 'error': 'page reloaded; inspect new session before editing'})
                        del self.pending[rid]
            if not old and len(self.tabs) >= 64:
                stale = [k for k, v in self.tabs.items() if now-v['seen'] > 300 and not any(p['session']==k for p in self.pending.values())]
                for k in stale:
                    del self.tabs[k]
                if len(self.tabs) >= 64:
                    raise ValueError('session limit reached')
            self.tabs[key] = {**{k: identity[k] for k in keys}, 'seen': now}
            for rid, p in self.pending.items():
                if p['session'] == key and p['page'] == identity['page'] and not p['dispatched']:
                    p['dispatched'] = True
                    return {'id': rid, 'method': p['method'], 'params': p['params']}
            return None

    def list(self):
        with self.lock:
            now = time.monotonic()
            return [{**{k: v for k, v in tab.items() if k != 'seen'}, 'connected': now-tab['seen'] < 5,
                     'lastSeenSeconds': round(now-tab['seen'], 1)} for tab in self.tabs.values()]

    def select(self, session):
        with self.lock:
            tab = self.tabs.get(session)
            if not tab or time.monotonic()-tab['seen'] >= 5:
                raise ValueError('session offline or unknown; keep the intended tab visible')
            return (session, tab['page'])

    def reply(self, session, page, rid, reply):
        with self.lock:
            p = self.pending.get(rid)
            if p is None:
                return  # Late completion after timeout; never replay it.
            if (session, page) != (p['session'], p['page']):
                raise ValueError('reply belongs to another tab or page generation')
            del self.pending[rid]
            p['waiter'].put_nowait(reply)

    def call(self, target, method, params, timeout=30):
        if target is None:
            return {'ok': False, 'error': 'select a named tab with session.select first'}
        session, page = target
        rid = secrets.token_hex(16)
        waiter = queue.Queue(maxsize=1)
        with self.lock:
            tab = self.tabs.get(session)
            if not tab or tab['page'] != page:
                return {'ok': False, 'error': 'page changed; select and inspect the session again'}
            if time.monotonic()-tab['seen'] >= 5:
                return {'ok': False, 'error': 'selected tab offline; keep it visible'}
            if len(self.pending) >= 32:
                return {'ok': False, 'error': 'control queue is full'}
            self.pending[rid] = {'session':session, 'page':page, 'method':method, 'params':params,
                                 'waiter':waiter, 'dispatched':False}
        try:
            return waiter.get(timeout=timeout)
        except queue.Empty:
            return {'ok': False, 'error': 'browser timeout; inspect before retrying; outcome may be unknown'}
        finally:
            with self.lock:
                self.pending.pop(rid, None)
