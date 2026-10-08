// Isolated benchmark worker; this is not part of the Rust editor.
importScripts('/runtime/ort.all.min.js');
ort.env.wasm.numThreads = 1; // Matches the current non-isolated LAN preview.
ort.env.wasm.wasmPaths = '/runtime/';
const timed = async f => { const t=performance.now(); const value=await f(); return [value,performance.now()-t]; };
async function receipt(value) {
  postMessage(value);
  await fetch('/receipt',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(value)});
}
async function feeds(name, group, spec) {
  const result={};
  for(const [key,d] of Object.entries(spec[group])) {
    const buffer=await (await fetch(`/feeds/${name}/${d.file}`)).arrayBuffer();
    result[key]=new ort.Tensor(d.dtype==='int64'?'int64':'float32', d.dtype==='int64'?new BigInt64Array(buffer):new Float32Array(buffer),d.shape);
  }
  return result;
}
onmessage=async({data:{provider='wasm',names=['mobile','edge']}})=>{
  await receipt({stage:'environment',provider,secure:isSecureContext,webgpu:!!navigator.gpu,ua:navigator.userAgent,threads:navigator.hardwareConcurrency});
  for(const name of names) {
    let enc,dec;
    try {
      await receipt({name,provider,stage:'loading'});
      const spec=await (await fetch(`/feeds/${name}/feeds.json`)).json();
      const opts={executionProviders:[provider],graphOptimizationLevel:'all'};
      const encPath=spec.encoder_model || (name==='mobile'?'encoder.onnx':'vision_encoder.onnx');
      const decPath=spec.decoder_model || (name==='mobile'?'decoder.onnx':'prompt_encoder_mask_decoder.onnx');
      const options=path=>name!=='mobile'?{...opts,externalData:[{path:path+'_data',data:`/models/${name}/${path}_data`}]}:opts;
      let load;
      [enc,load]=await timed(()=>ort.InferenceSession.create(`/models/${name}/${encPath}`,options(encPath)));
      await receipt({name,provider,stage:'encoder_loaded',ms:load});
      const input=await feeds(name,'encoder',spec);
      let embedding,encoderMs;
      [embedding,encoderMs]=await timed(()=>enc.run(input));
      await receipt({name,provider,stage:'encoded',ms:encoderMs});
      [dec,load]=await timed(()=>ort.InferenceSession.create(`/models/${name}/${decPath}`,options(decPath)));
      const prompts=await feeds(name,'decoder',spec);
      // Replace saved native embeddings: device runs the complete pipeline itself.
      for(const [key,value] of Object.entries(embedding)) { prompts[key]?.dispose(); prompts[key]=value; }
      let output,decoderMs;
      [output,decoderMs]=await timed(()=>dec.run(prompts));
      const score=output[name==='mobile'?'iou_predictions':'iou_scores'];
      const scores=Array.from(await score.getData());
      let maskIoU=null;
      if(spec.reference_mask) {
        const ref=new Uint8Array(await(await fetch(`/feeds/${name}/${spec.reference_mask}`)).arrayBuffer());
        const tensor=output[name==='mobile'?'masks':'pred_masks'];
        const data=await tensor.getData();
        const best=scores.indexOf(Math.max(...scores));
        const size=tensor.dims.slice(-2).reduce((a,b)=>a*b,1);
        if(ref.length!==size)throw Error('Reference mask size mismatch');
        let intersection=0,union=0;
        for(let i=0;i<size;i++) {
          const a=data[best*size+i]>0,b=ref[i]>0;
          intersection+=a&&b?1:0;union+=a||b?1:0;
        }
        maskIoU=intersection/Math.max(1,union);
      }
      for(const v of Object.values(output))v.dispose();
      const warm=[];
      for(let n=0;n<3;n++) {
        const [o,ms]=await timed(()=>dec.run(prompts)); warm.push(ms);
        for(const v of Object.values(o))v.dispose();
      }
      const maxScoreError=Math.max(...scores.map((s,i)=>Math.abs(s-spec.native_scores[i])));
      const parityPass=maxScoreError<0.001 && (maskIoU===null || maskIoU>0.999);
      await receipt({name,provider,stage:parityPass?'complete':'parity_failed',encoderMs,decoderMs,warmDecoderMs:warm,scores,nativeScores:spec.native_scores,maskIoU,maxScoreError,parityPass});
      for(const v of Object.values(input))v.dispose();
      for(const v of Object.values(prompts))v.dispose();
    } catch(e) {
      await receipt({name,provider,stage:'error',error:String(e),stack:e.stack});
    } finally { if(enc)await enc.release(); if(dec)await dec.release(); }
  }
  await receipt({stage:'finished',provider});
};
