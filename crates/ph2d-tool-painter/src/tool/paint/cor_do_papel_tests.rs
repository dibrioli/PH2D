//! **A cor do papel em cada meio** (pedido do dono, 2026-10-05: *«A cor do papel na watercolor
//! precisa ser revisto pois funciona mal. Deveria funcionar para todos os modos e deveria ter um
//! botão para aplicar no papel como um todo.»*).

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};

const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

/// Um traço horizontal de `x = 20` a `108` em `y = 64` sobre uma tela `128²` (opaca branca, ou
/// transparente), com o papel `papel`; 30 quadros parados depois do pen-up.
fn traco(meio: PaintMedia, transparente: bool, papel: [u8; 3]) -> PainterTool {
    let mut t = tool(128, meio, 6.0);
    if transparente {
        t.set_source(vec![0u8; 128 * 128 * 4], 128, 128);
        t.set_paint_media(meio);
        t.set_brush_size_px(6.0);
    }
    t.set_wet_relogio_fixo(true);
    t.set_paper_color_rgb8(papel[0], papel[1], papel[2]);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.on_canvas_pointer(cp([20.0, 64.0], PointerPhase::Down));
    for k in 1..=22 {
        #[allow(clippy::cast_precision_loss)]
        let x = 20.0 + 4.0 * k as f32;
        t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
        t.paint_tick(1.0 / 60.0);
    }
    t.on_canvas_pointer(cp([108.0, 64.0], PointerPhase::Up));
    for _ in 0..30 {
        t.paint_tick(1.0 / 60.0);
    }
    t
}

/// SONDA — o painel escreve onde · quem lê · o leitor DECIDE? Em cada meio, o mesmo traço com o
/// papel branco e o papel creme: os bytes da camada que mudam, sobre tela opaca e transparente, e
/// o texel do miolo e do papel limpo.
/// `cargo test -p ph2d-tool-painter --lib diag_a_cor_do_papel_em_cada_meio -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_a_cor_do_papel_em_cada_meio() {
    const CREME: [u8; 3] = [230, 200, 150];
    for meio in MEIOS {
        for transparente in [false, true] {
            let a = traco(meio, transparente, [255, 255, 255]);
            let b = traco(meio, transparente, CREME);
            let difere = a
                .canvas_rgba
                .chunks(4)
                .zip(b.canvas_rgba.chunks(4))
                .filter(|(p, q)| p != q)
                .count();
            let px = |t: &PainterTool, x: usize, y: usize| {
                let i = (y * 128 + x) * 4;
                [
                    t.canvas_rgba[i],
                    t.canvas_rgba[i + 1],
                    t.canvas_rgba[i + 2],
                    t.canvas_rgba[i + 3],
                ]
            };
            eprintln!(
                "{meio:?} tela {}: {difere} texels mudam com o papel creme · miolo branco {:?} creme {:?} · papel limpo branco {:?} creme {:?}",
                if transparente {
                    "transparente"
                } else {
                    "opaca"
                },
                px(&a, 64, 64),
                px(&b, 64, 64),
                px(&a, 64, 20),
                px(&b, 64, 20),
            );
        }
    }
}

/// SONDA — o dono (2026-10-05): *«digital realmente cobre o papel, mas não deveria»* · *«impasto
/// reconhece corretamente o papel»*. Com o Relief no máximo e o papel Cold, um traço largo em cada
/// meio: o desvio da luminância da IMAGEM MOSTRADA (composite + luz) no papel nu e dentro da tinta —
/// quanto do dente sobrevive sob a tinta. Relief 0 é o controlo (o dente some nos dois).
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_o_dente_sob_a_tinta -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_o_dente_sob_a_tinta() {
    let lum = |px: &[u8], i: usize| {
        0.299 * f64::from(px[i]) + 0.587 * f64::from(px[i + 1]) + 0.114 * f64::from(px[i + 2])
    };
    let bloco = |(x0, y0, x1, y1): (usize, usize, usize, usize)| {
        (y0..y1).flat_map(move |y| (x0..x1).map(move |x| (y * 128 + x) * 4))
    };
    // O dente: o desvio de `lum(relief 1) − lum(relief 0)` no bloco, e a média da luminância.
    let dente = |a: &[u8], b: &[u8], r| {
        let d: Vec<f64> = bloco(r).map(|i| lum(a, i) - lum(b, i)).collect();
        let m = d.iter().sum::<f64>() / d.len() as f64;
        let sd = (d.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / d.len() as f64).sqrt();
        let media = bloco(r).map(|i| lum(b, i)).sum::<f64>() / d.len() as f64;
        (sd, media)
    };
    for meio in MEIOS {
        let corre = |relief: f32| {
            let mut t = tool(128, meio, 30.0);
            t.set_wet_relogio_fixo(true);
            t.set_brush_paper_kind(26);
            t.paint.substrate_depth = relief;
            t.set_brush_color_srgb8([30, 60, 220]);
            t.on_canvas_pointer(cp([8.0, 64.0], PointerPhase::Down));
            for k in 1..=28 {
                #[allow(clippy::cast_precision_loss)]
                t.on_canvas_pointer(cp([8.0 + 4.0 * k as f32, 64.0], PointerPhase::Move));
                t.paint_tick(1.0 / 60.0);
            }
            t.on_canvas_pointer(cp([120.0, 64.0], PointerPhase::Up));
            for _ in 0..30 {
                t.paint_tick(1.0 / 60.0);
            }
            let (px, _, _) = t.take_preview_arc().expect("a imagem mostrada");
            (px.to_vec(), t.canvas_rgba.to_vec())
        };
        let ((v1, c1), (v0, c0)) = (corre(1.0), corre(0.0));
        let (nu, lnu) = dente(&v1, &v0, (40, 4, 88, 20));
        let (sob, lsob) = dente(&v1, &v0, (40, 56, 88, 72));
        let deposito = bloco((40, 56, 88, 72))
            .filter(|&i| c1[i..i + 4] != c0[i..i + 4])
            .count();
        eprintln!(
            "{meio:?}: dente no papel nu {nu:.2} (luz {lnu:.0}, {:.1} %) · sob a tinta {sob:.2} (luz {lsob:.0}, {:.1} %) · texels do depósito que o papel muda {deposito}/768",
            100.0 * nu / lnu,
            100.0 * sob / lsob,
        );
    }
}
