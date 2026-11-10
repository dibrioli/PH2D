//! **P13: as rampas sob a grelha da paleta** — os ids, o registo e o pedido de cada gesto: *Make
//! Ramp*, *New Ramp…* (o diálogo: Steps, Hue shift, Chroma, Light range; a base é a cor primária),
//! *Remove Ramp* e o *Cycle* da rampa onde está a amostra clicada (Period, Direction).

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::pixel_panel::{
    NewSpriteMode, PixelCycle, PixelCycleDirection, PixelPanelInfo, PixelRampAction,
    PixelRampConfig, PixelRampInfo, PixelRequest,
};
use ph2d_editor_core::widget::{ButtonState, CheckboxState, CheckboxValue, TextInputState};
use ph2d_i18n::tr;
use ph2d_tool_registry::hash_node_id;

use crate::state::with_current;

pub const NEW: NodeId = hash_node_id("pixel.ramp.new");
pub const MAKE: NodeId = hash_node_id("pixel.ramp.make");
pub const REMOVE: NodeId = hash_node_id("pixel.ramp.remove");
pub const CREATE: NodeId = hash_node_id("pixel.ramp.create");
pub const CYCLE: NodeId = hash_node_id("pixel.ramp.cycle");
pub const PERIOD: NodeId = hash_node_id("pixel.ramp.cycle.period");
/// A amostra de cada degrau da faixa (só pinta; o clique é o da grelha).
pub const STRIP: NodeId = hash_node_id("pixel.ramp.strip");

/// Os campos do New Ramp…: `(id, chave, mín, máx)`. Steps até 255: o campo é um `u8` e a paleta
/// leva 256 cores (o gerador recusa o que não cabe).
pub const FIELDS: [(NodeId, &str, f64, f64); 5] = [
    (
        hash_node_id("pixel.ramp.steps"),
        "pixel.ramp.steps",
        1.0,
        255.0, // LITERAL-PX-OK: degraus de uma rampa (contagem de índices de paleta, não píxel)
    ),
    (
        hash_node_id("pixel.ramp.hue_shift"),
        "pixel.ramp.hue_shift",
        -180.0, // LITERAL-PX-OK: graus de matiz
        180.0,  // LITERAL-PX-OK: graus de matiz
    ),
    (
        hash_node_id("pixel.ramp.chroma"),
        "pixel.ramp.chroma",
        0.0,
        100.0, // LITERAL-PX-OK: percentagem de croma
    ),
    (
        hash_node_id("pixel.ramp.light_min"),
        "pixel.ramp.light_min",
        0.0,
        100.0, // LITERAL-PX-OK: percentagem de luz
    ),
    (
        hash_node_id("pixel.ramp.light_max"),
        "pixel.ramp.light_max",
        0.0,
        100.0, // LITERAL-PX-OK: percentagem de luz
    ),
];

/// As direcções do ciclo, na ordem de [`PixelCycleDirection::ALL`].
pub const DIRECTIONS: [(PixelCycleDirection, NodeId, &str); 3] = [
    (
        PixelCycleDirection::Forward,
        hash_node_id("pixel.ramp.cycle.forward"),
        "pixel.cycle.direction.forward",
    ),
    (
        PixelCycleDirection::Reverse,
        hash_node_id("pixel.ramp.cycle.reverse"),
        "pixel.cycle.direction.reverse",
    ),
    (
        PixelCycleDirection::PingPong,
        hash_node_id("pixel.ramp.cycle.ping_pong"),
        "pixel.cycle.direction.ping_pong",
    ),
];

fn valores_de_fabrica() -> [f64; 5] {
    let c = PixelRampConfig::default();
    [
        f64::from(c.steps),
        f64::from(c.hue_shift_deg),
        f64::from(c.chroma_pct),
        f64::from(c.light_pct[0]),
        f64::from(c.light_pct[1]),
    ]
}

fn numero(store: &mut WidgetStore, id: NodeId, value: f64, (min, max): (f64, f64)) {
    store.register(
        id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value,
            buffer: value.to_string(),
            caret: 0,
            last_committed: value,
            selection_anchor: None,
        },
    );
    store.set_number_range(id, min, max, 1.0);
}

pub(crate) fn populate(store: &mut WidgetStore) {
    let botoes = [NEW, MAKE, REMOVE, CREATE].into_iter();
    for id in botoes.chain(DIRECTIONS.map(|(_, id, _)| id)) {
        let state = ButtonState::Normal;
        store.register(id, InteractiveState::Button { state });
    }
    let (state, value) = (CheckboxState::Normal, CheckboxValue::Unchecked);
    store.register(CYCLE, InteractiveState::Checkbox { state, value });
    store.register(STRIP, InteractiveState::Plain);
    for ((id, _, min, max), v) in FIELDS.into_iter().zip(valores_de_fabrica()) {
        numero(store, id, v, (min, max));
    }
    let omissao = PixelCycle::default_for(1);
    numero(
        store,
        PERIOD,
        f64::from(omissao.period_ms),
        (1.0, f64::from(u32::MAX)),
    );
}

/// A rampa onde está a última amostra clicada (a que os controlos do Cycle e o Remove mexem).
#[must_use]
pub fn atual(info: &PixelPanelInfo) -> Option<&PixelRampInfo> {
    let a = u8::try_from(info.anchor?).ok()?;
    info.ramps.iter().find(|r| r.indices.contains(&a))
}

/// Por quadro: o Period mostra o do MODELO, e o Cycle desliga-se fora de Indexed (com a dica).
pub(crate) fn quadro(store: &mut WidgetStore, info: &PixelPanelInfo) {
    if let Some(c) = atual(info).and_then(|r| r.cycle) {
        let v = f64::from(c.period_ms);
        if store.number_value(PERIOD) != Some(v) {
            store.set_number_value(PERIOD, v);
        }
    }
    let indexed = info.mode == NewSpriteMode::Indexed;
    if let Some(InteractiveState::Checkbox { state, .. }) = store.get_mut(CYCLE) {
        if !indexed {
            *state = CheckboxState::Disabled;
        } else if *state == CheckboxState::Disabled {
            *state = CheckboxState::Normal;
        }
    }
    let dica = if indexed {
        tr("pixel.cycle.tip")
    } else {
        tr("pixel.cycle.rgb_off.tip")
    };
    store.set_tooltip(CYCLE, dica);
}

fn config(store: &WidgetStore) -> PixelRampConfig {
    let [steps, hue, chroma, l0, l1] = FIELDS.map(|(id, _, min, max)| {
        store
            .number_value(id)
            .filter(|v| v.is_finite())
            .map_or(min, |v| v.round().clamp(min, max)) // CLAMP-OK: limites da tabela, v finito
    });
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "presos às faixas da tabela antes da conversão"
    )]
    PixelRampConfig {
        steps: steps as u8,
        hue_shift_deg: hue as i16,
        chroma_pct: chroma as u8,
        light_pct: [l0 as u8, l1 as u8],
    }
}

fn ciclo(r: &PixelRampInfo, cycle: Option<PixelCycle>) -> PixelRequest {
    PixelRequest::Ramp(PixelRampAction::Cycle { ramp: r.id, cycle })
}

/// O pedido de um gesto das rampas; `None` = não é delas.
pub(crate) fn evento(
    st: &mut crate::PixelToolsPanelState,
    host: &mut dyn PanelHostInternal,
    ev: &WidgetEvent,
) -> Option<EventOutcome> {
    let pedido = match *ev {
        WidgetEvent::Click(id) if id == NEW => {
            st.paleta.nova_rampa = !st.paleta.nova_rampa;
            return Some(EventOutcome::Consumed);
        }
        WidgetEvent::Click(id) if id == CREATE => {
            st.paleta.nova_rampa = false;
            let c = config(host.store());
            Some(PixelRequest::Ramp(PixelRampAction::New(c)))
        }
        WidgetEvent::Click(id) if id == MAKE => with_current(|i| {
            let ix: Vec<u8> = i
                .selected
                .iter()
                .filter_map(|&k| u8::try_from(k).ok())
                .collect();
            (!ix.is_empty()).then_some(PixelRequest::Ramp(PixelRampAction::Make(ix)))
        }),
        WidgetEvent::Click(id) if id == REMOVE => {
            with_current(|i| atual(i).map(|r| PixelRequest::Ramp(PixelRampAction::Remove(r.id))))
        }
        WidgetEvent::Toggled(id) if id == CYCLE => with_current(|i| {
            let r = atual(i).filter(|_| i.mode == NewSpriteMode::Indexed)?;
            let novo = r
                .cycle
                .is_none()
                .then(|| PixelCycle::default_for(r.indices.len()));
            Some(ciclo(r, novo))
        }),
        WidgetEvent::ValueChanged(id) if id == PERIOD => {
            let v = host.store().number_value(PERIOD).filter(|v| v.is_finite());
            with_current(|i| {
                let r = atual(i)?;
                let mut c = r.cycle?;
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "preso a 1..=u32::MAX antes da conversão"
                )]
                let ms = v?.round().clamp(1.0, f64::from(u32::MAX)) as u32; // CLAMP-OK: limites constantes, v finito
                c.period_ms = ms;
                Some(ciclo(r, Some(c)))
            })
        }
        WidgetEvent::Click(id) if DIRECTIONS.iter().any(|(_, d, _)| *d == id) => {
            with_current(|i| {
                let r = atual(i)?;
                let mut c = r.cycle?;
                c.direction = DIRECTIONS.iter().find(|(_, d, _)| *d == id)?.0;
                Some(ciclo(r, Some(c)))
            })
        }
        _ => return None,
    };
    Some(crate::seam::empurra(host, pedido))
}
