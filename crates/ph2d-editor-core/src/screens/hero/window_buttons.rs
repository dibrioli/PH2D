//! ⭐ **A FAIXA ÚNICA** (line/BarraDeMenus, onda 4): sem a barra de título do sistema, a faixa é o
//! topo da janela — minimizar, maximizar/restaurar e fechar à direita dela, o arrasto pelo fundo e
//! o redimensionar pelas bordas. Aqui mora a LEI (geometria, borda sob o ponteiro, duplo clique,
//! o pedido); a shell só a liga ao `winit` (`shells/desktop/src/faixa_unica.rs`).
//!
//! A shell diz que a janela é do app com [`WidgetStore::set_window_frame`] a cada quadro; sem isso
//! (barra do sistema) nenhum botão se pinta e a faixa fica como era.

use crate::icons::IconId;
use crate::ids;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, icon_glyph_side, paint_icon, resolve};
use crate::widget::ButtonState;
use crate::widget::cromo_barra::Barra;
use crate::zones::Rect;
use ph2d_a11y::NodeId;

pub use crate::interaction::state::window_frame::{
    Edge, PressAction, WindowPress, WindowRequest, edge_at, edge_band,
};
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// Largura de cada botão: a altura da faixa e um vão de cada lado (o glifo fica ao centro).
fn button_w(h: f32) -> f32 {
    h + Spacing::Md.px()
}

/// ⭐ **A PORTA da geometria.** Os três botões encostados à direita da faixa, pela ordem do sistema
/// (− □ ×); vazio quando a janela tem a barra do sistema (`frame == None`).
#[must_use]
pub fn rects(bar: Rect, frame: Option<bool>) -> Vec<(NodeId, IconId, Rect)> {
    let Some(maximized) = frame else {
        return Vec::new();
    };
    let w = button_w(bar.h);
    let max_icon = if maximized {
        IconId::WindowRestore
    } else {
        IconId::WindowMaximize
    };
    let botoes = [
        (ids::MENUBAR_WIN_MINIMIZE, IconId::WindowMinimize),
        (ids::MENUBAR_WIN_MAXIMIZE, max_icon),
        (ids::MENUBAR_WIN_CLOSE, IconId::Close),
    ];
    let x0 = bar.x + bar.w - w * botoes.len() as f32;
    botoes
        .iter()
        .enumerate()
        .map(|(i, (id, icon))| (*id, *icon, Rect::new(x0 + w * i as f32, bar.y, w, bar.h)))
        .collect()
}

/// Onde a fila começa (o resto da faixa acaba aqui), ou o fim da faixa sem botões.
#[must_use]
pub fn start_x(bar: Rect, botoes: &[(NodeId, IconId, Rect)]) -> f32 {
    botoes.first().map_or(bar.x + bar.w, |(.., r)| r.x)
}

/// Pinta os três e regista os alvos. O fechar acende em vermelho (`Danger`) sob o ponteiro, como em
/// todo o sistema; os outros dois têm o realce de um chip da faixa.
pub fn paint(
    botoes: &[(NodeId, IconId, Rect)],
    scene: &mut VectorScene,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let barra = Barra::de(theme);
    let cor = |t: ColorToken| barra.map_or_else(|| resolve(t, theme), |b| b.tinta().color(t));
    let raio = crate::paint::frame_radius(theme, Radius::Sm.px());
    for (id, icon, r) in botoes {
        let hover = matches!(
            store.button_state(*id).unwrap_or(ButtonState::Normal),
            ButtonState::Hovered | ButtonState::Focused | ButtonState::Pressed
        );
        let caixa = super::menu_faixa::caixa(*r);
        let tinta = if *id == ids::MENUBAR_WIN_CLOSE && hover {
            fill_rounded_rect(scene, caixa, raio, cor(ColorToken::Danger));
            cor(ColorToken::AccentFg)
        } else if let Some(b) = barra {
            b.chip(scene, caixa, false, hover)
        } else {
            if hover {
                fill_rounded_rect(scene, caixa, raio, resolve(ColorToken::BgElev, theme));
            }
            resolve(
                if hover {
                    ColorToken::Text1
                } else {
                    ColorToken::Text2
                },
                theme,
            )
        };
        let lado = icon_glyph_side(r.h);
        let glifo = Rect::new(
            r.x + (r.w - lado) * 0.5,
            r.y + (r.h - lado) * 0.5,
            lado,
            lado,
        );
        paint_icon(scene, *icon, glifo, tinta, StrokeToken::Default.px());
        hit_index.register(*id, *r);
    }
}

/// O clique num dos três vira o pedido à janela. `true` = consumido.
pub fn on_click(store: &mut WidgetStore, id: NodeId) -> bool {
    let pedido = match id {
        x if x == ids::MENUBAR_WIN_MINIMIZE => WindowRequest::Minimize,
        x if x == ids::MENUBAR_WIN_MAXIMIZE => WindowRequest::ToggleMaximize,
        x if x == ids::MENUBAR_WIN_CLOSE => WindowRequest::Close,
        _ => return false,
    };
    store.close_context_menu();
    store.request_window(pedido);
    true
}

/// ⭐ **A porta da shell para o Down primário** em `(x, y)` lógicos numa janela `size` lógica:
/// `None` com a barra do sistema ou quando o Down é do editor.
pub fn press(
    hero: &mut super::HeroScreen,
    x: f32,
    y: f32,
    size: (f32, f32),
    t_ns: u128,
) -> Option<PressAction> {
    let maximized = hero.store.window_frame()?;
    let hit = hero.hit_index.hit(x, y);
    hero.store
        .window_press_mut()
        .press(x, y, size, maximized, hit, t_ns)
}

/// O centro do maior trecho de fundo livre da faixa (onde o Down arrasta), varrido no alvo do último
/// quadro — o diagnóstico da shell (`PH2D_FAIXA_LOG`) e a prova no KWin miram aqui.
#[must_use]
pub fn free_backdrop_x(hero: &super::HeroScreen, width: f32) -> Option<f32> {
    let y = super::menu_bar::MENU_BAR_H * 0.5;
    let passo = Spacing::Xs.px();
    let (mut melhor, mut de) = (None::<(f32, f32)>, None::<f32>);
    let mut x = 0.0;
    while x <= width {
        let livre = x < width && hero.hit_index.hit(x, y) == Some(ids::MENUBAR_BACKDROP);
        match (livre, de) {
            (true, None) => de = Some(x),
            (false, Some(a)) => {
                if melhor.is_none_or(|(m0, m1)| x - a > m1 - m0) {
                    melhor = Some((a, x));
                }
                de = None;
            }
            _ => {}
        }
        x += passo;
    }
    melhor.map(|(a, b)| (a + b) * 0.5)
}

/// A borda sob o ponteiro, para o cursor de redimensionar (`None` com a barra do sistema).
#[must_use]
pub fn hover_edge(hero: &super::HeroScreen, x: f32, y: f32, size: (f32, f32)) -> Option<Edge> {
    let maximized = hero.store.window_frame()?;
    edge_at(x, y, size.0, size.1, maximized)
}

#[cfg(test)]
#[path = "window_buttons_tests.rs"]
mod tests;
