//! ⭐ **T61 — o RETRATO de um objecto** (`docs/CartoesUI/04_plano_de_construcao.md`, bloco 10): tudo
//! o que a shell publica ao Inspector sobre UM objecto, numa lista única. A shell publica cada
//! objecto preso por um cartão solto pelas MESMAS portas da seleção ([`com_cada_preso`]); o painel
//! instala o retrato do preso à volta da pintura, da semente e do gesto do cartão dele ([`com_preso`]).
//! O que é do projecto (unidade, árvore de tags) fica fora: é igual para todos.

use crate::slot_image::{self, SlotImage};
use crate::{state as s, state_components as c};
use ph2d_editor_core::screens::hero as h;
use ph2d_editor_core::{
    action_trigger_edits as at, constraint_edits as ce, counter_watch_edits as cw, hud_edits,
    luz_edits as luz, macro_edits, motion_input_edits as mi, nav_edits, parallax_edits,
    particles_edits, path_follow_edits, projectile_edits, ray_edits, script_edits, sequence_edits,
    shake_edits as sh, statemachine_edits, topdown_edits, tween_edits, vida_edits, weapon_edits,
};
use std::cell::RefCell;
use std::collections::BTreeMap;

macro_rules! retrato {
    ($($campo:ident: $tipo:ty = $le:expr, $poe:path;)*) => {
        /// Ver o cabeçalho do módulo.
        #[derive(Clone, Debug, Default, PartialEq)]
        pub(crate) struct Retrato {
            $($campo: $tipo,)*
        }

        impl Retrato {
            /// O que está publicado agora.
            pub(crate) fn captura() -> Self {
                Self { $($campo: $le,)* }
            }

            /// Publica este retrato no lugar do de agora.
            pub(crate) fn instala(self) {
                $($poe(self.$campo);)*
            }
        }
    };
}

retrato! {
    selecionados: usize =
        s::current_inspector_selecionados(), s::set_current_inspector_selecionados;
    sprite: Option<h::InspectorSpriteInfo> =
        s::current_inspector_sprite(), s::set_current_inspector_sprite;
    transform: Option<h::InspectorTransformInfo> =
        s::current_inspector_transform(), s::set_current_inspector_transform;
    visibility: Option<h::InspectorVisibilityInfo> =
        s::current_inspector_visibility(), s::set_current_inspector_visibility;
    instance: Option<h::InspectorInstanceInfo> =
        s::current_inspector_instance(), s::set_current_inspector_instance;
    properties: Option<h::InspectorPropertiesInfo> =
        s::current_inspector_properties(), s::set_current_inspector_properties;
    name: Option<h::InspectorNameInfo> = s::current_inspector_name(), s::set_current_inspector_name;
    ordering: Option<h::InspectorOrderingInfo> =
        s::current_inspector_ordering(), s::set_current_inspector_ordering;
    sampling: Option<h::InspectorSamplingInfo> =
        s::current_inspector_sampling(), s::set_current_inspector_sampling;
    anim: Option<h::InspectorAnimInfo> = s::current_inspector_anim(), s::set_current_inspector_anim;
    anchor: Option<h::InspectorAnchorInfo> =
        s::current_inspector_anchor(), s::set_current_inspector_anchor;
    slice: Option<h::InspectorSliceInfo> =
        s::current_inspector_slice(), s::set_current_inspector_slice;
    blend: Option<h::InspectorBlendInfo> =
        s::current_inspector_blend(), s::set_current_inspector_blend;
    joint: Option<h::InspectorJointInfo> =
        s::current_inspector_joint(), s::set_current_inspector_joint;
    wheel: Option<h::InspectorWheelInfo> =
        s::current_inspector_wheel(), s::set_current_inspector_wheel;
    player: Option<h::InspectorPlayerInfo> =
        s::current_inspector_player(), s::set_current_inspector_player;
    physics: Option<h::InspectorPhysicsInfo> =
        s::current_inspector_physics(), s::set_current_inspector_physics;
    visibility_section: Option<h::InspectorVisibilitySectionInfo> =
        s::current_inspector_visibility_section(), s::set_current_inspector_visibility_section;
    tags: Option<h::InspectorTagsInfo> = s::current_inspector_tags(), s::set_current_inspector_tags;
    timer: Option<h::InspectorTimerInfo> =
        c::current_inspector_timer(), c::set_current_inspector_timer;
    tween: Option<tween_edits::InspectorTweenInfo> =
        c::current_inspector_tween(), c::set_current_inspector_tween;
    path_follow: Option<path_follow_edits::InspectorPathFollowInfo> =
        c::current_inspector_path_follow(), c::set_current_inspector_path_follow;
    action: Option<h::InspectorActionInfo> =
        c::current_inspector_action(), c::set_current_inspector_action;
    audio: Option<h::InspectorAudioInfo> =
        c::current_inspector_audio(), c::set_current_inspector_audio;
    camera: Option<h::InspectorCameraInfo> =
        c::current_inspector_camera(), c::set_current_inspector_camera;
    factory: Option<h::InspectorFactoryInfo> =
        c::current_inspector_factory(), c::set_current_inspector_factory;
    topdown: Option<topdown_edits::InspectorTopDownInfo> =
        c::current_inspector_topdown(), c::set_current_inspector_topdown;
    projectile: Option<projectile_edits::InspectorProjectileInfo> =
        c::current_inspector_projectile(), c::set_current_inspector_projectile;
    ray: Option<ray_edits::InspectorRayInfo> =
        c::current_inspector_ray(), c::set_current_inspector_ray;
    parallax: Option<parallax_edits::InspectorParallaxInfo> =
        c::current_inspector_parallax(), c::set_current_inspector_parallax;
    weapon: Option<weapon_edits::InspectorWeaponInfo> =
        c::current_inspector_weapon(), c::set_current_inspector_weapon;
    nav: Option<nav_edits::InspectorNavInfo> =
        c::current_inspector_nav(), c::set_current_inspector_nav;
    vida: Option<vida_edits::InspectorVidaInfo> =
        c::current_inspector_vida(), c::set_current_inspector_vida;
    statemachine: Option<statemachine_edits::InspectorStateMachineInfo> =
        c::current_inspector_statemachine(), c::set_current_inspector_statemachine;
    hud: Option<hud_edits::InspectorHudInfo> =
        c::current_inspector_hud(), c::set_current_inspector_hud;
    sequence: Option<sequence_edits::InspectorSequenceInfo> =
        c::current_inspector_sequence(), c::set_current_inspector_sequence;
    counter_watch: Option<cw::InspectorCounterWatchInfo> =
        c::current_inspector_counter_watch(), c::set_current_inspector_counter_watch;
    action_trigger: Option<at::InspectorActionTriggerInfo> =
        c::current_inspector_action_trigger(), c::set_current_inspector_action_trigger;
    shake: Option<sh::InspectorShakeInfo> =
        c::current_inspector_shake(), c::set_current_inspector_shake;
    emitter: Option<sh::InspectorEmitterInfo> =
        c::current_inspector_emitter(), c::set_current_inspector_emitter;
    luz: Option<luz::InspectorLuzInfo> = c::current_inspector_luz(), c::set_current_inspector_luz;
    corpo_luz: Option<luz::InspectorCorpoDaLuzInfo> =
        c::current_inspector_corpo_luz(), c::set_current_inspector_corpo_luz;
    constraints: Option<ce::InspectorConstraintsInfo> =
        c::current_inspector_constraints(), c::set_current_inspector_constraints;
    particles: Option<particles_edits::InspectorParticlesInfo> =
        c::current_inspector_particles(), c::set_current_inspector_particles;
    macro_: Option<macro_edits::InspectorMacroInfo> =
        c::current_inspector_macro(), c::set_current_inspector_macro;
    motion_input: Option<mi::InspectorMotionInputInfo> =
        c::current_inspector_motion_input(), c::set_current_inspector_motion_input;
    script: Option<script_edits::InspectorScriptInfo> =
        c::current_inspector_script(), c::set_current_inspector_script;
    slot: Option<SlotImage> = slot_image::atual(), slot_image::set_current_slot_image;
}

thread_local! {
    static PRESOS: RefCell<BTreeMap<u64, Retrato>> = RefCell::default();
}

/// ⭐ A shell: corre `publica` uma vez por objecto preso — com o retrato dele instalado, pelas
/// portas da seleção — e guarda o que ela deixou publicado. No fim volta o da seleção.
pub fn com_cada_preso(pins: &[u64], mut publica: impl FnMut(u64)) {
    let selecao = Retrato::captura();
    PRESOS.with(|p| p.borrow_mut().retain(|b, _| pins.contains(b)));
    for &b in pins {
        // O de antes (as fases publicam metades); o primeiro parte do vazio, nunca da seleção.
        let r = PRESOS.with(|p| p.borrow_mut().remove(&b));
        r.unwrap_or_default().instala();
        publica(b);
        let r = Retrato::captura();
        PRESOS.with(|p| p.borrow_mut().insert(b, r));
    }
    selecao.instala();
}

/// `f` com o retrato de `bits` instalado, se ele estiver publicado; senão `f` com o de agora.
pub(crate) fn com_retrato<R>(bits: Option<u64>, f: impl FnOnce() -> R) -> R {
    let r = bits.and_then(|b| PRESOS.with(|p| p.borrow().get(&b).cloned()));
    let Some(r) = r else { return f() };
    let antes = Retrato::captura();
    r.instala();
    let out = f();
    antes.instala();
    out
}

/// O retrato publicado do preso `bits`.
pub(crate) fn de(bits: u64) -> Option<Retrato> {
    PRESOS.with(|p| p.borrow().get(&bits).cloned())
}

/// `f` com o retrato do preso `bits` no lugar do da seleção (`None` = ainda não publicado).
pub(crate) fn com_preso<R>(bits: u64, f: impl FnOnce() -> R) -> Option<R> {
    let r = PRESOS.with(|p| p.borrow().get(&bits).cloned())?;
    let antes = Retrato::captura();
    r.instala();
    let out = f();
    antes.instala();
    Some(out)
}
