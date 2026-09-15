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
    hero: &HeroScreen,
    sim: &mut SimWorld,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    catalogs: &ph2d_asset_index::CatalogTree,
    component_registry: &ph2d_ecs::scene::ComponentRegistry,
    inspector_selection: &[u64],
    selected_count: usize,
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
    let sel = inspector_selection;
    // ⭐⭐ **O ÍNDICE DE ASSETS** (plano `docs/Components/07`, wave A2) — a junção das duas fontes,
    // publicada para o navegador. ⚠️ Só com o painel ABERTO: é uma travessia do mundo, e pagá-la
    // com o painel fechado é trabalho que ninguém lê.
    // ⭐⭐ A TAXONOMIA (wave A3) — publicada como o índice, e pela mesma razão: o painel não pode
    // ser a segunda fonte de verdade sobre o que existe.
    ph2d_panel_asset_browser::set_current_catalogs(catalogs.clone());
    ph2d_app_components::asset_index_build::publish_for_frame(
        sim,
        asset_db,
        // ⭐⭐ Ele já chegava a esta função — o que faltava era chegar ao índice. É o mapa que
        // separa *«o artista trouxe isto»* de *«o boot pôs isto no `AssetDb`»*.
        atlas_asset_map,
        catalogs,
        // ⭐ O retrato de um prefab põe cada peça onde a TELA a põe, e o pivô autorado converte-se
        // com o `pixels_per_meter` do projecto (`Sprite::resolve_anchor`).
        hero.project.pixels_per_meter,
        hero.is_panel_visible(ph2d_panel_asset_browser::PANEL_ID),
    );
    // ⭐⭐⭐ **A seção COMPONENT** (ADR-0164 / F5) — o que esta cópia tem de diferente da receita.
    // ⚠️ **`None` quando o selecionado não é peça de cópia nenhuma**, e aí a seção não existe: é a
    // lei da F3 (o Inspector mostra o que o objeto TEM). ⛔ Ao contrário da §5 e da §12, ela NÃO se
    // publica «vazia com um +»: não há gesto de anexar uma instância — ela nasce de *Instantiate*.
    let inspector_instance = crate::render_loop::inspector_instance::build_instance_info(
        sim,
        component_registry,
        hero.gizmo.selection,
    );
    // ⭐⭐⭐ **E o CARTÃO DE PROPRIEDADES** (report do Enio, 2026-08-31) — *«o que este objecto DIZ
    // que é»*, lido das chaves do nome. ⚠️ **Ele NÃO depende de ser cópia**: é exactamente o caso
    // que faltava, e era por isso que reescrever as chaves não mudava nada no Inspector.
    let inspector_properties =
        crate::render_loop::inspector_properties::build_properties_info(sim, hero.gizmo.selection);
    // W3: the Join gesture needs exactly TWO bodies, and only the shell can
    // see the selection — the panel is handed one entity at a time. Asked once
    // here, so the painter (which offers the button) and the event handler
    // (which honours the click) read the same fact.
    // ⚠️ `sel.len() == 2`, not just `selected_count == 2`: `all()` on an empty
    // slice is TRUE, so a count that disagreed with the slice would offer the
    // button over nothing at all.
    // ⚠️ **`>= 2`, not `== 2`** (W-J4): three or more selected bodies make a
    // CHAIN of `n − 1` joints, and the count travels to the panel so the button
    // can SAY so. A label that says "Join Selected Bodies" over a five-body
    // selection is how an artist discovers a chain by accident.
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

    // W-Rig: quantas PARTES um clique em *Rig* tocaria — 0 quando não há aresta
    // pai→filho a ligar, e é esse zero que tira o botão da tela.
    //
    // ⚠️ **Da `iter_selected()`, NÃO do `inspector_selection`** — aquele vetor é
    // colhido só numa MULTI-seleção (`selected_count > 1`) e fica vazio no caso
    // único, que é precisamente o gesto do rig: marcar a raiz do personagem e
    // clicar. Lido dali, o botão só apareceria com dois objetos marcados, isto é
    // em toda situação menos a que ele existe para servir.
    let rig_parts = {
        let roots: Vec<u64> = hero.gizmo.iter_selected().collect();
        let plan = ph2d_app_physics::joint_rig::plan(sim, &roots);
        if plan.is_offered() {
            u8::try_from(plan.parts.len()).unwrap_or(u8::MAX)
        } else {
            0
        }
    };

    // W-PartFace: quantas PEÇAS estão penduradas no objeto selecionado — filhos
    // que carregam `Collider` e não `RigidBody`.
    //
    // ⚠️ **Só a shell pode contar**, e é por isso que o número atravessa a
    // fronteira em vez de o painel o derivar: `ChildOf` é a única aresta do ECS,
    // então não há como DESCER a árvore — cada candidato tem de SUBIR até achar
    // um corpo, e a lista de candidatos vem de uma query sobre o mundo inteiro.
    // Mesma classe do `rig_parts` logo acima, e o mesmo custo por frame.
    let part_count = hero.gizmo.selection.map_or(0, |b| {
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

    let inspector_physics = hero.gizmo.selection.and_then(|b| {
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
            // ⭐ A árvore — a row *Only for tag* mostra o CAMINHO do filtro (TOP-20 #9, W3c).
            tags,
        )
    });
    let inspector_joint = hero.gizmo.selection.and_then(|b| {
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
    let inspector_wheel = hero.gizmo.selection.and_then(|b| {
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
