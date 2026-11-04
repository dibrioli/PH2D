//! ⭐ **O cartão Advanced no vocabulário** (dono 03/11, doc `CartoesUI/08` §1.2: *«Card advanced tb
//! parece fora do novo padrão»*): os campos Figma, a grelha de camadas nos tons do cartão, a escolha
//! e as listas de propriedades do Figma e do Rive. O corpo antigo só vale em `PH2D_CARDS=0`.

use crate::controlos::avancado::{META_ROWS, META_SWATCH, META_TYPES, PROP_ROWS};
use crate::controlos::{CAMPO_MIN, caixa_para, card, dropdown, field, letra, num, seg_of, segs};
use crate::ids as g;
use crate::state::Choices;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget, corpo};
use ph2d_editor_core::widget::card_field::{self, Glyph};
use ph2d_editor_core::widget::{BitmaskGrid32, paint_bitmask_grid32_on_card};
use ph2d_i18n::tr;

/// O nome de cada entrada de Metadata.
const META_KEYS: [&str; 6] = [
    "panel.widget_gallery.adv.meta.hue_shift",
    "panel.widget_gallery.adv.meta.label",
    "panel.widget_gallery.adv.meta.tint",
    "panel.widget_gallery.adv.meta.entry_4",
    "panel.widget_gallery.adv.meta.entry_5",
    "panel.widget_gallery.adv.meta.entry_6",
];

fn bounds(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let k = [g::ADV_X, g::ADV_Y, g::ADV_W, g::ADV_H];
    let y = c.row(
        &[
            Cell::new(field(g::ADV_X, Glyph::Text(tr("card.glyph.x"))))
                .label(tr("chrome.sample.bounds")),
            Cell::new(field(g::ADV_Y, Glyph::Text(tr("card.glyph.y")))).key(&k),
        ],
        y,
    );
    c.row(
        &[
            Cell::new(field(
                g::ADV_W,
                Glyph::Text(tr("panel.widget_gallery.glyph.w")),
            )),
            Cell::new(field(
                g::ADV_H,
                Glyph::Text(tr("panel.widget_gallery.glyph.h")),
            )),
        ],
        y,
    )
}

/// Um rótulo sozinho, por cima de um widget que não é de linha. Devolve o fundo.
fn rotulo(c: &mut CardCtx<'_>, text: &str, y: f32) -> f32 {
    let y = y + m::LABEL_TOP;
    card_field::paint_label(c.scene, c.text_system, c.ink, c.x, y, c.w, text);
    y + m::LABEL_PX + m::LABEL_BOTTOM
}

/// As 32 camadas: a grelha (um bom widget, palavra do dono) nos tons do cartão; o clique alterna.
fn camadas(c: &mut CardCtx<'_>, ch: &Choices, y: f32) -> f32 {
    let y = rotulo(c, tr("chrome.sample.collision_layers"), y);
    let cell_h = m::ICON_CELL;
    let grid = BitmaskGrid32::new(g::CARD_ADVANCED, "", g::LAYERS, ch.layers);
    for (bit, id) in g::LAYERS.iter().enumerate() {
        c.hit_index
            .register(*id, BitmaskGrid32::cell_rect(c.x, y, c.w, cell_h, bit));
    }
    let store = c.store;
    paint_bitmask_grid32_on_card(
        &grid,
        c.x,
        y,
        c.w,
        cell_h,
        c.scene,
        c.text_system,
        c.ink,
        &|bit| store.hover_live(g::LAYERS[bit]) > 0.0,
    );
    y + BitmaskGrid32::grid_height(cell_h)
}

/// A escolha do Slice: a mesma pílula do Quality, em linhas quando a coluna não a leva numa só.
fn fatia(c: &mut CardCtx<'_>, ch: &Choices, y: f32) -> f32 {
    let modos = segs(g::SLICE, ch, c.store);
    corpo::choices(
        c.scene,
        c.text_system,
        c.store,
        c.hit_index,
        c.ink,
        c.x,
        c.w,
        y,
        tr("chrome.sample.slice_mode"),
        &g::SLICE,
        &modos,
    )
}

/// O título de um grupo e, à direita, o «+» dele (o idioma do Figma: a lista cresce do cabeçalho).
fn grupo(c: &mut CardCtx<'_>, ch: &Choices, text: &str, add: Option<NodeId>, y: f32) -> f32 {
    let (x0, w0) = (c.x, c.w);
    let fundo = if let Some(id) = add {
        let mais = [seg_of(id, ch, c.store)];
        c.x = x0 + w0 - m::FIELD_H;
        c.w = m::FIELD_H;
        let f = c.row(&[Cell::new(Widget::Bare(&mais))], y);
        c.x = x0;
        c.w = w0;
        f
    } else {
        y + m::FIELD_ROW_GAP + m::FIELD_H
    };
    let ty = fundo - m::FIELD_H + (m::FIELD_H - m::LABEL_PX) * 0.5;
    card_field::paint_label(
        c.scene,
        c.text_system,
        c.ink,
        c.x,
        ty,
        c.w - m::FIELD_H - m::FIELD_GAP,
        text,
    );
    fundo
}

/// Duas caixas e um ícone cabem numa linha, cada caixa com pelo menos `caixa`? Senão partem-se (à
/// Figma: nada se corta na coluna estreita, censo `nenhum_rotulo`).
fn cabem_lado_a_lado(c: &CardCtx<'_>, caixa: f32) -> bool {
    c.w >= caixa * 2.0 + m::FIELD_H + m::FIELD_GAP * 2.0
}

/// A largura de uma caixa de texto para mostrar `text` inteiro.
fn caixa_de_texto(c: &mut CardCtx<'_>, text: &str) -> f32 {
    c.text_system.prefix_width(text, m::VALUE_PX).ceil() + m::FIELD_PAD_R * 2.0
}

/// Uma entrada: o nome por cima; o tipo e o valor lado a lado; o «−» no fim. Na coluna que não leva
/// os dois, o valor desce para a linha de baixo, a toda a largura.
fn meta_row(c: &mut CardCtx<'_>, ch: &Choices, i: usize, y: f32) -> f32 {
    let r = g::META[i];
    let tipos = segs(r.options, ch, c.store);
    let tipo = Cell::new(dropdown(r.kind, &tipos)).label(tr(META_KEYS[i]));
    let menos = segs([r.remove], ch, c.store);
    let menos = Cell::new(Widget::Bare(&menos));
    let rgba = crate::valores::rgba(ch, META_SWATCH + i, c.ink);
    let hex = format!("{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2]);
    let (valor, precisa) = match ch.pick(META_TYPES[i]) {
        0 => (
            field(r.number, letra(tr("panel.widget_gallery.adv.type.float"))),
            CAMPO_MIN,
        ),
        1 => (
            Widget::Text {
                id: r.text,
                placeholder: tr("panel.widget_gallery.adv.text_default"),
            },
            caixa_de_texto(c, tr("panel.widget_gallery.adv.label_default")),
        ),
        _ => (
            Widget::Swatch {
                id: r.swatch,
                rgba: Some(rgba),
                text: &hex,
                edit: None,
                detach: None,
            },
            caixa_para(c, &hex),
        ),
    };
    let chip = tipos
        .iter()
        .map(|s| caixa_para(c, s.text))
        .fold(precisa, f32::max);
    if cabem_lado_a_lado(c, chip) {
        return c.row(&[tipo, Cell::new(valor), menos], y);
    }
    let y = c.row(&[tipo, menos], y);
    c.row(&[Cell::new(valor)], y)
}

fn metadata(c: &mut CardCtx<'_>, ch: &Choices, y: f32) -> f32 {
    let cheia = META_ROWS.iter().all(|f| ch.on(*f));
    let add = (!cheia).then_some(g::META_ADD);
    let mut y = grupo(c, ch, tr("chrome.sample.metadata"), add, y);
    for (i, f) in META_ROWS.iter().enumerate() {
        if ch.on(*f) {
            y = meta_row(c, ch, i, y);
        }
    }
    y
}

/// Chave, valor e o lixo, numa linha só.
fn propriedades(c: &mut CardCtx<'_>, ch: &Choices, y: f32) -> f32 {
    let cheia = PROP_ROWS.iter().all(|f| ch.on(*f));
    let add = (!cheia).then_some(g::PROP_ADD);
    let mut y = grupo(c, ch, tr("chrome.sample.custom_properties"), add, y);
    // O nome de fábrica mede a linha: a digitar, a linha não salta.
    let caixa = caixa_de_texto(c, tr("panel.widget_gallery.adv.prop_key_default"));
    for (i, f) in PROP_ROWS.iter().enumerate() {
        if ch.on(*f) {
            let r = g::PROPS[i];
            let lixo = segs([r.remove], ch, c.store);
            let lixo = Cell::new(Widget::Bare(&lixo));
            let chave = Cell::new(Widget::Text {
                id: r.key,
                placeholder: tr("panel.widget_gallery.adv.prop_new_key"),
            });
            let valor = Cell::new(Widget::Text {
                id: r.value,
                placeholder: tr("panel.widget_gallery.adv.prop_new_value"),
            });
            y = if cabem_lado_a_lado(c, caixa) {
                c.row(&[chave, valor, lixo], y)
            } else {
                let y = c.row(&[chave, lixo], y);
                c.row(&[valor], y)
            };
        }
    }
    y
}

pub(crate) fn advanced(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_ADVANCED,
        tr("panel.widget_gallery.card.advanced"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let y = bounds(c, y);
            let y = c.row(
                &[num!(
                    g::ADV_ROTATION,
                    IconId::CardAngle,
                    tr("chrome.sample.rotation")
                )],
                y,
            );
            let y = camadas(c, &ch, y);
            let y = fatia(c, &ch, y);
            let y = metadata(c, &ch, y);
            propriedades(c, &ch, y)
        }),
    )
}
