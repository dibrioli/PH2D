//! ⭐ **O editor da curva Custom do cartão da chave, do lado da timeline** (dono 13/11): publica a
//! curva normalizada da chave para o editor desenhar e transforma cada edição (arrastar, criar ou
//! apagar um ponto) em `SetInterp` (`Bezier` sem pontos, `Spline` com) — um passo de undo por
//! gesto (`BeginEdit` … `EndEdit`).
//!
//! O alcance segue o do menu ([`crate::presets::single_key`] / [`crate::presets::column`]): uma
//! chave dentro da seleção afina a seleção inteira; uma coluna do Summary passa a ser a seleção.

use ph2d_anim::{EaseSpline, Interp};
use ph2d_editor_core::WidgetStore;
use ph2d_editor_core::interaction::{TimelineBezierEdit, TimelineInterpScope as S};
use ph2d_timeline::{KeyView, SelectedKey, TimelineIntent, TimelineViewSnapshot, TrackView};

/// As chaves do alcance, cada uma com a sua faixa (para ler a chave seguinte).
fn chaves(scope: S, snap: &TimelineViewSnapshot) -> Vec<(&TrackView, usize)> {
    snap.tracks
        .iter()
        .flat_map(|t| {
            t.keys
                .iter()
                .enumerate()
                .filter(move |(_, k)| match scope {
                    S::Key { target, key } => t.target.get() == target && k.id.get() == key,
                    S::Column { t_bits } => k.t_seconds.to_bits() == t_bits,
                    S::StripFade { .. } => false,
                })
                .map(move |(i, _)| (t, i))
        })
        .collect()
}

/// A curva Custom de saída da chave `i` de `t` no quadrado unitário. A `Bezier` é a curva sem
/// pontos; a `Spline` é ela mesma; a de tangentes absolutas (`BezierW`) normaliza-se pela variação
/// até à chave seguinte; parada, sem seguinte ou outra interpolação não tem forma (`None`).
#[must_use]
pub(crate) fn normalizada(t: &TrackView, i: usize) -> Option<EaseSpline> {
    let k = &t.keys[i];
    let f = |v: f64| v as f32;
    match k.interp {
        Interp::Bezier { x1, y1, x2, y2 } => {
            Some(EaseSpline::from_bezier(f(x1), f(y1), f(x2), f(y2)))
        }
        Interp::Spline(sp) => Some(sp),
        Interp::BezierW { x1, dy1, x2, dy2 } => {
            let dv = f64::from(t.keys.get(i + 1)?.value - k.value);
            (dv.abs() > f64::EPSILON)
                .then(|| EaseSpline::from_bezier(f(x1), f(dy1 / dv), f(x2), f(1.0 + dy2 / dv)))
        }
        _ => None,
    }
}

/// A `Interp` de uma curva editada: sem pontos volta a ser a `Bezier` de duas alças.
#[must_use]
pub(crate) fn interp_da_curva(c: EaseSpline) -> Interp {
    if c.points().is_empty() {
        let ([x1, y1], [x2, y2]) = c.end_handles();
        let f = f64::from;
        Interp::bezier(f(x1), f(y1), f(x2), f(y2))
    } else {
        Interp::Spline(c)
    }
}

/// Publica a curva da chave do cartão aberto e aplica o lote da edição.
pub(crate) fn publica_e_drena(store: &mut WidgetStore, snap: &TimelineViewSnapshot) {
    let aberto = store.context_menu().and_then(|r| match r.kind {
        ph2d_editor_core::interaction::ContextMenuKind::TimelineSegment { scope } => Some(scope),
        _ => None,
    });
    let publicada = aberto
        .and_then(|scope| chaves(scope, snap).first().copied())
        .and_then(|(t, i)| normalizada(t, i));
    store.set_key_card_curve(publicada);
    if let Some(edit) = store.take_key_card_bez_edit() {
        for i in intents(edit, snap) {
            crate::state::push_intent(i);
        }
    }
}

/// O lote da edição em intents: abre o passo de undo, põe a curva no alcance, fecha-o.
#[must_use]
pub(crate) fn intents(
    edit: TimelineBezierEdit,
    snap: &TimelineViewSnapshot,
) -> Vec<TimelineIntent> {
    let interp = interp_da_curva(edit.curve);
    let alvo = chaves(edit.scope, snap);
    let mut out = Vec::new();
    if edit.begin {
        out.push(TimelineIntent::BeginEdit);
    }
    let sel = |(t, i): &(&TrackView, usize)| SelectedKey {
        target: t.target,
        key: t.keys[*i].id,
    };
    let escolhida = |(t, i): &(&TrackView, usize)| -> bool {
        let k: &KeyView = &t.keys[*i];
        k.selected
    };
    match edit.scope {
        S::Key { .. } => match alvo.first() {
            Some(c) if escolhida(c) => out.push(TimelineIntent::SetSelectedInterp { interp }),
            Some(c) => out.push(TimelineIntent::SetInterp {
                target: c.0.target,
                key: c.0.keys[c.1].id,
                interp,
            }),
            None => {}
        },
        S::Column { .. } if !alvo.is_empty() => {
            if !alvo.iter().all(escolhida) {
                out.push(TimelineIntent::ClearSelection);
                out.extend(alvo.iter().map(|c| TimelineIntent::AddToSelection(sel(c))));
            }
            out.push(TimelineIntent::SetSelectedInterp { interp });
        }
        _ => {}
    }
    if edit.end {
        out.push(TimelineIntent::EndEdit);
    }
    out
}

#[cfg(test)]
#[path = "cartao_bezier_tests.rs"]
mod tests;
