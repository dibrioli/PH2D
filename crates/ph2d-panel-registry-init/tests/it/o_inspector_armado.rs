//! ⭐⭐⭐ **O PONTO CEGO DECLARADO DA VARREDURA DE ELISÕES — o Inspector com um objecto na mão.**
//!
//! # ⛔⛔ O buraco
//!
//! A varredura irmã ([`super::nenhum_rotulo_do_app_pinta_nada`]) pinta **todo painel do registo no
//! estado de FÁBRICA**, e o estado de fábrica do Inspector é o **vazio**: sem selecção ele mede
//! **um** rótulo. As **28 portas condicionais** dele (`set_current_inspector_*`) ficavam, por
//! construção, fora de toda régua de largura deste repo — e o cabeçalho daquela varredura já o
//! declarava por escrito em 19/09:
//!
//! > *«O mesmo defeito estava vivo em todo chip da secção Tags do Inspector e em seis dos sete
//! > selos da Hierarquia, e esta varredura não podia vê-los: um painel de fábrica não tem objecto
//! > seleccionado.»*
//!
//! ⚠️ **Uma cegueira DECLARADA continua a ser uma cegueira.** O Inspector é o painel com mais
//! fileiras do app, e é o único cujo conteúdo depende do documento — logo é exactamente aquele em
//! que um rótulo cortado só aparece quando o artista escolhe o objecto certo.
//!
//! # ⭐⭐ O que este módulo é
//!
//! Uma **fixtura**: um objecto impossível que tem **TODAS** as secções ao mesmo tempo. ⚠️ Ele não
//! pretende ser um objecto que exista — pretende ser a UNIÃO das populações de rótulos, que é o que
//! uma régua de largura precisa. As secções não partilham colunas (cada fileira deriva a coluna do
//! próprio rótulo, `widget::property_row_columns`), logo armá-las juntas mede o mesmo que armá-las
//! uma a uma e custa **um** repintar em vez de 28.
//!
//! ⛔ **E as fileiras de LISTA trazem texto a sério** (tags, acções, animações, temporizadores,
//! âncoras, estados, propriedades de script): uma linha construída por `Default` tem a `String`
//! **vazia**, e *uma fixtura que não pinta palavra nenhuma mede-se igual a uma secção ausente*.
//!
//! # ⛔ A porta é DERIVADA, nunca uma lista escrita à mão
//!
//! [`toda_porta_do_inspector_e_armada`] lê as `pub fn set_current_inspector_*` do **fonte da crate
//! do painel** e exige que cada uma seja chamada aqui. Uma porta nova nasce **acusada** — é a única
//! forma de esta fixtura não envelhecer em silêncio, que é como as catracas deste repo viram
//! licença.

use ph2d_editor_core::action_trigger_edits::{
    InspectorActionTriggerInfo, InspectorTriggerRow, NoMapa,
};
use ph2d_editor_core::counter_watch_edits::{InspectorCounterWatchInfo, InspectorWatchRow};
use ph2d_editor_core::factory_edits::{
    InspectorFactory, InspectorFactoryInfo, InspectorLifecycle, InspectorSpawnWhere,
};
use ph2d_editor_core::hud_edits::InspectorHudInfo;
use ph2d_editor_core::particles_edits::InspectorParticlesInfo;
use ph2d_editor_core::path_follow_edits::InspectorPathFollowInfo;
use ph2d_editor_core::projectile_edits::InspectorProjectileInfo;
use ph2d_editor_core::ray_edits::InspectorRayInfo;
use ph2d_editor_core::screens::hero::{
    AddedRow, ApplyChoice, InspectorActionInfo, InspectorActionRow, InspectorAnchorInfo,
    InspectorAnchorRow, InspectorAnimInfo, InspectorAnimRow, InspectorAudioInfo,
    InspectorAudioSource, InspectorBlendInfo, InspectorCameraFollow, InspectorCameraInfo,
    InspectorCameraLimits, InspectorGameCamera, InspectorInstanceInfo, InspectorJointInfo,
    InspectorNameInfo, InspectorOrderingInfo, InspectorPhysicsInfo, InspectorPlayerInfo,
    InspectorPropertiesInfo, InspectorSamplingInfo, InspectorSliceInfo, InspectorSpriteInfo,
    InspectorSpriteSource, InspectorTagRow, InspectorTagsInfo, InspectorTimerInfo,
    InspectorTimerRow, InspectorTransformInfo, InspectorVisibilityInfo,
    InspectorVisibilitySectionInfo, InspectorWheelInfo, OrphanRow, PlayerLive, RemovedRow,
    VariantAxis, VariantChoice,
};
use ph2d_editor_core::script_edits::{
    InspectorScriptInfo, InspectorScriptOrphan, InspectorScriptProp, InspectorScriptStatus,
    InspectorScriptValue, PorqueOrfao,
};
use ph2d_editor_core::sequence_edits::InspectorSequenceInfo;
use ph2d_editor_core::shake_edits::{
    InspectorEmitterInfo, InspectorEmitterRow, InspectorShakeInfo,
};
use ph2d_editor_core::statemachine_edits::{
    InspectorStateMachineInfo, InspectorStateRow, InspectorTransitionRow,
};
use ph2d_editor_core::topdown_edits::{
    InspectorFacing, InspectorMoveDirections, InspectorTopDownInfo, InspectorViewpoint,
};
use ph2d_editor_core::tween_edits::{InspectorTweenInfo, InspectorTweenRow};
use ph2d_editor_core::weapon_edits::InspectorWeaponInfo;
use ph2d_panel_inspector as insp;

/// O objecto da fixtura. ⚠️ Um valor qualquer: nenhuma das leis medidas aqui o lê como endereço.
const BITS: u64 = 0x_A5_1D_E0;
/// A receita de que ele seria cópia (a secção Component compara os dois).
const RAIZ: u64 = 0x_A5_1D_E1;

// ─────────────────────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ A ARMAÇÃO
// ─────────────────────────────────────────────────────────────────────────────────────────────

thread_local! {
    /// ⭐⭐⭐ **QUANTOS objectos a fixtura declara seleccionados.**
    ///
    /// ⛔ Ela existe porque **vinte e uma** secções pintam uma frase gateada em
    /// `selected_count > 1`, e com a fixtura pregada em `1` nenhuma régua desta casa jamais as
    /// via: *uma fixtura que não contém o fenómeno não afirma nada sobre ele*.
    static SELECIONADOS: std::cell::Cell<usize> = const { std::cell::Cell::new(1) };
}

/// Quantos a fixtura declara agora.
pub fn selecionados() -> usize {
    SELECIONADOS.get()
}

/// ⚠️ Escreve-se ANTES do `arma_tudo` — os snapshots são construídos ali, de uma vez.
pub fn com_seleccao_de(n: usize) {
    SELECIONADOS.set(n);
}

/// **Arma as 28 portas condicionais do Inspector.**
///
/// ⚠️ Elas são `thread_local`, logo isto vale para a thread que chamar — que é a mesma que pinta.
#[allow(clippy::too_many_lines)]
pub fn arma_tudo() {
    // ⭐⭐⭐ **O facto do PAINEL** — a frase da selecção mora no cartão do topo desde 22/09, e
    //    sem esta linha a fixtura arma vinte e oito snapshots e **não** arma o único número que
    //    os resume. *Uma fixtura que não contém o fenómeno não afirma nada sobre ele.*
    insp::set_current_inspector_selecionados(selecionados());
    insp::set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: BITS,
        name: "Hero".to_string(),
    }));
    insp::set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: BITS,
        translation: [1.25, -0.5],
        rotation_rad: std::f32::consts::FRAC_PI_6,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    insp::set_current_inspector_visibility(Some(InspectorVisibilityInfo {
        entity_bits: BITS,
        visible: true,
        mixed: false,
    }));
    insp::set_current_inspector_sprite(Some(InspectorSpriteInfo {
        entity_bits: BITS,
        world_size: [1.0, 1.0],
        source_kind: InspectorSpriteSource::HandPacked {
            sheet: 1,
            region: 2,
        },
        source_precision: None,
        emissive: 0.0,
        sheet_label: Some("hero · idle_0".to_string()),
        source_pixels: Some((64, 64)),
        can_reimport: true,
        flip_x: false,
        flip_y: false,
        opacity: 1.0,
        tint_fill: false,
        hframes: 4,
        vframes: 2,
        frame: 1,
        tint: [1.0, 1.0, 1.0, 1.0],
        self_tint: [1.0, 1.0, 1.0, 1.0],
        per_corner_tint: [[1.0, 1.0, 1.0, 1.0]; 4],
        region_enabled: true,
        region_rect: [0.0, 0.0, 32.0, 32.0],
        region_filter_clip: true,
        centered: true,
        offset: [0.0, 0.0],
        selected_count: selecionados(),
        mixed: ph2d_editor_core::screens::hero::InspectorSpriteMixed::default(),
    }));
    insp::set_current_inspector_sampling(Some(InspectorSamplingInfo {
        entity_bits: BITS,
        filter_tag: 1,
        repeat_tag: 1,
        uv_scale: [1.0, 1.0],
        uv_offset: [0.0, 0.0],
        selected_count: selecionados(),
        mixed: ph2d_editor_core::screens::hero::InspectorSamplingMixed::default(),
    }));
    insp::set_current_inspector_blend(Some(InspectorBlendInfo {
        entity_bits: BITS,
        blend_tag: 1,
        selected_count: selecionados(),
        mixed: ph2d_editor_core::screens::hero::InspectorBlendMixed::default(),
    }));
    insp::set_current_inspector_visibility_section(Some(InspectorVisibilitySectionInfo {
        entity_bits: BITS,
        layer_mask: u32::MAX,
        clip_mode: 1,
        mask_mode: 1,
        alpha_cutoff: 0.5,
        mask_source: true,
        on_screen: true,
        rect: [0.0, 0.0, 4.0, 4.0],
        selected_count: selecionados(),
        mixed: ph2d_editor_core::screens::hero::InspectorVisibilityMixed::default(),
    }));
    insp::set_current_inspector_ordering(Some(InspectorOrderingInfo {
        entity_bits: BITS,
        z_index: Some(3),
        z_as_relative: true,
        show_behind_parent: false,
        sorting_layer: 1,
        order_in_layer: 2,
        y_sort_enabled: true,
        y_sort_point: 1,
        y_sort_axis: [0.0, 1.0],
        sorting_group: true,
        sort_at_root: true,
        top_level: false,
        selected_count: selecionados(),
        mixed: ph2d_editor_core::screens::hero::InspectorOrderingMixed::default(),
    }));
    insp::set_current_inspector_slice(Some(InspectorSliceInfo {
        entity_bits: BITS,
        present: true,
        draw_mode_tag: 1,
        borders: [4.0, 4.0, 4.0, 4.0],
        size: [2.0, 2.0],
        tile_modes: [1; 8],
        centre_tile_mode: 1,
        tile_mode_tag: 1,
        fill_center: true,
        selected_count: selecionados(),
        mixed: ph2d_editor_core::screens::hero::InspectorSliceMixed::default(),
    }));
    insp::set_current_inspector_anchor(Some(InspectorAnchorInfo {
        entity_bits: BITS,
        rows: vec![
            InspectorAnchorRow {
                name: "hand_right".to_string(),
                pos: [0.3, 0.1],
                rot_deg: 12.0,
                bounds: Some([0.0, 0.0, 0.2, 0.2]),
                center: Some([0.1, 0.1, 0.0, 0.0]),
                riders: 1,
            },
            InspectorAnchorRow {
                name: "muzzle".to_string(),
                pos: [0.5, 0.0],
                rot_deg: 0.0,
                bounds: None,
                center: None,
                riders: 0,
            },
        ],
        present: true,
        selected_count: selecionados(),
        mixed: false,
        parent_anchors: vec!["hand_right".to_string(), "back".to_string()],
        mount: Some("hand_right".to_string()),
        mount_offset: [0.0, 0.0],
        vis_in_editor: true,
        vis_at_runtime: false,
    }));
    insp::set_current_inspector_anim(Some(InspectorAnimInfo {
        entity_bits: BITS,
        rows: vec![InspectorAnimRow {
            name: "idle".to_string(),
            from: 0,
            to: 3,
            frame_ms: 100,
            direction_tag: 0,
            repeat: 0,
            hold_ms: 0,
            repeat_delay_ms: 0,
            signal_on_finish: "idle_done".to_string(),
            signal_on_loop: "idle_loop".to_string(),
            per_frame_ms: vec![100, 120, 100, 80],
        }],
        player_present: true,
        cells: 8,
        current: "idle".to_string(),
        playing: true,
        autoplay: true,
        speed: 1.0,
        direction_override_tag: 0,
        loop_override_tag: 0,
        frame: 1,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_instance(Some(InspectorInstanceInfo {
        entity_bits: BITS,
        master_name: "Hero Prefab".to_string(),
        overridden: vec!["Transform".to_string(), "Sprite".to_string()],
        orphan_rows: vec![OrphanRow {
            component: "AudioSource2D".to_string(),
            piece: "footsteps".to_string(),
            piece_id: 7,
            type_id: 9,
        }],
        removed_rows: vec![RemovedRow {
            piece_id: 11,
            name: "Shadow".to_string(),
        }],
        added_rows: vec![AddedRow {
            piece_id: 12,
            name: "Torch".to_string(),
            master_name: "Torch Prefab".to_string(),
        }],
        root_bits: RAIZ,
        is_variant: true,
        apply_levels: vec![ApplyChoice {
            master: RAIZ,
            name: "Hero Prefab".to_string(),
            innermost: true,
        }],
        apply_levels_beyond: 0,
    }));
    insp::set_current_inspector_properties(Some(InspectorPropertiesInfo {
        entity_bits: BITS,
        root_bits: RAIZ,
        rows: vec![
            VariantAxis {
                name: "size".to_string(),
                options: vec![
                    VariantChoice {
                        master: RAIZ,
                        label: "tall".to_string(),
                        current: true,
                    },
                    VariantChoice {
                        master: RAIZ + 1,
                        label: "short".to_string(),
                        current: false,
                    },
                ],
            },
            // ⭐⭐⭐ **UM EIXO QUE QUEBRA** — ele existe porque a cura do refluxo dos
            //    chips **nao era exercitada por fixtura nenhuma**: com dois rotulos curtos
            //    (`tall` / `short`) a fileira cabe em qualquer largura, e a mutacao que apagava a
            //    2.ª fileira SOBREVIVEU. *Um corpus que nunca faz a fileira quebrar nao testa a
            //    quebra* — a mesma lei do corpus no ponto NEUTRO de um knob.
            VariantAxis {
                name: "material".to_string(),
                options: vec![
                    VariantChoice {
                        master: RAIZ + 2,
                        label: "brushed steel".to_string(),
                        current: true,
                    },
                    VariantChoice {
                        master: RAIZ + 3,
                        label: "oxidised copper".to_string(),
                        current: false,
                    },
                    VariantChoice {
                        master: RAIZ + 4,
                        label: "matte plastic".to_string(),
                        current: false,
                    },
                ],
            },
        ],
        beyond: 0,
        source_name: Some("Hero (tall)".to_string()),
    }));
    insp::set_current_inspector_tags(Some(InspectorTagsInfo {
        entity_bits: BITS,
        on_object: vec![
            InspectorTagRow {
                id: 1,
                path: "Enemy".to_string(),
                label: "Enemy".to_string(),
                depth: 0,
            },
            InspectorTagRow {
                id: 2,
                path: "Enemy/Flying".to_string(),
                label: "Flying".to_string(),
                depth: 1,
            },
        ],
        full: false,
        selected_count: selecionados(),
    }));
    arma_a_fisica();
    arma_o_top20();
}

/// A metade de FÍSICA: corpo, junta e roldana.
fn arma_a_fisica() {
    insp::set_current_inspector_physics(Some(InspectorPhysicsInfo {
        entity_bits: BITS,
        has_body: true,
        kind_tag: 1,
        shape_tag: 1,
        radius: 0.5,
        half_x: 0.5,
        half_y: 0.5,
        cap_half_height: 0.25,
        density: 1.0,
        restitution: 0.2,
        friction: 0.6,
        layer: 1,
        offset: [0.0, 0.0],
        walk_grip: 1.0,
        walk_belt: 0.0,
        bake_seconds: 2.0,
        bake_start_seconds: 0.0,
        join_count: 1,
        rig_parts: 2,
        part_owner: "Hero".to_string(),
        has_collider: true,
        part_count: 2,
        join_draw_armed: false,
        join_kind_tag: 1,
        is_sensor: false,
        signal: "hit".to_string(),
        signal_leave: "left".to_string(),
        signal_tag: Some(1),
        signal_tag_path: "Enemy".to_string(),
        bake_channels_tag: 1,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: true,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_manual: false,
        mass_is_read: true,
        mass: 1.0,
        dominance: 0,
        restitution_combine_tag: 1,
        friction_combine_tag: 1,
        linear_damping: 0.1,
        angular_damping: 0.1,
        damp_mode_tag: 1,
        one_way: false,
        no_wall_cling: false,
        force: [0.0, 0.0],
        force_world_axes: true,
        area_drag: 0.0,
        area_density: 1.0,
        area_form_drag: 0.0,
        area_torque: 0.0,
        area_falloff: 0.0,
    }));
    insp::set_current_inspector_joint(Some(InspectorJointInfo {
        entity_bits: BITS,
        kind_tag: 1,
        body_a_name: "Hero".to_string(),
        body_b_name: "Platform".to_string(),
        bound: true,
        world_anchored: false,
        limits_enabled: true,
        limit_min_ui: -45.0,
        limit_max_ui: 45.0,
        motor_enabled: true,
        motor_mode_tag: 1,
        motor_speed_ui: 90.0,
        motor_target_ui: 0.0,
        motor_max_force: 10.0,
        rest_length: 1.0,
        stiffness: 20.0,
        damping: 1.0,
        max_length: 2.0,
        axis_mode_tag: [1, 1, 1],
        axis_min_ui: [-1.0, -1.0, -45.0],
        axis_max_ui: [1.0, 1.0, 45.0],
        motor_axis_tag: 1,
        pick_armed: 0,
        wheel_count: 1,
        break_enabled: true,
        break_force: 100.0,
        break_torque: 50.0,
        breaks_on_torque: false,
        active: true,
        soft: false,
        collide_connected: false,
        paste_targets: 1,
    }));
    insp::set_current_inspector_wheel(Some(InspectorWheelInfo {
        entity_bits: BITS,
        rope_name: "Rope".to_string(),
        bound: true,
        radius: 0.3,
        radius_out: 0.4,
        weston: true,
        gear: 2.0,
        order_ui: 1,
        wrap_tag: 1,
        motor_deg_per_s: 30.0,
        break_enabled: false,
        break_force: 0.0,
        mount_name: "Pivot".to_string(),
        mount_pick_armed: false,
        rope_pick_armed: false,
    }));
    insp::set_current_inspector_player(Some(InspectorPlayerInfo {
        entity_bits: BITS,
        has_player: true,
        mode_tag: 1,
        reaction_is_live: true,
        push_is_live: true,
        spring_is_live: true,
        float_height: 0.1,
        min_float_height: 0.05,
        min_float_known: true,
        cling_distance: 0.05,
        spring_strength: 50.0,
        spring_damping: 5.0,
        speed: 6.0,
        acceleration: 40.0,
        air_acceleration: 20.0,
        brake_scale: 1.0,
        max_slope_deg: 45.0,
        jump_height: 2.0,
        takeoff_gravity: 20.0,
        takeoff_speed: 8.0,
        peak_gravity: 12.0,
        peak_speed: 2.0,
        fall_gravity: 30.0,
        cut_gravity: 45.0,
        air_jumps: 1.0,
        air_jump_height: 1.5,
        coyote_time: 0.1,
        jump_buffer: 0.1,
        corner_reach: 0.2,
        corner_samples: 3.0,
        corner_lookahead: 0.2,
        lift_momentum: 1.0,
        wall_slide_speed: 2.0,
        wall_jump_height: 1.5,
        wall_jump_push: 4.0,
        wall_jump_lockout: 0.15,
        wall_reach: 0.1,
        foot_samples: 3.0,
        foot_spread: 0.4,
        wall_samples: 3.0,
        wall_spread: 0.4,
        wall_grab_stamina: 2.0,
        dash_speed: 14.0,
        dash_time: 0.15,
        dash_cooldown: 0.5,
        crouch_height: 0.5,
        crouch_speed: 3.0,
        swim_speed: 3.0,
        swim_acceleration: 10.0,
        swim_enter: 0.2,
        ledge_grab: 0.2,
        ledge_reach_y: 0.3,
        ledge_span: 0.2,
        ledge_offset_y: 0.1,
        ledge_speed: 2.0,
        glide_fall_speed: 1.5,
        max_fall_speed: 20.0,
        platform_lift: 1,
        walk_off_ledges: 1,
        crouch_walk_off_ledges: 1,
        crouch_armed: false,
        reaction_support: 1.0,
        reaction_movement: 1.0,
        reaction_push: 1.0,
        recorded_run_seconds: 3.5,
        discarded_run_seconds: 0.0,
        live: Some(PlayerLive {
            footing_tag: 1,
            facing: 1.0,
            velocity: [1.0, 0.0],
        }),
        emits_signals: true,
    }));
}

/// A metade do TOP-20: relógio, acções, som, câmera, fábrica, movimento, projéctil, cérebro,
/// partículas e script.
#[allow(clippy::too_many_lines)]
fn arma_o_top20() {
    insp::set_current_inspector_timer(Some(InspectorTimerInfo {
        entity_bits: BITS,
        rows: vec![InspectorTimerRow {
            name: "respawn".to_string(),
            duration_s: 2.5,
            repeat: true,
            autostart: true,
            signal: "respawn_done".to_string(),
        }],
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_action(Some(InspectorActionInfo {
        entity_bits: BITS,
        rows: vec![InspectorActionRow {
            on: "hit".to_string(),
            target: "Door".to_string(),
            verb_tag: 1,
            arg: "open".to_string(),
            uses_arg: true,
            // ⭐ Os TRÊS campos do suplente #24, que a `line/components` acrescentou no mesmo dia
            //    em que esta fixtura nasceu. ⚠️ Eles são postos ACESOS de propósito: esta fixtura
            //    existe para a varredura de elisões MEDIR os rótulos, e um controlo desligado não
            //    é pintado — *uma fixtura que não acende um controlo mede zero sobre ele*.
            uses_target: true,
            // `1` = `ActionTargetMode::Tag`, o modo COERENTE com o `target_tag` que esta linha já
            // declarava: com `Name` o `target_tag` nunca é lido e a fixtura contradizia-se.
            target_mode: 1,
            from_tag: 0,
            target_tag: Some(1),
            target_tag_path: "Enemy/Flying".to_string(),
        }],
        verb_labels: vec![
            "Show".to_string(),
            "Hide".to_string(),
            "Play Animation".to_string(),
        ],
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_audio(Some(InspectorAudioInfo {
        entity_bits: BITS,
        source: Some(InspectorAudioSource {
            sound: "footsteps.ogg".to_string(),
            volume_db: -6.0,
            pitch: 1.0,
            looping: true,
            autoplay: false,
            max_distance: 20.0,
            attenuation: 1.0,
            non_spatialized_radius: 1.0,
            panning_strength: 1.0,
            max_polyphony: 4,
            bus_tag: 0,
            file_missing: false,
            reachable_by_signal: true,
        }),
        is_listener: true,
        listener_count: 1,
        is_active_listener: true,
        bus_labels: vec!["Master".to_string(), "Sfx".to_string()],
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_camera(Some(InspectorCameraInfo {
        entity_bits: BITS,
        camera: InspectorGameCamera {
            height_world: 10.0,
            offset: [0.0, 0.0],
            priority: 0,
            dolly: 0.0,
            active: true,
            cull_mask: u32::MAX,
        },
        follow: Some(InspectorCameraFollow {
            target: "Hero".to_string(),
            damping: [0.2, 0.2],
            dead_zone: [0.5, 0.5],
            lookahead: [1.0, 0.0],
            offset: [0.0, 1.0],
            target_found: true,
        }),
        limits: Some(InspectorCameraLimits {
            min: [-20.0, -10.0],
            max: [20.0, 10.0],
            smaller_than_view: false,
        }),
        camera_count: 1,
        is_active_camera: true,
        preview_on: true,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_factory(Some(InspectorFactoryInfo {
        entity_bits: BITS,
        factory: Some(InspectorFactory {
            recipe: "Bullet".to_string(),
            recipe_found: true,
            on_signal: "fire".to_string(),
            spawn_where: InspectorSpawnWhere::Tagged,
            // ⭐ Campo novo da `line/components` (a MIRA da fábrica), ACESO pela mesma razão que os
            //    três do irmão acima: esta fixtura existe para os rótulos serem MEDIDOS.
            aim_from_spawner: true,
            area: [2.0, 2.0],
            tag: "Spawn/Left".to_string(),
            pick_random: true,
            burst: 3,
            alive_max: 16,
            total_max: 64,
            on_spawned: "spawned".to_string(),
            on_exhausted: "exhausted".to_string(),
            seed: 7,
            alive: 2,
        }),
        lifecycle: Some(InspectorLifecycle {
            lifetime_s: Some(3.0),
            on_death: "died".to_string(),
            outside_margin: Some(1.0),
        }),
        is_spawned: false,
        has_game_camera: true,
        clock_playing: true,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_topdown(Some(InspectorTopDownInfo {
        entity_bits: BITS,
        speed: 5.0,
        acceleration: 40.0,
        deceleration: 40.0,
        directions: InspectorMoveDirections::default(),
        viewpoint: InspectorViewpoint::default(),
        viewpoint_angle_deg: 30.0,
        facing: InspectorFacing::default(),
        turn_speed_deg: 540.0,
        min_slide_angle_deg: 5.0,
        max_slides: 4,
        default_controls: true,
        body_is_kinematic: true,
        has_body: true,
        conflicts_with_platformer: false,
        clock_playing: true,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_projectile(Some(InspectorProjectileInfo {
        entity_bits: BITS,
        initial_speed: 12.0,
        acceleration: 0.0,
        max_speed: 20.0,
        gravity: 0.0,
        bounciness: 1.0,
        max_bounces: 3,
        range: 30.0,
        face_velocity: true,
        homing_accel: 8.0,
        homing_target: "Hero".to_string(),
        homing_target_missing: false,
        body_is_kinematic: true,
        has_body: true,
        clock_playing: true,
        flight_over: false,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_statemachine(Some(InspectorStateMachineInfo {
        entity_bits: BITS,
        states: vec![
            InspectorStateRow {
                name: "Closed".to_string(),
                on_enter: "door_closed".to_string(),
                on_exit: "door_opening".to_string(),
                has_exit: true,
            },
            InspectorStateRow {
                name: "Open".to_string(),
                on_enter: "door_open".to_string(),
                on_exit: String::new(),
                has_exit: false,
            },
        ],
        transitions: vec![InspectorTransitionRow {
            from: 0,
            on: "toggle".to_string(),
            to: 1,
        }],
        initial: 0,
        current: Some(0),
        clock_playing: true,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_particles(Some(InspectorParticlesInfo {
        entity_bits: BITS,
        emitting: true,
        one_shot: false,
        amount: 16.0,
        life: 1.0,
        life_random: 0.0,
        explosiveness: 0.0,
        prewarm: 0.0,
        time_scale: 1.0,
        seed: 0.0,
        shape: 1,
        shape_size: [0.5, 0.5],
        speed: 4.0,
        speed_random: 0.0,
        angle: 90.0,
        spread: 30.0,
        gravity: [0.0, -9.8],
        damping: 0.0,
        size: 0.15,
        size_random: 0.0,
        size_end: 1.0,
        color: [1.0, 1.0, 1.0, 1.0],
        color_end: [1.0, 0.0, 0.0, 0.0],
        space: 0,
        start_on: "fire".to_string(),
        stop_on: "cease".to_string(),
        restart_on: "reset".to_string(),
        finished_signal: "burst_done".to_string(),
        clock_playing: true,
        alive: 7,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_script(Some(InspectorScriptInfo {
        entity_bits: BITS,
        source: "bob.luau".to_string(),
        status: InspectorScriptStatus::Ready,
        props: vec![
            InspectorScriptProp {
                name: "speed".to_string(),
                value: InspectorScriptValue::Number(2.0),
                own: true,
                options: Vec::new(),
                min: Some(0.0),
                max: Some(10.0),
                step: Some(0.1),
            },
            InspectorScriptProp {
                name: "greeting".to_string(),
                value: InspectorScriptValue::Text("hello".to_string()),
                own: false,
                options: Vec::new(),
                min: None,
                max: None,
                step: None,
            },
            InspectorScriptProp {
                name: "chatty".to_string(),
                value: InspectorScriptValue::Bool(true),
                own: false,
                options: Vec::new(),
                min: None,
                max: None,
                step: None,
            },
        ],
        orphans: vec![InspectorScriptOrphan {
            name: "legacy_speed".to_string(),
            value: InspectorScriptValue::Number(1.0),
            wants: PorqueOrfao::NaoDeclarado,
        }],
        kept: 1,
        failure: None,
        clock_playing: true,
        also_physics: false,
        selected_count: selecionados(),
    }));
    // ⭐⭐⭐ **AS NOVE PORTAS que a `line/components` acrescentou em 17–20/09.** Elas chegaram
    //    depois desta fixtura e o censo DERIVADO acusou-as na integração — que é exactamente
    //    para o que ele existe (*«a secção nº 29 chega, ninguém se lembra desta fixtura, e a
    //    régua de largura volta a medir um painel vazio em silêncio»*).
    // ⚠️ **Cada uma é armada com CONTEÚDO e não com um `default()`**: a varredura de elisões
    //    mede o que é PINTADO, e uma lista vazia não pinta uma única palavra.
    insp::set_current_inspector_action_trigger(Some(InspectorActionTriggerInfo {
        entity_bits: BITS,
        rows: vec![InspectorTriggerRow {
            action: "jump".to_string(),
            edge: 0,
            signal: "jumped".to_string(),
            no_mapa: NoMapa::Ligada,
        }],
        clock_playing: true,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_counter_watch(Some(InspectorCounterWatchInfo {
        entity_bits: BITS,
        rows: vec![InspectorWatchRow {
            counter: "Score".to_string(),
            compare: 1,
            value: 100,
            signal: "won".to_string(),
            once: true,
            scope_own: false,
            counter_existe: true,
            valor_vivo: Some(42),
        }],
        clock_playing: true,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_emitter(Some(InspectorEmitterInfo {
        entity_bits: BITS,
        rows: vec![InspectorEmitterRow {
            on: "explosion".to_string(),
            de: 1,
            forca: 0.8,
            dentro: 2.0,
            fora: 12.0,
        }],
        ha_camera_que_treme: true,
        clock_playing: true,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_shake(Some(InspectorShakeInfo {
        entity_bits: BITS,
        amplitude: 0.35,
        frequencia: 18.0,
        decaimento: 1.4,
        expoente: 2,
        semente: 9,
        trauma: 0.5,
        activa: true,
        clock_playing: true,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_hud(Some(InspectorHudInfo {
        entity_bits: BITS,
        has_canvas: true,
        canvas_parent: Some("HUD".to_string()),
        ref_w: 1920.0,
        ref_h: 1080.0,
        fit: 1,
        tem_camera: true,
        has_label: true,
        source: 1,
        source_name: "Score".to_string(),
        prefix: "Score: ".to_string(),
        suffix: " pts".to_string(),
        vivo: "Score: 42 pts".to_string(),
        has_button: true,
        signal: "pause".to_string(),
        disabled: false,
        has_counter: true,
        counter_name: "Score".to_string(),
        counter_start: 0.0,
        counter_keep: false,
        counter_value: 42,
    }));
    insp::set_current_inspector_path_follow(Some(InspectorPathFollowInfo {
        entity_bits: BITS,
        caminho: "Patrol".to_string(),
        nome_existe: true,
        nome_tem_forma: true,
        relogio: 0,
        relogios: 2,
        duracao_us: Some(2_000_000),
        repeat: true,
        autostart: true,
        ciclo: 1,
        familia: 2,
        modo: 1,
        ao_acabar: 1,
        deslocamento: 0.25,
        alinha: true,
        angulo: 90.0,
        lado: 0.5,
        clock_playing: true,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_ray(Some(InspectorRayInfo {
        entity_bits: BITS,
        origin_x: 0.0,
        origin_y: 0.5,
        dir_x: 1.0,
        dir_y: 0.0,
        reach: 8.0,
        layer: 3,
        on_enter: "spotted".to_string(),
        on_exit: "lost".to_string(),
        sees: "Player".to_string(),
        sees_at: 4.5,
        clock_playing: true,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_sequence(Some(InspectorSequenceInfo {
        entity_bits: BITS,
        container: "Cutscene".to_string(),
        nomes: vec!["Intro".to_string(), "Reveal".to_string()],
        escolhido: Some(1),
        duracao_da_cutscene: 6.5,
        tem_relogio: true,
        a_correr: true,
        duracao_do_relogio: 3.0,
        t: 1.25,
        clock_playing: true,
        vista_deixa_correr: true,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_tween(Some(InspectorTweenInfo {
        entity_bits: BITS,
        rows: vec![InspectorTweenRow {
            canal: 1,
            de: [0.0, 0.0, 0.0, 1.0],
            para: [1.0, 0.0, 0.0, 1.0],
            familia: 2,
            modo: 1,
            ao_acabar: 1,
            ciclo: 1,
            duracao_us: Some(500_000),
            repeat: false,
            autostart: true,
        }],
        tem_sprite: true,
        selected_count: selecionados(),
    }));
    insp::set_current_inspector_weapon(Some(InspectorWeaponInfo {
        entity_bits: BITS,
        on_signal: "fire".to_string(),
        cooldown_ms: 120,
        ammo_counter: "Ammo".to_string(),
        reload_ms: 900,
        reload_on: "reload".to_string(),
        on_fire: "shot".to_string(),
        on_empty: "click".to_string(),
        on_reloaded: "ready".to_string(),
        // ⭐ A RESERVA: o depósito de onde a recarga tira, e o que ele tem AGORA.
        reserve_counter: "Ammo Box".to_string(),
        reserva: Some(24),
        municao: Some(7),
        pente: 12,
        recarregando: false,
        clock_playing: true,
        selected_count: selecionados(),
    }));
}

/// **Desarma as 28 portas.** ⚠️ Sem isto a varredura de fábrica passaria a medir um Inspector
/// armado — *o estado que uma fixtura deixa para trás é o estado que a régua seguinte mede*.
pub fn desarma_tudo() {
    insp::set_current_inspector_selecionados(0);
    insp::set_current_inspector_action_trigger(None);
    insp::set_current_inspector_counter_watch(None);
    insp::set_current_inspector_emitter(None);
    insp::set_current_inspector_hud(None);
    insp::set_current_inspector_path_follow(None);
    insp::set_current_inspector_ray(None);
    insp::set_current_inspector_sequence(None);
    insp::set_current_inspector_shake(None);
    insp::set_current_inspector_tween(None);
    insp::set_current_inspector_weapon(None);
    insp::set_current_inspector_name(None);
    insp::set_current_inspector_transform(None);
    insp::set_current_inspector_visibility(None);
    insp::set_current_inspector_sprite(None);
    insp::set_current_inspector_sampling(None);
    insp::set_current_inspector_blend(None);
    insp::set_current_inspector_visibility_section(None);
    insp::set_current_inspector_ordering(None);
    insp::set_current_inspector_slice(None);
    insp::set_current_inspector_anchor(None);
    insp::set_current_inspector_anim(None);
    insp::set_current_inspector_instance(None);
    insp::set_current_inspector_properties(None);
    insp::set_current_inspector_tags(None);
    insp::set_current_inspector_physics(None);
    insp::set_current_inspector_joint(None);
    insp::set_current_inspector_wheel(None);
    insp::set_current_inspector_player(None);
    insp::set_current_inspector_timer(None);
    insp::set_current_inspector_action(None);
    insp::set_current_inspector_audio(None);
    insp::set_current_inspector_camera(None);
    insp::set_current_inspector_factory(None);
    insp::set_current_inspector_topdown(None);
    insp::set_current_inspector_projectile(None);
    insp::set_current_inspector_statemachine(None);
    insp::set_current_inspector_particles(None);
    insp::set_current_inspector_script(None);
}

/// ⭐⭐⭐ **A TABELA QUE DESARMA UMA PORTA DE CADA VEZ** — o preço de cada secção, em píxeis.
///
/// ⛔⛔ **A [`arma_tudo`] monta um objecto IMPOSSÍVEL**, e o cabeçalho deste ficheiro di-lo por
/// escrito: *«ele não pretende ser um objecto que exista»*. Ela serve a régua de LARGURA, que
/// precisa da união das populações de rótulos. ⚠️ **Ela NÃO serve a régua de ALTURA**: medir a
/// dívida de ecrã do Inspector nos `14 987 px` dela é pôr uma wave a perseguir um estado que
/// nenhum artista alcança.
///
/// ⇒ esta tabela deixa a fixtura **compor-se**: armar tudo e desarmar um subconjunto dá a altura
/// de um objecto que EXISTE, e desarmar uma porta de cada vez dá o **preço** de cada secção.
///
/// ⚠️ [`a_tabela_de_portas_cobre_todas_as_portas`] deriva a lista do fonte da [`desarma_tudo`] —
/// uma porta nova nasce acusada aqui também.
pub const PORTAS: &[(&str, fn())] = &[
    ("action_trigger", || {
        insp::set_current_inspector_action_trigger(None)
    }),
    ("counter_watch", || {
        insp::set_current_inspector_counter_watch(None)
    }),
    ("emitter", || insp::set_current_inspector_emitter(None)),
    ("hud", || insp::set_current_inspector_hud(None)),
    ("path_follow", || {
        insp::set_current_inspector_path_follow(None)
    }),
    ("ray", || insp::set_current_inspector_ray(None)),
    ("sequence", || insp::set_current_inspector_sequence(None)),
    ("shake", || insp::set_current_inspector_shake(None)),
    ("tween", || insp::set_current_inspector_tween(None)),
    ("weapon", || insp::set_current_inspector_weapon(None)),
    ("name", || insp::set_current_inspector_name(None)),
    ("transform", || insp::set_current_inspector_transform(None)),
    ("visibility", || {
        insp::set_current_inspector_visibility(None)
    }),
    ("sprite", || insp::set_current_inspector_sprite(None)),
    ("sampling", || insp::set_current_inspector_sampling(None)),
    ("blend", || insp::set_current_inspector_blend(None)),
    ("visibility_section", || {
        insp::set_current_inspector_visibility_section(None)
    }),
    ("ordering", || insp::set_current_inspector_ordering(None)),
    ("slice", || insp::set_current_inspector_slice(None)),
    ("anchor", || insp::set_current_inspector_anchor(None)),
    ("anim", || insp::set_current_inspector_anim(None)),
    ("instance", || insp::set_current_inspector_instance(None)),
    ("properties", || {
        insp::set_current_inspector_properties(None)
    }),
    ("tags", || insp::set_current_inspector_tags(None)),
    ("physics", || insp::set_current_inspector_physics(None)),
    ("joint", || insp::set_current_inspector_joint(None)),
    ("wheel", || insp::set_current_inspector_wheel(None)),
    ("player", || insp::set_current_inspector_player(None)),
    ("timer", || insp::set_current_inspector_timer(None)),
    ("action", || insp::set_current_inspector_action(None)),
    ("audio", || insp::set_current_inspector_audio(None)),
    ("camera", || insp::set_current_inspector_camera(None)),
    ("factory", || insp::set_current_inspector_factory(None)),
    ("topdown", || insp::set_current_inspector_topdown(None)),
    ("projectile", || {
        insp::set_current_inspector_projectile(None)
    }),
    ("statemachine", || {
        insp::set_current_inspector_statemachine(None)
    }),
    ("particles", || insp::set_current_inspector_particles(None)),
    ("script", || insp::set_current_inspector_script(None)),
];

// ─────────────────────────────────────────────────────────────────────────────────────────────
// ⛔⛔ O CENSO DERIVADO — uma porta nova nasce ACUSADA
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// O fonte desta fixtura, lido no momento da compilação.
const ESTE_FICHEIRO: &str = include_str!("o_inspector_armado.rs");

/// ⭐ As portas condicionais do Inspector — **lidas do fonte da crate do painel**, nunca de uma
/// lista escrita aqui.
fn portas_do_inspector() -> Vec<String> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ph2d-panel-inspector/src")
        .canonicalize()
        .expect("a crate do painel do Inspector é vizinha desta");
    let mut portas = Vec::new();
    for e in std::fs::read_dir(&dir).expect("o src do painel lê-se") {
        let p = e.expect("entrada de directório").path();
        if p.extension().is_none_or(|x| x != "rs") {
            continue;
        }
        let src = std::fs::read_to_string(&p).expect("ficheiro de fonte");
        for (i, _) in src.match_indices("pub fn set_current_inspector_") {
            let resto = &src[i + "pub fn set_current_inspector_".len()..];
            let Some(nome) = resto.split('(').next() else {
                continue;
            };
            // ⭐⭐⭐ **UMA PORTA DE SECÇÃO recebe um `Option`; um FACTO DO PAINEL não.**
            //
            // ⛔⛔ A 1.ª redacção colhia TODO `set_current_inspector_*` e presumia a semântica de
            //    `Option` (`Some(` arma, `(None)` desarma). Em 2026-09-22 chegou o primeiro que
            //    não é uma secção — `set_current_inspector_selecionados(usize)`, o número de
            //    objectos escolhidos, que é um facto do PAINEL — e os TRÊS gates deste ficheiro
            //    reprovaram sobre produto CERTO, um deles com um nome recortado a meio
            //    (`"selecionados(0);\n    "`).
            //
            // ⭐ O discriminador é DERIVADO da assinatura e não uma lista escrita à mão: *uma
            //   secção pode estar ausente, logo o setter dela aceita `None`.* Uma porta nova nasce
            //   coberta; um facto novo do painel nasce de fora, sem ninguém se lembrar de nada.
            let cabeca: String = resto.chars().take(200).collect();
            let assinatura = cabeca.split(')').next().unwrap_or("");
            if !assinatura.contains("Option<") {
                continue;
            }
            portas.push(format!("set_current_inspector_{nome}"));
        }
    }
    portas.sort_unstable();
    portas.dedup();
    portas
}

/// ⛔ **Piso de população.** *Uma varredura que lesse zero portas aprovaria uma fixtura vazia* —
/// a forma exacta que este repo já pagou com o censo por prefixo de nome.
const PISO_DE_PORTAS: usize = 28;

/// ⭐⭐⭐ **TODA PORTA CONDICIONAL DO INSPECTOR É ARMADA POR ESTA FIXTURA.**
///
/// ⚠️ É o que impede a cegueira de voltar por acréscimo: a secção nº 29 chega, ninguém se lembra
/// desta fixtura, e a régua de largura volta a medir um painel vazio **em silêncio**.
#[test]
fn toda_porta_do_inspector_e_armada() {
    let portas = portas_do_inspector();
    assert!(
        portas.len() >= PISO_DE_PORTAS,
        "o censo leu {} portas (piso {PISO_DE_PORTAS}) — ou a extracção deixou de casar, e uma \
         extracção que lê pouco devolve ZERO faltas e lê-se como aprovação",
        portas.len()
    );
    let em_falta: Vec<String> = portas
        .into_iter()
        .filter(|p| !ESTE_FICHEIRO.contains(&format!("insp::{p}(Some(")))
        .collect();
    assert!(
        em_falta.is_empty(),
        "estas secções do Inspector NÃO são armadas — a varredura de elisões continua cega a \
         elas:\n  {}",
        em_falta.join("\n  ")
    );
}

/// ⭐⭐ **E TODA PORTA É DESARMADA.**
///
/// ⛔ A metade que a suíte inteira paga se faltar: as portas são `thread_local` e o binário de
/// teste corre vários módulos na mesma thread. *Uma fixtura que não limpa o que armou transforma
/// todo gate seguinte num gate sobre outro documento.*
#[test]
fn toda_porta_do_inspector_e_desarmada() {
    let em_falta: Vec<String> = portas_do_inspector()
        .into_iter()
        .filter(|p| !ESTE_FICHEIRO.contains(&format!("insp::{p}(None)")))
        .collect();
    assert!(
        em_falta.is_empty(),
        "estas secções ficam ARMADAS depois da varredura:\n  {}",
        em_falta.join("\n  ")
    );
}

/// ⛔ **A [`PORTAS`] cobre TODAS as portas** — derivada do fonte da [`desarma_tudo`], nunca de uma
/// lista escrita à mão. Uma porta nova nasce acusada nos dois sítios.
#[test]
fn a_tabela_de_portas_cobre_todas_as_portas() {
    let corpo = ESTE_FICHEIRO
        .split_once("pub fn desarma_tudo() {")
        .expect("a desarma_tudo tem de existir")
        .1;
    let corpo = corpo.split_once("\n}").expect("o fim da desarma_tudo").0;
    let mut no_fonte: Vec<&str> = Vec::new();
    for pedaco in corpo.split("insp::set_current_inspector_").skip(1) {
        // ⚠️ **O `(None)` tem de ESTAR no pedaço.** Sem esta guarda, uma chamada que desarme um
        //    facto do painel com outro valor (`(0)`) faz o `split` devolver o resto INTEIRO do
        //    corpo como se fosse um nome de porta — foi o que aconteceu em 2026-09-22, e o gate
        //    acusou `"selecionados(0);\n    "`.
        if !pedaco.contains("(None)") {
            continue;
        }
        if let Some(n) = pedaco.split("(None)").next() {
            no_fonte.push(n);
        }
    }
    // ⚠️ O piso: sem ele, um `split` que devolvesse nada deixaria a comparação abaixo
    //    trivialmente verdadeira sobre uma tabela vazia.
    assert!(
        no_fonte.len() >= 30,
        "a leitura da `desarma_tudo` achou só {} portas — a varredura partiu-se",
        no_fonte.len()
    );
    let na_tabela: Vec<&str> = PORTAS.iter().map(|(n, _)| *n).collect();
    let em_falta: Vec<&&str> = no_fonte.iter().filter(|n| !na_tabela.contains(n)).collect();
    assert!(
        em_falta.is_empty(),
        "estas portas não estão na `PORTAS`: {em_falta:?}\n\
         ⇒ sem elas a régua de ALTURA não consegue desarmar a secção, e ela entra em todo cenário \
         «objecto real» como se fosse obrigatória."
    );
    let sobra: Vec<&&str> = na_tabela.iter().filter(|n| !no_fonte.contains(n)).collect();
    assert!(
        sobra.is_empty(),
        "estas entradas da `PORTAS` já não existem no painel: {sobra:?}"
    );
}
