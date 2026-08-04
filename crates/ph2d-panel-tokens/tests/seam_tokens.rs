//! Seam do painel de **TOKENS** (plano UI/UX W6, degrau 1) — as linhas estão vivas sob o MOUSE, e
//! cada controlo aparece **só onde faz sentido**.
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist e **pula a checagem de focabilidade no store** — a
//! lacuna que já deixou as 36 células da matriz de física e os dez chips do impasto *pintados,
//! hit-registrados e mortos sob o ponteiro*.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_tokens::state::TokensPanelState;
use ph2d_panel_tokens::{TokensIntent, TokensPanel, drain_intents, ids};
use ph2d_tokens::color::Color;
use ph2d_tokens::overrides::{clear_color_overrides, set_color_override};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

/// Um host com o painel aberto. ⚠️ `with_panel` é o construtor que RODA o `populate` — o
/// `MockPanelHost::new()` o pula, e um gate escrito sobre ele fica verde com os widgets mortos sob
/// o mouse (o defeito que a lista de dez ferramentas do impasto já pagou).
fn host() -> (MockPanelHost, TokensPanelState) {
    let mut h = MockPanelHost::with_panel::<TokensPanel>();
    h.set_panel_visible(TokensPanel::ID, true);
    (h, TokensPanelState)
}

fn rect_of(id: ph2d_a11y::NodeId) -> Option<Rect> {
    let (mut h, mut st) = host();
    h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, id)
}

/// Clica de verdade e devolve os intents que o painel enfileirou.
fn click(id: ph2d_a11y::NodeId, what: &str) -> Vec<TokensIntent> {
    let _ = drain_intents(); // a fila é thread-local; um teste não pode herdar o vizinho
    let (mut h, mut st) = host();
    let r = h
        .painted_rect::<TokensPanel>(&mut st, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "o ponteiro sobre {what} nao virou Click — ele esta' desenhado e nao existe para o \
         dispatcher (falta o `register` no populate)"
    );
    for ev in evs {
        h.apply_panel_event::<TokensPanel>(&mut st, ev);
    }
    drain_intents()
}

/// **Cada token do design system ganha uma linha, e a swatch é alvo de PICKER.**
///
/// ⚠️ A metade do picker é o que separa este gate de um verde vazio: registada como BOTÃO, a
/// swatch acende sob o mouse e **nunca abre o OKLCH** — a cor ficaria ineditável com todos os
/// outros gates verdes (a cicatriz que a lista de peças da W5b já pagou).
#[test]
fn every_token_gets_a_row_whose_swatch_is_a_picker_target() {
    clear_color_overrides();
    let (mut h, mut st) = host();
    for row in 0..ColorToken::ALL.len() {
        let id = ids::tokens_swatch_id(row);
        assert!(
            h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, id)
                .is_some(),
            "a linha {row} ({}) nao foi pintada",
            ColorToken::ALL[row].key()
        );
        assert!(
            h.store().is_picker_swatch(id),
            "a swatch da linha {row} nao e' alvo de picker — o clique acende e nao abre o OKLCH"
        );
    }
}

/// **O Reset de uma linha só existe quando ela está AUTORADA** — a metade da AUSÊNCIA.
///
/// ⚠️ Sem ela o painel pintaria ~80 botões *Reset* inertes, que é o botão-morto que este repo
/// persegue: ele ensina o artista a duvidar dos outros.
#[test]
fn the_per_row_reset_appears_only_on_an_authored_token() {
    clear_color_overrides();
    assert!(
        rect_of(ids::tokens_reset_id(0)).is_none(),
        "o Reset foi oferecido sobre um token de fabrica"
    );
    set_color_override(
        Theme::default(),
        ColorToken::ALL[0],
        Some(Color::from_hex(0x00FF00)),
    );
    assert!(
        rect_of(ids::tokens_reset_id(0)).is_some(),
        "o Reset nao apareceu sobre um token autorado"
    );
    clear_color_overrides();
}

/// **O *Reset This Mode* só existe com o que resetar** — o mesmo raciocínio, no escopo do modo.
#[test]
fn the_reset_all_appears_only_when_the_mode_has_authored_tokens() {
    clear_color_overrides();
    assert!(rect_of(ids::TOKENS_RESET_ALL).is_none());
    set_color_override(
        Theme::default(),
        ColorToken::ALL[0],
        Some(Color::from_hex(0x00FF00)),
    );
    assert!(rect_of(ids::TOKENS_RESET_ALL).is_some());
    clear_color_overrides();
}

/// **O clique num Reset chega ao barramento como o intent da LINHA CERTA.**
///
/// ⚠️ O oráculo é o ÍNDICE, não a presença do intent: encaminhar a linha 3 como a 0 é o mesmo
/// defeito com outra roupa, e um `assert!(!intents.is_empty())` não o veria.
#[test]
fn a_reset_click_names_the_row_it_sits_on() {
    clear_color_overrides();
    // Duas linhas autoradas, para a lista ter mais de uma e o índice poder estar errado.
    for row in [0usize, 3] {
        set_color_override(
            Theme::default(),
            ColorToken::ALL[row],
            Some(Color::from_hex(0x00FF00)),
        );
    }
    assert_eq!(
        click(ids::tokens_reset_id(3), "o Reset da linha 3"),
        vec![TokensIntent::Reset(3)]
    );
    clear_color_overrides();
}

/// **O *Reset This Mode* chega ao barramento.**
#[test]
fn the_reset_all_click_reaches_the_bus() {
    clear_color_overrides();
    set_color_override(
        Theme::default(),
        ColorToken::ALL[0],
        Some(Color::from_hex(0x00FF00)),
    );
    assert_eq!(
        click(ids::TOKENS_RESET_ALL, "Reset This Mode"),
        vec![TokensIntent::ResetAll]
    );
    clear_color_overrides();
}

/// **A swatch mostra a cor EFETIVA, não a de fábrica** — o painel é a autoridade sobre o que o
/// app está a usar.
///
/// ⚠️ Uma swatch que afirmasse o valor de fábrica sob um token autorado seria a rachura que a row
/// de Token do vetor já documenta: um número na tela que o desenho não usa.
#[test]
fn the_swatch_shows_the_effective_colour() {
    clear_color_overrides();
    let theme = Theme::default();
    let token = ColorToken::ALL[0];
    let factory = token.resolve(theme);
    let mine = Color::from_hex(0x00FF00);
    assert_ne!(factory, mine);
    set_color_override(theme, token, Some(mine));
    assert_eq!(
        token.resolve(theme),
        mine,
        "a porta que o painel pinta nao devolveu a cor autorada"
    );
    clear_color_overrides();
    assert_eq!(token.resolve(theme), factory);
}

/// **Fechado, o painel LARGA o próprio rect** — a metade da arrumação.
///
/// ⚠️ Sem a limpeza o `panel_at` continua a devolver `TOKENS_PANEL` depois de fechado, e a roda do
/// rato rola um painel que não está na tela.
///
/// ⚠️ **E o gate tem de usar o `paint_hidden`.** A minha 1ª versão chamava o `painted_rect` com a
/// visibilidade posta a `false` e afirmava *"não pintou linha nenhuma"* — mas todo helper de paint
/// deste harness **FORÇA o painel visível** (senão um paint gateado em `panel_visible` devolveria
/// antes de desenhar, e os helpers existem para ler o que ele desenhou). O gate nasceu VERMELHO
/// sobre produto correto, medindo o ramo aberto e chamando-lhe fechado; o `paint_hidden` é o
/// helper que a auditoria de 2026-07-29 (§4 D-K) criou exactamente para este ramo.
#[test]
fn a_closed_panel_drops_its_rect() {
    clear_color_overrides();
    let (mut h, mut st) = host();
    // Aberto: o rect é publicado (o controle — sem ele o gate abaixo é verde por vácuo).
    let _ = h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, ids::tokens_swatch_id(0));
    assert!(
        h.store().panel_rect(ids::TOKENS_PANEL).is_some(),
        "o painel aberto nao publicou o proprio rect"
    );
    h.paint_hidden::<TokensPanel>(&mut st, VIEWPORT);
    assert!(
        h.store().panel_rect(ids::TOKENS_PANEL).is_none(),
        "o painel fechado deixou o rect para tras — a roda rolaria um painel que nao esta' na tela"
    );
}
