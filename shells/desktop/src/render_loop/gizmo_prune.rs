//! Poda da seleção do gizmo: **só a morte de uma entidade tira alguém dali**.
//!
//! O mundo não avisa quando uma entidade some (despawn em cascata, delete do pai),
//! então a seleção fica com bits pendurados: `selected_len()` continua > 1 e o
//! gizmo global segue pintando em volta do único sprite que sobrou (Enio,
//! 2026-06-08: "se deletar algumas e sobrar 1, o gizmo global fica aparecendo
//! mesmo com uma sprite"). A poda roda uma vez por frame, antes de reconstruir as
//! views.
//!
//! O teste é a **morte da entidade** — e não a ausência de `GizmoView`.
//! O atalho antigo ("sem view = morreu") só valia enquanto tudo que se podia
//! selecionar era um sprite. Desde o [ADR-0110] a seleção também carrega entidades
//! **vetoriais**, que não têm `Sprite` e portanto nunca têm view: com o atalho,
//! toda entidade vetorial era expulsa da seleção no mesmo frame em que entrava — a
//! linha acendia e apagava sozinha na Hierarquia, e o pen perdia a seleção (e com
//! ela os handles) um frame depois.
//!
//! Sem view apenas não se pinta o gizmo de sprite. A seleção continua viva.
//!
//! [ADR-0110]: ../../../../docs/architecture/decisions/0110-vector-nodes-are-ecs-entities-one-hierarchy.md

use ph2d_ecs::SimWorld;
use ph2d_editor::screens::hero::GizmoStateGroup;

/// A entidade ainda existe no `SimWorld`.
pub(super) fn entity_alive(sim: &SimWorld, bits: u64) -> bool {
    sim.world()
        .get_entity(ph2d_ecs::Entity::from_bits(bits))
        .is_ok()
}

/// Tira da seleção as entidades que morreram. Se o primário morreu, promove o
/// extra mais antigo (ou esvazia). Nada mais sai daqui.
pub(super) fn prune_dead(gizmo: &mut GizmoStateGroup, sim: &SimWorld) {
    gizmo.extra_selection.retain(|b| entity_alive(sim, *b));
    if gizmo.selection.is_some_and(|b| !entity_alive(sim, b)) {
        gizmo.selection = if gizmo.extra_selection.is_empty() {
            None
        } else {
            Some(gizmo.extra_selection.remove(0))
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::{Name, Transform};

    fn spawn(sim: &mut SimWorld, name: &str) -> u64 {
        sim.world_mut()
            .spawn((Transform::default(), Name::new(name)))
            .id()
            .to_bits()
    }

    /// REGRESSÃO (ADR-0110; Enio 2026-07-09: "os handles não aparecem mais").
    /// Uma entidade VIVA sem `Sprite` — um path vetorial — não tem `GizmoView`, e
    /// mesmo assim continua selecionada. Era esta poda que a expulsava todo frame.
    #[test]
    fn a_live_entity_without_a_sprite_is_never_pruned() {
        let mut sim = SimWorld::default();
        let (vec_path, other) = (spawn(&mut sim, "Path 1"), spawn(&mut sim, "Path 2"));
        let mut gizmo = GizmoStateGroup::default();
        gizmo.replace_selection(Some(vec_path));
        gizmo.add_to_selection(other);

        for _ in 0..3 {
            prune_dead(&mut gizmo, &sim);
        }
        assert_eq!(gizmo.selection, Some(vec_path));
        assert_eq!(gizmo.extra_selection, vec![other]);
    }

    /// O que a poda existe para fazer: entidade morta sai, e o primário morto é
    /// substituído pelo extra mais antigo.
    #[test]
    fn a_dead_primary_is_replaced_by_the_oldest_extra_and_dead_extras_leave() {
        let mut sim = SimWorld::default();
        let (a, b, c) = (
            spawn(&mut sim, "A"),
            spawn(&mut sim, "B"),
            spawn(&mut sim, "C"),
        );
        let mut gizmo = GizmoStateGroup::default();
        gizmo.replace_selection(Some(a));
        gizmo.add_to_selection(b);
        gizmo.add_to_selection(c);

        sim.world_mut().despawn(ph2d_ecs::Entity::from_bits(b));
        prune_dead(&mut gizmo, &sim);
        assert_eq!(gizmo.selection, Some(a), "o primário está vivo");
        assert_eq!(gizmo.extra_selection, vec![c], "o extra morto saiu");

        sim.world_mut().despawn(ph2d_ecs::Entity::from_bits(a));
        prune_dead(&mut gizmo, &sim);
        assert_eq!(gizmo.selection, Some(c), "promoveu o extra");
        assert!(gizmo.extra_selection.is_empty());

        sim.world_mut().despawn(ph2d_ecs::Entity::from_bits(c));
        prune_dead(&mut gizmo, &sim);
        assert_eq!(gizmo.selection, None, "morreram todos");
        assert_eq!(gizmo.selected_len(), 0);
    }
}
