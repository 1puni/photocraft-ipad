// Atomic per-tab IndexedDB snapshots. A lease prevents duplicated tabs from overwriting each other.
let db, key, owner, timer, pending=false, enabled=false, curtain;
const leaseMs=6000;
const fresh=()=>globalThis.crypto?.randomUUID?.() || `${Date.now()}-${Math.random().toString(36).slice(2)}`;
function transaction(stores, mode, action) {
  return new Promise((resolve,reject)=>{
    const tx=db.transaction(stores,mode);let result, failure;
    tx.oncomplete=()=>resolve(result);
    tx.onabort=()=>reject(failure || tx.error || Error('Browser storage transaction aborted'));
    tx.onerror=()=>{};
    try { action(tx, value=>{result=value}, error=>{failure=error;tx.abort()}); }
    catch(error){failure=error;tx.abort();}
  });
}
function openDatabase() {
  return new Promise((resolve,reject)=>{
    const request=indexedDB.open('photocraft-recovery',1);
    request.onupgradeneeded=()=>{request.result.createObjectStore('snapshots');request.result.createObjectStore('leases');};
    request.onsuccess=()=>resolve(request.result);
    request.onerror=()=>reject(request.error);
    request.onblocked=()=>reject(Error('Close older PhotoCraft windows to enable recovery'));
  });
}
function claim() {
  return transaction(['leases'],'readwrite',(tx,done)=>{
    const store=tx.objectStore('leases'), request=store.get(key);
    request.onsuccess=()=>{
      const old=request.result;
      if(old && old.owner!==owner && Date.now()-old.seen<leaseMs){done(false);return;}
      store.put({owner,seen:Date.now()},key);done(true);
    };
  });
}
function renew() {
  return transaction(['leases'],'readwrite',(tx,done,fail)=>{
    const store=tx.objectStore('leases'), request=store.get(key);
    request.onsuccess=()=>{
      if(request.result?.owner!==owner){fail(Error('This tab lost its recovery lease. Reload to recover; save a download first.'));return;}
      store.put({owner,seen:Date.now()},key);done(true);
    };
  });
}
export function recovery_status(message, unsaved) {
  pending=unsaved;
  const node=document.getElementById('pc-recovery');
  if(node)node.textContent=message;
  document.documentElement.dataset.recovery=message;
  const short=unsaved?'Unsaved changes':'Recovery ready';
  document.documentElement.dataset.recoveryState=short;
  const summary=document.querySelector?.('#pc-session summary');
  if(summary?.dataset.sessionLabel)summary.textContent=summary.dataset.sessionLabel+' · '+short;
}
export async function recovery_start(identity) {
  const ident=JSON.parse(identity);key=ident.id;owner=ident.page;
  curtain=document.createElement('div');curtain.id='pc-recovery-loading';curtain.textContent='Checking this tab’s recovery copy…';
  curtain.style.cssText='position:fixed;inset:0;z-index:9998;display:grid;place-items:center;background:#252525;color:white;font:18px system-ui';document.body.append(curtain);
  if(window.sessionStorage.getItem('photocraft.control.tabId')!==key)throw Error('Tab identity cannot be retained in this browser');
  db=await openDatabase();
  db.onversionchange=()=>{enabled=false;db.close();recovery_status('Autosave paused: storage changed. Save a download.',true);};
  const source=key;
  if(!await claim()) {
    // Give the prior page time to stop heartbeating on a normal reload.
    await new Promise(resolve=>setTimeout(resolve,leaseMs+500));
    if(!await claim()){key=fresh();if(!await claim())throw Error('Could not isolate this tab');}
  }
  const saved=await transaction(['snapshots'],'readonly',(tx,done)=>{
    const r=tx.objectStore('snapshots').get(source);r.onsuccess=()=>done(r.result);
  });
  if(key!==source && saved) {
    await transaction(['snapshots'],'readwrite',(tx,done)=>{tx.objectStore('snapshots').put(saved,key);done(true);});
  }
  enabled=true;
  timer=setInterval(()=>{if(enabled)renew().catch(error=>{enabled=false;recovery_status('Autosave paused: '+error.message,true);});},2000);
  return {id:key, snapshot:saved || null};
}
export function recovery_ready(message) { curtain?.remove();recovery_status(message,false); }
export function recovery_failed(message) { enabled=false;curtain?.remove();recovery_status('Autosave unavailable: '+message+' Save a download.',true); }
export async function recovery_save(metadata, documents) {
  if(!enabled)throw Error('Recovery storage is unavailable; save a download');
  const snapshot={version:1,metadata,documents:Array.from(documents),savedAt:Date.now()};
  await transaction(['snapshots','leases'],'readwrite',(tx,done,fail)=>{
    const leases=tx.objectStore('leases'),request=leases.get(key);
    request.onsuccess=()=>{
      if(request.result?.owner!==owner){fail(Error('Recovery lease changed; refusing to overwrite another tab'));return;}
      try {
        tx.objectStore('snapshots').put(snapshot,key);
        leases.put({owner,seen:Date.now()},key);done(true);
      } catch(error) { fail(error); }
    };
  });
  return new Date(snapshot.savedAt).toLocaleTimeString();
}
window.addEventListener('beforeunload',event=>{if(pending){event.preventDefault();event.returnValue='';}});
window.addEventListener('pagehide',()=>clearInterval(timer));
window.addEventListener('pageshow',event=>{if(event.persisted && enabled)timer=setInterval(()=>renew().catch(e=>recovery_failed(e.message)),2000);});
