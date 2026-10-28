//! **U1 da auditoria W3b**: o Glaze layering (K-M) é do TRAÇO (pousa na célula como `vidrado`), logo viaja com o
//! preset e com a ferramenta, como os knobs. Doc: `docs/Painter/49_realshader/AUDITORIA_W3b.md`.

use super::tests::pelo_ficheiro;
use super::*;

fn com_glaze(on: bool) -> Preset {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    t.paint.wetpaint.km_glaze = on;
    let nome = if on { "A" } else { "B" };
    let (prm, texturas) = t.wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, "Teste", nome));
    pelo_ficheiro(prm, texturas)
}

/// Gravar A com o Glaze ligado → aplicar B desligado → reaplicar A: volta ligado (pelo ficheiro, ida e volta).
#[test]
fn o_glaze_viaja_com_o_preset() {
    let (a, b) = (com_glaze(true), com_glaze(false));
    assert_eq!(a.parametros.engine.get(CHAVE_GLAZE), Some(&1.0));
    assert!(
        !b.parametros.engine.contains_key(CHAVE_GLAZE),
        "desligado não se grava: o preset sem ele é o de antes, ao byte"
    );
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    t.wet_preset_aplicar(&a);
    assert!(t.paint.wetpaint.km_glaze, "A liga");
    t.wet_preset_aplicar(&b);
    assert!(!t.paint.wetpaint.km_glaze, "B desliga");
    t.wet_preset_aplicar(&a);
    assert!(t.paint.wetpaint.km_glaze, "A de novo liga");
}

/// Um preset sem a chave (todo preset de antes dela, e toda a fábrica) = o de fábrica: desligado — mesmo com o Glaze
/// ligado na mão (um preset é o pincel inteiro, a regra dos knobs).
#[test]
fn preset_sem_o_glaze_e_o_de_fabrica() {
    let antigo = com_glaze(false);
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    t.paint.wetpaint.km_glaze = true;
    t.wet_preset_aplicar(&antigo);
    assert!(!t.paint.wetpaint.km_glaze);
}

/// Cada ferramenta tem o seu Glaze (`tool_link`): ligado no Pincel, a Máscara não o herda, e voltar ao Pincel devolve-o.
/// Com «Sync with other tools», todos partilham.
#[test]
fn o_glaze_e_por_ferramenta() {
    let mut t = PainterTool::default();
    t.set_paint_tool_mode("wetpaint");
    t.paint.wetpaint.km_glaze = true;
    t.set_paint_tool_mode("mask");
    assert!(!t.paint.wetpaint.km_glaze, "a outra ferramenta tem o seu");
    t.set_paint_tool_mode("wetpaint");
    assert!(t.paint.wetpaint.km_glaze, "volta o do pincel");
    t.toggle_link_shared_settings(); // liga: partilhado
    t.set_paint_tool_mode("mask");
    assert!(t.paint.wetpaint.km_glaze, "ligadas, partilham");
    t.paint.wetpaint.km_glaze = false;
    t.toggle_link_shared_settings(); // desliga: cada uma fica com o de agora
    t.set_paint_tool_mode("wetpaint");
    assert!(
        !t.paint.wetpaint.km_glaze,
        "desligar o Sync dá a todas o de agora"
    );
}
