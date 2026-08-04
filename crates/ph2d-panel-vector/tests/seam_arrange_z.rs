//! Seam do **Z-INDEX GLOBAL** na seção Arrange (Enio, 2026-08-04) — o campo existe quando há
//! resposta, não existe quando não há, e **está vivo sob o mouse**.
//!
//! ⚠️ **A metade do CLIQUE é o que este arquivo ganhou nesta wave.** Enquanto o Z era um readout
//! derivado (um `label_line` sem id), o que dava para medir era a CONSEQUÊNCIA dele — a fileira de
//! botões descer uma linha. Agora ele é o número que o artista ESCREVE, então o gate pode afirmar
//! o que interessa: que o retângulo pintado responde a um ponteiro. Um campo pintado, registrado e
//! **morto sob o mouse** é a falha que este arquivo existe para pegar.

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

/// O topo da fileira de z-order com (ou sem) o Z publicado.
fn to_back_y(z: Option<f32>) -> f32 {
    state::set_z_index(z);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_ARRANGE_TO_BACK)
        .expect("o botao To Back tem de ser pintado sempre")
        .y
}

/// **Publicar o Z acrescenta uma linha** — o campo ocupa espaço, logo existe.
///
/// A mutação que o mata é apagar a linha do `arrange_section`: os dois `y` colapsam, e o número
/// que manda na pilha deixa de ser autorável.
#[test]
fn the_z_field_takes_a_row_of_its_own() {
    let without = to_back_y(None);
    let with = to_back_y(Some(0.0));
    assert!(
        with > without,
        "publicar o Z nao empurrou a fileira de z-order: o campo nao foi pintado \
         ({with} contra {without})"
    );
    state::set_z_index(None);
}

/// **Sem resposta única, sem linha** — a metade da AUSÊNCIA.
///
/// ⚠️ Uma seleção múltipla (ou nenhuma) não tem UM Z; um campo ali escreveria numa forma que o
/// artista não nomeou.
#[test]
fn no_selection_paints_no_z_row() {
    let a = to_back_y(None);
    let b = to_back_y(None);
    assert!(
        (a - b).abs() < f32::EPSILON,
        "o layout da secao Arrange nao e' estavel sem Z publicado"
    );
    // E com o Z publicado ele muda — o controle que torna o gate acima não-vazio.
    assert!(to_back_y(Some(0.0)) > a);
    state::set_z_index(None);
}

/// **O campo está VIVO sob o mouse.**
///
/// ⚠️ Um `painted_rect` prova que o retângulo foi desenhado e registado no índice de hit; ele
/// **não** prova que o widget existe no store. Sem o `register` do `populate` o campo pinta,
/// aparece no índice e não responde a ponteiro nenhum — a forma exata em que um controlo nasce
/// morto. O oráculo é o clique REAL sobre o retângulo que o paint devolveu.
#[test]
fn the_z_field_answers_a_pointer() {
    state::set_z_index(Some(3.0));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_ARRANGE_Z)
        .expect("o campo Z tem de ser pintado com um Z publicado");
    let evs = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        !evs.is_empty(),
        "clicar no campo Z nao produziu evento nenhum — ele esta' morto sob o mouse"
    );
    state::set_z_index(None);
}
