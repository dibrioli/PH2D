//! Track list (W2.E4) — the left label column of the dope-sheet: the "+Track"
//! dropdown (header row, aligned with the ruler strip) and one row per binding
//! (property label + a short object tag).
//!
//! "+Track" is a dropdown button; opening it lists the six properties as a
//! popover overlay. Picking one raises `PanelEvent::Click(<prop id>)`; the shell
//! binds the *currently selected* sprite's matching property (it owns the
//! selection) and closes the dropdown. Each row's time area paints its key
//! **diamonds** (selected keys in the accent colour), culled to the visible
//! span, and registers their [`TimelineHitKind`] hit targets (+ a `Lane`
//! background) so the dope-sheet gestures reach `interact` (E5b: click-select,
//! drag-move, clear-on-empty).

#[path = "tracks_label.rs"]
mod label;
#[path = "tracks_lista.rs"]
mod lista;
#[cfg(test)]
pub(crate) use label::track_label;
pub(crate) use label::{objeto_snap, prop_label, propriedade_snap, track_label_snap};
#[cfg(test)]
use lista::lista_y;
pub(crate) use lista::paint_add_track_popover;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind, TrackMenuKind};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_centered, rect_to_vello, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Button, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{Extrap, TimelineViewSnapshot};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Affine, BezPath, Brush, Fill, Stroke};

use crate::ids;
use crate::linhas::Linha;
use crate::state::TimelinePanelState;
use crate::{geom, graph, graph_paint, summary_paint};

/// Width of the left label column (property + object tag + the live value). Era `132`: cresceu
/// a fatia do número (`VALUE_W` + respiro), senão o nome da row sai cortado.
pub(crate) const LABEL_COL_W: f32 = 176.0; // LITERAL-PX-OK: track-label column width
/// Width of the expand/collapse twirl at the head of each row.
pub(crate) const TWIRL_W: f32 = 24.0; // LITERAL-PX-OK: row twirl hit column width
/// Half the twirl triangle's flat edge.
const TWIRL_BASE_HALF: f32 = 5.0; // LITERAL-PX-OK: row twirl triangle half-base
/// Height of the twirl triangle, flat edge to apex.
const TWIRL_TIP: f32 = 9.0; // LITERAL-PX-OK: row twirl triangle height
pub(crate) const DIAMOND_H: f32 = 4.5; // LITERAL-PX-OK: keyframe diamond half-size
/// A ROVING key draws as a small dot instead of the diamond (AE convention:
/// its time is derived, not authored).
const ROVE_DOT_R: f32 = 2.5; // LITERAL-PX-OK: roving key dot radius
/// Horizontal half-width of a key's clickable hit rect (larger than the visual
/// diamond so a small target is easy to grab).
pub(crate) const KEY_HIT_HW: f32 = 7.0; // LITERAL-PX-OK: keyframe grab half-width

/// A meia-diagonal do losango: a métrica dos cartões (`ph2d_tokens`), ou a clássica.
pub(crate) fn chave_h() -> f32 {
    if ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        ph2d_tokens::TIMELINE_KEY_HALF_PX
    } else {
        DIAMOND_H
    }
}

/// A meia-largura do alvo de clique da chave: cobre o losango e o contorno que se pintam.
pub(crate) fn chave_hit_hw() -> f32 {
    if ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        ph2d_tokens::TIMELINE_KEY_HIT_HALF_PX
    } else {
        KEY_HIT_HW
    }
}

/// ⭐ **A chave pintada** (doc 15 onda 2, dono 09/11): nos cartões, o losango do tamanho da maquete
/// com contorno escuro (`Bg0`) por baixo, para se ler sobre a barra; no clássico, o de sempre.
pub(crate) fn paint_chave(ctx: &mut PaintCtx, theme: Theme, cx: f32, cy: f32, tok: ColorToken) {
    if ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        let h = ph2d_tokens::TIMELINE_KEY_HALF_PX;
        let anel = h + ph2d_tokens::TIMELINE_KEY_RING_PX;
        let t = match tok {
            ColorToken::TimelineKeySelected => ph2d_tokens::TimelineTinta::ChaveEscolhida,
            _ => ph2d_tokens::TimelineTinta::Chave,
        };
        let fundo = crate::folha_tinta::tinta(ph2d_tokens::TimelineTinta::FitaPropriedade, theme);
        paint_diamond(ctx, cx, cy, anel, fundo);
        paint_diamond(ctx, cx, cy, h, crate::folha_tinta::tinta(t, theme));
    } else {
        paint_diamond(ctx, cx, cy, DIAMOND_H, resolve(tok, theme));
    }
}
/// Extrapolation marks (plan §6): a dashed line + mode badge on the dope-sheet
/// over the region a non-Hold Pre/Post governs. Dash geometry + the smallest
/// region worth marking / labelling.
const DASH_LEN: f32 = 4.0; // LITERAL-PX-OK: extrapolation dash length
const DASH_GAP: f32 = 3.0; // LITERAL-PX-OK: extrapolation dash gap
const DASH_TH: f32 = 1.0; // LITERAL-PX-OK: extrapolation dash thickness
/// Horizontal padding inside the mode badge (each side). The badge width is the
/// MEASURED label width plus this, so no label ever wraps (a fixed width clipped
/// "Ping-Pong" to two lines) and it survives i18n.
const EXTRAP_BADGE_PAD: f32 = 4.0; // LITERAL-PX-OK: extrapolation badge horizontal padding
/// Clearance between a boundary key and the badge, so the label never overlaps
/// the diamond it sits beside.
const EXTRAP_BADGE_GAP: f32 = 10.0; // LITERAL-PX-OK: extrapolation badge clearance from the key
const EXTRAP_MARK_MIN_W: f32 = 6.0; // LITERAL-PX-OK: skip a region too small to dash

/// Paint the "+ Track" dropdown button filling `header` (the label-column slice
/// aligned with the ruler strip). The property list opens as an overlay popover
/// (see [`paint_add_track_popover`], painted last).
pub(crate) fn paint_add_track(ctx: &mut PaintCtx, theme: Theme, header: Rect, aberta: bool) {
    if ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        crate::stack_add_header::lista_de_cartao(
            ctx,
            theme,
            ids::TIMELINE_ADD_TRACK,
            aberta,
            header,
        );
        return;
    }
    let state = ctx.host.store().button_visual(ids::TIMELINE_ADD_TRACK);
    let btn = Button::new(
        ids::TIMELINE_ADD_TRACK,
        ph2d_i18n::tr("panel.timeline.add_track"),
    )
    .visual(state);
    paint_button(&btn, header, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_ADD_TRACK, header);
}

/// Paint one row per binding in `region`: the label (left `label_w` column) and
/// the key diamonds in the time area (mapped by `time_x`/`view_start`/`px_per_s`,
/// culled to the visible span). Selected keys paint in the accent colour, shifted
/// by `preview_dx` px while a key drag is in flight.
///
/// Registers a [`TimelineHitKind::Lane`] background over the time area (click =
/// clear selection) and one [`TimelineHitKind::Key`] hit per visible diamond
/// (click = select, drag = move), so `dispatch` streams gestures the panel's
/// `interact` step drains. Hits are keyed by the key's *identity* (stable across
/// frames), not its position.
pub(crate) fn paint_rows(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    view: graph::TimeView,
    preview_dx: f32,
    state: &mut TimelinePanelState,
    snap: &TimelineViewSnapshot,
) {
    let (region, label_w) = (g.rows, g.label_w);
    let bottom = region.y + region.h;
    // Lane background over the time area — a click here clears the selection or
    // rubber-bands a box-select.
    let lane = lane_rect(g, view);
    ctx.host.store_mut().register(
        ids::TIMELINE_LANES,
        InteractiveState::TimelineSurface {
            parent: ph2d_editor_core::ids::TIMELINE_PANEL,
            kind: TimelineHitKind::Lane,
            canvas: lane,
        },
    );
    ctx.host.hit_index_mut().register(ids::TIMELINE_LANES, lane);

    // Rows overflow the band while scrolling and an expanded one is tall; clip
    // so a half-visible graph never bleeds over the ruler or the scrollbar.
    ctx.scene.push_clip(&rect_to_vello(region));
    if ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        crate::linhas::pinta_folha_maquete(ctx, theme, g);
    }
    // The stack band, above everything, scrolling with it. Two painters over ONE band, each
    // refusing on the same question (`tab::rows`): lanes on Arrange and inside a container,
    // the container LIST at the Containers tab's root.
    crate::stack_lane_paint::paint(ctx, theme, g, view, state, snap);
    crate::container_list::paint(ctx, theme, g, view, state, snap);
    // The master row, above track 0 and scrolling with it.
    summary_paint::paint(ctx, theme, g, view, preview_dx, state, snap);
    let lay = crate::linhas::layout(
        snap,
        state.tab,
        &state.expanded,
        &state.grupos_abertos,
        state.graph_h,
        region.y,
        state.scroll_y,
    );
    // A linha «Macro steps» (W5d) vai logo a seguir à última faixa.
    let fim = lay.bandas.last().map_or(
        region.y - state.scroll_y
            + geom::stack_h(snap, state.tab)
            + geom::summary_h(snap, state.tab),
        |b| b.y + b.h,
    );
    let arvore = ph2d_editor_core::panel::cards::ferramenta::cards_on();
    let unidades = crate::tracks_value::Unidades::of(ctx.host.project());
    for (n, b) in lay.bandas.iter().enumerate() {
        let (y, h) = (b.y, b.h);
        // Cull the rows outside the band entirely: a long track list then neither
        // paints nor registers off-screen hits.
        if y + h <= region.y || y >= bottom {
            continue;
        }
        // Zebra so lanes read as discrete rows; o cabeçalho do objecto tem o fundo dele. Nos
        // cartões, a maquete B1: coluna e fita com tons próprios e um fio em vez da zebra.
        if arvore {
            crate::linhas::pinta_linha_maquete(ctx, theme, g, b.linha, (y, ROW_H_PX));
        } else if let Some(tok) = crate::linhas::fundo_da_linha(b.linha, n) {
            fill_row(ctx, theme, Rect::new(region.x, y, region.w, ROW_H_PX), tok);
        }
        let (i, grupo) = match b.linha {
            Linha::Grupo(gi) => {
                let aberto = state.grupos_abertos.contains(&lay.grupos[gi].chave);
                let linha = (&lay.grupos[gi], y, aberto);
                crate::linhas::paint_grupo(ctx, theme, g, view, preview_dx, snap, linha);
                continue;
            }
            Linha::Objeto(e) => {
                crate::linhas::paint_objeto(ctx, theme, g, view, preview_dx, snap, (e, y));
                continue;
            }
            Linha::Faixa { i, grupo } => (i, grupo.map(|gi| &lay.grupos[gi])),
        };
        let track = &snap.tracks[i];
        // Um membro de um grupo aberto recua uma coluna, sob o ▶ do grupo.
        // Nos cartões a faixa recua sob o cabeçalho do objecto, e leva o ícone da propriedade.
        let x0 = region.x + if grupo.is_some() { TWIRL_W } else { 0.0 };
        let icone = if arvore {
            crate::linhas::icone_da_propriedade(track.prop)
        } else {
            None
        };
        let t = track.target.get();
        let twirl = TimelineHitKind::Twirl { target: t };
        let open = state.is_expanded(t);
        paint_twirl(ctx, theme, open, (x0, y), ids::timeline_twirl_id(t), twirl);
        // The label slice (after the twirl) is a right-click surface: dispatch
        // opens the track-row context menu on it (Delete Track). Inert to
        // primary gestures — see the `Row` arm in `interact`.
        let row_id = ids::timeline_row_id(track.target.get());
        let row_hit = Rect::new(
            x0 + TWIRL_W,
            y,
            (region.x + label_w - x0 - TWIRL_W).max(0.0),
            ROW_H_PX,
        );
        ctx.host.store_mut().register(
            row_id,
            InteractiveState::TimelineSurface {
                parent: ph2d_editor_core::ids::TIMELINE_PANEL,
                kind: TimelineHitKind::Row {
                    target: track.target.get(),
                    // Só o painel conhece o `PropKind` da row, e é ele que decide qual
                    // menu o botão direito abre — cada família de track tem ações que
                    // as outras não têm, e oferecê-las em toda row seria item morto.
                    menu: match track.prop {
                        ph2d_timeline::PropKind::Position => TrackMenuKind::Path,
                        ph2d_timeline::PropKind::TranslationX
                        | ph2d_timeline::PropKind::TranslationY => TrackMenuKind::Axis,
                        // Time Remap is sampled through its own clock, so per-track
                        // extrapolation is inert on it — a menu without the cascades.
                        ph2d_timeline::PropKind::TimeRemap => TrackMenuKind::TimeRemap,
                        _ => TrackMenuKind::Plain,
                    },
                },
                canvas: row_hit,
            },
        );
        ctx.host.hit_index_mut().register(row_id, row_hit);
        let text = match grupo {
            Some(gr) => crate::linhas::rotulo_membro(snap, gr, i),
            None if arvore => label::propriedade_snap(snap, track.prop, unidades),
            None => track_label_snap(snap, track.entity, track.prop),
        };
        let color = if track.missing {
            resolve(ColorToken::TimelineMissing, theme)
        } else if arvore {
            crate::folha_tinta::tinta(ph2d_tokens::TimelineTinta::TextoPropriedade, theme)
        } else {
            resolve(ColorToken::Text1, theme)
        };
        // O nome (elidido, nunca quebrado) e o VALOR VIVO da propriedade — os dois no irmão
        // `tracks_value`, porque a fatia de um é o que sobra do outro.
        crate::tracks_value::paint_name_and_value(
            ctx,
            theme,
            snap,
            track,
            (&text, color),
            (
                x0 + TWIRL_W + crate::linhas::icone_w(icone),
                region.x + label_w,
                y,
            ),
        );
        if let Some(ic) = icone {
            let cor = crate::folha_tinta::tinta(ph2d_tokens::TimelineTinta::TextoValor, theme);
            crate::linhas::paint_icone(ctx, ic, (x0 + TWIRL_W, y), cor);
        }
        if arvore {
            crate::linhas::paint_barras(ctx, theme, track, view, preview_dx, y);
        }
        paint_lane_keys(ctx, theme, track, view, preview_dx, y, lane);
        if h > ROW_H_PX {
            let band = Rect::new(region.x, y + ROW_H_PX, region.w, h - ROW_H_PX);
            graph_paint::paint_track(ctx, theme, state, band, label_w, view, track, &snap.values);
        }
    }
    // The time-scale box + grips over the current key selection (crown-jewels §4).
    // Registered AFTER the diamonds so the grips win the hit where they sit.
    crate::macro_steps::paint(ctx, theme, region, label_w, view, state, snap, fim, lane);
    crate::scale_drag::paint_selection_box(ctx, theme, g, view, state, snap);
    ctx.scene.pop_layer();
}

/// The time area of the rows band — the canvas every dope-sheet key hit reports.
pub(crate) fn lane_rect(g: &geom::Geom, view: graph::TimeView) -> Rect {
    let right = g.rows.x + g.rows.w;
    Rect::new(
        view.time_x,
        g.rows.y,
        (right - view.time_x).max(0.0),
        g.rows.h,
    )
}

/// One row's key diamonds + their hit targets, on the dope-sheet strip.
fn paint_lane_keys(
    ctx: &mut PaintCtx,
    theme: Theme,
    track: &ph2d_timeline::TrackView,
    view: graph::TimeView,
    preview_dx: f32,
    y: f32,
    lane: Rect,
) {
    let (time_x, right) = (view.time_x, view.right);
    let cy = y + ROW_H_PX * 0.5;
    // The extrapolation marks go UNDER the diamonds (drawn first): a dashed line +
    // a mode badge over the region a non-Hold Pre/Post governs.
    paint_extrap_marks(ctx, theme, track, view, cy);
    for k in &track.keys {
        let base_x = time_x + ((k.t_seconds - view.view_start) * view.px_per_s) as f32;
        // Selected keys ride the live drag preview.
        let kx = if k.selected {
            base_x + preview_dx
        } else {
            base_x
        };
        if kx < time_x - chave_h() || kx > right + chave_h() {
            continue;
        }
        let tok = if k.selected {
            ColorToken::TimelineKeySelected
        } else {
            ColorToken::TimelineKey
        };
        if k.roving {
            // AE convention: a roving key is a small circle, not a diamond.
            ph2d_editor_core::paint::fill_circle(
                ctx.scene,
                kx,
                cy,
                ROVE_DOT_R,
                resolve(tok, theme),
            );
        } else {
            paint_chave(ctx, theme, kx, cy, tok);
        }
        // Hit target keyed by identity (stable across frames/reorders). The grab
        // rect follows the drawn (previewed) position.
        let id = ids::timeline_key_hit_id(track.target.get(), k.id.get());
        let hit = Rect::new(kx - chave_hit_hw(), y, chave_hit_hw() * 2.0, ROW_H_PX);
        ctx.host.store_mut().register(
            id,
            InteractiveState::TimelineSurface {
                parent: ph2d_editor_core::ids::TIMELINE_PANEL,
                kind: TimelineHitKind::Key {
                    target: track.target.get(),
                    key: k.id.get(),
                },
                canvas: lane,
            },
        );
        ctx.host.hit_index_mut().register(id, hit);
    }
}

/// The i18n key for a mode's badge — `None` for [`Extrap::Hold`], the default
/// that is DRAWN AS NOTHING. This one door is the "only when it is not the plain
/// Hold" rule the marks obey (both the dash and the badge ask it).
fn extrap_i18n_key(mode: Extrap) -> Option<&'static str> {
    match mode {
        Extrap::Hold => None,
        Extrap::Loop => Some("panel.timeline.extrap.loop"),
        Extrap::PingPong => Some("panel.timeline.extrap.pingpong"),
        Extrap::Continue => Some("panel.timeline.extrap.continue"),
    }
}

/// Mark the dope-sheet regions a non-Hold Pre/Post governs: a dashed line from the
/// time-area edge to the first/last key, plus a small mode badge. Nothing is drawn
/// for `Hold` (the clean default) — the region's SIDE (left of the first key vs
/// right of the last) reads Pre vs Post, and the badge names the mode.
fn paint_extrap_marks(
    ctx: &mut PaintCtx,
    theme: Theme,
    track: &ph2d_timeline::TrackView,
    view: graph::TimeView,
    cy: f32,
) {
    let (Some(first), Some(last)) = (track.keys.first(), track.keys.last()) else {
        return; // no key to anchor the extrapolated region to
    };
    let (time_x, right) = (view.time_x, view.right);
    let x_at = |t: f64| time_x + ((t - view.view_start) * view.px_per_s) as f32;
    let (first_x, last_x) = (x_at(first.t_seconds), x_at(last.t_seconds));
    // Pre: the region left of the first key, clamped to the visible time area. The
    // badge sits at the INNER edge (adjacent to the keys), where it reads.
    if let Some(key) = extrap_i18n_key(track.pre) {
        let (a, b) = (time_x, first_x.min(right));
        paint_extrap_span(ctx, theme, a, b, cy, key, false);
    }
    // Post: the region right of the last key.
    if let Some(key) = extrap_i18n_key(track.post) {
        let (a, b) = (last_x.max(time_x), right);
        paint_extrap_span(ctx, theme, a, b, cy, key, true);
    }
}

/// One extrapolation span `[a, b]` at `cy`: a dashed line plus the mode badge (i18n
/// `key`) at the inner edge — `b` for the Pre side, `a` for the Post side (`post`).
fn paint_extrap_span(
    ctx: &mut PaintCtx,
    theme: Theme,
    a: f32,
    b: f32,
    cy: f32,
    key: &str,
    post: bool,
) {
    if b - a < EXTRAP_MARK_MIN_W {
        return; // region too small to read a dash
    }
    let dash = resolve(ColorToken::Text3, theme);
    let mut x = a;
    while x < b {
        let seg = (x + DASH_LEN).min(b) - x;
        fill_rounded_rect(
            ctx.scene,
            Rect::new(x, cy - DASH_TH * 0.5, seg, DASH_TH),
            0.0,
            dash,
        );
        x += DASH_LEN + DASH_GAP;
    }
    // Badge: MEASURED so no label wraps ("Ping-Pong" clipped to two lines under a
    // fixed width), and sat a gap AWAY from the boundary key so the label never
    // overlaps the diamond. Only drawn when the region can hold badge + gap.
    let label = ph2d_i18n::tr(key);
    let text_w = ctx
        .text_system
        .layout(label, TypeToken::Xs.px(), f32::INFINITY)
        .width();
    let badge_w = text_w + EXTRAP_BADGE_PAD * 2.0;
    if b - a >= badge_w + EXTRAP_BADGE_GAP {
        let bx = if post {
            a + EXTRAP_BADGE_GAP
        } else {
            b - EXTRAP_BADGE_GAP - badge_w
        };
        let rect = Rect::new(bx, cy - ROW_H_PX * 0.5, badge_w, ROW_H_PX);
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            label,
            rect,
            TypeToken::Xs.px(),
            resolve(ColorToken::Text2, theme),
        );
    }
}

/// The expand/collapse twirl at the head of a row: a right-pointing triangle
/// that turns down when what it opens (a track's graph editor, a group's
/// members) is open. `kind` is the hit it registers under `id`.
pub(crate) fn paint_twirl(
    ctx: &mut PaintCtx,
    theme: Theme,
    open: bool,
    (x, y): (f32, f32),
    id: NodeId,
    kind: TimelineHitKind,
) {
    let (cx, cy) = (x + TWIRL_W * 0.5, y + ROW_H_PX * 0.5);
    // An isosceles triangle: `BASE` across the flat edge, `TIP` from that edge to
    // the apex. Rotated a quarter turn when the row opens (right → down).
    let (b, t) = (TWIRL_BASE_HALF, TWIRL_TIP);
    let corners = if open {
        [
            (cx - b, cy - t * 0.5),
            (cx + b, cy - t * 0.5),
            (cx, cy + t * 0.5),
        ]
    } else {
        [
            (cx - t * 0.5, cy - b),
            (cx + t * 0.5, cy),
            (cx - t * 0.5, cy + b),
        ]
    };
    let mut p = BezPath::new();
    p.move_to((f64::from(corners[0].0), f64::from(corners[0].1)));
    for (px, py) in &corners[1..] {
        p.line_to((f64::from(*px), f64::from(*py)));
    }
    p.close_path();
    let tok = if open {
        ColorToken::Accent
    } else {
        ColorToken::Text2
    };
    ctx.scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(resolve(tok, theme)),
        None,
        &p,
    );
    let hit = Rect::new(x, y, TWIRL_W, ROW_H_PX);
    ctx.host.store_mut().register(
        id,
        InteractiveState::TimelineSurface {
            parent: ph2d_editor_core::ids::TIMELINE_PANEL,
            kind,
            canvas: hit,
        },
    );
    ctx.host.hit_index_mut().register(id, hit);
}

/// The live box-select rubber band: marching-ants accent outline, no fill (so
/// the diamonds underneath stay readable). Painted over the rows.
pub(crate) fn paint_marquee(ctx: &mut PaintCtx, theme: Theme, rect: Rect) {
    let stroke = Stroke::new(StrokeToken::Thin.px() as f64).with_dashes(0.0, [4.0, 3.0]); // LITERAL-PX-OK: marching-ants dash pattern
    ctx.scene.inner_mut().stroke(
        &stroke,
        Affine::IDENTITY,
        &Brush::Solid(resolve(ColorToken::Accent, theme)),
        None,
        &rect_to_vello(rect),
    );
}

/// Fill a keyframe diamond centred at `(cx, cy)` with half-size `h`.
pub(crate) fn paint_diamond(
    ctx: &mut PaintCtx,
    cx: f32,
    cy: f32,
    h: f32,
    color: ph2d_vector::Color,
) {
    let (cx, cy, h) = (cx as f64, cy as f64, h as f64);
    let mut p = BezPath::new();
    p.move_to((cx, cy - h));
    p.line_to((cx + h, cy));
    p.line_to((cx, cy + h));
    p.line_to((cx - h, cy));
    p.close_path();
    ctx.scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(color),
        None,
        &p,
    );
}

/// A subtle zebra fill for alternate rows.
fn fill_row(ctx: &mut PaintCtx, theme: Theme, rect: Rect, tok: ColorToken) {
    fill_rounded_rect(
        ctx.scene,
        rect,
        ph2d_editor_core::paint::frame_radius(theme, Radius::Xs.px()),
        crate::folha_tinta::cor(tok, theme),
    );
}

#[cfg(test)]
#[path = "tracks_tests.rs"]
mod tests;
