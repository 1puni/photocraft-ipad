const fs=require('fs'),vm=require('vm'),assert=require('node:assert/strict');
const source=fs.readFileSync('src/session_identity.js','utf8').replaceAll('export function','function');
const storage=(seed={})=>{const m=new Map(Object.entries(seed));return {getItem:k=>m.get(k),setItem:(k,v)=>m.set(k,v),m};};
function page(local,tab){let now=1000,id=0;const elements=[];const el=()=>({textContent:'',append(){},querySelector(){return {}},setAttribute(){},addEventListener(){}});const c={window:{localStorage:local,sessionStorage:tab,addEventListener:(name,fn)=>{c.onStorage=fn}},navigator:{userAgent:'iPad',platform:'MacIntel',maxTouchPoints:5},document:{head:{append(){}},body:{append(x){elements.push(x)}},createElement:el},Date:{now:()=>now},Math,JSON,String,globalThis:{crypto:{randomUUID:()=>`page-${++id}-${Math.random()}`}}};vm.createContext(c);vm.runInContext(source,c);return {call:expr=>vm.runInContext(expr,c),advance:ms=>now+=ms};}
const local=storage(),tab=storage();const a=page(local,tab),first=JSON.parse(a.call('identity()'));
assert.equal(first.deviceName,'iPad');assert.equal(first.document,'');
a.call('state.deviceName="V’s iPad";state.tabName="Panel 002";persist()');
const reload=page(local,tab),second=JSON.parse(reload.call('identity()'));
assert.equal(second.id,first.id);assert.notEqual(second.page,first.page);assert.equal(second.tabName,'Panel 002');assert.equal(second.deviceName,'V’s iPad');
const separate=page(local,storage());assert.notEqual(JSON.parse(separate.call('identity()')).id,first.id);
const duplicate=page(local,storage(Object.fromEntries(tab.m)));duplicate.call('conflict()');duplicate.advance(7000);const split=JSON.parse(duplicate.call('conflict()'));assert.notEqual(split.id,first.id);assert.equal(split.tabName,'Panel 002 copy');
const blocked={getItem(){throw Error('blocked')},setItem(){throw Error('blocked')}};assert.ok(JSON.parse(page(blocked,blocked).call('identity()')).id);
console.log('Identity reload, per-tab separation, duplication, device labels and storage-denied fallback passed');

a.call('onStorage({key:"photocraft.control.deviceName",newValue:"Shared Mac"})');assert.equal(JSON.parse(a.call('identity()')).deviceName,'Shared Mac');
console.log('Device rename propagates to other open tabs');
