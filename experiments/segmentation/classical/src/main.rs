//! Evaluation harness, not a shipped command. Trusted, generated inputs only.
use photocraft_algo::{
    segment::{self, ImageSampler, RgbImage, grabcut, quick},
    selection::{self, Region},
};
use photocraft_geom::Rect;
use serde_json::{Value, json};
use std::{error::Error, fs, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args().nth(1).ok_or("manifest required")?;
    let v: Value = serde_json::from_slice(&fs::read(path)?)?;
    let w = v["width"].as_u64().ok_or("width")? as usize;
    let h = v["height"].as_u64().ok_or("height")? as usize;
    let bytes = fs::read(v["rgb"].as_str().ok_or("rgb")?)?;
    if bytes.len() != w * h * 3 {
        return Err("RGB length".into());
    }
    let img = RgbImage {
        w,
        h,
        px: bytes
            .chunks_exact(3)
            .map(|p| [p[0] as f32 / 255., p[1] as f32 / 255., p[2] as f32 / 255.])
            .collect(),
    };
    let smp = ImageSampler {
        img: &img,
        origin: (0, 0),
    };
    let canvas = Rect::new(0, 0, w as i32, h as i32);
    let b: Vec<i32> = v["box"]
        .as_array()
        .ok_or("box")?
        .iter()
        .map(|n| n.as_i64().unwrap_or(0) as i32)
        .collect();
    if b.len() != 4 {
        return Err("box length".into());
    }
    let rect = Rect::new(b[0], b[1], b[2], b[3]);
    let points = v["points"].as_array().ok_or("points")?;
    let point = |p: &Value| {
        (
            p[0].as_f64().unwrap_or(0.) as f32,
            p[1].as_f64().unwrap_or(0.) as f32,
        )
    };
    let out = v["out"].as_str().ok_or("out")?;
    fs::create_dir_all(out)?;
    let mut results = Vec::new();
    let mut save = |name: &str, r: Option<&Region>, ms: f64| -> Result<(), Box<dyn Error>> {
        let mask: Vec<u8> = (0..h)
            .flat_map(|y| {
                (0..w)
                    .map(move |x| r.map_or(0, |r| (r.at(x as i32, y as i32) * 255.).round() as u8))
            })
            .collect();
        fs::write(format!("{out}/{name}.mask"), mask)?;
        results.push(json!({"variant": name,"ms": ms}));
        Ok(())
    };
    let t = Instant::now();
    let rgba: Vec<[u8; 4]> = bytes
        .chunks_exact(3)
        .map(|p| [p[0], p[1], p[2], 255])
        .collect();
    let p = point(&points[0]);
    let wand = selection::wand_region(&rgba, canvas, (p.0 as i32, p.1 as i32), 32., true, true);
    save("wand", wand.as_ref(), t.elapsed().as_secs_f64() * 1000.)?;
    let t = Instant::now();
    let q = quick::quick_select(&smp, canvas, &[p], 12., quick::WORK_PX);
    save("quick", q.as_ref(), t.elapsed().as_secs_f64() * 1000.)?;
    let t = Instant::now();
    let obj = grabcut::object_select(&smp, canvas, rect, 160_000);
    save("object", obj.as_ref(), t.elapsed().as_secs_f64() * 1000.)?;
    // Existing correction semantics: independent region, then add/subtract.
    let t = Instant::now();
    let mut current: Vec<u8> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| {
            obj.as_ref()
                .map_or(0, |r| (r.at(x as i32, y as i32) * 255.) as u8)
        })
        .collect();
    for pt in points {
        if let Some(r) = quick::quick_select(&smp, canvas, &[point(pt)], 12., quick::WORK_PX) {
            for y in 0..h {
                for x in 0..w {
                    let a = (r.at(x as i32, y as i32) * 255.) as u8;
                    let m = &mut current[y * w + x];
                    *m = if pt[2].as_i64() == Some(1) {
                        (*m).max(a)
                    } else {
                        m.saturating_sub(a)
                    };
                }
            }
        }
    }
    let corrected = Region {
        bbox: canvas,
        mask: current,
    };
    save(
        "current_corrections",
        Some(&corrected),
        t.elapsed().as_secs_f64() * 1000.,
    )?;
    // Prototype: retain all foreground/background marks in one GrabCut solve.
    let t = Instant::now();
    let step = segment::scale_for(canvas, 160_000);
    let small = img.downsample(step);
    let mut trimap: Vec<u8> = (0..small.h)
        .flat_map(|y| (0..small.w).map(move |x| (x, y)))
        .map(|(x, y)| {
            let (x, y) = (
                (x * step + step / 2).min(w - 1) as i32,
                (y * step + step / 2).min(h - 1) as i32,
            );
            if !rect.contains(x, y) {
                grabcut::BG
            } else if obj.as_ref().is_some_and(|r| r.at(x, y) > 0.5) {
                grabcut::PR_FG
            } else {
                grabcut::PR_BG
            }
        })
        .collect();
    for pt in points {
        let (x, y) = point(pt);
        let seeds = quick::stroke_mask(
            &[(x / step as f32, y / step as f32)],
            (6. / step as f32).max(0.75),
            small.w,
            small.h,
        );
        for (a, s) in trimap.iter_mut().zip(seeds) {
            if s {
                *a = if pt[2].as_i64() == Some(1) {
                    grabcut::FG
                } else {
                    grabcut::BG
                };
            }
        }
    }
    let region = grabcut::grabcut(&small, &mut trimap, 5).and_then(|(fg, bg)| {
        let low: Vec<bool> = trimap
            .iter()
            .map(|t| *t == grabcut::FG || *t == grabcut::PR_FG)
            .collect();
        segment::finish_region(&smp, canvas, step, &low, small.w, small.h, Some((&fg, &bg)))
    });
    save(
        "remembered_corrections",
        region.as_ref(),
        t.elapsed().as_secs_f64() * 1000.,
    )?;
    println!("{}", serde_json::to_string(&results)?);
    Ok(())
}
