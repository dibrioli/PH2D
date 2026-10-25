//! ⭐ **Os ITENS das listas dos componentes** ([`crate::lista`], F30 §A1) — cada campo de item com a
//! chave serde dentro do item e a `View` que o faz canal (as escolhas). O rótulo e o `field_id` são
//! os do descritor do componente quando ele os tem (gate `o_item_e_o_mesmo_campo_do_componente`).

use crate::lista::{ListaDesc, codigo as c};
use crate::{FieldDesc, FieldKind as K, Propagation, View, WHOLE};

const fn f(field_id: u16, label_key: &'static str, kind: K, key: &'static str) -> FieldDesc {
    FieldDesc {
        field_id,
        label_key,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
        name: "",
        key,
        view: View::Same,
    }
}

const DE_QUEM: View = crate::options![
    "anyone" => Anyone @ "ecs.signal_from.anyone",
    "myself" => Myself @ "ecs.signal_from.myself",
];

const TIMER: &[FieldDesc] = &[
    f(1, "component.field.timer_fields.1", K::Text, "name"),
    f(
        2,
        "component.field.timer_fields.2",
        K::Scalar,
        "duration_us",
    ),
    f(3, "component.field.timer_fields.3", K::Toggle, "repeat"),
    f(4, "component.field.timer_fields.4", K::Toggle, "autostart"),
    f(5, "component.field.timer_fields.5", K::Text, "signal"),
];

const TWEEN: &[FieldDesc] = &[
    f(1, "component.field.tween_fields.1", K::Enum, "canal").viewed(crate::options![
        "opacity" => Opacity @ "tween.canal.opacity",
        "tint" => Tint @ "tween.canal.tint",
        "silhouette" => Silhueta @ "tween.canal.silhueta",
        "position_x" => PositionX @ "tween.canal.position_x",
        "position_y" => PositionY @ "tween.canal.position_y",
        "scale_x" => ScaleX @ "tween.canal.scale_x",
        "scale_y" => ScaleY @ "tween.canal.scale_y",
        "rotation" => Rotation @ "tween.canal.rotation",
    ]),
    f(2, "component.field.tween_fields.2", K::Vec4, "de"),
    f(3, "component.field.tween_fields.3", K::Vec4, "para"),
    f(
        4,
        "component.field.tween_fields.4",
        K::Enum,
        "easing/family",
    )
    .viewed(crate::options![
        "linear" => Linear @ "anim.easing.family.linear",
        "quad" => Quad @ "anim.easing.family.quad",
        "cubic" => Cubic @ "anim.easing.family.cubic",
        "quart" => Quart @ "anim.easing.family.quart",
        "quint" => Quint @ "anim.easing.family.quint",
        "back" => Back @ "anim.easing.family.back",
        "bounce" => Bounce @ "anim.easing.family.bounce",
        "sine" => Sine @ "anim.easing.family.sine",
        "expo" => Expo @ "anim.easing.family.expo",
        "circ" => Circ @ "anim.easing.family.circ",
        "elastic" => Elastic @ "anim.easing.family.elastic",
    ]),
    f(5, "component.field.tween_fields.5", K::Enum, "easing/mode").viewed(crate::options![
        "in" => In @ "anim.easing.mode.in",
        "out" => Out @ "anim.easing.mode.out",
        "in_out" => InOut @ "anim.easing.mode.in_out",
    ]),
    f(6, "component.field.tween_fields.6", K::Enum, "ao_acabar").viewed(crate::options![
        "hold" => Hold @ "tween.ao_acabar.hold",
        "rewind" => Rewind @ "tween.ao_acabar.rewind",
    ]),
    f(7, "component.field.tween_fields.7", K::Enum, "ciclo").viewed(crate::options![
        "restart" => Reinicia @ "tween.ciclo.reinicia",
        "ping_pong" => PingPong @ "tween.ciclo.ping_pong",
    ]),
];

const ESTADO: &[FieldDesc] = &[
    f(1, "component.field.state_machine_fields.1", K::Text, "name"),
    f(
        2,
        "component.field.state_machine_fields.2",
        K::Text,
        "on_enter",
    ),
    f(
        3,
        "component.field.state_machine_fields.3",
        K::Text,
        "on_exit",
    ),
];

/// ⚠️ `from`/`to` são ÍNDICES de estado: discretos (`Int`), nunca interpolados.
const SETA: &[FieldDesc] = &[
    f(4, "component.field.state_machine_fields.4", K::Int, "from"),
    f(5, "component.field.state_machine_fields.5", K::Text, "on"),
    f(6, "component.field.state_machine_fields.6", K::Int, "to"),
];

/// ⚠️ A comparação não tem palavra (o chip é `≤ ≥ =`): o rótulo vazio deixa o número.
const VIGIA: &[FieldDesc] = &[
    f(1, "component.field.watch_fields.1", K::Text, "counter"),
    f(2, "component.field.watch_fields.2", K::Enum, "compare").viewed(crate::options![
        "at_most" => AtMost @ "",
        "at_least" => AtLeast @ "",
        "exactly" => Exactly @ "",
    ]),
    f(3, "component.field.watch_fields.3", K::Int, "value"),
    f(4, "component.field.watch_fields.4", K::Text, "signal"),
    f(5, "component.field.watch_fields.5", K::Toggle, "once"),
    f(
        6,
        "panel.inspector.counter_watch.only_this_object",
        K::Enum,
        "scope",
    )
    .viewed(View::Toggle(stringify!(Own), stringify!(World))),
];

/// ⚠️ A TAG do alvo (`target_by: Tagged(id)`) é uma referência: o canal escolhe o modo e guarda a tag.
const ACAO: &[FieldDesc] = &[
    f(1, "component.field.action_fields.1", K::Text, "on"),
    f(2, "component.field.action_fields.2", K::Text, "target"),
    f(3, "component.field.action_fields.3", K::Enum, "verb").viewed(crate::options![
        "start_timer" => StartTimer @ "ecs.signal_verb.start_timer",
        "stop_timer" => StopTimer @ "ecs.signal_verb.stop_timer",
        "show" => Show @ "ecs.signal_verb.show",
        "hide" => Hide @ "ecs.signal_verb.hide",
        "toggle_visibility" => ToggleVisibility @ "ecs.signal_verb.toggle_visibility",
        "play_sound" => PlaySound @ "ecs.signal_verb.play_sound",
        "stop_sound" => StopSound @ "ecs.signal_verb.stop_sound",
        "add_to_counter" => AddToCounter @ "ecs.signal_verb.add_to_counter",
        "destroy" => Destroy @ "ecs.signal_verb.destroy",
        "restart_run" => RestartRun @ "ecs.signal_verb.restart_run",
        "damage" => Damage @ "ecs.signal_verb.damage",
        "heal" => Heal @ "ecs.signal_verb.heal",
        "start_navigation" => StartNavigation @ "ecs.signal_verb.start_navigation",
        "stop_navigation" => StopNavigation @ "ecs.signal_verb.stop_navigation",
        "play_clip" => PlayClip @ "ecs.signal_verb.play_clip",
    ]),
    f(4, "component.field.action_fields.4", K::Text, "arg"),
    f(5, "panel.inspector.actions.target_by", K::Enum, "target_by").viewed(crate::options![
        "name" => Named @ "panel.inspector.actions.name",
        "tag" => {"Tagged": 0} @ "panel.inspector.actions.tag",
        "who_hit" => Other @ "panel.inspector.actions.who_hit",
    ]),
    f(6, "panel.inspector.actions.source", K::Enum, "from").viewed(DE_QUEM),
];

const GATILHO: &[FieldDesc] = &[
    f(1, "component.field.trigger_fields.1", K::Text, "action"),
    f(2, "component.field.trigger_fields.2", K::Enum, "edge").viewed(crate::options![
        "on_press" => Press @ "panel.inspector.trigger.on_press",
        "on_release" => Release @ "panel.inspector.trigger.on_release",
        "while_held" => Hold @ "panel.inspector.trigger.while_held",
    ]),
    f(3, "component.field.trigger_fields.3", K::Text, "signal"),
];

const FONTE_DE_ABANAO: &[FieldDesc] = &[
    f(0, "component.field.shake_source_fields.0", K::Text, "on"),
    f(1, "component.field.shake_source_fields.1", K::Enum, "de").viewed(DE_QUEM),
    f(
        2,
        "component.field.shake_source_fields.2",
        K::Scalar,
        "forca",
    ),
    f(
        3,
        "component.field.shake_source_fields.3",
        K::Scalar,
        "dentro",
    ),
    f(
        4,
        "component.field.shake_source_fields.4",
        K::Scalar,
        "fora",
    ),
];

const RESISTENCIA: &[FieldDesc] = &[
    f(1, "panel.inspector.vida.resist_kind", K::Text, "kind"),
    f(2, "panel.inspector.vida.resist_rate", K::Scalar, "rate"),
    f(
        3,
        "panel.inspector.vida.resist_absorbs",
        K::Toggle,
        "absorbs",
    ),
];

/// ⚠️ A caixa e o miolo só são canal quando a âncora os TEM (`null` não publica): pôr e tirar é a
/// forma da âncora, não um valor.
const ANCORA: &[FieldDesc] = &[
    f(1, "panel.inspector.anchors.name_label", K::Text, "name"),
    f(
        2,
        "panel.inspector.anchors.position_x_y_px",
        K::Vec2,
        "transform/translation",
    ),
    f(
        3,
        "panel.inspector.anchors.rotation_deg",
        K::Angle,
        "transform/rotation",
    ),
    f(
        4,
        "panel.inspector.anchors.bounds_x_y_w_h",
        K::Vec4,
        "bounds",
    ),
    f(
        5,
        "panel.inspector.anchors.center_x_y_w_h",
        K::Vec4,
        "center",
    ),
];

const fn l(
    codigo: u16,
    componente: &'static str,
    caminho: &'static str,
    campos: &'static [FieldDesc],
) -> ListaDesc {
    ListaDesc {
        codigo,
        componente,
        caminho,
        campos,
    }
}

/// As listas, por código.
pub const LISTAS: &[ListaDesc] = &[
    l(c::TIMERS, "ph2d::ecs::Timers", WHOLE, TIMER),
    l(c::TWEENS, "ph2d::ecs::Tweens", WHOLE, TWEEN),
    l(c::ESTADOS, "ph2d::ecs::StateMachine", "states", ESTADO),
    l(c::SETAS, "ph2d::ecs::StateMachine", "transitions", SETA),
    l(c::VIGIA, "ph2d::ecs::CounterWatch", WHOLE, VIGIA),
    l(c::ACOES, "ph2d::ecs::SignalActions", WHOLE, ACAO),
    l(c::GATILHOS, "ph2d::ecs::SignalOnAction", WHOLE, GATILHO),
    l(c::ABANAO, "ph2d::ecs::ShakeEmitter", WHOLE, FONTE_DE_ABANAO),
    l(
        c::RESISTENCIAS,
        "ph2d::physics::Health",
        "resistances",
        RESISTENCIA,
    ),
    l(c::ANCORAS, "ph2d::ecs::NamedAnchorList", WHOLE, ANCORA),
];

/// ⛔ As listas do catálogo SEM canal (o censo `toda_lista_do_catalogo_tem_codigo`). O porquê:
/// - `SpriteAnimations` — as tags são intervalos de quadros da folha, o parâmetro da própria
///   animação (plano 35 §8), não um valor dela.
pub const SEM_CANAL: &[&str] = &["ph2d::ecs::SpriteAnimations"];

/// As listas que o catálogo esconde do script por terem secção própria (`Hidden::OwnSection`).
pub const DE_SECCAO: &[(&str, &str)] = &[("ph2d::physics::Health", "resistances")];
