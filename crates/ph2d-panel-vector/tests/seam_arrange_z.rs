//! Seam do **Z-INDEX** na seção Arrange (Enio, 2026-08-04) — a linha existe quando há resposta, e
//! não existe quando não há.
//!
//! ⚠️ **O oráculo é o LUGAR dos botões, não o texto.** O readout é um `label_line`: sem id e sem
//! hit-rect, por decisão (*um facto que se lê é uma FRASE; um facto sobre que se age é um botão*).
//! Então o que se mede é a CONSEQUÊNCIA dele — a fileira de z-order desce uma linha. Um gate que
//! procurasse a string estaria a afirmar a implementação; este afirma que o espaço foi ocupado.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids, state};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// O topo da fileira de z-order com (ou sem) o readout publicado.
fn to_back_y(z: Option<(u32, u32)>) -> f32 {
    state::set_z_index(z);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_ARRANGE_TO_BACK)
        .expect("o botao To Back tem de ser pintado sempre")
        .y
}

/// **Publicar o Z-index acrescenta uma linha** — o readout ocupa espaço, logo existe.
///
/// A mutação que o mata é apagar a linha do `arrange_section`: os dois `y` colapsam, e o artista
/// fica sem saber onde a forma está na pilha (que é o que os quatro botões movem).
#[test]
fn the_z_readout_takes_a_row_of_its_own() {
    let without = to_back_y(None);
    let with = to_back_y(Some((1, 3)));
    assert!(
        with > without,
        "publicar o Z-index nao empurrou a fileira de z-order: o readout nao foi pintado \
         ({with} contra {without})"
    );
    state::set_z_index(None);
}

/// **Sem resposta única, sem linha** — a metade da AUSÊNCIA.
///
/// ⚠️ Uma seleção múltipla (ou nenhuma) não tem UM lugar na pilha; pintar um número ali seria
/// afirmar sobre uma das formas e calar as outras.
#[test]
fn no_selection_paints_no_z_row() {
    let a = to_back_y(None);
    let b = to_back_y(None);
    assert!(
        (a - b).abs() < f32::EPSILON,
        "o layout da secao Arrange nao e' estavel sem Z publicado"
    );
    // E com o Z publicado ele muda — o controle que torna o gate acima não-vazio.
    assert!(to_back_y(Some((0, 2))) > a);
    state::set_z_index(None);
}
