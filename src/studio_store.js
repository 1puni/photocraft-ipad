// Same-origin document persistence. Credentials and filesystem paths never enter links.
let binding, badge;
const key='photocraft.studio.binding.v1';
function persist(){try{sessionStorage.setItem(key,JSON.stringify(binding));}catch{/* Acknowledged disk saves still succeed if optional tab metadata storage is unavailable. */}}
export function studio_config(){
 const q=new URLSearchParams(location.search),panel=q.get('studioPanel'),composition=q.get('studioComposition');
 if(!panel&&!composition)return '';
 if(!/^[a-zA-Z0-9][a-zA-Z0-9_-]{0,180}$/.test(panel||'')||!/^[a-zA-Z0-9][a-zA-Z0-9_-]{0,180}$/.test(composition||''))throw Error('Invalid studio document link');
 let old;try{old=JSON.parse(sessionStorage.getItem(key));}catch{}
 if(old?.panel===panel&&old?.composition===composition&&old?.detached)return '';
 binding=old?.panel===panel&&old?.composition===composition?old:{panel,composition};
 return JSON.stringify(binding);
}
const endpoint=()=>'/studio-api/documents/'+binding.panel+'/'+binding.composition;
async function checked(r){if(!r.ok){let value;try{value=await r.json();}catch{}throw Error(value?.error||'Studio request failed ('+r.status+')');}return r;}
export async function studio_open(){
 const info=await(await checked(await fetch(endpoint(),{cache:'no-store'}))).json();
 // Metadata and PSD can race another save: the PSD response's version owns these bytes.
 const response=await checked(await fetch(endpoint()+'/psd',{cache:'no-store'}));
 binding.version=response.headers.get('ETag')||info.version;
 return {bytes:new Uint8Array(await response.arrayBuffer()),version:binding.version};
}
export function studio_bind(documentId){binding.documentId=documentId;persist();}
export async function studio_save(bytes){
 const response=await checked(await fetch(endpoint()+'/psd',{method:'PUT',headers:{'Content-Type':'application/octet-stream','If-Match':binding.version},body:bytes}));
 const result=await response.json();if(!result.saved||!result.version)throw Error('Studio did not confirm the save');
 binding.version=result.version;persist();return result.refreshError?'Saved to studio; preview refresh pending':'Saved to studio';
}
export function studio_status(text,error){
 if(!badge){badge=document.createElement('div');badge.id='pc-studio-save';Object.assign(badge.style,{position:'fixed',left:'12px',bottom:'10px',zIndex:9998,font:'13px system-ui',padding:'7px',background:'#303030',color:'#eee',borderRadius:'8px',maxWidth:'55vw',pointerEvents:'none'});badge.setAttribute('role','status');document.body.append(badge);}
 badge.textContent=text;badge.style.color=error?'#ffbfab':'#eee';badge.dataset.state=error?'error':text.startsWith('Saving')?'saving':'ready';
}

export function studio_detach(){binding.detached=true;persist();studio_status('Download copy · no longer bound to studio',false);}
