//! Seam da família **NUMÉRICA** do painel de Tokens (plano UI/UX W4c.1) — irmão do
//! [`seam_tokens`], cortado pelo mesmo assunto que o `paint_num.rs`.
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou) onde o controlo é um BOTÃO. Para
//! o **chip** ele é misto, e a divisão está nomeada em vez de escondida: `set_number_value` **PANICA
//! se o widget não for um `NumberInput`** (a metade estrutural, que é o que o torna alcançável pela
//! máquina de digitação/arrasto) e o `ValueChanged` despachado prova a metade do ROTEAMENTO. Um
//! Down+Up sobre um chip é um gesto de FOCO, não uma edição — ele não emitiria valor nenhum.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_tokens::state::TokensPanelState;
use ph2d_panel_tokens::{TokensIntent, TokensPanel, drain_intents, ids};
use ph2d_tokens::num_overrides::{NumValue, clear_num_overrides, set_num_override};
use ph2d_tokens::overrides::{TokenValue, clear_color_overrides, set_color_override};
use ph2d_tokens::{ColorToken, NumToken, Theme};
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

/// ⚠️ `with_panel` é o construtor que RODA o `populate` — o `MockPanelHost::new()` o pula, e um
/// gate escrito sobre ele fica verde com os widgets mortos sob o mouse.
fn host() -> (MockPanelHost, TokensPanelState) {
    let mut h = MockPanelHost::with_panel::<TokensPanel>();
    h.set_panel_visible(TokensPanel::ID, true);
    (h, TokensPanelState::default())
}

fn fresh() {
    clear_num_overrides();
    clear_color_overrides();
    let _ = drain_intents();
}

fn rect_of(id: ph2d_a11y::NodeId) -> Option<Rect> {
    let (mut h, mut st) = host();
    h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, id)
}

/// Clica de verdade e devolve os intents que o painel enfileirou.
fn click(id: ph2d_a11y::NodeId, what: &str) -> Vec<TokensIntent> {
    let _ = drain_intents();
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

/// **Cada token numérico ganha uma linha, e o chip é um `NumberInput` COM FAIXA.**
///
/// ⚠️ As duas metades. Sem o `NumberInput` o chip acende sob o mouse e nunca aceita um número — o
/// valor ficaria inedidável com todos os outros gates verdes. Sem a **faixa** ele deriva o passo do
/// texto do buffer e varre ~50 unidades por PIXEL: um pixel de arrasto bate no teto e o chip vira
/// um interruptor min↔max, **com a digitação a continuar a funcionar** — que é porque esta classe
/// de bug sobrevive a revisão.
#[test]
fn every_numeric_token_gets_a_row_whose_chip_is_a_number_input_with_a_range() {
    fresh();
    let (mut h, mut st) = host();
    for row in 0..NumToken::ALL.len() {
        let id = ids::tokens_num_chip_id(row);
        assert!(
            h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, id)
                .is_some(),
            "a linha numerica {row} ({}) nao foi pintada",
            NumToken::ALL[row].key()
        );
        assert!(
            h.store().number_range(id).is_some(),
            "o chip da linha {row} ({}) nao tem faixa registada — ele vira um interruptor \
             min<->max no arrasto",
            NumToken::ALL[row].key()
        );
        // ⚠️ PANICA se o widget nao for um `NumberInput` — e' essa a asserção estrutural.
        h.set_number_value(id, 1.0);
    }
}

/// **O clique num chip chega ao barramento como o `NumSet` da LINHA CERTA, com o número certo.**
///
/// ⚠️ O oráculo é o par `(linha, valor)`: encaminhar a linha 3 como a 0 é o mesmo defeito com outra
/// roupa, e um `assert!(!intents.is_empty())` não o veria.
#[test]
fn a_chip_edit_names_the_row_and_the_number() {
    fresh();
    let (mut h, mut st) = host();
    let row = 3usize;
    let id = ids::tokens_num_chip_id(row);
    // O paint tem de correr antes: e' ele que espelha o efetivo no chip.
    let _ = h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, id);
    h.set_number_value(id, 13.0);
    h.apply_panel_event::<TokensPanel>(&mut st, WidgetEvent::ValueChanged(id));
    assert_eq!(
        drain_intents(),
        vec![TokensIntent::NumSet { row, px: 13.0 }]
    );
}

/// **O Reset de uma linha numérica só existe quando ela está AUTORADA** — a metade da AUSÊNCIA.
#[test]
fn the_numeric_reset_appears_only_on_an_authored_token() {
    fresh();
    assert!(
        rect_of(ids::tokens_num_reset_id(0)).is_none(),
        "o Reset foi oferecido sobre um token de fabrica"
    );
    set_num_override(
        Theme::default(),
        NumToken::ALL[0],
        Some(NumValue::Literal(13.0)),
    )
    .unwrap();
    assert!(
        rect_of(ids::tokens_num_reset_id(0)).is_some(),
        "o Reset nao apareceu sobre um token autorado"
    );
    fresh();
}

#[test]
fn a_numeric_reset_click_names_the_row_it_sits_on() {
    fresh();
    for row in [0usize, 3] {
        set_num_override(
            Theme::default(),
            NumToken::ALL[row],
            Some(NumValue::Literal(13.0)),
        )
        .unwrap();
    }
    assert_eq!(
        click(ids::tokens_num_reset_id(3), "o Reset da linha numerica 3"),
        vec![TokensIntent::NumReset(3)]
    );
    fresh();
}

/// **O elo numérico ARMA no 1º clique e DISPARA no 2º** — e o 1º não escreve nada.
///
/// ⚠️ A metade *"o 1º não enfileira"* é a que importa: um elo que emitisse já no arme escreveria
/// sobre um alvo que o artista ainda não escolheu.
#[test]
fn the_numeric_link_arms_first_and_only_then_fires() {
    fresh();
    let (mut h, mut st) = host();
    let arm = ids::tokens_num_link_id(1);
    let target = ids::tokens_num_link_id(4);
    for (id, what) in [(arm, "o elo da linha 1"), (target, "o elo da linha 4")] {
        let r = h
            .painted_rect::<TokensPanel>(&mut st, VIEWPORT, id)
            .unwrap_or_else(|| panic!("{what} nao foi pintado"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
        let evs = h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
        for ev in evs {
            h.apply_panel_event::<TokensPanel>(&mut st, ev);
        }
        if id == arm {
            assert_eq!(st.armed_num(), Some(1), "o 1o clique nao ARMOU a linha 1");
            assert!(
                drain_intents().is_empty(),
                "o ARME enfileirou uma edicao — o alvo ainda nao foi escolhido"
            );
        }
    }
    assert_eq!(st.armed_num(), None, "o 2o clique nao desarmou");
    assert_eq!(
        drain_intents(),
        vec![TokensIntent::NumLink { from: 1, to: 4 }]
    );
}

/// **Um gesto armado na família de COR é abandonado ao clicar num elo numérico** — nunca fechado.
///
/// ⚠️ Um elo px→cor não tem valor a devolver, e o `armed` guarda a FAMÍLIA justamente para que o
/// caso seja inexprimível: com um índice cru, clicar o elo numérico da linha 4 com a cor 1 armada
/// enfileiraria `Link { from: 1, to: 4 }` **na tabela de cor**, em silêncio.
#[test]
fn arming_a_colour_row_and_clicking_a_numeric_link_abandons_the_gesture() {
    fresh();
    let (mut h, mut st) = host();
    for (id, what) in [
        (ids::tokens_link_id(1), "o elo de COR da linha 1"),
        (ids::tokens_num_link_id(4), "o elo NUMERICO da linha 4"),
    ] {
        let r = h
            .painted_rect::<TokensPanel>(&mut st, VIEWPORT, id)
            .unwrap_or_else(|| panic!("{what} nao foi pintado"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
        let evs = h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
        for ev in evs {
            h.apply_panel_event::<TokensPanel>(&mut st, ev);
        }
    }
    assert_eq!(st.armed(), None, "o arme de COR devia ter sido abandonado");
    assert_eq!(st.armed_num(), Some(4), "o clique numerico devia ARMAR");
    assert!(
        drain_intents().is_empty(),
        "um elo atravessou as familias — nao ha' valor que uma cor de^ a um espacamento"
    );
}

/// **O chip mostra o valor EFETIVO** — o painel é a autoridade sobre o que o app usaria.
#[test]
fn the_chip_shows_the_effective_px() {
    fresh();
    let theme = Theme::default();
    let token = NumToken::ALL[0];
    let factory = token.factory_px();
    assert_ne!(factory, 13.0);
    set_num_override(theme, token, Some(NumValue::Literal(13.0))).unwrap();

    let (mut h, mut st) = host();
    let id = ids::tokens_num_chip_id(0);
    let _ = h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, id);
    assert_eq!(
        h.store().number_value(id),
        Some(13.0),
        "o chip nao espelhou o valor autorado"
    );
    fresh();
}

/// ⚠️ **O *Reset This Mode* aparece com um token NUMÉRICO autorado e NENHUMA cor.**
///
/// É o gate que apanha a contagem a somar só uma família: com ela a contar apenas cores, um modo
/// com a escala inteira re-vestida diria *"0 authored"* e **não ofereceria o botão que a desfaz** —
/// trabalho preso sem gesto que o solte.
#[test]
fn the_reset_all_appears_when_only_a_numeric_token_is_authored() {
    fresh();
    assert!(
        rect_of(ids::TOKENS_RESET_ALL).is_none(),
        "controle: um modo de fabrica nao devia oferecer o Reset"
    );
    set_num_override(
        Theme::default(),
        NumToken::ALL[0],
        Some(NumValue::Literal(13.0)),
    )
    .unwrap();
    assert!(
        rect_of(ids::TOKENS_RESET_ALL).is_some(),
        "a escala autorada nao acordou o Reset This Mode — a contagem esta' a somar so' as cores"
    );
    fresh();
}

/// As duas listas coexistem, e **os ids não colidem**: o mesmo índice em famílias diferentes tem de
/// pintar em sítios diferentes.
#[test]
fn the_two_families_do_not_share_a_rect() {
    fresh();
    // Autorados, para as duas linhas terem o mesmo conjunto de controlos.
    set_color_override(
        Theme::default(),
        ColorToken::ALL[0],
        Some(TokenValue::Literal(ph2d_tokens::color::Color::from_hex(
            0x00FF00,
        ))),
    )
    .unwrap();
    set_num_override(
        Theme::default(),
        NumToken::ALL[0],
        Some(NumValue::Literal(13.0)),
    )
    .unwrap();
    let (mut h, mut st) = host();
    let a = h
        .painted_rect::<TokensPanel>(&mut st, VIEWPORT, ids::tokens_reset_id(0))
        .expect("o Reset de cor da linha 0");
    let b = h
        .painted_rect::<TokensPanel>(&mut st, VIEWPORT, ids::tokens_num_reset_id(0))
        .expect("o Reset numerico da linha 0");
    assert!(
        (a.y - b.y).abs() > f32::EPSILON,
        "as duas linhas 0 pintam no MESMO sitio — os ids colidiram"
    );
    fresh();
}
