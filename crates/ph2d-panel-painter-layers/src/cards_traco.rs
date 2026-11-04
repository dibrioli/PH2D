//! O cartão Stroke em linhas de cartão (fatia H7, `docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos ids, mesmos eventos, as
//! condições de `paint_stroke_section`. Cada barra vira a caixa ligada a ela: é a barra que continua a avisar o painel, e o
//! chip mostra a mesma lei da pista. Spacing e Continuous ficam na mesma linha (E10). Capturado: só o título do Jitter.

use crate::cards_captura::{Captura, chave_da_peca, x_de, y_de};
use crate::cards_linhas::{Item, aberta, botoes, escolha, fila, grupo};
use crate::paint_stroke::{self, jitter_linhas};
use crate::{cards_topo, esmaecer, number_field, paint_brush, state};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, Seg};
use ph2d_editor_core::widget::card_field::CardFieldSpec;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{
    BRUSH_AIRBRUSH_RATE_MAX_S, BRUSH_AIRBRUSH_RATE_MIN_S, BrushSettings, LinhaDaFerramenta,
    StrokeMethod,
};

const SECAO: NodeId = pids::PAINTER_BRUSH_STROKE_SECTION;
const JITTER: NodeId = pids::PAINTER_BRUSH_JITTER_SECTION;

/// Uma barra com o chip ligado, e a posição dela na pista.
#[derive(Clone, Copy)]
struct Barra {
    chave: &'static str,
    slider: NodeId,
    chip: NodeId,
    pista: f32,
}

impl Barra {
    fn item(self) -> Item {
        Item::Barra {
            chave: self.chave,
            ids: [self.slider, self.chip],
        }
    }
}

/// A resposta do `plano_do_corpo`.
pub(crate) fn presente(b: &BrushSettings) -> bool {
    !b.is_inpaint
}

pub(crate) fn titulo() -> &'static str {
    tr("panel.painter_layers.stroke.stroke")
}

fn metodo(b: &BrushSettings) -> StrokeMethod {
    StrokeMethod::from_u8(b.stroke_method)
}

fn barra(chave: &'static str, slider: NodeId, chip: NodeId, pista: f32) -> Barra {
    Barra {
        chave,
        slider,
        chip,
        pista,
    }
}

fn espacamento(b: &BrushSettings) -> Barra {
    barra(
        "panel.painter_layers.stroke.spacing",
        pids::PAINTER_BRUSH_SPACING,
        pids::PAINTER_BRUSH_SPACING_CHIP,
        b.spacing,
    )
}

fn da_grelha(b: &BrushSettings) -> [Barra; 5] {
    [
        barra(
            "panel.painter_layers.stroke.grid.cell_x",
            pids::PAINTER_BRUSH_GRID_CELL[0],
            pids::PAINTER_BRUSH_GRID_CELL_CHIPS[0],
            b.grid_cell[0],
        ),
        barra(
            "panel.painter_layers.stroke.grid.cell_y",
            pids::PAINTER_BRUSH_GRID_CELL[1],
            pids::PAINTER_BRUSH_GRID_CELL_CHIPS[1],
            b.grid_cell[1],
        ),
        barra(
            "panel.painter_layers.stroke.grid.offset_x",
            pids::PAINTER_BRUSH_GRID_OFFSET[0],
            pids::PAINTER_BRUSH_GRID_OFFSET_CHIPS[0],
            b.grid_offset[0],
        ),
        barra(
            "panel.painter_layers.stroke.grid.offset_y",
            pids::PAINTER_BRUSH_GRID_OFFSET[1],
            pids::PAINTER_BRUSH_GRID_OFFSET_CHIPS[1],
            b.grid_offset[1],
        ),
        barra(
            "panel.painter_layers.stroke.grid.cell_fit",
            pids::PAINTER_BRUSH_GRID_FIT,
            pids::PAINTER_BRUSH_GRID_FIT_CHIP,
            b.grid_fit,
        ),
    ]
}

fn do_desvio(b: &BrushSettings) -> Barra {
    barra(
        "panel.painter_layers.stroke.distance",
        pids::PAINTER_BRUSH_OFFSET,
        pids::PAINTER_BRUSH_OFFSET_CHIP,
        b.offset,
    )
}

/// O Rate, antes do Jitter.
fn do_tempo(b: &BrushSettings) -> Barra {
    let span = BRUSH_AIRBRUSH_RATE_MAX_S - BRUSH_AIRBRUSH_RATE_MIN_S;
    let pista = ((b.airbrush_rate_s - BRUSH_AIRBRUSH_RATE_MIN_S) / span).clamp(0.0, 1.0);
    barra(
        "panel.painter_layers.stroke.rate",
        pids::PAINTER_BRUSH_RATE,
        pids::PAINTER_BRUSH_RATE_CHIP,
        pista,
    )
}

/// As barras do fim: Dash e Length, Samples, Stabilize.
fn do_fim(l: &BrushSettings, v: &BrushSettings) -> Vec<Barra> {
    let m = metodo(l);
    let mut n = Vec::new();
    if m.uses_dash() {
        n.push(barra(
            "panel.painter_layers.stroke.dash",
            pids::PAINTER_BRUSH_DASH_RATIO,
            pids::PAINTER_BRUSH_DASH_RATIO_CHIP,
            v.dash_ratio,
        ));
        n.push(barra(
            "panel.painter_layers.stroke.length",
            pids::PAINTER_BRUSH_DASH_LENGTH,
            pids::PAINTER_BRUSH_DASH_LENGTH_CHIP,
            paint_stroke::count_to_norm(v.dash_samples),
        ));
    }
    n.push(barra(
        "panel.painter_layers.stroke.samples",
        pids::PAINTER_BRUSH_INPUT_SAMPLES,
        pids::PAINTER_BRUSH_INPUT_SAMPLES_CHIP,
        paint_stroke::count_to_norm(v.input_samples),
    ));
    if m.uses_stabilizer() {
        n.push(barra(
            "panel.painter_layers.stroke.stabilize",
            pids::PAINTER_BRUSH_STABILIZE,
            pids::PAINTER_BRUSH_STABILIZE_CHIP,
            v.stabilizer,
        ));
    }
    n
}

/// As barras do Jitter. O Unit é o único menu.
fn do_jitter(l: &BrushSettings, v: &BrushSettings) -> Vec<Barra> {
    jitter_linhas::fileiras(l)
        .filter_map(|f| match f {
            jitter_linhas::Fileira::Unidade => None,
            jitter_linhas::Fileira::Pincel {
                chave,
                slider,
                chip,
                pista,
            } => Some(barra(chave, slider, chip, pista(v))),
            jitter_linhas::Fileira::Marca(linha) => {
                let i = jitter_linhas::indice_da_marca(linha);
                let (escala, desloc) = jitter_linhas::escala_da_marca(linha);
                Some(barra(
                    crate::paint_stroke::jitter_card::ROTULOS_DA_MARCA[i],
                    pids::PAINTER_MARCA_FIELDS[i],
                    pids::PAINTER_MARCA_CHIPS[i],
                    (v.marca[i] as f32 - desloc) / escala,
                ))
            }
        })
        .collect()
}

/// Todas as barras à vista neste estado: só serve quem as arma.
fn todas(l: &BrushSettings, v: &BrushSettings) -> Vec<Barra> {
    let m = metodo(l);
    let mut n = Vec::new();
    if m.uses_spacing() {
        n.push(espacamento(v));
    }
    if matches!(m, StrokeMethod::GridStamp) {
        n.extend(da_grelha(v));
    }
    if m.has_open_shape() {
        n.push(do_desvio(v));
    }
    if m.uses_rate() {
        n.push(do_tempo(v));
    }
    if m.allows_jitter() {
        n.extend(do_jitter(l, v));
    }
    n.extend(do_fim(l, v));
    n
}

/// O passo do chip e o número que ele mostra para a pista: a lei é a do store.
fn arma(store: &mut WidgetStore, b: &Barra, fabrica: &Barra) {
    let passo = store
        .number_range(b.chip)
        .map_or(cards_topo::PASSO_DO_PINCEL, |(_, _, s)| s);
    let de_fabrica = store.linked_chip_display(b.chip, fabrica.pista);
    store.mark_card_field(
        b.chip,
        CardFieldSpec {
            step: passo,
            scrub: passo,
            default: Some(f64::from(de_fabrica)),
        },
    );
    let mostrado = store.linked_chip_display(b.chip, b.pista);
    number_field::mirror_value(store, b.chip, mostrado, cards_topo::CASAS_DO_PINCEL);
    let _ = esmaecer::poe_dica(store, &[b.slider, b.chip]);
}

/// Com o store MUTÁVEL, antes de pintar.
pub(crate) fn prepare(ctx: &mut PaintCtx, b: &BrushSettings) {
    if !presente(b) {
        return;
    }
    let store = ctx.host.store_mut();
    let fabrica = todas(b, &paint_brush::FALLBACK_BRUSH);
    for (barra, f) in todas(b, b).iter().zip(&fabrica) {
        arma(store, barra, f);
    }
    for id in [
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        pids::PAINTER_BRUSH_JITTER_UNIT,
        pids::PAINTER_BRUSH_CONTINUOUS_SPACING,
        pids::PAINTER_BRUSH_SPACE_ATTEN,
        pids::PAINTER_BRUSH_LINE_DIMENSIONS,
        pids::PAINTER_BRUSH_GRID_SHOW,
        pids::PAINTER_BRUSH_EDGE_TO_EDGE,
        pids::PAINTER_BRUSH_OFFSET_TRIM,
    ] {
        let _ = esmaecer::poe_dica(store, &[id]);
    }
    for id in [
        pids::PAINTER_BRUSH_STROKE_APPLY,
        pids::PAINTER_BRUSH_STROKE_APPLY_KEEP,
        pids::PAINTER_BRUSH_STROKE_DELETE,
        pids::PAINTER_BRUSH_STROKE_EDIT,
        pids::PAINTER_BRUSH_STROKE_SIMPLIFY,
        pids::PAINTER_BRUSH_STROKE_MERGE,
        pids::PAINTER_BRUSH_STROKE_SAVE_OBJECT,
    ] {
        crate::paint::register_button(store, id);
    }
}

/// Onde a caixa aberta ficou é o que a lista das opções precisa.
pub(crate) fn listas_abertas(ctx: &mut PaintCtx, b: &BrushSettings) {
    let aberto = |ctx: &mut PaintCtx, id| {
        if aberta(ctx.host.store(), id) {
            ctx.host.hit_index_mut().rect_for(id)
        } else {
            None
        }
    };
    if let Some(r) = aberto(ctx, core_ids::PAINTER_BRUSH_STROKE_METHOD) {
        state::set_pending_brush_stroke_method_dd(Some((r, b.stroke_method)));
    }
    if let Some(r) = aberto(ctx, pids::PAINTER_BRUSH_JITTER_UNIT) {
        state::set_pending_brush_jitter_unit_dd(Some((r, b.jitter_unit)));
    }
}

/// O título do Jitter, na posição do quadro anterior: dobra dentro do cartão.
pub(crate) fn titulo_do_jitter(
    ctx: &mut PaintCtx,
    ink: CardInk,
    (x, w): (f32, f32),
    brush: &BrushSettings,
) -> Option<Captura> {
    metodo(brush).allows_jitter().then(|| {
        let chave = chave_da_peca(SECAO, 0);
        let topo = ctx.slot.y;
        let tema = ink.theme;
        let x = x_de(chave, x);
        Captura::da_peca(ctx, chave, SECAO, ink, (x, y_de(chave, topo)), |ctx, y| {
            let (y, dobra) = crate::paint_brush_top::paint_sub_section(
                ctx,
                tema,
                x,
                w,
                y,
                tr("panel.painter_layers.stroke.jitter.jitter"),
                JITTER,
            );
            dobra.map_or(y, |d| crate::paint_brush_top::end_fold(ctx, d, y))
        })
    })
}

fn em_fila(c: &mut CardCtx<'_>, barras: &[Barra], y: f32) -> f32 {
    let itens: Vec<Item> = barras.iter().map(|b| b.item()).collect();
    fila(c, &itens, y)
}

fn caixa(id: NodeId, chave: &'static str, on: bool) -> Item {
    Item::Caixa { id, chave, on }
}

/// A figura aberta na tela: a operação, os verbos, o desvio paralelo.
fn da_figura(c: &mut CardCtx<'_>, b: &BrushSettings, m: StrokeMethod, y: f32) -> f32 {
    let op = usize::from(b.stroke_op_mode);
    let segs = [
        (
            pids::PAINTER_STROKE_OP_OVERLAY,
            "panel.painter_layers.stroke.overlay",
        ),
        (
            pids::PAINTER_STROKE_OP_ADD,
            "panel.painter_layers.stroke.add",
        ),
        (
            pids::PAINTER_STROKE_OP_REMOVE,
            "panel.painter_layers.stroke.remove",
        ),
    ]
    .iter()
    .enumerate()
    .map(|(i, (id, k))| Seg {
        id: *id,
        icon: None,
        text: tr(k),
        on: i == op,
    })
    .collect::<Vec<_>>();
    let mut y = escolha(c, tr("card.painter.stroke.operation"), &segs, y);
    let mut curvas = Vec::with_capacity(3);
    if b.can_simplify {
        curvas.push((
            pids::PAINTER_BRUSH_STROKE_SIMPLIFY,
            tr("panel.painter_layers.stroke.simplify"),
        ));
    }
    curvas.push((
        pids::PAINTER_BRUSH_STROKE_MERGE,
        tr("panel.painter_layers.stroke.merge_curves"),
    ));
    if m.is_convertible_shape() {
        curvas.push((
            pids::PAINTER_BRUSH_STROKE_EDIT,
            tr("panel.painter_layers.stroke.convert_to_curve"),
        ));
    }
    y = botoes(c, &curvas, y);
    y = botoes(
        c,
        &[
            (
                pids::PAINTER_BRUSH_STROKE_APPLY,
                tr("panel.painter_layers.stroke.apply"),
            ),
            (
                pids::PAINTER_BRUSH_STROKE_APPLY_KEEP,
                tr("panel.painter_layers.stroke.apply_and_keep"),
            ),
            (
                pids::PAINTER_BRUSH_STROKE_DELETE,
                tr("card.painter.stroke.delete"),
            ),
        ],
        y,
    );
    let y = grupo(c, "panel.painter_layers.stroke.offset", y);
    fila(
        c,
        &[
            do_desvio(b).item(),
            caixa(
                pids::PAINTER_BRUSH_OFFSET_TRIM,
                "panel.painter_layers.stroke.trim",
                b.offset_trim,
            ),
        ],
        y,
    )
}

/// O corpo.
pub(crate) fn corpo(c: &mut CardCtx<'_>, titulo_do_jitter: Option<&Captura>, y: f32) -> f32 {
    let b = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    let m = metodo(&b);
    let mut topo = vec![Item::Lista {
        id: core_ids::PAINTER_BRUSH_STROKE_METHOD,
        chave: "panel.painter_layers.stroke.method",
        texto: paint_stroke::stroke_method_name(b.stroke_method),
    }];
    let mut espaco = Vec::new();
    if m.uses_spacing() {
        espaco.push(espacamento(&b).item());
        if b.le(LinhaDaFerramenta::Continuo) {
            espaco.push(caixa(
                pids::PAINTER_BRUSH_CONTINUOUS_SPACING,
                "panel.painter_layers.stroke.continuous_spacing",
                b.continuous_spacing,
            ));
        }
        if b.accumulate_offered() {
            espaco.push(caixa(
                pids::PAINTER_BRUSH_SPACE_ATTEN,
                "panel.painter_layers.stroke.adjust_strength",
                b.space_attenuation,
            ));
        }
    }
    if b.has_drawn_curve {
        topo.push(Item::Botao {
            id: pids::PAINTER_BRUSH_STROKE_SAVE_OBJECT,
            texto: tr("card.painter.stroke.save"),
        });
    }
    let mut y = fila(c, &topo, y);
    y = fila(c, &espaco, y);
    if matches!(m, StrokeMethod::GridStamp) {
        y = grupo(c, "panel.painter_layers.stroke.grid.grid", y);
        let mut itens: Vec<Item> = da_grelha(&b).iter().map(|g| g.item()).collect();
        itens.push(caixa(
            pids::PAINTER_BRUSH_GRID_SHOW,
            "panel.painter_layers.stroke.grid.show_grid",
            b.grid_show,
        ));
        y = fila(c, &itens, y);
    }
    let mut soltos = Vec::new();
    if matches!(m, StrokeMethod::Line) {
        soltos.push(caixa(
            pids::PAINTER_BRUSH_LINE_DIMENSIONS,
            "panel.painter_layers.stroke.dimensions",
            b.line_show_dimensions,
        ));
    }
    if m.has_open_shape() {
        y = fila(c, &soltos, y);
        soltos.clear();
        y = da_figura(c, &b, m, y);
    }
    if m.uses_edge_to_edge() {
        soltos.push(caixa(
            pids::PAINTER_BRUSH_EDGE_TO_EDGE,
            "panel.painter_layers.stroke.edge_to_edge",
            b.edge_to_edge,
        ));
    }
    if m.uses_rate() {
        soltos.push(do_tempo(&b).item());
    }
    y = fila(c, &soltos, y);
    if m.allows_jitter() {
        y = crate::cards_shape::repete(c, titulo_do_jitter, y);
        if !c.store.is_collapsed(JITTER) {
            let mut itens = vec![Item::Lista {
                id: pids::PAINTER_BRUSH_JITTER_UNIT,
                chave: "panel.painter_layers.stroke.jitter.unit",
                texto: paint_stroke::jitter_card::jitter_unit_name(b.jitter_unit),
            }];
            itens.extend(do_jitter(&b, &b).iter().map(|g| g.item()));
            y = fila(c, &itens, y);
        }
    }
    em_fila(c, &do_fim(&b, &b), y)
}
