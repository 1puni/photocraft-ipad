"""Download pinned public weights, or stage the browser comparison after compare.py."""
import argparse
import hashlib
import json
import shutil
import urllib.request
from pathlib import Path

p=argparse.ArgumentParser()
p.add_argument('--lab',type=Path,required=True)
p.add_argument('--stage-browser',action='store_true')
a=p.parse_args()
lab=a.lab.resolve()
here=Path(__file__).parent
mobile='https://huggingface.co/Acly/MobileSAM/resolve/0d3b403339b4674a82493d5e97964dd78089ddc8/'
edge='https://huggingface.co/onnx-community/EdgeTAM-ONNX/resolve/9c77c7bff7fd0f3079585fa17af7f730ddc531ed/onnx/'
manifest={}
expected=json.loads((here/'evidence/models.json').read_text())
for name in ['mobile','edge','edge_quantized','edge_qdecoder']:
    folder=lab/'models'/name
    folder.mkdir(parents=True,exist_ok=True)
    if name=='mobile':
        assets={'encoder.onnx':mobile+'mobile_sam_image_encoder.onnx','decoder.onnx':mobile+'sam_mask_decoder_multi.onnx'}
    else:
        encsuffix='_quantized' if name=='edge_quantized' else ''
        decsuffix='_quantized' if name!='edge' else ''
        files=[f'vision_encoder{encsuffix}.onnx',f'prompt_encoder_mask_decoder{decsuffix}.onnx']
        assets={f:edge+f for model in files for f in [model,model+'_data']}
    for filename,url in assets.items():
        dest=folder/filename
        if not dest.exists():
            tmp=dest.with_suffix(dest.suffix+'.partial')
            urllib.request.urlretrieve(url,tmp)
            tmp.replace(dest)
        manifest[f'{name}/{filename}']={'url':url,'bytes':dest.stat().st_size,'sha256':hashlib.sha256(dest.read_bytes()).hexdigest()}
        if manifest[f'{name}/{filename}'] != expected[f'{name}/{filename}']:
            raise ValueError(f'Model identity mismatch: {name}/{filename}')
(lab/'models.json').write_text(json.dumps(manifest,indent=2))
if a.stage_browser:
    public=lab/'public'
    public.mkdir(exist_ok=True)
    shutil.copytree(lab/'models',public/'models',dirs_exist_ok=True)
    shutil.copytree(lab/'results/browser',public/'feeds',dirs_exist_ok=True)
    shutil.copyfile(here/'browser.html',public/'index.html')
    shutil.copyfile(here/'browser-worker.js',public/'browser-worker.js')
    shutil.copyfile(lab/'results/comparison.jpg',public/'comparison.jpg')
    runtime=public/'runtime'
    runtime.mkdir(exist_ok=True)
    for filename in ['ort.all.min.js','ort.all.min.js.map','ort-wasm-simd-threaded.jsep.mjs','ort-wasm-simd-threaded.jsep.wasm','ort-wasm-simd-threaded.mjs','ort-wasm-simd-threaded.wasm']:
        shutil.copyfile(lab/'package/dist'/filename,runtime/filename)
