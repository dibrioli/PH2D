//! Gates do **light table** (T3.9) — módulo-irmão do `flip_strip_tests` pelo cap de LOC.
//!
//! Foco: o botão **Pin** fixa a chave atual como referência, e a referência CHEGA ao passe
//! de fantasmas mesmo fora do alcance. As duas metades são gates separados de propósito —
//! a primeira prova que o botão guarda o número, a segunda que o artista o VÊ, e um
//! light table pode falhar em qualquer uma delas com a outra verde.

use super::*;
// As fixtures compartilhadas moram no irmão (é lá que a tira inteira é montada); um
// segundo `doc_with_key0` aqui seria uma segunda cena para a mesma pergunta.
use super::tests::{click, doc_with_key0};
use ph2d_flip::{Hold, KeyKind};

/// 🔴 **O botão Pin fixa a chave ATUAL no light table — e clicar de novo a solta.**
///
/// O gesto ponta a ponta pelo caminho real (o `PanelEvent` que o painel empurra). Sem o
/// braço, o botão seria pintado, roteado, e inerte: a lista de fantasmas nunca mudaria e
/// o artista concluiria que o light table "não faz nada".
#[test]
fn the_pin_button_toggles_the_current_key_in_the_light_table() {
    let (mut doc, _oid, lid, mut ph) = doc_with_key0();
    let mut strip = FlipStrip::default();
    assert!(
        strip.pinned_keys().is_empty(),
        "nasce sem referência nenhuma"
    );

    let changed = click(
        ph2d_editor::ids::FLIP_KEY_PIN,
        &mut doc,
        lid,
        &mut ph,
        &mut strip,
    );
    assert_eq!(strip.pinned_keys(), &[0], "a chave 0 virou referência");
    assert!(
        !changed,
        "fixar não é edição de DOCUMENTO — nenhum pixel mudou, e um passo de undo por \
         'quero ver aquele quadro' seria ruído na fila"
    );

    click(
        ph2d_editor::ids::FLIP_KEY_PIN,
        &mut doc,
        lid,
        &mut ph,
        &mut strip,
    );
    assert!(
        strip.pinned_keys().is_empty(),
        "o mesmo botão solta (é um toggle, não dois verbos)"
    );
}

/// 🔴 **Um pin fixado aparece como FANTASMA fora do alcance** — a ponta que fecha o
/// light table (o T3.9 do plano W3).
///
/// Este é o gate que separa "o botão guarda um número" de "o artista vê o quadro": ele
/// pergunta ao MESMO `ph2d_flip::ghosts` que o passe de render consome. Mutação que
/// sangra: o snapshot deixar de passar `pinned_keys()` adiante (a chave 24 some da lista).
#[test]
fn a_pinned_key_reaches_the_ghost_pass_from_outside_the_range() {
    let (mut doc, oid, lid, _ph) = doc_with_key0();
    {
        let obj = doc.object_mut(oid).unwrap();
        obj.insert_frame(lid, 12, Hold::Implicit, KeyKind::Keyframe);
        obj.insert_frame(lid, 24, Hold::Implicit, KeyKind::Keyframe);
    }
    let mut strip = FlipStrip::default();
    strip.toggle_pin(24);

    let obj = doc.object(oid).unwrap();
    let layer = obj.layer(lid).unwrap();
    // No quadro 0, com alcance ±1, a chave 24 está MUITO fora — só o pin a traz.
    let ghosts = ph2d_flip::ghosts(
        layer,
        0,
        &obj.onion,
        strip.selected_keys(),
        strip.pinned_keys(),
    );
    assert!(
        ghosts.iter().any(|g| g.key == 24),
        "a referência fixada tem de chegar ao passe de fantasmas: {ghosts:?}"
    );
    // E sem o pin ela não chega (o controle — senão o gate ficaria verde num mundo onde
    // TUDO vira fantasma).
    let plain = ph2d_flip::ghosts(layer, 0, &obj.onion, strip.selected_keys(), &[]);
    assert!(!plain.iter().any(|g| g.key == 24));
}

/// 🔴 **A folha do Shift & Trace acompanha a chave MOVIDA** — o trace é o 3º estado de
/// sessão chaveado por quadro, e entra pela MESMA porta dos pins e da seleção: mover a
/// célula não pode deixar o deslocamento apontando um quadro sem chave (o fantasma
/// voltaria ao lugar em silêncio, e o artista perderia o posicionamento que acabou de
/// fazer). Mutação que sangra: o `remap_session_after_move` pular o mapa.
#[test]
fn a_moved_key_carries_its_traced_sheet_along() {
    let mut strip = FlipStrip::default();
    let shift = ph2d_flip::Pose::from_translation(ph2d_core::Vec2::new(3.0, -2.0));
    strip.trace.insert(4, shift);
    strip.remap_session_after_move(4, 6);
    assert!(!strip.trace.contains_key(&4), "a folha nao ficou para tras");
    assert_eq!(
        strip.trace.get(&6),
        Some(&shift),
        "o deslocamento viajou com a chave"
    );
}

/// 🔴 **E acompanha o EMPURRÃO da exposição** — esticar a primeira chave empurra a fila;
/// as folhas das chaves empurradas vão junto (e a de quem NÃO foi empurrado fica).
#[test]
fn stretching_a_hold_pushes_the_sheets_that_the_keys_push() {
    let mut strip = FlipStrip::default();
    let a = ph2d_flip::Pose::from_translation(ph2d_core::Vec2::new(1.0, 0.0));
    let b = ph2d_flip::Pose::from_translation(ph2d_core::Vec2::new(0.0, 1.0));
    strip.trace.insert(0, a);
    strip.trace.insert(8, b);
    strip.remap_session_after_hold(0, 2);
    assert_eq!(
        strip.trace.get(&0),
        Some(&a),
        "quem esta antes do empurrao fica"
    );
    assert_eq!(
        strip.trace.get(&10),
        Some(&b),
        "quem foi empurrado leva a folha"
    );
    assert!(!strip.trace.contains_key(&8));
}

/// 🔴 **O Reset Shifts devolve toda folha ao lugar, pelo caminho REAL do botão** — e não
/// é edição de documento (zero passo de undo: exibição, não obra).
#[test]
fn the_reset_button_clears_every_traced_sheet() {
    let (mut doc, _oid, lid, mut ph) = doc_with_key0();
    let mut strip = FlipStrip::default();
    strip.trace.insert(
        0,
        ph2d_flip::Pose::from_translation(ph2d_core::Vec2::new(5.0, 5.0)),
    );
    let changed = click(
        ph2d_editor::ids::FLIP_TRACE_RESET,
        &mut doc,
        lid,
        &mut ph,
        &mut strip,
    );
    assert!(strip.trace.is_empty(), "toda folha voltou ao lugar");
    assert!(!changed, "reset de exibicao nao e passo de undo");
}
