//! **O que fica para trás numa troca** (missão 4K, B02.3, B02.4 e B02.A1 do
//! [BACKLOG](../../../../../../../docs/Painter/missao_4k/BACKLOG.md)): o conta-gotas dentro do Wet, o Dual e o olho da
//! máscara de um documento que sai.

use super::super::agua_sobre_oleo_tests::{H, W, com_oleo};
use super::*;

const A: u64 = 1;
const B: u64 = 2;

/// ⭐ T1-33 (B02.3): o conta-gotas no Wet é momentâneo — a água continua a correr e o slot do pincel fica.
#[test]
fn o_conta_gotas_no_wet_nao_adormece_a_agua() {
    let mut t = com_oleo();
    let (modo, slot) = (t.paint.paint_mode, t.slot_atual());
    t.set_paint_tool_mode("eyedropper");
    assert!(t.eyedropper_armed(), "o conta-gotas não armou");
    assert!(
        t.paint.wetpaint.session.is_some(),
        "o conta-gotas adormeceu a água"
    );
    assert_eq!(t.slot_atual(), slot, "o conta-gotas trocou o slot");
    assert!(t.paint.paint_mode == modo, "o conta-gotas saiu do Wet");
    t.ticks_em_blocos(10);
    t.set_paint_tool_mode("brush");
    assert!(t.paint.wetpaint.session.is_some(), "a água parou na volta");
    assert_eq!(t.slot_atual(), slot, "a volta trocou o slot");
}

/// B02.3 (O2-16): acordar com uma sessão viva recusa — e a tinta adormecida fica na lista.
#[test]
fn a_recusa_de_acordar_nao_perde_a_adormecida() {
    let mut t = com_oleo();
    assert!(t.wet_adormece(), "fixture: a tinta adormece");
    assert_eq!(t.paint.wetpaint.adormecidas.len(), 1, "fixture");
    t.paint.wetpaint.session = com_oleo().paint.wetpaint.session.take().into();
    assert!(!t.wet_acorda(), "com uma sessão viva, acordar recusa");
    assert_eq!(
        t.paint.wetpaint.adormecidas.len(),
        1,
        "a recusa levou a tinta adormecida"
    );
}

/// B02.4 (DF-10): o traço do Dual (máscara do tamanho da tela, marcas pendentes) é do documento que sai.
#[test]
fn o_rebind_larga_o_traco_do_dual() {
    let mut t = PainterTool::default();
    t.bind_document(A, vec![255u8; W * H * 4], W as u32, H as u32);
    t.paint.dual.poe_traco_de_sonda(W * H * 4);
    assert!(t.paint.dual.tem_traco(), "fixture: o traço do Dual");
    t.bind_document(B, vec![255u8; W * H * 4], W as u32, H as u32);
    assert!(
        !t.paint.dual.tem_traco(),
        "o traço do Dual de A atravessou para B"
    );
}

/// O documento `id` com uma máscara na camada activa: a mesma construção dá o mesmo id de camada.
fn com_mascara(t: &mut PainterTool, id: u64) -> RtLayerId {
    t.bind_document(id, vec![255u8; W * H * 4], W as u32, H as u32);
    t.add_mask_to_active().expect("a máscara")
}

/// ⭐ B02.A1 (AU-01-2): o olho de cinza da máscara é do documento — os ids colidem, o de A não acende o cinza em B.
#[test]
fn o_olho_da_mascara_nao_atravessa_documentos() {
    let mut u = PainterTool::default();
    com_mascara(&mut u, B);
    let doc_b = u.tira_documento(B).expect("o documento B");
    let mut t = PainterTool::default();
    let m = com_mascara(&mut t, A);
    t.install_document(B, doc_b);
    t.toggle_mask_view_grayscale(m);
    assert_eq!(t.mask_view_grayscale(), Some(m.0), "fixture: o olho aberto");
    t.bind_document(B, vec![255u8; W * H * 4], W as u32, H as u32);
    assert_eq!(
        t.mask_view_grayscale(),
        None,
        "o olho da máscara de A acendeu o cinza em B"
    );
    t.bind_document(A, vec![255u8; W * H * 4], W as u32, H as u32);
    assert_eq!(
        t.mask_view_grayscale(),
        Some(m.0),
        "o olho de A não voltou com ele"
    );
}

/// ⭐ B02.2 (IN-8): a tinta por assar e o pedido de Apply são do documento. A → B → A: B não herda nenhum dos dois, A
/// volta com os dois — e a troca seguinte não o deita fora como se a sprite o reconstruísse.
#[test]
fn a_tinta_por_assar_viaja_com_o_documento() {
    use ph2d_editor_core::tool::RasterEditTool;
    let branco = || vec![255u8; W * H * 4];
    let mut t = PainterTool::default();
    t.bind_document(A, branco(), W as u32, H as u32);
    t.set_paint_media(crate::tool::paint::PaintMedia::Digital);
    super::super::adormecida_tests::traco_de_outro_meio(&mut t, (40.0, 40.0), (160.0, 100.0));
    let pintado = t.canvas_rgba.as_ref().clone();
    assert!(
        t.has_unbaked_edits() && pintado != branco(),
        "fixture: o traço por assar em A"
    );
    t.request_commit();
    t.bind_document(B, branco(), W as u32, H as u32);
    assert!(!t.has_unbaked_edits(), "B herdou a tinta por assar de A");
    assert!(
        !t.take_pending_commit(),
        "o Apply pedido em A disparou em B"
    );
    t.bind_document(A, branco(), W as u32, H as u32);
    assert!(t.has_unbaked_edits(), "A voltou sem a tinta por assar");
    assert!(t.take_pending_commit(), "o Apply pedido em A perdeu-se");
    t.bind_document(B, branco(), W as u32, H as u32);
    t.bind_document(A, branco(), W as u32, H as u32);
    assert!(
        t.canvas_rgba.as_ref() == &pintado,
        "o traço por assar sumiu de A na segunda troca"
    );
}

/// B02.2: o assado adiado (sair do Painter com tinta por assar) é do documento que saiu, não do que entra.
#[test]
fn o_assado_adiado_nao_atravessa_documentos() {
    let mut t = PainterTool::default();
    t.bind_document(A, vec![255u8; W * H * 4], W as u32, H as u32);
    (t.edited_since_bind, t.deferred_bake) = (true, true); // o `on_deactivate` com tinta por assar
    t.bind_document(B, vec![255u8; W * H * 4], W as u32, H as u32);
    assert!(
        !t.take_deferred_bake(),
        "o assado adiado de A disparou em B"
    );
    t.bind_document(A, vec![255u8; W * H * 4], W as u32, H as u32);
    assert!(t.take_deferred_bake(), "o assado adiado de A perdeu-se");
}
