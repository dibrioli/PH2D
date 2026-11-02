//! A costura da P7b no painel PIXEL: o registo e o pedido do pincel de padrão, dos pincéis
//! guardados e da Dynamics, e o arrasto das barras de duas pegas (filho de [`crate::seam`]).

use crate::ids::{self, DynRow};
use crate::state::with_current;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::pixel_panel::{
    PIXEL_STABILIZER_ON, PIXEL_THRESHOLD_FULL, PixelBrushAction, PixelChannel, PixelDynamics,
    PixelOptions, PixelPanelInfo, PixelRequest, PixelSensor, PixelThreshold,
};
use ph2d_editor_core::widget::{
    ButtonState, CheckboxState, CheckboxValue, TextInputState, range_drag, range_handle_at,
};
use ph2d_i18n::tr;

pub(crate) fn populate(store: &mut WidgetStore) {
    let botoes = ids::PATTERN_ALIGNS
        .iter()
        .map(|(_, id, _)| *id)
        .chain(ids::TURNS.iter().map(|(_, id)| *id))
        .chain(ids::DITHERS.iter().map(|(_, id)| *id))
        .chain(ids::sensor_ids().map(|(.., id)| id))
        .chain([ids::SAVE_HERE, ids::DYNAMICS]);
    for id in botoes {
        let state = ButtonState::Normal;
        store.register(id, InteractiveState::Button { state });
    }
    for (_, id, k) in ids::PATTERN_ALIGNS {
        store.set_tooltip(id, tr(k));
    }
    store.set_tooltip(ids::DYNAMICS, tr("panel.pixel.dynamics.tip"));
    store.set_tooltip(ids::SAVE_HERE, tr("panel.pixel.brush.save_here_tip"));
    let state = CheckboxState::Normal;
    let value = CheckboxValue::Unchecked;
    store.register(
        ids::DYN_STABILIZER,
        InteractiveState::Checkbox { state, value },
    );
    let omissao = PixelOptions::default();
    for (id, (min, max, step)) in ids::DYN_NUMBERS {
        let value = numero(&omissao, id).unwrap_or(min);
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
        store.set_number_range(id, min, max, step);
    }
    for (_, id, _) in ids::THRESHOLDS {
        store.register(id, InteractiveState::Plain);
    }
}

/// O registo POR QUADRO dos espaços guardados: recuperar, o cadeado (a dica diz o estado) e apagar.
pub(crate) fn register_brushes(store: &mut WidgetStore, trancados: &[bool]) {
    for (n, &locked) in trancados.iter().enumerate() {
        for id in [
            ids::brush_slot_id(n),
            ids::brush_lock_id(n),
            ids::brush_delete_id(n),
        ] {
            let state = ButtonState::Normal;
            store.register_if_absent(id, InteractiveState::Button { state });
        }
        store.set_tooltip(ids::brush_slot_id(n), tr("panel.pixel.brush.slot_tip"));
        let k = if locked {
            "panel.pixel.brush.locked_tip"
        } else {
            "panel.pixel.brush.unlocked_tip"
        };
        store.set_tooltip(ids::brush_lock_id(n), tr(k));
        store.set_tooltip(ids::brush_delete_id(n), tr("panel.pixel.brush.delete_tip"));
    }
}

/// A linha `row` da grelha.
pub(crate) fn canal(d: &PixelDynamics, row: DynRow) -> PixelChannel {
    match row {
        DynRow::Size => d.size,
        DynRow::Angle => d.angle,
        DynRow::Gradient => d.gradient,
    }
}

fn canal_mut(d: &mut PixelDynamics, row: DynRow) -> &mut PixelChannel {
    match row {
        DynRow::Size => &mut d.size,
        DynRow::Angle => &mut d.angle,
        DynRow::Gradient => &mut d.gradient,
    }
}

/// O valor do campo `id` da Dynamics.
pub(crate) fn numero(o: &PixelOptions, id: NodeId) -> Option<f64> {
    let d = &o.dynamics;
    Some(match id {
        ids::DYN_STABILIZER_LEN => f64::from(d.stabilizer.max(1)),
        ids::DYN_MIN_SIZE => f64::from(d.size.min),
        ids::DYN_MIN_ANGLE => f64::from(d.angle.min),
        _ => return None,
    })
}

/// Os campos da Dynamics como o modelo os diz (a pintura semeia-os).
pub(crate) fn numeros(o: &PixelOptions) -> impl Iterator<Item = (NodeId, f64)> + '_ {
    ids::DYN_NUMBERS
        .iter()
        .filter_map(|(id, _)| Some((*id, numero(o, *id)?)))
}

/// As opções com o campo `id` da Dynamics em `v` (preso à faixa, inteiro).
fn com_numero(mut o: PixelOptions, id: NodeId, v: f64) -> Option<PixelOptions> {
    let (_, (min, max, _)) = ids::DYN_NUMBERS.iter().find(|(n, _)| *n == id)?;
    if !v.is_finite() {
        return None;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "presa à faixa do campo antes da conversão"
    )]
    let i = ph2d_editor_core::math::safe_clamp_f64(v.round(), *min, *max) as i32;
    let d = &mut o.dynamics;
    match id {
        ids::DYN_STABILIZER_LEN => d.stabilizer = u32::try_from(i).unwrap_or(1),
        ids::DYN_MIN_SIZE => d.size.min = i,
        _ => d.angle.min = i,
    }
    Some(o)
}

/// O pedido de um clique da P7b, se o id é um destes (antes dos segmentos das opções: uma forma
/// clicada larga o padrão, o que as opções sozinhas não dizem).
pub(crate) fn pedido(info: &PixelPanelInfo, id: NodeId) -> Option<PixelRequest> {
    let brush = |a| Some(PixelRequest::Brush(a));
    if let Some((s, _)) = ids::SHAPES.iter().find(|(_, b)| *b == id) {
        return brush(PixelBrushAction::Shape(*s));
    }
    if let Some((t, _)) = ids::TURNS.iter().find(|(_, b)| *b == id) {
        return info.pattern.and_then(|_| brush(PixelBrushAction::Turn(*t)));
    }
    if id == ids::SAVE_HERE {
        return brush(PixelBrushAction::SaveHere);
    }
    for n in 0..info.brushes.len() {
        if id == ids::brush_slot_id(n) {
            return brush(PixelBrushAction::Recall(n));
        }
        if id == ids::brush_lock_id(n) {
            return brush(PixelBrushAction::ToggleLock(n));
        }
        if id == ids::brush_delete_id(n) {
            return brush(PixelBrushAction::Delete(n));
        }
    }
    let mut o = info.options;
    if let Some((a, ..)) = ids::PATTERN_ALIGNS.iter().find(|(_, b, _)| *b == id) {
        o.pattern_align = *a;
    } else if let Some((m, _)) = ids::DITHERS.iter().find(|(_, b)| *b == id) {
        o.dynamics.dither = *m;
    } else if let Some((row, s, _)) = ids::sensor_ids().find(|(.., b)| *b == id) {
        canal_mut(&mut o.dynamics, row).sensor = s;
    } else {
        return None;
    }
    Some(PixelRequest::Options(Box::new(o)))
}

/// A caixa do Stabilizer: ligar põe a corda do Trial ([`PIXEL_STABILIZER_ON`]), desligar põe 0.
pub(crate) fn alterna(info: &PixelPanelInfo, id: NodeId) -> Option<PixelRequest> {
    if id != ids::DYN_STABILIZER {
        return None;
    }
    let mut o = info.options;
    let d = &mut o.dynamics;
    d.stabilizer = if d.stabilizer > 0 {
        0
    } else {
        PIXEL_STABILIZER_ON
    };
    Some(PixelRequest::Options(Box::new(o)))
}

/// Um campo da Dynamics mudou.
pub(crate) fn campo(info: &PixelPanelInfo, id: NodeId, v: Option<f64>) -> Option<PixelRequest> {
    let o = com_numero(info.options, id, v?)?;
    info.editing.then_some(PixelRequest::Options(Box::new(o)))
}

/// **O arrasto das barras de duas pegas**: no premir agarra a pega mais perto; a cada quadro em
/// que o ponteiro se moveu, ela segue-o (a mesma conta que se pintou).
pub(crate) fn arrasta_limiar(
    st: &mut crate::PixelToolsPanelState,
    host: &mut dyn PanelHostInternal,
) {
    let activo = host.store().active_id();
    let Some(k) = activo.and_then(|a| ids::THRESHOLDS.iter().position(|(_, id, _)| *id == a))
    else {
        st.pincel.pega = None;
        return;
    };
    let (Some(trilha), Some([x, _])) = (st.pincel.trilhas[k], host.store().last_pointer()) else {
        return;
    };
    let pedido = with_current(|info| {
        if !info.editing {
            return None;
        }
        let mut o = info.options;
        let t = if ids::THRESHOLDS[k].0 == PixelSensor::Pressure {
            &mut o.dynamics.pressure
        } else {
            &mut o.dynamics.velocity
        };
        let full = f32::from(PIXEL_THRESHOLD_FULL);
        let (lo, hi) = (f32::from(t.lo) / full, f32::from(t.hi) / full);
        let pega = match st.pincel.pega {
            Some((j, p)) if j == k => p,
            _ => range_handle_at(trilha, x, lo, hi),
        };
        st.pincel.pega = Some((k, pega));
        let (lo, hi) = range_drag(trilha, x, pega, lo, hi);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "fracção 0..=1 da barra vezes o fim dela"
        )]
        let mil = |v: f32| (v * full).round() as u16;
        let novo = PixelThreshold {
            lo: mil(lo),
            hi: mil(hi),
        };
        if novo == *t {
            return None;
        }
        *t = novo;
        Some(PixelRequest::Options(Box::new(o)))
    });
    if let Some(r) = pedido {
        host.bus_mut().push(EditorAction::Pixel(r));
    }
}
