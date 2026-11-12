//! ⭐ **Arrastar um chip da barra para uma faixa** (`docs/Timeline/17_plano_funcoes_novas.md`, item 6,
//! maquete B4). O chip é um [`TimelineHitKind::SourceChip`]: o clique escolhe-o como sempre; o arrasto
//! para além da folga escolhe-o TAMBÉM e leva-o — na faixa sob o ponteiro pinta-se o fantasma da peça
//! (a fonte, a duração que ela terá, no tempo do ponteiro encaixado ao quadro) e a largada é o MESMO
//! `AddStrip` do «+» da faixa. Abaixo da última faixa o fantasma mora numa faixa nova e a largada cria
//! a faixa e a peça num passo de undo (Unity Timeline / Premiere). Fora da área das faixas, ou Esc:
//! nada. Um container não se larga dentro de si mesmo (o fantasma diz a recusa).
//!
//! O gesto só grava onde o ponteiro está; o alvo resolve-se na pintura, onde a geometria das faixas é
//! conhecida (o mesmo desenho do `box_select::commit`).

use std::cell::Cell;

use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{
    GesturePhase, PRESS_SLOP_PX, TimelineGesture, TimelineHitKind, WidgetEvent,
};
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, rect_to_vello, stroke_rounded_rect_dashed,
};
use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{StripSource, TimelineIntent, TimelineViewSnapshot};
use ph2d_tokens::chrome::{TIMELINE_FANTASMA_ALPHA, TIMELINE_FANTASMA_TRACO_PX};
use ph2d_tokens::{ColorToken, PECAS, StrokeToken, TIMELINE_PECA_RAIO_PX, Theme, TimelineTinta};
use ph2d_vector::Color;

use crate::geom;
use crate::graph::TimeView;
use crate::pecas_cartoes::{PecaDesenho, indice_do_clipe, pinta_rotulo};
use crate::state::{self, TimelinePanelState};

/// How long a strip of an EMPTY clip is: a clip with no keys has no duration, and
/// a strip of zero seconds paints as nothing and cannot be grabbed to fix.
///
/// It floors ONLY the empty case. Padding a short clip's span to a second used to
/// look harmless and was not: a 0.4 s clip in a 1 s box is a strip playing at 0.4x
/// before anyone asked it to (`slice == span * speed`), and the first stretch would
/// have snapped its rate to match.
const MIN_NEW_STRIP_S: f64 = 1.0;

/// O chip a caminho: onde foi apertado, onde o ponteiro está e o alvo resolvido na última pintura.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ChipDrag {
    /// O `NodeId` cru do chip (o da opção da lista).
    pub(crate) option: u64,
    x0: f32,
    y0: f32,
    x: f32,
    y: f32,
    /// Passou da folga: é um arrasto, não um clique.
    pub(crate) ativo: bool,
    /// O botão soltou: a próxima pintura larga (ou não) e esquece.
    solto: bool,
    pub(crate) alvo: Option<Alvo>,
}

/// Onde a peça nasceria, e como.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Alvo {
    /// A faixa; `== lanes.len()` quando [`Self::nova`].
    pub(crate) lane: usize,
    /// Abaixo da última faixa: a largada cria a faixa.
    pub(crate) nova: bool,
    pub(crate) source: StripSource,
    pub(crate) t: f64,
    pub(crate) len: f64,
    /// Um container dentro de si mesmo: o fantasma diz não e a largada não faz nada.
    pub(crate) recusa: bool,
    /// Numa linha de DENTRO da peça aberta: `(instância, lane de dentro)` — a largada sai embrulhada.
    pub(crate) dentro: Option<(ph2d_timeline::StripId, usize)>,
    /// O rect da peça nova — o mesmo que a strip pinta depois da largada.
    pub(crate) rect: Rect,
}

thread_local! {
    static ARRASTADO: Cell<Option<u64>> = const { Cell::new(None) };
}

/// O chip que está na mão (para o realce `.dr` da barra), se há um a arrastar.
pub(crate) fn arrastado() -> Option<u64> {
    ARRASTADO.get()
}

/// Publica o chip na mão para a barra deste quadro.
pub(crate) fn publica(state: &TimelinePanelState) {
    ARRASTADO.set(state.chip_drag.filter(|d| d.ativo).map(|d| d.option));
}

/// ⭐ **A fonte ativa e a duração da peça que ela faz** — a porta única do «+» da faixa e da largada:
/// o container escolhido (a duração dele) ou o clipe (`transport_sources::placed_clip`); vazio → 1 s.
pub(crate) fn fonte_ativa(
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
) -> (StripSource, f64) {
    let (source, len) = match state.source_container {
        Some(c) => (
            StripSource::Container(u16::try_from(c).unwrap_or(u16::MAX)),
            snap.containers.get(c).map_or(0.0, |v| v.length),
        ),
        // The clip — the active one, or one of ANOTHER animation picked in the Scene's
        // Arrange (onda 4, item 5) — at its EFFECTIVE length (last key, not authored
        // duration): the number `add_strip` sizes the slice to, so it is born at speed 1.
        None => crate::transport_sources::placed_clip(state, snap),
    };
    (source, if len > 0.0 { len } else { MIN_NEW_STRIP_S })
}

/// Um gesto sobre o chip `option`.
pub(crate) fn apply(
    state: &mut TimelinePanelState,
    host: &mut dyn PanelHostInternal,
    option: u64,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => {
            state.chip_drag = Some(ChipDrag {
                option,
                x0: g.x,
                y0: g.y,
                x: g.x,
                y: g.y,
                ativo: false,
                solto: false,
                alvo: None,
            });
        }
        GesturePhase::Update | GesturePhase::End => {
            let Some(mut d) = state.chip_drag else {
                return; // o Esc já cancelou
            };
            d.x = g.x;
            d.y = g.y;
            let antes = d.ativo;
            d.ativo |= (g.x - d.x0).abs().max((g.y - d.y0).abs()) > PRESS_SLOP_PX;
            d.solto = matches!(g.phase, GesturePhase::End);
            if d.solto && !d.ativo {
                state.chip_drag = None;
                escolhe(state, host, option);
                return;
            }
            state.chip_drag = Some(d);
            // Levar o chip é também escolhê-lo: a largada põe a fonte ATIVA, a mesma do «+».
            if d.ativo && !antes {
                escolhe(state, host, option);
            }
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            state.chip_drag = None;
            escolhe(state, host, option);
        }
    }
}

/// O clique de sempre do chip (a linha da lista que ele é).
fn escolhe(state: &mut TimelinePanelState, host: &mut dyn PanelHostInternal, option: u64) {
    let _ = crate::transport_clips::apply_event(state, host, WidgetEvent::Click(NodeId(option)));
}

/// Resolve o alvo contra a geometria deste quadro e, se o botão soltou, larga.
pub(crate) fn alveja(
    state: &mut TimelinePanelState,
    g: &geom::Geom,
    view: TimeView,
    snap: &TimelineViewSnapshot,
) {
    let Some(mut d) = state.chip_drag.filter(|d| d.ativo) else {
        return;
    };
    d.alvo = alvo(state, g, view, snap, (d.x, d.y));
    if !d.solto {
        state.chip_drag = Some(d);
        return;
    }
    state.chip_drag = None;
    publica(state);
    if let Some(a) = d.alvo.filter(|a| !a.recusa) {
        larga(a);
    }
}

/// A largada: o `AddStrip` do «+»; numa faixa nova, a faixa e a peça num só passo de undo.
fn larga(a: Alvo) {
    let peca = TimelineIntent::AddStrip {
        lane: a.lane,
        source: a.source,
        t_start: a.t,
        t_end: a.t + a.len,
    };
    if let Some((instancia, _)) = a.dentro {
        state::push_intent(TimelineIntent::NoLugar {
            instancia,
            intent: Box::new(peca),
        });
    } else if a.nova {
        for i in [
            TimelineIntent::BeginEdit,
            TimelineIntent::AddLane,
            peca,
            TimelineIntent::EndEdit,
        ] {
            state::push_intent(i);
        }
    } else {
        state::push_intent(peca);
    }
}

fn alvo(
    state: &TimelinePanelState,
    g: &geom::Geom,
    view: TimeView,
    snap: &TimelineViewSnapshot,
    (x, y): (f32, f32),
) -> Option<Alvo> {
    if crate::tab::rows(state.tab, snap) != crate::tab::Rows::Lanes {
        return None;
    }
    let rows = g.rows;
    if x < view.time_x || x >= view.right || y < rows.y || y >= rows.y + rows.h {
        return None;
    }
    let n = snap.lanes.len();
    // A linha sob o ponteiro pelas MESMAS bandas que a pintura usa (as de dentro incluídas).
    let aberta = crate::no_lugar::vista(snap, state.tab);
    let (lane, dentro, y0, h) = match geom::linha_sob(snap, state.tab, rows.y, state.scroll_y, y)? {
        geom::LinhaSob::Fora { lane, top, h } => (lane, None, top, h),
        geom::LinhaSob::Dentro { k, top, h } => (k, aberta.map(|v| (v.instancia, k)), top, h),
        geom::LinhaSob::Nova { top, h } => (n, None, top, h),
    };
    let nova = dentro.is_none() && lane == n;
    let (source, len) = fonte_ativa(state, snap);
    // Numa linha de dentro: o quadro do container, e a peça nasce a velocidade 1 LÁ DENTRO.
    let (t, len) = match (dentro, aberta) {
        (Some(_), Some(v)) => (
            crate::strip_drag::snapped_em(snap, n + lane, view.t(x)),
            len * v.mapa.escala().abs(),
        ),
        _ => (crate::strip_drag::snapped(state, snap, view.t(x)), len),
    };
    let dentro_de_si = match source {
        StripSource::Container(c) => {
            let c = usize::from(c);
            crate::state::edit_path().iter().any(|p| p.container == c)
                // …nem no interior aberto dele mesmo (os ciclos fundos o modelo recusa).
                || (dentro.is_some() && aberta.is_some_and(|v| v.container == c))
        }
        StripSource::Clip(_) => false,
    };
    let pad = geom::peca_margem();
    #[expect(clippy::cast_possible_truncation, reason = "a span on screen")]
    let w = (len * view.px_per_s) as f32;
    Some(Alvo {
        lane,
        nova,
        source,
        t,
        len,
        recusa: dentro_de_si || (nova && n >= ph2d_timeline::MAX_LANES),
        dentro,
        rect: Rect::new(view.x(t), y0 + pad, w, (h - pad * 2.0).max(0.0)),
    })
}

/// A cor da peça que a fonte faz (a mesma que a strip vestirá).
fn tinta_da_fonte(state: &TimelinePanelState, snap: &TimelineViewSnapshot) -> TimelineTinta {
    match fonte_ativa(state, snap).0 {
        StripSource::Container(_) => TimelineTinta::PecaContainer,
        StripSource::Clip(_) => PECAS[indice_do_clipe(snap, &nome(state, snap)) % PECAS.len()],
    }
}

/// O nome da fonte ativa.
fn nome(state: &TimelinePanelState, snap: &TimelineViewSnapshot) -> String {
    match fonte_ativa(state, snap).0 {
        StripSource::Container(c) => snap
            .containers
            .get(usize::from(c))
            .map(|c| c.name.clone())
            .unwrap_or_default(),
        StripSource::Clip(i) => state.source_clip.as_ref().map_or_else(
            || {
                snap.clip_index
                    .iter()
                    .position(|&k| k == usize::from(i))
                    .and_then(|p| snap.clips.get(p))
                    .cloned()
                    .unwrap_or_default()
            },
            |(_, n)| n.clone(),
        ),
    }
}

/// As tintas do fantasma: `[fundo, contorno]` — a cor da fonte, ou o perigo na recusa.
pub(crate) fn cores_do_fantasma(
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
    recusa: bool,
    theme: Theme,
) -> [Color; 2] {
    let cor = if recusa {
        ph2d_editor_core::paint::resolve(ColorToken::Danger, theme)
    } else {
        crate::folha_tinta::tinta(tinta_da_fonte(state, snap), theme)
    };
    [cor.multiply_alpha(TIMELINE_FANTASMA_ALPHA), cor]
}

/// A cor da fonte do chip `option` na mão (o realce `.dr`); `None` se não é ele.
pub(crate) fn tinta_do_chip(
    option: NodeId,
    snap: &TimelineViewSnapshot,
    nome_do_chip: &str,
) -> Option<TimelineTinta> {
    (arrastado() == Some(option.0)).then(|| match crate::transport_clips::source_at(option) {
        Some(Ok(_)) => TimelineTinta::PecaContainer,
        _ => PECAS[indice_do_clipe(snap, nome_do_chip) % PECAS.len()],
    })
}

/// O fantasma da peça sob o ponteiro, recortado à área de tempo das faixas.
pub(crate) fn paint_fantasma(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    view: TimeView,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
) {
    let Some(a) = state.chip_drag.and_then(|d| d.alvo) else {
        return;
    };
    let banda = Rect::new(
        view.time_x,
        g.rows.y,
        (view.right - view.time_x).max(0.0),
        g.rows.h,
    );
    let [fundo, borda] = cores_do_fantasma(state, snap, a.recusa, theme);
    let r = a.rect;
    let raio = frame_radius(theme, TIMELINE_PECA_RAIO_PX)
        .min(r.h * 0.5)
        .min(r.w * 0.5);
    ctx.scene.push_clip(&rect_to_vello(banda));
    fill_rounded_rect(ctx.scene, r, raio, fundo);
    let fio = StrokeToken::Thin.px();
    // O contorno TRACEJADO é a marca do fantasma (`1px dashed` da B4), na cor da fonte.
    stroke_rounded_rect_dashed(
        ctx.scene,
        Rect::new(r.x + fio * 0.5, r.y + fio * 0.5, r.w - fio, r.h - fio),
        raio,
        fio,
        (TIMELINE_FANTASMA_TRACO_PX, TIMELINE_FANTASMA_TRACO_PX),
        borda,
    );
    let nome = nome(state, snap);
    let p = PecaDesenho {
        tinta: tinta_da_fonte(state, snap),
        icon: if matches!(a.source, StripSource::Container(_)) {
            IconId::Cube
        } else {
            IconId::Film
        },
        nome: &nome,
        extra: None,
        muda: false,
        escolhida: false,
        theme,
    };
    pinta_rotulo(ctx, r, (r.x, r.x + r.w), &p);
    ctx.scene.pop_layer();
}

/// O Esc cancela o chip na mão; devolve se havia um.
pub(crate) fn cancela(state: &mut TimelinePanelState) -> bool {
    state.chip_drag.take().is_some()
}

/// A superfície que o chip regista no lugar do botão.
pub(crate) const fn superficie(option: NodeId) -> TimelineHitKind {
    TimelineHitKind::SourceChip { option: option.0 }
}

#[cfg(test)]
#[path = "chip_drag_tests.rs"]
mod tests;
