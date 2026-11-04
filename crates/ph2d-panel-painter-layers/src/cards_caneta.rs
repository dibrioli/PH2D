//! O cartão Pen em linhas de cartão (fatia H7, `docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos ids, mesmos eventos, as
//! condições de `paint_pen_section`. As linhas da mão vão por três grupos: pressão, inclinação, velocidade. O Reset continua
//! a ser um MENU, agora no cabeçalho do cartão.

use crate::cards_linhas::{Item, aberta, fila, grupo, grupo_inicial, numero};
use crate::paint_pen::{self, ROTULOS};
use crate::paint_wetpaint::Numero;
use crate::{esmaecer, paint_brush, state};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{BrushSettings, LINHAS_DA_MAO, PaintMedia};

/// O índice de cada linha em [`LINHAS_DA_MAO`], agrupadas por quem manda nelas.
const PRESSAO: [usize; 11] = [0, 1, 2, 3, 4, 5, 18, 19, 22, 6, 7];
const INCLINACAO: [usize; 8] = [8, 9, 10, 11, 12, 13, 14, 15];
const VELOCIDADE: [usize; 4] = [16, 17, 20, 21];

/// O rótulo do cartão da linha que tem unidade: ela sai da caixa e vai para o nome.
const COM_UNIDADE: [(usize, &str); 5] = [
    (7, "card.painter.pen.pulled_string"),
    (9, "card.painter.pen.tilt_start"),
    (10, "card.painter.pen.tilt_full"),
    (14, "card.painter.pen.tilt_lag"),
    (20, "card.painter.pen.speed_lag"),
];

/// A resposta do `plano_do_corpo`.
pub(crate) fn presente(b: &BrushSettings) -> bool {
    !b.is_inpaint
}

pub(crate) fn titulo() -> &'static str {
    tr("panel.painter_layers.pen.title")
}

fn a_vista(l: &BrushSettings, i: usize) -> bool {
    LINHAS_DA_MAO[i].lida(PaintMedia::from_u8(l.media), l.watercolor_shape_auto)
        && paint_pen::le_a_linha(l, pids::PAINTER_PEN_FIELDS[i])
}

fn e_a_rotacao(i: usize) -> bool {
    pids::PAINTER_PEN_FIELDS[i] == pids::PAINTER_PEN_ROTATION
}

fn rotulo(i: usize) -> &'static str {
    COM_UNIDADE
        .iter()
        .find(|(n, _)| *n == i)
        .map_or(ROTULOS[i], |(_, k)| k)
}

fn linha_numerica(i: usize, v: &BrushSettings) -> Numero {
    let (lo, hi, passo) = LINHAS_DA_MAO[i].faixa();
    numero(
        rotulo(i),
        pids::PAINTER_PEN_FIELDS[i],
        v.mao[i] as f32,
        (lo as f32, hi as f32),
        passo,
        paint_pen::casas(passo),
    )
}

/// Os números à vista: só serve quem os arma.
fn numeros(l: &BrushSettings, v: &BrushSettings) -> Vec<Numero> {
    (0..LINHAS_DA_MAO.len())
        .filter(|&i| a_vista(l, i) && !e_a_rotacao(i))
        .map(|i| linha_numerica(i, v))
        .collect()
}

/// Com o store MUTÁVEL, antes de pintar.
pub(crate) fn prepare(ctx: &mut PaintCtx, b: &BrushSettings) {
    if !presente(b) {
        return;
    }
    let store = ctx.host.store_mut();
    crate::cards_wet::arma_numeros(
        store,
        &numeros(b, b),
        &numeros(b, &paint_brush::FALLBACK_BRUSH),
    );
    let _ = esmaecer::poe_dica(store, &[pids::PAINTER_PEN_ROTATION]);
}

/// O menu da rotação, debaixo da caixa; o do Reset, debaixo do cabeçalho, à largura do corpo.
pub(crate) fn listas_abertas(ctx: &mut PaintCtx, b: &BrushSettings, (x, w): (f32, f32)) {
    if aberta(ctx.host.store(), pids::PAINTER_PEN_ROTATION)
        && let Some(r) = ctx
            .host
            .hit_index_mut()
            .rect_for(pids::PAINTER_PEN_ROTATION)
    {
        let i = pids::PAINTER_PEN_FIELDS
            .iter()
            .position(|id| *id == pids::PAINTER_PEN_ROTATION)
            .unwrap_or_default();
        paint_pen::abre_a_rotacao(r, b.mao[i]);
    }
    if aberta(ctx.host.store(), pids::PAINTER_PEN_RESET)
        && let Some(r) = ctx.host.hit_index_mut().rect_for(pids::PAINTER_PEN_RESET)
    {
        paint_pen::abre_o_reset(Rect::new(x, r.y, w, r.h), b.mao_do_preset);
    }
}

fn itens_de(linhas: &[usize], b: &BrushSettings) -> Vec<Item> {
    linhas
        .iter()
        .filter(|&&i| a_vista(b, i))
        .map(|&i| {
            if e_a_rotacao(i) {
                Item::ListaComMarca {
                    id: pids::PAINTER_PEN_ROTATION,
                    chave: ROTULOS[i],
                    texto: paint_pen::nome_da_rotacao(b.mao[i]),
                }
            } else {
                Item::Num(linha_numerica(i, b))
            }
        })
        .collect()
}

/// O corpo.
pub(crate) fn corpo(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let b = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    let mut y = y;
    let mut primeiro = true;
    for (chave, linhas) in [
        ("card.painter.pen.pressure", &PRESSAO[..]),
        ("card.painter.pen.tilt", &INCLINACAO[..]),
        ("card.painter.pen.speed", &VELOCIDADE[..]),
    ] {
        let itens = itens_de(linhas, &b);
        if itens.is_empty() {
            continue;
        }
        y = if primeiro {
            grupo_inicial(c, chave, y)
        } else {
            grupo(c, chave, y)
        };
        primeiro = false;
        y = fila(c, &itens, y);
    }
    y
}
