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
use ph2d_tokens::overrides::{TokenValue, clear_color_overrides, set_color_override};
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
    (h, TokensPanelState::default())
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
        Some(TokenValue::Literal(Color::from_hex(0x00FF00))),
    )
    .unwrap();
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
        Some(TokenValue::Literal(Color::from_hex(0x00FF00))),
    )
    .unwrap();
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
            Some(TokenValue::Literal(Color::from_hex(0x00FF00))),
        )
        .unwrap();
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
        Some(TokenValue::Literal(Color::from_hex(0x00FF00))),
    )
    .unwrap();
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
    set_color_override(theme, token, Some(TokenValue::Literal(mine))).unwrap();
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

// ── O ELO (plano UI/UX W4b) ──────────────────────────────────────────────────

/// Dirige um gesto de VÁRIOS cliques no MESMO host — o `click` acima constrói um host por chamada,
/// e o elo é um gesto de dois toques cujo estado tem de sobreviver entre eles.
///
/// ⚠️ É essa a diferença que torna este helper obrigatório: com um host por clique o `armed`
/// nasceria e morreria dentro de cada um, e o gate ficaria verde afirmando que o gesto "não
/// enfileira nada" — que é exactamente o que um elo quebrado faz.
fn click_seq(ids_to_click: &[ph2d_a11y::NodeId]) -> (Vec<TokensIntent>, TokensPanelState) {
    let _ = drain_intents();
    let (mut h, mut st) = host();
    let mut t = SEC;
    for &id in ids_to_click {
        let r = h
            .painted_rect::<TokensPanel>(&mut st, VIEWPORT, id)
            .expect("o botao tem de ser PINTADO com area clicavel");
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, t));
        let evs = h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, t + SEC / 100));
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "o ponteiro nao virou Click — o botao esta' desenhado e MORTO sob o rato"
        );
        for ev in evs {
            h.apply_panel_event::<TokensPanel>(&mut st, ev);
        }
        t += SEC;
    }
    (drain_intents(), st)
}

/// **Toda linha oferece o elo, e ele está VIVO sob o rato.**
///
/// ⚠️ Oferecido em TODA linha de propósito: qualquer token pode seguir qualquer outro, e escondê-lo
/// nas linhas de fábrica tornaria o gesto alcançável só onde ele já foi feito.
#[test]
fn every_row_offers_a_live_link_button() {
    clear_color_overrides();
    for row in [0, ColorToken::ALL.len() / 2, ColorToken::ALL.len() - 1] {
        assert!(
            rect_of(ids::tokens_link_id(row)).is_some(),
            "a linha {row} nao pintou o botao de elo"
        );
    }
    // E a última linha responde de verdade — um teto que só o pintor conhecesse deixaria o fim da
    // lista desenhado e mudo.
    let last = ColorToken::ALL.len() - 1;
    let (intents, st) = click_seq(&[ids::tokens_link_id(last)]);
    assert!(intents.is_empty(), "armar NAO e' uma edicao de documento");
    assert_eq!(st.armed(), Some(last), "o clique nao armou a linha");
    clear_color_overrides();
}

/// **Dois cliques fazem o elo, e o sentido é `armada → clicada`.**
///
/// ⚠️ O sentido é a metade que compila igual invertida e escreve no token errado: o artista arma a
/// linha que quer MUDAR e depois aponta para onde ela deve olhar.
#[test]
fn two_clicks_link_the_armed_row_to_the_clicked_one() {
    clear_color_overrides();
    let (intents, st) = click_seq(&[ids::tokens_link_id(2), ids::tokens_link_id(7)]);
    assert_eq!(
        intents,
        vec![TokensIntent::Link { from: 2, to: 7 }],
        "o elo saiu com o sentido trocado (ou nao saiu)"
    );
    assert_eq!(st.armed(), None, "o gesto tinha de terminar desarmado");
    clear_color_overrides();
}

/// **Clicar a MESMA linha desiste** — o mesmo botão desfaz o próprio gesto.
///
/// ⚠️ E o auto-elo fica inalcançável por aqui de graça, o que **não** dispensa a recusa no modelo:
/// esta é a UI, e a porta é a lei.
#[test]
fn clicking_the_same_row_gives_the_gesture_up() {
    clear_color_overrides();
    let (intents, st) = click_seq(&[ids::tokens_link_id(4), ids::tokens_link_id(4)]);
    assert!(
        intents.is_empty(),
        "clicar a propria linha enfileirou um elo — um token nao pode seguir a si mesmo"
    );
    assert_eq!(st.armed(), None, "o segundo clique nao desistiu");
    clear_color_overrides();
}

/// **Fechar o painel desiste de um elo em curso.**
///
/// Um gesto não sobrevive à superfície onde ele estava a ser feito — senão reabrir o painel dias
/// depois deixaria o próximo clique a fechar um elo que ninguém se lembra de ter começado.
#[test]
fn closing_the_panel_gives_the_gesture_up() {
    clear_color_overrides();
    let (_, st) = click_seq(&[ids::tokens_link_id(1), ids::TOKENS_CLOSE]);
    assert_eq!(st.armed(), None, "o elo sobreviveu ao painel fechar");
    clear_color_overrides();
}
