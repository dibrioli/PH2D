//! O cartão Watercolor em linhas de cartão (fatia H4, `docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos ids, mesmos eventos;
//! as caixas de `card_frame` viram grupos.

use crate::cards_linhas::{aberta, botoes, grupo, grupo_inicial, numero, registra_lista};
use crate::cards_wet::{arma_numeros, campo_e_caixa, numeros};
use crate::paint_texture::Previa;
use crate::paint_wetpaint::Numero;
use crate::{
    esmaecer, number_field as nf, paint_brush, paint_watercolor as pw,
    paint_watercolor_flow as flow, state,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, Widget};
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{
    BrushSettings, FLOW_SIZE_MAX, FLOW_SIZE_MIN, LinhaDaFerramenta, PaintMedia,
};

const UNIDADE: (f32, f32) = (0.0, 1.0);
const PASSO_DO_SEGUNDO: f64 = 1.0; // LITERAL-PX-OK: whole seconds, the step of Drying Time
const FLOW: NodeId = pids::PAINTER_WATERCOLOR_FLOW_KIND;

/// A resposta do `plano_do_corpo`: o meio, e a Borracha apaga como no Digital.
pub(crate) fn presente(b: &BrushSettings) -> bool {
    !b.is_inpaint
        && PaintMedia::from_u8(b.media) == PaintMedia::Watercolor
        && b.le(LinhaDaFerramenta::Aguada)
}

pub(crate) fn titulo() -> &'static str {
    tr("panel.painter_layers.watercolor.watercolor")
}

/// A faixa da prévia do Flow: ela blita uma imagem e pede o `PaintCtx`.
pub(crate) fn previa(w: f32, brush: &BrushSettings) -> Previa {
    flow::previa_do_flow(w, brush)
}

fn da_lavagem(b: &BrushSettings) -> [Numero; 6] {
    let (fino, largo) = (nf::FINE_STEP, nf::SIZE_STEP);
    [
        numero(
            "panel.painter_layers.watercolor.body",
            pids::PAINTER_WATERCOLOR_FILL,
            b.fill,
            UNIDADE,
            fino,
            2,
        ),
        numero(
            "panel.painter_layers.watercolor.concentration",
            pids::PAINTER_WATERCOLOR_DEPTH,
            b.depth,
            (pw::DEPTH_MIN, pw::DEPTH_MAX),
            fino,
            2,
        ),
        numero(
            "panel.painter_layers.watercolor.opacity",
            pids::PAINTER_WATERCOLOR_OPACITY,
            b.opacity,
            UNIDADE,
            fino,
            2,
        ),
        numero(
            "panel.painter_layers.watercolor.edge_darkening",
            pids::PAINTER_WATERCOLOR_EDGE,
            b.edge_gain,
            (0.0, pw::EDGE_MAX),
            fino,
            2,
        ),
        numero(
            "card.painter.watercolor.bleed",
            pids::PAINTER_WATERCOLOR_SPREAD,
            b.edge_spread,
            (pw::SPREAD_MIN, pw::SPREAD_MAX),
            largo,
            1,
        ),
        numero(
            "card.painter.watercolor.ragged_edge",
            pids::PAINTER_WATERCOLOR_WARP,
            b.warp,
            (0.0, pw::WARP_MAX),
            largo,
            1,
        ),
    ]
}

fn do_flow(b: &BrushSettings) -> [Numero; 2] {
    [
        numero(
            "panel.painter_layers.watercolor.flow_size",
            pids::PAINTER_WATERCOLOR_FLOW_SIZE,
            b.flow_size,
            (FLOW_SIZE_MIN, FLOW_SIZE_MAX),
            nf::FINE_STEP,
            2,
        ),
        numero(
            "card.painter.watercolor.flow_angle",
            pids::PAINTER_WATERCOLOR_FLOW_ANGLE,
            f32::from(b.flow_angle),
            (0.0, flow::ANGLE_MAX),
            nf::ANGLE_STEP,
            0,
        ),
    ]
}

fn da_borda(b: &BrushSettings) -> Numero {
    numero(
        "panel.painter_layers.watercolor.paper_edge",
        pids::PAINTER_WATERCOLOR_PAPER_EDGE,
        b.paper_edge,
        UNIDADE,
        nf::FINE_STEP,
        2,
    )
}

fn do_pincel(b: &BrushSettings) -> [Numero; 3] {
    let f = nf::FINE_STEP;
    [
        numero(
            "panel.painter_layers.watercolor.charge",
            pids::PAINTER_WATERCOLOR_CHARGE,
            b.wet_charge,
            UNIDADE,
            f,
            2,
        ),
        numero(
            "panel.painter_layers.watercolor.dilution",
            pids::PAINTER_WATERCOLOR_DILUTION,
            b.wet_dilution,
            UNIDADE,
            f,
            2,
        ),
        numero(
            "panel.painter_layers.watercolor.pull",
            pids::PAINTER_WATERCOLOR_PULL,
            b.wet_pull,
            UNIDADE,
            f,
            2,
        ),
    ]
}

fn da_agua(b: &BrushSettings) -> [Numero; 2] {
    let f = nf::FINE_STEP;
    [
        numero(
            "panel.painter_layers.watercolor.rewet",
            pids::PAINTER_WATERCOLOR_WET,
            b.wet_rewet,
            UNIDADE,
            f,
            2,
        ),
        numero(
            "panel.painter_layers.watercolor.smudge",
            pids::PAINTER_WATERCOLOR_SMUDGE,
            b.wet_smudge,
            UNIDADE,
            f,
            2,
        ),
    ]
}

fn da_secagem(b: &BrushSettings) -> [Numero; 2] {
    [
        numero(
            "card.painter.watercolor.drying_time",
            pids::PAINTER_WATERCOLOR_DRY_TIME,
            b.dry_time_s,
            (pw::DRY_TIME_MIN, pw::DRY_TIME_MAX),
            PASSO_DO_SEGUNDO,
            0,
        ),
        numero(
            "panel.painter_layers.watercolor.preview",
            pids::PAINTER_WATERCOLOR_WET_PREVIEW,
            b.wet_preview,
            UNIDADE,
            nf::FINE_STEP,
            2,
        ),
    ]
}

fn todos(b: &BrushSettings) -> Vec<Numero> {
    let mut n: Vec<Numero> = da_lavagem(b).into();
    n.extend(do_flow(b));
    n.push(da_borda(b));
    n.extend(do_pincel(b));
    n.extend(da_agua(b));
    n.extend(da_secagem(b));
    n
}

/// Com o store MUTÁVEL, antes de pintar.
pub(crate) fn prepare(store: &mut WidgetStore, b: &BrushSettings) {
    if !presente(b) {
        return;
    }
    arma_numeros(store, &todos(b), &todos(&paint_brush::FALLBACK_BRUSH));
    let _ = esmaecer::poe_dica(store, &[pids::PAINTER_WATERCOLOR_SMOOTH_EDGES]);
    registra_lista(store, FLOW, usize::from(b.flow_kind));
}

/// Onde a caixa aberta ficou é o que a lista das opções do Flow precisa.
pub(crate) fn listas_abertas(ctx: &mut PaintCtx, b: &BrushSettings) {
    if aberta(ctx.host.store(), FLOW)
        && let Some(r) = ctx.host.hit_index_mut().rect_for(FLOW)
    {
        state::set_pending_flow_kind_dd(Some((r, b.flow_kind)));
    }
}

/// O corpo, pela ordem do painel de antes: cada caixa é um grupo, e o que cabe aos pares divide a linha.
pub(crate) fn corpo(c: &mut CardCtx<'_>, previa: &Previa, y: f32) -> f32 {
    let b = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    if !b.watercolor {
        return y;
    }
    let y = grupo_inicial(c, "panel.painter_layers.watercolor.wash", y);
    let y = numeros(c, &da_lavagem(&b), y);
    // Sem marca de animação: o Flow nunca a teve (`each_control_keeps_its_animation_mark`).
    let y = c.full(
        Some(tr("panel.painter_layers.watercolor.flow")),
        Widget::Dropdown {
            id: FLOW,
            text: flow::rotulo_do_flow(b.flow_kind),
            options: &[],
        },
        y,
    );
    let y = crate::cards_rampa::previa(c, Some(previa), y);
    let y = numeros(c, &do_flow(&b), y);
    let suave = (
        pids::PAINTER_WATERCOLOR_SMOOTH_EDGES,
        "panel.painter_layers.watercolor.smooth_edges",
        b.smooth_edges,
    );
    let y = campo_e_caixa(c, &da_borda(&b), &suave, y);
    let y = grupo(c, "panel.painter_layers.watercolor.brush", y);
    let y = numeros(c, &do_pincel(&b), y);
    let y = grupo(c, "panel.painter_layers.watercolor.water", y);
    let y = numeros(c, &da_agua(&b), y);
    let y = grupo(c, "panel.painter_layers.watercolor.wetness", y);
    let y = numeros(c, &da_secagem(&b), y);
    botoes(
        c,
        &[
            (
                pids::PAINTER_WATERCOLOR_DRY_NOW,
                tr("panel.painter_layers.watercolor.dry"),
            ),
            (
                pids::PAINTER_WATERCOLOR_WET_NOW,
                tr("panel.painter_layers.watercolor.wet"),
            ),
        ],
        y,
    )
}
