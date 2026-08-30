//! **A fileira *Weight* aparece quando — e só quando — a fonte tem o eixo `wght`.**
//!
//! # As duas fixturas, e por que a segunda é obrigatória
//!
//! - **O DEFEITO:** uma fonte ESTÁTICA. Sem `fvar` o `skrifa` ignora a localização de eixo, logo o
//!   slider é pintado, arrastável e **inerte**. Nenhum gate de registo o vê: ele está registado,
//!   focável, e o handler existe.
//! - **O CONTROLO POSITIVO:** uma fonte variável **só de peso** (`fvar = ['wght']`) — a espécie
//!   mais comum, e há uma na máquina de desenvolvimento (`Cantarell-VF`). ⛔ **Sem esta metade,
//!   *esconder sempre* passaria no teste de cima** — e esconder sempre é precisamente o que a cura
//!   óbvia faria: a regra que esconde a secção AXES é `axes.is_empty()`, e aquela lista **exclui o
//!   `wght` por construção**, então uma variável só-de-peso publica-a vazia com um Weight vivo.
//!
//! ⚠️ **A fixtura aqui é o ESTADO PUBLICADO, não o ficheiro de fonte** — e é isso que torna as
//! duas metades portáteis. Quem lê o `fvar` é a shell (`vec_font::has_weight_axis`), e é lá que a
//! leitura é medida contra fontes reais; aqui mede-se o consumidor.

use crate::state::VectorPanelState;
use crate::{TextAxisSlot, VectorPanel, ids};
use ph2d_editor_core::zones::Rect;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

/// Publica o estado de texto de uma fonte com `has_weight` e `extra_axes` eixos ALÉM do peso.
fn publish_font(has_weight: bool, extra_axes: usize) {
    crate::set_current_text_visible(true);
    crate::set_current_text_has_weight(has_weight);
    crate::set_current_text_axes(
        (0..extra_axes)
            .map(|i| TextAxisSlot {
                name: format!("Axis {i}"),
                min: 0.0,
                max: 100.0,
                value: 50.0,
            })
            .collect(),
    );
}

fn painted(id: ph2d_a11y::NodeId) -> bool {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
        .is_some()
}

/// **O DEFEITO: fonte estática ⇒ nenhuma fileira Weight.**
///
/// Mutação que tem de sangrar: apagar o `if state::text_has_weight_axis()` de
/// `paint_text_sections::text_section`.
#[test]
fn a_static_font_paints_no_weight_row() {
    publish_font(false, 0);
    // ⚠️ Metade JUSTA: sem ela, uma secção TEXT inteiramente morta passaria — e o gate estaria a
    // medir a própria fixtura em vez do produto.
    assert!(
        painted(ids::VECTOR_TEXT_SIZE),
        "a secção TEXT não pintou nem o Size — ela está morta, e este gate não mede nada"
    );
    assert!(
        !painted(ids::VECTOR_TEXT_WEIGHT),
        "a fileira Weight foi pintada para uma fonte ESTÁTICA. Sem `fvar` o skrifa ignora a \
         localização de eixo: o slider arrasta e a letra não muda — um controlo morto que nenhum \
         gate de registo apanha."
    );
}

/// **O CONTROLO POSITIVO: variável só de PESO ⇒ Weight aparece, AXES esconde-se.**
///
/// É a forma exacta da `Cantarell-VF` (`fvar = ['wght']`), e é a fixtura que refuta a cura óbvia:
/// a lista de eixos está VAZIA — a secção AXES some com razão — e o Weight tem de ficar.
///
/// Mutação que tem de sangrar: trocar a condição do pintor por `!axes.is_empty()`, que é a regra
/// da AXES.
#[test]
fn a_weight_only_variable_font_keeps_its_weight_row() {
    publish_font(true, 0);
    assert!(
        painted(ids::VECTOR_TEXT_WEIGHT),
        "a fileira Weight sumiu numa fonte VARIÁVEL só de peso. É a espécie mais comum, e é \
         exactamente o caso em que a regra da secção AXES (`axes.is_empty()`) esconde um controlo \
         vivo e correcto — as duas perguntas não são a mesma."
    );
    assert!(
        !painted(ids::vector_text_axis_id(0)),
        "a secção AXES pintou uma fileira sem eixo extra nenhum publicado"
    );
}

/// **Uma variável com peso E outros eixos mostra as duas coisas** — o caso da InterVariable
/// embutida (`wght` + `opsz`), que é o default do app.
#[test]
fn a_font_with_weight_and_more_shows_both_sections() {
    publish_font(true, 2);
    assert!(painted(ids::VECTOR_TEXT_WEIGHT), "Weight sumiu na embutida");
    assert!(
        painted(ids::vector_text_axis_id(0)) && painted(ids::vector_text_axis_id(1)),
        "os eixos extras não foram pintados"
    );
}
