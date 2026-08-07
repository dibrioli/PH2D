//! Os gates da tabela lateral de bindings.

use super::*;

/// **Uma propriedade, UM token.** Bindar de novo substitui — não empilha.
///
/// Duas entradas para o mesmo alvo seriam duas respostas a *"de que cor é este preenchimento?"*, e
/// qual vence dependeria da ordem de inserção.
#[test]
fn a_property_has_exactly_one_token() {
    let mut b = VecBindings::default();
    b.set(BoundProp::Fill, "accent");
    b.set(BoundProp::Fill, "danger");
    assert_eq!(b.entries.len(), 1, "substituiu, nao empilhou");
    assert_eq!(b.get(BoundProp::Fill), Some("danger"));
}

/// As duas propriedades são independentes, e soltar uma não solta a outra.
#[test]
fn the_two_properties_do_not_shadow_each_other() {
    let mut b = VecBindings::default();
    b.set(BoundProp::Fill, "accent");
    b.set(BoundProp::StrokeColor, "border");
    assert_eq!(b.get(BoundProp::Fill), Some("accent"));
    assert_eq!(b.get(BoundProp::StrokeColor), Some("border"));

    b.clear(BoundProp::Fill);
    assert_eq!(b.get(BoundProp::Fill), None);
    assert_eq!(
        b.get(BoundProp::StrokeColor),
        Some("border"),
        "soltar o preenchimento nao pode soltar o traco"
    );
    assert!(!b.is_empty());
    b.clear(BoundProp::StrokeColor);
    assert!(b.is_empty(), "sem entradas, o componente desanexa");
}

/// ⚠️ **Os discriminantes são valores de ARQUIVO.** Este gate não os deriva da função sob teste —
/// ele os afirma como literais, porque um `as u16` sobre o próprio enum seria sempre verde e
/// inserir um variant no meio da lista re-interpretaria todo binding já salvo em silêncio.
#[test]
fn the_wire_discriminants_are_pinned() {
    assert_eq!(BoundProp::Fill as u16, 0);
    assert_eq!(BoundProp::StrokeColor as u16, 1);
    assert_eq!(BoundProp::StrokeWidth as u16, 2);
    assert_eq!(BoundProp::LayoutGapMain as u16, 3);
    assert_eq!(BoundProp::LayoutGapCross as u16, 4);
}

/// O round-trip do save preserva o par — o binding é documento, não estado de sessão.
#[test]
fn a_binding_survives_the_file() {
    let mut b = VecBindings::default();
    b.set(BoundProp::StrokeColor, "accent-hover");
    let bytes = postcard::to_allocvec(&b).expect("serializa");
    let back: VecBindings = postcard::from_bytes(&bytes).expect("desserializa");
    assert_eq!(back, b);
}

/// A lista que o painel OFERECE cobre tudo o que o modelo sabe prender.
///
/// ⚠️ Ela é afirmada por VALOR e não iterando o próprio `ALL` — um gate que percorre a lista sob
/// teste encolhe junto com ela e fica verde sobre a row que sumiu (o oráculo auto-referente que
/// esta linha já pagou uma vez, na multi-resolução do Wet Paint).
#[test]
fn every_bindable_property_is_offered() {
    assert_eq!(
        BoundProp::ALL,
        &[
            BoundProp::Fill,
            BoundProp::StrokeColor,
            BoundProp::StrokeWidth,
            BoundProp::LayoutGapMain,
            BoundProp::LayoutGapCross,
        ]
    );
    for p in BoundProp::ALL {
        assert!(!p.label().is_empty(), "toda propriedade tem rotulo");
    }
}

/// **O código volta ao alvo**, e todo alvo tem código — é o par que liga o clique do picker ao
/// componente.
///
/// ⚠️ O gate percorre `ALL` para a ida e afirma o desconhecido por VALOR para a volta: sem a
/// segunda metade, um `from_code` que devolvesse `Fill` para qualquer número passaria na primeira.
#[test]
fn every_target_round_trips_through_its_wire_code() {
    for &p in BoundProp::ALL {
        assert_eq!(BoundProp::from_code(p as u16), Some(p));
    }
    assert_eq!(
        BoundProp::from_code(BoundProp::ALL.len() as u16),
        None,
        "um codigo que nao existe nao pode virar um alvo"
    );
    assert_eq!(BoundProp::from_code(u16::MAX), None);
}

/// Um alvo de COR e um de COMPRIMENTO na mesma forma não se apagam.
///
/// ⚠️ A fixture mistura as duas famílias de propósito: as entradas são ordenadas por `BoundProp` e
/// os alvos numéricos entraram DEPOIS dos de cor, então é aqui que uma ordenação errada apareceria.
#[test]
fn a_colour_and_a_length_coexist_on_the_same_shape() {
    let mut b = VecBindings::default();
    b.set(BoundProp::StrokeWidth, "stroke.default");
    b.set(BoundProp::Fill, "accent");
    b.set(BoundProp::LayoutGapMain, "spacing.md");
    assert_eq!(b.get(BoundProp::Fill), Some("accent"));
    assert_eq!(b.get(BoundProp::StrokeWidth), Some("stroke.default"));
    assert_eq!(b.get(BoundProp::LayoutGapMain), Some("spacing.md"));
    assert!(
        b.entries.windows(2).all(|w| w[0].0 < w[1].0),
        "as entradas ficam ordenadas pelo alvo"
    );
}
