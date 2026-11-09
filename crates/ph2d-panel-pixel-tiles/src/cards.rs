//! Os cartões: Draw e Tileset, e o da janela de criar ou converter.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::Panel;
use ph2d_editor_core::panel::cards::ferramenta::{
    Numero, aos_pares, caixa, caixas_aos_pares, campo, cartao, opcoes,
};
use ph2d_editor_core::panel::cards::user_data;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_i18n::{tr, tr_with};
use ph2d_pixel_core::Forma;
use ph2d_pixel_tiles::desenho::Modo;
use ph2d_tokens::{ColorToken, StrokeToken};

use crate::canal::{Alvo, Contexto, Mao, Tela};
use crate::state::TILE_INICIAL;
use crate::{PixelTilesPanel, PixelTilesPanelState, ids};

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

/// A dica da janela: o que acontece com os tiles ao confirmar.
const fn dica_da_janela(tela: Tela, existente: bool) -> &'static str {
    match (tela, existente) {
        (Tela::Converter, true) => "panel.pixel.tiles.convert_hint_shared",
        (Tela::Converter, false) => "panel.pixel.tiles.convert_hint",
        (_, true) => "panel.pixel.tiles.shared_hint",
        (_, false) => "panel.pixel.tiles.new_hint",
    }
}

/// A linha da forma da grelha.
fn linha_da_forma(c: &mut CardCtx<'_>, forma: Forma, y: f32) -> f32 {
    let nome = |f: Forma| match f {
        Forma::Square => tr("panel.pixel.tiles.shape_square"),
        Forma::Iso => tr("panel.pixel.tiles.shape_iso"),
        Forma::Hex => tr("panel.pixel.tiles.shape_hex"),
        Forma::HexFlat => tr("panel.pixel.tiles.shape_hex_flat"),
    };
    let lista: Vec<_> = ids::SHAPES
        .iter()
        .zip(Forma::TODAS)
        .map(|(id, f)| (*id, nome(f)))
        .collect();
    let aceso = Forma::TODAS.iter().position(|f| *f == forma).unwrap_or(0);
    let o = opcoes(&lista, aceso);
    c.full(Some(tr("panel.pixel.tiles.shape")), Widget::Choice(&o), y)
}

/// O cartão da janela de criar ou converter: os tilesets do sprite, ou um novo com o tamanho dele.
pub(crate) fn janela(st: &PixelTilesPanelState, x: &Contexto) -> Vec<CardDef<'static>> {
    let escolhido = st.escolhido(x);
    let (tela, flips, forma) = (st.tela, st.flips, st.forma);
    let quadrado = escolhido.map_or(st.tile_w == st.tile_h, |c| c.tile_w == c.tile_h);
    let lista: Vec<(NodeId, String, bool)> = x
        .conjuntos
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let rotulo = tr_with(
                "panel.pixel.tiles.set_label",
                &[
                    ("i", &(i + 1).to_string()),
                    ("n", &c.tiles.to_string()),
                    ("w", &c.tile_w.to_string()),
                    ("h", &c.tile_h.to_string()),
                ],
            );
            (
                ids::conjunto(i),
                rotulo,
                escolhido.is_some_and(|e| e.id == c.id),
            )
        })
        .collect();
    vec![cartao(
        ids::TILES_SEC_GRID,
        tr("panel.pixel.tiles.tileset"),
        None,
        move |c, mut y| {
            for (id, rotulo, aceso) in &lista {
                let o = opcoes(&[(*id, rotulo.as_str())], usize::from(!*aceso));
                y = c.full(None, Widget::Choice(&o), y);
            }
            if !lista.is_empty() {
                let novo = (ids::NEW_SET, tr("panel.pixel.tiles.new_set"));
                let o = opcoes(&[novo], usize::from(escolhido.is_some()));
                y = c.full(None, Widget::Choice(&o), y);
            }
            if escolhido.is_none() {
                y = aos_pares(
                    c,
                    &[
                        campo(ids::TILE_W, tr("panel.pixel.tiles.tile_w")),
                        campo(ids::TILE_H, tr("panel.pixel.tiles.tile_h")),
                    ],
                    y,
                );
                if tela != Tela::Converter {
                    y = linha_da_forma(c, forma, y);
                }
            }
            if tela == Tela::Converter && quadrado {
                y = caixa(
                    c,
                    ids::MATCH_FLIPS,
                    "panel.pixel.tiles.match_flips",
                    flips,
                    y,
                );
            }
            c.hint(tr(dica_da_janela(tela, escolhido.is_some())), y)
        },
    )]
}

fn dica_do_modo(m: Mao) -> &'static str {
    match (m.alvo, m.modo) {
        (Alvo::Tiles, _) => "panel.pixel.tiles.hint_tiles",
        (Alvo::Terreno, _) => "panel.pixel.tiles.hint_terrain",
        (Alvo::Pixels, Modo::Manual) => "panel.pixel.tiles.hint_manual",
        (Alvo::Pixels, Modo::Auto) => "panel.pixel.tiles.hint_auto",
        (Alvo::Pixels, Modo::Stack) => "panel.pixel.tiles.hint_stack",
    }
}

fn cartao_draw(m: Mao, forma: Forma) -> CardDef<'static> {
    cartao(
        ids::SEC_DRAW,
        tr("panel.pixel.tiles.draw"),
        None,
        move |c, y| {
            let mut lista = vec![
                (ids::TARGET_PIXELS, tr("panel.pixel.tiles.pixels")),
                (ids::TARGET_TILES, tr("panel.pixel.tiles.tiles")),
            ];
            if forma.tem_terreno() {
                lista.push((ids::TARGET_TERRAIN, tr("panel.pixel.tiles.terrain")));
            }
            let aceso = match m.alvo {
                Alvo::Pixels => 0,
                Alvo::Tiles => 1,
                Alvo::Terreno => 2,
            };
            let alvos = opcoes(&lista, aceso);
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
            // FRAME-RAW-OK: o contorno de acento diz qual tile está escolhido ou sob o cursor — é a mensagem
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
    let forma = x.forma;
    let user = x.user.clone();
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
            let y = linha_da_forma(c, forma, y);
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
            let ud = ids::user_data();
            let y = match &user {
                Some(_) => c.row(&[add, del, user_data::toggle_cell(&ud)], y),
                None => c.row(&[add, del], y),
            };
            match &user {
                Some(v) => user_data::open_body(c, &ud, v, y),
                None => y,
            }
        },
    );
    def.summary = Some(resumo);
    def
}

fn m_gap() -> f32 {
    ph2d_editor_core::panel::cards::banda::metrics::FIELD_ROW_GAP
}

/// O cartão Terrain: os terrenos do tileset, criar um, pintar as formas dele e apagar.
fn cartao_terreno(m: Mao, x: &Contexto) -> CardDef<'static> {
    let nomes = x.terrenos.clone();
    cartao(
        ids::SEC_TERRAIN,
        tr("panel.pixel.tiles.terrain"),
        None,
        move |c, mut y| {
            for (i, nome) in nomes.iter().enumerate() {
                let o = opcoes(
                    &[(ids::terreno(i), nome.as_str())],
                    usize::from(i as u32 != m.terreno),
                );
                y = c.full(None, Widget::Choice(&o), y);
            }
            let botao = |id, chave| {
                Cell::new(Widget::Button {
                    id,
                    text: tr(chave),
                    primary: false,
                })
            };
            let mut linha = vec![botao(ids::NEW_TERRAIN, "panel.pixel.tiles.new_terrain")];
            if !nomes.is_empty() {
                linha.push(botao(
                    ids::TERRAIN_TEMPLATE,
                    "panel.pixel.tiles.terrain_template",
                ));
            }
            if !nomes.is_empty() {
                linha.push(botao(
                    ids::DELETE_TERRAIN,
                    "panel.pixel.tiles.delete_terrain",
                ));
            }
            y = c.row(&linha, y);
            c.hint(tr("panel.pixel.tiles.terrain_help"), y)
        },
    )
}

/// O texto sem camada tilemap: sem Pixel Sprite em Draw não há menu Layers a usar (foto `P12_fecho_05`).
pub(crate) fn vazio(x: &Contexto) -> &'static str {
    if x.com_sprite {
        "panel.pixel.tiles.empty"
    } else {
        "panel.pixel.tiles.no_document"
    }
}

/// Os cartões do painel, com a camada de tilemap activa.
pub(crate) fn tileset(st: &PixelTilesPanelState, x: &Contexto) -> Vec<CardDef<'static>> {
    let Some(m) = x.mao else {
        // Aberta pelo menu *Window* sem camada tilemap: diz como fazer uma (a marca é a janela).
        let chave = vazio(x);
        let vazio = move |c: &mut CardCtx<'_>, y| c.hint(tr(chave), y);
        return vec![cartao(
            ids::SEC_TILESET,
            PixelTilesPanel::TITLE.tr(),
            None,
            vazio,
        )];
    };
    let mut v = vec![cartao_draw(m, x.forma)];
    if x.forma.tem_terreno() {
        v.push(cartao_terreno(m, x));
    }
    v.push(cartao_tileset(m, st, x));
    v
}

#[cfg(test)]
mod tests {
    use super::vazio;
    use crate::canal::Contexto;

    /// ⭐ Foto `P12_fecho_05`: sem Pixel Sprite em Draw a janela mandava usar o menu Layers, que fora
    /// do Draw não está à vista. Cada estado tem o seu texto, e os dois estão na tabela.
    #[test]
    fn sem_camada_tilemap_diz_o_que_fazer_em_cada_estado() {
        let com = Contexto {
            com_sprite: true,
            ..Contexto::default()
        };
        assert_eq!(vazio(&Contexto::default()), "panel.pixel.tiles.no_document");
        assert_eq!(vazio(&com), "panel.pixel.tiles.empty");
        for x in [Contexto::default(), com] {
            let k = vazio(&x);
            assert_ne!(ph2d_i18n::tr(k), k, "{k} sem texto");
        }
    }
}
