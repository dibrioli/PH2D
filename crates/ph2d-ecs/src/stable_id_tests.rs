//! Testes do [`super`] — irmão pelo idioma de `#[path]` que a crate já usa em 5 sítios.

use super::*;
use crate::Transform;

fn world_with(n: usize) -> World {
    let mut w = World::new();
    for _ in 0..n {
        w.spawn(Transform::IDENTITY);
    }
    w
}

/// **Toda entidade editável recebe um id, e nenhum é `0`.**
#[test]
fn every_editable_entity_gets_a_non_zero_id() {
    let mut w = world_with(4);
    assert!(assign_missing_stable_ids(&mut w));
    let ids: Vec<u64> = w.query::<&StableId>().iter(&w).map(|s| s.0).collect();
    assert_eq!(ids.len(), 4, "as quatro receberam id");
    assert!(
        ids.iter().all(|&i| i >= StableId::FIRST),
        "o 0 e' reservado para 'nenhum': {ids:?}",
    );
}

/// **Idempotente** — a 2.ª corrida não muda nada e diz que não mudou.
///
/// ⚠️ É o que permite chamá-la todo o quadro ao lado do `assign_missing_root_order` sem que
/// ela seja trabalho: sem esta propriedade, a varredura reescreveria ids por quadro e **toda
/// captura de undo viraria um passo espúrio** — exatamente o defeito da classe BUGS #15.
#[test]
fn running_twice_changes_nothing() {
    let mut w = world_with(3);
    assert!(assign_missing_stable_ids(&mut w));
    let before: Vec<u64> = {
        let mut v: Vec<u64> = w.query::<&StableId>().iter(&w).map(|s| s.0).collect();
        v.sort_unstable();
        v
    };
    assert!(
        !assign_missing_stable_ids(&mut w),
        "a 2a corrida nao tem nada a fazer",
    );
    let after: Vec<u64> = {
        let mut v: Vec<u64> = w.query::<&StableId>().iter(&w).map(|s| s.0).collect();
        v.sort_unstable();
        v
    };
    assert_eq!(before, after, "os ids nao se mexem");
}

/// ⭐ **Ids NUNCA são reusados — nem depois de a entidade que os tinha desaparecer.**
///
/// É a invariante que separa este contador de um `max(existentes)+1`: apagar a entidade de
/// id mais alto e criar outra devolveria o MESMO id, e uma referência guardada (uma junta,
/// um binding) passaria a apontar para um objeto que não é o que ela nomeava.
#[test]
fn an_id_is_never_reused_after_its_entity_is_gone() {
    let mut w = world_with(2);
    assign_missing_stable_ids(&mut w);
    let highest = w
        .query::<(Entity, &StableId)>()
        .iter(&w)
        .max_by_key(|(_, s)| s.0)
        .map(|(e, s)| (e, s.0))
        .unwrap();
    w.despawn(highest.0);

    w.spawn(Transform::IDENTITY);
    assign_missing_stable_ids(&mut w);
    let ids: Vec<u64> = w.query::<&StableId>().iter(&w).map(|s| s.0).collect();
    assert!(
        !ids.contains(&highest.1),
        "o id {} da entidade apagada foi REUSADO — ids: {ids:?}",
        highest.1,
    );
}

/// **O contador reconcilia-se contra o mundo e nunca desce.**
///
/// O cenário é um load: o ficheiro traz um contador atrasado (ou nenhum) e o mundo já tem
/// ids altos. Sem a reconciliação, a próxima entidade nasceria com um id vivo.
#[test]
fn the_counter_never_hands_out_a_live_id() {
    let mut w = World::new();
    w.spawn((Transform::IDENTITY, StableId(500)));
    w.spawn((Transform::IDENTITY, StableId(900)));
    // Um contador atrasado, como um ficheiro antigo o traria.
    w.insert_resource(StableIdCounter::new(3));
    w.spawn(Transform::IDENTITY);

    assign_missing_stable_ids(&mut w);
    let ids: Vec<u64> = w.query::<&StableId>().iter(&w).map(|s| s.0).collect();
    assert!(
        ids.contains(&901),
        "a entidade nova tinha de vir depois do maior id vivo (900); ids: {ids:?}",
    );
    assert_eq!(
        w.resource::<StableIdCounter>().next_free(),
        902,
        "o contador avancou e ficou monotonico",
    );
}

/// **`reconcile_at_least` sobe, nunca desce** — a monotonicidade é do TIPO, não da promessa.
#[test]
fn reconcile_only_ever_climbs() {
    let mut c = StableIdCounter::new(10);
    c.reconcile_at_least(4);
    assert_eq!(c.next_free(), 10, "nao desceu");
    c.reconcile_at_least(42);
    assert_eq!(c.next_free(), 42, "subiu");
}

/// **Um contador semeado a `0` não entrega o id reservado.**
#[test]
fn a_zero_seed_still_never_hands_out_none() {
    assert_eq!(StableIdCounter::new(0).next_free(), StableId::FIRST);
    let mut w = world_with(1);
    w.insert_resource(StableIdCounter::new(0));
    assign_missing_stable_ids(&mut w);
    let id = w.query::<&StableId>().iter(&w).next().copied().unwrap();
    assert!(!id.is_none(), "0 e' 'nenhum', nunca um objeto");
}

/// ⚠️ **A prova MEDIDA de que `to_bits()` não é a ordem de criação no bevy 0.18.**
///
/// Este teste existe para que ninguém volte a trocar o `index()` da varredura por `to_bits()`
/// "para ficar igual ao `assign_missing_root_order`". Com `to_bits` as três entidades saíam
/// com os ids **`3, 2, 1`** — o `to_bits` do 0.18 **inverte** a ordem de criação.
///
/// ⛔ Isto **não** acusa o `RootOrder` de nada: lá a chave é a mesma que a árvore usa, e o
/// que importa é concordarem. Aqui o id é lido por humanos e pela migração da F1.
#[test]
fn to_bits_is_not_creation_order_which_is_why_the_sweep_uses_index() {
    let mut w = World::new();
    let a = w.spawn(Transform::IDENTITY).id();
    let b = w.spawn(Transform::IDENTITY).id();
    assert!(
        a.index() < b.index(),
        "o index e' ascendente com o spawn: {} {}",
        a.index(),
        b.index(),
    );
    assert!(
        a.to_bits() > b.to_bits(),
        "MEDIDO no bevy 0.18: o to_bits INVERTE a criacao ({} vs {}). Se esta asserção \
         falhar, o bevy mudou a codificacao — releia o cabecalho de `stable_id.rs` antes de \
         mexer na chave da varredura.",
        a.to_bits(),
        b.to_bits(),
    );
}

/// **A atribuição segue a ordem de SPAWN, não a do archetype.**
///
/// É a metade determinista da varredura: dois mundos construídos pela mesma sequência de
/// gestos têm de receber os mesmos ids, senão o `state_hash` deixa de ser função do
/// conteúdo e o replay 3-OS (HR-5) cai.
#[test]
fn ids_follow_spawn_order() {
    let mut w = World::new();
    let a = w.spawn(Transform::IDENTITY).id();
    let b = w.spawn(Transform::IDENTITY).id();
    let c = w.spawn(Transform::IDENTITY).id();
    assign_missing_stable_ids(&mut w);
    let ida = w.get::<StableId>(a).unwrap().0;
    let idb = w.get::<StableId>(b).unwrap().0;
    let idc = w.get::<StableId>(c).unwrap().0;
    assert!(
        ida < idb && idb < idc,
        "a ordem de spawn tem de ser a ordem dos ids: {ida} {idb} {idc}",
    );
}

/// **Uma entidade que JÁ tem id não é tocada** — é o caso do restore, e é a razão de a
/// varredura não colidir com o `snapshot_to_world` (ver o cabeçalho do módulo).
#[test]
fn a_restored_entity_keeps_the_id_it_came_with() {
    let mut w = World::new();
    let e = w.spawn((Transform::IDENTITY, StableId(77))).id();
    assign_missing_stable_ids(&mut w);
    assert_eq!(
        w.get::<StableId>(e).unwrap().0,
        77,
        "o id restaurado tem de sobreviver a varredura",
    );
}

/// A resolução nos dois sentidos, e o `NONE` que nunca resolve.
#[test]
fn an_id_resolves_back_to_its_entity() {
    let mut w = world_with(3);
    assign_missing_stable_ids(&mut w);
    let (e, id) = w
        .query::<(Entity, &StableId)>()
        .iter(&w)
        .map(|(e, s)| (e, *s))
        .next()
        .unwrap();
    assert_eq!(entity_of_stable_id(&mut w, id), Some(e));
    assert_eq!(stable_id_of(&w, e), Some(id));
    assert_eq!(
        entity_of_stable_id(&mut w, StableId::NONE),
        None,
        "'nenhum' nunca resolve para uma entidade",
    );
}
