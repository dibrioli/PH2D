//! Os cartões: Draw e Tileset, e o da janela de criar ou converter.

use ph2d_editor_core::panel::cards::ferramenta::{
    Numero, aos_pares, caixa, caixas_aos_pares, campo, cartao, opcoes,
};
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_i18n::{tr, tr_with};
use ph2d_pixel_tiles::desenho::Modo;
use ph2d_tokens::{ColorToken, StrokeToken};

use crate::canal::{Alvo, Contexto, Mao, Tela};
use crate::state::TILE_INICIAL;
use crate::{PixelTilesPanelState, ids};

/// O lado de uma célula do tileset, em alturas de campo: cabem 4 por linha na janela.
const LADO_DA_CELULA: f32 = 1.25; // LITERAL-PX-OK: razão de alturas, não píxeis

pub(crate) fn numeros(st: &PixelTilesPanelState) -> Vec<Numero> {
    let n = |chip, valor: u16| Numero {
        chip,
        step: 1.0,
        scrub: 1.0,
        fabrica: f64::from(TILE_INICIAL),
        valor: f64::from(valor),
    };
    vec![n(ids::TILE_W, st.tile_w), n(ids::TILE_H, st.tile_h)]
}

/// O cartão da janela de criar ou converter.
pub(crate) fn janela(st: &PixelTilesPanelState) -> Vec<CardDef<'static>> {
    let (tela, flips, quadrado) = (st.tela, st.flips, st.tile_w == st.tile_h);
    vec![cartao(
        ids::TILES_SEC_GRID,
        tr("panel.pixel.tiles.grid"),
        None,
        move |c, y| {
            let y = aos_pares(
                c,
                &[
                    campo(ids::TILE_W, tr("panel.pixel.tiles.tile_w")),
                    campo(ids::TILE_H, tr("panel.pixel.tiles.tile_h")),
                ],
                y,
            );
            if tela != Tela::Converter {
                return c.hint(tr("panel.pixel.tiles.new_hint"), y);
            }
            let y = if quadrado {
                caixa(
                    c,
                    ids::MATCH_FLIPS,
                    "panel.pixel.tiles.match_flips",
                    flips,
                    y,
                )
            } else {
                y
            };
            c.hint(tr("panel.pixel.tiles.convert_hint"), y)
        },
    )]
}

fn dica_do_modo(m: Mao) -> &'static str {
    match (m.alvo, m.modo) {
        (Alvo::Tiles, _) => "panel.pixel.tiles.hint_tiles",
        (Alvo::Pixels, Modo::Manual) => "panel.pixel.tiles.hint_manual",
        (Alvo::Pixels, Modo::Auto) => "panel.pixel.tiles.hint_auto",
        (Alvo::Pixels, Modo::Stack) => "panel.pixel.tiles.hint_stack",
    }
}

fn cartao_draw(m: Mao) -> CardDef<'static> {
    cartao(
        ids::SEC_DRAW,
        tr("panel.pixel.tiles.draw"),
        None,
        move |c, y| {
            let alvos = opcoes(
                &[
                    (ids::TARGET_PIXELS, tr("panel.pixel.tiles.pixels")),
                    (ids::TARGET_TILES, tr("panel.pixel.tiles.tiles")),
                ],
                usize::from(m.alvo == Alvo::Tiles),
            );
            let y = c.full(None, Widget::Choice(&alvos), y);
            let y = if m.alvo == Alvo::Pixels {
                let aceso = match m.modo {
                    Modo::Manual => 0,
                    Modo::Auto => 1,
                    Modo::Stack => 2,
                };
                let modos = opcoes(
                    &[
                        (ids::MODE_MANUAL, tr("panel.pixel.tiles.manual")),
                        (ids::MODE_AUTO, tr("panel.pixel.tiles.auto")),
                        (ids::MODE_STACK, tr("panel.pixel.tiles.stack")),
                    ],
                    aceso,
                );
                c.full(None, Widget::Choice(&modos), y)
            } else {
                y
            };
            c.hint(tr(dica_do_modo(m)), y)
        },
    )
}

/// A grelha de tiles; a célula inteira é o alvo do clique.
fn grelha(
    c: &mut CardCtx<'_>,
    imagens: &[Option<ph2d_vector::StableImage>],
    lados: (u16, u16),
    escolhido: u32,
    top: f32,
) -> f32 {
    use ph2d_editor_core::paint::{fill_rounded_rect, stroke_rounded_rect};
    use ph2d_editor_core::panel::cards::banda::metrics as m;
    use ph2d_editor_core::zones::Rect;
    let vao = m::FIELD_GAP;
    let pedido = m::FIELD_H * LADO_DA_CELULA;
    let colunas = (((c.w + vao) / (pedido + vao)).floor() as usize).max(1);
    let lado = (c.w - vao * (colunas - 1) as f32) / colunas as f32;
    let (tw, th) = (f32::from(lados.0), f32::from(lados.1));
    let e = (lado / tw.max(1.0)).min(lado / th.max(1.0));
    for (i, img) in imagens.iter().enumerate() {
        let id = ids::tile(i);
        let q = Rect::new(
            c.x + (i % colunas) as f32 * (lado + vao),
            top + (i / colunas) as f32 * (lado + vao),
            lado,
            lado,
        );
        fill_rounded_rect(
            c.scene,
            q,
            m::SEG_RADIUS,
            c.ink.color(ColorToken::CardField),
        );
        if let Some(img) = img {
            let (dw, dh) = (tw * e, th * e);
            let (x, y) = (q.x + (lado - dw) * 0.5, q.y + (lado - dh) * 0.5);
            let dest = (
                f64::from(x),
                f64::from(y),
                f64::from(x + dw),
                f64::from(y + dh),
            );
            c.scene
                .draw_stable_image(img, dest, ph2d_vector::ImageQuality::Low);
        }
        let quente = c.store.button_visual(id).0 != ph2d_editor_core::widget::ButtonState::Normal;
        if i as u32 == escolhido || quente {
            stroke_rounded_rect(
                c.scene,
                q,
                m::SEG_RADIUS,
                StrokeToken::Default.px(),
                c.ink.color(ColorToken::Accent),
            );
        }
        c.hit_index.register(id, q);
    }
    top + imagens.len().div_ceil(colunas) as f32 * (lado + vao) - vao
}

fn cartao_tileset(m: Mao, st: &PixelTilesPanelState, x: &Contexto) -> CardDef<'static> {
    let imagens = st.imagens.clone();
    let lados = (x.tile_w, x.tile_h);
    let resumo = tr_with(
        "panel.pixel.tiles.summary",
        &[
            ("n", &imagens.len().saturating_sub(1).to_string()),
            ("w", &x.tile_w.to_string()),
            ("h", &x.tile_h.to_string()),
        ],
    );
    let mut def = cartao(
        ids::SEC_TILESET,
        tr("panel.pixel.tiles.tileset"),
        None,
        move |c, y| {
            let y = grelha(c, &imagens, lados, m.tile, y) + m_gap();
            let mut caixas = vec![
                (ids::FLIP_X, "panel.pixel.tiles.flip_x", m.flip_x),
                (ids::FLIP_Y, "panel.pixel.tiles.flip_y", m.flip_y),
            ];
            if lados.0 == lados.1 {
                caixas.push((ids::FLIP_D, "panel.pixel.tiles.flip_d", m.flip_d));
            }
            let y = caixas_aos_pares(c, &caixas, y);
            let add = Cell::new(Widget::Button {
                id: ids::ADD_TILE,
                text: tr("panel.pixel.tiles.add"),
                primary: false,
            });
            let del = Cell::new(Widget::Button {
                id: ids::DELETE_TILE,
                text: tr("panel.pixel.tiles.delete"),
                primary: false,
            });
            c.row(&[add, del], y)
        },
    );
    def.summary = Some(resumo);
    def
}

fn m_gap() -> f32 {
    ph2d_editor_core::panel::cards::banda::metrics::FIELD_ROW_GAP
}

/// Os cartões do painel, com a camada de tilemap activa.
pub(crate) fn tileset(st: &PixelTilesPanelState, x: &Contexto) -> Vec<CardDef<'static>> {
    let Some(m) = x.mao else { return Vec::new() };
    vec![cartao_draw(m), cartao_tileset(m, st, x)]
}
