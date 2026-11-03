//! ⭐ **A lista «Color styles» REAL (`panel::cards::estilos`) sobre estilos fictícios** (doc
//! `CartoesUI/08` §1.1): o ⠿ do Fill/Stroke abre-a pelo despacho dos cartões; a galeria faz o papel
//! da cena do Vetor: publica a vista enquanto a lista dela está aberta e atende os pedidos.

use crate::controlos::SWATCHES;
use crate::ids as g;
use crate::state::{FILLS, Flag, STROKES, WidgetGalleryState};
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::estilos::{self, Estilo, Pedido};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_tokens::{ColorToken, Theme};

/// A amostra de partida de cada cartão com ⠿ (a 1.ª do Fill, a 1.ª do Stroke).
const STROKE_BASE: usize = 3;

const FABRICA: [(&str, ColorToken); 4] = [
    ("panel.widget_gallery.style.brand", ColorToken::Accent),
    ("panel.widget_gallery.style.alert", ColorToken::Danger),
    ("panel.widget_gallery.style.done", ColorToken::Success),
    ("panel.widget_gallery.style.ink", ColorToken::Text1),
];

/// A primeira linha presente de uma lista (a camada a que a lista de estilos se aplica).
fn primeira(flags: [Flag; 3], ch: &crate::state::Choices) -> usize {
    flags.iter().position(|f| ch.on(*f)).unwrap_or(0)
}

fn atende(state: &mut WidgetGalleryState, p: Pedido) {
    let ch = &mut state.choices;
    let cor_de = |v: &[Estilo], id: u32| v.iter().find(|e| e.id == id).map(|e| e.rgba);
    let proximo = |v: &[Estilo]| v.iter().map(|e| e.id).max().unwrap_or(0) + 1;
    match p {
        Pedido::Aplicar { estilo, camada } => {
            ch.linked[camada] = Some(estilo);
            ch.colors[camada] = cor_de(&state.estilos, estilo);
        }
        Pedido::Novo { camada, nome } => {
            let id = proximo(&state.estilos);
            let rgba = ch.colors[camada].unwrap_or_else(|| {
                let ink = CardInk::of_list(Theme::default());
                ink.color(SWATCHES[camada].2).to_rgba8().to_u8_array()
            });
            state.estilos.push(Estilo {
                id,
                name: nome,
                rgba,
                usos: 0,
            });
            ch.linked[camada] = Some(id);
            ch.colors[camada] = Some(rgba);
        }
        Pedido::Renomear { estilo, nome } => {
            if let Some(e) = state.estilos.iter_mut().find(|e| e.id == estilo) {
                e.name = nome;
            }
        }
        Pedido::Duplicar { estilo, nome } => {
            let id = proximo(&state.estilos);
            if let Some(rgba) = cor_de(&state.estilos, estilo) {
                state.estilos.push(Estilo {
                    id,
                    name: nome,
                    rgba,
                    usos: 0,
                });
            }
        }
        Pedido::Apagar { estilo } => {
            state.estilos.retain(|e| e.id != estilo);
            for l in &mut ch.linked {
                if *l == Some(estilo) {
                    *l = None;
                }
            }
        }
        Pedido::Cor { estilo, rgba } => {
            if let Some(e) = state.estilos.iter_mut().find(|e| e.id == estilo) {
                e.rgba = rgba;
            }
            for (l, c) in ch.linked.iter().zip(&mut ch.colors) {
                if *l == Some(estilo) {
                    *c = Some(rgba);
                }
            }
        }
        Pedido::Soltar { camada } => ch.linked[camada] = None,
    }
}

/// Por quadro, antes de pintar.
pub(crate) fn sync(state: &mut WidgetGalleryState, store: &WidgetStore, theme: Theme) {
    if state.estilos.is_empty() && !state.styles_seeded {
        state.styles_seeded = true;
        let ink = CardInk::of_card(store.section_theme(g::CARD_FILL), theme);
        for (i, (key, token)) in FABRICA.iter().enumerate() {
            state.estilos.push(Estilo {
                id: i as u32 + 1,
                name: ph2d_i18n::tr(key).to_owned(),
                rgba: ink.color(*token).to_rgba8().to_u8_array(),
                usos: 0,
            });
        }
    }
    estilos::declare(g::CARD_FILL, primeira(FILLS, &state.choices));
    estilos::declare(
        g::CARD_STROKE,
        STROKE_BASE + primeira(STROKES, &state.choices),
    );
    let aberta = estilos::aberta_em(g::CARD_FILL) || estilos::aberta_em(g::CARD_STROKE);
    // Os pedidos são da galeria se a lista era dela; o «soltar» de uma amostra ligada também.
    for p in estilos::take_pedidos() {
        let nosso = state.styles_open
            || aberta
            || matches!(&p, Pedido::Soltar { camada }
                if state.choices.linked.get(*camada).is_some_and(Option::is_some));
        if nosso {
            atende(state, p);
        } else {
            estilos::pede(p);
        }
    }
    if aberta {
        for e in &mut state.estilos {
            e.usos = state
                .choices
                .linked
                .iter()
                .filter(|l| **l == Some(e.id))
                .count();
        }
        estilos::publica(state.estilos.clone(), state.choices.linked.to_vec());
    }
    state.styles_open = aberta;
}
