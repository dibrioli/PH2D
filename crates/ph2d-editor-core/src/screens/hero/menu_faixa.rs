//! ⭐ **A FAIXA QUE RESPONDE** (versão C, dono 12/11): o botão «Menu» à esquerda (os cinco menus em
//! cascata), o campo «Find anything» no centro (abre a busca do `Ctrl+K`) e o jogo à direita. As
//! abas de documento ficam com o vão entre o «Menu» e o campo.
//!
//! [`faixa`] é a porta da geometria — o pintor, o registo de alvos e os testes perguntam a ela.

use crate::icons::IconId;
use crate::ids;
use crate::interaction::{ContextMenuKind, HitIndex, WidgetStore};
use crate::paint::{
    fill_rounded_rect, icon_glyph_side, paint_icon, paint_text, paint_text_centered,
    rect_for_label, resolve,
};
use crate::widget::ButtonState;
use crate::widget::cromo_barra::Barra;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, INLINE_ICON_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

use super::context_menu_overlay::MENU_W;

/// O campo nunca passa de dois menus de largura; abaixo de um, vira a lupa.
fn campo_max_w() -> f32 {
    MENU_W * 2.0
}

/// A geometria da faixa.
#[derive(Clone, Debug)]
pub struct Faixa {
    /// O botão «Menu».
    pub menu: Rect,
    /// O campo de busca — ou a lupa, se [`Self::lupa`].
    pub find: Rect,
    /// A janela é estreita: o campo encolheu para um botão de lupa.
    pub lupa: bool,
    /// A fila do jogo ([`super::transport_bar::rects`]).
    pub jogo: Vec<(NodeId, IconId, Rect)>,
    /// O vão das abas de documento (de, até).
    pub docs: (f32, f32),
    /// Os botões da janela, encostados à direita (vazio com a barra do sistema).
    pub janela: Vec<(NodeId, IconId, Rect)>,
}

/// ⭐ **A PORTA.** «Menu» encostado à esquerda, o jogo à direita; o campo centra-se na barra e
/// nunca toma mais de metade do vão livre (a outra metade é das abas).
#[must_use]
pub fn faixa(bar: Rect, text_system: &mut TextSystem, frame: Option<bool>) -> Faixa {
    let font = TypeToken::Sm.px();
    let rotulo = text_system.prefix_width(tr("chrome.menu_bar.menu"), font);
    let menu_w = Spacing::Sm.px() + icon_glyph_side(bar.h) + rect_for_label(rotulo);
    let menu = Rect::new(bar.x + Spacing::Sm.px(), bar.y, menu_w, bar.h);
    let janela = super::window_buttons::rects(bar, frame);
    let fim = super::window_buttons::start_x(bar, &janela);
    let play_w = super::transport_bar::play_width(text_system, bar.h);
    let jogo = super::transport_bar::rects(bar, menu.x + menu.w, fim, play_w);
    let gap = super::document_tabs::gap_px();
    let de = menu.x + menu.w + gap;
    let ate = jogo.first().map_or(fim, |(_, _, r)| r.x) - gap;
    let w = ((ate - de) * 0.5).min(campo_max_w());
    let (find, lupa) = if w >= MENU_W {
        let x = (bar.x + (bar.w - w) * 0.5).min(ate - w);
        (Rect::new(x, bar.y, w, bar.h), false)
    } else {
        let lado = bar.h.min((ate - de).max(0.0));
        (Rect::new(ate - lado, bar.y, lado, bar.h), true)
    };
    Faixa {
        menu,
        find,
        lupa,
        jogo,
        docs: (de, find.x - gap),
        janela,
    }
}

/// A caixa de um controlo dentro da faixa: recuada do topo e do fundo como a pílula do jogo.
pub(super) fn caixa(r: Rect) -> Rect {
    let d = m::FIELD_BORDER;
    Rect::new(r.x, r.y + d, r.w, (r.h - d * 2.0).max(0.0))
}

/// ⭐ **A caixa do nome de um botão «glifo + nome»** (o «Menu», o Play), do fim do glifo (`de`) à
/// borda direita de `r`. A largura é a que a geometria somou ([`rect_for_label`]), refeita pela
/// porta: `(x + w) − x` fica ULPs abaixo dela e a elisão (`<=`) cortava o nome («Me…», «Pl…»).
pub(super) fn caixa_do_nome(text_system: &mut TextSystem, nome: &str, r: Rect, de: f32) -> Rect {
    let resto = (r.x + r.w - de).max(0.0);
    let medido = rect_for_label(text_system.prefix_width(nome, TypeToken::Sm.px()));
    // A menos de um filete é a mesma caixa; mais longe, o botão não foi medido por este nome.
    let w = if (medido - resto).abs() < StrokeToken::Thin.px() {
        medido
    } else {
        resto
    };
    Rect::new(r.x + r.w - w, r.y, w, r.h)
}

fn pairado(store: &WidgetStore, id: NodeId) -> bool {
    matches!(
        store.button_state(id),
        Some(ButtonState::Hovered | ButtonState::Focused | ButtonState::Pressed)
    )
}

/// Pinta o «Menu» e o campo e regista os alvos deles (o jogo e as abas pintam-se nos donos).
pub fn paint(
    f: &Faixa,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    motion: &crate::motion::UiMotion,
) {
    let barra = Barra::de(theme);
    let cor = |t: ColorToken| barra.map_or_else(|| resolve(t, theme), |b| b.tinta().color(t));
    let raio = crate::paint::frame_radius(theme, Radius::Sm.px());
    let font = TypeToken::Sm.px();

    // ── «Menu» ── aceso enquanto a cascata dele está aberta: diz de onde os menus saíram.
    let aberto = store.menu_chain().first().map(|r| r.kind) == Some(ContextMenuKind::MenuBarRoot);
    let hover = pairado(store, ids::MENUBAR_W);
    let tinta = match barra {
        Some(b) => b.chip(scene, caixa(f.menu), aberto, hover),
        None => {
            if aberto || hover {
                let fundo = if aberto {
                    ColorToken::AccentSoft
                } else {
                    ColorToken::BgElev
                };
                fill_rounded_rect(scene, f.menu, raio, resolve(fundo, theme));
            }
            // O título lê o relógio da UI (Expressivo/Discreto); aberto fica fora do eixo.
            let estado = store
                .button_state(ids::MENUBAR_W)
                .unwrap_or(ButtonState::Normal);
            let t = crate::widget::chip_axis_t(estado, aberto, motion.get(ids::MENUBAR_W));
            let fg = if aberto {
                ColorToken::Accent
            } else {
                ColorToken::Text1
            };
            crate::widget::chip_axis_color(t, ColorToken::Text2, ColorToken::Text1, fg, theme)
        }
    };
    let lado = icon_glyph_side(f.menu.h);
    let gx = f.menu.x + Spacing::Sm.px();
    let glifo = Rect::new(gx, f.menu.y + (f.menu.h - lado) * 0.5, lado, lado);
    paint_icon(scene, IconId::Menu, glifo, tinta, StrokeToken::Default.px());
    let rotulo = tr("chrome.menu_bar.menu");
    let nome = caixa_do_nome(text_system, rotulo, f.menu, gx + lado);
    paint_text_centered(text_system, scene, rotulo, nome, font, tinta);
    hit_index.register(ids::MENUBAR_W, f.menu);

    // ── «Find anything» ── um campo que é um botão: clicar abre a busca.
    if f.find.w <= 0.0 {
        return;
    }
    let hover = pairado(store, ids::MENUBAR_FIND);
    let cx = caixa(f.find);
    match barra {
        Some(b) => b.campo(scene, cx, false, hover),
        None => {
            fill_rounded_rect(scene, cx, raio, resolve(ColorToken::BgElev, theme));
        }
    }
    let apagado = cor(if hover {
        ColorToken::Text2
    } else {
        ColorToken::Text3
    });
    let lupa = INLINE_ICON_PX;
    let pad = Spacing::Md.px();
    let lx = if f.lupa {
        cx.x + (cx.w - lupa) * 0.5
    } else {
        cx.x + pad
    };
    let glifo = Rect::new(lx, cx.y + (cx.h - lupa) * 0.5, lupa, lupa);
    paint_icon(
        scene,
        IconId::Search,
        glifo,
        apagado,
        StrokeToken::Default.px(),
    );
    hit_index.register(ids::MENUBAR_FIND, f.find);
    if f.lupa {
        return;
    }
    let ty = cx.y + (cx.h - font) * 0.5;
    let tecla = tr("chrome.menu_bar.find_key");
    let tw = text_system.prefix_width(tecla, font);
    let tx = cx.x + cx.w - pad - tw;
    paint_text(
        text_system,
        scene,
        tecla,
        tx,
        ty,
        font,
        tw,
        cor(ColorToken::Text3),
    );
    let x = lx + lupa + ph2d_tokens::icon_label_gap_px();
    let max_w = (tx - Spacing::Md.px() - x).max(0.0);
    paint_text(
        text_system,
        scene,
        tr("chrome.menu_bar.find"),
        x,
        ty,
        font,
        max_w,
        apagado,
    );
}

#[cfg(test)]
#[path = "menu_faixa_tests.rs"]
mod tests;
