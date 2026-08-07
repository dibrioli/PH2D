//! **O TETO de um valor autorado** — a dívida que a W4c.1 nomeou e a W4c.2 tinha de pagar
//! (plano UI/UX W4c.2; CLAUDE.md §0: *meça antes de escrever um limite*).
//!
//! # A pergunta não é sobre um recurso — é sobre ALCANÇAR o desfazer
//!
//! Um espaçamento absurdo não consome memória nem largura de banda; ele consome **tela**. E o
//! painel de Tokens **desenha-se a si mesmo com os tokens que edita**, então a pergunta honesta é
//! uma só: *depois de digitar um número enorme, o artista ainda alcança o botão que o desfaz?*
//!
//! # ⚠️ TRÊS afirmações minhas que a medição derrubou
//!
//! Cada uma era a diferença entre um gate honesto e um gate verde-sobre-nada, e ficam escritas
//! porque a próxima pessoa a mexer aqui vai ter as mesmas três.
//!
//! 1. *"O `Reset This Mode` vive num cabeçalho que não rola."* **Falso** — ele é pintado dentro do
//!    corpo rolável, e com `spacing.* = 1024 px` a caixa dele pousa em `y = 2206` numa viewport de
//!    900. Medir a posição com a rolagem em **zero** responde *onde o botão está*, nunca *se o
//!    artista chega lá*, e as duas divergem exactamente no regime que interessa.
//! 2. *"Então basta rolar até ao fim."* **Também falso** — o botão fica perto do TOPO do conteúdo,
//!    então a rolagem máxima passa **por cima** dele (`y = −108244`). A pergunta certa não é um
//!    extremo, é *existe ALGUMA rolagem que o põe na tela?*
//! 3. *"O controle é a escala de fábrica."* **Falso** — o `Reset This Mode` só é pintado quando há
//!    algo autorado (um reset sobre um modo de fábrica seria um clique que não faz nada), então
//!    uma fixture limpa não tem botão nenhum a medir. O controle tem de autorar **um** token.
//!
//! ⚠️ **E é por isso que esta wave não escreve cap nenhum:** a sonda mede `y ≈ 158 + 2·px`, o que
//! põe o botão fora da tela por volta de **~357 px** *nesta* viewport — o número é função da altura
//! da janela, então qualquer literal estaria errado para metade dos monitores. O escape não é um
//! teto, é a **ROLAGEM**, e ela é medida abaixo em vez de assumida.
//!
//! Rodar a medição: `cargo test -p ph2d-panel-tokens --test scale_ceiling -- --ignored --nocapture`

use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_tokens::state::TokensPanelState;
use ph2d_panel_tokens::{TokensPanel, ids};
use ph2d_tokens::num_overrides::{NumValue, clear_num_overrides, set_num_override};
use ph2d_tokens::{NumToken, Spacing, Theme};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn host() -> (MockPanelHost, TokensPanelState) {
    let mut h = MockPanelHost::with_panel::<TokensPanel>();
    h.set_panel_visible(TokensPanel::ID, true);
    (h, TokensPanelState::default())
}

/// Autora TODA a escala de espaçamento em `px` e publica — o pior caso, não um token solto.
fn author_whole_spacing_scale(px: f32) {
    clear_num_overrides();
    for tok in NumToken::ALL {
        if matches!(tok, NumToken::Spacing(_)) {
            set_num_override(Theme::Forge, *tok, Some(NumValue::Literal(px)))
                .expect("um literal nunca fecha um laco");
        }
    }
    ph2d_tokens::num_runtime::publish(Theme::Forge);
}

fn factory() {
    clear_num_overrides();
    ph2d_tokens::num_runtime::publish(Theme::Forge);
}

/// Autora UM token no seu próprio valor de fábrica — a escala não se move, e o botão existe.
fn author_one_neutral_token() {
    clear_num_overrides();
    set_num_override(
        Theme::Forge,
        NumToken::Spacing(Spacing::Md),
        Some(NumValue::Literal(Spacing::Md.factory_px())),
    )
    .expect("um literal nunca fecha um laco");
    ph2d_tokens::num_runtime::publish(Theme::Forge);
}

/// **A rolagem que põe o desfazer na tela**, se alguma — a pergunta de ALCANCE, medida.
///
/// ⚠️ **RESOLVIDA, não amostrada, e a 1ª versão errou por isso.** Ela varria a faixa de rolagem em
/// 400 passos e reportava `NENHUMA` em `spacing.* = 65536 px` — mas ali o passo vale ~3440 px
/// contra uma janela de 900, então a varredura **saltava por cima** do botão. Era a resolução da
/// fixture a falar, apresentada como um defeito do produto.
///
/// O painel desloca o conteúdo por `−scroll`, então a posição é **afim na rolagem**: mede-se `y`
/// com a rolagem em zero, resolve-se o valor que traz a caixa para o meio da viewport, e
/// **confirma-se pintando lá**. Exacto, e sem laço.
fn scroll_that_reaches_the_undo() -> Option<(f32, Rect)> {
    let (mut h, mut st) = host();
    let at_zero = h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, ids::TOKENS_RESET_ALL)?;
    let max = (ph2d_panel_tokens::last_content_h() - ph2d_panel_tokens::last_visible_h()).max(0.0);

    // Quanto é preciso rolar para a caixa pousar no meio da tela — clampado à faixa que existe.
    let want = (at_zero.y - VIEWPORT.h * 0.5).clamp(0.0, max);
    h.set_panel_scroll(ids::TOKENS_PANEL, want);
    let r = h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, ids::TOKENS_RESET_ALL)?;
    (r.w > 0.0 && r.h > 0.0 && r.y + r.h > VIEWPORT.y && r.y < VIEWPORT.y + VIEWPORT.h)
        .then_some((want, r))
}

/// **A metade que decide se existe cap: o desfazer continua ALCANÇÁVEL.**
///
/// ⚠️ O oráculo é o *Reset This Mode* — o botão que devolve o modo inteiro à fábrica. Se ele deixar
/// de ser alcançável, o artista fica preso e o cap deixa de ser opcional; enquanto ele for, um
/// valor absurdo é feio e **reversível**, que é o que um design system permite por desenho.
#[test]
fn the_panel_survives_an_absurd_scale() {
    for px in [64.0_f32, 256.0, 1024.0, 4096.0, 65536.0] {
        author_whole_spacing_scale(px);
        let found = scroll_that_reaches_the_undo();
        factory();
        assert!(
            found.is_some(),
            "com spacing.* = {px} px nenhuma rolagem poe o *Reset This Mode* na tela — o artista \
             ficou sem como desfazer, e ai o cap deixa de ser opcional"
        );
    }
}

/// O **CONTROLE**: com a escala de fábrica o botão está lá, e sem rolar.
///
/// ⚠️ Ele autora **um** token porque o `Reset This Mode` só é pintado quando há o que resetar —
/// uma fixture limpa não teria botão nenhum, e o gate acima passaria por vácuo.
#[test]
fn the_undo_is_right_there_at_the_factory_scale() {
    author_one_neutral_token();
    let (s, r) = scroll_that_reaches_the_undo().expect("na fabrica o desfazer TEM de existir");
    factory();
    assert!(
        s.abs() < f32::EPSILON,
        "na escala de fabrica o *Reset This Mode* devia estar visivel sem rolar (precisou de {s})"
    );
    assert!(r.w > 0.0 && r.h > 0.0);
}

/// **O painel RESPONDE à escala** — sem isto o gate de alcance ficaria verde num painel que
/// ignorasse a tabela viva por completo, medindo um desenho que não se mexe.
#[test]
fn the_panel_moves_when_the_scale_moves() {
    author_one_neutral_token();
    let (_, a) = scroll_that_reaches_the_undo().expect("fabrica");

    author_whole_spacing_scale(Spacing::Xl4.factory_px() * 4.0);
    let (_, b) = scroll_that_reaches_the_undo().expect("escala grande");
    factory();

    assert!(
        (a.y - b.y).abs() > 0.5,
        "a escala autorada nao moveu NADA no painel ({a:?} == {b:?}) — ou ele nao le a tabela \
         viva, ou o gate de alcance esta' a medir uma coisa que nao responde"
    );
}

/// **A SONDA** — o número do penhasco, para quem um dia precise dele.
#[test]
#[ignore = "sonda: onde o desfazer pousa, e que rolagem o alcanca"]
fn measure_where_the_undo_lands() {
    println!("spacing.* (px) |  y (scroll 0) | rolagem que o alcanca |   y ja' rolado");
    for px in [
        Spacing::Md.factory_px(),
        16.0,
        32.0,
        64.0,
        128.0,
        256.0,
        357.0,
        1024.0,
        4096.0,
        65536.0,
    ] {
        author_whole_spacing_scale(px);
        let (mut h, mut st) = host();
        let top = h
            .painted_rect::<TokensPanel>(&mut st, VIEWPORT, ids::TOKENS_RESET_ALL)
            .map_or(f32::NAN, |r| r.y);
        match scroll_that_reaches_the_undo() {
            Some((s, r)) => println!("{px:>13.1} | {top:>13.1} | {s:>21.1} | {:>14.1}", r.y),
            None => println!("{px:>13.1} | {top:>13.1} | {:>21} | {:>14}", "NENHUMA", "-"),
        }
    }
    factory();
}
