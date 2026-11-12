//! ⭐ **O container aberto no lugar** (doc 17 item 10, maquete C2) — a metade do painel.
//!
//! A peça aberta é DERIVADA da escolhida (item 1): no Arrange dos cartões, a strip escolhida que
//! toca um container abre-o logo abaixo da lane dela. Um estado só; Esc, vazio ou outra peça fecham.
//! O modelo (`ph2d_timeline::NoLugarView`) já traz as lanes de dentro no tempo da cena; aqui só se
//! decide quando abrir, onde as linhas moram e como uma edição de dentro sai embrulhada
//! (`TimelineIntent::NoLugar`).
//!
//! **As lanes de dentro no hit:** `TimelineHitKind::Strip { lane }` com `lane = lanes.len() + k`
//! (k = a lane de dentro) — fora do alcance das de fora, sem variante nova no `editor-core`.

use std::cell::Cell;

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_timeline::{EnterStep, StripId, TimelineIntent, TimelineViewSnapshot};

use crate::ids;
use crate::state::{self, TimelinePanelState};
use crate::tab::Tab;

thread_local! {
    /// A instância aberta no lugar, publicada a cada pintura (a shell carimba-a com o `edit_path`).
    static NO_LUGAR: Cell<Option<StripId>> = const { Cell::new(None) };
}

/// **A porta da shell:** o caminho e a instância aberta, carimbados na MESMA linha.
#[must_use]
pub fn caminho() -> (Vec<EnterStep>, Option<StripId>) {
    (state::edit_path(), NO_LUGAR.with(Cell::get))
}

/// Publica a peça aberta: a escolhida, no Arrange dos cartões, se tocar um container.
pub(crate) fn publica(st: &TimelinePanelState, snap: &TimelineViewSnapshot) {
    let aberta = (ph2d_editor_core::panel::cards::ferramenta::cards_on()
        && st.tab == Tab::Arrange
        && crate::tab::rows(st.tab, snap) == crate::tab::Rows::Lanes)
        .then_some(st.strip_escolhida)
        .flatten()
        .filter(|id| {
            snap.lanes
                .iter()
                .flat_map(|l| &l.strips)
                .any(|s| s.id == *id && s.container.is_some())
        });
    NO_LUGAR.with(|c| c.set(aberta));
}

/// O painel escondido não deixa nada aberto.
pub(crate) fn larga() {
    NO_LUGAR.with(|c| c.set(None));
}

/// A vista aberta, se este quadro a pinta (cartões, aba de lanes, lane da instância existe).
pub(crate) fn vista(snap: &TimelineViewSnapshot, tab: Tab) -> Option<&ph2d_timeline::NoLugarView> {
    let v = snap.no_lugar.as_ref()?;
    (ph2d_editor_core::panel::cards::ferramenta::cards_on()
        && crate::tab::rows(tab, snap) == crate::tab::Rows::Lanes
        && v.lane < snap.lanes.len())
    .then_some(v)
}

/// A lane de dentro que o índice de hit `lane` codifica, se for uma.
pub(crate) fn interior(snap: &TimelineViewSnapshot, lane: usize) -> Option<usize> {
    let v = snap.no_lugar.as_ref()?;
    lane.checked_sub(snap.lanes.len())
        .filter(|k| *k < v.lanes.len())
}

/// A strip ABERTA da lane de dentro `k` (as irmãs não se editam).
pub(crate) fn strip_de_dentro(
    snap: &TimelineViewSnapshot,
    k: usize,
    id: StripId,
) -> Option<&ph2d_timeline::StripView> {
    snap.no_lugar
        .as_ref()?
        .lanes
        .get(k)?
        .strips
        .iter()
        .find(|s| s.aberta && s.view.id == id)
        .map(|s| &s.view)
}

/// Quantas lanes de dentro (para o grampo do arrasto entre lanes).
pub(crate) fn n_lanes(snap: &TimelineViewSnapshot) -> usize {
    snap.no_lugar.as_ref().map_or(0, |v| v.lanes.len())
}

/// **Embrulha** uma intenção de strip cujo `lane` (e `to_lane`) é um índice de hit de dentro:
/// desconta o deslocamento e entrega-a ao modelo, em tempo de CENA, como `NoLugar`.
pub(crate) fn embrulha(snap: &TimelineViewSnapshot, intent: TimelineIntent) -> TimelineIntent {
    use TimelineIntent as I;
    let Some(v) = snap.no_lugar.as_ref() else {
        return intent;
    };
    let base = snap.lanes.len();
    let de = |l: usize| l.saturating_sub(base);
    let lane_de = |i: &I| match i {
        I::MoveStrip { lane, .. }
        | I::TrimStrip { lane, .. }
        | I::StretchStrip { lane, .. }
        | I::SetStripEase { lane, .. }
        | I::SetStripLead { lane, .. } => Some(*lane),
        _ => None,
    };
    if lane_de(&intent).is_none_or(|l| l < base) {
        return intent;
    }
    let dentro = match intent {
        I::MoveStrip {
            lane,
            to_lane,
            id,
            t_start,
        } => I::MoveStrip {
            lane: de(lane),
            to_lane: de(to_lane),
            id,
            t_start,
        },
        I::TrimStrip {
            lane,
            id,
            edge,
            t,
            from,
        } => I::TrimStrip {
            lane: de(lane),
            id,
            edge,
            t,
            from,
        },
        I::StretchStrip {
            lane,
            id,
            edge,
            t,
            from,
        } => I::StretchStrip {
            lane: de(lane),
            id,
            edge,
            t,
            from,
        },
        I::SetStripEase {
            lane,
            id,
            edge,
            seconds,
        } => I::SetStripEase {
            lane: de(lane),
            id,
            edge,
            seconds,
        },
        I::SetStripLead {
            lane,
            id,
            edge,
            seconds,
        } => I::SetStripLead {
            lane: de(lane),
            id,
            edge,
            seconds,
        },
        other => other,
    };
    I::NoLugar {
        instancia: v.instancia,
        intent: Box::new(dentro),
    }
}

/// O loop que a régua mostra e arrasta: o do container, na cena, enquanto aberto.
pub(crate) fn loop_range(snap: &TimelineViewSnapshot) -> Option<(f64, f64)> {
    match snap.no_lugar.as_ref() {
        Some(v) if ph2d_editor_core::panel::cards::ferramenta::cards_on() => v.loop_cena,
        _ => snap.loop_range,
    }
}

/// O arrasto do loop aberto: `SetContainerLoop` embrulhado (o modelo converte para o local).
pub(crate) fn loop_intent(
    snap: &TimelineViewSnapshot,
    range: (f64, f64),
) -> Option<TimelineIntent> {
    let v = snap.no_lugar.as_ref()?;
    ph2d_editor_core::panel::cards::ferramenta::cards_on().then(|| TimelineIntent::NoLugar {
        instancia: v.instancia,
        intent: Box::new(TimelineIntent::SetContainerLoop {
            container: v.container,
            range: Some(range),
            ping_pong: snap.loop_ping_pong,
        }),
    })
}

/// O olho e o peso das lanes de dentro → `SetLaneMuted`/`SetLaneWeight` embrulhados.
pub(crate) fn evento(
    st: &mut TimelinePanelState,
    ev: WidgetEvent,
    host: &mut dyn PanelHostInternal,
) -> Option<EventOutcome> {
    let snap = state::current_snapshot();
    let v = snap.no_lugar.as_ref()?;
    let embrulha = |intent| TimelineIntent::NoLugar {
        instancia: v.instancia,
        intent: Box::new(intent),
    };
    match ev {
        WidgetEvent::Click(id) => {
            let k = ids::TIMELINE_NO_LUGAR_MUTE.iter().position(|&b| b == id)?;
            if let Some(l) = v.lanes.get(k) {
                state::push_intent(embrulha(TimelineIntent::SetLaneMuted {
                    lane: k,
                    muted: !l.muted,
                }));
            }
            Some(EventOutcome::Consumed)
        }
        WidgetEvent::Focus(id) => {
            ids::TIMELINE_NO_LUGAR_WEIGHT
                .iter()
                .position(|&w| w == id)?;
            if st.weight_edit.is_none() {
                st.weight_edit = Some(usize::MAX);
                state::push_intent(TimelineIntent::BeginEdit);
            }
            Some(EventOutcome::Consumed)
        }
        WidgetEvent::Blur(id) | WidgetEvent::Submit(id)
            if ids::TIMELINE_NO_LUGAR_WEIGHT.contains(&id) =>
        {
            if st.weight_edit.take().is_some() {
                state::push_intent(TimelineIntent::EndEdit);
            }
            Some(EventOutcome::Consumed)
        }
        WidgetEvent::ValueChanged(id) => {
            let k = ids::TIMELINE_NO_LUGAR_WEIGHT
                .iter()
                .position(|&w| w == id)?;
            if k < v.lanes.len() {
                let weight = host.store().number_value(id).unwrap_or(1.0);
                state::push_intent(embrulha(TimelineIntent::SetLaneWeight { lane: k, weight }));
            }
            Some(EventOutcome::Consumed)
        }
        _ => None,
    }
}
