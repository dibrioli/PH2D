//! ⭐ **A Strength é o tecto, o Flow é a taxa** (dono 24/10: *«Glaze Round, com o mouse, Strength máximo, ainda
//! é transparente e não tem como configurar isso»*). A lei do Photoshop/Krita, pelo RATO, no Digital.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: usize = 256;
const H: usize = 128;
const Y: f32 = 64.0;
const PX: usize = 128;

fn passada(t: &mut PainterTool) {
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, Y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(30.0, PointerPhase::Down));
    let mut x = 30.0;
    while x < 226.0 {
        x += 3.0;
        t.on_canvas_pointer(cp(x, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(226.0, PointerPhase::Up));
}

/// Escurecimento do preto sobre branco no centro do traço (`0..255`).
fn centro(t: &PainterTool) -> u8 {
    255 - t.canvas_rgba[(Y as usize * W + PX) * 4]
}

fn tela() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source([255u8; 4].repeat(W * H), W as u32, H as u32);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([0, 0, 0]);
    t
}

#[test]
fn o_glaze_round_com_flow_e_strength_no_maximo_pinta_opaco_com_o_rato() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    let p = &bib
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Glaze Round")
        .expect("Glaze Round de fábrica")
        .preset;
    let monta = || {
        let mut t = tela();
        t.preset_aplicar(p).expect("meio");
        t.set_brush_size_px(40.0);
        t.set_brush_color_srgb8([0, 0, 0]);
        t.set_brush_strength(1.0);
        t
    };
    // O CONTROLO — o que o dono viu: de fábrica (Flow 0,03) a Strength 1 numa passada ainda é transparente.
    let mut f = monta();
    passada(&mut f);
    let de_fabrica = centro(&f);
    assert!(
        de_fabrica < 200,
        "de fábrica o Glaze Round é translúcido: {de_fabrica}/255"
    );
    let mut t = monta();
    t.set_brush_flow(1.0);
    assert!(
        (t.brush_settings().flow - 1.0).abs() < 1e-6,
        "o painel lê o Flow que o tool tem"
    );
    passada(&mut t);
    let c = centro(&t);
    assert!(
        c >= 250,
        "Strength 1 · Flow 1 com o rato = opaco no centro, got {c}/255"
    );
}

/// Um traço SÓ (Down … Up) que esfrega `idas` vezes a mesma linha: o tecto do Accumulate off é POR TRAÇO (dois
/// traços empilham, como no Photoshop), por isso é dentro de um traço que a taxa acumula até a Strength.
fn esfrega(t: &mut PainterTool, idas: usize) -> Vec<u8> {
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, Y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let mut lidos = Vec::new();
    t.on_canvas_pointer(cp(30.0, PointerPhase::Down));
    for k in 0..idas {
        let (a, b) = if k % 2 == 0 {
            (30.0, 226.0)
        } else {
            (226.0, 30.0)
        };
        let n = 66;
        for j in 1..=n {
            let x = a + (b - a) * j as f32 / n as f32;
            t.on_canvas_pointer(cp(x, PointerPhase::Move));
        }
        lidos.push(centro(t));
    }
    t.on_canvas_pointer(cp(30.0, PointerPhase::Up));
    lidos
}

#[test]
fn com_accumulate_off_o_flow_acumula_ate_a_strength_e_nunca_passa() {
    let mut t = tela();
    t.paint.brush.accumulate = false;
    t.set_brush_strength(0.6);
    t.set_brush_flow(0.1);
    let idas = esfrega(&mut t, 40);
    let tecto = (0.6_f32 * 255.0).round() as u8; // 153
    let (uma, fim) = (idas[0], *idas.last().unwrap());
    assert!(
        uma + 20 < tecto,
        "uma ida de Flow 0,1 fica bem abaixo do tecto: {uma} vs {tecto} ({idas:?})"
    );
    // O degrau de 8 bits: um dab só move o texel se `flow·(tecto − m)·255 ≥ ½`, logo o patamar fica a
    // `½/flow` níveis do tecto (Flow 0,1 → 5; medido 148/153). É o recurso «tela RGBA8», não a lei.
    let degrau = (0.5 / 0.1_f32).ceil() as u8 + 1;
    assert!(
        fim + degrau >= tecto,
        "esfregar converge à Strength (0,6 → {tecto}), não a Strength×Flow: {idas:?}"
    );
    assert!(
        idas.iter().all(|&v| v <= tecto + 1),
        "e nunca passa do tecto {tecto}: {idas:?}"
    );
}
