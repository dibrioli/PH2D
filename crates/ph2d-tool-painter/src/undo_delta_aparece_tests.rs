//! O relevo que NASCE no 1.º traço Impasto guarda só a janela (F1p, B44.8): ao bit contra o caminho que
//! segura o plano inteiro (o oráculo, [`super::set_relevo_aparece`]`(false)`), em cada passo de undo/redo.

use crate::tool::PainterTool;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: u32 = 640;
const H: u32 = 360;

type Estado = (
    Vec<u8>,
    Vec<(u64, Vec<u32>)>,
    Vec<(u64, Vec<u8>)>,
    Vec<(u64, Vec<u8>)>,
);

fn estado(t: &PainterTool) -> Estado {
    let m = t.snapshot_model();
    let id = |k: &crate::layers::LayerId| k.0;
    (
        m.canvas_rgba.to_vec(),
        m.heights
            .iter()
            .map(|(k, p)| (id(k), p.iter().map(|v| v.to_bits()).collect()))
            .collect(),
        m.covers.iter().map(|(k, p)| (id(k), p.to_vec())).collect(),
        m.mats
            .iter()
            .map(|(k, p)| (id(k), p.as_flattened().to_vec()))
            .collect(),
    )
}

fn traco(t: &mut PainterTool, y: f32) {
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(120.0, PointerPhase::Down));
    for k in 1..=20 {
        t.on_canvas_pointer(cp(120.0 + 12.0 * k as f32, PointerPhase::Move));
        if k % 4 == 0 {
            t.on_tick(1000.0 / 60.0);
        }
    }
    t.on_canvas_pointer(cp(380.0, PointerPhase::Up));
    for _ in 0..4 {
        t.on_tick(1000.0 / 60.0);
    }
}

/// Os estados de cada passo (1.º traço · 2.º · undo · undo · redo · redo) e os bytes do undo do 1.º traço.
fn corrida(aparece: bool) -> (Vec<Estado>, usize) {
    super::set_relevo_aparece(aparece);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(crate::PaintMedia::Impasto);
    t.set_wet_relogio_fixo(true);
    t.sonda_raio(30.0);
    t.set_brush_color_srgb8([20, 60, 200]);
    let u0 = t.undo_retained_bytes();
    let mut es = Vec::new();
    traco(&mut t, 150.0);
    let bytes = t.undo_retained_bytes() - u0;
    es.push(estado(&t));
    traco(&mut t, 200.0);
    es.push(estado(&t));
    for passo in [PainterTool::undo_last, PainterTool::undo_last] {
        assert!(passo(&mut t));
        es.push(estado(&t));
    }
    for passo in [PainterTool::redo_last, PainterTool::redo_last] {
        assert!(passo(&mut t));
        es.push(estado(&t));
    }
    super::set_relevo_aparece(true);
    (es, bytes)
}

#[test]
fn o_relevo_que_nasce_guarda_so_a_janela_e_desfaz_ao_bit() {
    let (velho, bytes_velho) = corrida(false);
    let (novo, bytes_novo) = corrida(true);
    assert!(
        !velho[0].1.is_empty() && velho[0].1 != velho[3].1,
        "o 1.º traço não fez nascer relevo — régua morta"
    );
    for (i, (a, b)) in velho.iter().zip(&novo).enumerate() {
        assert!(a == b, "passo {i}: o relevo que nasce diverge do oráculo");
    }
    assert!(novo[3].1.is_empty(), "desfazer o nascimento tira os planos");
    assert!(novo[5] == novo[1], "refazer tudo devolve o 2.º traço");
    let tela = (W * H) as usize;
    assert!(
        bytes_velho >= tela * 12,
        "o oráculo segura o plano inteiro ({bytes_velho} < {})",
        tela * 12
    );
    // O oráculo = a janela da tela + os três planos INTEIROS (12 B/px); a cura = a janela da tela + as três
    // janelas, que o 1.º traço cobre em menos de metade da tela.
    assert!(
        bytes_novo + tela * 6 < bytes_velho,
        "o 1.º traço segura {bytes_novo} B de undo (oráculo {bytes_velho}, tela {tela} px) — o nascimento \
         voltou a pagar a tela"
    );
}
