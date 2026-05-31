//! Inspector §8 (Visibility section) snapshot + commit handler — Sprite
//! Inspector v2 W3. Split from `inspector_ordering.rs` to keep both under
//! the HR-18 600-LOC shell cap (handoff §2.1 step 6).
//!
//! Each [`VisibilityFieldEdit`] maps to an OPTIONAL ECS component
//! (presence = override, spec §02 / §6): `VisibilityLayer` (absent → all
//! 32 layers), `ClipChildren`, `MaskInteraction`, `OnScreenEnabler`. A
//! `0` / `Disabled` / `None` / all-layers value detaches the component.
//! Reuses `inspector_ordering`'s `queue_set` / `queue_remove`; the caller
//! applies the queued commands via `apply_editor_commands`.

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{
    ClipChildren, ClipMode, Entity, Mask2D, MaskInteraction, MaskMode, OnScreenEnabler, SimWorld,
    VisibilityLayer, World,
};
use ph2d_editor::{InspectorVisibilityMixed, InspectorVisibilitySectionInfo, VisibilityFieldEdit};

use super::inspector_ordering::{queue_remove, queue_set};

fn clip_mode_tag(m: ClipMode) -> u8 {
    match m {
        ClipMode::Disabled => 0,
        ClipMode::ClipOnly => 1,
        ClipMode::ClipAndDraw => 2,
    }
}

fn clip_mode_from_tag(t: u8) -> ClipMode {
    match t {
        1 => ClipMode::ClipOnly,
        2 => ClipMode::ClipAndDraw,
        _ => ClipMode::Disabled,
    }
}

fn mask_mode_tag(m: MaskMode) -> u8 {
    match m {
        MaskMode::None => 0,
        MaskMode::VisibleInside => 1,
        MaskMode::VisibleOutside => 2,
    }
}

fn mask_mode_from_tag(t: u8) -> MaskMode {
    match t {
        1 => MaskMode::VisibleInside,
        2 => MaskMode::VisibleOutside,
        _ => MaskMode::None,
    }
}

/// The §8 visibility fields of one entity (absent component → its
/// canonical default). Shared by the snapshot producer + the BulkSelect
/// compare so the two never drift.
type VisFields = (u32, u8, u8, f32, bool, bool, [f32; 4]);

fn visibility_fields(world: &World, entity: Entity) -> VisFields {
    let layer_mask = world
        .get::<VisibilityLayer>(entity)
        .map_or(VisibilityLayer::ALL, |v| v.0);
    let clip_mode = world
        .get::<ClipChildren>(entity)
        .map_or(0, |c| clip_mode_tag(c.mode));
    let mask = world.get::<MaskInteraction>(entity).copied();
    let mask_mode = mask.map_or(0, |m| mask_mode_tag(m.mode));
    let alpha_cutoff = mask.map_or(MaskInteraction::DEFAULT_CUTOFF, |m| m.alpha_cutoff);
    let mask_source = world.get::<Mask2D>(entity).is_some();
    let on = world.get::<OnScreenEnabler>(entity).copied();
    (
        layer_mask,
        clip_mode,
        mask_mode,
        alpha_cutoff,
        mask_source,
        on.is_some(),
        on.map_or([0.0; 4], |o| o.rect),
    )
}

/// Build the §8 visibility-section snapshot, or `None` when the entity
/// has no `Transform` (not Inspector-worthy).
#[allow(clippy::float_cmp)] // exact compare: same stored value = not mixed
pub(super) fn build_visibility_section_info(
    world: &World,
    entity_bits: u64,
    selected: &[u64],
    selected_count: usize,
) -> Option<InspectorVisibilitySectionInfo> {
    let entity = Entity::from_bits(entity_bits);
    world.get::<ph2d_ecs::Transform>(entity)?;
    let (layer_mask, clip_mode, mask_mode, alpha_cutoff, mask_source, on_screen, rect) =
        visibility_fields(world, entity);
    let mut mixed = InspectorVisibilityMixed::default();
    if selected.len() > 1 {
        for &bits in selected {
            let f = visibility_fields(world, Entity::from_bits(bits));
            mixed.layer_mask |= f.0 != layer_mask;
            mixed.clip_mode |= f.1 != clip_mode;
            mixed.mask_mode |= f.2 != mask_mode;
            mixed.alpha_cutoff |= f.3 != alpha_cutoff;
            mixed.mask_source |= f.4 != mask_source;
            mixed.on_screen |= f.5 != on_screen;
            mixed.rect |= f.6 != rect;
        }
    }
    Some(InspectorVisibilitySectionInfo {
        entity_bits,
        layer_mask,
        clip_mode,
        mask_mode,
        alpha_cutoff,
        mask_source,
        on_screen,
        rect,
        selected_count,
        mixed,
    })
}

/// Apply one [`VisibilityFieldEdit`] (§8) by queueing the right
/// `SetComponent` / `RemoveComponent` on the optional component it maps
/// to (presence = override).
pub(super) fn apply_visibility_section_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: VisibilityFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    match edit {
        VisibilityFieldEdit::LayerBit(n, on) => {
            let cur = sim
                .world()
                .get::<VisibilityLayer>(entity)
                .map_or(VisibilityLayer::ALL, |v| v.0);
            let next = if on {
                cur | (1u32 << n as u32)
            } else {
                cur & !(1u32 << n as u32)
            };
            // Absence == ALL: detach when the mask is back to every layer.
            if next == VisibilityLayer::ALL {
                queue_remove(queue, registry, entity_bits, "ph2d::ecs::VisibilityLayer");
            } else {
                queue_set(
                    queue,
                    registry,
                    entity_bits,
                    "ph2d::ecs::VisibilityLayer",
                    &VisibilityLayer(next),
                );
            }
        }
        VisibilityFieldEdit::ClipMode(0) => {
            queue_remove(queue, registry, entity_bits, "ph2d::ecs::ClipChildren")
        }
        VisibilityFieldEdit::ClipMode(t) => queue_set(
            queue,
            registry,
            entity_bits,
            "ph2d::ecs::ClipChildren",
            &ClipChildren::new(clip_mode_from_tag(t)),
        ),
        VisibilityFieldEdit::MaskMode(0) => {
            queue_remove(queue, registry, entity_bits, "ph2d::ecs::MaskInteraction")
        }
        VisibilityFieldEdit::MaskMode(t) => {
            // Preserve the existing cutoff when switching mode.
            let cutoff = sim
                .world()
                .get::<MaskInteraction>(entity)
                .map_or(MaskInteraction::DEFAULT_CUTOFF, |m| m.alpha_cutoff);
            queue_set(
                queue,
                registry,
                entity_bits,
                "ph2d::ecs::MaskInteraction",
                &MaskInteraction {
                    mode: mask_mode_from_tag(t),
                    alpha_cutoff: cutoff,
                }
                .clamped(),
            )
        }
        VisibilityFieldEdit::AlphaCutoff(v) => {
            // RMW: keep the mode (no-op if no MaskInteraction yet).
            let mut m = sim
                .world()
                .get::<MaskInteraction>(entity)
                .copied()
                .unwrap_or_default();
            m.alpha_cutoff = v;
            queue_set(
                queue,
                registry,
                entity_bits,
                "ph2d::ecs::MaskInteraction",
                &m.clamped(),
            );
        }
        VisibilityFieldEdit::MaskSource(true) => queue_set(
            queue,
            registry,
            entity_bits,
            "ph2d::ecs::Mask2D",
            &Mask2D::new(),
        ),
        VisibilityFieldEdit::MaskSource(false) => {
            queue_remove(queue, registry, entity_bits, "ph2d::ecs::Mask2D")
        }
        VisibilityFieldEdit::OnScreen(true) => queue_set(
            queue,
            registry,
            entity_bits,
            "ph2d::ecs::OnScreenEnabler",
            &OnScreenEnabler::default(),
        ),
        VisibilityFieldEdit::OnScreen(false) => {
            queue_remove(queue, registry, entity_bits, "ph2d::ecs::OnScreenEnabler")
        }
        VisibilityFieldEdit::RectX(_)
        | VisibilityFieldEdit::RectY(_)
        | VisibilityFieldEdit::RectW(_)
        | VisibilityFieldEdit::RectH(_) => {
            let mut o = sim
                .world()
                .get::<OnScreenEnabler>(entity)
                .copied()
                .unwrap_or_default();
            match edit {
                VisibilityFieldEdit::RectX(v) => o.rect[0] = v,
                VisibilityFieldEdit::RectY(v) => o.rect[1] = v,
                VisibilityFieldEdit::RectW(v) => o.rect[2] = v,
                VisibilityFieldEdit::RectH(v) => o.rect[3] = v,
                _ => unreachable!(),
            }
            queue_set(
                queue,
                registry,
                entity_bits,
                "ph2d::ecs::OnScreenEnabler",
                &o,
            );
        }
    }
}
