//! ⭐ **O dente enche aos poucos** (dono 09/11, o Pastel: «enche o papel paulatinamente em várias repassadas»): no Height
//! com `Tooth fill`, o papel que um traço vê sobe pelo pó que a camada já tinha no pen-down
//! ([`ph2d_painter_brush::GrainLaw::papel_com_po`]). Pelo ponteiro, no Digital, contra o mesmo pincel sem enchimento.

use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{LinhaDaPonta, PaintMedia, PainterTool};

const W: usize = 400;
const H: usize = 120;
const PAPEL_RUGOSO: u8 = 27; // TextureKind::PaperRough
const TILED: u8 = 1;
const HEIGHT: f64 = 4.0; // GrainMode::Height

/// Lápis preto raio 16, Height com maciez, o grão preso ao papel; `fill` = o enchimento do dente.
fn lapis(fill: f64) -> PainterTool {
    let mut t = PainterTool::default();
    // A camada VAZIA (o desenho novo do app: a camada transparente sobre o papel) — uma camada opaca é tinta, e encheria
    // o dente já no 1.º traço.
    t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(16.0);
    t.set_brush_falloff(8);
    t.set_brush_color_srgb8([0, 0, 0]);
    t.set_brush_texture_kind(PAPEL_RUGOSO);
    t.set_brush_texture_mapping(TILED);
    t.set_brush_texture_size(0, 6.0);
    t.set_brush_texture_size(1, 6.0);
    t.set_ponta(LinhaDaPonta::GrainMode, HEIGHT);
    t.set_ponta(LinhaDaPonta::Softness, 0.3);
    // A ponta afunda pela pressão (o K1): a meia pressão só os picos do papel recebem tinta.
    t.set_ponta(LinhaDaPonta::DepthJitter, 1.0);
    t.set_ponta(LinhaDaPonta::DepthPressure, 1.0);
    t.set_ponta(LinhaDaPonta::Fill, fill);
    t
}

/// Um traço reto a pressão `p` (x 40 → 360, y 60).
fn traco(t: &mut PainterTool, p: f32) {
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 60.0],
        pressure: p,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(40.0, PointerPhase::Down));
    let mut x = 40.0;
    while x < 360.0 {
        x += 3.0;
        t.on_canvas_pointer(cp(x, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(x, PointerPhase::Up));
}

/// A tinta média do miolo (x 80..320, y 56..64) pelo alfa da camada: `0` o papel, `1` a tinta cheia.
fn cobre(px: &[u8]) -> f64 {
    let pts: Vec<_> = (80..320)
        .flat_map(|x| (56..64).map(move |y| (x, y)))
        .collect();
    let tinta: f64 = pts
        .iter()
        .map(|&(x, y)| f64::from(px[(y * W + x) * 4 + 3]) / 255.0)
        .sum();
    tinta / pts.len() as f64
}

/// ⭐ A 1.ª passada é a de sempre AO BIT (a camada vazia não tem pó); a 2.ª cobre mais com o dente a encher; e a mesma
/// entrada dá o mesmo bit (o pó é o do pen-down, imutável entre bandas).
#[test]
fn a_segunda_passada_enche_o_dente_e_a_primeira_e_a_de_sempre() {
    let (mut a, mut b) = (lapis(0.0), lapis(0.6));
    traco(&mut a, 0.5);
    traco(&mut b, 0.5);
    let (a1, b1) = (
        a.imagem_aplicada().expect("a imagem").0,
        b.imagem_aplicada().expect("a imagem").0,
    );
    assert_eq!(a1, b1, "a 1.ª passada mudou com o enchimento");
    traco(&mut a, 0.5);
    traco(&mut b, 0.5);
    let (a2, b2) = (
        a.imagem_aplicada().expect("a imagem").0,
        b.imagem_aplicada().expect("a imagem").0,
    );
    let (c1, ca, cb) = (cobre(&a1), cobre(&a2), cobre(&b2));
    assert!(
        c1 < 0.9,
        "a 1.ª passada já cobre tudo ({c1:.3}): a régua não vê o dente"
    );
    assert!(
        cb > ca + 0.05,
        "o dente não encheu: 1.ª {c1:.3} · 2.ª sem {ca:.3} · 2.ª com {cb:.3}"
    );
    let mut c = lapis(0.6);
    traco(&mut c, 0.5);
    traco(&mut c, 0.5);
    assert_eq!(
        c.imagem_aplicada().expect("a imagem").0,
        b2,
        "a mesma entrada deu outro bit"
    );
}

/// ⭐ O pó é a tinta que a camada TEM: desfeitos os dois traços, o seguinte volta a ser a 1.ª passada (sem plano à parte
/// que o desfazer esquecesse, e sem o pó do pen-down anterior).
#[test]
fn desfeito_o_traco_o_po_vai_com_ele() {
    let mut a = lapis(0.6);
    traco(&mut a, 0.5);
    let so_um = a.imagem_aplicada().expect("a imagem").0;
    traco(&mut a, 0.5);
    assert_ne!(a.imagem_aplicada().expect("a imagem").0, so_um);
    assert!(a.undo_last() && a.undo_last(), "os dois traços desfazem-se");
    traco(&mut a, 0.5);
    assert_eq!(a.imagem_aplicada().expect("a imagem").0, so_um);
}
