//! ⭐ **Os pontos da curva Custom no gráfico da timeline** (dono 15/11: «permita criar pontos no
//! gráfico da timeline e editar sem criar keyframes, exatamente igual é feito no easing custom»).
//!
//! Os MESMOS gestos do editor do cartão da chave, sobre o trecho de uma chave seleccionada na
//! vista de valor: carregar na linha da curva cria um ponto ali (e agarra-o); arrastar um ponto ou
//! uma alça edita-o; o botão direito num ponto apaga-o; Alt ao agarrar uma alça quebra o ponto em
//! Corner; com um ponto escolhido, Corner · Smooth · Symmetric na coluna da esquerda. Nenhuma chave
//! nasce: cada gesto grava a `Interp` do trecho (`SetInterp`), um passo de undo por gesto.
//! A lei (o trecho que tem curva, as contas plano ↔ quadrado) é de `ph2d_timeline::segment_curve`.

use ph2d_anim::{EaseSpline, HandleKind, SplineSide};
use ph2d_editor_core::interaction::{
    GesturePhase, InteractiveState, TimelineGesture, TimelineHitKind, WidgetEvent,
};
use ph2d_editor_core::paint::{fill_circle, fill_rounded_rect, resolve, stroke_polyline};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx};
use ph2d_editor_core::widget::{TabItem, Tabs, TabsVariant, paint_tabs_with_hover};
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerButton;
use ph2d_timeline::{
    KeyView, TimelineIntent, TrackView, interp_of_curve, plane_to_unit, segment_curve,
    unit_to_plane,
};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, StrokeToken, Theme};

use crate::graph::{Band, TimeView};
use crate::graph_paint::{HANDLE_HIT_R, HANDLE_R};
use crate::ids;
use crate::state::{TimelinePanelState, push_intent};

/// O alcance vertical da linha da curva que ainda cria um ponto (o `CURVA_HIT` do cartão).
const LINHA_HIT: f32 = 8.0; // LITERAL-PX-OK: o mesmo alcance do editor do cartão da chave
/// A largura de cada pedaço clicável da linha da curva.
const LINHA_PASSO: f32 = 6.0; // LITERAL-PX-OK: passo dos pedaços de clique ao longo da curva
/// O ponto pinta-se maior que a ponta de uma alça (o cartão: 6 × 5).
const PONTO_R: f32 = HANDLE_R * 1.2; // LITERAL-PX-OK: razão do cartão (PONTO_R / ALCA_R)
/// O miolo da alça (anel): a fração do raio que fica vazia.
const ANEL: f32 = 0.55; // LITERAL-PX-OK: fração do raio, não uma medida

/// Um gesto em curso sobre um ponto (ou numa alça dele), ou o ponto que um carregar na linha da
/// curva vai criar. Resolve-se no `paint`, que conhece a banda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PontoDrag {
    pub target: u64,
    pub key: u64,
    /// O nó da `EaseSpline` (válido depois do primeiro quadro quando `novo`).
    pub node: usize,
    /// `0` = o ponto, `1` = alça de chegada, `2` = alça de saída.
    pub part: u8,
    pub x: f32,
    pub y: f32,
    /// A faixa de valores da banda, congelada no primeiro quadro (como no arrasto de alça).
    pub range: Option<(f64, f64)>,
    pub ending: bool,
    /// Carregou-se na LINHA: o primeiro quadro cria o ponto debaixo do rato.
    pub novo: bool,
    /// Alt ao agarrar uma alça: o primeiro quadro põe o ponto em Corner.
    pub corner: bool,
}

/// O gesto num ponto ou numa alça dele.
pub(crate) fn apply_point_gesture(
    state: &mut TimelinePanelState,
    (target, key, node, part): (u64, u64, u8, u8),
    g: TimelineGesture,
) {
    if g.button == PointerButton::Secondary {
        if part == 0 && matches!(g.phase, GesturePhase::Click) {
            state.ponto_apaga = Some((target, key, usize::from(node)));
        }
        return;
    }
    state.ponto_escolhido = Some((target, key, usize::from(node)));
    let d = PontoDrag {
        target,
        key,
        node: usize::from(node),
        part,
        x: g.x,
        y: g.y,
        range: None,
        ending: false,
        novo: false,
        corner: part != 0 && g.mods.alt,
    };
    segue(state, d, g);
}

/// O gesto na linha da curva: cria um ponto onde se carregou (um toque também cria).
pub(crate) fn apply_line_gesture(
    state: &mut TimelinePanelState,
    (target, key): (u64, u64),
    g: TimelineGesture,
) {
    let d = PontoDrag {
        target,
        key,
        node: 0,
        part: 0,
        x: g.x,
        y: g.y,
        range: None,
        ending: false,
        novo: true,
        corner: false,
    };
    segue(state, d, g);
}

fn segue(state: &mut TimelinePanelState, d: PontoDrag, g: TimelineGesture) {
    match g.phase {
        GesturePhase::Begin => {
            push_intent(TimelineIntent::BeginEdit);
            state.ponto_drag = Some(d);
        }
        GesturePhase::Update => {
            if let Some(a) = state.ponto_drag.as_mut() {
                (a.x, a.y) = (g.x, g.y);
            }
        }
        GesturePhase::End => {
            if let Some(a) = state.ponto_drag.as_mut() {
                (a.x, a.y, a.ending) = (g.x, g.y, true);
            }
        }
        // Um toque: num ponto só o escolhe; na linha cria-o (o quadro resolve e fecha).
        GesturePhase::Click | GesturePhase::DoubleClick => match state.ponto_drag.as_mut() {
            Some(a) if a.novo => a.ending = true,
            _ => {
                state.ponto_drag = None;
                push_intent(TimelineIntent::EndEdit);
            }
        },
    }
}

/// O trecho que sai de `key` (a chave e a seguinte).
fn trecho(track: &TrackView, key: u64) -> Option<(&KeyView, &KeyView)> {
    let i = track.keys.iter().position(|k| k.id.get() == key)?;
    Some((track.keys.get(i)?, track.keys.get(i + 1)?))
}

fn grava(track: &TrackView, k0: &KeyView, c: EaseSpline) {
    push_intent(TimelineIntent::SetInterp {
        target: track.target,
        key: k0.id,
        interp: interp_of_curve(c),
    });
}

/// Resolve neste quadro o que os gestos pediram ao trecho desta banda: apagar, trocar o tipo,
/// criar ou mover. Só na vista de valor.
pub(crate) fn resolve_drag(
    state: &mut TimelinePanelState,
    band: &Band,
    view: TimeView,
    track: &TrackView,
) {
    let target = track.target.get();
    if let Some((_, key, node)) = state.ponto_apaga.filter(|a| a.0 == target) {
        state.ponto_apaga = None;
        if let Some((k0, k1)) = trecho(track, key)
            && let Some(mut c) = segment_curve(k0, k1)
            && c.remove(node)
        {
            state.ponto_escolhido = None;
            push_intent(TimelineIntent::BeginEdit);
            grava(track, k0, c);
            push_intent(TimelineIntent::EndEdit);
        }
    }
    if let Some(tipo) = state.ponto_tipo.take()
        && let Some((t, key, node)) = state.ponto_escolhido
        && t == target
        && let Some((k0, k1)) = trecho(track, key)
        && let Some(mut c) = segment_curve(k0, k1)
    {
        if c.set_kind(node, tipo) {
            push_intent(TimelineIntent::BeginEdit);
            grava(track, k0, c);
            push_intent(TimelineIntent::EndEdit);
        }
    }
    let Some(mut d) = state.ponto_drag.filter(|d| d.target == target) else {
        return;
    };
    // Um toque numa curva PRONTA (Ease, Elastic…) converte-a numa Custom com a mesma cara antes de
    // criar o ponto (dono 15/11) — só aqui, no toque: a conversão custa ~40 ms.
    let curva = trecho(track, d.key).and_then(|(k0, k1)| {
        let c = if d.novo {
            ph2d_timeline::insertable_curve(k0, k1)
        } else {
            segment_curve(k0, k1)
        };
        Some((k0, k1, c?))
    });
    let Some((k0, k1, mut c)) = curva else {
        // O trecho deixou de ter curva (apagado, retimado) a meio do gesto: fecha-se.
        state.ponto_drag = None;
        push_intent(TimelineIntent::EndEdit);
        return;
    };
    let [x, y] = plane_to_unit(k0, k1, view.t(d.x), band.value(d.y));
    if d.novo {
        let Some(i) = c.insert(f64::from(x)) else {
            // Não cabe mais nenhum ponto (ou o rato está em cima de um nó): nada muda.
            state.ponto_drag = None;
            push_intent(TimelineIntent::EndEdit);
            return;
        };
        (d.node, d.novo) = (i, false);
        state.ponto_escolhido = Some((target, d.key, i));
    } else if d.part == 0 {
        c.move_point(d.node, x, y);
    } else {
        if d.corner {
            c.set_kind(d.node, HandleKind::Corner);
            d.corner = false;
        }
        let lado = if d.part == 1 {
            SplineSide::In
        } else {
            SplineSide::Out
        };
        c.move_handle(d.node, lado, x, y);
    }
    grava(track, k0, c);
    if d.ending {
        state.ponto_drag = None;
        push_intent(TimelineIntent::EndEdit);
    } else {
        state.ponto_drag = Some(d);
    }
}

/// A faixa congelada da banda durante um gesto de ponto (ver [`PontoDrag::range`]).
pub(crate) fn frozen_range(state: &TimelinePanelState, target: u64) -> Option<(f64, f64)> {
    let d = state.ponto_drag.as_ref()?;
    (d.target == target).then_some(d.range).flatten()
}

/// Congela a faixa no primeiro quadro do gesto.
pub(crate) fn freeze_range(state: &mut TimelinePanelState, target: u64, range: (f64, f64)) {
    if let Some(d) = state.ponto_drag.as_mut()
        && d.target == target
        && d.range.is_none()
    {
        d.range = Some(range);
    }
}

/// Os trechos desta banda que mostram pontos: os de uma chave seleccionada, com curva editável.
fn trechos(track: &TrackView) -> impl Iterator<Item = (&KeyView, &KeyView, EaseSpline)> {
    track.keys.windows(2).filter_map(|w| {
        let (k0, k1) = (&w[0], &w[1]);
        let c = segment_curve(k0, k1).filter(|_| k0.selected || k1.selected)?;
        Some((k0, k1, c))
    })
}

fn regista(ctx: &mut PaintCtx, band: &Band, id: ph2d_a11y::NodeId, kind: TimelineHitKind, r: Rect) {
    ctx.host.store_mut().register(
        id,
        InteractiveState::TimelineSurface {
            parent: ph2d_editor_core::ids::TIMELINE_PANEL,
            kind,
            canvas: band.rect,
        },
    );
    ctx.host.hit_index_mut().register(id, r);
}

/// Os trechos desta banda cuja LINHA cria pontos: os com curva editável e os de curva pronta
/// (Ease, Elastic…, que o toque converte), de uma chave seleccionada.
fn trechos_de_linha(track: &TrackView) -> impl Iterator<Item = (&KeyView, &KeyView)> {
    track.keys.windows(2).filter_map(|w| {
        let (k0, k1) = (&w[0], &w[1]);
        let pronta = matches!(k0.interp, ph2d_anim::Interp::Eased(_))
            && k1.t_seconds > k0.t_seconds
            && k1.value != k0.value;
        let sim = (k0.selected || k1.selected) && (pronta || segment_curve(k0, k1).is_some());
        sim.then_some((k0, k1))
    })
}

/// Os pedaços clicáveis da LINHA da curva — registados ANTES das âncoras e das alças, que ganham
/// a sobreposição (o índice de cliques dá o último registado). Cada pedaço cobre o troço da curva
/// DESENHADA (o amostrador da animação) na sua coluna, `LINHA_HIT` acima e abaixo.
pub(crate) fn paint_line_hits(ctx: &mut PaintCtx, band: &Band, view: TimeView, track: &TrackView) {
    let target = track.target.get();
    for (k0, k1) in trechos_de_linha(track) {
        let (xa, xb) = (view.x(k0.t_seconds), view.x(k1.t_seconds));
        let (xa, xb) = (xa.max(band.rect.x), xb.min(band.rect.x + band.rect.w));
        let par = [k0.clone(), k1.clone()];
        let y_de = |x: f32| {
            let t = view.t(x).clamp(k0.t_seconds, k1.t_seconds); // CLAMP-OK: dentro do trecho
            band.y(f64::from(
                ph2d_timeline::sample_keys(&par, t).unwrap_or(k0.value),
            ))
        };
        let mut x = xa;
        let mut i = 0u64;
        while x < xb {
            let fim = (x + LINHA_PASSO).min(xb);
            // As pontas e o meio: uma onda do Elastic cabe num pedaço.
            let ys = [y_de(x), y_de(0.5 * (x + fim)), y_de(fim)];
            let cima = ys.iter().copied().fold(f32::INFINITY, f32::min) - LINHA_HIT;
            let baixo = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max) + LINHA_HIT;
            let id = ids::timeline_curve_line_hit_id(target, k0.id.get(), i);
            let kind = TimelineHitKind::CurveLine {
                target,
                key: k0.id.get(),
            };
            regista(
                ctx,
                band,
                id,
                kind,
                Rect::new(x, cima, fim - x, baixo - cima),
            );
            (x, i) = (fim, i + 1);
        }
    }
}

/// Os pontos do meio e as alças deles, sobre a curva — o ponto escolhido no acento, um Corner
/// quadrado (como no cartão). As alças registam-se antes dos pontos: o ponto ganha a sobreposição.
pub(crate) fn paint_points(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    band: &Band,
    view: TimeView,
    track: &TrackView,
) {
    let target = track.target.get();
    let tela = |k0, k1, p| {
        let (t, v) = unit_to_plane(k0, k1, p);
        (view.x(t), band.y(v))
    };
    let braco = resolve(ColorToken::TimelineHandleLine, theme);
    let cor = resolve(ColorToken::TimelineHandle, theme);
    let acento = resolve(ColorToken::TimelineKeyActive, theme);
    let miolo = resolve(ColorToken::BgElev, theme);
    for (k0, k1, c) in trechos(track) {
        let key = k0.id.get();
        for node in 1..c.node_count() - 1 {
            let no = c.node(node);
            let p = tela(k0, k1, no.p);
            for (part, h) in [(1u8, no.h_in), (2, no.h_out)] {
                let a = tela(k0, k1, h);
                stroke_polyline(ctx.scene, &[p, a], StrokeToken::Thin.px(), braco);
                fill_circle(ctx.scene, a.0, a.1, HANDLE_R, cor);
                fill_circle(ctx.scene, a.0, a.1, HANDLE_R * ANEL, miolo);
                let id = ids::timeline_curve_point_hit_id(target, key, node as u8, part);
                let kind = TimelineHitKind::CurvePoint {
                    target,
                    key,
                    node: node as u8,
                    part,
                };
                regista(ctx, band, id, kind, quadrado(a, HANDLE_HIT_R));
            }
            let escolhido = state.ponto_escolhido == Some((target, key, node));
            let tinta = if escolhido { acento } else { cor };
            if c.kind(node) == Some(HandleKind::Corner) {
                let r = quadrado(p, PONTO_R);
                fill_rounded_rect(ctx.scene, r, 0.0, tinta);
            } else {
                fill_circle(ctx.scene, p.0, p.1, PONTO_R, tinta);
            }
            let id = ids::timeline_curve_point_hit_id(target, key, node as u8, 0);
            let kind = TimelineHitKind::CurvePoint {
                target,
                key,
                node: node as u8,
                part: 0,
            };
            regista(ctx, band, id, kind, quadrado(p, HANDLE_HIT_R));
        }
    }
}

/// Na vista de VELOCIDADE (Override), cada ponto do meio é uma chave sem chave: a velocidade dele
/// de cada lado, com o braço horizontal da influência e a ponta que se agarra (o mesmo gesto das
/// chaves: vertical = velocidade, horizontal = influência). Corner num quadrado, como no valor.
pub(crate) fn paint_speed_arms(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    band: &Band,
    view: TimeView,
    track: &TrackView,
) {
    let target = track.target.get();
    let braco = resolve(ColorToken::TimelineHandleLine, theme);
    let cor = resolve(ColorToken::TimelineHandle, theme);
    let acento = resolve(ColorToken::TimelineKeyActive, theme);
    let miolo = resolve(ColorToken::BgElev, theme);
    for (k0, k1, c) in trechos(track) {
        let key = k0.id.get();
        for a in ph2d_timeline::point_speed_arms(k0, k1) {
            let (ax, px, py) = (view.x(a.t), view.x(a.tip_t), band.y(a.speed));
            stroke_polyline(
                ctx.scene,
                &[(ax, py), (px, py)],
                StrokeToken::Thin.px(),
                braco,
            );
            let escolhido = state.ponto_escolhido == Some((target, key, a.node));
            let tinta = if escolhido { acento } else { cor };
            if c.kind(a.node) == Some(HandleKind::Corner) {
                fill_rounded_rect(ctx.scene, quadrado((ax, py), PONTO_R), 0.0, tinta);
            } else {
                fill_circle(ctx.scene, ax, py, PONTO_R, tinta);
            }
            fill_circle(ctx.scene, px, py, HANDLE_R, cor);
            fill_circle(ctx.scene, px, py, HANDLE_R * ANEL, miolo);
            if py < band.rect.y || py > band.rect.y + band.rect.h {
                continue;
            }
            let node = a.node as u8;
            let id = ids::timeline_curve_point_hit_id(target, key, node, a.part);
            let kind = TimelineHitKind::CurvePoint {
                target,
                key,
                node,
                part: a.part,
            };
            regista(ctx, band, id, kind, quadrado((px, py), HANDLE_HIT_R));
        }
    }
}

/// Resolve neste quadro o arrasto de um braço de velocidade de um ponto (vista de velocidade).
pub(crate) fn resolve_speed_drag(
    state: &mut TimelinePanelState,
    band: &Band,
    view: TimeView,
    track: &TrackView,
) {
    let target = track.target.get();
    let Some(mut d) = state.ponto_drag.filter(|d| d.target == target) else {
        return;
    };
    let Some((k0, k1)) = trecho(track, d.key) else {
        state.ponto_drag = None;
        push_intent(TimelineIntent::EndEdit);
        return;
    };
    if d.part != 0 {
        let par = (d.node, d.part);
        let corner = d.corner.then(|| {
            let mut c = segment_curve(k0, k1)?;
            c.set_kind(d.node, HandleKind::Corner).then_some(c)
        });
        let base = match corner.flatten() {
            Some(c) => {
                let mut k = k0.clone();
                k.interp = interp_of_curve(c);
                k
            }
            None => k0.clone(),
        };
        d.corner = false;
        let (t, v) = (view.t(d.x), band.value(d.y));
        if let Some(c) = ph2d_timeline::spline_with_point_speed(&base, k1, par, t, v) {
            grava(track, k0, c);
        }
    }
    if d.ending || d.part == 0 {
        state.ponto_drag = None;
        push_intent(TimelineIntent::EndEdit);
    } else {
        state.ponto_drag = Some(d);
    }
}

fn quadrado((x, y): (f32, f32), r: f32) -> Rect {
    Rect::new(x - r, y - r, r * 2.0, r * 2.0)
}

const TIPOS: [(ph2d_a11y::NodeId, &str, HandleKind); 3] = [
    (
        ids::TIMELINE_POINT_CORNER,
        "panel.vector.vertex.corner",
        HandleKind::Corner,
    ),
    (
        ids::TIMELINE_POINT_SMOOTH,
        "panel.vector.vertex.smooth",
        HandleKind::Smooth,
    ),
    (
        ids::TIMELINE_POINT_SYMM,
        "panel.vector.vertex.symm",
        HandleKind::Symmetric,
    ),
];

/// O tipo de alça do ponto escolhido, se ele é desta banda e ainda existe.
fn tipo_escolhido(state: &TimelinePanelState, track: &TrackView) -> Option<HandleKind> {
    let (t, key, node) = state.ponto_escolhido?;
    if t != track.target.get() {
        return None;
    }
    let (k0, k1) = trecho(track, key)?;
    segment_curve(k0, k1)?.kind(node)
}

/// Corner · Smooth · Symmetric no meio da coluna da esquerda, com um ponto escolhido nesta banda
/// (o lugar da pílula Override · Retime, que só existe na vista de velocidade).
pub(crate) fn paint_kinds(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    (row, band_rect): (Rect, Rect),
    (label_w, track): (f32, &TrackView),
) {
    let Some(agora) = tipo_escolhido(state, track) else {
        return;
    };
    let pad = Spacing::Xs.px();
    let w = label_w - pad * 2.0;
    if w <= 0.0 || band_rect.h < ROW_H_PX + pad * 2.0 {
        return;
    }
    let rect = Rect::new(
        row.x + pad,
        band_rect.y + (band_rect.h - ROW_H_PX) * 0.5,
        w,
        ROW_H_PX,
    );
    let itens = TIPOS
        .iter()
        .map(|(id, chave, _)| TabItem::new(*id, ph2d_i18n::tr(chave)))
        .collect();
    let i = TIPOS.iter().position(|t| t.2 == agora).unwrap_or(1);
    let tabs = Tabs::new(ids::TIMELINE_POINT_KIND, "", itens)
        .selected(i)
        .variant(TabsVariant::Segmented);
    let quente = ctx.host.store().hot_id();
    let hovered = TIPOS.iter().position(|(id, _, _)| quente == Some(*id));
    paint_tabs_with_hover(&tabs, hovered, rect, ctx.scene, ctx.text_system, theme);
    for (j, (id, _, _)) in TIPOS.iter().enumerate() {
        ctx.host
            .hit_index_mut()
            .register(*id, tabs.tab_rect(rect, j));
    }
}

/// O clique num dos três tipos: troca o do ponto escolhido (resolvido no quadro, um passo de undo).
pub(crate) fn apply_event(state: &mut TimelinePanelState, ev: WidgetEvent) -> Option<EventOutcome> {
    let WidgetEvent::Click(id) = ev else {
        return None;
    };
    let tipo = TIPOS.iter().find(|t| t.0 == id)?.2;
    state.ponto_tipo = Some(tipo);
    Some(EventOutcome::Consumed)
}

#[cfg(test)]
#[path = "graph_pontos_tests.rs"]
mod tests;
