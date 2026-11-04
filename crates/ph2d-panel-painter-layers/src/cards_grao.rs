//! O cartão Grain em linhas de cartão (fatia H6, `docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos ids, mesmos eventos, as
//! condições de `paint_texture_section`. Capturados: a prévia, o editor da rampa e os dois títulos que dobram dentro do
//! cartão (Grain Behavior e Grain Colors, este com o Reset dele). A Texture layer continua no pintor de antes.

use crate::cards_captura::{Captura, chave_da_peca, x_de, y_de};
use crate::cards_linhas::{Item, fila, filete, grupo, numero, par_xy};
use crate::paint_ramp_widget::paint_ramp_body;
use crate::paint_texture_ramp::{grain_ramp_ids, vista_do_pincel};
use crate::paint_wetpaint::Numero;
use crate::{cards_shape, esmaecer, number_field as nf, paint_brush, state};
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{
    BrushSettings, CartaoDaPonta, LinhaDaFerramenta, TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX,
    TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN, TextureKind, TextureMapping,
};

const SECAO: NodeId = pids::PAINTER_BRUSH_TEXTURE_SECTION;
const COMPORTAMENTO: NodeId = pids::PAINTER_BRUSH_GRAIN_BEHAVIOR_SECTION;
const OFFSET: [NodeId; 2] = [
    pids::PAINTER_BRUSH_TEXTURE_OFFSET_X,
    pids::PAINTER_BRUSH_TEXTURE_OFFSET_Y,
];
const TAMANHO: [NodeId; 2] = [
    pids::PAINTER_BRUSH_TEXTURE_SIZE_X,
    pids::PAINTER_BRUSH_TEXTURE_SIZE_Y,
];
const ST_TAMANHO: [NodeId; 2] = [
    pids::PAINTER_BRUSH_STENCIL_SIZE_X,
    pids::PAINTER_BRUSH_STENCIL_SIZE_Y,
];
const ST_OFFSET: [NodeId; 2] = [
    pids::PAINTER_BRUSH_STENCIL_OFFSET_X,
    pids::PAINTER_BRUSH_STENCIL_OFFSET_Y,
];

/// A resposta do `plano_do_corpo`.
pub(crate) fn presente(b: &BrushSettings) -> bool {
    !b.is_inpaint
}

pub(crate) fn titulo() -> &'static str {
    tr("panel.painter_layers.grain.grain")
}

/// Na aguada, o slot do Grain é o mapa de granulação.
fn na_aguada(b: &BrushSettings) -> bool {
    b.watercolor && b.le(LinhaDaFerramenta::Aguada)
}

/// «Same as Paper» na aguada: o papel é o grão, e não sobra mais nada.
fn so_o_papel(b: &BrushSettings) -> bool {
    na_aguada(b) && b.grao_do_papel()
}

/// «Same as Paper» nos outros meios: fica a Depth, a lei do grão e as cores.
fn e_o_papel(b: &BrushSettings) -> bool {
    b.oferece_grain_same_as_paper() && b.grain_e_o_papel()
}

fn kind(b: &BrushSettings) -> TextureKind {
    TextureKind::from_u8(b.texture_kind)
}

fn proprio(b: &BrushSettings) -> bool {
    !so_o_papel(b) && !e_o_papel(b) && kind(b) != TextureKind::None
}

/// Há lei do grão e cores por pintar.
fn com_o_fim(b: &BrushSettings) -> bool {
    e_o_papel(b) || proprio(b)
}

fn stencil(b: &BrushSettings) -> bool {
    TextureMapping::from_u8(b.texture_mapping).is_stencil()
}

/// O que ainda pede o `PaintCtx`.
pub(crate) struct Pecas {
    previa: Option<Captura>,
    comportamento: Option<Captura>,
    rampa_titulo: Option<Captura>,
    rampa: Option<Captura>,
}

/// Corre as peças, uma vez por quadro, na posição do quadro anterior.
pub(crate) fn pecas(
    ctx: &mut PaintCtx,
    ink: CardInk,
    (x, w): (f32, f32),
    brush: BrushSettings,
) -> Pecas {
    let topo = ctx.slot.y;
    let tema = ink.theme;
    let ids = grain_ramp_ids();
    let quer_rampa = com_o_fim(&brush)
        && brush.texture_ramp_enabled
        && !ctx.host.store().is_collapsed(ids.section);
    let mut peca = |n: u64, quer: bool, pinta: &mut dyn FnMut(&mut PaintCtx, f32, f32) -> f32| {
        quer.then(|| {
            let chave = chave_da_peca(SECAO, n);
            let x = x_de(chave, x);
            Captura::da_peca(ctx, chave, SECAO, ink, (x, y_de(chave, topo)), |ctx, y| {
                pinta(ctx, x, y)
            })
        })
    };
    let fim = com_o_fim(&brush);
    let previa = peca(0, proprio(&brush), &mut |ctx, x, y| {
        crate::paint_texture::paint_texture_preview(
            ctx,
            tema,
            x,
            w,
            y,
            brush,
            state::current_brush_texture_image(),
        )
    });
    let comportamento = peca(1, fim, &mut |ctx, x, y| {
        let (y, dobra) = crate::paint_brush_top::paint_sub_section(
            ctx,
            tema,
            x,
            w,
            y,
            tr("panel.painter_layers.grain.behavior"),
            COMPORTAMENTO,
        );
        dobra.map_or(y, |d| crate::paint_brush_top::end_fold(ctx, d, y))
    });
    let rampa_titulo = peca(2, fim, &mut |ctx, x, y| {
        let (y, dobra) = crate::paint_brush_top::paint_collapsible_section(
            ctx,
            tema,
            x,
            w,
            y,
            tr("panel.painter_layers.grain.grain_colors"),
            ids.section,
            ids.reset,
        );
        dobra.map_or(y, |d| crate::paint_brush_top::end_fold(ctx, d, y))
    });
    let rampa = peca(3, quer_rampa, &mut |ctx, x, y| {
        let vista = vista_do_pincel(&brush, None);
        let h = crate::paint_ramp_widget::ramp_body_height(w, vista.bw_button);
        let area = Rect::new(x, y, w, h);
        esmaecer::talvez_esmaecido(ctx, tema, area, &[ids.enable], |ctx| {
            paint_ramp_body(ctx, tema, x, w, y, &ids, vista)
        })
    });
    Pecas {
        previa,
        comportamento,
        rampa_titulo,
        rampa,
    }
}

fn da_aguada(v: &BrushSettings) -> Numero {
    numero(
        "panel.painter_layers.paper.amount",
        pids::PAINTER_WATERCOLOR_GRANULATION,
        v.granulation,
        (0.0, 1.0),
        nf::FINE_STEP,
        2,
    )
}

fn profundidade(v: &BrushSettings) -> Numero {
    numero(
        "panel.painter_layers.grain.depth",
        pids::PAINTER_BRUSH_GRAIN_DEPTH,
        v.grain_depth.clamp(0.0, 1.0),
        (0.0, 1.0),
        nf::FINE_STEP,
        2,
    )
}

fn do_angulo(v: &BrushSettings) -> Numero {
    numero(
        "card.painter.shape.angle",
        pids::PAINTER_BRUSH_TEXTURE_ANGLE,
        f32::from(v.texture_angle_deg),
        (0.0, f32::from(TEX_ANGLE_MAX_DEG)),
        nf::ANGLE_STEP,
        0,
    )
}

fn do_stencil(v: &BrushSettings) -> Numero {
    numero(
        "card.painter.stencil.rotation",
        pids::PAINTER_BRUSH_STENCIL_ANGLE,
        f32::from(v.stencil_angle_deg),
        (0.0, f32::from(TEX_ANGLE_MAX_DEG)),
        nf::ANGLE_STEP,
        0,
    )
}

fn eixos(v: [f32; 2], ids: [NodeId; 2], faixa: (f32, f32), passo: f64) -> [Numero; 2] {
    [
        numero("", ids[0], v[0], faixa, passo, 2),
        numero("", ids[1], v[1], faixa, passo, 2),
    ]
}

const FAIXA_OFFSET: (f32, f32) = (TEX_OFFSET_MIN, TEX_OFFSET_MAX);
const FAIXA_TAMANHO: (f32, f32) = (TEX_SIZE_MIN, TEX_SIZE_MAX);

fn numeros_de(itens: Vec<Item>) -> impl Iterator<Item = Numero> {
    itens.into_iter().filter_map(|i| match i {
        Item::Num(n) => Some(n),
        _ => None,
    })
}

/// Os números à vista neste estado: só serve quem os arma. A ordem é a da tabela, não a da pintura.
fn todos(l: &BrushSettings, v: &BrushSettings) -> Vec<Numero> {
    let mut n = Vec::new();
    if na_aguada(l) {
        n.push(da_aguada(v));
    }
    if com_o_fim(l) {
        n.push(profundidade(v));
        n.extend(numeros_de(cards_shape::da_ponta(CartaoDaPonta::Grao, l, v)));
    }
    if proprio(l) {
        if stencil(l) {
            n.extend(eixos(
                v.stencil_size,
                ST_TAMANHO,
                FAIXA_TAMANHO,
                nf::SIZE_STEP,
            ));
            n.extend(eixos(
                v.stencil_offset,
                ST_OFFSET,
                FAIXA_OFFSET,
                nf::FINE_STEP,
            ));
            n.push(do_stencil(v));
        }
        n.push(do_angulo(v));
        n.extend(eixos(v.texture_offset, OFFSET, FAIXA_OFFSET, nf::FINE_STEP));
        n.extend(eixos(v.texture_size, TAMANHO, FAIXA_TAMANHO, nf::SIZE_STEP));
        n.extend(crate::cards_linhas::do_padrao(
            kind(l),
            &pids::PAINTER_BRUSH_TEXTURE_PARAMS,
            &v.texture_params,
        ));
    }
    n
}

/// Com o store MUTÁVEL, antes de pintar.
pub(crate) fn prepare(ctx: &mut PaintCtx, b: &BrushSettings) {
    if !presente(b) {
        return;
    }
    let store = ctx.host.store_mut();
    crate::cards_wet::arma_numeros(store, &todos(b, b), &todos(b, &paint_brush::FALLBACK_BRUSH));
    for id in [
        pids::PAINTER_WATERCOLOR_GRAN_SAME,
        pids::PAINTER_BRUSH_GRAIN_SAME_AS_PAPER,
        pids::PAINTER_BRUSH_TEXTURE_KIND,
        pids::PAINTER_BRUSH_TEXTURE_MAPPING,
        pids::PAINTER_BRUSH_TEXTURE_RAKE,
        pids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE,
    ] {
        let _ = esmaecer::poe_dica(store, &[id]);
    }
}

/// Onde a caixa aberta ficou é o que a lista das opções precisa.
pub(crate) fn listas_abertas(ctx: &mut PaintCtx, b: &BrushSettings) {
    let aberto = |ctx: &mut PaintCtx, id| {
        if crate::cards_linhas::aberta(ctx.host.store(), id) {
            ctx.host.hit_index_mut().rect_for(id)
        } else {
            None
        }
    };
    if let Some(r) = aberto(ctx, pids::PAINTER_BRUSH_TEXTURE_KIND) {
        state::set_pending_brush_texture_kind_dd(Some((r, b.texture_kind)));
    }
    if let Some(r) = aberto(ctx, pids::PAINTER_BRUSH_TEXTURE_MAPPING) {
        state::set_pending_brush_texture_mapping_dd(Some((r, b.texture_mapping)));
    }
}

/// O grupo do carimbo: a moldura dele na tela, independente da repetição do padrão.
fn do_carimbo(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let y = grupo(c, "panel.painter_layers.stencil.stencil", y);
    let y = par_xy(c, "panel.painter_layers.stencil.size", ST_TAMANHO, y);
    let y = par_xy(c, "panel.painter_layers.stencil.offset", ST_OFFSET, y);
    let y = fila(c, &[Item::Num(do_stencil(b))], y);
    filete(c, y)
}

/// A lei do grão e as cores: dois títulos que dobram dentro do cartão.
fn fim(c: &mut CardCtx<'_>, b: &BrushSettings, p: &Pecas, y: f32) -> f32 {
    let mut y = cards_shape::repete(c, p.comportamento.as_ref(), y);
    if !c.store.is_collapsed(COMPORTAMENTO) {
        y = fila(c, &cards_shape::da_ponta(CartaoDaPonta::Grao, b, b), y);
    }
    let ids = grain_ramp_ids();
    y = cards_shape::repete(c, p.rampa_titulo.as_ref(), y);
    if c.store.is_collapsed(ids.section) {
        return y;
    }
    y = fila(
        c,
        &[Item::Caixa {
            id: ids.enable,
            chave: "panel.painter_layers.ramp.use_color_ramp",
            on: b.texture_ramp_enabled,
        }],
        y,
    );
    cards_shape::repete(c, p.rampa.as_ref(), y)
}

/// O corpo.
pub(crate) fn corpo(c: &mut CardCtx<'_>, p: &Pecas, y: f32) -> f32 {
    let b = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    let mut y = y;
    if na_aguada(&b) {
        let linha = [
            Item::Caixa {
                id: pids::PAINTER_WATERCOLOR_GRAN_SAME,
                chave: "panel.painter_layers.paper.same_as_paper",
                on: b.granulation_use_paper,
            },
            Item::Num(da_aguada(&b)),
        ];
        y = fila(c, &linha, y);
        if so_o_papel(&b) {
            return y;
        }
    }
    let mesmo = Item::Caixa {
        id: pids::PAINTER_BRUSH_GRAIN_SAME_AS_PAPER,
        chave: "panel.painter_layers.paper.same_as_paper",
        on: b.grain_same_as_paper,
    };
    if e_o_papel(&b) {
        y = fila(c, &[mesmo, Item::Num(profundidade(&b))], y);
        return fim(c, &b, p, y);
    }
    let mut topo = Vec::with_capacity(2);
    if b.oferece_grain_same_as_paper() {
        topo.push(mesmo);
    }
    topo.push(Item::Lista {
        id: pids::PAINTER_BRUSH_TEXTURE_KIND,
        chave: "panel.painter_layers.grain.grain",
        texto: tr(kind(&b).name_key()),
    });
    y = fila(c, &topo, y);
    if !proprio(&b) {
        return y;
    }
    y = cards_shape::repete(c, p.previa.as_ref(), y);
    let mapping = TextureMapping::from_u8(b.texture_mapping);
    let mut linha = vec![Item::Lista {
        id: pids::PAINTER_BRUSH_TEXTURE_MAPPING,
        chave: "panel.painter_layers.grain.mapping",
        texto: tr(mapping.name_key()),
    }];
    if mapping.uses_dab_rotation() && !b.watercolor_active {
        linha.push(Item::Caixa {
            id: pids::PAINTER_BRUSH_TEXTURE_RAKE,
            chave: "panel.painter_layers.grain.rake",
            on: b.texture_rake,
        });
    }
    y = fila(c, &linha, y);
    if stencil(&b) {
        y = do_carimbo(c, &b, y);
    }
    y = par_xy(c, "panel.painter_layers.grain.offset", OFFSET, y);
    y = par_xy(c, "panel.painter_layers.grain.size", TAMANHO, y);
    let mut resto = vec![Item::Num(do_angulo(&b)), Item::Num(profundidade(&b))];
    resto.extend(
        crate::cards_linhas::do_padrao(
            kind(&b),
            &pids::PAINTER_BRUSH_TEXTURE_PARAMS,
            &b.texture_params,
        )
        .into_iter()
        .map(Item::Num),
    );
    y = fila(c, &resto, y);
    fim(c, &b, p, y)
}
