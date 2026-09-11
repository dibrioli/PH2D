//! Seam da **FÓRMULA** (plano UI/UX W4c.3) — o `f(x)`, o campo, e a lei do editor único.
//!
//! ⚠️ **O host de math aqui é de BRINQUEDO, e é deliberado.** O painel só pergunta *"há como
//! responder sobre fórmulas?"* — ele não parseia nada. Um gate que instalasse o parser real estaria
//! a medir a `ph2d-token-math` a partir do painel, e arrastaria o substrato de grafo para dentro
//! desta crate por causa de um teste.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_tokens::state::TokensPanelState;
use ph2d_panel_tokens::{TokensIntent, TokensPanel, drain_intents, ids};
use ph2d_tokens::num_expr::{MathHost, install_math, uninstall_math};
use ph2d_tokens::num_overrides::{NumValue, clear_num_overrides, set_num_override};
use ph2d_tokens::{NumToken, Theme};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;
/// A linha que os gates usam. Qualquer uma serve; fixá-la torna as falhas comparáveis.
const ROW: usize = 2;

/// Um host que sabe responder e nada mais — o painel só lhe pergunta se EXISTE.
fn arm_math() {
    install_math(MathHost {
        deps: |_| Ok(Vec::new()),
        eval: |_, _| Ok(1.0),
    });
}

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

/// ⚠️ `with_panel` RODA o `populate` — o `MockPanelHost::new()` o pula, e um gate escrito sobre ele
/// fica verde com os widgets mortos sob o rato.
fn host() -> (MockPanelHost, TokensPanelState) {
    let mut h = MockPanelHost::with_panel::<TokensPanel>();
    h.set_panel_visible(TokensPanel::ID, true);
    (h, TokensPanelState::default())
}

fn fresh() {
    clear_num_overrides();
    let _ = drain_intents();
}

/// Abre o campo pelo **gesto real** — clicar o `f(x)`. ⚠️ Um setter de teste para o `fx_open`
/// seria uma segunda porta para o estado do gesto, e um gate que a usasse ficaria verde no dia em
/// que o clique deixasse de a alcançar.
fn open_field(h: &mut MockPanelHost, st: &mut TokensPanelState, row: usize) {
    let id = ids::tokens_num_fx_id(row);
    let r = h
        .painted_rect::<TokensPanel>(st, VIEWPORT, id)
        .expect("o `f(x)` da linha");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    for ev in evs {
        h.apply_panel_event::<TokensPanel>(st, ev);
    }
    let _ = drain_intents();
}

fn author_formula(row: usize, src: &str) {
    set_num_override(
        Theme::default(),
        NumToken::ALL[row],
        Some(NumValue::Expr(src.to_string())),
    )
    .expect("o host de brinquedo aceita qualquer texto");
}

/// **A capacidade decide se o controlo EXISTE** — as duas metades, e é o gate central do botão.
///
/// ⚠️ Sem a metade da AUSÊNCIA isto seria um gate que não pode falhar: um `f(x)` pintado sempre
/// passaria na metade da presença, e o build sem math ofereceria um botão que não faz nada — o
/// modo de falha que o `set_ml_available` do AI Denoise existe para não ter.
#[test]
fn the_fx_button_exists_only_when_there_is_math() {
    fresh();
    uninstall_math();
    let (mut h, mut st) = host();
    assert!(
        h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, ids::tokens_num_fx_id(ROW))
            .is_none(),
        "o `f(x)` foi pintado sem host de math instalado — um botao que nao pode fazer nada"
    );

    arm_math();
    let (mut h, mut st) = host();
    assert!(
        h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, ids::tokens_num_fx_id(ROW))
            .is_some(),
        "com math instalada o `f(x)` continua sem existir — a capacidade nao alcanca o painel"
    );
    uninstall_math();
    fresh();
}

/// **Clicar no `f(x)` ABRE o campo e não escreve nada** — a mesma assimetria do ARMAR do elo.
#[test]
fn clicking_fx_opens_the_field_without_writing_anything() {
    fresh();
    arm_math();
    let (mut h, mut st) = host();
    let id = ids::tokens_num_fx_id(ROW);
    let r = h
        .painted_rect::<TokensPanel>(&mut st, VIEWPORT, id)
        .expect("o `f(x)` da linha");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "o ponteiro sobre o `f(x)` nao virou Click — ele esta' desenhado e nao existe para o \
         dispatcher (falta o `register` no populate)"
    );
    for ev in evs {
        h.apply_panel_event::<TokensPanel>(&mut st, ev);
    }
    assert_eq!(st.fx_open(), Some(ROW), "o clique nao abriu o campo");
    assert!(
        drain_intents().is_empty(),
        "abrir um campo enfileirou uma EDICAO — abrir nao muda o documento"
    );

    // E o campo passa a ser PINTADO, que é o que abrir significa.
    assert!(
        h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, ids::tokens_num_formula_id(ROW))
            .is_some(),
        "o campo nao foi pintado depois de o `f(x)` o abrir"
    );
    uninstall_math();
    fresh();
}

/// **UM SLOT, UM EDITOR.** Uma linha que carrega fórmula mostra o campo e **não** o chip.
///
/// ⚠️ É a lei que impede a destruição silenciosa: com os dois na tela, digitar `20` no chip de uma
/// linha que carrega `{spacing.md} * 2` apagaria a fórmula sem nada a dizer.
#[test]
fn a_row_with_a_formula_shows_the_field_and_not_the_chip() {
    fresh();
    arm_math();
    author_formula(ROW, "{spacing.md} * 2");
    let (mut h, mut st) = host();
    assert!(
        h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, ids::tokens_num_formula_id(ROW))
            .is_some(),
        "a linha carrega uma formula e nao pinta o campo — ela ficaria ineditavel"
    );
    assert!(
        h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, ids::tokens_num_chip_id(ROW))
            .is_none(),
        "a linha pinta o chip E o campo — dois editores para o mesmo valor, e o chip destroi a \
         formula em silencio"
    );
    // E o `f(x)` não é oferecido: a linha já tem fórmula, e o botão não teria trabalho.
    assert!(
        h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, ids::tokens_num_fx_id(ROW))
            .is_none(),
        "o `f(x)` e' oferecido numa linha que ja' tem formula — um clique que nao faz nada"
    );
    // ⚠️ E o CONTROLE: a linha vizinha, sem fórmula, continua com o chip.
    assert!(
        h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, ids::tokens_num_chip_id(ROW + 1))
            .is_some(),
        "a linha SEM formula perdeu o chip — o campo nao e' desta linha"
    );
    uninstall_math();
    fresh();
}

/// **O commit nomeia a LINHA e o TEXTO** — e as duas portas, Enter e perda de foco.
///
/// ⚠️ O oráculo é o par `(linha, texto)`: encaminhar a linha 2 como a 0 é o mesmo defeito com outra
/// roupa, e um `assert!(!intents.is_empty())` não o veria.
#[test]
fn a_commit_names_the_row_and_the_text() {
    for (n, ev) in [
        (
            "Submit",
            WidgetEvent::Submit(ids::tokens_num_formula_id(ROW)),
        ),
        ("Blur", WidgetEvent::Blur(ids::tokens_num_formula_id(ROW))),
    ] {
        fresh();
        arm_math();
        let (mut h, mut st) = host();
        let id = ids::tokens_num_formula_id(ROW);
        // Abre o campo pelo gesto e escreve — o paint é o que o regista no store.
        open_field(&mut h, &mut st, ROW);
        let _ = h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, id);
        h.set_text(id, "{spacing.md} * 2");
        h.apply_panel_event::<TokensPanel>(&mut st, ev);
        assert_eq!(
            drain_intents(),
            vec![TokensIntent::NumFormula {
                row: ROW,
                src: "{spacing.md} * 2".to_string()
            }],
            "o commit por {n} nao chegou ao barramento com a linha e o texto certos"
        );
        assert_eq!(
            st.fx_open(),
            None,
            "o campo ficou aberto depois do commit por {n} — fechar e' do painel e acontece sempre"
        );
        uninstall_math();
    }
    fresh();
}

/// **Um campo apagado SOLTA o token** — o texto vazio atravessa como está, e quem o lê é a shell.
///
/// ⚠️ O painel não decide *"vazio significa reset"*; ele reporta o que o artista escreveu. Um
/// segundo lugar a interpretar o vazio seria a segunda resposta a *"o que este gesto pede?"*.
#[test]
fn an_emptied_field_travels_as_empty_text() {
    fresh();
    arm_math();
    author_formula(ROW, "{spacing.md} * 2");
    let (mut h, mut st) = host();
    let id = ids::tokens_num_formula_id(ROW);
    let _ = h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, id);
    h.set_text(id, "");
    h.apply_panel_event::<TokensPanel>(&mut st, WidgetEvent::Submit(id));
    assert_eq!(
        drain_intents(),
        vec![TokensIntent::NumFormula {
            row: ROW,
            src: String::new()
        }]
    );
    uninstall_math();
    fresh();
}
