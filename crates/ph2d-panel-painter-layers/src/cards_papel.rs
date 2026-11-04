//! O cartão Paper em linhas de cartão (fatia H6, `docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos ids, mesmos eventos, as
//! condições de `paint_paper_section`. Só a prévia do padrão continua no pintor dela, capturada.

use crate::cards_linhas::{Item, fila, numero, par_xy, registra_lista};
use crate::paint_texture::Previa;
use crate::paint_watercolor_paper as papel;
use crate::paint_wetpaint::Numero;
use crate::{esmaecer, number_field as nf, paint_brush, state};
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{
    BrushSettings, LinhaDaFerramenta, PAPER_TOOTH_MAX, TEX_OFFSET_MAX, TEX_OFFSET_MIN,
    TEX_SIZE_MAX, TEX_SIZE_MIN, TextureKind,
};

const COR: NodeId = pids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB;
const OFFSET: [NodeId; 2] = [
    pids::PAINTER_WATERCOLOR_PAPER_OFFSET_X,
    pids::PAINTER_WATERCOLOR_PAPER_OFFSET_Y,
];
const TAMANHO: [NodeId; 2] = [
    pids::PAINTER_WATERCOLOR_PAPER_SIZE_X,
    pids::PAINTER_WATERCOLOR_PAPER_SIZE_Y,
];
const UNIDADE: (f32, f32) = (0.0, 1.0);

/// A resposta do `plano_do_corpo`.
pub(crate) fn presente(b: &BrushSettings) -> bool {
    !b.is_inpaint
}

pub(crate) fn titulo() -> &'static str {
    tr("panel.painter_layers.paper.paper")
}

fn tem_padrao(b: &BrushSettings) -> bool {
    TextureKind::from_u8(b.paper_kind) != TextureKind::None
}

/// A prévia do padrão, na posição do quadro anterior.
pub(crate) fn previa(w: f32, brush: &BrushSettings) -> Option<Previa> {
    tem_padrao(brush).then(|| {
        crate::paint_texture::previa_do_padrao(
            w,
            papel::paper_preview_view(brush),
            state::current_brush_paper_image(),
            None,
            ph2d_tool_painter::BRUSH_RAMP_ALPHA,
        )
    })
}

fn do_substrato(v: &BrushSettings) -> [Numero; 2] {
    [
        numero(
            "panel.painter_layers.paper.relief",
            pids::PAINTER_SUBSTRATE_RELIEF,
            v.substrate_relief,
            UNIDADE,
            nf::FINE_STEP,
            2,
        ),
        numero(
            "panel.painter_layers.paper.roughness",
            pids::PAINTER_SUBSTRATE_ROUGHNESS,
            v.substrate_roughness,
            UNIDADE,
            nf::FINE_STEP,
            2,
        ),
    ]
}

/// O Tooth, e os dois números da tela que só o Wet Paint lê. Sem padrão, o Tooth morde o papel de fábrica do Wet Paint.
fn do_fim(l: &BrushSettings, v: &BrushSettings) -> Vec<Numero> {
    let mut n = Vec::new();
    let le_o_dente = tem_padrao(l) || (l.wetpaint && l.le(LinhaDaFerramenta::DepositoMolhado));
    if le_o_dente && l.paper_tooth_offered() {
        n.push(numero(
            "panel.painter_layers.paper.tooth",
            pids::PAINTER_WATERCOLOR_PAPER_DEPTH,
            v.paper_depth.clamp(0.0, PAPER_TOOTH_MAX), // CLAMP-OK: 0 e a const PAPER_TOOTH_MAX (2,0)
            (0.0, PAPER_TOOTH_MAX),
            nf::FINE_STEP,
            2,
        ));
    }
    if l.wetpaint {
        let (min, _, max) = pids::PAINTER_WETPAINT_ABSORBENCY_FAIXA;
        n.push(numero(
            "panel.painter_layers.wetpaint.smooth_edges",
            pids::PAINTER_WETPAINT_SMOOTH_EDGES,
            v.wet_smooth_edges,
            UNIDADE,
            nf::FINE_STEP,
            2,
        ));
        n.push(numero(
            "panel.painter_layers.paper.absorbency",
            pids::PAINTER_WETPAINT_ABSORBENCY,
            v.wet_absorbency,
            (min as f32, max as f32),
            nf::FINE_STEP,
            2,
        ));
    }
    n
}

fn do_angulo(v: &BrushSettings) -> Numero {
    numero(
        "card.painter.shape.angle",
        pids::PAINTER_WATERCOLOR_PAPER_ANGLE,
        f32::from(v.paper_angle),
        (0.0, papel::ANGLE_MAX),
        nf::ANGLE_STEP,
        0,
    )
}

fn dos_eixos(v: &BrushSettings) -> [Numero; 4] {
    let par = |id, valor, faixa, passo| numero("", id, valor, faixa, passo, 2);
    let (o, t) = (
        (TEX_OFFSET_MIN, TEX_OFFSET_MAX),
        (TEX_SIZE_MIN, TEX_SIZE_MAX),
    );
    [
        par(OFFSET[0], v.paper_offset[0], o, nf::FINE_STEP),
        par(OFFSET[1], v.paper_offset[1], o, nf::FINE_STEP),
        par(TAMANHO[0], v.paper_size[0], t, nf::SIZE_STEP),
        par(TAMANHO[1], v.paper_size[1], t, nf::SIZE_STEP),
    ]
}

/// Os números do padrão escolhido (Contrast, Brightness, os de cada padrão).
fn do_padrao(l: &BrushSettings, v: &BrushSettings) -> Vec<Numero> {
    let kind = TextureKind::from_u8(l.paper_kind);
    crate::cards_linhas::do_padrao(
        kind,
        &pids::PAINTER_WATERCOLOR_PAPER_PARAMS,
        &v.paper_params,
    )
}

fn todos(l: &BrushSettings, v: &BrushSettings) -> Vec<Numero> {
    let mut n: Vec<Numero> = do_substrato(v).into();
    n.extend(do_fim(l, v));
    if tem_padrao(l) {
        n.push(do_angulo(v));
        n.extend(dos_eixos(v));
        n.extend(do_padrao(l, v));
    }
    n
}

/// Com o store MUTÁVEL, antes de pintar.
pub(crate) fn prepare(ctx: &mut PaintCtx, b: &BrushSettings) {
    if !presente(b) {
        return;
    }
    papel::paper_color_readback(ctx, *b);
    let store = ctx.host.store_mut();
    if store.picker_target() != Some(COR) {
        let c = papel::encode_rgb3(b.paper_color);
        store.set_widget_color(COR, [c[0], c[1], c[2], u8::MAX]);
    }
    crate::cards_wet::arma_numeros(store, &todos(b, b), &todos(b, &paint_brush::FALLBACK_BRUSH));
    registra_lista(
        store,
        pids::PAINTER_WATERCOLOR_PAPER_KIND,
        usize::from(b.paper_kind),
    );
    let _ = esmaecer::poe_dica(store, &[pids::PAINTER_WATERCOLOR_PAPER_KIND]);
}

/// A lista do Paper abriu no cartão.
pub(crate) fn listas_abertas(ctx: &mut PaintCtx, b: &BrushSettings) {
    let id = pids::PAINTER_WATERCOLOR_PAPER_KIND;
    if crate::cards_linhas::aberta(ctx.host.store(), id)
        && let Some(r) = ctx.host.hit_index_mut().rect_for(id)
    {
        state::set_pending_paper_kind_dd(Some((r, b.paper_kind)));
    }
}

fn em_fila(c: &mut CardCtx<'_>, lista: Vec<Numero>, y: f32) -> f32 {
    let itens: Vec<Item> = lista.into_iter().map(Item::Num).collect();
    fila(c, &itens, y)
}

/// O corpo.
pub(crate) fn corpo(c: &mut CardCtx<'_>, previa: Option<&Previa>, y: f32) -> f32 {
    let b = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    let topo = [
        Item::cor(c.store, COR, "panel.painter_layers.paper.color"),
        Item::Lista {
            id: pids::PAINTER_WATERCOLOR_PAPER_KIND,
            chave: "panel.painter_layers.paper.paper",
            texto: tr(TextureKind::from_u8(b.paper_kind).name_key()),
        },
    ];
    let y = fila(c, &topo, y);
    let y = em_fila(c, do_substrato(&b).into(), y);
    if !tem_padrao(&b) {
        return em_fila(c, do_fim(&b, &b), y);
    }
    let y = crate::cards_rampa::previa(c, previa, y);
    let y = par_xy(c, "panel.painter_layers.paper.offset", OFFSET, y);
    let y = par_xy(c, "panel.painter_layers.paper.size", TAMANHO, y);
    let mut resto = vec![do_angulo(&b)];
    resto.extend(do_fim(&b, &b));
    resto.extend(do_padrao(&b, &b));
    em_fila(c, resto, y)
}
