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
use ph2d_vector::{Color, VectorScene};

/// Os botões, pela ordem da barra. O dono os pediu nesta ordem.
pub const BOTOES: [(NodeId, IconId); 5] = [
    (ids::MENUBAR_RUN_WINDOW, IconId::PlayWindow),
    (ids::MENUBAR_RUN_FULLSCREEN, IconId::PlayFullscreen),
    (ids::MENUBAR_RUN_EDITOR, IconId::PlayEditor),
    (ids::TOPBAR_PAUSE, IconId::Pause),
    (ids::TOPBAR_RESET, IconId::Stop),
];

/// ⭐ O botão «Controls» (dono 10/11): à direita do Stop, depois de um separador, numa moldura própria.
pub const CONTROLS: (NodeId, IconId) = (ids::TOPBAR_PLAYER_CONTROLS, IconId::Gamepad);

/// Regista os botões. Chamado pelo `pre_populate` do hero.
pub fn populate(store: &mut WidgetStore) {
    for (id, _) in BOTOES.into_iter().chain([CONTROLS]) {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// O vão de cada lado do separador entre o Stop e o «Controls».
fn vao_do_separador() -> f32 {
    Spacing::Sm.px()
}

/// ⭐ **A porta da geometria:** os cinco do jogo centrados na barra e, se o centro cair sobre os vizinhos, no meio do
/// vão entre `start_x` e `end_x`; o [`CONTROLS`] por último, à direita deles depois do [`separador`]. Vazio se não
/// couberem: um botão por cima de um menu rouba-lhe o clique.
#[must_use]
pub fn rects(bar: Rect, start_x: f32, end_x: f32) -> Vec<(NodeId, IconId, Rect)> {
    let lado = bar.h;
    let jogo = lado * BOTOES.len() as f32;
    let total = jogo + vao_do_separador() * 2.0 + lado;
    let vao = Spacing::Sm.px();
    let (de, ate) = (start_x + vao, end_x - vao);
    if ate - de < total {
        return Vec::new();
    }
    let x = crate::math::safe_clamp(bar.x + (bar.w - jogo) * 0.5, de, ate - total);
    com_controls(BOTOES.iter().enumerate().map(|(i, (id, icon))| {
        let r = Rect::new(x + lado * i as f32, bar.y, lado, bar.h);
        (*id, *icon, r)
    }))
}

/// ⭐ A MESMA regra nas duas barras (a do Play e a da tela cheia): o [`CONTROLS`] à direita do último botão do jogo,
/// depois do vão do [`separador`], um quadrado da altura dele.
fn com_controls(jogo: impl Iterator<Item = (NodeId, IconId, Rect)>) -> Vec<(NodeId, IconId, Rect)> {
    let mut botoes: Vec<_> = jogo.collect();
    if let Some((.., u)) = botoes.last().copied() {
        let x = u.x + u.w + vao_do_separador() * 2.0;
        botoes.push((CONTROLS.0, CONTROLS.1, Rect::new(x, u.y, u.h, u.h)));
    }
    botoes
}

/// O ponto (lógico) está sobre o [`CONTROLS`] pintado no último quadro — numa barra ou na outra.
#[must_use]
pub fn sobre_o_controls(hero: &super::HeroScreen, x: f32, y: f32) -> bool {
    hero.hit_index.hit(x, y) == Some(CONTROLS.0)
}

/// ⭐ Com a tela «Controls» ABERTA (dono 10/11): o movimento sobre o botão é da barra (`true` — a shell passa-o ao hero,
/// que o acende no quadro, D32); fora dele a tela fica com o movimento e o botão apaga-se aqui, porque o hero não o vê.
pub fn movimento_com_a_tela(hero: &mut super::HeroScreen, x: f32, y: f32) -> bool {
    if sobre_o_controls(hero, x, y) {
        return true;
    }
    solta_o_controls(hero);
    false
}

/// O hover do [`CONTROLS`] (e a dica dele) apaga-se: o ponteiro saiu dele para a tela aberta ou para fora da janela.
pub fn solta_o_controls(hero: &mut super::HeroScreen) {
    if hero.store.hot_id() == Some(CONTROLS.0) {
        hero.store.set_hot(None);
        let state = ButtonState::Normal;
        hero.store
            .register(CONTROLS.0, InteractiveState::Button { state });
    }
}

/// O chão das barras do jogo: o da barra de menus (a tinta da lista em cartões, o `Bg1` no modo de antes) — a tela
/// cheia pinta-o sob os seus botões, e o separador lê-se sobre ele com o mesmo contraste nas duas.
#[must_use]
pub fn chao_da_barra(theme: Theme) -> Color {
    crate::widget::cromo_barra::Barra::de(theme)
        .map_or_else(|| resolve(ColorToken::Bg1, theme), |b| b.chao())
}

/// A pílula do grupo do jogo: só os cinco — o [`CONTROLS`] tem a moldura dele.
#[must_use]
pub fn pilula(botoes: &[(NodeId, IconId, Rect)]) -> Option<Rect> {
    let do_jogo = botoes.iter().filter(|(id, ..)| *id != CONTROLS.0);
    crate::widget::cromo_barra::uniao(do_jogo.map(|(_, _, r)| *r))
}

/// O separador vertical entre o Stop e o «Controls» (o filete da casa, ao meio do vão); `None` sem o botão.
#[must_use]
pub fn separador(botoes: &[(NodeId, IconId, Rect)]) -> Option<Rect> {
    let i = botoes.iter().position(|(id, ..)| *id == CONTROLS.0)?;
    let (antes, r) = (botoes.get(i.checked_sub(1)?)?.2, botoes[i].2);
    let w = StrokeToken::Thin.px();
    let inset = Spacing::Xs.px();
    let x = (antes.x + antes.w + r.x - w) * 0.5;
    Some(Rect::new(x, r.y + inset, w, r.h - inset * 2.0))
}

/// ⭐ **A shell publica a corrida a cada quadro** (`palco_janela`): o hero completa-a com o mapa (o jogo tem a tela?)
/// e reescreve a frase do «Controls» — o porquê de estar desligado e a tecla que o mapa dá à tela.
pub fn publish(hero: &mut super::HeroScreen, run: GameRun) {
    let c = hero.input_map.controls();
    let run = GameRun {
        has_controls_screen: c.players_can_change_keys,
        ..run
    };
    let chave = if !run.has_controls_screen {
        "input_map_try.bar_button_locked"
    } else if run.controls_enabled() {
        "input_map_try.bar_button"
    } else {
        "input_map_try.bar_button_off"
    };
    let frase = ph2d_i18n::tr(chave).replace("{key}", &c.open_key.label());
    hero.view.game_run = run;
    hero.store.set_tooltip(CONTROLS.0, frase);
}

/// ⭐ Os botões da tela cheia (dono 07/11: sem eles não havia como parar): no topo do jogo, os três do jogo centrados
/// (jogar é o da tela cheia, a volta do Pause) e o [`CONTROLS`] à direita deles, como na barra do Play.
#[must_use]
pub fn rects_palco(area: Rect) -> Vec<(NodeId, IconId, Rect)> {
    let lado = super::menu_bar::MENU_BAR_H;
    let botoes = [BOTOES[1], BOTOES[3], BOTOES[4]];
    let inicio = area.x + (area.w - lado * botoes.len() as f32) * 0.5;
    let y = area.y + Spacing::Sm.px();
    com_controls(botoes.iter().enumerate().map(|(i, (id, icon))| {
        let r = Rect::new(inicio + lado * i as f32, y, lado, lado);
        (*id, *icon, r)
    }))
}

/// Pinta os botões da tela cheia, com um fundo que os separa do jogo, e a dica do que está sob a mão (a da casa, que
/// quebra em linhas: na tela cheia não há a barra de cima para a pintar).
pub fn paint_palco(
    area: Rect,
    run: GameRun,
    scene: &mut VectorScene,
    text_system: &mut ph2d_text::TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let botoes = rects_palco(area);
    if let Some(u) = crate::widget::cromo_barra::uniao(botoes.iter().map(|(_, _, r)| *r)) {
        let raio = crate::paint::frame_radius(theme, Radius::Sm.px());
        fill_rounded_rect(scene, u, raio, chao_da_barra(theme));
    }
    paint(&botoes, run, scene, theme, hit_index, store);
    let _ = super::topbar::paint_hover_tooltip(scene, text_system, theme, hit_index, store, area);
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
        None if id == CONTROLS.0 => run.controls_open,
        None => id == ids::TOPBAR_PAUSE && run.paused(),
    }
}

/// A caixa da moldura do «Controls» e o raio: o MESMO recuo e raio do realce de um segmento da pílula (o Pause aceso),
/// nos quatro lados, para não encostar nas bordas da barra.
#[must_use]
pub fn caixa_do_controls(r: Rect, theme: Theme) -> (Rect, f32) {
    use ph2d_card_stack::metrics as m;
    let recuo = m::FIELD_BORDER + m::SEG_PAD;
    let caixa = Rect::new(
        r.x + recuo,
        r.y + recuo,
        r.w - recuo * 2.0,
        r.h - recuo * 2.0,
    );
    let raio = crate::widget::cromo_barra::Barra::de(theme).map_or_else(
        || crate::paint::frame_radius(theme, Radius::Sm.px()),
        |_| m::SEG_RADIUS,
    );
    (caixa, raio)
}

/// ⭐ A moldura PRÓPRIA do «Controls»: um contorno (filete da casa, raio do campo) sem o chão da pílula do jogo, para
/// ler como outro grupo. Aceso = o fundo escolhido e o glifo no acento; desligado = o glifo apagado. Devolve a cor do
/// glifo.
fn moldura_controls(
    scene: &mut VectorScene,
    r: Rect,
    theme: Theme,
    run: GameRun,
    hover: bool,
) -> Color {
    let barra = crate::widget::cromo_barra::Barra::de(theme);
    let borda = StrokeToken::Thin.px();
    let (caixa, raio) = caixa_do_controls(r, theme);
    let (ligado, on) = (run.controls_enabled(), run.controls_open);
    let cor = |t: ColorToken| barra.map_or_else(|| resolve(t, theme), |b| b.tinta().color(t));
    let fundo = if on {
        Some(ColorToken::AccentSoft)
    } else {
        (hover && ligado).then_some(ColorToken::BgElev)
    };
    if let Some(f) = fundo {
        fill_rounded_rect(scene, caixa, raio, cor(f));
    }
    // ⭐ Aceso e pairado sobe um degrau (onda 4): o acento de hover da casa (`AccentHover`, o do botão Accent).
    let acento = if hover {
        ColorToken::AccentHover
    } else {
        ColorToken::Accent
    };
    let filete = if on { acento } else { ColorToken::Border };
    // O traço centra-se no caminho: recuado meia espessura, a borda de FORA é a caixa (a do realce do Pause), sem 1 px a mais.
    let meio = borda * 0.5;
    let traco = Rect::new(
        caixa.x + meio,
        caixa.y + meio,
        caixa.w - borda,
        caixa.h - borda,
    );
    // FRAME-RAW-OK: o filete do Controls casa com o realce do Pause na MESMA barra (onda 4 do InputMap): a tinta vem da barra.
    crate::paint::stroke_rounded_rect(scene, traco, raio - meio, borda, cor(filete));
    cor(if on {
        acento
    } else if !ligado {
        ColorToken::TextDisabled
    } else if hover {
        ColorToken::Text1
    } else {
        ColorToken::Text2
    })
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
    if let (Some(b), Some(u)) = (barra, pilula(botoes)) {
        b.pilula(scene, u);
    }
    if let Some(s) = separador(botoes) {
        let cor = barra.map_or_else(|| resolve(ColorToken::Border, theme), |b| b.divisoria());
        scene.fill_rect(crate::paint::rect_to_vello(s), cor);
    }
    for (id, icon, r) in botoes {
        let on = aceso(*id, run);
        let hover = matches!(
            store.button_state(*id).unwrap_or(ButtonState::Normal),
            ButtonState::Hovered | ButtonState::Focused | ButtonState::Pressed
        );
        let cor = if *id == CONTROLS.0 {
            moldura_controls(scene, *r, theme, run, hover)
        } else if let Some(b) = barra {
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
