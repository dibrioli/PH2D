//! Hierarchy drag-reparent drain (`EditorAction::HierReparent`).
//!
//! Wave 3.1 stage A — extracted from `hero_intents.rs` as part of
//! the HR-18 closeout split. Behavior-preserving lift.

use ph2d_ecs::SimWorld;

/// Drain a `pending_reparent` hierarchy intent: move a dragged
/// hierarchy row to a new parent (or root), positioning it relative
/// to a target sibling (before / after). Guards against cycles
/// (refuses to make dragged a descendant of itself). Re-inserts
/// `ChildOf` on every sibling in the desired order so bevy_ecs's
/// `Children` list reflects the user-chosen sequence.
///
/// Returns `false` — never pushes a toast (silent reparent matches
/// existing UX). Caller does not set title_dirty.
pub(crate) fn drain_reparent(
    intent: ph2d_editor::screens::hero::HierReparentIntent,
    live: &crate::HeroLive,
    sim: &mut SimWorld,
) -> bool {
    use ph2d_ecs::Transform;
    let Some(dragged_bits) = live.bridge.entity_for(intent.dragged) else {
        return false;
    };
    let dragged = ph2d_ecs::Entity::from_bits(dragged_bits);

    let new_parent_entity = if let Some(parent_node) = intent.new_parent
        && let Some(parent_bits) = live.bridge.entity_for(parent_node)
    {
        Some(ph2d_ecs::Entity::from_bits(parent_bits))
    } else if let Some(before_node) = intent.before
        && let Some(target_bits) = live.bridge.entity_for(before_node)
    {
        let target = ph2d_ecs::Entity::from_bits(target_bits);
        sim.world()
            .get::<ph2d_ecs::ChildOf>(target)
            .map(|c| c.parent())
    } else if let Some(after_node) = intent.after
        && let Some(target_bits) = live.bridge.entity_for(after_node)
    {
        let target = ph2d_ecs::Entity::from_bits(target_bits);
        sim.world()
            .get::<ph2d_ecs::ChildOf>(target)
            .map(|c| c.parent())
    } else {
        None
    };
    // World-transform preservation (Enio 2026-06-08): a sprite that is
    // rotated / displaced must NOT jump when it gains or loses a parent —
    // its GLOBAL position + rotation must be kept. We capture the dragged
    // entity's current world transform HERE (under its OLD parent chain),
    // and after the `ChildOf` change re-solve its LOCAL transform so the
    // world transform round-trips. Without this, only `ChildOf` changes and
    // `parent_world_transform` then composes a different chain → the sprite
    // visibly snaps to a new pose.
    let old_local = sim.world().get::<Transform>(dragged).copied();
    let old_world = old_local
        .map(|l| Transform::compose(ph2d_ecs::parent_world_transform(sim.world(), dragged), l));
    // ⭐ **A MESMA lei para um nó de MODELAGEM 3D** (ADR-0161 W30), e ela precisa de linha própria
    // por uma razão de tipo: a pose de uma peça de modelagem não é `Transform` — é `FieldPose`, com
    // rotação em quaternion e escala uniforme —, então o `old_world` acima sai **`None`** e a peça
    // saltava ao mudar de pai. Ver `ph2d_field_ecs::set_world_xform`.
    let old_field_world = ph2d_field_ecs::field_world_xform(sim.world(), dragged);
    let sim_w = sim.world_mut();
    let would_cycle = new_parent_entity.is_some_and(|np| {
        let mut current = Some(np);
        while let Some(c) = current {
            if c == dragged {
                return true;
            }
            current = sim_w.get::<ph2d_ecs::ChildOf>(c).map(|c| c.parent());
        }
        false
    });
    if would_cycle {
        return false;
    }
    // Step 1: pick the new ChildOf relation.
    if let Ok(mut entry) = sim_w.get_entity_mut(dragged) {
        match new_parent_entity {
            Some(p) => {
                entry.insert(ph2d_ecs::ChildOf(p));
            }
            None => {
                entry.remove::<ph2d_ecs::ChildOf>();
            }
        }
    }
    // M14.7 polish: root drops need an explicit `RootOrder`.
    if new_parent_entity.is_none() {
        let mut roots: Vec<ph2d_ecs::Entity> = {
            let mut q = sim_w.query_filtered::<ph2d_ecs::Entity, (
                ph2d_ecs::With<Transform>,
                ph2d_ecs::Without<ph2d_ecs::ChildOf>,
            )>();
            let mut acc: Vec<(ph2d_ecs::Entity, u32)> = Vec::new();
            for entity in q.iter(sim_w) {
                if entity == dragged {
                    continue;
                }
                let order = sim_w
                    .get::<ph2d_ecs::RootOrder>(entity)
                    .map(|r| r.0)
                    .unwrap_or(u32::MAX);
                acc.push((entity, order));
            }
            acc.sort_unstable_by(|a, b| {
                a.1.cmp(&b.1)
                    .then_with(|| a.0.to_bits().cmp(&b.0.to_bits()))
            });
            acc.into_iter().map(|(e, _)| e).collect()
        };
        let before_target = intent
            .before
            .and_then(|n| live.bridge.entity_for(n))
            .map(ph2d_ecs::Entity::from_bits);
        let after_target = intent
            .after
            .and_then(|n| live.bridge.entity_for(n))
            .map(ph2d_ecs::Entity::from_bits);
        let insert_at = if let Some(b) = before_target {
            roots.iter().position(|e| *e == b).unwrap_or(roots.len())
        } else if let Some(a) = after_target {
            roots
                .iter()
                .position(|e| *e == a)
                .map(|i| i + 1)
                .unwrap_or(roots.len())
        } else {
            roots.len()
        };
        roots.insert(insert_at.min(roots.len()), dragged);
        for (idx, e) in roots.iter().enumerate() {
            if let Ok(mut entry) = sim_w.get_entity_mut(*e) {
                entry.insert(ph2d_ecs::RootOrder(idx as u32));
            }
        }
    }
    // Step 2: enforce sibling order.
    let target_kind: Option<(ph2d_ecs::Entity, bool)> = if let Some(before_node) = intent.before
        && let Some(b) = live.bridge.entity_for(before_node)
    {
        Some((ph2d_ecs::Entity::from_bits(b), true))
    } else if let Some(after_node) = intent.after
        && let Some(a) = live.bridge.entity_for(after_node)
    {
        Some((ph2d_ecs::Entity::from_bits(a), false))
    } else {
        None
    };
    if let (Some(parent), Some((target, place_before))) = (new_parent_entity, target_kind) {
        let current: Vec<ph2d_ecs::Entity> = sim_w
            .get::<bevy_ecs::hierarchy::Children>(parent)
            .map(|c| c.iter().copied().filter(|e| *e != dragged).collect())
            .unwrap_or_default();
        let mut desired: Vec<ph2d_ecs::Entity> = Vec::with_capacity(current.len() + 1);
        let mut inserted = false;
        for &c in &current {
            if !inserted && c == target && place_before {
                desired.push(dragged);
                inserted = true;
            }
            desired.push(c);
            if !inserted && c == target && !place_before {
                desired.push(dragged);
                inserted = true;
            }
        }
        if !inserted {
            desired.push(dragged);
        }
        for &child in &desired {
            if let Ok(mut entry) = sim_w.get_entity_mut(child) {
                entry.remove::<ph2d_ecs::ChildOf>();
                entry.insert(ph2d_ecs::ChildOf(parent));
            }
        }
        // ⭐ **E a ordem vira DADO** (ADR-0164 F1). O bloco acima reescreve a lista `Children`
        // do bevy, que é **memória de runtime**: ela não entra no snapshot, então até esta
        // wave reordenar irmãos não era desfazível, não sobrevivia a um Ctrl+Z de outra ação e
        // não sobrevivia ao save (classe BUGS #15, medida na auditoria de 21/08).
        //
        // ⚠️ **As duas escritas são precisas.** O `ChildOf` acima é o que o bevy usa AGORA (o
        // pai e a relação); o `SiblingOrder` aqui é o que o FICHEIRO guarda, e é dele que o
        // `world_to_snapshot` e a árvore do painel leem a ordem. Tirar uma das duas dá metade
        // do gesto: sem a de cima o objeto não muda de pai; sem esta ele volta ao lugar antigo
        // no próximo Ctrl+Z.
        //
        // ⚠️ Escreve com comparação (o `set_sibling_order` só toca no que difere): reescrever
        // o mesmo número em todo irmão marcaria o arquétipo de cada um como mudado, e o diff
        // do undo registaria um passo espúrio por quadro em que o gesto corresse.
        ph2d_ecs::set_sibling_order(sim_w, parent, &desired);
    }
    // Re-solve the dragged entity's LOCAL transform so its captured world
    // transform survives the parent change. The new parent chain is now in
    // place, so `parent_world_transform` reflects the FINAL parent.
    // `new_local = parent_world⁻¹ ∘ old_world`: translation via the canonical
    // world→local inverse, rotation/scale/skew by subtraction/division
    // (mirrors the additive skew cascade in `Transform::compose`). The
    // translation inverse drops parent skew — matching every other gizmo
    // helper; sprites carry skew only on leaves, never on rig ancestors.
    if let Some(old_world) = old_world {
        let new_parent = ph2d_ecs::parent_world_transform(sim_w, dragged);
        let np = ph2d_editor::TransformSnapshot {
            translation: [new_parent.translation.x, new_parent.translation.y],
            rotation: new_parent.rotation,
            scale: [new_parent.scale.x, new_parent.scale.y],
        };
        let local_t = ph2d_editor::world_translation_to_local(
            np,
            [old_world.translation.x, old_world.translation.y],
        );
        let psx = if new_parent.scale.x.abs() > 1e-6 {
            new_parent.scale.x
        } else {
            1.0
        };
        let psy = if new_parent.scale.y.abs() > 1e-6 {
            new_parent.scale.y
        } else {
            1.0
        };
        if let Some(mut t) = sim_w.get_mut::<Transform>(dragged) {
            t.translation = ph2d_core::Vec2::new(local_t[0], local_t[1]);
            t.rotation = old_world.rotation - new_parent.rotation;
            t.scale = ph2d_core::Vec2::new(old_world.scale.x / psx, old_world.scale.y / psy);
            t.skew_x = old_world.skew_x - new_parent.skew_x;
            t.skew_y = old_world.skew_y - new_parent.skew_y;
        }
    }
    // ⭐ **E o mesmo para a peça de MODELAGEM 3D** (W30) — a cadeia de pais nova já está no sítio, e
    // a pose local é re-resolvida para o mundo capturado sobreviver.
    if let Some(w) = old_field_world {
        ph2d_field_ecs::set_world_xform(sim_w, dragged, w);
    }
    // **LARGAR UM SPRITE DENTRO DE UMA FOLHA PÕE-NO DENTRO DELA** (Enio 2026-08-19: *"Para
    // acrescentar um sprite numa sheet o usuário deve colocá-la como filha da sheet na hierarquia.
    // Se assim fizer automaticamente a sprite deve se deslocar para dentro das fronteiras"*).
    //
    // ⚠️ **Isto ANULA de propósito a preservação de mundo** que as linhas acima acabam de fazer, e
    // só para este pai. A preservação existe porque um sprite não deve saltar ao ganhar um pai —
    // mas a folha é o caso em que ele DEVE: ficar onde estava significaria ficar fora da folha de
    // que acabou de ser feito filho, e o artista veria a hierarquia dizer uma coisa e o canvas
    // outra. *A regra geral está certa; a folha é a excepção que se nomeia, não que se esconde.*
    //
    // A ordem também é de propósito: confinar DEPOIS de a pose local estar re-resolvida, senão
    // confinaríamos a pose antiga, do pai anterior.
    crate::sheet_bounds::confine(sim, dragged);
    false
}
