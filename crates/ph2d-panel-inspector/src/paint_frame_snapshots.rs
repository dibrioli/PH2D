//! ⭐ **OS SNAPSHOTS VIVOS DE UM QUADRO do Inspector** — irmão de [`super`] por tecto de LOC
//! (2026-09-14: a secção TAGS levou o pai a `606` contra `600`).
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e a fronteira já estava escrita** no doc do bloco: *ler os
//! snapshots e decidir se alguma secção está viva* é uma pergunta só, e não é a mesma que *como
//! cada secção é embrulhada na moldura*, que é o que fica no pai.

use super::{any_live_section, physics_family_infos};

/// **Todos os snapshots vivos deste quadro, lidos de uma vez.**
///
/// ⚠️ **Extraído do `paint_inspector` em 2026-08-23**, quando a §11 o levou acima da tolerância.
/// Mas o corte não é só de LOC: ler os treze snapshots e decidir se **alguma** seção está viva é
/// uma pergunta só, e ela vivia espalhada por vinte e cinco linhas no meio do orquestrador.
///
/// ⚠️ **O `any_section` é DERIVADO aqui**, junto de quem o alimenta. Enquanto ele estava longe,
/// acrescentar um snapshot novo e esquecer a linha dele naquela lista não dava erro nenhum — dava
/// um painel que se declarava vazio com uma seção pintada dentro.
pub(crate) struct LiveSnapshots {
    pub transform_info: Option<ph2d_editor_core::screens::hero::InspectorTransformInfo>,
    pub sprite_info: Option<ph2d_editor_core::screens::hero::InspectorSpriteInfo>,
    pub visibility_info: Option<ph2d_editor_core::screens::hero::InspectorVisibilityInfo>,
    pub ordering_info: Option<ph2d_editor_core::screens::hero::InspectorOrderingInfo>,
    pub sampling_info: Option<ph2d_editor_core::screens::hero::InspectorSamplingInfo>,
    pub slice_info: Option<ph2d_editor_core::screens::hero::InspectorSliceInfo>,
    pub anchor_info: Option<ph2d_editor_core::screens::hero::InspectorAnchorInfo>,
    pub anim_info: Option<ph2d_editor_core::screens::hero::InspectorAnimInfo>,
    /// ⭐⭐⭐ **A secção TIMERS** (TOP-20 #2, W3) — `Some` só quando o objecto tem `Timers`.
    /// ⚠️ **ENTRA no `any_section`**, ao contrário da §5/§11/§12: aquelas três só existem sobre
    /// uma sprite, que o `sprite_info` já representa; um `Timers` vale para **qualquer** objecto
    /// (`ObjectKinds::ANY`), incluindo um objecto VAZIO — que sem esta linha mostraria o painel a
    /// dizer que não há nada por baixo de uma secção que está lá.
    pub timer_info: Option<ph2d_editor_core::screens::hero::InspectorTimerInfo>,
    /// ⭐⭐⭐ **A secção SIGNAL ACTIONS** — `Some` só quando o objecto tem `SignalActions`.
    /// ⚠️ **ENTRA no `any_section`**, pela razão do `timer_info`: ela vale para qualquer objecto.
    pub action_info: Option<ph2d_editor_core::screens::hero::InspectorActionInfo>,
    /// ⭐⭐⭐ **A secção TAGS** (TOP-20 #9) — `Some` só quando o objecto tem `Tags`.
    /// ⚠️ **ENTRA no `any_section`**, pela razão do `timer_info`: um objecto vazio é tão marcável
    /// quanto uma sprite (`ObjectKinds::ANY` no descritor).
    pub tags_info: Option<ph2d_editor_core::screens::hero::InspectorTagsInfo>,
    /// ⭐⭐⭐ **A secção AUDIO** (TOP-20 #4) — `Some` quando o objecto tem a FONTE, as ORELHAS, ou
    /// as duas. ⚠️ **ENTRA no `any_section`**, pela razão do `timer_info`: ela vale para qualquer
    /// objecto, e um objecto que só tenha o marcador de ouvinte não está representado por mais
    /// nenhum snapshot.
    pub audio_info: Option<ph2d_editor_core::screens::hero::InspectorAudioInfo>,
    /// ⭐⭐⭐ **A secção CAMERA** (TOP-20 #7) — a câmera, quem ela segue e a cerca dela.
    ///
    /// ⚠️ **ENTRA no `any_section`** pela mesma razão do `timer_info`: ela vale para qualquer
    /// objecto, e uma câmera fixa não está representada por mais nenhum snapshot.
    pub camera_info: Option<ph2d_editor_core::screens::hero::InspectorCameraInfo>,
    /// ⭐ O snapshot das secções FACTORY e LIFECYCLE (TOP-20 #11 e #12).
    pub factory_info: Option<ph2d_editor_core::screens::hero::InspectorFactoryInfo>,
    /// ⭐ O MOVER DE VISTA DE CIMA (TOP-20 #13).
    pub topdown_info: Option<ph2d_editor_core::topdown_edits::InspectorTopDownInfo>,
    /// ⭐ O PROJÉCTIL (TOP-20 #14).
    pub projectile_info: Option<ph2d_editor_core::projectile_edits::InspectorProjectileInfo>,
    /// ⭐⭐⭐ A secção RAY SENSOR (suplente #21).
    pub ray_info: Option<ph2d_editor_core::ray_edits::InspectorRayInfo>,
    pub parallax_info: Option<ph2d_editor_core::parallax_edits::InspectorParallaxInfo>,
    /// ⭐⭐⭐ A secção WEAPON — a arma do jogador.
    pub weapon_info: Option<ph2d_editor_core::weapon_edits::InspectorWeaponInfo>,
    /// ⭐ O snapshot do CÉREBRO (TOP-20 #15).
    pub statemachine_info: Option<ph2d_editor_core::statemachine_edits::InspectorStateMachineInfo>,
    /// ⭐ O snapshot do SCRIPT (TOP-20 #16).
    pub script_info: Option<ph2d_editor_core::script_edits::InspectorScriptInfo>,
    /// ⭐ O snapshot do EMISSOR DE PARTÍCULAS (TOP-20 #18).
    ///
    /// ⚠️ **ENTRA no `any_section`**, pela razão do `timer_info`: um emissor vale para qualquer
    /// objecto — inclusive um objecto VAZIO, que é o caso comum de uma tocha — e nenhum outro
    /// snapshot o representa.
    pub particles_info: Option<ph2d_editor_core::particles_edits::InspectorParticlesInfo>,
    /// ⭐⭐⭐ O HUD (TOP-20 #20).
    pub hud_info: Option<ph2d_editor_core::hud_edits::InspectorHudInfo>,
    /// ⭐ A CUTSCENE (TOP-20 #19).
    pub sequence_info: Option<ph2d_editor_core::sequence_edits::InspectorSequenceInfo>,
    /// ⭐ A VIGIA DO CONTADOR.
    pub watch_info: Option<ph2d_editor_core::counter_watch_edits::InspectorCounterWatchInfo>,
    /// ⭐⭐⭐ O GATILHO (suplente #24) — a mão de quem joga.
    ///
    /// ⚠️ **ENTRA no `any_section`**, pela razão do `timer_info`: um gatilho vale para qualquer
    /// objecto — inclusive um objecto VAZIO chamado «Controlos», que é o sítio natural para as
    /// teclas de um jogo — e nenhum outro snapshot o representa.
    pub trigger_info: Option<ph2d_editor_core::action_trigger_edits::InspectorActionTriggerInfo>,
    /// O TWEEN (suplente #22).
    pub tween_info: Option<ph2d_editor_core::tween_edits::InspectorTweenInfo>,
    /// ⭐⭐⭐ O SEGUIDOR DE CAMINHO (suplente #23) — o objecto que anda sobre a curva desenhada.
    pub path_follow_info: Option<ph2d_editor_core::path_follow_edits::InspectorPathFollowInfo>,
    /// ⭐⭐⭐ O ABANÃO DA CÂMERA (suplente #25) — *como* esta câmera treme.
    pub shake_info: Option<ph2d_editor_core::shake_edits::InspectorShakeInfo>,
    /// ⭐⭐⭐ O EMISSOR DE ABANÃO (suplente #25) — *ao ouvir o quê* este objecto abana a vista.
    pub emitter_info: Option<ph2d_editor_core::shake_edits::InspectorEmitterInfo>,
    pub blend_info: Option<ph2d_editor_core::screens::hero::InspectorBlendInfo>,
    pub physics_info: Option<ph2d_editor_core::screens::hero::InspectorPhysicsInfo>,
    pub joint_info: Option<ph2d_editor_core::screens::hero::InspectorJointInfo>,
    pub wheel_info: Option<ph2d_editor_core::screens::hero::InspectorWheelInfo>,
    pub player_info: Option<ph2d_editor_core::screens::hero::InspectorPlayerInfo>,
    /// ⭐ **A seção COMPONENT** (ADR-0164 / F5) — `Some` só quando o selecionado é peça de
    /// uma cópia. ⚠️ **Não entra no `any_section`**, e pela razão das outras três: uma peça de
    /// instância tem sempre `Transform`, logo o `transform_info` já a representa — contá-la
    /// outra vez não mudaria a resposta.
    pub instance_info: Option<ph2d_editor_core::screens::hero::InspectorInstanceInfo>,
    /// ⭐ **O CARTÃO DE PROPRIEDADES** — `Some` quando o nome do objecto (ou o do mestre dele)
    /// declara alguma. ⚠️ **Não entra no `any_section`**, pela razão dos irmãos: quem tem
    /// propriedades tem `Transform`, logo já está representado.
    pub properties_info: Option<ph2d_editor_core::screens::hero::InspectorPropertiesInfo>,
    pub name_present: bool,
    /// Alguma seção viva? ⚠️ A §5 9-Slice, a §11 Animation e a §12 Sockets **não** entram nesta
    /// conta, e é deliberado: as três só existem sobre uma sprite, que já está representada pelo
    /// `sprite_info`. Contá-las outra vez não mudaria a resposta.
    pub any_section: bool,
}

impl LiveSnapshots {
    /// Lê os treze do estado do painel e decide o `any_section`.
    pub(crate) fn fetch() -> Self {
        let transform_info = crate::state::current_inspector_transform();
        let sprite_info = crate::state::current_inspector_sprite();
        let visibility_info = crate::state::current_inspector_visibility();
        let ordering_info = crate::state::current_inspector_ordering();
        let sampling_info = crate::state::current_inspector_sampling();
        let blend_info = crate::state::current_inspector_blend();
        let (physics_info, joint_info, wheel_info, player_info) = physics_family_infos();
        let instance_info = crate::state::current_inspector_instance();
        let properties_info = crate::state::current_inspector_properties();
        let name_present = crate::state::current_inspector_name_is_some();
        let timer_info = crate::state_components::current_inspector_timer();
        let action_info = crate::state_components::current_inspector_action();
        let audio_info = crate::state_components::current_inspector_audio();
        let camera_info = crate::state_components::current_inspector_camera();
        let factory_info = crate::state_components::current_inspector_factory();
        let topdown_info = crate::state_components::current_inspector_topdown();
        let projectile_info = crate::state_components::current_inspector_projectile();
        let ray_info = crate::state_components::current_inspector_ray();
        let parallax_info = crate::state_components::current_inspector_parallax();
        let weapon_info = crate::state_components::current_inspector_weapon();
        let statemachine_info = crate::state_components::current_inspector_statemachine();
        let script_info = crate::state_components::current_inspector_script();
        let particles_info = crate::state_components::current_inspector_particles();
        let hud_info = crate::state_components::current_inspector_hud();
        let sequence_info = crate::state_components::current_inspector_sequence();
        let watch_info = crate::state_components::current_inspector_counter_watch();
        let trigger_info = crate::state_components::current_inspector_action_trigger();
        let tween_info = crate::state_components::current_inspector_tween();
        let path_follow_info = crate::state_components::current_inspector_path_follow();
        let shake_info = crate::state_components::current_inspector_shake();
        let emitter_info = crate::state_components::current_inspector_emitter();
        let tags_info = crate::state::current_inspector_tags();
        let any_section = any_live_section([
            transform_info.is_some(),
            sprite_info.is_some(),
            visibility_info.is_some(),
            ordering_info.is_some(),
            sampling_info.is_some(),
            blend_info.is_some(),
            physics_info.is_some(),
            joint_info.is_some(),
            wheel_info.is_some(),
            player_info.is_some(),
            name_present,
            timer_info.is_some(),
            action_info.is_some(),
            audio_info.is_some(),
            camera_info.is_some(),
            factory_info.is_some(),
            topdown_info.is_some(),
            projectile_info.is_some(),
            ray_info.is_some(),
            parallax_info.is_some(),
            weapon_info.is_some(),
            statemachine_info.is_some(),
            script_info.is_some(),
            particles_info.is_some(),
            hud_info.is_some(),
            sequence_info.is_some(),
            watch_info.is_some(),
            trigger_info.is_some(),
            tween_info.is_some(),
            path_follow_info.is_some(),
            shake_info.is_some(),
            emitter_info.is_some(),
            tags_info.is_some(),
        ]);
        Self {
            transform_info,
            sprite_info,
            visibility_info,
            ordering_info,
            sampling_info,
            slice_info: crate::state::current_inspector_slice(),
            anchor_info: crate::state::current_inspector_anchor(),
            anim_info: crate::state::current_inspector_anim(),
            timer_info,
            action_info,
            audio_info,
            camera_info,
            factory_info,
            topdown_info,
            projectile_info,
            ray_info,
            parallax_info,
            weapon_info,
            statemachine_info,
            script_info,
            particles_info,
            hud_info,
            sequence_info,
            watch_info,
            trigger_info,
            tween_info,
            path_follow_info,
            shake_info,
            emitter_info,
            tags_info,
            blend_info,
            physics_info,
            joint_info,
            wheel_info,
            player_info,
            instance_info,
            properties_info,
            name_present,
            any_section,
        }
    }
}
