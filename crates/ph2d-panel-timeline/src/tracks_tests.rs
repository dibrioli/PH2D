//! Os gates do [`super`] — o rótulo de um canal e o badge de extrapolação.
//!
//! Filho por `#[path]`, e não módulo irmão: `use super::*` tem de alcançar `track_label` e
//! `extrap_i18n_key`, que são privados. O corte foi feito quando o arquivo cruzou o cap de
//! 600 LOC do painel (592 → 649 com a FASE C.3), e é o corte barato: o que sobra em
//! `tracks.rs` é o que PINTA.

use super::*;
// O  é usado só por estes gates — o pai deixou de o importar
// quando os rótulos saíram para o .
use ph2d_timeline::PropKind;

/// The one door that says "mark it or not": every non-Hold mode has a badge
/// key; Hold is the plain default and gets NONE (no dash, no badge).
#[test]
fn only_a_non_hold_side_is_marked() {
    assert_eq!(extrap_i18n_key(Extrap::Hold), None, "Hold draws nothing");
    for m in [Extrap::Loop, Extrap::PingPong, Extrap::Continue] {
        let key = extrap_i18n_key(m).expect("a non-Hold mode is marked");
        assert!(
            !ph2d_i18n::tr(key).is_empty(),
            "{m:?} resolves to a real badge label"
        );
    }
}

/// **Com nome, o rótulo diz QUEM antes de dizer O QUÊ; sem nome, cai no id curto.**
///
/// As duas metades num gate só de propósito: o fallback é o que impede a cura de virar
/// uma regressão para objetos transientes (que não têm `Name`), e afirmar só a metade
/// bonita deixaria um rótulo `" · Position X"` órfão passar.
///
/// **Mutação que deve sangrar:** `None` devolver `prop_label` puro (aí duas rows de
/// objetos diferentes ficam indistinguíveis).
#[test]
fn the_label_names_the_object_first_and_falls_back_to_the_short_id() {
    let named = track_label(Some("Ball"), 7_294, PropKind::TranslationX);
    assert!(
        named.starts_with("Ball"),
        "o nome vem primeiro — é o que se varre numa coluna estreita: {named:?}"
    );
    assert!(
        named.contains(prop_label(PropKind::TranslationX)),
        "e a propriedade continua lá: {named:?}"
    );
    assert!(
        !named.contains('#'),
        "com nome, o id curto não é ruído a mais na linha: {named:?}"
    );

    let bare = track_label(None, 7_294, PropKind::TranslationX);
    assert!(
        bare.contains("#7294"),
        "sem nome, o rótulo tem de continuar DISTINGUINDO dois objetos: {bare:?}"
    );
    assert_ne!(
        bare,
        track_label(None, 1_294, PropKind::TranslationX),
        "duas rows da mesma propriedade em objetos diferentes não podem ler igual"
    );
}
