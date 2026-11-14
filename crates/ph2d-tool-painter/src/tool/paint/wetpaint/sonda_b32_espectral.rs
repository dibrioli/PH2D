//! **B32.1 — a mistura do Wet, K–M de 3 canais × espectral de 10 bandas, no MESMO processo**
//! (`ph2d_pigment::espectral::fixa_lei_do_wet`). Duas sondas ignoradas:
//!
//! - a prancha: tiras vermelho→azul e azul→vermelho nos Wet de fábrica com «pigment mixing», 3 fundos ×
//!   1080p/4K, antes (K–M) | depois (espectral), em `docs/Painter/missao_4k/pranchas/B32/`, e o croma do
//!   cruzamento de cada tira;
//! - o custo: ms por `step_simulation` da poça 4K com as duas tintas, as leis intercaladas, o MÍNIMO.
//!
//! ```text
//! cargo test -p ph2d-tool-painter --profile smoke --lib sonda_b32 -- --ignored --nocapture --test-threads 1
//! ```
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool as _, Tool as _,
};
use ph2d_pigment::ColorMix;
use ph2d_pigment::espectral::fixa_lei_do_wet;
use std::time::Instant;

const VERMELHO: [u8; 3] = [191, 0, 18];
const AZUL: [u8; 3] = [23, 101, 179];
const PINCEIS: [&str; 4] = [
    "Express Oil",
    "Fan Brush",
    "Knife Smooth 3",
    "Marker Bullet",
];

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn biblioteca() -> Biblioteca {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
}

fn preset(b: &Biblioteca, nome: &str) -> ph2d_brush_presets::Preset {
    b.entradas
        .iter()
        .find(|e| e.relativo.starts_with("WetPainter/") && e.preset.parametros.name == nome)
        .unwrap_or_else(|| panic!("o pincel de fábrica {nome}"))
        .preset
        .clone()
}

fn risca(t: &mut PainterTool, de: [f32; 2], ate: [f32; 2], cor: [u8; 3]) {
    t.set_brush_color_srgb8(cor);
    let n = 48;
    t.on_canvas_pointer(cp(de, PointerPhase::Down));
    for k in 1..=n {
        let f = k as f32 / n as f32;
        let p = [de[0] + (ate[0] - de[0]) * f, de[1] + (ate[1] - de[1]) * f];
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(ate, PointerPhase::Up));
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
        for (i, p) in v.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            let c = cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)];
            p[..3].copy_from_slice(&c);
        }
    }
    v
}

/// A poça: as duas tiras (em cima vermelho e depois azul por cima; em baixo o contrário) e `tiques`; devolve
/// também os ms dos três traços que caem em tinta molhada (onde a mistura corre).
fn cena(
    b: &Biblioteca,
    nome: &str,
    (w, h): (u32, u32),
    qual: u8,
    lei: ColorMix,
    tiques: usize,
) -> (PainterTool, f64) {
    fixa_lei_do_wet(lei);
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&preset(b, nome)).is_empty());
    assert!(t.paint.wetpaint.km_mixing, "{nome}: sem pigment mixing");
    let s = h as f32 / 1080.0;
    t.sonda_raio(36.0 * s);
    assert!(t.ensure_wet_session());
    let (fw, fh) = (w as f32, h as f32);
    let (a, z, m0, m1) = (0.2 * fw, 0.8 * fw, 0.42 * fw, 0.58 * fw);
    risca(&mut t, [a, 0.38 * fh], [m1, 0.38 * fh], VERMELHO);
    let t0 = Instant::now();
    risca(&mut t, [m0, 0.38 * fh], [z, 0.38 * fh], AZUL);
    risca(&mut t, [a, 0.62 * fh], [m1, 0.62 * fh], AZUL);
    risca(&mut t, [m0, 0.62 * fh], [z, 0.62 * fh], VERMELHO);
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    t.ticks_em_blocos(tiques);
    (t, ms)
}

fn imagem(t: &mut PainterTool) -> Vec<u8> {
    t.wet_bring_home();
    t.invalidate_composite();
    t.take_preview_arc().expect("o preview").0.to_vec()
}

/// A mediana 9×9 do cruzamento `(x, y)` e o croma `max − min`.
fn cruzamento(img: &[u8], w: u32, x: u32, y: u32) -> ([u8; 3], u8) {
    let c = [0, 1, 2].map(|k| {
        let mut v: Vec<u8> = (y - 4..=y + 4)
            .flat_map(|yy| (x - 4..=x + 4).map(move |xx| (yy, xx)))
            .map(|(yy, xx)| img[((yy * w + xx) * 4) as usize + k])
            .collect();
        v.sort_unstable();
        v[v.len() / 2]
    });
    (c, c.iter().max().unwrap() - c.iter().min().unwrap())
}

#[test]
#[ignore = "prancha B32.1: escreve PNGs em docs/Painter/missao_4k/pranchas/B32/"]
fn sonda_b32_prancha_km_x_espectral() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B32"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    let b = biblioteca();
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, fundo_nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            for nome in PINCEIS {
                let mut par = [ColorMix::Km, ColorMix::Espectral].map(|lei| {
                    let mut t = cena(&b, nome, (w, h), qual, lei, 30).0;
                    imagem(&mut t)
                });
                for (k, img) in par.iter().enumerate() {
                    let lei = ["km", "espectral"][k];
                    for (tira, yf) in [("vermelho→azul", 38u32), ("azul→vermelho", 62)] {
                        let (c, croma) = cruzamento(img, w, w / 2, h * yf / 100);
                        eprintln!(
                            "B32 {res} {fundo_nome} {nome} {tira} {lei}: {},{},{} croma {croma}",
                            c[0], c[1], c[2]
                        );
                    }
                }
                // As duas tiras empilhadas (y 30–46 % e 54–70 %), x 30–70 %: o cruzamento e as duas tintas.
                let (x0, cw, bh, gap) = (w * 30 / 100, w * 40 / 100, h * 16 / 100, 8u32);
                let mut out =
                    image::RgbImage::from_pixel(cw * 2 + gap, bh * 2, image::Rgb([40, 40, 40]));
                for (k, img) in par.iter_mut().enumerate() {
                    for (b, y0) in [h * 30 / 100, h * 54 / 100].into_iter().enumerate() {
                        for y in 0..bh {
                            for x in 0..cw {
                                let i = (((y + y0) * w + x + x0) * 4) as usize;
                                let p = image::Rgb([img[i], img[i + 1], img[i + 2]]);
                                out.put_pixel(x + k as u32 * (cw + gap), y + b as u32 * bh, p);
                            }
                        }
                    }
                }
                let arq = nome.to_lowercase().replace(' ', "_");
                out.save(format!("{dir}/{res}_{fundo_nome}_{arq}_km_espectral.png"))
                    .expect("png");
            }
        }
    }
    fixa_lei_do_wet(ColorMix::Espectral);
}

#[test]
#[ignore = "sonda de custo B32.1 (--profile smoke): ms por passo do Wet 4K, K–M × espectral"]
fn sonda_b32_custo_do_passo_km_x_espectral() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    const RODADAS: usize = 9;
    let b = biblioteca();
    for nome in PINCEIS {
        // Os traços sobre a tinta molhada: cenas inteiras intercaladas, o mínimo de 3.
        let mut traco = [f64::MAX; 2];
        for r in 0..3 {
            for k in [r % 2, 1 - r % 2] {
                let lei = [ColorMix::Km, ColorMix::Espectral][k];
                traco[k] = traco[k].min(cena(&b, nome, (3840, 2160), 0, lei, 0).1);
            }
        }
        let mut t = cena(&b, nome, (3840, 2160), 0, ColorMix::Espectral, 0).0;
        t.wet_bring_home();
        let sess = t.paint.wetpaint.session.as_mut().expect("sessão");
        let e = &mut *sess.engine;
        ph2d_wet_paint::solver::rebuild_active_region(e.active_grid_mut());
        let snap = ph2d_wet_paint::grid::snapshot_grid(e.active_grid());
        let mut passo = [f64::MAX; 2];
        for r in 0..RODADAS {
            for k in [r % 2, 1 - r % 2] {
                fixa_lei_do_wet([ColorMix::Km, ColorMix::Espectral][k]);
                ph2d_wet_paint::grid::restore_grid(e.active_grid_mut(), &snap);
                let t0 = Instant::now();
                e.step_simulation();
                passo[k] = passo[k].min(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
        eprintln!(
            "B32 custo 4K {nome}: passo K–M {:.3} ms · espectral {:.3} ms (×{:.2}) | 3 traços sobre o molhado \
             K–M {:.0} ms · espectral {:.0} ms (×{:.2})",
            passo[0],
            passo[1],
            passo[1] / passo[0].max(1e-9),
            traco[0],
            traco[1],
            traco[1] / traco[0]
        );
    }
    fixa_lei_do_wet(ColorMix::Espectral);
}
