//! **A linha «Macro steps»** (ADR-0186, W5d; `docs/Macros/02_plano_W5.md` §5): na vista Keys, quando
//! o clipe activo é o de uma macro, um losango no FIM de cada etapa — o instante que o painel MACROS
//! mostra e para onde leva a cabeça (W15, smoke do dono 22/10) — (`snap.macro_etapas`, que a
//! shell enche pela `ph2d-app-macros`). Arrastar uma etapa leva-a SOZINHA para outro instante (W16, o
//! dono: «mover key não deve empurrar outros») — pode sobrepor-se a outra ou passar-lhe à frente — e, ao
//! largar, o painel pede `Instante(fim + Δ)` à família pelo barramento (`EditorAction::Macro`): o arrasto
//! é um EVENTO que a família lê, nunca uma chamada.
//! ⛔ Não é uma chave na faixa «Macro Time» (plano §2: esticaria o relógio — câmara lenta).

use crate::graph::TimeView;
use crate::state::{MacroDrag, TimelinePanelState};
use crate::{ids, key_drag, tab::Tab, tracks};
use ph2d_editor_core::interaction::{
    GesturePhase, InteractiveState, TimelineGesture, TimelineHitKind,
};
use ph2d_editor_core::macro_edits::{CampoDaEtapa, MacroRequest};
use ph2d_editor_core::paint::{paint_text_elided, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};

/// A altura da linha: uma fileira na vista Keys com etapas, nada nas outras.
pub(crate) fn macro_h(snap: &TimelineViewSnapshot, tab: Tab) -> f32 {
    if tab.shows_keys() && !snap.macro_etapas.is_empty() {
        ROW_H_PX
    } else {
        0.0
    }
}

/// O deslocamento em px da etapa `i` durante o arrasto: só a arrastada anda.
fn preview_dx(state: &TimelinePanelState, snap: &TimelineViewSnapshot, i: usize) -> f32 {
    match state.macro_drag {
        Some(d) if i == d.index => {
            let s = key_drag::drag_delta_seconds(d.start_x, d.cur_x, state.px_per_s, snap);
            #[expect(clippy::cast_possible_truncation, reason = "px de ecrã")]
            let dx = (s * state.px_per_s) as f32;
            dx
        }
        _ => 0.0,
    }
}

/// Pinta a linha em `y` (logo a seguir à última faixa) e regista os losangos.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    label_w: f32,
    view: TimeView,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
    y: f32,
    lane: Rect,
) {
    if macro_h(snap, state.tab) <= 0.0 || y >= region.y + region.h || y + ROW_H_PX <= region.y {
        return;
    }
    let font = TypeToken::Sm.px();
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        ph2d_i18n::tr("panel.timeline.macro_steps"),
        region.x + Spacing::Sm.px(),
        y + (ROW_H_PX - font) * 0.5,
        font,
        (label_w - Spacing::Sm.px() * 2.0).max(0.0),
        resolve(ColorToken::Text2, theme),
    );
    let cy = y + ROW_H_PX * 0.5;
    for (i, e) in snap.macro_etapas.iter().enumerate() {
        let x = view.x(e.fim) + preview_dx(state, snap, i);
        if x < view.time_x - tracks::KEY_HIT_HW || x > view.right + tracks::KEY_HIT_HW {
            continue;
        }
        let tok = if state.macro_drag.is_some_and(|d| d.index == i) {
            ColorToken::TimelineKeySelected
        } else {
            ColorToken::TimelineKey
        };
        tracks::paint_diamond(ctx, x, cy, tracks::DIAMOND_H, resolve(tok, theme));
        let id = ids::timeline_macro_step_id(i);
        ctx.host.store_mut().register(
            id,
            InteractiveState::TimelineSurface {
                parent: ph2d_editor_core::ids::TIMELINE_PANEL,
                kind: TimelineHitKind::MacroStep { index: i },
                canvas: lane,
            },
        );
        let hit = Rect::new(
            x - tracks::KEY_HIT_HW,
            y,
            tracks::KEY_HIT_HW * 2.0,
            ROW_H_PX,
        );
        ctx.host.hit_index_mut().register(id, hit);
    }
}

/// O gesto num losango: arrastar re-cronometra ao largar; clicar leva a cabeça ao losango, sem snap.
pub(crate) fn apply(
    state: &mut TimelinePanelState,
    snap: &TimelineViewSnapshot,
    index: usize,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => {
            state.macro_drag = Some(MacroDrag {
                index,
                start_x: g.x,
                cur_x: g.x,
            });
        }
        GesturePhase::Update => {
            if let Some(d) = state.macro_drag.as_mut() {
                d.cur_x = g.x;
            }
        }
        GesturePhase::End => {
            let Some(d) = state.macro_drag.take() else {
                return;
            };
            let delta = key_drag::drag_delta_seconds(d.start_x, g.x, state.px_per_s, snap);
            if delta != 0.0
                && let Some(e) = snap.macro_etapas.get(d.index)
            {
                state.macro_pedido = Some(MacroRequest::Edita {
                    entity_bits: e.entity_bits,
                    trilha: e.trilha,
                    k: e.k,
                    campo: CampoDaEtapa::Instante(e.fim + delta),
                });
            }
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            state.macro_drag = None;
            if let Some(e) = snap.macro_etapas.get(index) {
                crate::state::push_intent(TimelineIntent::ScrubExact(e.fim));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap() -> TimelineViewSnapshot {
        TimelineViewSnapshot {
            fps: 60.0,
            frame_snap: true,
            macro_etapas: (0..3)
                .map(|k| ph2d_timeline::EtapaDaMacroView {
                    entity_bits: 9,
                    trilha: 0,
                    k,
                    #[expect(clippy::cast_precision_loss, reason = "poucas etapas")]
                    fim: k as f64,
                    espera_s: 0.25,
                })
                .collect(),
            ..TimelineViewSnapshot::default()
        }
    }

    /// A pré-visualização é a da ESCRITA: só a arrastada anda (W16).
    /// Mutações que sangram: `i == d.index` → `i >= d.index` · `<=` .
    #[test]
    fn arrastar_uma_etapa_mostra_so_ela_a_andar() {
        let s = snap();
        let mut st = TimelinePanelState::default();
        st.macro_drag = Some(MacroDrag {
            index: 1,
            start_x: 100.0,
            cur_x: 100.0 + st.px_per_s as f32,
        });
        let dx: Vec<f32> = (0..3).map(|i| preview_dx(&st, &s, i)).collect();
        assert_eq!((dx[0], dx[2]), (0.0, 0.0), "as outras ficam: {dx:?}");
        assert!(dx[1] > 0.0, "{dx:?}");
    }

    /// Largar pede o instante do losango + Δ (absoluto), e só largar pede.
    #[test]
    fn largar_pede_o_instante_do_losango_mais_o_delta() {
        use ph2d_editor_core::interaction::{GestureMods, TimelineHitKind};
        let s = snap();
        let mut st = TimelinePanelState::default();
        let g = |phase, x| TimelineGesture {
            surface: ph2d_a11y::NodeId(0),
            kind: TimelineHitKind::MacroStep { index: 2 },
            phase,
            x,
            y: 0.0,
            button: ph2d_host::PointerButton::Primary,
            mods: GestureMods::default(),
        };
        apply(&mut st, &s, 2, g(GesturePhase::Begin, 10.0));
        apply(&mut st, &s, 2, g(GesturePhase::Update, 70.0));
        assert_eq!(st.macro_pedido, None, "arrastar não pede antes de largar");
        #[expect(clippy::cast_possible_truncation, reason = "px de ecrã")]
        let meio = (0.5 * st.px_per_s) as f32;
        apply(&mut st, &s, 2, g(GesturePhase::End, 10.0 + meio));
        assert_eq!(
            st.macro_pedido,
            Some(MacroRequest::Edita {
                entity_bits: 9,
                trilha: 0,
                k: 2,
                campo: CampoDaEtapa::Instante(2.5),
            })
        );
        assert_eq!(st.macro_drag, None);
    }

    /// Clicar num losango leva a cabeça ao instante DELE, sem o arredondar ao quadro (W15).
    #[test]
    fn clicar_num_losango_leva_a_cabeca_ao_instante_dele() {
        use ph2d_editor_core::interaction::{GestureMods, TimelineHitKind};
        let s = snap();
        let mut st = TimelinePanelState::default();
        let _ = crate::state::drain_intents();
        apply(
            &mut st,
            &s,
            1,
            TimelineGesture {
                surface: ph2d_a11y::NodeId(0),
                kind: TimelineHitKind::MacroStep { index: 1 },
                phase: GesturePhase::Click,
                x: 10.0,
                y: 0.0,
                button: ph2d_host::PointerButton::Primary,
                mods: GestureMods::default(),
            },
        );
        assert_eq!(
            crate::state::drain_intents(),
            [TimelineIntent::ScrubExact(s.macro_etapas[1].fim)]
        );
    }
}
