//! Testes do [`super`] — irmão pelo idioma de `#[path]` que a crate já usa.
//!
//! # ⛔⛔ Este ficheiro NÃO EXISTIA, e é por isso que a inversão sobreviveu
//!
//! A [`super::assign_missing_root_order`] é foundational, corre em **todo** quadro e decide a
//! pilha de z do canvas inteiro — e não tinha um único gate próprio. Quando a chave das raízes foi
//! unificada numa porta partilhada ([`crate::root_key`], 2026-08-27), os dois leitores que a
//! documentação nomeia (a lista e o `propagate_transforms`) passaram a `index()` e **este terceiro
//! leitor ficou em `to_bits()`** — que o gate
//! `to_bits_is_not_creation_order_which_is_why_the_sweep_uses_index` da irmã
//! [`crate::assign_missing_stable_ids`] MEDE como sendo a ordem **INVERTIDA** de criação.
//!
//! ⇒ a varredura cujo doc promete *«a tela não muda; ela só para de escorregar»* **virava a pilha
//! de z ao contrário** no primeiro quadro, para toda cena cujas raízes nascem sem número.
//!
//! Report do dono (2026-09-15): *«Para o Hero ser visível deve ficar abaixo na Hierarchy»* — o
//! chão, criado PRIMEIRO, recebia a ordem mais alta e passava a desenhar **por cima de tudo**.

use super::*;
use crate::Transform;

/// `n` raízes criadas em sequência, sem `RootOrder` nenhum.
fn roots(n: usize) -> (World, Vec<Entity>) {
    let mut w = World::new();
    let es: Vec<Entity> = (0..n).map(|_| w.spawn(Transform::IDENTITY).id()).collect();
    (w, es)
}

/// A ordem em que a árvore (e o canvas) as mostra — **pela porta partilhada**.
fn como_a_arvore_mostra(w: &World, es: &[Entity]) -> Vec<Entity> {
    let mut v = es.to_vec();
    v.sort_by_key(|&e| crate::root_key(w, e));
    v
}

/// ⭐⭐⭐ **A varredura congela a ordem que a árvore JÁ MOSTRA — e é a promessa do doc dela.**
///
/// ⚠️ Antes da cura este gate lia a lista **exactamente ao contrário**.
#[test]
fn the_sweep_freezes_the_order_the_tree_already_showed() {
    let (mut w, es) = roots(4);
    let antes = como_a_arvore_mostra(&w, &es);
    assert!(assign_missing_root_order(&mut w));
    let depois = como_a_arvore_mostra(&w, &es);
    assert_eq!(
        antes, depois,
        "a varredura MEXEU na ordem que a arvore mostrava.\n\
         ⚠️ Ela e' o terceiro leitor da chave das raizes: se nao usar a MESMA (`crate::root_key`), \
         a pilha de z do canvas vira-se ao contrario no primeiro quadro."
    );
}

/// ⭐⭐ **E a ordem congelada é a de CRIAÇÃO** — a metade que o artista lê.
///
/// ⛔ Sem este gate, a cura podia ser *«ordenar por qualquer coisa estável»*: um empate resolvido
/// de forma consistente mas arbitrária passa no gate de cima e continua a pôr o primeiro objecto
/// criado no fim da pilha.
#[test]
fn and_the_frozen_order_is_the_creation_order() {
    let (mut w, es) = roots(4);
    assert!(assign_missing_root_order(&mut w));
    let ordens: Vec<u32> = es
        .iter()
        .map(|&e| w.get::<RootOrder>(e).expect("numero").0)
        .collect();
    let mut esperado = ordens.clone();
    esperado.sort_unstable();
    assert_eq!(
        ordens, esperado,
        "a primeira raiz criada tem de receber o MENOR numero (desenha ATRAS): {ordens:?}"
    );
}

/// ⚠️ **As sem-número entram DEPOIS das explícitas** — a metade que o doc já prometia e que a
/// cura não pode partir.
#[test]
fn the_unordered_ones_collate_after_the_explicit_ones() {
    let mut w = World::new();
    let explicita = w.spawn((Transform::IDENTITY, RootOrder(7))).id();
    let nova = w.spawn(Transform::IDENTITY).id();
    assert!(assign_missing_root_order(&mut w));
    let a = w.get::<RootOrder>(explicita).expect("numero").0;
    let b = w.get::<RootOrder>(nova).expect("numero").0;
    assert_eq!(a, 7, "uma ordem explicita nao se toca");
    assert!(
        b > a,
        "a nova ({b}) tinha de entrar depois da explicita ({a})"
    );
}

/// A varredura é idempotente — corre todo o quadro.
#[test]
fn running_it_twice_is_a_no_op() {
    let (mut w, es) = roots(3);
    assert!(assign_missing_root_order(&mut w));
    let antes: Vec<u32> = es
        .iter()
        .map(|&e| w.get::<RootOrder>(e).expect("numero").0)
        .collect();
    assert!(!assign_missing_root_order(&mut w), "nao havia nada a fazer");
    let depois: Vec<u32> = es
        .iter()
        .map(|&e| w.get::<RootOrder>(e).expect("numero").0)
        .collect();
    assert_eq!(antes, depois);
}
