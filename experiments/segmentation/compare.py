"""Reproducible exploratory comparison. Run with uv; see README.md.

Synthetic IoU measures binary geometry, not photo quality or alpha matting.
Real images have identical fixed prompts but no ground-truth masks.
"""
import argparse
import json
import subprocess
import time
from pathlib import Path

import cv2
import numpy as np
import onnxruntime as ort
from PIL import Image, ImageDraw
from skimage import data


def suite(root):
    cases = []
    def add(name, rgb, box, points, truth=None):
        out = root / name
        out.mkdir(parents=True, exist_ok=True)
        Image.fromarray(rgb).save(out / 'image.png')
        rgb.tofile(out / 'image.rgb')
        c = dict(name=name, width=rgb.shape[1], height=rgb.shape[0],
                 rgb=str(out / 'image.rgb'), box=box, points=points, out=str(out))
        (out / 'case.json').write_text(json.dumps(c))
        if truth is not None:
            Image.fromarray(truth.astype('uint8') * 255).save(out / 'truth.png')
        cases.append(c)
    add('astronaut', data.astronaut(), [30, 5, 460, 512],
        [[240, 330, 1], [170, 65, 1], [430, 100, 0]])
    add('cat', data.chelsea(), [10, 5, 435, 300],
        [[230, 150, 1], [180, 60, 1], [430, 25, 0]])
    # Target is the cup including its handle, excluding saucer and spoon.
    add('coffee', data.coffee(), [160, 10, 425, 315],
        [[300, 160, 1], [225, 270, 1], [445, 155, 0], [360, 285, 0]])
    y, x = np.mgrid[:512, :512]
    circle = ((x-250)**2+(y-250)**2) < 155**2
    rng = np.random.default_rng(123)
    for name in ['colour', 'similar', 'multicolour', 'holes_thin']:
        truth = circle.copy()
        bg = np.full((512, 512, 3), [70, 100, 140], dtype=float)
        fg = np.full_like(bg, [190, 95, 45])
        if name == 'similar':
            fg[:] = [85, 108, 143]
        if name == 'multicolour':
            fg[x > 250] = [40, 170, 120]
            # Similar-colour distractor touches the target's right side.
            bg[(x > 390) & (y > 210) & (y < 290)] = [40, 170, 120]
        if name == 'holes_thin':
            truth &= ((x-275)**2+(y-250)**2 > 45**2)
            truth |= (abs(y-(0.25*x+80)) < 2) & (x > 260) & (x < 490)
        rgb = np.where(truth[..., None], fg, bg)
        rgb += rng.normal(0, 3, rgb.shape)
        points = [[200, 260, 1], [330, 250, 1], [455, 250, 0]]
        if name == 'holes_thin':
            points = [[200, 260, 1], [275, 250, 0], [430, 187, 1]]
        add(name, rgb.clip(0, 255).astype('uint8'), [70, 60, 505, 420], points, truth)
    (root / 'cases.json').write_text(json.dumps(cases, indent=2))
    return cases


def session(path):
    options = ort.SessionOptions()
    options.intra_op_num_threads = 4
    options.inter_op_num_threads = 1
    return ort.InferenceSession(str(path), options, providers=['CPUExecutionProvider'])


def timed(fn):
    start = time.perf_counter()
    result = fn()
    return result, (time.perf_counter()-start)*1000


def ml(cases, models, name):
    base = models / name
    suffix = '_quantized' if name.endswith('_quantized') else ''
    encfile = 'encoder.onnx' if name == 'mobile' else f'vision_encoder{suffix}.onnx'
    decsuffix = '_quantized' if name in ['edge_quantized','edge_qdecoder'] else ''
    decfile = 'decoder.onnx' if name == 'mobile' else f'prompt_encoder_mask_decoder{decsuffix}.onnx'
    enc = session(base / encfile)
    dec = session(base / decfile)
    rows = []
    for c in cases:
        out = Path(c['out'])
        im = Image.open(out / 'image.png')
        w, h = im.size
        pts = np.array(c['points'], dtype='float32')
        box = np.array(c['box'], dtype='float32')
        if name == 'mobile':
            scale = 1024 / max(w, h)
            size = (int(w*scale+0.5), int(h*scale+0.5))
            pixels = np.asarray(im.resize(size, Image.Resampling.BILINEAR), dtype='float32')
            inp = {'input_image': pixels}
            pointscale = np.array([size[0]/w, size[1]/h], dtype='float32')
        else:
            pixels = np.asarray(im.resize((1024, 1024), Image.Resampling.BILINEAR), dtype='float32') / 255
            pixels = ((pixels - np.array([.485,.456,.406], dtype='float32')) / np.array([.229,.224,.225], dtype='float32')).transpose(2,0,1)[None].copy()
            inp = {'pixel_values': pixels}
            pointscale = np.array([1024/w, 1024/h], dtype='float32')
        embeddings, enc_ms = timed(lambda: enc.run(None, inp))
        for prompt in ['point', 'box', 'corrected']:
            selected = pts[:1] if prompt != 'corrected' else pts
            xy = selected[:,:2]*pointscale
            labels = selected[:,2]
            if name == 'mobile':
                if prompt != 'point':
                    xy = np.concatenate([xy, box.reshape(2,2)*pointscale])
                    labels = np.concatenate([labels, [2,3]]).astype('float32')
                else:
                    xy = np.concatenate([xy, [[0,0]]]).astype('float32')
                    labels = np.concatenate([labels, [-1]]).astype('float32')
                feed = dict(image_embeddings=embeddings[0], point_coords=xy[None], point_labels=labels[None],
                            mask_input=np.zeros((1,1,256,256), dtype='float32'),
                            has_mask_input=np.zeros(1,dtype='float32'), orig_im_size=np.array([h,w],dtype='float32'))
            else:
                boxes = (box*np.tile(pointscale,2)).reshape(1,1,4) if prompt != 'point' else np.empty((1,0,4),dtype='float32')
                feed = dict(zip([o.name for o in enc.get_outputs()], embeddings))
                feed.update(input_points=xy[None,None],input_labels=labels.astype('int64')[None,None],input_boxes=boxes)
            outputs, dec_ms = timed(lambda: dec.run(None, feed))
            result = dict(zip([o.name for o in dec.get_outputs()], outputs))
            scores = result['iou_predictions' if name == 'mobile' else 'iou_scores'].reshape(-1)
            masks = result['masks' if name == 'mobile' else 'pred_masks']
            masks = masks.reshape(-1,*masks.shape[-2:])
            best = int(scores.argmax())
            logits = cv2.resize(masks[best], (w,h), interpolation=cv2.INTER_LINEAR)
            mask = (logits > 0).astype('uint8')*255
            variant = f'{name}_{prompt}'
            mask.tofile(out / f'{variant}.mask')
            row = dict(case=c['name'],variant=variant,encoder_ms=enc_ms,decoder_ms=dec_ms,score=float(scores[best]))
            rows.append(row)
            print(json.dumps(row), flush=True)
            if c['name'] == 'coffee' and prompt == 'corrected':
                # Exact native-tested feeds, for browser runtime parity and latency.
                browser = out.parent / 'browser' / name
                browser.mkdir(parents=True,exist_ok=True)
                desc={'encoder_model':encfile,'decoder_model':decfile}
                for group,tensors in [('encoder',inp),('decoder',feed)]:
                    desc[group]={}
                    for key,value in tensors.items():
                        file=f'{group}-{key}.bin'
                        value.tofile(browser/file)
                        desc[group][key]=dict(file=file,shape=list(value.shape),dtype=str(value.dtype))
                desc['native_mask_count']=int((mask>0).sum())
                desc['native_scores']=scores.tolist()
                (masks[best]>0).astype('uint8').tofile(browser/'reference-mask.bin')
                desc['reference_mask']='reference-mask.bin'
                (browser/'feeds.json').write_text(json.dumps(desc,indent=2))
    return rows


def gallery(cases, rows, root):
    variants=['object','current_corrections','remembered_corrections','mobile_corrected','edge_corrected','edge_quantized_corrected','edge_qdecoder_corrected']
    thumbw, thumbh = 240, 220
    canvas=Image.new('RGB',(thumbw*(len(variants)+1),thumbh*len(cases)), '#202020')
    draw=ImageDraw.Draw(canvas)
    for r,c in enumerate(cases):
        out=Path(c['out'])
        im=np.asarray(Image.open(out/'image.png'))
        truth=np.asarray(Image.open(out/'truth.png'))>127 if (out/'truth.png').exists() else None
        for col,v in enumerate(['image']+variants):
            shown=im.copy()
            label=v
            if v!='image':
                p=out/f'{v}.mask'
                if not p.exists(): continue
                mask=np.fromfile(p,dtype='uint8').reshape(im.shape[:2])>127
                shown[~mask]=(shown[~mask]*.2).astype('uint8')
                if truth is not None:
                    iou=float((mask&truth).sum()/max(1,(mask|truth).sum()))
                    label+=f' {iou:.3f}'
                    for row in rows:
                        if row['case']==c['name'] and row['variant']==v: row['iou']=iou
            else:
                label=c['name']
            pic=Image.fromarray(shown)
            pic.thumbnail((thumbw-8,thumbh-30))
            canvas.paste(pic,(col*thumbw,r*thumbh+25))
            draw.text((col*thumbw+4,r*thumbh+5),label,fill='white')
    canvas.save(root/'comparison.jpg',quality=94)


if __name__=='__main__':
    p=argparse.ArgumentParser()
    p.add_argument('--lab',type=Path,required=True)
    p.add_argument('--models',nargs='*',default=['mobile','edge'])
    p.add_argument('--append',action='store_true',help='Retain previous results and skip rerunning classical variants')
    args=p.parse_args()
    root=args.lab.resolve()/'results'
    root.mkdir(parents=True,exist_ok=True)
    cases=suite(root)
    binary=Path(__file__).parent/'classical/target/release/selection-comparison'
    rows=json.loads((root/'results.json').read_text()) if args.append else []
    for c in ([] if args.append else cases):
        r=subprocess.run([str(binary),str(Path(c['out'])/'case.json')],capture_output=True,text=True,check=True)
        rows.extend(dict(case=c['name'],**row) for row in json.loads(r.stdout))
        print('classical',c['name'],r.stdout,flush=True)
    for name in args.models:
        try:
            rows.extend(ml(cases,args.lab.resolve()/'models',name))
        except Exception as exc:
            (root/f'{name}-error.txt').write_text(repr(exc))
            raise
    gallery(cases,rows,root)
    (root/'results.json').write_text(json.dumps(rows,indent=2))
