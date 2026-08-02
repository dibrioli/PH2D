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
    assert_eq!(BoundProp::ALL, &[BoundProp::Fill, BoundProp::StrokeColor]);
    for p in BoundProp::ALL {
        assert!(!p.label().is_empty(), "toda propriedade tem rotulo");
    }
}
