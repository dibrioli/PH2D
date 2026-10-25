//! A lei de [`super`] — largar uma ou várias linhas da Hierarquia (F30 §F, report do dono 25/10).

use super::{Desfecho, Queda, quem_viaja, reparentar};
use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform};

fn raiz(sim: &mut SimWorld, nome: &str, ordem: u32, x: f32) -> Entity {
    let mut t = Transform::default();
    t.translation = ph2d_core::Vec2::new(x, 0.0);
    sim.world_mut()
        .spawn((t, Name::new(nome), RootOrder(ordem)))
        .id()
}

fn pai_de(sim: &SimWorld, e: Entity) -> Option<Entity> {
    sim.world().get::<ChildOf>(e).map(ChildOf::parent)
}

fn mundo_x(sim: &SimWorld, e: Entity) -> f32 {
    let l = *sim.world().get::<Transform>(e).expect("transform");
    Transform::compose(ph2d_ecs::parent_world_transform(sim.world(), e), l)
        .translation
        .x
}

/// A cena: P (x=100, escala 2) e A, B, C, D raízes por esta ordem.
fn cena() -> (SimWorld, Entity, [Entity; 4]) {
    let mut sim = SimWorld::new();
    let p = raiz(&mut sim, "P", 0, 100.0);
    sim.world_mut().get_mut::<Transform>(p).expect("p").scale = ph2d_core::Vec2::new(2.0, 2.0);
    let a = raiz(&mut sim, "A", 1, 10.0);
    let b = raiz(&mut sim, "B", 2, 20.0);
    let c = raiz(&mut sim, "C", 3, 30.0);
    let d = raiz(&mut sim, "D", 4, 40.0);
    (sim, p, [a, b, c, d])
}

#[test]
fn largar_varios_seleccionados_num_objecto_torna_os_todos_filhos() {
    let (mut sim, p, [a, b, c, d]) = cena();
    // Seleccionados C e A (C clicado primeiro); agarra-se o A.
    let quem = quem_viaja(sim.world_mut(), a, &[c, a]);
    let r = reparentar(
        &mut sim,
        &quem,
        Queda {
            new_parent: Some(p),
            ..Queda::default()
        },
    );
    assert!(matches!(r, Desfecho::Movidos { .. }), "{r:?}");
    assert_eq!(pai_de(&sim, a), Some(p));
    assert_eq!(pai_de(&sim, c), Some(p));
    assert_eq!(pai_de(&sim, b), None, "B nao estava seleccionado");
    assert_eq!(pai_de(&sim, d), None);
}

#[test]
fn agarrar_uma_linha_nao_seleccionada_leva_so_ela() {
    let (mut sim, p, [a, b, c, _]) = cena();
    let quem = quem_viaja(sim.world_mut(), b, &[a, c]);
    assert_eq!(quem, vec![b]);
    let _ = reparentar(
        &mut sim,
        &quem,
        Queda {
            new_parent: Some(p),
            ..Queda::default()
        },
    );
    assert_eq!(pai_de(&sim, b), Some(p));
    assert_eq!(pai_de(&sim, a), None);
    assert_eq!(pai_de(&sim, c), None);
}

#[test]
fn largar_varios_mantem_a_ordem_e_o_lugar_no_mundo() {
    let (mut sim, p, [a, b, c, d]) = cena();
    let antes: Vec<f32> = [a, b, c, d].iter().map(|&e| mundo_x(&sim, e)).collect();
    // Ordem de clique D, A, C — a da Hierarquia é A, C, D.
    let quem = quem_viaja(sim.world_mut(), c, &[d, a, c]);
    assert_eq!(
        quem,
        vec![a, c, d],
        "o bloco segue a ordem da Hierarquia, nao a do clique"
    );
    let _ = reparentar(
        &mut sim,
        &quem,
        Queda {
            new_parent: Some(p),
            ..Queda::default()
        },
    );
    assert_eq!(ph2d_ecs::ordered_children(sim.world(), p), vec![a, c, d]);
    for (i, &e) in [a, b, c, d].iter().enumerate() {
        assert!(
            (mundo_x(&sim, e) - antes[i]).abs() < 1e-3,
            "a peca {i} saltou no mundo: {} -> {}",
            antes[i],
            mundo_x(&sim, e)
        );
    }
    // E com o pai a ESCALA 2, o local não é o mundo — prova que a conta correu.
    let la = sim.world().get::<Transform>(a).expect("a").translation.x;
    assert!(
        (la - (10.0 - 100.0) / 2.0).abs() < 1e-3,
        "local de A = {la}"
    );
}

#[test]
fn nao_se_larga_um_objecto_dentro_de_si_proprio() {
    let (mut sim, _p, [a, b, c, _]) = cena();
    sim.world_mut().entity_mut(c).insert(ChildOf(b));
    // Largar {A, B} dentro de B: recusado.
    let quem = quem_viaja(sim.world_mut(), a, &[a, b]);
    assert_eq!(
        reparentar(
            &mut sim,
            &quem,
            Queda {
                new_parent: Some(b),
                ..Queda::default()
            }
        ),
        Desfecho::RecusaCiclo
    );
    // Dentro de C, filho de B (seleccionado): recusado também.
    assert_eq!(
        reparentar(
            &mut sim,
            &quem,
            Queda {
                new_parent: Some(c),
                ..Queda::default()
            }
        ),
        Desfecho::RecusaCiclo
    );
    // ⛔ E nada mudou — atómico.
    assert_eq!(pai_de(&sim, a), None);
    assert_eq!(pai_de(&sim, b), None);
    assert_eq!(pai_de(&sim, c), Some(b));
}

#[test]
fn um_seleccionado_filho_de_outro_seleccionado_vai_junto_com_o_pai() {
    let (mut sim, p, [a, b, c, _]) = cena();
    sim.world_mut().entity_mut(c).insert(ChildOf(b));
    let quem = quem_viaja(sim.world_mut(), c, &[a, b, c]);
    assert_eq!(quem, vec![a, b], "C viaja dentro do B");
    let _ = reparentar(
        &mut sim,
        &quem,
        Queda {
            new_parent: Some(p),
            ..Queda::default()
        },
    );
    assert_eq!(pai_de(&sim, b), Some(p));
    assert_eq!(pai_de(&sim, c), Some(b), "C nao foi arrancado do B");
}

#[test]
fn reordenar_varios_entre_as_raizes_leva_o_bloco_para_o_alvo() {
    let (mut sim, p, [a, b, c, d]) = cena();
    // {B, D} largados ANTES do A.
    let quem = quem_viaja(sim.world_mut(), d, &[b, d]);
    let _ = reparentar(
        &mut sim,
        &quem,
        Queda {
            before: Some(a),
            ..Queda::default()
        },
    );
    let mut raizes: Vec<(u32, Entity)> = [p, a, b, c, d]
        .iter()
        .map(|&e| (sim.world().get::<RootOrder>(e).expect("ordem").0, e))
        .collect();
    raizes.sort_unstable();
    let ordem: Vec<Entity> = raizes.into_iter().map(|(_, e)| e).collect();
    assert_eq!(ordem, vec![p, b, d, a, c]);
}

#[test]
fn largar_varios_na_raiz_tira_os_todos_do_pai() {
    let (mut sim, p, [a, b, c, _]) = cena();
    for e in [a, b, c] {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    let antes: Vec<f32> = [a, c].iter().map(|&e| mundo_x(&sim, e)).collect();
    let quem = quem_viaja(sim.world_mut(), a, &[a, c]);
    let _ = reparentar(&mut sim, &quem, Queda::default());
    assert_eq!(pai_de(&sim, a), None);
    assert_eq!(pai_de(&sim, c), None);
    assert_eq!(pai_de(&sim, b), Some(p), "B nao estava seleccionado");
    assert!((mundo_x(&sim, a) - antes[0]).abs() < 1e-3);
    assert!((mundo_x(&sim, c) - antes[1]).abs() < 1e-3);
}
