//! ⭐⭐ **A tag da RECEITA chega às cópias** — a decisão do dono D4 (*«sim»*), gate 20 do plano de
//! Tags (`docs/Components/08_plano_tags.md` §5.2).
//!
//! ⚠️ **Não há código de tags no sync, e é de propósito**: o `Tags` é um componente REGISTADO com
//! o descritor `Propagate`, e a lei geral das instâncias (ADR-0164) é que o faz viajar. Este gate
//! prova que a lei geral o alcança — e que a cópia pode ter a sua própria lista, que é a outra
//! metade da D4.
//!
//! ⚠️ **Compara-se o COMPONENTE inteiro, e nunca a lista directa**: ler `direct_ids` fora da porta é
//! o que o censo `only_the_door_reads_tags` proíbe, e a igualdade do `Tags` é a mesma pergunta.

use crate::test_support::{instantiate, pass, piece, plain_master, registo};
use ph2d_ecs::SimWorld;
use ph2d_ecs::tags::Tags;
use ph2d_physics_ecs::PhysicsBridge;
use ph2d_tags::TagId;

fn tags_de(sim: &SimWorld, root: ph2d_ecs::Entity) -> Option<Tags> {
    sim.world().get::<Tags>(piece(sim, root, "Arm")).cloned()
}

/// ⭐⭐⭐ **Marcar a peça da receita marca a mesma peça em todas as cópias; uma cópia nova nasce
/// marcada; e a lista própria de uma cópia sobrevive ao passe.**
///
/// **Mutações que devem sangrar:** o descritor do `Tags` com `InstanceLocal` (a receita deixaria de
/// chegar) · o `Tags` fora do registo (nem chega nem é copiado) · o passe a sobrescrever a excepção
/// da cópia.
#[test]
fn a_recipe_tag_reaches_every_copy() {
    let mut sim = SimWorld::new();
    let r = registo();
    let bridge = PhysicsBridge::new();
    let mut echo = crate::instance_sync::MasterEcho::default();
    let master = plain_master(&mut sim);
    let a = instantiate(&mut sim, &r, master, None).expect("instancia");
    let b = instantiate(&mut sim, &r, master, None).expect("instancia");
    let _ = pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(tags_de(&sim, a), None, "controlo: ninguem tem tags ainda");

    // O artista marca o braço da RECEITA.
    let marca = Tags::from_ids([TagId(4)]);
    let braco = piece(&sim, master, "Arm");
    sim.world_mut().entity_mut(braco).insert(marca.clone());
    assert!(
        pass(&mut sim, &r, &bridge, &mut echo) > 0,
        "o sync nao escreveu nada"
    );
    for (i, root) in [a, b].into_iter().enumerate() {
        assert_eq!(
            tags_de(&sim, root).as_ref(),
            Some(&marca),
            "a copia {} nao recebeu a tag da receita",
            i + 1
        );
    }

    // Uma cópia NOVA nasce marcada.
    let c = instantiate(&mut sim, &r, master, None).expect("instancia");
    assert_eq!(tags_de(&sim, c).as_ref(), Some(&marca));

    // A cópia `a` ganha a sua própria lista — e o passe não a devolve à da receita.
    let propria = Tags::from_ids([TagId(4), TagId(7)]);
    let braco_a = piece(&sim, a, "Arm");
    sim.world_mut().entity_mut(braco_a).insert(propria.clone());
    let _ = pass(&mut sim, &r, &bridge, &mut echo);
    let _ = pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        tags_de(&sim, a).as_ref(),
        Some(&propria),
        "o passe apagou a lista propria da copia"
    );
    assert_eq!(
        tags_de(&sim, b).as_ref(),
        Some(&marca),
        "a excepcao de uma copia vazou para a irma"
    );
}
