//! N10 W2a2: o sujo que o Ctrl+Z vivo entrega à óptica é a caixa do restauro, não a tela.
use crate::tool::PainterTool;
use crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_wet_paint::painter::conta_sujo;
use std::sync::atomic::Ordering::Relaxed;

const W: usize = 1280;
const H: usize = 800;

fn ponto(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// `""` = a ferramenta de omissão no Wet (sem preset).
fn tool(nome: &str) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_paper_field(|b| b.paper = crate::tool::paint::substrate_relief::default_paper());
    if !nome.is_empty() {
        let p = de_fabrica_ou_candidato(nome);
        assert!(t.wet_preset_aplicar(&p).is_empty());
        t.wet_preset_aplicar_volumes(&p.parametros.volumes);
    }
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([30, 60, 200]);
    t
}

fn traco(t: &mut PainterTool, y: f32) {
    t.on_canvas_pointer(ponto(200.0, y, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(ponto(200.0 + 4.0 * k as f32, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponto(320.0, y, PointerPhase::Up));
    for _ in 0..8 {
        t.on_tick(1000.0 / 60.0);
    }
}

fn quadros(t: &mut PainterTool, n: usize) {
    for _ in 0..n {
        t.on_tick(1000.0 / 60.0);
    }
}

/// As células que o Ctrl+Z (ou o redo) entrega à óptica, e a tela depois de ela compor.
fn volta(t: &mut PainterTool, refaz: bool, cheio: bool) -> (u64, Vec<u8>) {
    ph2d_wet_paint::painter::UNDO_SUJO_DOS_PLANOS.store(cheio, Relaxed);
    conta_sujo::CONTA_SUJO.store(true, Relaxed);
    let _ = conta_sujo::tira();
    assert!(if refaz { t.redo_last() } else { t.undo_last() });
    conta_sujo::CONTA_SUJO.store(false, Relaxed);
    ph2d_wet_paint::painter::UNDO_SUJO_DOS_PLANOS.store(false, Relaxed);
    let celulas = conta_sujo::tira().iter().map(|c| c.2).sum();
    quadros(t, 4);
    (celulas, t.canvas_rgba.to_vec())
}

/// ⭐ **N10 W2a2 — o Ctrl+Z vivo suja a CAIXA do restauro, não a tela** (o 1.º traço aloca os planos preguiçosos do
/// relevo/óleo; o undo devolve-os vazios, e isso marcava `Full`: a óptica e a dobra do relevo da tela inteira a 4K).
/// Oráculo = a mesma ferramenta com o sujo `Full` de antes ([`ph2d_wet_paint::painter::UNDO_SUJO_DOS_PLANOS`]): a
/// tela depois do undo e do redo, 1.º e 2.º traço, é a mesma ao byte; e o sujo estreito é < 1/2 da grade (onde o oráculo é `Full`, a cura vale).
#[test]
fn o_ctrl_z_vivo_suja_a_caixa_e_pinta_o_mesmo_que_a_tela_inteira() {
    let grade = (W * H) as u64;
    let mut curados = 0;
    for nome in [
        "Filbert",
        "Wet Liner",
        "",
        "Wet Round",
        "Knife Oily",
        "Thick Impasto",
    ] {
        let (mut a, mut b) = (tool(nome), tool(nome));
        for (traco_n, y) in [(1, 120.0), (2, 260.0)] {
            for t in [&mut a, &mut b] {
                traco(t, y);
            }
            for refaz in [false, true] {
                let (ca, ta) = volta(&mut a, refaz, true);
                let (cb, tb) = volta(&mut b, refaz, false);
                let caso = format!("{nome:?} traço {traco_n} refaz={refaz}");
                let dif = ta.iter().zip(&tb).filter(|(x, y)| x != y).count();
                assert_eq!(
                    dif, 0,
                    "{caso}: a tela difere do oráculo Full em {dif} bytes"
                );
                assert!(
                    cb < grade / 2,
                    "{caso}: o sujo estreito tem {cb} células (Full = {ca}, grade {grade})"
                );
                eprintln!("{caso}: sujo {cb} células · oráculo {ca}");
                curados += usize::from(ca == grade);
            }
        }
        // O que a óptica guarda da tela (a dobra do relevo, os planos dela) é o que o traço SEGUINTE lê: desfeito o
        // 2.º, um traço por cima da caixa restaurada e para lá dela pinta o mesmo que depois do oráculo.
        let (ca, _) = volta(&mut a, false, true);
        let (cb, _) = volta(&mut b, false, false);
        for t in [&mut a, &mut b] {
            t.on_canvas_pointer(ponto(120.0, 150.0, PointerPhase::Down));
            for k in 1..=60 {
                t.on_canvas_pointer(ponto(
                    120.0 + 6.0 * k as f32,
                    150.0 + 2.0 * k as f32,
                    PointerPhase::Move,
                ));
                t.on_tick(1000.0 / 60.0);
            }
            t.on_canvas_pointer(ponto(480.0, 270.0, PointerPhase::Up));
            quadros(t, 8);
        }
        let dif = a
            .canvas_rgba
            .iter()
            .zip(b.canvas_rgba.iter())
            .filter(|(x, y)| x != y)
            .count();
        assert_eq!(
            dif, 0,
            "{nome:?}: o traço depois do undo (sujo {cb} × {ca}) difere do oráculo em {dif} bytes"
        );
    }
    // Filbert, Wet Liner e Knife Oily, 1.º traço, undo e redo: o oráculo é a tela inteira.
    assert!(
        curados >= 6,
        "só {curados} casos com o oráculo Full: a régua não exerce a cura"
    );
}
