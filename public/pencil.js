const pad = document.querySelector('#pad');
const ctx = pad.getContext('2d');
const output = id => document.getElementById(id);
let active = null, last = null;
const report = {
  observedAt: new Date().toISOString(), userAgent: navigator.userAgent,
  secureContext: isSecureContext, pointerEvents: 'PointerEvent' in window,
  coalescedEventsAPI: 'PointerEvent' in window && 'getCoalescedEvents' in PointerEvent.prototype,
  webGPUAPI: 'gpu' in navigator, maxTouchPoints: navigator.maxTouchPoints,
  penSamples: 0, strokes: 0, pressure: [null,null], tiltX: [null,null], tiltY: [null,null], twist: [null,null], ignoredTouches: 0,
};
function resize(){
  // Preserve the drawing through rotation or Safari's changing browser chrome.
  const copy = document.createElement('canvas'); copy.width=pad.width; copy.height=pad.height;
  copy.getContext('2d').drawImage(pad,0,0);
  const bounds = pad.getBoundingClientRect(), ratio = devicePixelRatio || 1;
  pad.width=Math.round(bounds.width*ratio); pad.height=Math.round(bounds.height*ratio);
  ctx.drawImage(copy,0,0); ctx.setTransform(ratio,0,0,ratio,0,0); last=null;
}
new ResizeObserver(resize).observe(pad);
function refresh(){
  output('report').textContent = JSON.stringify(report,null,2);
  output('summary').textContent = report.penSamples
    ? `${report.strokes} Pencil strokes · ${report.penSamples} samples · pressure ${report.pressure.map(v=>v.toFixed(3)).join(' to ')}. A changing range confirms reported pressure; a constant value does not.`
    : 'No pen samples yet. A mouse checks the pad, but cannot prove Pencil support.';
}
function sample(e){
  const r=pad.getBoundingClientRect(), p={x:e.clientX-r.left,y:e.clientY-r.top};
  output('kind').textContent=e.pointerType;
  output('pressure').textContent=e.pressure.toFixed(3);
  output('tilt').textContent=`${e.tiltX}° / ${e.tiltY}°`; output('twist').textContent=`${e.twist}°`;
  if(e.pointerType==='pen'){
    report.penSamples++;
    for(const key of ['pressure','tiltX','tiltY','twist']){
      const v=e[key], range=report[key]; range[0]=range[0]===null?v:Math.min(range[0],v); range[1]=range[1]===null?v:Math.max(range[1],v);
    }
  }
  ctx.strokeStyle='#245d85';ctx.fillStyle='#245d85';ctx.lineCap='round';ctx.lineJoin='round';ctx.lineWidth=2+24*(e.pointerType==='pen'?e.pressure:0.5);
  if(last){ctx.beginPath();ctx.moveTo(last.x,last.y);ctx.lineTo(p.x,p.y);ctx.stroke();}
  else{ctx.beginPath();ctx.arc(p.x,p.y,ctx.lineWidth/2,0,Math.PI*2);ctx.fill();}
  last=p;
}
pad.addEventListener('pointerdown', e=>{
  if(e.pointerType==='touch'){report.ignoredTouches++;refresh();return;}
  if(active!==null)return;
  e.preventDefault();pad.setPointerCapture(e.pointerId);active=e.pointerId;last=null;
  if(e.pointerType==='pen')report.strokes++;sample(e);refresh();
});
pad.addEventListener('pointermove',e=>{
  if(e.pointerId!==active)return;e.preventDefault();
  const events=e.getCoalescedEvents?.()||[];output('samples').textContent=String(events.length||1);
  for(const point of events.length?events:[e])sample(point);refresh();
});
for(const kind of ['pointerup','pointercancel','lostpointercapture'])pad.addEventListener(kind,e=>{if(e.pointerId===active){active=null;last=null;refresh();}});
output('clear').onclick=()=>{ctx.save();ctx.setTransform(1,0,0,1,0,0);ctx.clearRect(0,0,pad.width,pad.height);ctx.restore();last=null;};
output('download').onclick=()=>{const a=document.createElement('a'),url=URL.createObjectURL(new Blob([JSON.stringify(report,null,2)],{type:'application/json'}));a.href=url;a.download='photocraft-pencil-test.json';a.click();setTimeout(()=>URL.revokeObjectURL(url),30000);};
refresh();
