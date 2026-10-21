//! ⭐ **A linha de escolha tem o losango nos DOIS feitios** (plano 35 §5) — ao lado do nome (cabe
//! numa fileira) e em paleta (o nome por cima). Nascido de uma mutação sobrevivente (F24c): o seam do
//! painel do vetor caía só na paleta, e o ramo «ao lado» ficava sem observador.

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::property_row::{Seccao, escolha, paint_choice_row};
use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_id, key_mark_target, set_key_marks};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const A: NodeId = NodeId(0x51_0001);
const B: NodeId = NodeId(0x51_0002);

/// Pinta a escolha `A | B` na largura `w`; devolve (ao lado?, o alvo do losango registado).
fn pinta(w: f32) -> (bool, Option<NodeId>) {
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::without_system_fonts();
    let mut hit = HitIndex::new();
    let store = WidgetStore::default();
    let ((), pintadas) = escolha::medindo(|| {
        let _ = paint_choice_row(
            &mut scene,
            &mut ts,
            Theme::Dark,
            &mut hit,
            &store,
            0.0,
            w,
            0.0,
            "Cap",
            &[("Butt", true, A), ("Round", false, B)],
            Seccao::apenas_campos(1),
        );
    });
    let alvo = key_mark_id(A);
    (
        pintadas
            .first()
            .expect("a escolha passou pela porta")
            .ao_lado,
        hit.rect_for(alvo).map(|_| alvo),
    )
}

/// (Mutação: o `decorador` de qualquer um dos dois ramos de volta ao ponto ⇒ o alvo some.)
#[test]
fn a_escolha_tem_o_losango_ao_lado_e_em_paleta() {
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    set_key_marks(BTreeMap::from([(A, KeyMark::Chave)]));
    let (ao_lado, alvo) = pinta(600.0);
    assert!(
        ao_lado,
        "600 px cabe numa fileira — o feitio medido tem de ser «ao lado»"
    );
    assert_eq!(
        alvo,
        Some(key_mark_id(A)),
        "ao lado: o losango não foi registado"
    );
    assert_eq!(
        key_mark_target(key_mark_id(A)).map(|c| c.fields),
        Some(vec![A, B])
    );
    let (ao_lado, alvo) = pinta(40.0);
    assert!(
        !ao_lado,
        "40 px não cabe — o feitio medido tem de ser a paleta"
    );
    assert_eq!(
        alvo,
        Some(key_mark_id(A)),
        "paleta: o losango não foi registado"
    );
    set_key_marks(BTreeMap::new());
    let (_, alvo) = pinta(600.0);
    assert_eq!(alvo, None, "sem canal, a linha não tem alvo de losango");
}
