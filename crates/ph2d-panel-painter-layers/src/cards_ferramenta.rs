//! As linhas de Line, Clone, Inpaint e dos verbos do Sculpt, em linhas de cartão (fatia H8,
//! `docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos ids, mesmos eventos. Qual verbo pinta qual número é a tabela de
//! `paint_sculpt`, e o que a Line oferece é a de `line_barras`.

use crate::cards_linhas::{Item, Pista, aberta, botoes, fila, registra_lista};
use crate::{line_barras, paint_line, state};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget};
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{BrushSettings, LineKind, LinhaDaFerramenta};

const INPAINT: [Pista; 3] = [
    Pista {
        chave: "panel.painter_layers.inpaint.patch_size",
        ids: [
            pids::PAINTER_INPAINT_PATCH_SLIDER,
            pids::PAINTER_INPAINT_PATCH_CHIP,
        ],
        le: |b| b.inpaint_patch,
        casas: 0,
    },
    Pista {
        chave: "panel.painter_layers.inpaint.quality",
        ids: [
            pids::PAINTER_INPAINT_QUALITY_SLIDER,
            pids::PAINTER_INPAINT_QUALITY_CHIP,
        ],
        le: |b| b.inpaint_quality,
        casas: 0,
    },
    Pista {
        chave: "panel.painter_layers.inpaint.search",
        ids: [
            pids::PAINTER_INPAINT_SEARCH_SLIDER,
            pids::PAINTER_INPAINT_SEARCH_CHIP,
        ],
        le: |b| b.inpaint_search,
        casas: 0,
    },
];

const RAIO: Pista = Pista {
    chave: "card.painter.sculpt.radius",
    ids: [
        pids::PAINTER_SCULPT_RADIUS_SLIDER,
        pids::PAINTER_SCULPT_RADIUS_CHIP,
    ],
    le: |b| b.sculpt_radius,
    casas: 0,
};
const PLANO: Pista = Pista {
    chave: "panel.painter_layers.sculpt.offset",
    ids: [
        pids::PAINTER_SCULPT_OFFSET_SLIDER,
        pids::PAINTER_SCULPT_OFFSET_CHIP,
    ],
    le: |b| b.sculpt_offset,
    casas: 2,
};
const FOLGA: Pista = Pista {
    chave: "panel.painter_layers.sculpt.depth",
    ids: [
        pids::PAINTER_SCULPT_DEPTH_SLIDER,
        pids::PAINTER_SCULPT_DEPTH_CHIP,
    ],
    le: |b| b.sculpt_depth,
    casas: 2,
};
const GUME: Pista = Pista {
    chave: "card.painter.sculpt.angle",
    ids: [
        pids::PAINTER_SCULPT_ANGLE_SLIDER,
        pids::PAINTER_SCULPT_ANGLE_CHIP,
    ],
    le: |b| b.sculpt_angle,
    casas: 0,
};
const MACIO: Pista = Pista {
    chave: "card.painter.sculpt.smooth",
    ids: [
        pids::PAINTER_SCULPT_SMOOTH_SLIDER,
        pids::PAINTER_SCULPT_SMOOTH_CHIP,
    ],
    le: |b| b.sculpt_smooth,
    casas: 0,
};

/// A família de botões do verbo: `paint_sculpt::paint_sculpt_rows`.
fn do_verbo(b: &BrushSettings) -> Vec<Pista> {
    match b.sculpt_knob_family {
        1 if b.sculpt_is_chisel => vec![PLANO, GUME],
        1 => vec![PLANO],
        2 if b.sculpt_is_inflate => vec![FOLGA, MACIO],
        2 => vec![FOLGA],
        _ => vec![RAIO],
    }
}

fn esculpe(b: &BrushSettings) -> bool {
    b.impasto && b.is_sculpt
}

/// Com o store e o barramento, antes de pintar.
pub(crate) fn prepare(ctx: &mut PaintCtx, b: &BrushSettings) {
    let store = ctx.host.store_mut();
    registra_lista(store, pids::PAINTER_LINE_TYPE, usize::from(b.line_kind));
    let kind = LineKind::from_wire(b.line_kind);
    for barra in line_barras::barras_de(kind) {
        let casas = if barra.inteiro {
            0
        } else {
            crate::cards_topo::CASAS_DO_PINCEL
        };
        crate::cards_linhas::arma_barra(
            store,
            [barra.slider, barra.chip],
            (barra.pista)(*b),
            (barra.pista)(crate::paint_brush::FALLBACK_BRUSH),
            casas,
        );
    }
    let _ = crate::esmaecer::poe_dica(store, &[pids::PAINTER_LINE_SOLID]);
    if b.is_clone {
        crate::paint::register_button(store, pids::PAINTER_BRUSH_CLONE_SET_SOURCE);
    }
    if b.is_inpaint {
        INPAINT.iter().for_each(|p| p.arma(store, b));
    }
    if esculpe(b) {
        prepara_escultura(store, b);
    }
}

fn prepara_escultura(store: &mut WidgetStore, b: &BrushSettings) {
    do_verbo(b).iter().for_each(|p| p.arma(store, b));
    for id in [
        pids::PAINTER_SCULPT_FILTER,
        pids::PAINTER_SCULPT_FILTER_STROKE,
    ] {
        crate::paint::register_button(store, id);
    }
}

/// Onde a caixa aberta ficou é o que a lista das opções precisa.
pub(crate) fn listas_abertas(ctx: &mut PaintCtx, b: &BrushSettings) {
    let id = pids::PAINTER_LINE_TYPE;
    if aberta(ctx.host.store(), id)
        && let Some(r) = ctx.host.hit_index_mut().rect_for(id)
    {
        state::set_pending_line_type_dd(Some((r, b.line_kind)));
    }
}

/// Solid, o tipo, os números que ESTE tipo lê (`None` e `Speed` não têm nenhum) e a caixa do Composite, irmã curta.
pub(crate) fn linha(
    c: &mut CardCtx<'_>,
    b: &BrushSettings,
    sobra: Option<(&'static str, &'static [NodeId; 2])>,
    composto: bool,
    y: f32,
) -> f32 {
    let kind = LineKind::from_wire(b.line_kind);
    let mut itens = Vec::new();
    if let Some((chave, ids)) = sobra {
        itens.push(Item::Barra { chave, ids: *ids });
    }
    if b.is_clone {
        itens.push(Item::Caixa {
            id: pids::PAINTER_BRUSH_CLONE_ALIGNED,
            chave: "panel.painter_layers.clone.aligned",
            on: b.clone_aligned,
        });
    }
    if b.le(LinhaDaFerramenta::Solido) {
        itens.push(Item::Caixa {
            id: pids::PAINTER_LINE_SOLID,
            chave: "panel.painter_layers.line.solid",
            on: b.style_solid,
        });
    }
    itens.push(Item::Lista {
        id: pids::PAINTER_LINE_TYPE,
        chave: "card.painter.line.type",
        texto: paint_line::kind_name(kind),
    });
    itens.extend(
        line_barras::barras_de(kind)
            .iter()
            .map(|barra| Item::Barra {
                chave: barra.chave,
                ids: [barra.slider, barra.chip],
            }),
    );
    if let Some((id, chave, le)) = paint_line::checkbox_of(kind) {
        itens.push(Item::Caixa {
            id,
            chave,
            on: le(*b),
        });
    }
    if composto {
        itens.push(Item::Caixa {
            id: pids::PAINTER_BRUSH_COMPOSITE_ENABLE,
            chave: "panel.painter_layers.composite.composite_brush",
            on: b.composite_enabled,
        });
    }
    fila(c, &itens, y)
}

/// O botão arma a amostragem e fica azul até o clique na tela; a dica só antes da 1.ª amostra. Aligned vive na fila do traço, irmã curta.
pub(crate) fn clone(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let texto = if b.clone_sample_armed {
        tr("panel.painter_layers.clone.sample_hint")
    } else if b.clone_has_source {
        tr("panel.painter_layers.clone.set_source_resample")
    } else {
        tr("panel.painter_layers.clone.set_source")
    };
    let y = c.row(
        &[Cell::new(Widget::Button {
            id: pids::PAINTER_BRUSH_CLONE_SET_SOURCE,
            text: texto,
            primary: b.clone_sample_armed,
        })],
        y,
    );
    if b.clone_has_source {
        y
    } else {
        c.hint(tr("panel.painter_layers.clone.hint"), y)
    }
}

pub(crate) fn inpaint(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let y = c.hint(tr("panel.painter_layers.inpaint.hint"), y);
    let itens: Vec<Item> = INPAINT.iter().map(Pista::item).collect();
    fila(c, &itens, y)
}

/// Os números que o verbo usa, e o filtro onde o verbo remodela.
pub(crate) fn escultura(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let mut itens: Vec<Item> = do_verbo(b).iter().map(Pista::item).collect();
    if b.sculpt_knob_family == 1 && b.sculpt_is_chisel {
        itens.push(Item::Caixa {
            id: pids::PAINTER_SCULPT_RAKE,
            chave: "panel.painter_layers.sculpt.rake",
            on: b.sculpt_rake,
        });
    }
    let y = fila(c, &itens, y);
    if !b.sculpt_filters {
        return y;
    }
    let mut filtros = vec![(
        pids::PAINTER_SCULPT_FILTER,
        tr("panel.painter_layers.sculpt.filter_layer"),
    )];
    if b.sculpt_can_filter_stroke {
        filtros.push((
            pids::PAINTER_SCULPT_FILTER_STROKE,
            tr("panel.painter_layers.sculpt.filter_stroke"),
        ));
    }
    botoes(c, &filtros, y)
}
