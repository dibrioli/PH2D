//! Os gates do passe dos ids de item (F30 §A1).

use super::*;
use crate::scene::registry::{ComponentRegistry, register_ecs_components};
use crate::{StableIdCounter, Timer, Timers, Transform, assign_missing_stable_ids};

fn timer(nome: &str) -> Timer {
    Timer {
        name: nome.into(),
        ..Timer::default()
    }
}

fn ids(w: &World, e: Entity) -> Vec<u32> {
    w.get::<Timers>(e)
        .expect("timers")
        .0
        .iter()
        .map(|t| t.item.0)
        .collect()
}

fn com_timers(w: &mut World, nomes: &[&str]) -> Entity {
    w.spawn((
        Transform::IDENTITY,
        Timers(nomes.iter().map(|n| timer(n)).collect()),
    ))
    .id()
}

/// ⭐ Todo item recebe um id ≠ 0, na ordem da lista, e o passe é idempotente.
#[test]
fn todo_item_recebe_um_id_e_o_passe_e_idempotente() {
    let mut w = World::new();
    let e = com_timers(&mut w, &["a", "b", "c"]);
    assert!(assign_missing_stable_ids(&mut w));
    let v = ids(&w, e);
    assert!(v.iter().all(|&i| i != 0), "{v:?}");
    assert!(
        v.windows(2).all(|p| p[0] < p[1]),
        "na ordem da lista: {v:?}"
    );
    assert!(
        !assign_missing_item_ids(&mut w),
        "a 2.ª corrida não muda nada"
    );
    assert_eq!(ids(&w, e), v);
}

/// ⭐⭐ **Um id de item nunca se reusa**: apagar o último e acrescentar outro dá um id NOVO — a chave
/// órfã do apagado não acorda no novo.
#[test]
fn um_id_de_item_nunca_se_reusa() {
    let mut w = World::new();
    let e = com_timers(&mut w, &["a", "b"]);
    assign_missing_stable_ids(&mut w);
    let antes = ids(&w, e);
    let apagado = antes[1];
    {
        let mut t = w.get_mut::<Timers>(e).expect("timers");
        t.0.pop();
        t.0.push(timer("c"));
    }
    assign_missing_stable_ids(&mut w);
    let depois = ids(&w, e);
    assert_eq!(depois[0], antes[0], "o que ficou guarda o id");
    assert_ne!(
        depois[1], apagado,
        "o id do apagado não renasce: {antes:?} → {depois:?}"
    );
    assert!(depois[1] > apagado);
}

/// ⭐⭐ **O contador reconcilia com os ids de item** antes de alocar: um documento cujo contador
/// vem atrasado não entrega um id que um item já tem.
#[test]
fn o_contador_reconcilia_com_os_ids_de_item() {
    let mut w = World::new();
    let mut a = timer("a");
    a.item = ItemId(50);
    let e = w
        .spawn((Transform::IDENTITY, Timers(vec![a, timer("b")])))
        .id();
    w.insert_resource(StableIdCounter::new(1));
    assign_missing_item_ids(&mut w);
    let v = ids(&w, e);
    assert_eq!(v[0], 50);
    assert!(v[1] > 50, "o novo não colide com o vivo: {v:?}");
    let c = w.resource::<StableIdCounter>().next_free();
    assert!(c > u64::from(v[1]), "o contador ficou à frente: {c}");
}

/// ⭐ Um item DUPLICADO na mesma lista (a cópia de uma linha) ganha um id novo; o primeiro fica.
#[test]
fn um_item_duplicado_na_mesma_lista_ganha_id_novo() {
    let mut w = World::new();
    let e = com_timers(&mut w, &["a"]);
    assign_missing_stable_ids(&mut w);
    let a = w.get::<Timers>(e).expect("timers").0[0].clone();
    w.get_mut::<Timers>(e).expect("timers").0.push(a.clone());
    assign_missing_stable_ids(&mut w);
    let v = ids(&w, e);
    assert_eq!(v[0], a.item.0, "o primeiro guarda o id");
    assert_ne!(v[1], v[0]);
}

/// ⭐⭐ **Copiar um objecto guarda os ids dos itens** — a chave leva a entidade, e o sync
/// mestre→instância compara bytes: refazer os ids da cópia brigaria com ele a cada quadro. A
/// unicidade é dentro da entidade (o teste acima).
#[test]
fn copiar_um_objecto_guarda_os_ids_dos_itens() {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    let mut w = World::new();
    let e = com_timers(&mut w, &["a", "b"]);
    assign_missing_stable_ids(&mut w);
    let antes = ids(&w, e);
    let copia = crate::deep_copy_subtree(&mut w, &reg, e, None).expect("copia");
    assign_missing_stable_ids(&mut w);
    assert_eq!(ids(&w, copia.root), antes, "a cópia leva os ids");
    assert_eq!(ids(&w, e), antes, "o original não muda");
}

/// O registo conhece os nove componentes do `ph2d-ecs` com itens, e cada um está no registo de
/// componentes (senão a migração não o acha).
#[test]
fn os_passes_desta_crate_sao_componentes_registados() {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    for p in desta_crate() {
        assert!(reg.get_by_name(p.canonical).is_some(), "{}", p.canonical);
    }
}

fn filtro(kinds: &[u8]) -> crate::VecFilter {
    crate::VecFilter::from_ops(kinds.iter().map(|k| crate::FxOp::new(*k)).collect())
}

fn ids_do_filtro(w: &World, e: Entity) -> Vec<u32> {
    (w.get::<crate::VecFilter>(e).expect("filtro").ids().iter())
        .map(|i| i.0)
        .collect()
}

/// ⭐⭐ **Um id repetido no mesmo filtro é refeito** (F31 §B2): fica o primeiro na ordem da pilha,
/// o segundo recebe um novo; outra entidade pode ter o mesmo id (a cópia leva os ids).
#[test]
fn um_id_repetido_no_mesmo_filtro_e_refeito() {
    use crate::FxOp;
    let mut w = World::new();
    let mut f = filtro(&[FxOp::BLUR, FxOp::GLOW, FxOp::DROP_SHADOW]);
    f.com_ids(&[ItemId(40), ItemId(40), ItemId::NONE]);
    let e = w.spawn((Transform::IDENTITY, f.clone())).id();
    let copia = w.spawn((Transform::IDENTITY, f)).id();
    assert!(assign_missing_item_ids(&mut w));
    let v = ids_do_filtro(&w, e);
    assert_eq!(v[0], 40, "fica o primeiro");
    assert!(
        v[1] != 40 && v[1] != 0 && v[2] != 0 && v[1] != v[2],
        "{v:?}"
    );
    assert_eq!(
        ids_do_filtro(&w, copia)[0],
        40,
        "outra entidade guarda o seu"
    );
    assert!(!assign_missing_item_ids(&mut w), "idempotente");
}

/// ⭐⭐ **O contador reconcilia com os ids dos filtros**: um contador atrasado (um ficheiro antigo)
/// nunca entrega um id que um filtro já tem.
#[test]
fn o_contador_reconcilia_com_os_ids_dos_filtros() {
    use crate::FxOp;
    let mut w = World::new();
    w.insert_resource(StableIdCounter::new(5));
    let mut f = filtro(&[FxOp::BLUR, FxOp::GLOW]);
    f.com_ids(&[ItemId(900), ItemId::NONE]);
    let e = w.spawn((Transform::IDENTITY, f)).id();
    assign_missing_item_ids(&mut w);
    let v = ids_do_filtro(&w, e);
    assert!(v[1] > 900, "o novo passa o maior vivo: {v:?}");
    assert!(w.resource::<StableIdCounter>().next_free() > u64::from(v[1]));
}
