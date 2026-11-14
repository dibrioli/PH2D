//! ⭐ **O editor da curva Custom do cartão da chave, do lado da timeline** (dono 13/11): publica a
//! Bézier normalizada da chave para o editor desenhar e transforma o arrasto das alças em
//! `SetInterp(Bezier)` — um passo de undo por arrasto (`BeginEdit` … `EndEdit`).
//!
//! O alcance segue o do menu ([`crate::presets::single_key`] / [`crate::presets::column`]): uma
//! chave dentro da seleção afina a seleção inteira; uma coluna do Summary passa a ser a seleção.

use ph2d_anim::Interp;
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

/// A Bézier de saída da chave `i` de `t` no quadrado unitário. A de tangentes absolutas
/// (`BezierW`) normaliza-se pela variação até à chave seguinte; parada ou sem seguinte não tem
/// forma normalizada (`None`).
#[must_use]
pub(crate) fn normalizada(t: &TrackView, i: usize) -> Option<[f32; 4]> {
    let k = &t.keys[i];
    match k.interp {
        Interp::Bezier { x1, y1, x2, y2 } => Some([x1 as f32, y1 as f32, x2 as f32, y2 as f32]),
        Interp::BezierW { x1, dy1, x2, dy2 } => {
            let dv = f64::from(t.keys.get(i + 1)?.value - k.value);
            (dv.abs() > f64::EPSILON).then(|| {
                [
                    x1 as f32,
                    (dy1 / dv) as f32,
                    x2 as f32,
                    (1.0 + dy2 / dv) as f32,
                ]
            })
        }
        _ => None,
    }
}

/// Publica a Bézier da chave do cartão aberto e aplica o lote do arrasto das alças.
pub(crate) fn publica_e_drena(store: &mut WidgetStore, snap: &TimelineViewSnapshot) {
    let aberto = store.context_menu().and_then(|r| match r.kind {
        ph2d_editor_core::interaction::ContextMenuKind::TimelineSegment { scope } => Some(scope),
        _ => None,
    });
    let publicada = aberto
        .and_then(|scope| chaves(scope, snap).first().copied())
        .and_then(|(t, i)| normalizada(t, i));
    store.set_key_card_bezier(publicada);
    if let Some(edit) = store.take_key_card_bez_edit() {
        for i in intents(edit, snap) {
            crate::state::push_intent(i);
        }
    }
}

/// O lote do arrasto em intents: abre o passo de undo, põe a Bézier no alcance, fecha-o.
#[must_use]
pub(crate) fn intents(
    edit: TimelineBezierEdit,
    snap: &TimelineViewSnapshot,
) -> Vec<TimelineIntent> {
    let [x1, y1, x2, y2] = edit.handles.map(f64::from);
    let interp = Interp::bezier(x1, y1, x2, y2);
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
