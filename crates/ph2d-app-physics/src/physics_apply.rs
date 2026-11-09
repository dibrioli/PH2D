//! §11 Physics Body — the ECS write an edit turns into (ADR-0131 D8).
//!
//! Split from the sibling `inspector_physics` (the snapshot BUILDER) when the
//! material-combine arm (W-Material) pushed that file past the shell's 600-LOC
//! cap. `inspector_physics` re-exports [`apply_physics_edit`], so every caller
//! keeps its `inspector_physics::apply_physics_edit` path. Build reads; this
//! writes — the two halves of the section, one question each.

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor_core::PhysicsFieldEdit;

use crate::physics_area::apply_area_edit;
use crate::physics_surface::apply_surface_edit;
use ph2d_inspector_ordering::{queue_remove, queue_set};
// A caixa que casa com o desenho — a porta vive no irmão `_seed`, com quem a semeia.
use crate::physics_seed::sprite_half_extents;

/// Apply one [`PhysicsFieldEdit`] (§11).
///
/// **`Add` derives the collider from the sprite's own bounds** (Unity/Godot fit the box to the renderer too);
/// a sprite-less entity falls back to a half-metre box.
pub fn apply_physics_edit(
    sim: &mut SimWorld,
    entity_bits: u64,
    edit: PhysicsFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    use ph2d_physics_ecs::{
        BodyKind, Collider, ColliderShape, CombineRule, DampMode, DampingOverride, Dominance,
        GravityScale, InitialVelocity, MassOverride, MaterialCombine, RigidBody,
    };
    const RIGID_BODY: &str = "ph2d::physics::RigidBody";
    const COLLIDER: &str = "ph2d::physics::Collider";
    const GRAVITY_SCALE: &str = "ph2d::physics::GravityScale";
    const INITIAL_VELOCITY: &str = "ph2d::physics::InitialVelocity";
    const MASS_OVERRIDE: &str = "ph2d::physics::MassOverride";
    const DOMINANCE: &str = "ph2d::physics::Dominance";
    const MATERIAL_COMBINE: &str = "ph2d::physics::MaterialCombine";
    const DAMPING_OVERRIDE: &str = "ph2d::physics::DampingOverride";

    // Num corpo, a edição da forma vai à 1.ª forma, ligada ou não (opção B): nunca o `Collider` nele.
    let pedido = Entity::from_bits(entity_bits);
    if sim.world().get_entity(pedido).is_err() {
        return; // um «Remove» espalhado já levou esta forma com o corpo dela
    }
    let forma = edit.e_da_forma() && sim.world().get::<RigidBody>(pedido).is_some();
    let entity_bits = match ph2d_physics_ecs::forma_para_editar(sim.world(), pedido) {
        Some(s) if forma => s.to_bits(),
        _ => entity_bits,
    };
    let entity = Entity::from_bits(entity_bits);
    // O «Add Shape» de um CORPO sem forma: a filha «Collision Shape» (opção B), nunca o `Collider` nele.
    if matches!(edit, PhysicsFieldEdit::AddShape) && sim.world().get::<RigidBody>(entity).is_some()
    {
        crate::physics_seed::forma_que_falta(sim.world_mut(), entity);
        return;
    }
    if crate::physics_colide::apply_collides_edit(sim.world(), entity, edit, queue, registry) {
        return;
    }
    if matches!(edit, PhysicsFieldEdit::MoveShapeToChild) {
        let _ = ph2d_physics_ecs::move_forma_para_filha(sim.world_mut(), entity);
        return;
    }
    let world = sim.world();

    // *Add Body* e *Make Independent Body*: a forma em filha (opção B), a autorada intacta (W-PartAdopt).
    if matches!(edit, PhysicsFieldEdit::Add) {
        let half = sprite_half_extents(world, entity);
        let imagem = world.get::<ph2d_render::Sprite>(entity).is_some();
        let w = sim.world_mut();
        ph2d_physics_ecs::torna_corpo(w, entity, BodyKind::Dynamic, half, imagem);
        return;
    }
    // A caixa do sprite sem tocar no `RigidBody`: peça de um corpo acima (W-Compound).
    if matches!(edit, PhysicsFieldEdit::AddShape) {
        let half = sprite_half_extents(world, entity);
        queue_set(
            queue,
            registry,
            entity_bits,
            COLLIDER,
            &Collider {
                shape: ColliderShape::Cuboid {
                    half_x: half[0],
                    half_y: half[1],
                },
                ..Collider::default()
            },
        );
        return;
    }
    if matches!(edit, PhysicsFieldEdit::FitToPicture) {
        if let Some(cur) = world.get::<Collider>(entity).copied()
            && let Some(imagem) = imagem_da_forma(world, entity)
        {
            let mut next = Collider {
                shape: cur.shape_fitted_to(imagem.size),
                offset: [0.0, 0.0],
                ..cur
            };
            next.sanitize();
            queue_set(queue, registry, entity_bits, COLLIDER, &next);
        }
        return;
    }
    if matches!(edit, PhysicsFieldEdit::Remove) {
        if world.get::<RigidBody>(pedido).is_some() {
            let imagem = |w: &ph2d_ecs::World, e| w.get::<ph2d_render::Sprite>(e).is_some();
            ph2d_physics_ecs::tira_corpo(sim.world_mut(), pedido, imagem);
        } else {
            queue_remove(queue, registry, entity_bits, RIGID_BODY);
            queue_remove(queue, registry, entity_bits, COLLIDER);
        }
        return;
    }
    if let PhysicsFieldEdit::Kind(tag) = edit {
        // An unknown tag means the chip list and the enum drifted: dropped, never folded onto `Dynamic`.
        let Some(kind) = BodyKind::from_tag(tag) else {
            debug_assert!(
                false,
                "§11 sent BodyKind tag {tag}, which no variant claims"
            );
            return;
        };
        // ⚠️ Only a body can change KIND: otherwise this arm would attach an orphan `RigidBody` no gesture removes.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        queue_set(
            queue,
            registry,
            entity_bits,
            RIGID_BODY,
            &RigidBody { kind },
        );
        return;
    }
    if let PhysicsFieldEdit::GravityScale(v) = edit {
        // A RigidBody-level property (the optional `GravityScale` component),
        // not a Collider field — so it is handled here, like `Kind`, before the
        // collider block below. Gated on a live body: without one there is
        // nothing to scale, and this would be a second arm that attaches an
        // orphan component to a plain sprite.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        // Detach at the neutral 1.0 so an unscaled body carries no component
        // (the presence-override idiom: absent = default, and a project file
        // stays free of no-op `1.0`s). Any other value attaches/updates.
        if v == GravityScale::NEUTRAL {
            queue_remove(queue, registry, entity_bits, GRAVITY_SCALE);
        } else {
            queue_set(
                queue,
                registry,
                entity_bits,
                GRAVITY_SCALE,
                &GravityScale(v),
            );
        }
        return;
    }
    if let PhysicsFieldEdit::LinvelX(_)
    | PhysicsFieldEdit::LinvelY(_)
    | PhysicsFieldEdit::Angvel(_) = edit
    {
        // Initial velocity (W9): another RigidBody-level, optional
        // presence-override component, so it is handled here beside gravity.
        // Read-modify-write the ONE axis the edit names — a partial write would
        // drop the other two — off the current component or REST if absent.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        let mut iv = world
            .get::<InitialVelocity>(entity)
            .copied()
            .unwrap_or(InitialVelocity::REST);
        match edit {
            PhysicsFieldEdit::LinvelX(v) => iv.linvel[0] = v,
            PhysicsFieldEdit::LinvelY(v) => iv.linvel[1] = v,
            PhysicsFieldEdit::Angvel(v) => iv.angvel = v,
            _ => unreachable!(),
        }
        // Detach at rest so a still body carries no component — the same
        // idiom as gravity, and it keeps the project file free of no-op zeros.
        if iv.is_rest() {
            queue_remove(queue, registry, entity_bits, INITIAL_VELOCITY);
        } else {
            queue_set(queue, registry, entity_bits, INITIAL_VELOCITY, &iv);
        }
        return;
    }
    // The five flags whose PRESENCE is their value (CCD, Freeze Rotation, Freeze
    // Position X/Y, One-Way). They differed only in a name and a unit struct, so they
    // live together in one table now — see `inspector_physics_markers`.
    if crate::physics_markers::apply_marker_edit(world, entity, entity_bits, edit, queue, registry)
    {
        return;
    }

    if let PhysicsFieldEdit::MassMode(manual) = edit {
        // Mass source (W-Mass): another RigidBody-level, optional presence-override
        // component (`MassOverride`), handled here beside the constraints. Gated on a
        // shape, live or not: a body with none is not in the solver.
        let viva = ph2d_physics_ecs::main_shape(world, entity)
            .and_then(|f| world.get::<Collider>(f).copied());
        if viva.is_none() && !ph2d_physics_ecs::tem_forma(world, entity) {
            return;
        }
        if manual {
            // Auto → Manual: seed the override with the mass the solver gives the body NOW so the
            // number does not jump (Unity does the same). Every shape off: `MASS_WITHOUT_SHAPE_KG`.
            let Some(col) = viva else {
                let agora = world
                    .get::<MassOverride>(entity)
                    .map_or(ph2d_physics_ecs::MASS_WITHOUT_SHAPE_KG, |m| m.0);
                queue_set(
                    queue,
                    registry,
                    entity_bits,
                    MASS_OVERRIDE,
                    &MassOverride(agora),
                );
                return;
            };
            // Live shapes: the AUTO mass, SUMMING the parts (W-PartMass: a two-half raft seeded
            // `0,600` against `1,200`). `QueryState::try_new` keeps the `&SimWorld` read pass.
            let candidates: Vec<Entity> = {
                let mut q = bevy_ecs::query::QueryState::<
                    Entity,
                    (
                        bevy_ecs::query::With<Collider>,
                        bevy_ecs::query::Without<ph2d_physics_ecs::RigidBody>,
                    ),
                >::try_new(world)
                .expect("query");
                q.iter(world).collect()
            };
            let seed =
                ph2d_physics_ecs::auto_mass_with_parts(sim.world(), entity, &col, candidates);
            queue_set(
                queue,
                registry,
                entity_bits,
                MASS_OVERRIDE,
                &MassOverride(seed),
            );
        } else {
            // Manual → Auto: drop the override so the body weighs density × area again
            // (the presence-override idiom — absent = the engine default).
            queue_remove(queue, registry, entity_bits, MASS_OVERRIDE);
        }
        return;
    }
    if let PhysicsFieldEdit::Mass(v) = edit {
        // The explicit mass value (Manual mode only). Clamp positive — a zero or
        // negative mass is degenerate. Gated on the override already existing (the
        // Mass row is painted only in Manual mode); a stale event must not attach an
        // override while the body is in Auto mode.
        if world.get::<MassOverride>(entity).is_none() {
            return;
        }
        queue_set(queue, registry, entity_bits, MASS_OVERRIDE, &{
            let mut m = MassOverride(v);
            m.sanitize();
            m
        });
        return;
    }
    if let PhysicsFieldEdit::Dominance(d) = edit {
        // Dominance (W-Dominance): a RigidBody-level valued override, handled here
        // beside gravity/mass. Gated on a live body: without one there is nothing to
        // prioritise, and this would attach an orphan component to a plain sprite.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        // Detach at the neutral 0 so a neutral body carries no component (the
        // presence-override idiom — a project file stays free of no-op zeros); any
        // other value attaches/updates.
        if d == 0 {
            queue_remove(queue, registry, entity_bits, DOMINANCE);
        } else {
            queue_set(queue, registry, entity_bits, DOMINANCE, &Dominance(d));
        }
        return;
    }
    if let PhysicsFieldEdit::RestitutionCombine(tag) | PhysicsFieldEdit::FrictionCombine(tag) = edit
    {
        // Collision-material combine (W-Material): read-modify-write the ONE rule the
        // edit names on the optional `MaterialCombine` component (a partial write
        // would drop the other). Gated on a live body: without one there is nothing
        // to give a material to, and this would attach an orphan to a plain sprite.
        // NOT Dynamic-only — a static floor's combine rule matters too.
        if !ph2d_physics_ecs::tem_corpo(world, entity) {
            return;
        }
        // A tag no rule claims is DROPPED, not folded onto Average — the chip that
        // sent it is the only thing that mints one, so an unknown value means the
        // chip list and the enum drifted (the `BodyKind::from_tag` discipline).
        let Some(rule) = CombineRule::from_tag(tag) else {
            debug_assert!(
                false,
                "§11 sent CombineRule tag {tag}, which no variant claims"
            );
            return;
        };
        let mut mat = world
            .get::<MaterialCombine>(entity)
            .copied()
            .unwrap_or_default();
        match edit {
            PhysicsFieldEdit::RestitutionCombine(_) => mat.restitution = rule,
            PhysicsFieldEdit::FrictionCombine(_) => mat.friction = rule,
            _ => unreachable!(),
        }
        // Detach at neutral (both Average) so a default material carries no component
        // — the presence-override idiom, keeping a project file free of the no-op.
        if mat.is_neutral() {
            queue_remove(queue, registry, entity_bits, MATERIAL_COMBINE);
        } else {
            queue_set(queue, registry, entity_bits, MATERIAL_COMBINE, &mat);
        }
        return;
    }
    // **A superfície de caminhada** (W-Surface) — no módulo IRMÃO, pela mesma
    // linha de corte que separou as zonas: *o que este CORPO é* × *o que esta
    // ÁREA faz a outros* × **de que esta SUPERFÍCIE é feita para quem anda**.
    // ⭐ O FILTRO por tag dos sinais (TOP-20 #9, W3c) — irmão do de superfície, e pela mesma razão.
    if crate::physics_surface::apply_signal_tag_filter_edit(
        world,
        entity,
        entity_bits,
        edit,
        queue,
        registry,
    ) {
        return;
    }
    if apply_surface_edit(world, entity, entity_bits, edit, queue, registry) {
        return;
    }
    if let PhysicsFieldEdit::LinearDamping(_)
    | PhysicsFieldEdit::AngularDamping(_)
    | PhysicsFieldEdit::DampMode(_) = edit
    {
        // Per-body damping (W-Damping): read-modify-write the ONE field the edit names
        // on the optional `DampingOverride` component (a partial write would drop the
        // others). Gated on a live body: without one there is nothing to damp, and this
        // would attach an orphan to a plain sprite.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        let mut d = world
            .get::<DampingOverride>(entity)
            .copied()
            .unwrap_or_default();
        match edit {
            // Drag coefficients are non-negative (a negative drag would ADD energy).
            PhysicsFieldEdit::LinearDamping(v) => d.linear = v,
            PhysicsFieldEdit::AngularDamping(v) => d.angular = v,
            PhysicsFieldEdit::DampMode(tag) => {
                // A tag no mode claims is DROPPED, not folded onto Combine (the
                // `BodyKind::from_tag` discipline).
                let Some(mode) = DampMode::from_tag(tag) else {
                    debug_assert!(
                        false,
                        "§11 sent DampMode tag {tag}, which no variant claims"
                    );
                    return;
                };
                d.mode = mode;
            }
            _ => unreachable!(),
        }
        d.sanitize();
        // Detach at neutral (zero drag AND Combine) so a body on the world default
        // carries no component — the presence-override idiom. `Replace` with zero drag
        // is NOT neutral (it forces zero damping, ignoring a world drag), so it stays.
        if d.is_neutral() {
            queue_remove(queue, registry, entity_bits, DAMPING_OVERRIDE);
        } else {
            queue_set(queue, registry, entity_bits, DAMPING_OVERRIDE, &d);
        }
        return;
    }

    // As sete rows da ZONA (W-Area .. W-AreaFalloff) — o que esta ÁREA faz a OUTROS corpos,
    // em vez de o que ESTE corpo é. É a mesma linha de corte que separa `components/area.rs`
    // de `components/overrides.rs`, e é aqui que a próxima wave de zona aterrissa.
    if apply_area_edit(world, entity, entity_bits, edit, queue, registry) {
        return;
    }

    // Everything else edits the collider, so read the live one and write it
    // back changed — a partial write would drop the fields not being edited.
    let Some(cur) = world.get::<Collider>(entity).copied() else {
        return;
    };
    let mut next = cur;
    match edit {
        // Not a field edit at all: creating a joint is one gesture over a
        // PAIR, and it is intercepted before the per-entity fan-out that
        // brings us here. Reaching this arm would mean the interception was
        // removed — and the fan-out would then create one joint per selected
        // body instead of one joint.
        // Not field edits: gestures over the SELECTION or app-state options, intercepted
        // before the per-entity fan-out (fanned out, Join/Bake/Rig would run once per body).
        PhysicsFieldEdit::Join
        | PhysicsFieldEdit::Bake
        | PhysicsFieldEdit::BakeChannels(_)
        | PhysicsFieldEdit::JoinKind(_)
        | PhysicsFieldEdit::JoinDraw
        | PhysicsFieldEdit::Rig
        | PhysicsFieldEdit::EditShapeOnCanvas => {}
        // Switching shape PRESERVES the footprint (box ↔ the ball inside it): no size jump.
        PhysicsFieldEdit::Shape(0) => {
            let r = match cur.shape {
                ColliderShape::Ball { radius } => radius,
                ColliderShape::Cuboid { half_x, half_y }
                | ColliderShape::Outline { half_x, half_y } => half_x.min(half_y),
                ColliderShape::Capsule { radius, .. } => radius, // the cap IS this ball
            };
            next.shape = ColliderShape::Ball {
                radius: r.max(1e-3),
            };
        }
        PhysicsFieldEdit::Shape(1) => {
            let (hx, hy) = match cur.shape {
                ColliderShape::Ball { radius } => (radius, radius),
                ColliderShape::Cuboid { half_x, half_y }
                | ColliderShape::Outline { half_x, half_y } => (half_x, half_y),
                // The box covering the capsule: caps wide, TOTAL half-extent tall.
                ColliderShape::Capsule {
                    half_height,
                    radius,
                } => (radius, half_height + radius),
            };
            next.shape = ColliderShape::Cuboid {
                half_x: hx.max(1e-3),
                half_y: hy.max(1e-3),
            };
        }
        PhysicsFieldEdit::Shape(2) => {
            let (half_height, radius) = match cur.shape {
                // A zero-segment capsule is exactly that ball: the silhouette stays.
                ColliderShape::Ball { radius } => (0.0, radius),
                // Radius from the SMALLER half-extent (never wider than the box), the segment
                // takes the rest: the total half-extent lands back on `half_y` exactly.
                ColliderShape::Cuboid { half_x, half_y }
                | ColliderShape::Outline { half_x, half_y } => {
                    let r = half_x.min(half_y);
                    ((half_y - r).max(0.0), r)
                }
                ColliderShape::Capsule {
                    half_height,
                    radius,
                } => (half_height, radius),
            };
            next.shape = ColliderShape::Capsule {
                half_height: half_height.max(0.0),
                radius: radius.max(1e-3),
            };
        }
        // O contorno nasce na caixa de agora; os anéis vêm do traço (`ph2d_physics_outline::retrace_edited`).
        PhysicsFieldEdit::Shape(3) => {
            let [half_x, half_y] = cur.local_aabb().1.map(|h| h.max(1e-3));
            next.shape = ColliderShape::Outline { half_x, half_y };
        }
        // A tag no shape claims is DROPPED, never folded onto a plausible neighbour (as `Kind`):
        // a catch-all would be a chip that quietly selects a different shape.
        PhysicsFieldEdit::Shape(_) => {}
        // ⚠️ Radius must not FORCE a ball: a capsule's cap radius is the same
        // quantity under the same name, so on a capsule this edits the caps and
        // leaves it a capsule. Forcing `Ball` here would delete the artist's
        // capsule the first time they touched its radius.
        PhysicsFieldEdit::Radius(v) => {
            next.shape = match cur.shape {
                ColliderShape::Capsule { half_height, .. } => ColliderShape::Capsule {
                    half_height,
                    radius: v,
                },
                _ => ColliderShape::Ball {
                    radius: v,
                },
            };
        }
        // Only meaningful on a capsule, and the row is painted only there.
        PhysicsFieldEdit::CapHalfHeight(v) => {
            if let ColliderShape::Capsule { radius, .. } = cur.shape {
                next.shape = ColliderShape::Capsule {
                    half_height: v,
                    radius,
                };
            }
        }
        PhysicsFieldEdit::HalfX(v) => {
            if let ColliderShape::Cuboid { half_y, .. } = cur.shape {
                next.shape = ColliderShape::Cuboid {
                    half_x: v,
                    half_y,
                };
            }
        }
        PhysicsFieldEdit::HalfY(v) => {
            if let ColliderShape::Cuboid { half_x, .. } = cur.shape {
                next.shape = ColliderShape::Cuboid {
                    half_x,
                    half_y: v,
                };
            }
        }
        // Collider offset — a signed position, no clamp; writes the ONE axis the edit names.
        PhysicsFieldEdit::ShapeDrag { medidas, offset } => {
            next.offset = offset;
            next.shape = match cur.shape {
                ColliderShape::Cuboid { .. } => ColliderShape::Cuboid {
                    half_x: medidas[0],
                    half_y: medidas[1],
                },
                ColliderShape::Outline { .. } => ColliderShape::Outline {
                    half_x: medidas[0],
                    half_y: medidas[1],
                },
                ColliderShape::Ball { .. } => ColliderShape::Ball { radius: medidas[0] },
                ColliderShape::Capsule { .. } => ColliderShape::Capsule {
                    half_height: medidas[1],
                    radius: medidas[0],
                },
            };
        }
        PhysicsFieldEdit::OffsetX(v) => next.offset[0] = v,
        PhysicsFieldEdit::OffsetY(v) => next.offset[1] = v,
        PhysicsFieldEdit::Density(v) => next.density = v,
        PhysicsFieldEdit::Restitution(v) => next.restitution = v,
        PhysicsFieldEdit::Friction(v) => next.friction = v,
        // Clamped (by `Collider::sanitize`, below) to the layers that exist: `groups_for`
        // would silently fold an overflow onto the last layer.
        PhysicsFieldEdit::Layer(n) => next.layer = n,
        PhysicsFieldEdit::Sensor(s) => next.is_sensor = s,
        PhysicsFieldEdit::CollidesWith { .. } | PhysicsFieldEdit::UseLayerDefault => unreachable!("above"),
        PhysicsFieldEdit::Add
        | PhysicsFieldEdit::AddShape
        | PhysicsFieldEdit::MoveShapeToChild
        | PhysicsFieldEdit::Remove
        | PhysicsFieldEdit::FitToPicture
        | PhysicsFieldEdit::Kind(_)
        | PhysicsFieldEdit::GravityScale(_)
        | PhysicsFieldEdit::LinvelX(_)
        | PhysicsFieldEdit::LinvelY(_)
        | PhysicsFieldEdit::Angvel(_)
        | PhysicsFieldEdit::Ccd(_)
        | PhysicsFieldEdit::LockRotation(_)
        | PhysicsFieldEdit::LockPositionX(_)
        | PhysicsFieldEdit::LockPositionY(_)
        | PhysicsFieldEdit::MassMode(_)
        | PhysicsFieldEdit::Mass(_)
        | PhysicsFieldEdit::Dominance(_)
        | PhysicsFieldEdit::RestitutionCombine(_)
        | PhysicsFieldEdit::FrictionCombine(_)
        | PhysicsFieldEdit::LinearDamping(_)
        | PhysicsFieldEdit::AngularDamping(_)
        | PhysicsFieldEdit::DampMode(_)
        | PhysicsFieldEdit::OneWay(_)
        | PhysicsFieldEdit::NoWallCling(_)
        | PhysicsFieldEdit::ForceWorldAxes(_)
        | PhysicsFieldEdit::ForceX(_)
        | PhysicsFieldEdit::ForceY(_)
        | PhysicsFieldEdit::AreaDrag(_)
        | PhysicsFieldEdit::AreaDensity(_)
        | PhysicsFieldEdit::AreaFormDrag(_)
        | PhysicsFieldEdit::AreaTorque(_)
        | PhysicsFieldEdit::AreaFalloff(_)
        | PhysicsFieldEdit::WalkGrip(_)
        | PhysicsFieldEdit::WalkBelt(_)
        // ⭐ O filtro por tag é escrito pelo irmão do de superfície, acima.
        | PhysicsFieldEdit::SignalTagFilter(_) => {
            unreachable!("handled above")
        }
    }
    next.sanitize();
    if next != cur {
        queue_set(queue, registry, entity_bits, COLLIDER, &next);
    }
}

/// A imagem que a forma `e` cobre (opção B) — a lei mora no produtor do «Outline».
pub(crate) fn imagem_da_forma(world: &ph2d_ecs::World, e: Entity) -> Option<&ph2d_render::Sprite> {
    ph2d_physics_outline::picture_owner(world, e).and_then(|o| world.get(o))
}
