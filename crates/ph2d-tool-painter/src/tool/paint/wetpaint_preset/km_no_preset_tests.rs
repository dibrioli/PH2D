//! O **Pigment mixing (K-M)** é do PINCEL (dono 30/10, J11 item 1): ligado de fábrica, viaja com o preset; o
//! ficheiro só o grava DESLIGADO (ausente = ligado — a fábrica e todo preset de antes ficam ao byte).

use super::tests::pelo_ficheiro;
use super::*;

fn com_km(on: bool) -> Preset {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    t.paint.wetpaint.km_mixing = on;
    let (prm, texturas) = t.wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, "Teste", "K"));
    pelo_ficheiro(prm, texturas)
}

/// A ferramenta de fábrica traz o K-M ligado, e um preset sem a chave liga-o mesmo com a mão desligada.
#[test]
fn o_km_de_fabrica_e_ligado_e_preset_sem_chave_liga() {
    let t = PainterTool::default();
    assert!(t.paint.wetpaint.km_mixing, "de fábrica: ligado");
    let ligado = com_km(true);
    assert!(
        !ligado.parametros.engine.contains_key(CHAVE_KM),
        "ligado não se grava: o preset sem a chave é o de antes, ao byte"
    );
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    t.paint.wetpaint.km_mixing = false;
    t.wet_preset_aplicar(&ligado);
    assert!(t.paint.wetpaint.km_mixing, "preset sem a chave ⇒ ligado");
}

/// Gravado desligado volta desligado (pelo ficheiro); outro preset sem a chave volta a ligar.
#[test]
fn o_km_desligado_viaja_com_o_preset() {
    let (off, on) = (com_km(false), com_km(true));
    assert_eq!(off.parametros.engine.get(CHAVE_KM), Some(&0.0));
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    t.wet_preset_aplicar(&off);
    assert!(!t.paint.wetpaint.km_mixing, "o desligado volta desligado");
    t.wet_preset_aplicar(&on);
    assert!(t.paint.wetpaint.km_mixing, "o outro liga");
}

/// O Reset do pincel volta a ligar.
#[test]
fn o_reset_do_pincel_liga_o_km() {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    t.paint.wetpaint.km_mixing = false;
    t.reset_brush_wetpaint();
    assert!(t.paint.wetpaint.km_mixing);
}
