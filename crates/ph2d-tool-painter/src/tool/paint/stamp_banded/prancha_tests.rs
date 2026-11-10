//! **Prancha B25.2**: a ponta reta a 45° (o canto chega a `√2·r`) num traço curvo, 3 fundos × 1080p/4K,
//! com os fios da máquina. «Antes» = a porta do disco ([`super::PORTA_DO_DISCO`], o raio cru), «depois» =
//! a porta pelo alcance real; oráculo no mesmo processo. Escreve em `docs/Painter/missao_4k/pranchas/B25a/`.

use super::PORTA_DO_DISCO;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool as _, Tool as _,
};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn fundo(w: u32, h: u32, blocos: bool) -> Vec<u8> {
    let cores = [
        [200u8, 60, 50],
        [60, 150, 210],
        [240, 200, 60],
        [90, 170, 90],
    ];
    let mut v = vec![255u8; (w * h * 4) as usize];
    if blocos {
        for (i, p) in v.chunks_exact_mut(4).enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            let c = cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)];
            p[..3].copy_from_slice(&c);
        }
    }
    v
}

/// A imagem achatada depois do traço: 40 eventos por quadro (a mão rápida num quadro lento).
#[allow(clippy::cast_precision_loss)]
fn cena(w: u32, h: u32, qual: u8, antes: bool, fios: Option<usize>) -> Vec<u8> {
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    let s = h as f32 / 1080.0;
    t.paint.brush.radius_px = 40.0 * s;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.dab_quina = 1.0;
    t.paint.brush.dab_angle_deg = 45;
    t.paint.brush.color = [0.1, 0.2, 0.8];
    let (cx, cy, l) = (w as f32 / 2.0, h as f32 / 2.0, h as f32 / 4.0);
    let passos = 120;
    let ponto = |k: usize| {
        let f = k as f32 / passos as f32;
        [
            cx - l + 2.0 * l * f,
            cy + 0.45 * l * (std::f32::consts::TAU * f).sin(),
        ]
    };
    PORTA_DO_DISCO.with(|p| p.set(antes));
    super::FIOS.with(|f| f.set(fios));
    t.on_canvas_pointer(cp(ponto(0), PointerPhase::Down));
    for k in 1..passos {
        t.on_canvas_pointer(cp(ponto(k), PointerPhase::Move));
        if k % 40 == 0 {
            t.on_tick(1000.0 / 60.0);
        }
    }
    t.on_canvas_pointer(cp(ponto(passos), PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
    PORTA_DO_DISCO.with(|p| p.set(false));
    super::FIOS.with(|f| f.set(None));
    t.invalidate_composite();
    t.take_preview_arc().expect("o preview").0.to_vec()
}

fn recorte(img: &[u8], w: u32, x0: u32, y0: u32, lado: u32, z: u32) -> Vec<[u8; 4]> {
    let mut out = Vec::with_capacity((lado * lado * z * z) as usize);
    for y in 0..lado * z {
        for x in 0..lado * z {
            let i = (((y0 + y / z) * w + x0 + x / z) * 4) as usize;
            out.push([img[i], img[i + 1], img[i + 2], img[i + 3]]);
        }
    }
    out
}

/// Antes | depois | o que muda (vermelho): o recorte central `h/2` e, embaixo, a lupa 4× em `lupa_c`.
fn monta(a: &[u8], d: &[u8], w: u32, h: u32, lupa_c: (u32, u32)) -> image::RgbaImage {
    let lado = h / 2;
    let (x0, y0) = ((w - lado) / 2, (h - lado) / 2);
    let lupa = lado / 4;
    let lx = lupa_c.0.saturating_sub(lupa / 2).min(w - lupa);
    let ly = lupa_c.1.saturating_sub(lupa / 2).min(h - lupa);
    let gap = 8u32;
    let mut par = image::RgbaImage::from_pixel(
        3 * lado + 2 * gap,
        2 * lado + gap,
        image::Rgba([40, 40, 40, 255]),
    );
    for (linha, (ox, oy, l, z)) in [(x0, y0, lado, 1u32), (lx, ly, lupa, 4)]
        .into_iter()
        .enumerate()
    {
        let (ra, rd) = (recorte(a, w, ox, oy, l, z), recorte(d, w, ox, oy, l, z));
        let top = linha as u32 * (lado + gap);
        for (i, (pa, pd)) in ra.iter().zip(&rd).enumerate() {
            let (x, y) = (i as u32 % lado, top + i as u32 / lado);
            let m = (0..4).map(|c| pa[c].abs_diff(pd[c])).max().unwrap_or(0);
            let k = 255 - (u16::from(m) * 32).min(255) as u8;
            par.put_pixel(x, y, image::Rgba([pa[0], pa[1], pa[2], 255]));
            par.put_pixel(x + lado + gap, y, image::Rgba([pd[0], pd[1], pd[2], 255]));
            par.put_pixel(x + 2 * (lado + gap), y, image::Rgba([255, k, k, 255]));
        }
    }
    par
}

#[test]
#[ignore = "prancha B25.2: escreve PNGs em docs/Painter/missao_4k/pranchas/B25a/"]
fn prancha_b25_2_a_quina_rodada_na_borda_da_banda() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B25a"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            for fios in [None, Some(2), Some(7)] {
                let (a, d) = (cena(w, h, qual, true, fios), cena(w, h, qual, false, fios));
                let n = a
                    .chunks_exact(4)
                    .zip(d.chunks_exact(4))
                    .filter(|(x, y)| x != y);
                eprintln!("PRANCHA {res} {nome} fios {fios:?}: {} píxeis", n.count());
            }
            // A imagem: o lote a 32 fios (o mesmo corte que a máquina faz com um lote grande).
            let a = cena(w, h, qual, true, Some(32));
            let d = cena(w, h, qual, false, Some(32));
            let difs: Vec<u32> = a
                .chunks_exact(4)
                .zip(d.chunks_exact(4))
                .enumerate()
                .filter(|(_, (x, y))| x != y)
                .map(|(i, _)| i as u32)
                .collect();
            let pior = a
                .iter()
                .zip(&d)
                .map(|(x, y)| x.abs_diff(*y))
                .max()
                .unwrap_or(0);
            eprintln!(
                "PRANCHA {res} {nome}: {} píxeis diferentes, pior {pior}",
                difs.len()
            );
            let lupa_c = difs.first().map_or((w / 2, h / 2), |&i| (i % w, i / w));
            monta(&a, &d, w, h, lupa_c)
                .save(format!("{dir}/{res}_{nome}_antes_depois.png"))
                .expect("png");
        }
    }
}
