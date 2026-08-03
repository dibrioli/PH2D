//! Gates do modelo de componentes (plano UI/UX W5) — a metade que não precisa de cena.
//!
//! O que se prova aqui é o **invariante que o undo consome**: a lista de overrides é canônica, e
//! escrever pela porta a mantém assim. O resto (mestre propaga, override sobrevive) mora no
//! produtor, porque só lá existe geometria contra a qual medir.

use super::*;

/// **Duas instâncias logicamente iguais guardam os MESMOS bytes.**
///
/// ⚠️ É o gate que justifica a porta existir. O `canonicalize` do undo global ordena as entidades
/// pelos **BYTES dos componentes**, então uma lista de overrides em ordem de chegada faria duas
/// instâncias iguais compararem diferente — e o `post_frame_undo` gravaria um passo espúrio por
/// frame, que é exatamente o defeito que o `canonicalize` existe para matar.
#[test]
fn the_same_overrides_in_any_order_are_the_same_bytes() {
    let mut a = VecInstance::new(7);
    a.set(30, OverrideSlot::Hidden);
    a.set(10, OverrideSlot::Fill([1, 2, 3, 4]));
    a.set(20, OverrideSlot::Fill([9, 9, 9, 9]));

    let mut b = VecInstance::new(7);
    b.set(20, OverrideSlot::Fill([9, 9, 9, 9]));
    b.set(10, OverrideSlot::Fill([1, 2, 3, 4]));
    b.set(30, OverrideSlot::Hidden);

    assert!(a.is_canonical(), "a lista tem de sair ordenada: {a:?}");
    assert_eq!(
        postcard::to_allocvec(&a).expect("serializa"),
        postcard::to_allocvec(&b).expect("serializa"),
        "mesma regra, ordens de chegada diferentes, bytes diferentes: um passo de undo por frame"
    );
}

/// **Reescrever a MESMA espécie na MESMA peça substitui — não empilha.**
///
/// Arrastar um slider de cor emite um override por frame; se cada um deles fosse uma entrada, a
/// lista cresceria sem teto e o consumidor teria de decidir qual vale.
#[test]
fn a_second_write_of_the_same_slot_replaces_it() {
    let mut i = VecInstance::new(1);
    i.set(10, OverrideSlot::Fill([1, 0, 0, 255]));
    i.set(10, OverrideSlot::Fill([2, 0, 0, 255]));
    assert_eq!(i.overrides.len(), 1, "empilhou: {:?}", i.overrides);
    assert_eq!(i.get(10, 0), Some(OverrideSlot::Fill([2, 0, 0, 255])));
}

/// **Espécies DIFERENTES na mesma peça coexistem** — uma peça pode ser recolorida *e* escondida.
///
/// ⚠️ É a metade oposta do gate acima, e sem ela a substituição poderia estar a colapsar espécies
/// (o `Hidden` a comer o `Fill`) com o outro gate ainda verde.
#[test]
fn two_different_slots_on_one_piece_coexist() {
    let mut i = VecInstance::new(1);
    i.set(10, OverrideSlot::Fill([1, 0, 0, 255]));
    i.set(10, OverrideSlot::Hidden);
    assert_eq!(i.overrides.len(), 2);
    assert_eq!(i.get(10, 0), Some(OverrideSlot::Fill([1, 0, 0, 255])));
    assert_eq!(i.get(10, 1), Some(OverrideSlot::Hidden));
    assert!(i.is_canonical());
}

/// **Reset deixa a instância idêntica a uma recém-criada** — não "quase".
#[test]
fn reset_makes_it_byte_identical_to_a_fresh_instance() {
    let mut i = VecInstance::new(42);
    i.set(10, OverrideSlot::Fill([1, 2, 3, 4]));
    i.set(11, OverrideSlot::Hidden);
    i.reset();
    assert_eq!(
        postcard::to_allocvec(&i).expect("serializa"),
        postcard::to_allocvec(&VecInstance::new(42)).expect("serializa")
    );
}

/// A ordenação é por **espécie**, nunca pelo valor — senão mudar uma cor MOVE a entrada de lugar,
/// e duas instâncias iguais voltam a poder guardar ordens diferentes.
#[test]
fn the_sort_key_is_the_kind_not_the_value() {
    let mut i = VecInstance::new(1);
    i.set(10, OverrideSlot::Fill([255, 255, 255, 255]));
    i.set(11, OverrideSlot::Fill([0, 0, 0, 0]));
    let before: Vec<u64> = i.overrides.iter().map(|o| o.sub).collect();
    i.set(10, OverrideSlot::Fill([0, 0, 0, 0]));
    let after: Vec<u64> = i.overrides.iter().map(|o| o.sub).collect();
    assert_eq!(before, after, "mudar a cor reordenou a lista");
}
