//! Seam do **BINDING DE TOKEN** (plano UI/UX §4/W4) — as rows e as opções estão vivas sob o
//! MOUSE, não só pintadas.
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — foi essa lacuna que deixou as 36 células da matriz de física e os dez chips de
//! ferramenta do Painter *pintados, hit-registrados e mortos sob o ponteiro*.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{TokenBindings, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids, state};
use ph2d_tokens::ColorToken;
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

fn bound(fill: Option<&str>, stroke: Option<&str>, stroke_exists: bool) -> TokenBindings {
    TokenBindings {
        fill: fill.map(str::to_owned),
        stroke: stroke.map(str::to_owned),
        stroke_exists,
    }
}

/// Um Down+Up REAL sobre o retângulo que o painel pintou para `id`.
///
/// ⚠️ Não afirma nada sobre o evento produzido: um chip de `Dropdown` abre por estado INTERNO do
/// dispatcher (não emite `WidgetEvent`), e uma opção emite `Click`. Quem afirma é o chamador — o
/// gate do chip pergunta se ele ABRIU, o da opção se o Click chegou ao BUS.
fn press(host: &mut MockPanelHost, st: &mut VectorPanelState, id: ph2d_a11y::NodeId, what: &str) {
    let r = host
        .painted_rect::<VectorPanel>(st, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(st, ev);
    }
}

/// **Os dois chips abrem** — sem isto o picker é inalcançável e a wave inteira é.
#[test]
fn the_two_token_chips_are_reachable() {
    state::set_token_bindings(Some(bound(None, None, true)));
    for (chip, what) in [
        (ids::VECTOR_TOKEN_FILL, "o chip de Fill"),
        (ids::VECTOR_TOKEN_STROKE, "o chip de Stroke"),
    ] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        assert_eq!(
            host.dropdown_is_open(chip),
            Some(false),
            "{what} nasce fechado — e tem de estar REGISTADO como Dropdown"
        );
        press(&mut host, &mut st, chip, what);
        assert_eq!(
            host.dropdown_is_open(chip),
            Some(true),
            "o ponteiro sobre {what} nao o abriu — ele esta' desenhado e nao existe para o \
             dispatcher (falta o `register` no populate)"
        );
    }
    state::set_token_bindings(None);
}

/// **A escolha chega ao BUS.**
///
/// Fora da allowlist do `event_clicks` a linha pinta, acende sob o mouse e o Click morre no
/// painel: o artista escolheria um token e a arte não mudaria, sem nada a dizer por quê.
///
/// ⚠️ O gate cobre a linha de SOLTAR (índice 0) **e** um token — as duas são rows do mesmo
/// popover, mas só a segunda passa pela tabela `ColorToken::ALL`.
#[test]
fn choosing_a_token_reaches_the_bus() {
    state::set_token_bindings(Some(bound(None, None, true)));
    for (prop, what) in [(0_u16, "Fill"), (1, "Stroke")] {
        for (i, row) in [(0_usize, "soltar"), (1, "o primeiro token")] {
            let mut host = MockPanelHost::with_panel::<VectorPanel>();
            let mut st = VectorPanelState;
            let chip = if prop == 0 {
                ids::VECTOR_TOKEN_FILL
            } else {
                ids::VECTOR_TOKEN_STROKE
            };
            host.set_dropdown_open(chip, true);
            let _ = host.drained_actions();
            let id = ids::vector_token_option_id(prop, i);
            press(&mut host, &mut st, id, &format!("{row} em {what}"));
            assert!(
                host.drained_actions().into_iter().any(|a| matches!(
                    a,
                    EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id
                )),
                "a escolha ({row} em {what}) nao chegou ao bus — ela acende sob o mouse e nao faz \
                 nada (falta a linha na allowlist do event_clicks)"
            );
        }
    }
    state::set_token_bindings(None);
}

/// **Sem seleção não há row de token** — e é isto que a impede de ser dois controles mortos em
/// toda tela sem forma selecionada.
#[test]
fn the_token_rows_are_absent_without_a_selection() {
    state::set_token_bindings(None);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    for id in [ids::VECTOR_TOKEN_FILL, ids::VECTOR_TOKEN_STROKE] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
                .is_none(),
            "a row de token foi pintada sem selecao"
        );
    }
}

/// ⚠️ **A row do TRAÇO só existe quando há traço.**
///
/// Um token de cor não inventa a largura que falta (`VecPath::painted`), então numa forma sem
/// traço ela seria um controle que o artista escolhe e que não muda um pixel. O gate mede as duas
/// metades — com traço a row existe, sem traço não —, e a do Fill continua lá nas duas.
#[test]
fn the_stroke_row_follows_the_stroke_and_the_fill_row_does_not() {
    for (stroke_exists, expect) in [(true, true), (false, false)] {
        state::set_token_bindings(Some(bound(None, None, stroke_exists)));
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        assert_eq!(
            host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_TOKEN_STROKE)
                .is_some(),
            expect,
            "a row do traco tem de seguir a existencia do traco (stroke_exists={stroke_exists})"
        );
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_TOKEN_FILL)
                .is_some(),
            "a row do preenchimento existe nos dois casos — uma cor descreve um fill por inteiro"
        );
    }
    state::set_token_bindings(None);
}

/// **TODA opção do picker é clicável** — inclusive a última.
///
/// ⚠️ A contagem vem da tabela (`ColorToken::ALL`), a mesma que o `populate` percorre: um número
/// escrito à mão em qualquer um dos dois deixaria o último token pintado e morto no dia em que a
/// tabela crescesse, e ninguém olharia para aquele arquivo.
#[test]
fn every_option_the_picker_paints_is_registered() {
    state::set_token_bindings(Some(bound(None, None, true)));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    host.set_dropdown_open(ids::VECTOR_TOKEN_FILL, true);
    let _ = host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_TOKEN_FILL);
    // A última linha é `ColorToken::ALL.len()` (as 80 + a de soltar, base 0).
    let last = ids::vector_token_option_id(0, ColorToken::ALL.len());
    assert!(
        host.store().get(last).is_some(),
        "a ULTIMA opcao do picker nao esta' registada — ela pinta e morre sob o mouse"
    );
    state::set_token_bindings(None);
}

/// **O picker DESTACA o token vigente, não a linha de soltar.**
///
/// Repro do smoke de 2026-08-02 (*"ao reabrir o dropdown já está em None embora eu tenha escolhido
/// outro"*): a linha destacada saía de um `0` cravado, então o picker afirmava "None" sobre uma
/// forma bindada — e escolher None de volta parecia um no-op, porque ele já dizia estar lá.
#[test]
fn the_picker_highlights_the_bound_token_not_none() {
    let accent = ColorToken::ALL
        .iter()
        .position(|t| t.key() == "accent")
        .expect("accent existe");

    // Sem binding: a linha vigente é a de SOLTAR (índice 0).
    state::set_token_bindings(Some(bound(None, None, true)));
    assert_eq!(ph2d_panel_vector::selected_token_row(0), 0);

    // Com binding: a linha do token, e NUNCA a 0.
    state::set_token_bindings(Some(bound(Some("accent"), None, true)));
    assert_eq!(
        ph2d_panel_vector::selected_token_row(0),
        accent + 1,
        "o picker tem de destacar o token vigente"
    );
    assert_ne!(
        ph2d_panel_vector::selected_token_row(0),
        0,
        "destacar None sobre uma forma bindada faz escolher None parecer um no-op"
    );

    // E as duas propriedades não se confundem.
    assert_eq!(
        ph2d_panel_vector::selected_token_row(1),
        0,
        "o traco nao esta' bindado — a linha dele e' a de soltar"
    );
    state::set_token_bindings(None);
}

/// **Escolher FECHA o chip.**
///
/// ⚠️ O light-dismiss não dispara aqui — o clique é DENTRO do popover —, então sem o braço no
/// `apply_event` o card fica aberto por cima da secção e o artista tem de o dispensar à mão
/// (reportado no smoke de 2026-08-02).
#[test]
fn choosing_a_token_closes_the_chip() {
    state::set_token_bindings(Some(bound(None, None, true)));
    for (prop, chip) in [
        (0_u16, ids::VECTOR_TOKEN_FILL),
        (1, ids::VECTOR_TOKEN_STROKE),
    ] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        host.set_dropdown_open(chip, true);
        press(
            &mut host,
            &mut st,
            ids::vector_token_option_id(prop, 1),
            "a escolha",
        );
        assert_eq!(
            host.dropdown_is_open(chip),
            Some(false),
            "o chip ficou ABERTO depois da escolha — o artista tem de o dispensar a' mao"
        );
    }
    state::set_token_bindings(None);
}

/// **A RACHURA é desenhada nas DUAS swatches, e só quando um token as cobre.**
///
/// ⚠️ Arch-gate sobre o fonte, e não um oráculo de pixels: a rachura é desenho PURO (uma chamada
/// de fill), então mutá-la deixa toda a suíte verde — o mesmo ponto cego que o anel do pincel do
/// Painter documenta. A APARÊNCIA dela é do smoke; o que se pode afirmar aqui é que ela é chamada,
/// para as duas, e **atrás da pergunta certa**.
///
/// A geometria em si é gateada onde ela vive (`paint_shapes::fill_slash`).
#[test]
fn the_slash_is_painted_for_both_swatches_and_only_when_bound() {
    const SECTIONS: &str = include_str!("../src/paint_sections.rs");
    assert_eq!(
        SECTIONS.matches("self.token_slash(").count(),
        2,
        "a rachura tem de sair nas DUAS swatches — a que nao a tem afirma uma cor que a arte nao usa"
    );
    for guard in [
        "b.fill.is_some()) {\n            self.token_slash(",
        "b.stroke.is_some()) {\n            self.token_slash(",
    ] {
        assert!(
            SECTIONS.contains(guard),
            "a rachura saiu de tras da pergunta *ha' token nesta propriedade?* — riscada sempre, \
             ela deixa de significar coisa nenhuma"
        );
    }
}
