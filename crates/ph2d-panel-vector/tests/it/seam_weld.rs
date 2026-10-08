//! ⭐⭐⭐ Seam do botão **Weld** (plano 39) — ele está vivo sob o MOUSE, e o clique chega ao bus.
//!
//! # Porque este arquivo nasceu depois do motor
//!
//! Report do Enio (2026-09-01): *"não funcionou ainda o Weld"* — sobre um motor cujos gates de lei
//! estavam todos verdes. ⚠️ **Nenhum deles atravessava o painel**: eles chamam `apply_vec_weld`
//! directamente, e um verbo cujo BOTÃO não fala com ninguém lê-se exactamente como um motor
//! partido. É a lição que a fileira de chips da booleana já custou uma wave (`seam_bool.rs`).
//!
//! O gesto é REAL (Down+Up sobre o rectângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — a forma em que um controlo nasce *pintado, hit-registrado e morto sob o ponteiro*.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, state};

// ⚠️ Alta de propósito: a porta da rolagem (`scroll_area`, spec `04_a_rolagem_unica`) recorta o
// CLIQUE à banda do corpo, logo um controlo abaixo da dobra deixa de estar registado. A 900 px
// estes gates clicavam fantasmas invisíveis; a 4000 o corpo inteiro cabe e o clique é real.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};



/// ⛔ **Com a selecção VAZIA o botão não existe** — a metade da ausência.
///
/// A seção Path é um COMANDO sobre a selecção (`section_scope::WHEN_SELECTED`): sem alvo, ela seria
/// um cabeçalho com botões que só sabem recusar.
#[test]
fn with_nothing_selected_there_is_no_weld_button() {
    state::set_current_selection_count(0);
    let mut host = crate::host_aberto();
    let mut panel_state = VectorPanelState;
    let r = host.painted_rect::<VectorPanel>(
        &mut panel_state,
        VIEWPORT,
        ph2d_panel_vector::ids::VECTOR_PATH_WELD,
    );
    state::set_current_selection_count(1);
    assert!(
        r.is_none(),
        "o Weld subiu com a selecao vazia — um botao que so' sabe recusar"
    );
}
