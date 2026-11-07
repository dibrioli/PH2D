//! ⭐ **Os botões do jogo no centro da barra de menus** (dono 07/11, como a Godot; o menu *Run* saiu):
//! jogar numa janela, em tela cheia ou no editor, pausar e parar. Parar é o `TOPBAR_RESET`: pára e volta ao começo.

use crate::game_stage::{GameRun, GameStage};
use crate::icons::IconId;
use crate::ids;
use crate::interaction::{HitIndex, InteractiveState, WidgetStore};
use crate::paint::{fill_rounded_rect, resolve};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// Os botões, pela ordem da barra. O dono os pediu nesta ordem.
pub const BOTOES: [(NodeId, IconId); 5] = [
    (ids::MENUBAR_RUN_WINDOW, IconId::PlayWindow),
    (ids::MENUBAR_RUN_FULLSCREEN, IconId::PlayFullscreen),
    (ids::MENUBAR_RUN_EDITOR, IconId::PlayEditor),
    (ids::TOPBAR_PAUSE, IconId::Pause),
    (ids::TOPBAR_RESET, IconId::Stop),
];

/// Regista os botões. Chamado pelo `pre_populate` do hero.
pub fn populate(store: &mut WidgetStore) {
    for (id, _) in BOTOES {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// ⭐ **A porta da geometria:** centrados na barra e, se o centro cair sobre os vizinhos, no meio do vão
/// entre `start_x` e `end_x`. Vazio se não couberem: um botão por cima de um menu rouba-lhe o clique.
#[must_use]
pub fn rects(bar: Rect, start_x: f32, end_x: f32) -> Vec<(NodeId, IconId, Rect)> {
    let lado = bar.h;
    let total = lado * BOTOES.len() as f32;
    let vao = Spacing::Sm.px();
    let (de, ate) = (start_x + vao, end_x - vao);
    if ate - de < total {
        return Vec::new();
    }
    let x = (bar.x + (bar.w - total) * 0.5).clamp(de, ate - total);
    BOTOES
        .iter()
        .enumerate()
        .map(|(i, (id, icon))| {
            let r = Rect::new(x + lado * i as f32, bar.y, lado, bar.h);
            (*id, *icon, r)
        })
        .collect()
}

/// ⭐ Os botões da tela cheia (dono 07/11: sem eles não havia como parar): no topo do jogo, centrados. Jogar é o da tela
/// cheia, a volta do Pause.
#[must_use]
pub fn rects_palco(area: Rect) -> Vec<(NodeId, IconId, Rect)> {
    let lado = super::menu_bar::MENU_BAR_H;
    let botoes = [BOTOES[1], BOTOES[3], BOTOES[4]];
    let inicio = area.x + (area.w - lado * botoes.len() as f32) * 0.5;
    let y = area.y + Spacing::Sm.px();
    botoes
        .iter()
        .enumerate()
        .map(|(i, (id, icon))| {
            (
                *id,
                *icon,
                Rect::new(inicio + lado * i as f32, y, lado, lado),
            )
        })
        .collect()
}

/// Pinta os botões da tela cheia, com um fundo que os separa do jogo.
pub fn paint_palco(
    area: Rect,
    run: GameRun,
    scene: &mut VectorScene,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let botoes = rects_palco(area);
    if let Some(u) = crate::widget::cromo_barra::uniao(botoes.iter().map(|(_, _, r)| *r)) {
        let raio = crate::paint::frame_radius(theme, Radius::Sm.px());
        fill_rounded_rect(scene, u, raio, resolve(ColorToken::Bg1, theme));
    }
    paint(&botoes, run, scene, theme, hit_index, store);
}

/// Este botão está aceso? O Play do palco que corre, ou o Pause com o jogo parado a meio.
#[must_use]
pub fn aceso(id: NodeId, run: GameRun) -> bool {
    let palco = [
        (ids::MENUBAR_RUN_WINDOW, GameStage::Window),
        (ids::MENUBAR_RUN_FULLSCREEN, GameStage::Fullscreen),
        (ids::MENUBAR_RUN_EDITOR, GameStage::Editor),
    ]
    .into_iter()
    .find_map(|(botao, s)| (botao == id).then_some(s));
    match palco {
        Some(s) => run.stage == Some(s) && !run.paused(),
        None => id == ids::TOPBAR_PAUSE && run.paused(),
    }
}

/// Pinta os botões e regista os alvos.
pub fn paint(
    botoes: &[(NodeId, IconId, Rect)],
    run: GameRun,
    scene: &mut VectorScene,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let barra = crate::widget::cromo_barra::Barra::de(theme);
    if let (Some(b), Some(u)) = (
        barra,
        crate::widget::cromo_barra::uniao(botoes.iter().map(|(_, _, r)| *r)),
    ) {
        b.pilula(scene, u);
    }
    for (id, icon, r) in botoes {
        let on = aceso(*id, run);
        let hover = matches!(
            store.button_state(*id).unwrap_or(ButtonState::Normal),
            ButtonState::Hovered | ButtonState::Focused | ButtonState::Pressed
        );
        let cor = if let Some(b) = barra {
            b.segmento(scene, *r, on, hover)
        } else {
            let fundo = if on {
                Some(ColorToken::AccentSoft)
            } else {
                hover.then_some(ColorToken::BgElev)
            };
            if let Some(f) = fundo {
                let raio = crate::paint::frame_radius(theme, Radius::Sm.px());
                fill_rounded_rect(scene, *r, raio, resolve(f, theme));
            }
            resolve(
                if on {
                    ColorToken::Accent
                } else {
                    ColorToken::Text2
                },
                theme,
            )
        };
        let lado = crate::paint::icon_glyph_side(r.h);
        let glifo = Rect::new(
            r.x + (r.w - lado) * 0.5,
            r.y + (r.h - lado) * 0.5,
            lado,
            lado,
        );
        crate::paint::paint_icon(scene, *icon, glifo, cor, StrokeToken::Default.px());
        hit_index.register(*id, *r);
    }
}

#[cfg(test)]
#[path = "transport_bar_tests.rs"]
mod tests;
