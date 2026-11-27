// Install fixture dependency: npm install --prefix .cache/recovery-tests --save-exact fake-indexeddb@6.2.8
const {indexedDB,IDBObjectStore}=require('./.cache/recovery-tests/node_modules/fake-indexeddb');
const vm=require('node:vm'),fs=require('node:fs'),assert=require('node:assert/strict');
const source=fs.readFileSync('src/recovery_store.js','utf8').replaceAll('export async function','async function').replaceAll('export function','function');
let now=10000,sequence=0;
function page(id,owner,onWait){
  const note={},events={},timers=[];
  class Clock extends Date {static now(){return now}}
  const c={indexedDB,Date:Clock,Array,Uint8Array,Error,Promise,JSON,Math,crypto:{randomUUID:()=>`fork-${++sequence}`},
    document:{getElementById:()=>note,documentElement:{dataset:{}},body:{append(){}},createElement:()=>({style:{},remove(){}})},
    window:{sessionStorage:{getItem:()=>id==='denied'?null:id},addEventListener:(name,fn)=>events[name]=fn},
    setInterval:fn=>{timers.push(fn);return timers.length},clearInterval(){},
    setTimeout:async(fn,ms)=>{now+=ms;if(onWait)await onWait();fn()}};
  vm.createContext(c);vm.runInContext(source,c);
  return {start:()=>c.recovery_start(JSON.stringify({id,page:owner})),save:(meta,docs)=>c.recovery_save(meta,docs),renew:()=>vm.runInContext('renew()',c),ready:()=>c.recovery_ready('ready'),status:c.recovery_status,events,note};
}
(async()=>{
  const a=page('tab-a','page-1');assert.equal((await a.start()).snapshot,null);
  await a.save('first',[new Uint8Array([1,2,255])]);
  const b=page('tab-b','page-2');assert.equal((await b.start()).snapshot,null);
  await b.save('second',[new Uint8Array([9])]);
  const reload=page('tab-a','page-3');const restored=await reload.start();
  assert.equal(restored.id,'tab-a');assert.equal(restored.snapshot.metadata,'first');assert.deepEqual([...restored.snapshot.documents[0]],[1,2,255]);
  await assert.rejects(a.save('stale',[new Uint8Array([0])]),/lease changed/);
  const duplicate=page('tab-a','page-4',()=>reload.renew());const fork=await duplicate.start();
  assert.notEqual(fork.id,'tab-a');assert.equal(fork.snapshot.metadata,'first');
  await duplicate.save('fork edit',[new Uint8Array([8])]);
  // A quota/clone failure must abort the transaction and retain the prior committed bytes.
  const originalPut=IDBObjectStore.prototype.put;
  IDBObjectStore.prototype.put=function(value,...args){const request=originalPut.call(this,value,...args);if(value?.metadata==='quota')throw Error('QuotaExceededError');return request};
  await assert.rejects(reload.save('quota',[new Uint8Array([7])]),/QuotaExceededError/);
  IDBObjectStore.prototype.put=originalPut;
  const afterFailure=page('tab-a','page-5');assert.equal((await afterFailure.start()).snapshot.metadata,'first');
  const forkReload=page(fork.id,'page-6');assert.equal((await forkReload.start()).snapshot.metadata,'fork edit');
  await afterFailure.save('empty',[]);
  const closed=page('tab-a','page-7');assert.equal((await closed.start()).snapshot.documents.length,0);
  closed.status('pending',true);let prevented=false;closed.events.beforeunload({preventDefault(){prevented=true}});assert.ok(prevented);
  closed.ready();prevented=false;closed.events.beforeunload({preventDefault(){prevented=true}});assert.equal(prevented,false);
  const denied=page('denied','page-8');
  // The browser must not claim reload recovery if its tab identity cannot persist.
  await assert.rejects(denied.start(),/Tab identity/);
  console.log('PASS: atomic reload, independent tabs, duplicate fork, stale-page refusal, quota rollback, empty workspace, pending reload guard');
})().catch(error=>{console.error(error);process.exitCode=1});
