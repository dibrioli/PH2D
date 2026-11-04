//! O cartão Randomize Color em linhas de cartão (fatia H6, `docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos ids, mesmos eventos.
//! Hue, Saturation e Value numa linha, como o H, S, B do seletor de cor do Figma; os irmãos curtos aos pares.

use crate::cards_linhas::{Item, com_lei, escolha, fila, numero, numero_e_cor};
use crate::paint_stroke::jitter_card;
use crate::paint_wetpaint::Numero;
use crate::{esmaecer, marca_cor, number_field, paint_brush};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget, corpo as linhas};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{BrushSettings, CartaoDaMarca, LINHAS_DA_MARCA, LinhaDaFerramenta};

const HSV: [[NodeId; 2]; 3] = [
    [
        pids::PAINTER_BRUSH_RANDOMIZE_SLIDERS[0],
        pids::PAINTER_BRUSH_RANDOMIZE_CHIPS[0],
    ],
    [
        pids::PAINTER_BRUSH_RANDOMIZE_SLIDERS[1],
        pids::PAINTER_BRUSH_RANDOMIZE_CHIPS[1],
    ],
    [
        pids::PAINTER_BRUSH_RANDOMIZE_SLIDERS[2],
        pids::PAINTER_BRUSH_RANDOMIZE_CHIPS[2],
    ],
];
const NOMES: [&str; 3] = [
    "panel.painter_layers.brush.hue",
    "panel.painter_layers.brush.saturation",
    "panel.painter_layers.brush.value",
];
const COLUNAS: f32 = 3.0; // LITERAL-PX-OK: H, S e V na mesma linha
const SECUNDARIA: NodeId = pids::PAINTER_MARCA_SECONDARY;

/// A resposta do `plano_do_corpo`.
pub(crate) fn presente(b: &BrushSettings) -> bool {
    b.le(LinhaDaFerramenta::Randomize)
}

pub(crate) fn titulo() -> &'static str {
    tr("panel.painter_layers.brush.randomize_color")
}

/// Os números da cor pela mão, pela ordem da tabela: o 1.º é o irmão da 2.ª cor.
fn da_mao(v: &BrushSettings) -> Vec<Numero> {
    jitter_card::linhas_do_cartao(CartaoDaMarca::Cor)
        .map(|i| {
            let (lo, hi, passo) = LINHAS_DA_MARCA[i].faixa();
            numero(
                jitter_card::ROTULOS_DA_MARCA[i],
                pids::PAINTER_MARCA_FIELDS[i],
                v.marca[i] as f32,
                (lo as f32, hi as f32),
                passo,
                2,
            )
        })
        .collect()
}

/// Com o store MUTÁVEL, antes de pintar.
pub(crate) fn prepare(ctx: &mut PaintCtx, b: &BrushSettings) {
    if !presente(b) {
        return;
    }
    marca_cor::le_a_secundaria(ctx, b);
    let store = ctx.host.store_mut();
    let fabrica = paint_brush::FALLBACK_BRUSH;
    for (slot, par) in HSV.iter().enumerate() {
        store.mark_card_field(
            par[1],
            CardFieldSpec {
                step: crate::cards_topo::PASSO_DO_PINCEL,
                scrub: crate::cards_topo::PASSO_DO_PINCEL,
                default: Some(f64::from(fabrica.color_jitter[slot])),
            },
        );
        number_field::mirror_value(
            store,
            par[1],
            b.color_jitter[slot],
            crate::cards_topo::CASAS_DO_PINCEL,
        );
        let _ = esmaecer::poe_dica(store, par);
    }
    crate::cards_wet::arma_numeros(store, &da_mao(b), &da_mao(&fabrica));
    let _ = esmaecer::poe_dica(store, &pids::PAINTER_BRUSH_COLOR_JITTER_SCOPES);
    let _ = esmaecer::poe_dica(store, &[SECUNDARIA]);
}

/// Os três na mesma linha; em coluna estreita, dois a dois (05 §4, lei 5).
fn matiz_saturacao_valor(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let cells: Vec<Cell<'_>> = HSV
        .iter()
        .zip(NOMES)
        .map(|(par, chave)| {
            let rotulo = tr(chave);
            Cell::new(Widget::Field {
                id: par[1],
                glyph: Glyph::Text(linhas::letra_da_grandeza(rotulo)),
            })
            .label(rotulo)
            .key(par)
        })
        .collect();
    let coluna = (c.w - (COLUNAS - 1.0) * m::FIELD_GAP) / COLUNAS;
    if coluna >= m::ICON_CELL + m::FIELD_VALUE_MIN_W {
        c.row(&cells, y)
    } else {
        c.compact(&cells, y)
    }
}

/// O corpo.
pub(crate) fn corpo(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let b = crate::state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    let y = matiz_saturacao_valor(c, y);
    let [por_marca, por_traco] = pids::PAINTER_BRUSH_COLOR_JITTER_SCOPES;
    let quando = [
        Seg {
            id: por_marca,
            icon: None,
            text: tr("panel.painter_layers.brush.every_dab"),
            on: b.color_jitter_scope == 0,
        },
        Seg {
            id: por_traco,
            icon: None,
            text: tr("panel.painter_layers.brush.every_stroke"),
            on: b.color_jitter_scope != 0,
        },
    ];
    let y = escolha(c, tr("card.painter.randomize.when"), &quando, y);
    let mut mao = da_mao(&b).into_iter();
    let Some(irmao) = mao.next() else {
        return y;
    };
    let y = com_lei(c, irmao.id, y, |c, y| {
        numero_e_cor(c, &irmao, "panel.painter_marca.secondary", SECUNDARIA, y)
    });
    let resto: Vec<Item> = mao.map(Item::Num).collect();
    fila(c, &resto, y)
}
