//! **Prancha B13.4**: a cadência da secagem é do TILE 64×64. Duas poças finas nos cantos (sup. esquerdo e
//! inf. direito), ainda molhadas; depois a gravidade forte (`gravity_override = [0, 1]`, nas três sessões) e um
//! pingo de água no meio que CORRE ladeira abaixo. «Antes» = a cadência global
//! ([`ph2d_wet_paint::drying::por_tile::CADENCIA_GLOBAL`]: o pingo muda o ritmo de TODAS as poças), «depois» = a
//! do tile, no mesmo processo; «sem pingo» é a régua: depois × sem pingo nas poças tem de dar 0. PNG em
//! `docs/Painter/missao_4k/pranchas/B13.4/`: a poça esquerda e a direita | antes | depois | o que muda.

use super::fundo;
use crate::{PaintMedia, PainterTool};
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool as _, Tool as _,
};
use std::sync::atomic::Ordering::Relaxed;

/// Os centros das duas poças, em fração da folha.
const POCAS: [[f32; 2]; 2] = [[0.06, 0.2], [0.94, 0.8]];

fn traco(t: &mut PainterTool, a: [f32; 2], b: [f32; 2]) {
    let passos = 40;
    let cp = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let p = |k: usize| {
        let f = k as f32 / passos as f32;
        [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]
    };
    t.on_canvas_pointer(cp(p(0), PointerPhase::Down));
    for k in 1..passos {
        t.on_canvas_pointer(cp(p(k), PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(p(passos), PointerPhase::Up));
}

/// O preview e os passos em que a água correu (`dry_every == 3`).
fn cena(w: u32, h: u32, qual: u8, pingo: bool) -> (Vec<u8>, usize) {
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    t.add_raster_layer("tinta").expect("a camada transparente");
    let (wf, hf, s) = (w as f32, h as f32, h as f32 / 1080.0);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.paint.wetpaint.tilt_on = false;
    t.set_brush_size_px(16.0 * s);
    t.set_brush_color_srgb8([40, 60, 190]);
    for [cx, cy] in POCAS {
        let (x, y) = (cx * wf, cy * hf);
        traco(&mut t, [x - 30.0 * s, y], [x + 30.0 * s, y]);
    }
    t.wet_bring_home();
    t.paint
        .wetpaint
        .session
        .as_mut()
        .expect("a sessão")
        .engine
        .sim
        .gravity_override = Some([0.0, 1.0]);
    if pingo {
        t.set_brush_size_px(60.0 * s);
        t.set_brush_color_srgb8([200, 40, 40]);
        traco(&mut t, [0.4 * wf, 0.3 * hf], [0.6 * wf, 0.3 * hf]);
    }
    let mut correndo = 0;
    for _ in 0..160 {
        t.on_tick(1000.0 / 60.0);
        t.wet_bring_home();
        let e = &t.paint.wetpaint.session.as_ref().expect("a sessão").engine;
        correndo += usize::from(e.sim.dry_every == 3);
    }
    t.invalidate_composite();
    (t.take_preview_arc().expect("o preview").0.to_vec(), correndo)
}

/// A caixa `lado × lado` de cada poça (ampliada 2×): antes | depois | diferença, uma linha por poça.
fn caixas(w: u32, h: u32) -> [(u32, u32); 2] {
    let lado = h / 8;
    POCAS.map(|[cx, cy]| {
        let x0 = ((cx * w as f32) as u32).saturating_sub(lado / 2).min(w - lado);
        let y0 = ((cy * h as f32) as u32).saturating_sub(lado / 2).min(h - lado);
        (x0, y0)
    })
}

fn nas_pocas(a: &[u8], d: &[u8], w: u32, h: u32) -> usize {
    let lado = h / 8;
    caixas(w, h)
        .into_iter()
        .map(|(x0, y0)| {
            (y0..y0 + lado)
                .flat_map(|y| (x0..x0 + lado).map(move |x| ((y * w + x) * 4) as usize))
                .filter(|&i| a[i..i + 4] != d[i..i + 4])
                .count()
        })
        .sum()
}

fn monta(a: &[u8], d: &[u8], w: u32, h: u32) -> image::RgbaImage {
    let (lado, z, gap) = (h / 8, 2u32, 8u32);
    let q = lado * z;
    let mut par =
        image::RgbaImage::from_pixel(3 * q + 2 * gap, 2 * q + gap, image::Rgba([40, 40, 40, 255]));
    for (linha, (x0, y0)) in caixas(w, h).into_iter().enumerate() {
        for y in 0..q {
            for x in 0..q {
                let i = (((y0 + y / z) * w + x0 + x / z) * 4) as usize;
                let (pa, pd) = (&a[i..i + 4], &d[i..i + 4]);
                let m = (0..4).map(|c| pa[c].abs_diff(pd[c])).max().unwrap_or(0);
                let k = 255 - (u16::from(m) * 32).min(255) as u8;
                let top = linha as u32 * (q + gap) + y;
                par.put_pixel(x, top, image::Rgba([pa[0], pa[1], pa[2], 255]));
                par.put_pixel(x + q + gap, top, image::Rgba([pd[0], pd[1], pd[2], 255]));
                par.put_pixel(x + 2 * (q + gap), top, image::Rgba([255, k, k, 255]));
            }
        }
    }
    par
}

#[test]
#[ignore = "prancha B13.4: escreve PNGs em docs/Painter/missao_4k/pranchas/B13.4/"]
fn prancha_b13_4_a_secagem_por_tile() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B13.4"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            ph2d_wet_paint::drying::por_tile::CADENCIA_GLOBAL.store(true, Relaxed);
            let (a, ca) = cena(w, h, qual, true);
            ph2d_wet_paint::drying::por_tile::CADENCIA_GLOBAL.store(false, Relaxed);
            let (d, cd) = cena(w, h, qual, true);
            let (sem, c0) = cena(w, h, qual, false);
            let antes = nas_pocas(&a, &d, w, h);
            let regua = nas_pocas(&sem, &d, w, h);
            eprintln!(
                "PRANCHA {res} {nome}: a correr antes {ca} · depois {cd} · sem pingo {c0}; nas poças antes×depois \
                 {antes} píxeis, depois×sem pingo {regua}"
            );
            // A contagem de passos a correr não distingue (a gravidade também corre as poças a cada passo): o que
            // importa é a lei antiga mudar as poças com o pingo e a nova não.
            assert!(antes > 0, "a cadência global não mudou as poças: a prancha não mostra nada");
            assert_eq!(regua, 0, "o pingo longe ainda muda as poças ({regua} píxeis)");
            monta(&a, &d, w, h)
                .save(format!("{dir}/{res}_{nome}_antes_depois.png"))
                .expect("png");
        }
    }
}
