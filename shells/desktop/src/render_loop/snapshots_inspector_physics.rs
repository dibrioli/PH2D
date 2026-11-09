//! **As secções de FÍSICA do Inspector** (o corpo, a junta, a roda) e as duas que viajam com elas
//! (o índice de assets e o cartão de propriedades) — irmã do [`super::snapshots_inspector`] por
//! `#[path]`.
//!
//! ⚠️ **O corte foi imposto pelo tecto de LOC do HR-18 e está certo por RESPONSABILIDADE:** o
//! ficheiro-mãe responde *«que secções o Inspector publica neste quadro»*; este responde *«o que a
//! família da FÍSICA tem para mostrar»*, que é a metade que pede oito contagens do mundo (o Join,
//! o Rig, as peças, as âncoras, a roda) e que nenhuma das outras secções precisa de ver.
//!
//! ⛔ **Nunca subir o número de `FILE_OVERAGE_OK`: ele só desce.**

use super::*;

/// As secções que o [`physics`] constrói, com os nomes que a [`publish`] publica.
pub(super) struct PhysicsSections {
    pub(super) inspector_instance: Option<ph2d_editor_core::screens::hero::InspectorInstanceInfo>,
    pub(super) inspector_properties:
        Option<ph2d_editor_core::screens::hero::InspectorPropertiesInfo>,
    pub(super) inspector_physics: Option<ph2d_editor_core::InspectorPhysicsInfo>,
    pub(super) inspector_joint: Option<ph2d_editor_core::InspectorJointInfo>,
    pub(super) inspector_wheel: Option<ph2d_editor_core::InspectorWheelInfo>,
}

/// O índice de assets publicado ao navegador, a secção COMPONENT e o cartão de propriedades, e a família da física:
/// o corpo (com as contagens do Join, do Rig e das peças), a junta e a roda.
#[allow(clippy::too_many_arguments)]
pub(super) fn physics(
    alvo: &Alvo,
    sim: &mut SimWorld,
    component_registry: &ph2d_ecs::scene::ComponentRegistry,
    join_draw_armed: bool,
    join_kind_tag: u8,
    bake_range: (f32, f32),
    bake_channels_tag: u8,
    joint_body_pick: Option<(u64, bool)>,
    joint_paste_targets: usize,
    wheel_body_pick: Option<u64>,
    wheel_rope_pick: Option<u64>,
    // ⭐ A árvore de tags — a row *Only for tag* da §11 (TOP-20 #9, W3c).
    tags: &ph2d_tags::TagTree,
) -> PhysicsSections {
    let (sel, selected_count) = (alvo.todos.as_slice(), alvo.n);
    let inspector_instance = crate::render_loop::inspector_instance::build_instance_info(
        sim,
        component_registry,
        alvo.bits,
    );
    // ⭐⭐⭐ **E o CARTÃO DE PROPRIEDADES** (report do Enio, 2026-08-31) — *«o que este objecto DIZ
    // que é»*, lido das chaves do nome. ⚠️ **Ele NÃO depende de ser cópia**: é exactamente o caso
    // que faltava, e era por isso que reescrever as chaves não mudava nada no Inspector.
    let inspector_properties =
        crate::render_loop::inspector_properties::build_properties_info(sim, alvo.bits);
    // W3/W-J4: o *Join* pede DOIS corpos ou mais (três fazem uma CADEIA de n − 1, e o rótulo di-lo);
    // `sel.len()`, não só a contagem: `all()` sobre vazio é verdade.
    let join_count = if sel.len() >= 2
        && selected_count == sel.len()
        && sel.iter().all(|&b| {
            let e = ph2d_ecs::Entity::from_bits(b);
            sim.world().get::<ph2d_physics_ecs::RigidBody>(e).is_some()
                && sim.world().get::<ph2d_physics_ecs::Collider>(e).is_some()
        }) {
        u8::try_from(sel.len()).unwrap_or(u8::MAX)
    } else {
        0
    };

    // W-Rig: quantas PARTES um *Rig* tocaria (0 tira o botão) — das raízes do alvo, também com uma só.
    let rig_parts = {
        let roots = if sel.is_empty() {
            alvo.bits.into_iter().collect()
        } else {
            sel.to_vec()
        };
        let plan = ph2d_app_physics::joint_rig::plan(sim, &roots);
        if plan.is_offered() {
            u8::try_from(plan.parts.len()).unwrap_or(u8::MAX)
        } else {
            0
        }
    };

    // W-PartFace: as PEÇAS penduradas no objecto (filhos com `Collider` sem `RigidBody`); só a shell
    // conta — o ECS só sobe (`ChildOf`), então cada candidato sobe até achar um corpo.
    let part_count = alvo.bits.map_or(0, |b| {
        let owner = ph2d_ecs::Entity::from_bits(b);
        let mut q = sim.world_mut().query_filtered::<ph2d_ecs::Entity, (
            bevy_ecs::query::With<ph2d_physics_ecs::Collider>,
            bevy_ecs::query::Without<ph2d_physics_ecs::RigidBody>,
        )>();
        let candidates: Vec<ph2d_ecs::Entity> = q.iter(sim.world()).collect();
        u8::try_from(ph2d_physics_ecs::count_parts(
            sim.world(),
            owner,
            candidates,
        ))
        .unwrap_or(u8::MAX)
    });

    let inspector_physics = alvo.bits.and_then(|b| {
        ph2d_app_physics::inspector::body::build_physics_info(
            sim.world(),
            b,
            join_count,
            rig_parts,
            part_count,
            join_draw_armed,
            join_kind_tag,
            bake_range,
            bake_channels_tag,
            tags,
        )
        .map(|i| ph2d_app_physics::inspector::body::com_a_selecao(sim.world(), i, sel))
    });
    let inspector_joint = alvo.bits.and_then(|b| {
        // The eyedropper of the slot with an armed pick FOR THIS joint paints
        // pressed; 0 otherwise.
        let pick_armed = match joint_body_pick {
            Some((j, slot_b)) if j == b => {
                if slot_b {
                    2
                } else {
                    1
                }
            }
            _ => 0,
        };
        ph2d_app_physics::joint::build_joint_info(sim, b, pick_armed, joint_paste_targets)
    });
    // §13 Pulley Wheel (W-Pulley W1) — a irmã da §12, e a seleção é a MESMA
    // pergunta: uma roldana é uma entidade, então ela é o objeto selecionado.
    let inspector_wheel = alvo.bits.and_then(|b| {
        ph2d_app_physics::joint_wheel::build_wheel_info(
            sim,
            b,
            wheel_body_pick == Some(b),
            wheel_rope_pick == Some(b),
        )
    });
    PhysicsSections {
        inspector_instance,
        inspector_properties,
        inspector_physics,
        inspector_joint,
        inspector_wheel,
    }
}

/// ⭐⭐⭐ **As duas secções cujo AGORA vem de um componente DERIVADO** — HEALTH/DAMAGE (plano 28)
/// lê o `HealthNow` e NAV (plano 30) lê o `NavNow`, os dois publicados pela ponte da física no fim
/// de cada despacho. ⚠️ Mudaram-se do [`super::late`] pelo tecto de função (`203` contra `200`),
/// e cabem aqui por RESPONSABILIDADE: os quatro componentes vivem na `ph2d-physics-ecs`.
pub(super) fn vida_e_nav(
    selection: Option<u64>,
    sim: &SimWorld,
    selected_count: usize,
    clock_playing: bool,
) -> (
    Option<ph2d_editor_core::vida_edits::InspectorVidaInfo>,
    Option<ph2d_editor_core::nav_edits::InspectorNavInfo>,
) {
    let Some(b) = selection else {
        return (None, None);
    };
    let w = sim.world();
    (
        ph2d_app_components::vida_inspector::build_vida_info(w, b, selected_count, clock_playing),
        ph2d_app_components::nav_inspector::build_info(w, b, selected_count, clock_playing),
    )
}
