//! Testes do [`super`] — a porta que as duas metades da entrega partilham.

use super::*;
use crate::{BodyKind, RigidBody};
use ph2d_ecs::Transform;

fn mundo() -> World {
    World::new()
}

/// A lista que a varredura visita, na ordem em que ela visita.
fn visitados(w: &World) -> Vec<Entity> {
    let mut v = Vec::new();
    for_each_keyboard_driven(w, |e| v.push(e));
    v
}

#[test]
fn um_player_de_plataforma_le_o_teclado() {
    let mut w = mundo();
    let e = w
        .spawn((Transform::IDENTITY, PlatformPlayer::default()))
        .id();
    assert!(reads_the_keyboard(&w, e));
    assert_eq!(visitados(&w), vec![e]);
}

/// ⭐⭐⭐ **O gate do report do dono**: o mover de vista de cima tem de estar na lista.
#[test]
fn e_um_mover_de_vista_de_cima_com_os_controlos_de_fabrica_tambem() {
    let mut w = mundo();
    let e = w
        .spawn((Transform::IDENTITY, TopDownPlayer::default()))
        .id();
    assert!(
        TopDownPlayer::default().default_controls,
        "o default do componente e' LER o teclado — se isto mudar, a cena de smoke muda com ele"
    );
    assert!(reads_the_keyboard(&w, e));
    assert_eq!(visitados(&w), vec![e]);
}

/// ⚠️ **O CONTROLO**: desligado, ele é motor puro e a varredura não o vê.
#[test]
fn mas_nao_quando_os_controlos_de_fabrica_estao_desligados() {
    let mut w = mundo();
    let e = w
        .spawn((
            Transform::IDENTITY,
            TopDownPlayer {
                default_controls: false,
                ..TopDownPlayer::default()
            },
        ))
        .id();
    assert!(!reads_the_keyboard(&w, e));
    assert!(visitados(&w).is_empty());
}

/// ⚠️ **Com os DOIS na mesma entidade ela é visitada UMA vez** — senão a contagem de players que o
/// chamador usa para decidir *«isto é uma corrida?»* mentiria.
#[test]
fn com_os_dois_movers_a_entidade_e_visitada_uma_vez_so() {
    let mut w = mundo();
    let e = w
        .spawn((
            Transform::IDENTITY,
            PlatformPlayer::default(),
            TopDownPlayer::default(),
        ))
        .id();
    assert_eq!(visitados(&w), vec![e]);
}

/// Um corpo qualquer não lê nada.
#[test]
fn um_corpo_sem_mover_nenhum_nao_le_o_teclado() {
    let mut w = mundo();
    let e = w
        .spawn((
            Transform::IDENTITY,
            RigidBody {
                kind: BodyKind::Dynamic,
            },
        ))
        .id();
    assert!(!reads_the_keyboard(&w, e));
    assert!(visitados(&w).is_empty());
}
