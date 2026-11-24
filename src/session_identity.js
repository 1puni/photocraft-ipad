// Identity is metadata, not authentication. Artwork is never stored here.
const prefix = 'photocraft.control.';
const fresh = () => globalThis.crypto?.randomUUID?.() || `${Date.now()}-${Math.random().toString(36).slice(2)}`;
function read(storage, key, fallback) {
  try { const value = storage.getItem(prefix+key) || fallback(); storage.setItem(prefix+key,value); return value; }
  catch { return fallback(); }
}
let local, tab;
try { local = window.localStorage; tab = window.sessionStorage; } catch {}
const deviceId = read(local,'deviceId',fresh);
const deviceName = read(local,'deviceName',()=> /iPad/.test(navigator.userAgent) || (navigator.platform==='MacIntel' && navigator.maxTouchPoints>1) ? 'iPad' : /Mac/.test(navigator.platform) ? 'Mac' : 'Browser');
let state = { id:read(tab,'tabId',fresh), page:fresh(), deviceId, deviceName,
  tabName:read(tab,'tabName',()=> 'Studio '+fresh().slice(-4)), document:'' };
let button, panel, label, status, conflictSince=0;
function persist() { try { local.setItem(prefix+'deviceName',state.deviceName);tab.setItem(prefix+'tabName',state.tabName);tab.setItem(prefix+'tabId',state.id); } catch {} }
function paint(message) {
  if (!button) {
    const style=document.createElement('style'); style.textContent=`#pc-session{position:fixed;right:12px;bottom:10px;z-index:9999;font:13px system-ui;color:#eee;background:#303030;border:1px solid #777;border-radius:8px;padding:7px;max-width:calc(100vw - 24px)}#pc-session summary{cursor:pointer;overflow-wrap:anywhere}#pc-session label{display:block;margin:8px 0}#pc-session input{max-width:180px;margin-left:8px}#pc-session p{max-width:290px;margin:8px 0}`;document.head.append(style);
    panel=document.createElement('details');panel.id='pc-session';button=document.createElement('summary');panel.append(button);
    for (const [key,title] of [['deviceName','Device'],['tabName','Tab']]) {
      const row=document.createElement('label');row.textContent=title;const input=document.createElement('input');input.value=state[key];input.maxLength=80;input.setAttribute('aria-label',title+' name');input.addEventListener('change',()=>{state[key]=input.value.trim().slice(0,80)||title;input.value=state[key];persist();paint('Connecting…');});row.append(input);panel.append(row);
    }
    label=document.createElement('p');status=document.createElement('p');panel.append(label,status);
    const note=document.createElement('p');note.id='pc-recovery';note.textContent='Checking local recovery…';panel.append(note);document.body.append(panel);
  }
  const title=`${state.deviceName} · ${state.tabName}`;button.dataset.sessionLabel=title;button.textContent=title+(document.documentElement?.dataset?.recoveryState ? ' · '+document.documentElement.dataset.recoveryState : '');label.textContent=(state.document||'No document')+' · '+state.id.slice(-8);status.textContent=message;document.title=`${title} — ${state.document||'PhotoCraft'}`;
}
export function identity() { paint('Connecting…');return JSON.stringify(state); }
export function metadata(name) { const next=String(name).slice(0,160);if(state.document!==next){state.document=next;if(button)paint(status.textContent);}return JSON.stringify(state); }
export function connection(message) { paint(message); }
export function conflict() {
  // A duplicated tab inherits sessionStorage. A real reload stops the old heartbeat;
  // wait for it to expire before deciding this is a concurrent duplicate.
  if (!conflictSince) conflictSince=Date.now();
  if (Date.now()-conflictSince>6500) { state.id=fresh();state.tabName+=' copy';state.tabName=state.tabName.slice(0,80);persist();conflictSince=0; }
  paint('Reconnecting tab…');return JSON.stringify(state);
}
export function connected() { conflictSince=0;paint('Connected · select this tab to control it'); }

window.addEventListener?.('storage', event => {
  if (event.key === prefix+'deviceName' && event.newValue) {
    state.deviceName=event.newValue.slice(0,80);
    if (button) {
      panel.querySelector('input[aria-label="Device name"]').value=state.deviceName;
      paint(status.textContent);
    }
  }
});

export function recovery_identity(id) {
  if (state.id!==id) {state.id=id;state.tabName=(state.tabName+' copy').slice(0,80);persist();}
  paint('Local session');
}
