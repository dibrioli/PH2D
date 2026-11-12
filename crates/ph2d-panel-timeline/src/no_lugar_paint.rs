//! ⭐ **As lanes de dentro da peça aberta no lugar** (doc 17 item 10, maquete C2): recuadas logo
//! abaixo da lane da instância; a aberta a cores dentro da moldura laranja, as irmãs esmaecidas e
//! só com a dica. As linhas vêm de `geom::no_lugar_bands` — a mesma conta que empurra as de fora.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text_elided, rect_to_vello, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{LaneView, NoLugarView, StripView, TimelineViewSnapshot};
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TimelineTinta, TypeToken};

use crate::faixas_coluna::{botao_discreto, coluna_da_faixa, fundo, paint_weight};
use crate::graph::TimeView;
use crate::stack_ease_grip::{EASE_IN, EASE_OUT, blend_px, strip_grips};
use crate::stack_lane_paint::{clipped, hit_plan, put_strip_hit};
use crate::state::TimelinePanelState;
use crate::strip_paint::{Fase, paint_strip};
use crate::{geom, ids};

/// Pinta as lanes de dentro e a moldura da janela aberta. No-op sem peça aberta.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    view: TimeView,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
) {
    let Some(v) = crate::no_lugar::vista(snap, state.tab) else {
        return;
    };
    let region = g.rows;
    let bands = geom::no_lugar_bands(snap, state.tab, region.y, state.scroll_y);
    let time_band = Rect::new(
        view.time_x,
        region.y,
        (view.right - view.time_x).max(0.0),
        region.h,
    );
    let laranja = crate::folha_tinta::tinta(TimelineTinta::Mudado, theme);
    for &(k, y, h) in &bands {
        if y + h <= region.y || y >= region.y + region.h {
            continue;
        }
        let row = Rect::new(region.x, y, region.w, h);
        coluna(ctx, theme, g, v, k, row);
        // O véu laranja da janela aberta, por trás das fitas (o mesmo suave do selo «plays»).
        let (a, b) = (view.x(v.janela.0), view.x(v.janela.1));
        if let Some(veu) = clipped(Rect::new(a, row.y, b - a, row.h), time_band) {
            fill_rounded_rect(ctx.scene, veu, 0.0, resolve(ColorToken::WarnSoft, theme));
        }
        fita(ctx, theme, view, snap, v, k, row, time_band);
    }
    // A moldura laranja: a janela da aberta, sobre todas as linhas de dentro, e a faixa fina na de fora.
    let (Some(&(_, y0, _)), Some(&(_, y1, h1))) = (bands.first(), bands.last()) else {
        return;
    };
    let (x0, x1) = (view.x(v.janela.0), view.x(v.janela.1));
    let w = StrokeToken::Default.px();
    ctx.scene.push_clip(&rect_to_vello(time_band));
    stroke_rounded_rect(
        ctx.scene,
        Rect::new(x0, y0, (x1 - x0).max(0.0), y1 + h1 - y0),
        Radius::Xs.px(),
        w,
        laranja,
    );
    fill_rounded_rect(
        ctx.scene,
        Rect::new(x0, y0 - w * 2.0, (x1 - x0).max(0.0), w * 2.0),
        0.0,
        laranja,
    );
    ctx.scene.pop_layer();
}

/// A coluna de uma lane de dentro: recuada, camadas · nome · peso · olho (sem `+` nem menu).
fn coluna(ctx: &mut PaintCtx, theme: Theme, g: &geom::Geom, v: &NoLugarView, k: usize, row: Rect) {
    let lane = &v.lanes[k];
    fundo(ctx, theme, g, row, None);
    let c = coluna_da_faixa(row, g.label_w);
    let recuo = Spacing::Lg.px();
    let dim = lane.muted || lane.weight <= 0.0;
    let cor = crate::folha_tinta::tinta(
        if dim {
            TimelineTinta::TextoValor
        } else {
            TimelineTinta::TextoPropriedade
        },
        theme,
    );
    let icone = Rect::new(c.icone.x + recuo, c.icone.y, c.icone.w, c.icone.h);
    paint_icon(
        ctx.scene,
        IconId::Layers,
        icone,
        cor,
        StrokeToken::Default.px(),
    );
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &lane.name,
        c.nome.0 + recuo,
        c.texto_y,
        TypeToken::Sm.px(),
        (c.nome.1 - recuo).max(0.0),
        cor,
    );
    let band = Rect::new(g.rows.x, g.rows.y, g.label_w, g.rows.h);
    let como = LaneView {
        name: lane.name.clone(),
        muted: lane.muted,
        weight: lane.weight,
        mode: ph2d_timeline::LaneMode::default(),
        strips: Vec::new(),
    };
    paint_weight(
        ctx,
        theme,
        ids::TIMELINE_NO_LUGAR_WEIGHT[k],
        c.peso,
        band,
        &como,
    );
    let olho = if lane.muted {
        (IconId::EyeClosed, TimelineTinta::OlhoMudo)
    } else {
        (IconId::Eye, TimelineTinta::TextoValor)
    };
    botao_discreto(
        ctx,
        theme,
        ids::TIMELINE_NO_LUGAR_MUTE[k],
        (c.mudo, band),
        olho,
    );
}

/// As fitas de uma lane de dentro: cada uma recortada à janela da SUA instância.
#[allow(clippy::too_many_arguments)] // a linha inteira da fita: agrupar esconde a conta
fn fita(
    ctx: &mut PaintCtx,
    theme: Theme,
    view: TimeView,
    snap: &TimelineViewSnapshot,
    v: &NoLugarView,
    k: usize,
    row: Rect,
    time_band: Rect,
) {
    let hit_lane = snap.lanes.len() + k;
    let fora = &snap.lanes[v.lane].strips;
    let pad = geom::peca_margem();
    let mut abertas: Vec<(&StripView, Rect)> = Vec::new();
    for (n, s) in v.lanes[k].strips.iter().enumerate() {
        let Some(inst) = fora.iter().find(|o| o.id == s.instancia) else {
            continue;
        };
        let janela = Rect::new(
            view.x(inst.t_start),
            row.y,
            view.x(inst.t_end) - view.x(inst.t_start),
            row.h,
        );
        let (x0, x1) = (view.x(s.view.t_start), view.x(s.view.t_end));
        let Some(recorte) = clipped(janela, time_band) else {
            continue;
        };
        if x1 - x0 < 1.0 {
            continue;
        }
        let body = Rect::new(x0, row.y + pad, x1 - x0, (row.h - pad * 2.0).max(0.0));
        let tinta = crate::pecas_cartoes::tinta_do_strip(&s.view, snap);
        ctx.scene.push_clip(&rect_to_vello(recorte));
        if s.aberta {
            for fase in [Fase::Base, Fase::Topo] {
                paint_strip(
                    ctx,
                    theme,
                    view,
                    body,
                    &s.view,
                    (false, false),
                    hit_lane,
                    tinta,
                    fase,
                );
            }
        } else {
            // A irmã: corpo e rótulo esmaecidos como os de uma lane muda, sem alça nenhuma.
            paint_strip(
                ctx,
                theme,
                view,
                body,
                &s.view,
                (true, false),
                hit_lane,
                tinta,
                Fase::Base,
            );
            if let Some(p) = crate::pecas_cartoes::peca_do_strip(&s.view, tinta, true, theme) {
                crate::pecas_cartoes::rotulo_do_strip(ctx, body, &s.view, view, p);
            }
        }
        ctx.scene.pop_layer();
        if s.aberta {
            abertas.push((&s.view, body));
        } else if let Some(r) = clipped(body, recorte) {
            // A irmã: a mesma peça, só a dica.
            let id = ids::timeline_no_lugar_irma_id(k as u64, s.view.id.0, n as u64);
            let dica = ph2d_i18n::tr("panel.timeline.no_lugar.same_piece");
            let store = ctx.host.store_mut();
            store.register(id, InteractiveState::Plain);
            if store.tooltip_for(id) != Some(dica) {
                store.set_tooltip(id, dica);
            }
            ctx.host.hit_index_mut().register(id, r);
        }
    }
    // Só a aberta se edita: o mesmo plano de hits das de fora, recortado à janela dela.
    let janela = Rect::new(
        view.x(v.janela.0),
        time_band.y,
        view.x(v.janela.1) - view.x(v.janela.0),
        time_band.h,
    );
    let Some(banda) = clipped(janela, time_band) else {
        return;
    };
    let spans: Vec<(u64, Rect)> = abertas.iter().map(|(s, b)| (s.id.0, *b)).collect();
    let eases: Vec<(u64, u8, Rect, bool)> = abertas
        .iter()
        .filter_map(|(s, body)| {
            let (a, b) = strip_grips(
                *body,
                blend_px(view, s.t_start, s.lead_in),
                blend_px(view, s.t_start, s.blend_in),
                blend_px(view, s.t_start, s.blend_out),
                blend_px(view, s.t_end, s.lead_out),
            )?;
            Some([
                (s.id.0, EASE_IN, a, s.ease_locked_in),
                (s.id.0, EASE_OUT, b, s.ease_locked_out),
            ])
        })
        .flatten()
        .collect();
    for (strip, edge, rect) in hit_plan(&spans, &[], &eases, banda) {
        put_strip_hit(ctx, row, rect, hit_lane, strip, edge);
    }
}
