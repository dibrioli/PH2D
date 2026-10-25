//! A forma GRAVADA dos componentes com itens até ao `PROJECT_SCHEMA` 194 (sem `item`) — só para o
//! degrau `194 → 195` ([`crate::item_id::migra_v194`]). ⛔ Congelada: não acompanhe o tipo vivo.

use serde::Deserialize;

use crate::ItemId;
use crate::counter_watch::{Compare, CounterScope, CounterWatchRow};
use crate::named_anchor::{AnchorData, NamedAnchor};
use crate::signal_actions::{SignalAction, SignalFrom, SignalTarget, SignalVerb};
use crate::signal_on_action::{ActionEdge, ActionTriggerRow};
use crate::state_machine::{MachineState, Transition};
use crate::{
    CounterWatch, NamedAnchorList, ShakeEmitter, ShakeSource, SignalActions, SignalOnAction,
    StateMachine, Timer, Timers, Transform, Tweens,
};

const N: ItemId = ItemId::NONE;

/// Uma lista v194 de `T` que vira a lista viva.
macro_rules! lista {
    ($v:ident, $vivo:ident, $item:ident) => {
        #[derive(Deserialize)]
        pub(crate) struct $v(Vec<$item>);
        impl From<$v> for $vivo {
            fn from(v: $v) -> Self {
                Self(v.0.into_iter().map(Into::into).collect())
            }
        }
    };
}

#[derive(Deserialize)]
pub(crate) struct TimerV194 {
    name: String,
    duration_us: u64,
    repeat: bool,
    autostart: bool,
    signal: String,
}
impl From<TimerV194> for Timer {
    fn from(t: TimerV194) -> Self {
        Self {
            name: t.name,
            duration_us: t.duration_us,
            repeat: t.repeat,
            autostart: t.autostart,
            signal: t.signal,
            item: N,
        }
    }
}
lista!(TimersV194, Timers, TimerV194);

#[derive(Deserialize)]
pub(crate) struct TweenV194 {
    canal: ph2d_tween::Canal,
    de: [f32; 4],
    para: [f32; 4],
    easing: ph2d_tween::Easing,
    ao_acabar: ph2d_tween::AoAcabar,
    ciclo: ph2d_tween::Ciclo,
}
impl From<TweenV194> for ph2d_tween::Tween {
    fn from(t: TweenV194) -> Self {
        Self {
            canal: t.canal,
            de: t.de,
            para: t.para,
            easing: t.easing,
            ao_acabar: t.ao_acabar,
            ciclo: t.ciclo,
            item: N,
        }
    }
}
lista!(TweensV194, Tweens, TweenV194);

#[derive(Deserialize)]
pub(crate) struct MachineStateV194 {
    name: String,
    on_enter: String,
    on_exit: String,
}
impl From<MachineStateV194> for MachineState {
    fn from(s: MachineStateV194) -> Self {
        Self {
            name: s.name,
            on_enter: s.on_enter,
            on_exit: s.on_exit,
            item: N,
        }
    }
}
#[derive(Deserialize)]
pub(crate) struct TransitionV194 {
    from: u8,
    on: String,
    to: u8,
}
impl From<TransitionV194> for Transition {
    fn from(t: TransitionV194) -> Self {
        Self {
            from: t.from,
            on: t.on,
            to: t.to,
            item: N,
        }
    }
}
#[derive(Deserialize)]
pub(crate) struct StateMachineV194 {
    states: Vec<MachineStateV194>,
    transitions: Vec<TransitionV194>,
    initial: u8,
}
impl From<StateMachineV194> for StateMachine {
    fn from(m: StateMachineV194) -> Self {
        Self {
            states: m.states.into_iter().map(Into::into).collect(),
            transitions: m.transitions.into_iter().map(Into::into).collect(),
            initial: m.initial,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct CounterWatchRowV194 {
    counter: String,
    compare: Compare,
    value: i64,
    signal: String,
    once: bool,
    scope: CounterScope,
}
impl From<CounterWatchRowV194> for CounterWatchRow {
    fn from(r: CounterWatchRowV194) -> Self {
        Self {
            counter: r.counter,
            compare: r.compare,
            value: r.value,
            signal: r.signal,
            once: r.once,
            scope: r.scope,
            item: N,
        }
    }
}
lista!(CounterWatchV194, CounterWatch, CounterWatchRowV194);

#[derive(Deserialize)]
pub(crate) struct SignalActionV194 {
    on: String,
    target: String,
    verb: SignalVerb,
    arg: String,
    target_by: SignalTarget,
    from: SignalFrom,
}
impl From<SignalActionV194> for SignalAction {
    fn from(a: SignalActionV194) -> Self {
        Self {
            on: a.on,
            target: a.target,
            verb: a.verb,
            arg: a.arg,
            target_by: a.target_by,
            from: a.from,
            item: N,
        }
    }
}
lista!(SignalActionsV194, SignalActions, SignalActionV194);

#[derive(Deserialize)]
pub(crate) struct ActionTriggerRowV194 {
    action: String,
    edge: ActionEdge,
    signal: String,
}
impl From<ActionTriggerRowV194> for ActionTriggerRow {
    fn from(r: ActionTriggerRowV194) -> Self {
        Self {
            action: r.action,
            edge: r.edge,
            signal: r.signal,
            item: N,
        }
    }
}
lista!(SignalOnActionV194, SignalOnAction, ActionTriggerRowV194);

#[derive(Deserialize)]
pub(crate) struct ShakeSourceV194 {
    on: String,
    de: SignalFrom,
    forca: f32,
    dentro: f32,
    fora: f32,
}
impl From<ShakeSourceV194> for ShakeSource {
    fn from(s: ShakeSourceV194) -> Self {
        Self {
            on: s.on,
            de: s.de,
            forca: s.forca,
            dentro: s.dentro,
            fora: s.fora,
            item: N,
        }
    }
}
lista!(ShakeEmitterV194, ShakeEmitter, ShakeSourceV194);

#[derive(Deserialize)]
pub(crate) struct NamedAnchorV194 {
    name: String,
    transform: Transform,
    bounds: Option<[f32; 4]>,
    center: Option<[f32; 4]>,
    user_data: AnchorData,
}
impl From<NamedAnchorV194> for NamedAnchor {
    fn from(a: NamedAnchorV194) -> Self {
        Self {
            name: a.name,
            transform: a.transform,
            bounds: a.bounds,
            center: a.center,
            user_data: a.user_data,
            item: N,
        }
    }
}
#[derive(Deserialize)]
pub(crate) struct NamedAnchorListV194(Vec<NamedAnchorV194>);
impl From<NamedAnchorListV194> for NamedAnchorList {
    fn from(v: NamedAnchorListV194) -> Self {
        Self(v.0.into_iter().map(Into::into).collect())
    }
}

crate::com_itens!(Timers: 0);
crate::com_itens!(Tweens: 0);
crate::com_itens!(StateMachine: states, transitions);
crate::com_itens!(CounterWatch: 0);
crate::com_itens!(SignalActions: 0);
crate::com_itens!(SignalOnAction: 0);
crate::com_itens!(ShakeEmitter: 0);
crate::com_itens!(NamedAnchorList: 0);
