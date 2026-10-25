//! A Color, o Solid, o Type (Blend) e o «On empty» do Wet Paint MEDIDOS num traço de fábrica: cada um muda a
//! tinta, e o «On empty» é inerte onde o código diz (no Mix; sobre a tela opaca).

use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::BrushBlend;

const W: u32 = 200;
const H: u32 = 120;
/// Um laço fechado que atravessa a metade OPACA (x < 100) e a TRANSPARENTE.
const LACO: [[f32; 2]; 5] = [[30.0, 30.0], [170.0, 30.0], [170.0, 90.0], [30.0, 90.0], [30.0, 30.0]];

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// O Wet Paint de fábrica sobre a tela (`opaca`: toda vermelha; senão metade vermelha, metade vazia), `arma` por
/// cima, e o laço pintado.
fn pinta(opaca: bool, arma: impl Fn(&mut PainterTool)) -> Vec<u8> {
    let mut px = vec![0u8; (W * H * 4) as usize];
    for (i, p) in px.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        if opaca || (i as u32 % W) < W / 2 {
            p.copy_from_slice(&[200, 60, 40, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(px, W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_brush_color_srgb8([30, 70, 200]);
    arma(&mut t);
    t.on_canvas_pointer(cp(LACO[0], PointerPhase::Down));
    for w in LACO.windows(2) {
        for k in 1..=12 {
            let f = k as f32 / 12.0;
            let p = [
                w[0][0] + (w[1][0] - w[0][0]) * f,
                w[0][1] + (w[1][1] - w[0][1]) * f,
            ];
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
            t.paint_tick(1.0 / 60.0);
        }
    }
    t.on_canvas_pointer(cp(LACO[4], PointerPhase::Up));
    for _ in 0..20 {
        t.paint_tick(1.0 / 60.0);
    }
    t.wet_bring_home();
    t.canvas_rgba.to_vec()
}

fn difere(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
}

#[test]
fn os_controlos_de_cor_e_modo_do_wet_mudam_a_tinta() {
    let base = pinta(false, |_| {});
    assert_eq!(base, pinta(false, |_| {}), "o controlo: o traço repete-se ao byte");
    let multiplica = |t: &mut PainterTool| t.set_brush_blend(BrushBlend::Multiply as u8);
    let casos: [(&str, Vec<u8>); 4] = [
        ("Color", pinta(false, |t| t.set_brush_color_srgb8([200, 180, 20]))),
        ("Solid", pinta(false, |t| t.toggle_style_solid())),
        ("Type", pinta(false, multiplica)),
        (
            "On empty (Multiply, metade vazia)",
            pinta(false, |t| {
                multiplica(t);
                t.set_blend_sobre_vazio(true);
            }),
        ),
    ];
    for (nome, v) in &casos {
        let contra = if nome.starts_with("On empty") {
            &casos[2].1
        } else {
            &base
        };
        assert!(difere(contra, v) > 0, "{nome} não mudou a tinta");
    }
    // Inerte onde a lei diz: no Mix, e sobre a tela OPACA (as duas leis coincidem com αb = 1).
    let vazio = |t: &mut PainterTool| t.set_blend_sobre_vazio(true);
    assert_eq!(difere(&base, &pinta(false, vazio)), 0, "On empty agiu no Mix");
    let mult_opaca = pinta(true, multiplica);
    let mult_opaca_vazio = pinta(true, |t| {
        multiplica(t);
        t.set_blend_sobre_vazio(true);
    });
    assert_eq!(
        difere(&mult_opaca, &mult_opaca_vazio),
        0,
        "On empty agiu sobre a tela opaca"
    );
}
