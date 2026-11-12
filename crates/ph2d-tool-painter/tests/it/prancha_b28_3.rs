//! **Prancha B28.3** (missão 4K): a amostra do Color mixing nos texels pares
//! (`ph2d_painter_brush::mistura_opaca::AMOSTRA_ESPARSA`) contra a inteira, no MESMO processo — Hard Round e Sketch
//! Pencil com Wet blend, r100, 3 fundos × 1080p/4K. Grava `antes | depois | Δ×32` e imprime o Δ médio/máx e o
//! mínimo do relógio de cada variante (intercaladas).
//!
//! ```text
//! cargo test -p ph2d-tool-painter --profile smoke --test it prancha_b28_3 -- --ignored --nocapture
//! ```

use crate::exame_bancada::{Fonte, pinceis};
use ph2d_editor_core::Tool as _;
use ph2d_editor_core::tool::{
    CanvasPaintTool as _, CanvasPointer, PointerPhase, RasterEditTool as _,
};
use ph2d_painter_brush::mistura_opaca::{AMOSTRA_ESPARSA, CARIMBOS_ESPARSOS};
use ph2d_tool_painter::PainterTool;
use std::sync::atomic::Ordering;
use std::time::Instant;

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
            p[..3].copy_from_slice(&cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)]);
        }
    }
    v
}

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A imagem achatada e os ms dos eventos de um zigue-zague que atravessa os blocos.
fn cena(nome: &str, w: u32, h: u32, qual: u8, esparsa: bool) -> (Vec<u8>, f64) {
    cena_r(nome, w, h, qual, esparsa, 100.0)
}

fn cena_r(nome: &str, w: u32, h: u32, qual: u8, esparsa: bool, raio: f32) -> (Vec<u8>, f64) {
    let mut ps = pinceis(nome);
    let Fonte::Preset { preset, .. } = &mut ps[0].fonte else {
        panic!("preset")
    };
    let o = preset
        .parametros
        .brush
        .entry("mistura_opaca".to_owned())
        .or_insert_with(|| serde_json::json!({}));
    o["modo"] = serde_json::json!("WetBlend");
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    ps[0].aplica(&mut t, true);
    t.set_brush_size_px(raio);
    t.set_brush_color_srgb8([30, 50, 140]);
    AMOSTRA_ESPARSA.store(esparsa, Ordering::Relaxed);
    let (wf, hf) = (w as f32, h as f32);
    let pernas = [
        ([0.3 * wf, 0.35 * hf], [0.7 * wf, 0.35 * hf]),
        ([0.7 * wf, 0.35 * hf], [0.3 * wf, 0.65 * hf]),
        ([0.3 * wf, 0.65 * hf], [0.7 * wf, 0.65 * hf]),
    ];
    t.on_tick(1000.0 / 60.0);
    let pre = t
        .take_preview_arc()
        .map(|p| p.0.to_vec())
        .unwrap_or_default();
    CARIMBOS_ESPARSOS.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    t.on_canvas_pointer(ponto(pernas[0].0, PointerPhase::Down));
    let mut n = 0;
    for (a, b) in pernas {
        for k in 1..=40 {
            let f = k as f32 / 40.0;
            t.on_canvas_pointer(ponto(
                [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f],
                PointerPhase::Move,
            ));
            n += 1;
            if n % 3 == 0 {
                t.on_tick(1000.0 / 60.0);
            }
        }
    }
    t.on_canvas_pointer(ponto(pernas[2].1, PointerPhase::Up));
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    t.on_tick(1000.0 / 60.0);
    AMOSTRA_ESPARSA.store(true, Ordering::Relaxed);
    let armou = CARIMBOS_ESPARSOS.load(Ordering::Relaxed);
    assert!(
        armou > 0 || !esparsa,
        "{nome} {w}x{h} fundo {qual}: nenhum carimbo esparso"
    );
    let pos = t.take_preview_arc().expect("o preview").0.to_vec();
    let pintados = pos
        .as_chunks::<4>()
        .0
        .iter()
        .zip(pre.as_chunks::<4>().0.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        pintados > 1000,
        "{nome} {w}x{h} fundo {qual}: o traço não pintou ({pintados} texels)"
    );
    (pos, ms)
}

/// `antes | depois | Δ×32`, recorte `[w/2 ± h/4] × [h/4, 3h/4]`.
fn monta(antes: &[u8], depois: &[u8], w: u32, h: u32) -> image::RgbaImage {
    let lado = h / 2;
    let (x0, y0) = ((w - lado) / 2, h / 4);
    let gap = 8u32;
    let mut par =
        image::RgbaImage::from_pixel(3 * lado + 2 * gap, lado, image::Rgba([40, 40, 40, 255]));
    for y in 0..lado {
        for x in 0..lado {
            let i = (((y0 + y) * w + x0 + x) * 4) as usize;
            for (k, img) in [antes, depois].iter().enumerate() {
                par.put_pixel(
                    x + k as u32 * (lado + gap),
                    y,
                    image::Rgba([img[i], img[i + 1], img[i + 2], 255]),
                );
            }
            let m = (0..4)
                .map(|c| antes[i + c].abs_diff(depois[i + c]))
                .max()
                .unwrap_or(0);
            let k = 255 - (u16::from(m) * 32).min(255) as u8;
            par.put_pixel(x + 2 * (lado + gap), y, image::Rgba([255, k, k, 255]));
        }
    }
    par
}

#[test]
#[ignore = "prancha B28.3: grava as PNG em docs/Painter/missao_4k/pranchas/B28.3/"]
fn prancha_b28_3_amostra_esparsa_do_color_mixing() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B28.3"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for nome in ["Hard Round", "Sketch Pencil"] {
        for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
            for (qual, fundo_nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
                // Antes = amostra inteira + pegada sem Grain em série na tela inteira; depois = as duas curas.
                // Intercalado: antes, depois, depois, antes; o mínimo de cada.
                let corre = |novo: bool| {
                    ph2d_painter_brush::mistura_opaca::PEGADAS_LOCAIS_SEM_GRAO
                        .store(novo, Ordering::Relaxed);
                    let r = cena(nome, w, h, qual, novo);
                    ph2d_painter_brush::mistura_opaca::PEGADAS_LOCAIS_SEM_GRAO
                        .store(true, Ordering::Relaxed);
                    r
                };
                let (antes, a1) = corre(false);
                let (depois, d1) = corre(true);
                let (_, d2) = corre(true);
                let (_, a2) = corre(false);
                let (mut soma, mut max, mut n) = (0u64, 0u8, 0u64);
                for (a, d) in antes.iter().zip(&depois) {
                    let x = a.abs_diff(*d);
                    soma += u64::from(x);
                    max = max.max(x);
                    n += u64::from(x > 0);
                }
                println!(
                    "[b28.3] {nome:<14} {res:<5} {fundo_nome:<6} antes {:>8.1} ms · depois {:>8.1} ms · Δ médio {:.4} (nos tocados {:.2}) máx {max}",
                    a1.min(a2),
                    d1.min(d2),
                    soma as f64 / antes.len() as f64,
                    soma as f64 / n.max(1) as f64,
                );
                let arq = nome.replace(' ', "_").to_lowercase();
                monta(&antes, &depois, w, h)
                    .save(format!("{dir}/{res}_{fundo_nome}_{arq}_antes_depois.png"))
                    .expect("png");
            }
        }
    }
}

/// B28.3: a pegada SEM Grain do lote em buffers locais, em paralelo entre os dabs, dá a tela da rota de antes (o kernel
/// em série na máscara da tela inteira) AO BIT — Hard Round e Sketch Pencil com Wet blend, nos 3 fundos.
#[test]
fn a_pegada_local_sem_grao_da_a_tela_da_rota_de_antes_ao_bit() {
    use ph2d_painter_brush::mistura_opaca::PEGADAS_LOCAIS_SEM_GRAO;
    for nome in ["Hard Round", "Sketch Pencil"] {
        for qual in 0..3u8 {
            PEGADAS_LOCAIS_SEM_GRAO.store(false, Ordering::Relaxed);
            let (antes, _) = cena_r(nome, 640, 360, qual, true, 40.0);
            PEGADAS_LOCAIS_SEM_GRAO.store(true, Ordering::Relaxed);
            let (depois, _) = cena_r(nome, 640, 360, qual, true, 40.0);
            let n = antes.iter().zip(&depois).filter(|(a, b)| a != b).count();
            assert_eq!(n, 0, "{nome} fundo {qual}: {n} bytes mudaram");
        }
    }
}

/// Missão 4K onda 3: a película do Wet blend pintada uma LINHA por tarefa dá a tela das faixas de ladrilhos AO BIT —
/// Hard Round (r40 e r100), Wet Wash Mix (a meia definição) e Round Mixing, nos 3 fundos.
#[test]
fn a_pelicula_por_linhas_da_a_tela_das_faixas_ao_bit() {
    use ph2d_painter_brush::mistura_opaca::PELICULA_POR_LINHAS;
    for (nome, raio) in [
        ("Hard Round", 40.0),
        ("Hard Round", 100.0),
        ("Wet Wash Mix", 100.0),
        ("Round Mixing", 100.0),
    ] {
        for qual in 0..3u8 {
            PELICULA_POR_LINHAS.store(false, Ordering::Relaxed);
            let (antes, _) = cena_r(nome, 640, 360, qual, true, raio);
            PELICULA_POR_LINHAS.store(true, Ordering::Relaxed);
            let (depois, _) = cena_r(nome, 640, 360, qual, true, raio);
            let n = antes.iter().zip(&depois).filter(|(a, b)| a != b).count();
            assert_eq!(n, 0, "{nome} r{raio} fundo {qual}: {n} bytes mudaram");
        }
    }
}

/// Missão 4K onda 3: as pegadas do lote COM o Shape (as cerdas) em buffers locais, em paralelo entre os dabs, dão a tela
/// da rota em série na máscara da tela AO BIT — Oil Paint, Gouache, Thick Oil Paint e Dry Gouache, nos 3 fundos.
#[test]
fn a_pegada_local_com_forma_da_a_tela_da_rota_em_serie_ao_bit() {
    use ph2d_painter_brush::mistura_opaca::PEGADAS_LOCAIS_COM_FORMA;
    for nome in ["Oil Paint", "Gouache", "Thick Oil Paint", "Dry Gouache"] {
        for qual in 0..3u8 {
            PEGADAS_LOCAIS_COM_FORMA.store(false, Ordering::Relaxed);
            let (antes, _) = cena_r(nome, 640, 360, qual, true, 40.0);
            PEGADAS_LOCAIS_COM_FORMA.store(true, Ordering::Relaxed);
            let (depois, _) = cena_r(nome, 640, 360, qual, true, 40.0);
            let n = antes.iter().zip(&depois).filter(|(a, b)| a != b).count();
            assert_eq!(n, 0, "{nome} fundo {qual}: {n} bytes mudaram");
        }
    }
}
