//! §11 Physics Body — the snapshot the Inspector reads (ADR-0131 D8); the write is `physics_apply`.

use bevy_ecs::world::World;
use ph2d_ecs::Entity;
use ph2d_editor_core::InspectorPhysicsInfo;
use ph2d_physics_ecs::excecao::estado_da_selecao;

/// The WRITE half, in the sibling module. Re-exported so callers are unchanged.
pub use crate::physics_apply::apply_physics_edit;

/// **O nome do corpo ancestral mais próximo** (vazio sem nenhum), pelo MESMO walk da ponte
/// (`owner_body`): *Add Shape to X* e a face de peça nunca nomeiam outro dono.
fn nearest_body_name(world: &World, e: ph2d_ecs::Entity) -> String {
    ph2d_physics_ecs::owner_body(world, e).map_or_else(String::new, |p| {
        world.get::<ph2d_ecs::Name>(p).map_or_else(
            || ph2d_i18n::tr("app.physics.body.the_body_above").to_string(),
            |n| n.as_str().to_string(),
        )
    })
}

/// O «Collides with» diante da seleção `sel` (bits): misto e «Differs».
pub fn com_a_selecao(w: &World, mut i: InspectorPhysicsInfo, sel: &[u64]) -> InspectorPhysicsInfo {
    let sel: Vec<Entity> = sel.iter().copied().map(Entity::from_bits).collect();
    let s = estado_da_selecao(w, Entity::from_bits(i.entity_bits), &sel);
    (i.collides_mixed, i.collides_differs, i.collides_pinned) = (s.misto, s.difere, s.fixos);
    i
}

/// The §11 snapshot: `Some` even without a body (the *Add* face); each argument is a shell-only fact.
#[allow(clippy::too_many_arguments)]
pub fn build_physics_info(
    world: &World,
    entity_bits: u64,
    join_count: u8,
    // W-Rig: partes que um rig tocaria; `0` = o botão não é oferecido.
    rig_parts: u8,
    // W-PartFace: as PEÇAS penduradas neste corpo; só a shell conta (query sobre o mundo inteiro).
    part_count: u8,
    join_draw_armed: bool,
    join_kind_tag: u8,
    bake_range: (f32, f32),
    bake_channels_tag: u8,
    // ⭐ A árvore de tags (TOP-20 #9, W3c) — a row *Only for tag* mostra o CAMINHO do filtro, não o
    // número dele. ⚠️ O painel não conhece a árvore.
    tree: &ph2d_tags::TagTree,
) -> Option<InspectorPhysicsInfo> {
    let (bake_start_seconds, bake_seconds) = bake_range;
    use ph2d_physics_ecs::{
        AreaBuoyancy, AreaDrag, AreaEffector, AreaFalloff, AreaFormDrag, AreaTorque, Ccd, Collider,
        ColliderShape, DampingOverride, Dominance, GravityScale, InitialVelocity, LockPositionX,
        LockPositionY, LockRotation, MassOverride, MaterialCombine, OneWayPlatform, RigidBody,
        WalkSurface,
    };
    let entity = Entity::from_bits(entity_bits);
    world.get::<ph2d_ecs::Transform>(entity)?;
    let rb = world.get::<RigidBody>(entity);
    // Opção B: o lado-forma de um corpo lê-se da 1.ª forma (a filha «Collision Shape»), ligada ou
    // não — o olho fechado nunca esconde o cartão do corpo (dono 09/11).
    let forma = rb
        .and(ph2d_physics_ecs::forma_para_editar(world, entity))
        .unwrap_or(entity);
    let col = world.get::<Collider>(forma);
    // ⭐ A §11 aparece com um dos seus componentes ou com um gesto que só ela oferece (ADR-0166 /
    // F3): o `Collider` sozinho (peça de um corpo ancestral, W-PartFace), o `rig_parts` (o tronco do
    // Rig costuma não ter corpo nem collider) e um CORPO ACIMA na árvore (dono 2026-09-14: o botão
    // **Add Shape to X**, que a paleta genérica não sabe nomear).
    let part_owner = if rb.is_some() {
        // Um corpo não é peça de ninguém (docs do `nearest_body_name`).
        String::new()
    } else {
        nearest_body_name(world, entity)
    };
    if rb.is_none() && col.is_none() && rig_parts == 0 && part_owner.is_empty() {
        return None;
    }
    // Opção B (B4): o corpo de forma na filha só mostra o Body; a forma dele diz de quem é.
    let shape_on_child = rb.is_some() && forma != entity;
    let shape_of =
        if rb.is_none() && col.is_some() && world.get::<ph2d_render::Sprite>(entity).is_none() {
            part_owner.clone()
        } else {
            String::new()
        };
    // Optional per-body gravity multiplier (W8); absent = the neutral 1.0.
    let gravity_scale = world
        .get::<GravityScale>(entity)
        .map_or(GravityScale::NEUTRAL, |g| g.0);
    // Optional authored initial velocity (W9); absent = at rest.
    let iv = world
        .get::<InitialVelocity>(entity)
        .copied()
        .unwrap_or(InitialVelocity::REST);
    // Optional CCD marker (W-CCD); its presence is the flag, absent = discrete.
    let ccd = world.get::<Ccd>(entity).is_some();
    // Optional LockRotation marker (Freeze Rotation); presence is the flag.
    let lock_rotation = world.get::<LockRotation>(entity).is_some();
    // Optional Freeze Position markers (W-LockPos); each presence is a flag.
    let lock_x = world.get::<LockPositionX>(entity).is_some();
    let lock_y = world.get::<LockPositionY>(entity).is_some();
    // A superfície de caminhada (W-Surface); ausente = o neutro.
    let surface = world
        .get::<WalkSurface>(forma)
        .copied()
        .unwrap_or(WalkSurface::NEUTRAL);
    // Optional mass override (W-Mass): presence = Manual (kg); absent = Auto (density).
    let mass_ov = world.get::<MassOverride>(entity).map(|m| m.0);
    // Optional dominance (W-Dominance); absent = neutral 0.
    let dominance = world.get::<Dominance>(entity).map_or(0, |d| d.0);
    // Optional material combine (W-Material); absent = both Average; any body kind.
    let material = world
        .get::<MaterialCombine>(forma)
        .copied()
        .unwrap_or_default();
    // Optional damping override (W-Damping); absent = world drag; rows Dynamic-only in paint.
    let damping = world
        .get::<DampingOverride>(entity)
        .copied()
        .unwrap_or_default();
    // Optional OneWayPlatform marker (W-OneWay); presence is the flag, any body kind.
    let one_way = world.get::<OneWayPlatform>(forma).is_some();
    // Optional NoWallCling marker (W-WallMaterial), from the SAME entity as the collider.
    let no_wall_cling = world.get::<ph2d_physics_ecs::NoWallCling>(entity).is_some();
    // Optional AreaForceWorldAxes marker (W-AreaFrame): the zone's force on world axes.
    let force_world_axes = world
        .get::<ph2d_physics_ecs::AreaForceWorldAxes>(forma)
        .is_some();
    // Optional AreaEffector (W-Area); absent = pushes nothing. Rows are SENSOR-only in paint.
    let force = world
        .get::<AreaEffector>(forma)
        .map_or([0.0, 0.0], |a| a.force);
    // Optional AreaDrag (W-AreaDrag) — the medium half; Sensor-only in paint.
    let area_drag = world.get::<AreaDrag>(forma).map_or(0.0, |d| d.0);
    // Optional AreaBuoyancy (W-Buoyancy) — a densidade do fluido.
    let area_density = world.get::<AreaBuoyancy>(forma).map_or(0.0, |b| b.0);
    let area_form_drag = world.get::<AreaFormDrag>(forma).map_or(0.0, |f| f.0);
    // Optional AreaTorque (W-AreaTorque) — o giro que a área imprime; mesma condição de Sensor.
    let area_torque = world.get::<AreaTorque>(forma).map_or(0.0, |t| t.0);
    // Optional AreaFalloff (W-AreaFalloff) — o empurrão perde do centro à borda; idem.
    let area_falloff = world.get::<AreaFalloff>(forma).map_or(0.0, |f| f.0);
    // ⚠️ A pergunta é pelo COLLIDER, não pelo par (W-PartFace): uma peça é simulada.
    let Some(col) = col else {
        // The empty face; the dimensions are display seeds the panel never shows.
        return Some(InspectorPhysicsInfo {
            entity_bits,
            has_body: false,
            has_collider: false,
            body_without_shape: rb.is_some() && part_count == 0,
            shape_on_child: false,
            shape_of: String::new(),
            part_count: 0,
            no_wall_cling: false,
            bake_seconds,
            bake_start_seconds,
            kind_tag: 0,
            shape_tag: 1,
            radius: 0.5,
            half_x: 0.5,
            half_y: 0.5,
            density: 1.0,
            restitution: Collider::DEFAULT_RESTITUTION,
            friction: Collider::DEFAULT_FRICTION,
            layer: 0,
            collides_override: [0; 2],
            collides_mixed: 0,
            collides_differs: false,
            collides_pinned: 0,
            // Sem corpo não há joint; o rig (abaixo) CRIA os corpos, por isso fica.
            join_count: 0,
            rig_parts,
            // W-Compound: só a face VAZIA vira peça.
            part_owner,
            join_draw_armed,
            join_kind_tag,
            is_sensor: false,
            // Sem collider não há sinal nem filtro.
            signal: String::new(),
            signal_leave: String::new(),
            signal_tag: None,
            signal_tag_path: String::new(),
            bake_channels_tag,
            gravity_scale: GravityScale::NEUTRAL,
            cap_half_height: 0.25,
            shape_follows: None,
            outline: None,
            shape_edit_on: false,
            estica: Vec::new(),
            linvel: InitialVelocity::REST.linvel,
            angvel: InitialVelocity::REST.angvel,
            ccd: false,
            lock_rotation: false,
            offset: [0.0, 0.0],
            look: ph2d_editor_core::ShapeLookFacts::NONE,
            walk_grip: WalkSurface::NEUTRAL.grip,
            walk_belt: WalkSurface::NEUTRAL.belt,
            lock_x: false,
            lock_y: false,
            mass_manual: false,
            mass_is_read: false,
            mass: 1.0,
            dominance: 0,
            restitution_combine_tag: 0,
            friction_combine_tag: 0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            damp_mode_tag: 0,
            one_way: false,
            force: [0.0, 0.0],
            force_world_axes: false,
            area_drag: 0.0,
            area_density: 0.0,
            area_form_drag: 0.0,
            area_torque: 0.0,
            area_falloff: 0.0,
        });
    };
    // Each arm also seeds the OTHER shapes' rows (display only; `apply_physics_edit` converts).
    let caixa = |tag: u8, half_x: f32, half_y: f32| {
        let cap = (half_y - half_x).max(0.0);
        (tag, half_x.max(half_y), half_x, half_y, cap)
    };
    let (shape_tag, radius, half_x, half_y, cap_half_height) = match col.shape {
        ColliderShape::Ball { radius } => (0u8, radius, radius, radius, 0.0),
        ColliderShape::Cuboid { half_x, half_y } => caixa(1, half_x, half_y),
        ColliderShape::Outline { half_x, half_y } => caixa(3, half_x, half_y),
        // The capsule's box equivalent: `radius` wide, `half_height + radius` tall.
        ColliderShape::Capsule {
            half_height,
            radius,
        } => (2u8, radius, radius, half_height + radius, half_height),
    };
    Some(InspectorPhysicsInfo {
        entity_bits,
        has_body: rb.is_some(),
        has_collider: true,
        body_without_shape: false,
        shape_on_child,
        shape_of,
        // ⚠️ Só um CORPO tem peças penduradas nele; uma peça não hospeda peças.
        part_count: if rb.is_some() { part_count } else { 0 },
        // O tipo que governa a forma: o cartão Shape da filha esconde o One-way num Dynamic.
        kind_tag: ph2d_physics_ecs::governing_kind(world, entity).map_or(0, |k| k.tag()),
        shape_tag,
        radius,
        half_x,
        half_y,
        density: col.density,
        restitution: col.restitution,
        friction: col.friction,
        layer: col.layer,
        collides_override: ph2d_physics_ecs::excecao::mascaras(world, forma),
        collides_mixed: 0,
        // A seleção (`com_a_selecao`) diz se alguma forma difere.
        collides_differs: false,
        collides_pinned: 0,
        // ⚠️ Sem corpo não há metade de joint, seja qual for a seleção.
        join_count: if rb.is_some() { join_count } else { 0 },
        rig_parts,
        // Uma peça NOMEIA o dono no cabeçalho da própria face; um corpo não é peça.
        part_owner,
        join_draw_armed,
        join_kind_tag,
        bake_seconds,
        bake_start_seconds,
        is_sensor: col.is_sensor,
        // Os dois nomes AUTORADOS: sem eles as rows seriam write-only.
        signal: world
            .get::<ph2d_physics_ecs::SignalOnHit>(forma)
            .and_then(ph2d_physics_ecs::SignalOnHit::name)
            .unwrap_or_default()
            .to_string(),
        signal_leave: world
            .get::<ph2d_physics_ecs::SignalOnLeave>(forma)
            .and_then(ph2d_physics_ecs::SignalOnLeave::name)
            .unwrap_or_default()
            .to_string(),
        // ⭐ O FILTRO dos dois nomes acima. ⚠️ **`Some(0)` (por escolher) e o caminho VAZIO com um
        // id vivo são histórias diferentes** — ver o doc do `SignalTagFilter`.
        signal_tag: world
            .get::<ph2d_physics_ecs::SignalTagFilter>(forma)
            .map(|f| f.0),
        signal_tag_path: world
            .get::<ph2d_physics_ecs::SignalTagFilter>(forma)
            .and_then(|f| f.tag())
            .and_then(|id| tree.get(ph2d_tags::TagId(id)))
            .map(|t| t.path.clone())
            .unwrap_or_default(),
        bake_channels_tag,
        gravity_scale,
        cap_half_height,
        shape_follows: ph2d_physics_outline::follows(world, forma),
        outline: ph2d_physics_outline::card_info(world, forma),
        shape_edit_on: ph2d_shape_handles::editando(world) == Some(entity_bits),
        estica: super::estica::quem_estica(world, forma)
            .iter()
            .map(|e| e.to_bits())
            .collect(),
        linvel: iv.linvel,
        angvel: iv.angvel,
        ccd,
        lock_rotation,
        offset: col.offset,
        // O cartão Look só do objeto «Collision Shape» selecionado (um `Collider` sem corpo).
        look: ph2d_shape_handles::look::fatos(world, forma, col, forma == entity && rb.is_none()),
        // A superfície AUTORADA — ausente, o neutro. Sem estas duas linhas as
        // rows seriam write-only, o defeito que a família das zonas shipou
        // inteira e que custou um report do Enio.
        walk_grip: surface.grip,
        walk_belt: surface.belt,
        lock_x,
        lock_y,
        // Manual mode = the override is present; its value is shown in the Mass row.
        // In Auto mode the Mass row is not shown, so its value is unused (0.0).
        mass_manual: mass_ov.is_some(),
        // ⚠️ **Dynamic OU player cinemático QUE TRANSMITE** — ver `mass_is_read`.
        // A 3ª lei transmite o peso de um player Snap ao chão pela massa do
        // corpo, então ali ela deixou de ser o número que o rapier ignora.
        //
        // ⚠️ **E a W-KinPure moveu este número:** sob o *puro sangue* a 3ª lei
        // está calada, então NADA lê a massa outra vez — a pergunta é *"alguém
        // a lê?"*, e a resposta mudou quando o terceiro modo chegou. Deixar a
        // condição de 2026-08-08 de pé teria devolvido o toggle Auto/Manual ao
        // estado de controle morto que a W-KinWeight existiu para curar.
        mass_is_read: rb.is_some_and(|b| {
            b.kind == ph2d_physics_ecs::BodyKind::Dynamic
                || (b.kind == ph2d_physics_ecs::BodyKind::Kinematic
                    && world
                        .get::<ph2d_physics_ecs::PlatformPlayer>(entity)
                        .is_some()
                    && world
                        .get::<ph2d_physics_ecs::PlayerMode>(entity)
                        .copied()
                        .unwrap_or_default()
                        .transmits())
        }),
        mass: mass_ov.unwrap_or(0.0),
        dominance,
        restitution_combine_tag: material.restitution.tag(),
        friction_combine_tag: material.friction.tag(),
        linear_damping: damping.linear,
        angular_damping: damping.angular,
        damp_mode_tag: damping.mode.tag(),
        one_way,
        no_wall_cling,
        force,
        force_world_axes,
        area_drag,
        area_density,
        area_form_drag,
        area_torque,
        area_falloff,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    use ph2d_physics_ecs::{
        AreaEffector, AreaForceWorldAxes, BodyKind, Collider, ColliderShape, RigidBody,
        SignalOnHit, SignalOnLeave,
    };

    /// **O snapshot CARREGA os dois nomes de sinal** (W-Signal · W-SignalLeave).
    ///
    /// ⚠️ **A metade que faltava desde o W-Signal.** O `InspectorPhysicsInfo` não
    /// tinha campo de sinal nenhum, então a row era **write-only por
    /// construção**: digitar `door` funcionava e re-selecionar a entidade
    /// mostrava um campo em branco, indistinguível de *"o nome não foi
    /// guardado"*. O gate de seam prova que o painel MOSTRA o que o snapshot
    /// carrega; este prova que o snapshot carrega o que o COMPONENTE diz — e as
    /// duas metades juntas são a volta inteira.
    ///
    /// Mutação (o `build_physics_info` devolver `String::new()` num dos dois) ⇒
    /// este gate sangra e o de seam fica VERDE, que é exatamente por que os dois
    /// existem.
    #[test]
    fn the_snapshot_carries_both_authored_signal_names() {
        let mut world = World::new();
        let e = world
            .spawn((
                Transform::from_translation(Vec2::new(0.0, 0.0)),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 1.0,
                        half_y: 1.0,
                    },
                    ..Collider::default()
                },
                SignalOnHit("door_open".to_string()),
                SignalOnLeave("door_close".to_string()),
            ))
            .id();
        let info = build_physics_info(
            &world,
            e.to_bits(),
            0,
            0,
            0,
            false,
            0,
            (0.0, 5.0),
            0,
            &ph2d_tags::TagTree::new(),
        )
        .expect("um corpo tem Transform, então a §11 o descreve");
        assert_eq!(
            info.signal, "door_open",
            "o nome de CHEGADA não chegou ao snapshot — a row é write-only"
        );
        assert_eq!(
            info.signal_leave, "door_close",
            "o nome de SAÍDA não chegou ao snapshot — a row é write-only"
        );
        // E o silêncio é silêncio: uma entidade sem os componentes traz os dois
        // vazios, senão a row nasceria mostrando o nome da seleção anterior.
        let plain = world
            .spawn((
                Transform::from_translation(Vec2::new(0.0, 0.0)),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider::default(),
            ))
            .id();
        let info = build_physics_info(
            &world,
            plain.to_bits(),
            0,
            0,
            0,
            false,
            0,
            (0.0, 5.0),
            0,
            &ph2d_tags::TagTree::new(),
        )
        .expect("idem");
        assert!(info.signal.is_empty() && info.signal_leave.is_empty());
    }

    /// **Selecting a zone shows the frame it was AUTHORED with** (W-AreaFrame).
    ///
    /// The read half of the marker. Its failure mode is the one Enio found in the five
    /// area rows on 2026-07-22: authoring works, the solver honours it, and re-selecting
    /// the zone shows the OTHER value — a control that lies about the state of the thing
    /// it edits. The panel's chip highlight is drawn straight from this field, and it is
    /// the only observable the seam cannot reach (a `seg_row`'s selection is a highlight
    /// in the scene, not a widget value), so the gate belongs here, at the source.
    #[test]
    fn selecting_a_zone_shows_the_force_frame_it_was_authored_with() {
        let zone = |marked: bool| {
            let mut world = World::new();
            let e = world
                .spawn((
                    Transform::from_translation(Vec2::new(0.0, 0.0)),
                    RigidBody {
                        kind: BodyKind::Static,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: 1.0,
                            half_y: 1.0,
                        },
                        is_sensor: true,
                        ..Collider::default()
                    },
                    AreaEffector { force: [3.0, 0.0] },
                ))
                .id();
            if marked {
                world.entity_mut(e).insert(AreaForceWorldAxes);
            }
            build_physics_info(
                &world,
                e.to_bits(),
                0,
                0,
                0,
                false,
                0,
                (0.0, 5.0),
                0,
                &ph2d_tags::TagTree::new(),
            )
            .expect("a zone has a Transform, so §11 describes it")
            .force_world_axes
        };
        assert!(
            !zone(false),
            "an unmarked zone is authored in its OWN frame — the default"
        );
        assert!(
            zone(true),
            "a zone carrying `AreaForceWorldAxes` must read back as world-axes; the \
             builder is not reading the marker, so the chip would show the wrong side"
        );
    }
}
