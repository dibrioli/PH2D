//! **O Inspector da seleção** — cada secção construída pela ordem de sempre (a sprite, o transform, a visibilidade,
//! o nome, a ordenação e a amostragem, o 9-slice e as âncoras, o índice de assets, a instância e as propriedades, a
//! física, as juntas e a roda, a animação, os timers, as acções, o áudio, a câmara) e publicada ao painel. Filho do
//! [`super`] (`snapshots`) por `#[path]` (OBRA 3 da `line/render-bodies`).

use super::*;

/// O objecto que o Inspector retrata: a seleção, ou um objecto preso por um cartão solto (T61).
pub(super) struct Alvo {
    pub(super) bits: Option<u64>,
    pub(super) n: usize,
    /// A multi-seleção inteira — vazia com um só (o caso comum salta o «Mixed»).
    pub(super) todos: Vec<u64>,
}

impl Alvo {
    pub(super) fn da_selecao(hero: &HeroScreen) -> Self {
        let n = hero.gizmo.selected_len();
        let todos = if n > 1 {
            hero.gizmo.iter_selected().collect()
        } else {
            Vec::new()
        };
        Self {
            bits: hero.gizmo.selection,
            n,
            todos,
        }
    }

    pub(super) fn preso(b: u64) -> Self {
        Self {
            bits: Some(b),
            n: 1,
            todos: Vec::new(),
        }
    }
}

/// O que é do PROJECTO, não de um objecto (catálogos, índice de assets, árvore de tags, unidade):
/// uma vez por quadro, fora do retrato de cada objecto.
pub(super) fn publish_global(
    hero: &HeroScreen,
    sim: &mut SimWorld,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    catalogs: &ph2d_asset_index::CatalogTree,
    tags: &ph2d_tags::TagTree,
) {
    ph2d_panel_asset_browser::set_current_catalogs(catalogs.clone());
    ph2d_app_components::asset_index_build::publish_for_frame(
        sim,
        asset_db,
        atlas_asset_map,
        catalogs,
        hero.project.pixels_per_meter,
        hero.is_panel_visible(ph2d_panel_asset_browser::PANEL_ID),
        &hero.documents.asset_listing(), // D17: os Kanbans e os Quadros
    );
    let (unit, ppm) = (hero.project.display_unit, hero.project.pixels_per_meter);
    ph2d_panel_inspector::set_current_tag_tree(crate::render_loop::inspector_tags::tag_tree_rows(
        tags,
    ));
    ph2d_panel_inspector::set_current_display_unit(unit, ppm);
}

/// O Inspector de `alvo` neste quadro: cada secção construída pela ordem de sempre e publicada ao
/// painel. O player (§14) chega construído.
#[allow(clippy::too_many_arguments)]
pub(super) fn publish(
    hero: &HeroScreen,
    alvo: &Alvo,
    sim: &mut SimWorld,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    sheets: &BTreeMap<u32, ph2d_sprite_sheet::AuthoredSheet>,
    renderer: &ph2d_render::SpriteRenderer,
    game_camera_preview: bool,
    preview_drive: &ph2d_preview_drive::PreviewDrive,
    // ⭐ O relógio anda? — a secção FACTORY di-lo (TOP-20 #11).
    clock_playing: bool,
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
    // ⭐ **Os projécteis cujo voo acabou neste quadro** (TOP-20 #14) — o readout que a fase da
    // física deixou no `PhysicsState`. ⚠️ O facto vive na PONTE (o estado de voo é memória de
    // tique, não componente) e o Inspector não a alcança; este é o mesmo caminho do `clock_playing`.
    projectile_over: &[u64],
    // ⭐⭐⭐ **O que cada RAIO vê neste quadro** (suplente #21) — `(olha, visto, distância)`, pelo
    // mesmo caminho do vizinho acima e pela mesma razão: o facto vive na PONTE.
    ray_hits: &[(u64, u64, f32)],
) {
    let (selected_count, inspector_selection) = (alvo.n, alvo.todos.as_slice());
    ph2d_panel_inspector::set_current_inspector_selecionados(selected_count);
    let inspector_sprite = super::inspector_sprite::sprite_info(
        hero,
        alvo,
        sim,
        atlas_asset_map,
        asset_db,
        renderer,
        sheets,
    );
    let Identity {
        inspector_transform,
        inspector_visibility,
        inspector_name,
    } = identity(alvo, sim);
    let sel = inspector_selection; // W3 §7/§9 snapshots (§7 sibling module)
    let inspector_ordering = alvo.bits.and_then(|b| {
        ph2d_inspector_ordering::build_ordering_info(sim.world(), b, sel, selected_count)
    });
    let inspector_sampling = alvo.bits.and_then(|b| {
        ph2d_inspector_ordering::build_sampling_info(sim.world(), b, sel, selected_count)
    });
    let inspector_blend = alvo.bits.and_then(|b| {
        ph2d_inspector_ordering::build_blend_info(sim.world(), b, sel, selected_count)
    });
    // §5 9-Slice. ⚠️ Publicado para TODA entidade digna de Inspector, com ou sem o componente:
    // é o snapshot que diz `present: false`, e é isso que faz a seção mostrar o «+ Add 9-Slice».
    // Publicar só quando o componente existe faria a seção aparecer depois de a feature estar
    // ligada — ou seja, nunca, porque não haveria por onde ligá-la.
    let inspector_slice = alvo.bits.and_then(|b| {
        crate::render_loop::inspector_slice::build_slice_info(sim.world(), b, sel, selected_count)
    });
    // §12 Sockets / Named Anchors (ADR-0072). Publicado para toda entidade digna de Inspector —
    // é o snapshot que diz `present: false`, e é isso que faz a seção mostrar o «+ Add Anchor».
    let inspector_anchor = alvo.bits.and_then(|b| {
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
        alvo,
        sim,
        component_registry,
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
        inspector_factory,
        inspector_topdown,
        inspector_projectile,
        inspector_ray,
        inspector_parallax,
        inspector_weapon,
        inspector_vida,
        inspector_nav,
        inspector_tween,
        inspector_path_follow,
        inspector_statemachine,
        inspector_visibility_section,
    } = late(
        hero,
        alvo,
        sim,
        game_camera_preview,
        preview_drive,
        tags,
        clock_playing,
        projectile_over,
        ray_hits,
    );
    // ⭐ A secção TAGS (TOP-20 #9) — `None` sem o componente (ADR-0166).
    let inspector_tags = alvo.bits.and_then(|b| {
        crate::render_loop::inspector_tags::build_tags_info(sim.world(), tags, b, selected_count)
    });
    // ADR-0029 C.1: os instantâneos vão para os thread-locals do painel.
    #[cfg(feature = "panel-inspector")]
    {
        slot_image_snapshot::publish_slot_image(
            alvo,
            sim,
            asset_db,
            atlas_asset_map,
            hero.project.pixels_per_meter,
        );
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
        ph2d_panel_inspector::set_current_inspector_factory(inspector_factory);
        ph2d_panel_inspector::set_current_inspector_topdown(inspector_topdown);
        ph2d_panel_inspector::set_current_inspector_projectile(inspector_projectile);
        ph2d_panel_inspector::set_current_inspector_ray(inspector_ray);
        ph2d_panel_inspector::set_current_inspector_parallax(inspector_parallax);
        ph2d_panel_inspector::set_current_inspector_weapon(inspector_weapon);
        ph2d_panel_inspector::set_current_inspector_vida(inspector_vida);
        ph2d_panel_inspector::set_current_inspector_nav(inspector_nav);
        ph2d_panel_inspector::set_current_inspector_tween(inspector_tween);
        ph2d_panel_inspector::set_current_inspector_path_follow(inspector_path_follow);
        ph2d_panel_inspector::set_current_inspector_statemachine(inspector_statemachine);
        ph2d_panel_inspector::set_current_inspector_tags(inspector_tags);
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
    /// ⭐ As secções FACTORY e LIFECYCLE (TOP-20 #11 e #12).
    inspector_factory: Option<ph2d_editor_core::InspectorFactoryInfo>,
    /// ⭐ A secção TOP-DOWN PLAYER (TOP-20 #13).
    inspector_topdown: Option<ph2d_editor_core::topdown_edits::InspectorTopDownInfo>,
    /// ⭐ A secção PROJECTILE MOTION (TOP-20 #14).
    inspector_projectile: Option<ph2d_editor_core::projectile_edits::InspectorProjectileInfo>,
    /// ⭐⭐⭐ A secção RAY SENSOR (suplente #21).
    inspector_ray: Option<ph2d_editor_core::ray_edits::InspectorRayInfo>,
    /// ⭐⭐⭐ A secção PARALLAX (plano 24).
    inspector_parallax: Option<ph2d_editor_core::parallax_edits::InspectorParallaxInfo>,
    inspector_weapon: Option<ph2d_editor_core::weapon_edits::InspectorWeaponInfo>,
    /// ⭐⭐⭐ As secções HEALTH e DAMAGE (plano 28, W3).
    inspector_vida: Option<ph2d_editor_core::vida_edits::InspectorVidaInfo>,
    inspector_nav: Option<ph2d_editor_core::nav_edits::InspectorNavInfo>,
    inspector_tween: Option<ph2d_editor_core::tween_edits::InspectorTweenInfo>,
    /// ⭐⭐⭐ A secção PATH FOLLOW (suplente #23).
    inspector_path_follow: Option<ph2d_editor_core::path_follow_edits::InspectorPathFollowInfo>,
    inspector_statemachine: Option<ph2d_editor_core::statemachine_edits::InspectorStateMachineInfo>,
    inspector_visibility_section: Option<ph2d_editor_core::InspectorVisibilitySectionInfo>,
}

/// As secções de componente opcional que vêm depois do player: animação, timers, acções, áudio, câmara, e a secção
/// Visibility.
///
/// ⚠️ **Oito argumentos, e a cura seria PIOR do que o aviso.** Eles não são estado partilhado: são
/// as oito coisas DIFERENTES que as secções tardias perguntam (a selecção, o mundo, a janela, a
/// pré-visualização da câmera, a árvore de tags, o relógio). Empacotá-las numa struct faria o
/// chamador construir um valor para o desmontar duas linhas abaixo, e a cada wave que traz uma
/// secção nova o campo entraria na struct **e** na chamada — dois sítios em vez de um.
///
/// ⛔ O aviso chegou aos oito na wave das TAGS + FÁBRICA (`tags` e `clock_playing`) e passou
/// despercebido ao portão de fecho dela. *Um `-D warnings` que só corre no `ship.sh` é um portão
/// que a linha não vê.*
#[allow(clippy::too_many_arguments)]
fn late(
    hero: &HeroScreen,
    alvo: &Alvo,
    sim: &mut SimWorld,
    game_camera_preview: bool,
    preview_drive: &ph2d_preview_drive::PreviewDrive,
    // ⭐ A árvore de tags (TOP-20 #9) — a secção SIGNAL ACTIONS mostra o CAMINHO da tag alvo.
    tags: &ph2d_tags::TagTree,
    // ⭐ O relógio anda? — a secção FACTORY di-lo, e é o que separa «avariada» de «à espera».
    clock_playing: bool,
    // ⭐ Os projécteis cujo voo acabou (TOP-20 #14) — ver o parâmetro homónimo do [`publish`].
    projectile_over: &[u64],
    // ⭐⭐⭐ O que cada RAIO vê (suplente #21) — ver o parâmetro homónimo do [`publish`].
    ray_hits: &[(u64, u64, f32)],
) -> LateSections {
    let (sel, selected_count) = (alvo.todos.as_slice(), alvo.n);
    let mut inspector_anim = alvo.bits.and_then(|b| {
        ph2d_app_sprite_anim::inspector::build_anim_info(sim.world(), b, selected_count)
    });
    // As teclas do mapa do herói são as opções «Key held» das regras (plano GameEngineFuncional 08).
    if let Some(a) = inspector_anim.as_mut() {
        for x in hero.input_map.actions() {
            a.card.rule_actions.push(x.name.clone());
        }
        a.card.live_rule =
            ph2d_app_game::personagem::regra_acesa(sim.world(), a.entity_bits, clock_playing);
    }
    // ⭐ A secção TIMERS — `None` para quem não tem o componente (ADR-0166).
    let inspector_timer = alvo.bits.and_then(|b| {
        ph2d_app_components::timer_inspector::build_timer_info(sim.world(), b, selected_count)
    });
    // ⭐ A secção SIGNAL ACTIONS — `None` para quem não tem o componente (ADR-0166).
    let inspector_action = alvo.bits.and_then(|b| {
        ph2d_app_components::signal_actions_inspector::build_action_info(
            sim.world(),
            tags,
            b,
            selected_count,
        )
    });
    // ⭐ AUDIO, `None` sem fonte nem orelhas (ADR-0166); `&mut` porque as três perguntas à cena
    // (quantas orelhas, qual manda, que acção o toca) são queries — não é escrita.
    let inspector_audio = alvo.bits.and_then(|b| {
        ph2d_app_components::audio_inspector::build_audio_info(sim.world_mut(), b, selected_count)
    });
    // ⭐ A secção CAMERA — `None` para quem não tem `GameCamera` (ADR-0166).
    //
    // ⚠️ **Ela pede a PROPORÇÃO do jogo** (o formato do projecto, plano GEF 17), e é a única da família: o aviso
    // *«a cerca é mais estreita que a vista»* é geometria do ECRÃ, e não dos quatro números da cerca.
    let inspector_camera = alvo.bits.and_then(|b| {
        ph2d_app_components::camera_inspector::build_camera_info(
            sim.world_mut(),
            b,
            selected_count,
            hero.project.game_format,
            hero.project.camera_frame,
            hero.project.camera_icon_px,
            game_camera_preview,
        )
        .map(|i| ph2d_editor_core::screens::hero::InspectorCameraInfo {
            canvas: hero.view.camera_canvas,
            ..i
        })
    });
    // ⭐ As secções FACTORY e LIFECYCLE — `None` para quem não tem nenhum dos três (ADR-0166).
    //
    // ⚠️ **Ela pede a ÁRVORE e o RELÓGIO**: o caminho da tag dos pontos de nascimento lê-se da
    // árvore, e *«a corrida é o relógio a andar»* é a frase que separa «avariada» de «à espera».
    let inspector_factory = alvo.bits.and_then(|b| {
        crate::render_loop::inspector_factory::build_factory_info(
            sim.world_mut(),
            tags,
            b,
            selected_count,
            clock_playing,
        )
    });
    // ⭐ A secção TOP-DOWN PLAYER — `None` para quem não tem o componente (ADR-0166).
    //
    // ⚠️ Ela pede o RELÓGIO pela mesma razão da irmã: *«a corrida é o relógio a andar»* é a frase
    // que separa «avariado» de «à espera».
    let inspector_topdown = alvo.bits.and_then(|b| {
        crate::render_loop::inspector_topdown::build_topdown_info(
            sim.world(),
            b,
            selected_count,
            clock_playing,
        )
    });
    // ⭐ A secção PROJECTILE MOTION — `None` para quem não tem o componente (ADR-0166).
    //
    // ⚠️ Ela pede uma coisa a mais: **se o voo já acabou**. Sem isso um projéctil que fez
    // exactamente o que devia lê-se como partido — está parado no ar com todos os números certos.
    let inspector_projectile = alvo.bits.and_then(|b| {
        let acabou = projectile_over.contains(&b);
        ph2d_app_components::projectile_inspector::build_projectile_info(
            sim.world(),
            b,
            selected_count,
            clock_playing,
            acabou,
        )
    });
    // ⭐⭐⭐ A secção RAY SENSOR (suplente #21) — `None` para quem não tem o componente (ADR-0166).
    //
    // ⚠️ Ela pede a LEITURA VIVA, que é o que a distingue das irmãs: sem *«vê a Parede, a 1,75 m»*
    // ela seria seis campos numa tabela, e o artista não teria como afinar um alcance a olhar.
    let inspector_ray = alvo.bits.and_then(|b| {
        let visto = ray_hits
            .iter()
            .find(|(olha, ..)| *olha == b)
            .map(|&(_, v, d)| (ph2d_ecs::Entity::from_bits(v), d));
        ph2d_app_components::ray_inspector::build_ray_info(
            sim.world(),
            b,
            selected_count,
            clock_playing,
            visto,
        )
    });

    // ⭐⭐⭐ A secção PARALLAX (plano 24) — `None` para quem não tem `ScrollFactor` (ADR-0166).
    //
    // ⚠️ Ela lê a CENA e não só o objecto: *há uma câmera do jogo?* é a razão nº 1 para nada se
    // mexer, e sem ela o painel mandaria o artista afinar um factor que a lei nem chega a ler.
    let inspector_parallax = alvo.bits.and_then(|b| {
        ph2d_app_components::parallax_inspector::build_parallax_info(
            sim,
            b,
            selected_count,
            game_camera_preview,
            preview_drive.drives_other_than(b, ph2d_preview_drive::Driver::ParallaxPose),
        )
    });
    // ⭐⭐⭐ A secção WEAPON (ADR-0166) — pede a MUNIÇÃO VIVA: sem *«4 de 6 balas»* ela seria oito
    // campos numa tabela, e o artista não teria como afinar uma cadência a olhar.
    let inspector_weapon = alvo.bits.and_then(|b| {
        ph2d_app_components::weapon_inspector::build_info(sim, b, clock_playing, selected_count)
    });
    // ⭐⭐⭐ HEALTH/DAMAGE (plano 28) e NAV (plano 30) — família da FÍSICA, no irmão dela.
    let (inspector_vida, inspector_nav) =
        fisica::vida_e_nav(alvo.bits, sim, selected_count, clock_playing);
    // ⭐⭐⭐ A secção TWEEN (suplente #22) — `None` para quem não tem o componente (ADR-0166).
    //
    // ⚠️ Ela pede DUAS colunas que não vêm do componente — *há timer neste índice?* e *há sprite?*
    // —, e é delas que sai a queixa: sem isso, um tween sem relógio parece-se com um partido.
    let inspector_tween = alvo.bits.and_then(|b| {
        ph2d_app_components::tween_inspector::build_tween_info(sim.world(), b, selected_count)
    });
    // ⭐ PATH FOLLOW (suplente #23), `None` sem o componente; `world_mut` porque a busca por nome
    // regista o componente de identidade num mundo que ainda não o viu (a armadilha da `tagged`).
    let inspector_path_follow = alvo.bits.and_then(|b| {
        ph2d_app_components::path_follow_inspector::build_path_follow_info(
            sim.world_mut(),
            b,
            selected_count,
            clock_playing,
        )
    });
    // ⭐⭐⭐ A secção STATE MACHINE (TOP-20 #15) — `None` para quem não tem o componente (ADR-0166).
    //
    // ⚠️ Ela lê o VIVO (`StateMachineRuntime`) para dizer *«Now: …»*, e é isso que a torna útil com
    // o relógio a andar: sem o readout o artista tem de simular a tabela de cabeça.
    let inspector_statemachine = alvo.bits.and_then(|b| {
        ph2d_app_components::statemachine_inspector::build_statemachine_info(
            sim.world(),
            b,
            selected_count,
            clock_playing,
        )
    });
    let inspector_visibility_section = alvo.bits.and_then(|b| {
        ph2d_app_components::inspector_visibility::build_visibility_section_info(
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
        inspector_factory,
        inspector_topdown,
        inspector_projectile,
        inspector_ray,
        inspector_parallax,
        inspector_weapon,
        inspector_vida,
        inspector_nav,
        inspector_tween,
        inspector_path_follow,
        inspector_statemachine,
        inspector_visibility_section,
    }
}

#[path = "snapshots_inspector_physics.rs"]
mod fisica;
/// ⭐ A metade da FÍSICA mudou-se para um ficheiro irmão (o tecto de LOC do HR-18, 2026-09-15) —
/// ver o cabeçalho dele. ⚠️ O `#[path]` é o mesmo idioma das fases-filhas do `render_loop`.
#[cfg(feature = "panel-inspector")]
#[path = "snapshots_slot_image.rs"]
mod slot_image_snapshot;
use fisica::{PhysicsSections, physics};

/// O que o [`identity`] constrói, com os nomes que a [`publish`] publica.
struct Identity {
    inspector_transform: Option<ph2d_editor_core::InspectorTransformInfo>,
    inspector_visibility: Option<ph2d_editor_core::InspectorVisibilityInfo>,
    inspector_name: Option<ph2d_editor_core::InspectorNameInfo>,
}

/// O transform, a visibilidade (com o «Mixed» da multi-seleção) e o nome da seleção primária.
fn identity(alvo: &Alvo, sim: &SimWorld) -> Identity {
    let inspector_selection = alvo.todos.as_slice();
    // M14.A: live Transform snapshot for the inspector. Same
    // ADR-0021 / HR-8 boundary as sprite snapshot — Inspector
    // never reads SimWorld; the host bridges. Lands on every
    // entity that has a `Transform` component, not just sprites
    // (so non-renderable entities still show their pose).
    let inspector_transform = alvo.bits.and_then(|bits| {
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
    let inspector_visibility = alvo.bits.and_then(|bits| {
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
    let inspector_name = alvo.bits.and_then(|bits| {
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
