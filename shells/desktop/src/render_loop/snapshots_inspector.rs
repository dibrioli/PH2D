//! **O Inspector da seleção** — cada secção construída pela ordem de sempre (a sprite, o transform, a visibilidade,
//! o nome, a ordenação e a amostragem, o 9-slice e as âncoras, o índice de assets, a instância e as propriedades, a
//! física, as juntas e a roda, a animação, os timers, as acções, o áudio, a câmara) e publicada ao painel. Filho do
//! [`super`] (`snapshots`) por `#[path]` (OBRA 3 da `line/render-bodies`).

use super::*;

/// O Inspector deste quadro: cada secção da seleção construída pela ordem de sempre e publicada ao painel. O player
/// (§14) chega construído — o bloco dele fica na `publish`, onde o gate dos segundos de corrida o lê.
#[allow(clippy::too_many_arguments)]
pub(super) fn publish(
    hero: &HeroScreen,
    sim: &mut SimWorld,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    catalogs: &ph2d_asset_index::CatalogTree,
    sheets: &BTreeMap<u32, ph2d_sprite_sheet::AuthoredSheet>,
    renderer: &ph2d_render::SpriteRenderer,
    window_size: WindowSize,
    game_camera_preview: bool,
    bake_range: (f32, f32),
    bake_channels_tag: u8,
    join_kind_tag: u8,
    joint_body_pick: Option<(u64, bool)>,
    joint_paste_targets: usize,
    wheel_body_pick: Option<u64>,
    wheel_rope_pick: Option<u64>,
    join_draw_armed: bool,
    component_registry: &ph2d_ecs::scene::ComponentRegistry,
    inspector_player: Option<ph2d_editor_core::InspectorPlayerInfo>,
    // ⭐ A árvore de tags — ver o parâmetro homónimo do `super::publish`.
    tags: &ph2d_tags::TagTree,
) {
    // M14.5 inspector phase (6.4/§9): publish a per-frame
    // snapshot of the selected sprite so `paint_inspector` can
    // surface the Render Source section + Reimport button
    // without crossing the ADR-0021 boundary into SimWorld.
    // BulkSelect (T2.0): the full selection (primary + extras). Only
    // collected (one alloc) for a MULTI-selection — single-select (the
    // common case) takes the empty path and skips the Mixed compare.
    let selected_count = hero.gizmo.selected_len();
    let inspector_selection: Vec<u64> = if selected_count > 1 {
        hero.gizmo.iter_selected().collect()
    } else {
        Vec::new()
    };
    let inspector_sprite = super::inspector_sprite::sprite_info(
        hero,
        sim,
        &inspector_selection,
        selected_count,
        atlas_asset_map,
        asset_db,
        renderer,
        sheets,
    );
    let Identity {
        inspector_transform,
        inspector_visibility,
        inspector_name,
    } = identity(hero, sim, &inspector_selection);
    let sel = &inspector_selection; // W3 §7/§9 snapshots (§7 sibling module)
    let inspector_ordering = hero.gizmo.selection.and_then(|b| {
        ph2d_inspector_ordering::build_ordering_info(sim.world(), b, sel, selected_count)
    });
    let inspector_sampling = hero.gizmo.selection.and_then(|b| {
        ph2d_inspector_ordering::build_sampling_info(sim.world(), b, sel, selected_count)
    });
    let inspector_blend = hero.gizmo.selection.and_then(|b| {
        ph2d_inspector_ordering::build_blend_info(sim.world(), b, sel, selected_count)
    });
    // §5 9-Slice. ⚠️ Publicado para TODA entidade digna de Inspector, com ou sem o componente:
    // é o snapshot que diz `present: false`, e é isso que faz a seção mostrar o «+ Add 9-Slice».
    // Publicar só quando o componente existe faria a seção aparecer depois de a feature estar
    // ligada — ou seja, nunca, porque não haveria por onde ligá-la.
    let inspector_slice = hero.gizmo.selection.and_then(|b| {
        crate::render_loop::inspector_slice::build_slice_info(sim.world(), b, sel, selected_count)
    });
    // §12 Sockets / Named Anchors (ADR-0072). Publicado para toda entidade digna de Inspector —
    // é o snapshot que diz `present: false`, e é isso que faz a seção mostrar o «+ Add Anchor».
    let inspector_anchor = hero.gizmo.selection.and_then(|b| {
        crate::render_loop::inspector_anchor::build_anchor_info(
            sim.world(),
            b,
            sel,
            selected_count,
            hero.project.pixels_per_meter,
        )
    });
    let PhysicsSections {
        inspector_instance,
        inspector_properties,
        inspector_physics,
        inspector_joint,
        inspector_wheel,
    } = physics(
        hero,
        sim,
        asset_db,
        atlas_asset_map,
        catalogs,
        component_registry,
        &inspector_selection,
        selected_count,
        join_draw_armed,
        join_kind_tag,
        bake_range,
        bake_channels_tag,
        joint_body_pick,
        joint_paste_targets,
        wheel_body_pick,
        wheel_rope_pick,
        tags,
    );
    let LateSections {
        inspector_anim,
        inspector_timer,
        inspector_action,
        inspector_audio,
        inspector_camera,
        inspector_visibility_section,
    } = late(
        hero,
        sim,
        &inspector_selection,
        selected_count,
        window_size,
        game_camera_preview,
        tags,
    );
    // ⭐⭐⭐ **A secção TAGS** (TOP-20 #9) — `None` para quem não tem o componente (ADR-0166).
    //
    // ⚠️⚠️ **Esta nota dizia que a árvore não tinha o que fazer na [`late`]** — *«ela recebe o que
    // sai da CENA, e enfiá-la lá obrigaria a receber um argumento que nenhuma das outras cinco
    // lê»*. A W3b desmentiu-a no dia seguinte: a secção SIGNAL ACTIONS também precisa da árvore
    // (para mostrar o CAMINHO da tag alvo), logo a `late` recebe-a e são DUAS a lê-la. *Uma
    // justificação que assenta em «só uma precisa disto» expira no dia em que a segunda aparece.*
    // Esta fica aqui e não lá por outra razão, essa estável: a `late` devolve um bloco de cinco
    // irmãs, e acrescentar uma sexta ao `LateSections` só para poupar uma linha não a torna irmã
    // de nada.
    let inspector_tags = hero.gizmo.selection.and_then(|b| {
        crate::render_loop::inspector_tags::build_tags_info(sim.world(), tags, b, selected_count)
    });
    // ADR-0029 Phase C.1: publish snapshots to the panel crate's
    // thread-locals (replaces the pre-C.1 `hero.inspector.<field>`
    // writes — the field no longer exists; the panel-owned state +
    // its thread-local snapshot setters do).
    #[cfg(feature = "panel-inspector")]
    {
        ph2d_panel_inspector::set_current_inspector_sprite(inspector_sprite);
        ph2d_panel_inspector::set_current_inspector_ordering(inspector_ordering);
        ph2d_panel_inspector::set_current_inspector_sampling(inspector_sampling);
        ph2d_panel_inspector::set_current_inspector_blend(inspector_blend);
        ph2d_panel_inspector::set_current_inspector_slice(inspector_slice);
        ph2d_panel_inspector::set_current_inspector_anchor(inspector_anchor);
        ph2d_panel_inspector::set_current_inspector_instance(inspector_instance);
        ph2d_panel_inspector::set_current_inspector_properties(inspector_properties);
        ph2d_panel_inspector::set_current_inspector_anim(inspector_anim);
        ph2d_panel_inspector::set_current_inspector_timer(inspector_timer);
        ph2d_panel_inspector::set_current_inspector_action(inspector_action);
        ph2d_panel_inspector::set_current_inspector_audio(inspector_audio);
        ph2d_panel_inspector::set_current_inspector_camera(inspector_camera);
        ph2d_panel_inspector::set_current_inspector_tags(inspector_tags);
        // ⭐ **A ÁRVORE DO PROJECTO** — publicada em TODO quadro, com ou sem selecção: ela não é
        // dado de um objecto, e o segundo consumidor (o alvo de uma *Signal Action*) vive num
        // objecto que pode não ter `Tags` nenhum.
        ph2d_panel_inspector::set_current_tag_tree(
            crate::render_loop::inspector_tags::tag_tree_rows(tags),
        );
        ph2d_panel_inspector::set_current_inspector_physics(inspector_physics);
        ph2d_panel_inspector::set_current_inspector_joint(inspector_joint);
        ph2d_panel_inspector::set_current_inspector_wheel(inspector_wheel);
        ph2d_panel_inspector::set_current_inspector_player(inspector_player);
        ph2d_panel_inspector::set_current_inspector_visibility_section(
            inspector_visibility_section,
        );
        ph2d_panel_inspector::set_current_inspector_transform(inspector_transform);
        ph2d_panel_inspector::set_current_inspector_visibility(inspector_visibility);
        ph2d_panel_inspector::set_current_inspector_name(inspector_name);
        ph2d_panel_inspector::set_current_display_unit(
            hero.project.display_unit,
            hero.project.pixels_per_meter,
        );
    }
    #[cfg(not(feature = "panel-inspector"))]
    {
        let _ = (
            inspector_physics,
            inspector_sprite,
            inspector_transform,
            inspector_visibility,
            inspector_visibility_section,
            inspector_name,
        );
    }
}

/// As secções que o [`late`] constrói, com os nomes que a [`publish`] publica.
struct LateSections {
    inspector_anim: Option<ph2d_editor_core::InspectorAnimInfo>,
    inspector_timer: Option<ph2d_editor_core::InspectorTimerInfo>,
    inspector_action: Option<ph2d_editor_core::InspectorActionInfo>,
    inspector_audio: Option<ph2d_editor_core::InspectorAudioInfo>,
    inspector_camera: Option<ph2d_editor_core::InspectorCameraInfo>,
    inspector_visibility_section: Option<ph2d_editor_core::InspectorVisibilitySectionInfo>,
}

/// As secções de componente opcional que vêm depois do player: animação, timers, acções, áudio, câmara, e a secção
/// Visibility.
fn late(
    hero: &HeroScreen,
    sim: &mut SimWorld,
    inspector_selection: &[u64],
    selected_count: usize,
    window_size: WindowSize,
    game_camera_preview: bool,
    // ⭐ A árvore de tags (TOP-20 #9) — a secção SIGNAL ACTIONS mostra o CAMINHO da tag alvo.
    tags: &ph2d_tags::TagTree,
) -> LateSections {
    let sel = inspector_selection;
    let inspector_anim = hero.gizmo.selection.and_then(|b| {
        crate::render_loop::inspector_anim::build_anim_info(sim.world(), b, selected_count)
    });
    // ⭐ A secção TIMERS — `None` para quem não tem o componente (ADR-0166).
    let inspector_timer = hero.gizmo.selection.and_then(|b| {
        crate::render_loop::inspector_timer::build_timer_info(sim.world(), b, selected_count)
    });
    // ⭐ A secção SIGNAL ACTIONS — `None` para quem não tem o componente (ADR-0166).
    let inspector_action = hero.gizmo.selection.and_then(|b| {
        crate::render_loop::inspector_action::build_action_info(
            sim.world(),
            tags,
            b,
            selected_count,
        )
    });
    // ⭐ A secção AUDIO — `None` para quem não tem a fonte NEM as orelhas (ADR-0166).
    //
    // ⚠️ **Ela pede o mundo em MUTÁVEL**, e é a única da família: as três coisas que ela deriva —
    // quantas orelhas a cena tem, qual delas manda, e se alguma tabela de acções manda isto tocar —
    // são **queries**, e um `QueryState` do bevy precisa de `&mut World` para se preparar. *Não é
    // escrita: é o preço de perguntar à cena em vez de adivinhar a partir do componente.*
    let inspector_audio = hero.gizmo.selection.and_then(|b| {
        crate::render_loop::inspector_audio::build_audio_info(sim.world_mut(), b, selected_count)
    });
    // ⭐ A secção CAMERA — `None` para quem não tem `GameCamera` (ADR-0166).
    //
    // ⚠️ **Ela pede a PROPORÇÃO da janela**, e é a única da família: o aviso *«a cerca é mais
    // estreita que a vista»* é geometria do ECRÃ, e não dos quatro números da cerca.
    let inspector_camera = hero.gizmo.selection.and_then(|b| {
        crate::render_loop::inspector_camera::build_camera_info(
            sim.world_mut(),
            b,
            selected_count,
            crate::render_loop::camera_2d::aspect_of(window_size),
            game_camera_preview,
        )
    });
    let inspector_visibility_section = hero.gizmo.selection.and_then(|b| {
        crate::render_loop::inspector_visibility::build_visibility_section_info(
            sim.world(),
            b,
            sel,
            selected_count,
        )
    });
    LateSections {
        inspector_anim,
        inspector_timer,
        inspector_action,
        inspector_audio,
        inspector_camera,
        inspector_visibility_section,
    }
}

/// As secções que o [`physics`] constrói, com os nomes que a [`publish`] publica.
struct PhysicsSections {
    inspector_instance: Option<ph2d_editor_core::screens::hero::InspectorInstanceInfo>,
    inspector_properties: Option<ph2d_editor_core::screens::hero::InspectorPropertiesInfo>,
    inspector_physics: Option<ph2d_editor_core::InspectorPhysicsInfo>,
    inspector_joint: Option<ph2d_editor_core::InspectorJointInfo>,
    inspector_wheel: Option<ph2d_editor_core::InspectorWheelInfo>,
}

/// O índice de assets publicado ao navegador, a secção COMPONENT e o cartão de propriedades, e a família da física:
/// o corpo (com as contagens do Join, do Rig e das peças), a junta e a roda.
#[allow(clippy::too_many_arguments)]
fn physics(
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

/// O que o [`identity`] constrói, com os nomes que a [`publish`] publica.
struct Identity {
    inspector_transform: Option<ph2d_editor_core::InspectorTransformInfo>,
    inspector_visibility: Option<ph2d_editor_core::InspectorVisibilityInfo>,
    inspector_name: Option<ph2d_editor_core::InspectorNameInfo>,
}

/// O transform, a visibilidade (com o «Mixed» da multi-seleção) e o nome da seleção primária.
fn identity(hero: &HeroScreen, sim: &SimWorld, inspector_selection: &[u64]) -> Identity {
    // M14.A: live Transform snapshot for the inspector. Same
    // ADR-0021 / HR-8 boundary as sprite snapshot — Inspector
    // never reads SimWorld; the host bridges. Lands on every
    // entity that has a `Transform` component, not just sprites
    // (so non-renderable entities still show their pose).
    let inspector_transform = hero.gizmo.selection.and_then(|bits| {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let t = sim.world().get::<Transform>(entity)?;
        Some(ph2d_editor_core::InspectorTransformInfo {
            entity_bits: bits,
            translation: [t.translation.x, t.translation.y],
            rotation_rad: t.rotation,
            scale: [t.scale.x, t.scale.y],
            skew_rad: [t.skew_x, t.skew_y],
        })
    });
    // M14.D: live Visibility snapshot. Absence-equals-visible
    // is the canonical invariant — entities without a
    // `Visibility` component render normally, so `None` from
    // `world.get::<Visibility>` maps to `visible = true`.
    // Only published when the selection has a `Transform`
    // (i.e. it's an Inspector-worthy entity); without a
    // Transform the Inspector hides the whole panel content.
    let inspector_visibility = hero.gizmo.selection.and_then(|bits| {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        sim.world().get::<Transform>(entity)?;
        let visible = sim
            .world()
            .get::<Visibility>(entity)
            .map(|v| !v.hidden)
            .unwrap_or(true);
        // ⚠️ A ausência de `Visibility` É visível — a mesma invariante que a leitura acima usa, e
        // por isso a comparação passa pela mesma expressão. Compará-las como `Option` diria que
        // uma sprite sem componente diverge de outra com `hidden: false`, e as duas estão visíveis.
        let mixed = inspector_selection.iter().any(|&other| {
            let e = ph2d_ecs::Entity::from_bits(other);
            sim.world()
                .get::<Visibility>(e)
                .map(|v| !v.hidden)
                .unwrap_or(true)
                != visible
        });
        Some(ph2d_editor_core::InspectorVisibilityInfo {
            entity_bits: bits,
            visible,
            mixed,
        })
    });
    // M14.E: live `Name` snapshot. Falls back to
    // `Entity_{hex}` when the entity has no Name component
    // yet — matches the existing `InspectorSpriteInfo::name`
    // shape. Same Transform-presence gate.
    let inspector_name = hero.gizmo.selection.and_then(|bits| {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        sim.world().get::<Transform>(entity)?;
        let name = sim
            .world()
            .get::<Name>(entity)
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("Entity_{bits:x}"));
        Some(ph2d_editor_core::InspectorNameInfo {
            entity_bits: bits,
            name,
        })
    });
    Identity {
        inspector_transform,
        inspector_visibility,
        inspector_name,
    }
}
