//! Sonda: o custo por quadro do pincel de fábrica da Aquarela (números do dono, 23/10) contra o
//! neutro de antes (`BrushSpec::default`), intercalados no mesmo processo; resumo = o MÍNIMO.

use super::*;
use crate::tool::paint::media::PaintMedia;

fn traco_ms(liga: &str, raio: f32) -> f64 {
    let neutro = liga != "tudo";
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 1024 * 1024 * 4], 1024, 1024);
    t.set_paint_media(PaintMedia::Watercolor);
    t.set_brush_size_px(2.0 * raio);
    if neutro {
        let d = BrushSpec::default();
        let b = &mut t.paint.brush;
        (b.fill, b.depth, b.edge_gain, b.edge_spread, b.warp) =
            (d.fill, d.depth, d.edge_gain, d.edge_spread, d.warp);
        b.edge_flow = d.edge_flow;
        (
            b.wet_charge,
            b.wet_dilution,
            b.wet_pull,
            b.wet_rewet,
            b.wet_smudge,
        ) = (
            d.wet_charge,
            d.wet_dilution,
            d.wet_pull,
            d.wet_rewet,
            d.wet_smudge,
        );
        match liga {
            "smudge" => b.wet_smudge = 0.5,
            "rewet" => b.wet_rewet = 0.5,
            "charge" => (b.wet_charge, b.wet_dilution, b.wet_pull) = (0.5, 0.2, 0.6),
            "forma" => {
                (b.fill, b.depth, b.edge_gain, b.edge_spread, b.warp) = (0.25, 2.0, 0.2, 4.0, 12.0);
                b.edge_flow.size = [2.0, 2.0];
            }
            _ => {}
        }
    }
    let mut pior = 0.0f64;
    let mut soma = 0.0f64;
    for y in [400.0f32, 430.0] {
        t.on_canvas_pointer(cp([200.0, y], PointerPhase::Down));
        for k in 1..=120 {
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp([200.0 + 5.0 * k as f32, y], PointerPhase::Move));
            frame(&mut t);
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            pior = pior.max(ms);
            soma += ms;
        }
        t.on_canvas_pointer(cp([800.0, y], PointerPhase::Up));
        frame(&mut t);
    }
    let _ = pior;
    soma / 240.0
}

#[test]
#[ignore = "sonda de velocidade"]
fn sonda_custo_aquarela_de_fabrica() {
    let vs = ["neutro", "smudge", "rewet", "charge", "forma", "tudo"];
    for raio in [60.0f32, 120.0] {
        let mut m = [f64::MAX; 6];
        for _ in 0..3 {
            for (i, v) in vs.iter().enumerate() {
                m[i] = m[i].min(traco_ms(v, raio));
            }
        }
        for (i, v) in vs.iter().enumerate() {
            eprintln!("raio {raio} {v:>7}: {:.2} ms/quadro", m[i]);
        }
    }
}

/// O custo do Rewet cresce com o que a sessão já pintou? Quadros 1–20 contra 101–120 de cada traço,
/// e o 4.º traço da sessão contra o 1.º.
#[test]
#[ignore = "sonda de velocidade"]
fn sonda_rewet_cresce_com_a_sessao() {
    for rewet in [0.0f32, 0.5] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 1024 * 1024 * 4], 1024, 1024);
        t.set_paint_media(PaintMedia::Watercolor);
        t.set_brush_size_px(120.0);
        let d = BrushSpec::default();
        let b = &mut t.paint.brush;
        (b.wet_charge, b.wet_dilution, b.wet_pull, b.wet_smudge) = (d.wet_charge, 0.0, 0.0, 0.0);
        b.wet_rewet = rewet;
        for (n, y) in [200.0f32, 330.0, 460.0, 590.0, 720.0].iter().enumerate() {
            let mut ms = Vec::new();
            t.on_canvas_pointer(cp([100.0, *y], PointerPhase::Down));
            for k in 1..=160 {
                let t0 = std::time::Instant::now();
                t.on_canvas_pointer(cp([100.0 + 5.0 * k as f32, *y], PointerPhase::Move));
                frame(&mut t);
                ms.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            t.on_canvas_pointer(cp([900.0, *y], PointerPhase::Up));
            frame(&mut t);
            let med = |s: &[f64]| s.iter().sum::<f64>() / s.len() as f64;
            eprintln!(
                "rewet {rewet} traço {}: quadros 1–20 {:.2} ms · 141–160 {:.2} ms",
                n + 1,
                med(&ms[..20]),
                med(&ms[140..])
            );
        }
    }
}

/// Desligar o «Automatic» da Shape pinta o mesmo carimbo? Por variante da fábrica nova.
#[test]
#[ignore = "sonda"]
fn sonda_automatic_desligado() {
    let pinta = |liga: &str, auto: bool| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 256 * 256 * 4], 256, 256);
        t.set_paint_media(PaintMedia::Watercolor);
        t.set_brush_size_px(40.0);
        t.set_brush_color_srgb8([40, 90, 200]);
        let d = BrushSpec::default();
        let b = &mut t.paint.brush;
        if liga != "tudo" {
            (b.fill, b.depth, b.edge_gain, b.edge_spread, b.warp) =
                (d.fill, d.depth, d.edge_gain, d.edge_spread, d.warp);
            b.edge_flow = d.edge_flow;
            (
                b.wet_charge,
                b.wet_dilution,
                b.wet_pull,
                b.wet_rewet,
                b.wet_smudge,
            ) = (
                d.wet_charge,
                d.wet_dilution,
                d.wet_pull,
                d.wet_rewet,
                d.wet_smudge,
            );
            match liga {
                "smudge" => b.wet_smudge = 0.5,
                "rewet" => b.wet_rewet = 0.5,
                "charge" => (b.wet_charge, b.wet_dilution, b.wet_pull) = (0.5, 0.2, 0.6),
                "forma" => {
                    (b.fill, b.depth, b.edge_gain, b.edge_spread, b.warp) =
                        (0.25, 2.0, 0.2, 4.0, 12.0);
                    b.edge_flow.size = [2.0, 2.0];
                }
                _ => {}
            }
        }
        if !auto {
            t.toggle_brush_watercolor_shape_auto();
        }
        for y in [100.0f32, 120.0] {
            t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
            for k in 1..=40 {
                t.on_canvas_pointer(cp(
                    [40.0 + 4.0 * k as f32, y + 0.3 * k as f32],
                    PointerPhase::Move,
                ));
                frame(&mut t);
            }
            t.on_canvas_pointer(cp([200.0, y + 12.0], PointerPhase::Up));
            frame(&mut t);
        }
        t.canvas_rgba.to_vec()
    };
    for v in ["neutro", "smudge", "rewet", "charge", "forma", "tudo"] {
        let (a, b) = (pinta(v, true), pinta(v, false));
        let (mut n, mut mx) = (0usize, 0u8);
        for (p, q) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0) {
            let d = (0..4).map(|c| p[c].abs_diff(q[c])).max().unwrap();
            if d > 0 {
                n += 1;
                mx = mx.max(d);
            }
        }
        eprintln!("{v:>7}: {n} texels diferem, máximo {mx}");
    }
}
