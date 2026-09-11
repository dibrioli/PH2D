//! Seam do **interop DTCG** (plano UI/UX W9) — os dois botões estão vivos sob o MOUSE, e são
//! oferecidos **sempre**, ao contrário do vizinho de cima.
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

/// ⚠️ `with_panel` é o construtor que RODA o `populate` — o `MockPanelHost::new()` o pula, e um
/// gate escrito sobre ele fica verde com os widgets mortos sob o mouse.
fn host() -> (MockPanelHost, TokensPanelState) {
    let mut h = MockPanelHost::with_panel::<TokensPanel>();
    h.set_panel_visible(TokensPanel::ID, true);
    (h, TokensPanelState::default())
}

fn rect_of(id: ph2d_a11y::NodeId) -> Option<Rect> {
    let (mut h, mut st) = host();
    h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, id)
}

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

/// **Os dois botões existem, são clicáveis, e cada um pede a SUA metade.**
#[test]
fn both_dtcg_buttons_are_alive_and_each_asks_for_its_own_half() {
    clear_color_overrides();
    assert_eq!(
        click(ids::TOKENS_DTCG_EXPORT, "Export DTCG"),
        vec![TokensIntent::ExportDtcg]
    );
    assert_eq!(
        click(ids::TOKENS_DTCG_IMPORT, "Import DTCG"),
        vec![TokensIntent::ImportDtcg]
    );
}

/// ⭐ **Eles são oferecidos com a tabela de FÁBRICA — e o *Reset This Mode* não.**
///
/// ⚠️ É a assimetria inteira num gate: um *Reset* de um modo intocado é um clique que não faz nada,
/// mas um EXPORT de um modo intocado é o design system inteiro, que é precisamente o que alguém
/// quer levar para outra ferramenta. O *Reset* é o **controlo** — sem ele este gate não distingue
/// *"o par é sempre oferecido"* de *"tudo é sempre oferecido"*.
#[test]
fn the_pair_is_offered_on_a_factory_table_and_the_reset_is_not() {
    clear_color_overrides();
    assert!(
        rect_of(ids::TOKENS_DTCG_EXPORT).is_some(),
        "o Export tem de existir num modo de fabrica"
    );
    assert!(
        rect_of(ids::TOKENS_DTCG_IMPORT).is_some(),
        "o Import tem de existir num modo de fabrica"
    );
    assert!(
        rect_of(ids::TOKENS_RESET_ALL).is_none(),
        "o CONTROLE falhou: o Reset This Mode apareceu sobre uma tabela intocada"
    );

    // E com algo autorado os três convivem, lado a lado.
    set_color_override(
        Theme::default(),
        ColorToken::Accent,
        Some(TokenValue::Literal(Color {
            r: 1,
            g: 2,
            b: 3,
            a: 255,
        })),
    )
    .expect("um literal nunca fecha um laco");
    assert!(rect_of(ids::TOKENS_RESET_ALL).is_some());
    assert!(rect_of(ids::TOKENS_DTCG_EXPORT).is_some());
    clear_color_overrides();
}

/// **Os dois ficam lado a lado, e não empilhados** — importar e exportar são a mesma operação em
/// dois sentidos, e separá-los em duas linhas faria o artista procurar o segundo.
#[test]
fn the_pair_shares_one_row_without_overlapping() {
    let a = rect_of(ids::TOKENS_DTCG_EXPORT).expect("Export pintado");
    let b = rect_of(ids::TOKENS_DTCG_IMPORT).expect("Import pintado");
    assert!(
        (a.y - b.y).abs() < 0.5,
        "os dois botoes tem de partilhar a linha: y={} e y={}",
        a.y,
        b.y
    );
    assert!(
        a.x + a.w <= b.x + 0.5,
        "os retangulos sobrepoem-se: um clique cairia no botao errado ({a:?} / {b:?})"
    );
    assert!(a.w > 0.0 && b.w > 0.0);
}
