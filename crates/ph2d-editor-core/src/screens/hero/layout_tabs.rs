//! ⭐⭐⭐ **AS ABAS DE LAYOUT** — o selector da decisão **D7**, na barra de cima (**D3**).
//!
//! > | eixo | onde vive |
//! > |---|---|
//! > | **Layout** | *barra de cima (abas)* |
//!
//! # ⭐ Elas ocupam o espaço que já estava vazio
//!
//! A barra de menus tem cinco títulos à esquerda (`File Edit View Window Run`, ~250 px) e **1100 px
//! de nada** à direita. As abas vão para lá, **encostadas à direita** — que é onde o Blender põe as
//! dele (*workspace tabs*, na topbar) e onde a mão as procura.
//!
//! ⛔ **Não é uma segunda faixa.** Uma faixa própria custaria mais 28 px de altura permanente ao
//! alvo de 1024 pontos, e a razão de a barra de menus existir foi precisamente não empilhar faixas
//! (ver `menu_bar`, sobre a `F9`).
//!
//! ⚠️ **O id de cada aba é DERIVADO do layout**, pela mesma lei do `slot_tabs::tab_node_id`: uma
//! constante por aba seria uma segunda lista a envelhecer ao lado de [`TaskLayout::ALL`].

use super::HeroScreen;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text_centered, resolve};
use crate::screens::task_layout::TaskLayout;
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// ⛔ **O salto que separa o id de uma ABA DE LAYOUT de tudo o resto.** Ver `slot_tabs::TAB_ID_SALT`
/// — XOR com uma constante é uma bijecção, logo o derivado não pode criar colisão nova.
const LAYOUT_TAB_SALT: u64 = 0x1a_1007_7ab5_0002;

/// O id do controlo desta aba.
#[must_use]
pub fn tab_node_id(layout: TaskLayout) -> NodeId {
    // O índice na tabela, não o `Debug`: um nome muda, uma posição na lista da D7 não.
    let i = TaskLayout::ALL
        .iter()
        .position(|l| *l == layout)
        .unwrap_or(0) as u64;
    NodeId(LAYOUT_TAB_SALT ^ (i + 1))
}

/// De que layout é esta aba?
#[must_use]
pub fn layout_for_tab(id: NodeId) -> Option<TaskLayout> {
    TaskLayout::ALL.into_iter().find(|l| tab_node_id(*l) == id)
}

/// Padding horizontal de cada aba. `fn` e não `const` porque `Spacing::px` não é `const fn`.
fn tab_pad_x() -> f32 {
    Spacing::Md.px()
}

/// ⭐ **A ÚNICA porta da geometria** — o pintor, o registo de hit e o despacho leem daqui.
///
/// As abas são encostadas à **direita** da barra; a largura de cada uma é a do próprio título, como
/// nos menus. ⚠️ Devolve vazio se elas não couberem sem tocar nos títulos dos menus: *uma aba por
/// cima de um menu é um clique que troca de tarefa quando o artista queria abrir o ficheiro.*
#[must_use]
pub fn tab_rects(
    bar: Rect,
    menus_end_x: f32,
    text_system: &mut TextSystem,
) -> Vec<(TaskLayout, Rect)> {
    let font = TypeToken::Sm.px();
    let widths: Vec<f32> = TaskLayout::ALL
        .iter()
        .map(|l| text_system.prefix_width(l.spec().title, font) + tab_pad_x() * 2.0)
        .collect();
    let total: f32 = widths.iter().sum();
    let mut x = bar.x + bar.w - total - Spacing::Sm.px();
    if x < menus_end_x {
        return Vec::new();
    }
    TaskLayout::ALL
        .into_iter()
        .zip(widths)
        .map(|(l, w)| {
            let r = Rect::new(x, bar.y, w, bar.h);
            x += w;
            (l, r)
        })
        .collect()
}

/// Regista os controlos das abas. Chamado pelo `pre_populate` do hero.
///
/// ⚠️ Sem `InteractiveState` uma aba é pintada e nasce **morta** — o Down não arma o `active` e o Up
/// nunca emite `Click`. É o defeito que matou o pill `[SHEET]` e os quatro pills de vetor.
pub fn populate(store: &mut WidgetStore) {
    for l in TaskLayout::ALL {
        store.register(
            tab_node_id(l),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// Pinta as abas e regista os alvos.
#[allow(clippy::too_many_arguments)]
pub fn paint(
    bar: Rect,
    menus_end_x: f32,
    active: TaskLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    for (l, r) in tab_rects(bar, menus_end_x, text_system) {
        let is_on = l == active;
        let state = store
            .button_state(tab_node_id(l))
            .unwrap_or(ButtonState::Normal);
        // ⚠️ **A aba ACTIVA fica fora do eixo do relógio**, pela mesma lei do chip activo do trilho:
        // *escolhida* não é uma quantidade — é o estado que diz *«a tela é esta»*.
        let bg = if is_on {
            Some(ColorToken::AccentSoft)
        } else if matches!(
            state,
            ButtonState::Hovered | ButtonState::Focused | ButtonState::Pressed
        ) {
            Some(ColorToken::BgElev)
        } else {
            None
        };
        if let Some(bg) = bg {
            fill_rounded_rect(
                scene,
                r,
                crate::paint::frame_radius(theme, Radius::Sm.px()),
                resolve(bg, theme),
            );
        }
        let fg = if is_on {
            ColorToken::Accent
        } else {
            ColorToken::Text2
        };
        paint_text_centered(
            text_system,
            scene,
            l.spec().title,
            r,
            TypeToken::Sm.px(),
            resolve(fg, theme),
        );
        hit_index.register(tab_node_id(l), r);
    }
}

/// ⭐ **Clicar numa aba arruma a tela** — e é tudo o que uma aba faz.
///
/// Corre no pré-despacho, pela razão do `slot_tabs`: o registo de painéis é caminhado antes do
/// `chrome::dispatch_all`, e um id derivado nunca chegaria a um handler de chrome.
pub fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    let Some(layout) = layout_for_tab(id) else {
        return false;
    };
    // ⚠️ **Clicar na aba ACTIVA não é um no-op silencioso: ela RE-ARRUMA.** É o gesto de *«devolve
    // esta tarefa ao que ela era»* que todo editor com workspaces tem, e sem ele o artista que
    // desarrumou não tem como voltar sem passar por outra aba.
    super::layout_switch::apply(hero, layout);
    true
}

#[cfg(test)]
#[path = "layout_tabs_tests.rs"]
mod tests;
