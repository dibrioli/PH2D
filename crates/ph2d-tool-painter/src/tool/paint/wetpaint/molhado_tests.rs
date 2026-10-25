//! **Molhado × molhado** (`Knob::WetOpen`, o cartão «Wet bloom») na ferramenta: o portão do sujo do levantar e as
//! sondas do smoke da janela 8 (os retângulos brancos com Wet bloom e Tip pickup altos; a queda de FPS ao mesclar).
//! Sondas: `PH2D_SONDA_DIR=<pasta> cargo test -p ph2d-tool-painter --profile smoke --lib wetpaint::molhado_tests:: --
//! --ignored --nocapture --test-threads=1`

use super::*;
use crate::tool::PainterTool;
use crate::tool::paint::tests::look_watercolor_arc::{Canvas, write_png};
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase};

const W: usize = 400;
const H: usize = 240;

/// O cenário do smoke: tela branca, Wet Paint, relógio fixo, Wet bloom e Tip pickup como pedidos.
fn ferramenta(w: usize, h: usize, bloom: f64, pickup: f64) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; w * h * 4], w as u32, h as u32);
    t.paint.brush = BrushSpec {
        radius_px: 30.0,
        color: [0.8, 0.1, 0.1],
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    for (id, v) in [
        (crate::ids::PAINTER_WETPAINT_WET_BLOOM, bloom),
        (crate::ids::PAINTER_WETPAINT_TIP_PICKUP, pickup),
    ] {
        assert!(t.route_brush_wetpaint_event(&PanelEvent::SetValue(id, v)));
    }
    t
}

/// ⭐ **O levantar do molhado entra no sujo** (smoke 25/10: «manchas brancas que começam retangulares e se resolvem»):
/// o Wet bloom escreve no grid FORA da pegada do traço (a outra tinta levantada debaixo do tubo), e a tela só recompõe o
/// retângulo sujo — o que ficava fora dele mostrava a tinta de antes até a simulação lá passar, cortado a direito na
/// borda do retângulo. Quadro a quadro do traço escuro sobre o vermelho molhado, a tela incremental é a do recomposite
/// INTEIRO. CONTROLO: o levantar MEXEU na tela (sem ele, outra tela). Mutação que sangra: `molhado_levanta` a devolver
/// `None` (o retângulo que ele tocou não chega ao sujo).
#[test]
fn o_levantar_do_molhado_entra_no_sujo_e_a_tela_e_a_do_recomposite() {
    let pinta = |empurra: f64| {
        let mut t = ferramenta(W, H, 1.0, 0.0);
        t.paint
            .wetpaint
            .knobs
            .set(ph2d_wet_paint::tuning::Knob::WetOpenPush, empurra);
        traco(
            &mut t,
            [0.8, 0.1, 0.1],
            30.0,
            [60.0, 110.0],
            [300.0, 110.0],
            &mut |_, _| {},
        );
        t.paint.brush.color = [0.12, 0.09, 0.07];
        t.paint.brush.radius_px = 24.0;
        t.paint.brush_by_mode.fill(t.paint.brush);
        let (de, ate) = ([170.0f32, 140.0f32], [370.0f32, 130.0f32]);
        t.on_canvas_pointer(cp(de, PointerPhase::Down));
        let mut diferentes = Vec::new();
        for k in 1..=40 {
            let a = k as f32 / 40.0;
            t.on_canvas_pointer(cp(
                [de[0] + (ate[0] - de[0]) * a, de[1] + (ate[1] - de[1]) * a],
                PointerPhase::Move,
            ));
            t.on_tick(1000.0 / 60.0);
            let incremental = t.canvas_rgba.to_vec();
            if let Some(sess) = t.paint.wetpaint.session.as_mut() {
                sess.bring_home();
                sess.engine.mark_dirty_full();
            }
            crate::tool::paint::wetpaint::composite_for_measure(&mut t);
            let n = incremental
                .as_chunks::<4>()
                .0
                .iter()
                .zip(t.canvas_rgba.as_chunks::<4>().0.iter())
                .filter(|(a, b)| a != b)
                .count();
            if n > 0 {
                diferentes.push((k, n));
            }
        }
        t.on_canvas_pointer(cp(ate, PointerPhase::Up));
        (diferentes, t.canvas_rgba.to_vec())
    };
    let (diferentes, com) = pinta(0.8);
    assert!(
        diferentes.is_empty(),
        "a tela incremental não é a do recomposite: (quadro, píxeis) {diferentes:?}"
    );
    let (_, sem) = pinta(0.0);
    assert_ne!(sem, com, "CONTROLO: o levantar não mexeu na tela");
}

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(
    t: &mut PainterTool,
    cor: [f32; 3],
    r: f32,
    de: [f32; 2],
    ate: [f32; 2],
    meio: &mut dyn FnMut(&PainterTool, usize),
) {
    t.paint.brush.color = cor;
    t.paint.brush.radius_px = r;
    t.paint.brush_by_mode.fill(t.paint.brush);
    t.on_canvas_pointer(cp(de, PointerPhase::Down));
    let n = 40;
    for k in 1..=n {
        let a = k as f32 / n as f32;
        t.on_canvas_pointer(cp(
            [de[0] + (ate[0] - de[0]) * a, de[1] + (ate[1] - de[1]) * a],
            PointerPhase::Move,
        ));
        t.on_tick(1000.0 / 60.0);
        meio(t, k);
    }
    t.on_canvas_pointer(cp(ate, PointerPhase::Up));
}

fn foto(t: &PainterTool, dir: &str, nome: &str) {
    let mut c = Canvas::new(W, H);
    for (i, px) in t.canvas_rgba.as_chunks::<4>().0.iter().enumerate() {
        let a = u32::from(px[3]);
        let sobre = |v: u8| ((u32::from(v) * a + 255 * (255 - a)) / 255) as u8;
        c.px[i] = [sobre(px[0]), sobre(px[1]), sobre(px[2])];
    }
    write_png(&std::path::Path::new(dir).join(format!("{nome}.png")), &c).expect("PNG");
}

#[test]
#[ignore = "sonda: fotografa os quadros; PH2D_SONDA_DIR"]
fn sonda_retangulos() {
    let dir = std::env::var("PH2D_SONDA_DIR").unwrap_or_else(|_| "/tmp".into());
    for (nome, bloom, pickup, empurra) in [
        ("ambos", 1.0, 1.0, None),
        ("bloom", 1.0, 0.0, None),
        ("pickup", 0.0, 1.0, None),
        ("nenhum", 0.0, 0.0, None),
        ("bloomsem", 1.0, 0.0, Some(0.0)),
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        t.paint.brush = BrushSpec {
            radius_px: 30.0,
            color: [0.8, 0.1, 0.1],
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        for (id, v) in [
            (crate::ids::PAINTER_WETPAINT_WET_BLOOM, bloom),
            (crate::ids::PAINTER_WETPAINT_TIP_PICKUP, pickup),
        ] {
            assert!(t.route_brush_wetpaint_event(&PanelEvent::SetValue(id, v)));
        }
        if let Some(v) = empurra {
            t.paint
                .wetpaint
                .knobs
                .set(ph2d_wet_paint::tuning::Knob::WetOpenPush, v);
        }
        traco(
            &mut t,
            [0.8, 0.1, 0.1],
            30.0,
            [60.0, 110.0],
            [300.0, 110.0],
            &mut |_, _| {},
        );
        foto(&t, &dir, &format!("{nome}_0_vermelho"));
        traco(
            &mut t,
            [0.12, 0.09, 0.07],
            24.0,
            [170.0, 140.0],
            [370.0, 130.0],
            &mut |t, k| {
                if k % 10 == 0 {
                    foto(t, &dir, &format!("{nome}_1_escuro_dab{k:02}"));
                }
            },
        );
        for q in 1..=240 {
            t.on_tick(1000.0 / 60.0);
            if [1, 5, 15, 40, 100, 240].contains(&q) {
                foto(&t, &dir, &format!("{nome}_2_q{q:03}"));
            }
        }
        eprintln!("{nome}: ok");
    }
}

/// A queda de FPS ao mesclar (smoke da janela 8): ms por quadro do traço escuro (Move + tick) e do assentar, sobre o
/// vermelho molhado × sobre papel limpo, por variante dos dois knobs; rodadas intercaladas, o MÍNIMO.
#[test]
#[ignore = "sonda: cronometra; correr sozinha"]
fn sonda_fps_mesclar() {
    const E: f32 = 3.0;
    let (w, h) = ((W as f32 * E) as usize, (H as f32 * E) as usize);
    let variantes = [
        ("ambos", 1.0, 1.0),
        ("bloom", 1.0, 0.0),
        ("pickup", 0.0, 1.0),
        ("nenhum", 0.0, 0.0),
    ];
    let mut melhor = [[[f64::MAX; 3]; 2]; 4];
    for _ronda in 0..5 {
        for (vi, &(_, bloom, pickup)) in variantes.iter().enumerate() {
            for (si, sobre) in [true, false].into_iter().enumerate() {
                let mut t = PainterTool::default();
                t.set_source(vec![255u8; w * h * 4], w as u32, h as u32);
                t.paint.brush = BrushSpec {
                    radius_px: 30.0 * E,
                    color: [0.8, 0.1, 0.1],
                    ..Default::default()
                };
                t.paint.brush_by_mode.fill(t.paint.brush);
                t.set_paint_tool_mode("wetpaint");
                t.set_wet_relogio_fixo(true);
                for (id, v) in [
                    (crate::ids::PAINTER_WETPAINT_WET_BLOOM, bloom),
                    (crate::ids::PAINTER_WETPAINT_TIP_PICKUP, pickup),
                ] {
                    assert!(t.route_brush_wetpaint_event(&PanelEvent::SetValue(id, v)));
                }
                let dy = if sobre { 0.0 } else { 120.0 * E };
                traco(
                    &mut t,
                    [0.8, 0.1, 0.1],
                    30.0 * E,
                    [60.0 * E, 70.0 * E],
                    [300.0 * E, 70.0 * E],
                    &mut |_, _| {},
                );
                let mut pior = 0.0f64;
                let t0 = std::time::Instant::now();
                let mut ultimo = t0;
                traco(
                    &mut t,
                    [0.12, 0.09, 0.07],
                    24.0 * E,
                    [170.0 * E, 90.0 * E + dy],
                    [370.0 * E, 85.0 * E + dy],
                    &mut |_, _| {
                        let agora = std::time::Instant::now();
                        pior = pior.max((agora - ultimo).as_secs_f64() * 1e3);
                        ultimo = agora;
                    },
                );
                let traco_ms = t0.elapsed().as_secs_f64() * 1e3 / 40.0;
                let t1 = std::time::Instant::now();
                for _ in 0..60 {
                    t.on_tick(1000.0 / 60.0);
                }
                let assenta_ms = t1.elapsed().as_secs_f64() * 1e3 / 60.0;
                let m = &mut melhor[vi][si];
                m[0] = m[0].min(traco_ms);
                m[1] = m[1].min(pior);
                m[2] = m[2].min(assenta_ms);
            }
        }
    }
    eprintln!(
        "loadavg {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    eprintln!(
        "{:8} {:>8} | {:>9} {:>9} {:>9} | {:>9} {:>9} {:>9}",
        "variante", "", "traço", "pior", "assenta", "traço", "pior", "assenta"
    );
    eprintln!(
        "{:8} {:>8} | {:^29} | {:^29}",
        "", "ms/qd", "sobre o vermelho molhado", "papel limpo"
    );
    for (vi, (nome, _, _)) in variantes.iter().enumerate() {
        let [a, b] = melhor[vi];
        eprintln!(
            "{nome:8} {:>8} | {:9.2} {:9.2} {:9.2} | {:9.2} {:9.2} {:9.2}",
            "", a[0], a[1], a[2], b[0], b[1], b[2]
        );
    }
}
