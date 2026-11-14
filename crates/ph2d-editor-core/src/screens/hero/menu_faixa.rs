//! ⭐ **A FAIXA QUE RESPONDE** (versão C, dono 12/11): o botão «Menu» à esquerda (os cinco menus em
//! cascata), o campo «Find anything» no centro (abre a busca do `Ctrl+K`) e o jogo à direita. As
//! abas de documento ficam com o vão entre o «Menu» e o campo.
//!
//! [`faixa`] é a porta da geometria — o pintor, o registo de alvos e os testes perguntam a ela.

use crate::icons::IconId;
use crate::ids;
use crate::interaction::{ContextMenuKind, HitIndex, WidgetStore};
use crate::paint::{
    fill_rounded_rect, folga_do_texto, paint_icon_nitido, paint_text, paint_text_centered,
    rect_for_label, resolve,
};
use crate::widget::ButtonState;
use crate::widget::cromo_barra::Barra;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_i18n::tr;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{
    BAR_ICON_PX, ColorToken, MENU_BAR_CTL_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken,
};
use ph2d_vector::VectorScene;

use super::context_menu_overlay::MENU_W;

/// A marca «PH2D» (dono 12/11, no lugar do W da maquete): o nome a negrito numa pílula.
fn marca_w(text_system: &mut TextSystem) -> f32 {
    let w = ph2d_text::com_peso_exato(|| {
        text_system.prefix_width_weighted(
            tr("chrome.menu_bar.brand"),
            TypeToken::Xs.px(),
            FontWeight::BOLD,
        )
    });
    w + Spacing::Sm.px() * 2.0
}

fn marca_h() -> f32 {
    TypeToken::Xs.px() + Spacing::Xs.px() * 2.0
}

/// O botão «Menu»: a marca, o nome e a seta que diz «abre».
fn menu_w(text_system: &mut TextSystem) -> f32 {
    let rotulo = text_system.prefix_width(tr("chrome.menu_bar.menu"), TypeToken::Sm.px());
    Spacing::Xs.px() * 2.0
        + marca_w(text_system)
        + rect_for_label(rotulo)
        + BAR_ICON_PX
        + Spacing::Xs.px()
}

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
    let menu = Rect::new(bar.x + Spacing::Xs.px(), bar.y, menu_w(text_system), bar.h);
    let janela = super::window_buttons::rects(bar, frame);
    let fim = super::window_buttons::start_x(bar, &janela);
    let play_w = super::transport_bar::play_width(text_system);
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

/// A caixa de um controlo dentro da faixa: `MENU_BAR_CTL_H_PX` de alto, centrada, nunca a tocar o topo
/// nem o fundo. TODO controlo da faixa tem esta altura (onda 7: eram quatro — 22, 24, 28 e 22).
pub(super) fn caixa(r: Rect) -> Rect {
    let d = m::FIELD_BORDER;
    let h = crate::math::safe_clamp(r.h - d * 2.0, 0.0, MENU_BAR_CTL_H_PX);
    Rect::new(r.x, r.y + (r.h - h) * 0.5, r.w, h)
}

/// O segmento DENTRO de uma pílula da faixa (a aba escolhida, o Play, o Pause): a [`caixa`] menos o
/// respiro do segmento de cada lado — a pílula e o controlo solto têm a mesma altura.
pub(super) fn segmento(r: Rect) -> Rect {
    let c = caixa(r);
    let d = m::SEG_PAD;
    Rect::new(
        c.x + d,
        c.y + d,
        (c.w - d * 2.0).max(0.0),
        (c.h - d * 2.0).max(0.0),
    )
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
    let raio = crate::paint::frame_radius(theme, Radius::Sm.px());
    let font = TypeToken::Sm.px();

    // ── «Menu» ── aceso enquanto a cascata dele está aberta: diz de onde os menus saíram.
    let tons = super::faixa_tons::tons(theme);
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
    // A marca «PH2D».
    let (mw, mh) = (marca_w(text_system), marca_h());
    let marca = Rect::new(
        f.menu.x + Spacing::Xs.px(),
        f.menu.y + (f.menu.h - mh) * 0.5,
        mw,
        mh,
    );
    fill_rounded_rect(scene, marca, mh * 0.5, tons.marca);
    let xs = TypeToken::Xs.px();
    let marca_nome = tr("chrome.menu_bar.brand");
    let nome_w = ph2d_text::com_peso_exato(|| {
        text_system.prefix_width_weighted(marca_nome, xs, FontWeight::BOLD)
    });
    let marca_y = marca.y + folga_do_texto(marca.h, xs);
    crate::paint::paint_text_elided_in_weight(
        text_system,
        scene,
        marca_nome,
        [marca.x + (marca.w - nome_w) * 0.5, marca_y],
        xs,
        marca.w,
        tons.marca_texto,
        FontWeight::BOLD,
    );
    // O nome «Menu» e a seta.
    let rotulo = tr("chrome.menu_bar.menu");
    let seta = BAR_ICON_PX;
    let fim_nome = f.menu.x + f.menu.w - seta - Spacing::Xs.px();
    let caixa_nome = Rect::new(
        marca.x + marca.w,
        f.menu.y,
        (fim_nome - marca.x - marca.w).max(0.0),
        f.menu.h,
    );
    paint_text_centered(text_system, scene, rotulo, caixa_nome, font, tinta);
    let glifo = Rect::new(fim_nome, f.menu.y + (f.menu.h - seta) * 0.5, seta, seta);
    paint_icon_nitido(scene, IconId::ChevronDown, glifo, tinta, false);
    hit_index.register(ids::MENUBAR_W, f.menu);

    // ── «Find anything» ── um campo que é um botão: clicar abre a busca.
    if f.find.w <= 0.0 {
        return;
    }
    let hover = pairado(store, ids::MENUBAR_FIND);
    let cx = caixa(f.find);
    let fino = StrokeToken::Thin.px();
    fill_rounded_rect(scene, cx, raio, tons.campo);
    let meio = fino * 0.5;
    let traco = Rect::new(cx.x + meio, cx.y + meio, cx.w - fino, cx.h - fino);
    // FRAME-RAW-OK: o filete É o que identifica o campo de busca contra a faixa (`faixa_tons`, CONTROLO_MIN).
    crate::paint::stroke_rounded_rect(
        scene,
        traco,
        raio - meio,
        fino,
        if hover { tons.texto } else { tons.contorno },
    );
    let apagado = if hover { tons.texto } else { tons.apagado };
    let lupa = BAR_ICON_PX;
    let pad = Spacing::Md.px();
    let lx = if f.lupa {
        cx.x + (cx.w - lupa) * 0.5
    } else {
        cx.x + pad
    };
    let glifo = Rect::new(lx, cx.y + (cx.h - lupa) * 0.5, lupa, lupa);
    paint_icon_nitido(scene, IconId::Search, glifo, apagado, false);
    hit_index.register(ids::MENUBAR_FIND, f.find);
    if f.lupa {
        return;
    }
    // A tecla numa plaquinha, encostada à direita.
    let tecla = tr("chrome.menu_bar.find_key");
    let tw = text_system.prefix_width(tecla, xs);
    let (ph, pw) = (xs + Spacing::Xxs.px() * 2.0, tw + Spacing::Sm.px() * 2.0);
    let placa = Rect::new(
        cx.x + cx.w - Spacing::Xs.px() - pw,
        cx.y + (cx.h - ph) * 0.5,
        pw,
        ph,
    );
    let raio_placa = crate::paint::frame_radius(theme, Radius::Sm.px()).min(ph * 0.5);
    fill_rounded_rect(scene, placa, raio_placa, tons.tecla);
    let tecla_y = placa.y + folga_do_texto(placa.h, xs);
    paint_text(
        text_system,
        scene,
        tecla,
        placa.x + Spacing::Sm.px(),
        tecla_y,
        xs,
        tw,
        tons.tecla_texto,
    );
    let convite = tr("chrome.menu_bar.find");
    let ty = cx.y + folga_do_texto(cx.h, font);
    let x = lx + lupa + ph2d_tokens::icon_label_gap_px();
    let max_w = (placa.x - Spacing::Md.px() - x).max(0.0);
    paint_text(text_system, scene, convite, x, ty, font, max_w, apagado);
}

#[cfg(test)]
#[path = "menu_faixa_tests.rs"]
mod tests;
