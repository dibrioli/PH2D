//! ⭐⭐ **A BARRA DE MENUS** — *File · Edit · View · Window*, a barra global da **D2**.
//!
//! Enio, 2026-08-30: *«Pode tirar também os botões do topo para começarmos a trabalhar a barra
//! superior.»* Os 29 pills saíram na wave anterior; esta é o que fica no lugar deles.
//!
//! # A lei desta barra: ela REALOJA, não constrói
//!
//! ⚠️ **Quase toda linha destes quatro menus leva um id que já existia.** O *Save* é o
//! `CTX_MENU_SAVE` do `chrome::io_menu`; o *Vector* é o `TOPBAR_VECTOR` que o pill levava, e quem
//! o despacha continua a ser o painel do vetor. É a **D2** ao pé da letra
//! (`docs/UI_New_and_Simple/00_DECISOES_DO_ENIO.md`): *existe um sítio canónico para cada comando*
//! — e um comando com dois ids tem dois sítios a apodrecer em separado.
//!
//! ⇒ nasceram **quatro** ids de linha, e só onde não havia verbo nenhum a alcançar: a régua, o
//! tema como categoria, as preferências, e a imagem nova (que só a tecla abria).
//!
//! # ⭐ Uma TABELA, três consumidores
//!
//! [`MENUS`] é a fonte, e [`menu_rects`] é a porta:
//!
//! | quem | o que pergunta |
//! |---|---|
//! | [`paint_menu_bar`] | onde desenho cada título |
//! | [`paint_menu_bar`] (hit) | que rectângulo registo para cada título |
//! | `interaction::dispatch::pointer_down_menus` | que menu abre este clique |
//!
//! ⛔ **Foi assim de propósito.** O trilho lateral tem a mesma aritmética escrita **duas vezes**
//! (o pintor em `widget/tool_rail/paint.rs`, o registo de hit em `left_rail.rs`) e nada no repo
//! liga as duas — um pintor horizontal com um hit vertical compilaria e passaria a suíte inteira.
//! Uma barra nova não repete isso.
//!
//! # ⚠️ A banda é a MESMA do chrome legado, e é uma escolha
//!
//! A barra ocupa `ChromeBands::top_bar_h`, que é a faixa que a barra de pills ocupava — logo a
//! `F9` **troca** as duas em vez de as empilhar. Duas faixas empilhadas custariam altura
//! permanente ao alvo de 1024 pontos por causa de um interruptor de bissecção.
//!
//! ⚠️ **E ela SUBTRAI altura, nunca flutua** (spec §4): a área de desenho começa por baixo dela,
//! como as colunas começam ao lado do trilho. *Uma barra global a flutuar reproduziria, num modelo
//! novo, o defeito da régua tapada que a wave anterior curou.*

use super::HeroLayout;
use super::ids;
use crate::interaction::{ContextMenuKind, HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text_centered, rect_to_vello, resolve};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A altura da barra — **uma linha de menu**, que é o que um título é.
///
/// ⚠️ Deriva do mesmo token que a linha de um menu aberto (`ROW_H_PX`), e não de um número
/// escolhido: o título e as linhas que ele abre são a mesma família de alvo.
pub const MENU_BAR_H: f32 = ROW_H_PX;

/// ⭐ **A TABELA** — título, id, e o menu que ele abre. A fonte única desta barra.
pub const MENUS: [(NodeId, &str, ContextMenuKind); 4] = [
    (ids::MENUBAR_FILE, "File", ContextMenuKind::MenuBarFile),
    (ids::MENUBAR_EDIT, "Edit", ContextMenuKind::MenuBarEdit),
    (ids::MENUBAR_VIEW, "View", ContextMenuKind::MenuBarView),
    (
        ids::MENUBAR_WINDOW,
        "Window",
        ContextMenuKind::MenuBarWindow,
    ),
];

/// Os ids de linha que **esta** barra trouxe — os que nenhum outro botão do app alcançava.
///
/// ⚠️ Existe para o registo não ser uma lista escrita à mão ao lado da tabela de rows: quem
/// acrescentar um verbo novo a um destes menus acrescenta-o aqui, e o gate
/// `every_menu_bar_row_is_registered` reprova se esquecer.
pub const OWN_ROWS: [NodeId; 4] = [
    ids::MENUBAR_FILE_NEW,
    ids::MENUBAR_EDIT_PREFERENCES,
    ids::MENUBAR_VIEW_THEME,
    ids::MENUBAR_VIEW_RULERS,
];

/// Padding horizontal de cada título dentro do seu alvo.
///
/// ⚠️ `fn` e não `const` porque `Spacing::px` não é `const fn` — a mesma razão por que
/// `widget::tool_rail_width_px` é função. O valor continua a ser o TOKEN, que é o que importa.
fn title_pad_x() -> f32 {
    Spacing::Md.px()
}

/// Recuo da barra à esquerda, para o primeiro título não nascer colado à borda da janela.
fn bar_inset_x() -> f32 {
    Spacing::Sm.px()
}

/// Regista os quatro títulos e as quatro linhas próprias. Chamado por
/// [`super::pre_populate::populate_shared`].
///
/// ⚠️ **Sem `InteractiveState` um item de menu é pintado e nasce morto**: ele não é focável, o
/// Down não arma o `active` e o Up nunca emite `Click`. É o mesmo defeito que matou o pill
/// `[SHEET]` e os quatro pills de vetor.
pub fn populate(store: &mut WidgetStore) {
    for (id, ..) in MENUS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    for id in OWN_ROWS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// ⭐ **A PORTA** — onde cada título fica, medido contra o texto real.
///
/// ⚠️ A largura sai do `prefix_width`, não de um número por título: *Window* é mais largo que
/// *File*, e alvos de largura fixa dariam ou um buraco ou um recorte conforme a fonte.
#[must_use]
pub fn menu_rects(
    bar: Rect,
    text_system: &mut TextSystem,
) -> [(NodeId, &'static str, Rect); MENUS.len()] {
    let font = TypeToken::Sm.px();
    let mut x = bar.x + bar_inset_x();
    std::array::from_fn(|i| {
        let (id, title, _) = MENUS[i];
        let w = text_system.prefix_width(title, font) + title_pad_x() * 2.0;
        let r = Rect::new(x, bar.y, w, bar.h);
        x += w;
        (id, title, r)
    })
}

/// Desenha a barra e regista os alvos dos títulos.
///
/// ⚠️ O título cujo menu está **aberto** fica realçado — sem isso, um menu aberto não diz de onde
/// saiu, e o segundo clique (o que o fecha) parece não ter alvo.
pub fn paint_menu_bar(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let bar = layout.top_bar;
    if bar.w <= 0.0 || bar.h <= 0.0 {
        return;
    }
    scene.fill_rect(rect_to_vello(bar), resolve(ColorToken::Bg1, theme));
    let open = store.context_menu().map(|r| r.kind);
    for (id, title, r) in menu_rects(bar, text_system) {
        let state = store.button_state(id).unwrap_or(ButtonState::Normal);
        let is_open = open.is_some_and(|k| MENUS.iter().any(|(mid, _, mk)| *mid == id && *mk == k));
        let bg = match state {
            _ if is_open => Some(ColorToken::AccentSoft),
            ButtonState::Hovered | ButtonState::Focused | ButtonState::Pressed => {
                Some(ColorToken::BgElev)
            }
            _ => None,
        };
        if let Some(bg) = bg {
            fill_rounded_rect(scene, r, Radius::Sm.px(), resolve(bg, theme));
        }
        let fg = if is_open {
            ColorToken::Accent
        } else {
            ColorToken::Text1
        };
        paint_text_centered(
            text_system,
            scene,
            title,
            r,
            TypeToken::Sm.px(),
            resolve(fg, theme),
        );
        hit_index.register(id, r);
    }
}

/// ⛔⛔ **O clique numa linha destes menus FECHA o menu antes de qualquer um agir** — e tem de
/// correr aqui, no topo do [`super::HeroScreen::apply_event`], porque o registo de painéis é
/// caminhado **antes** do `chrome::dispatch_all`: um `Click(TOPBAR_AUDIO_MIXER)` é consumido pelo
/// painel do mixer e o chrome nunca o vê. Um fecho escrito num handler de chrome ficaria morto
/// exactamente nas treze linhas do menu *Window*.
///
/// ⚠️ **As duas linhas que ABREM outro menu ficam de fora**, e não por precaução: elas fecham e
/// reabrem por si (`chrome::menu_bar`), e fechar aqui apagaria o `last_context_menu` de que o
/// `cascade_anchor` se serve.
pub fn close_on_row_click(hero: &mut super::HeroScreen, event: WidgetEvent) {
    let WidgetEvent::Click(id) = event else {
        return;
    };
    let Some(kind) = hero.store.context_menu().map(|r| r.kind) else {
        return;
    };
    if !MENUS.iter().any(|(_, _, k)| *k == kind) {
        return;
    }
    if id == ids::MENUBAR_EDIT_PREFERENCES || id == ids::MENUBAR_VIEW_THEME {
        return;
    }
    if super::menu_rows::menu_rows(kind)
        .iter()
        .any(|(rid, ..)| *rid == id)
    {
        hero.store.close_context_menu();
    }
}
