//! **Nenhuma fileira de eixo é pintada além do espaço de slots que alguém regista.**
//!
//! ## O defeito, medido em 2026-08-30
//!
//! A secção *AXES* do painel de Texto desenha um campo numérico por eixo de variação que a fonte
//! corrente publica (fora o `wght`, que tem slider próprio). Ela desenhava **sem tecto**; e o
//! registo (`populate`), o mapa `id → índice` (`state::text_axis_index`) e a publicação da shell
//! param todos em `ids::MAX_TEXT_VARIATION_AXES`.
//!
//! ⇒ do slot `MAX+1` em diante o campo saía com o **nome real do eixo** ao lado e o valor `0`:
//! ninguém o regista, ninguém lê o que se digita nele. *Um campo com o nome certo e nenhum leitor
//! é a pior forma deste defeito — ele convence.* Alcançável com a Roboto Flex, que publica ~12
//! eixos além do `wght`; o tecto era `6`.
//!
//! ## Por que o gate não mede o NÚMERO
//!
//! Subir o tecto sozinho só muda onde o defeito começa. O que este ficheiro defende é a
//! **igualdade das duas lentes** — o pintor nunca pinta um slot que o espaço de registo não
//! alcança —, e isso vale para qualquer valor de `MAX_TEXT_VARIATION_AXES`, incluindo o próximo
//! que alguém escolher.
//!
//! ⚠️ A fixtura publica **mais eixos que o tecto**, de propósito: uma que publicasse menos não
//! conteria o fenómeno e ficaria verde sobre o produto partido.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{TextAxisSlot, VectorPanel, ids};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

/// Quantos eixos a fixtura publica — o tecto MAIS folga, para que o excedente exista.
const PUBLISHED: usize = ids::MAX_TEXT_VARIATION_AXES + 6;

fn publish_axes(n: usize) {
    ph2d_panel_vector::set_current_text_visible(true);
    ph2d_panel_vector::set_current_text_axes(
        (0..n)
            .map(|i| TextAxisSlot {
                name: format!("Axis {i}"),
                min: 0.0,
                max: 100.0,
                value: 50.0,
            })
            .collect(),
    );
}

/// **O pintor pára no tecto, mesmo com uma fonte que publica mais.**
///
/// Mutação que tem de sangrar: tirar o `.take(ids::MAX_TEXT_VARIATION_AXES)` do laço em
/// `paint_text_sections::axes_section` — aí os slots excedentes voltam a ser pintados.
#[test]
fn the_panel_never_paints_an_axis_row_nobody_registers() {
    publish_axes(PUBLISHED);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;

    let mut painted = Vec::new();
    for i in 0..PUBLISHED {
        if host
            .painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::vector_text_axis_id(i))
            .is_some()
        {
            painted.push(i);
        }
    }

    // ⚠️ Metade JUSTA: sem ela, um painel que não pinta eixo NENHUM passaria — e a secção estaria
    // morta em vez de excessiva. O gate tem de ver o produto a funcionar antes de o limitar.
    assert!(
        !painted.is_empty(),
        "nenhuma fileira de eixo foi pintada com {PUBLISHED} eixos publicados — a secção AXES \
         está morta, e este gate estaria a medir a sua própria fixtura"
    );

    let over: Vec<usize> = painted
        .iter()
        .copied()
        .filter(|&i| i >= ids::MAX_TEXT_VARIATION_AXES)
        .collect();
    assert!(
        over.is_empty(),
        "o painel pintou o(s) slot(s) de eixo {over:?}, além do tecto \
         MAX_TEXT_VARIATION_AXES={}. Esses campos saem com o NOME REAL do eixo e o valor 0: \
         ninguém os regista e ninguém lê o que se digita neles.\n\
         \n\
         A cura não é subir o tecto — é o pintor consultá-lo. As duas lentes (o que se pinta e o \
         que se regista) têm de ser a mesma.",
        ids::MAX_TEXT_VARIATION_AXES
    );
}

/// **Controle: com poucos eixos, pinta-se exactamente o que a fonte publica.**
///
/// O `take` não pode ter virado um `truncate` fixo: uma fonte com dois eixos mostra dois campos,
/// não `MAX`. Sem este controle, `take(0)` passaria no teste de cima.
#[test]
fn a_font_with_two_axes_gets_exactly_two_rows() {
    publish_axes(2);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;

    for i in 0..2 {
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::vector_text_axis_id(i))
                .is_some(),
            "o eixo {i} de uma fonte com dois eixos não foi pintado"
        );
    }
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::vector_text_axis_id(2))
            .is_none(),
        "foi pintada uma terceira fileira para uma fonte que publica dois eixos"
    );
}
