//! ⭐⭐ **Costura do laboratório: abrir, fechar, e — o que importa — que CADA chip mova o estudo.**
//!
//! ⛔⛔ **Este gate existe porque esta linha pagou o defeito há dois dias.** Em 2026-09-01 o Enio
//! devolveu uma foto do trilho de ferramentas: os chips `MOVE`/`ROT`/`SCALE` estavam pintados,
//! clicáveis, com luz de rádio exclusiva — e **zero leitores em toda a árvore**. O sintoma de um
//! controlo morto e o de um ausente são o mesmo, e nenhuma sonda deste repo pergunta *"o valor que
//! este controlo escreve chega a alguém que decide?"*.
//!
//! A bancada tem sete chips. Se um deles não mexer no `WidgetLabState`, ele é um botão morto — e
//! num painel cuja razão de existir é comparar desenhos, um chip morto faz o Enio comparar duas
//! vezes a mesma coisa e concluir que os desenhos são iguais. *É o pior sítio possível para um
//! controlo mudo.*

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_panel_widget_lab::WidgetLabPanel;
use ph2d_panel_widget_lab::state::WidgetLabState;
use ph2d_ui_testkit::MockPanelHost;

/// Uma fotografia do estado que o pintor de facto lê. ⚠️ **Derivada dos campos**, não uma lista de
/// nomes: um eixo novo no `WidgetLabState` que ninguém acrescente aqui deixa de ser vigiado, e o
/// compilador avisa porque a desestruturação é exaustiva.
fn snapshot(s: &WidgetLabState) -> (usize, usize, usize, usize, bool, bool) {
    let WidgetLabState {
        rect: _,
        design,
        accent,
        radius,
        density,
        decorator,
        compare,
    } = s;
    (
        ph2d_panel_widget_lab::BoxDesign::ALL
            .iter()
            .position(|d| d == design)
            .unwrap_or(usize::MAX),
        *accent,
        *radius,
        *density,
        *decorator,
        *compare,
    )
}

/// ⭐⭐⭐ **Os sete chips mexem no estudo.** Um que não mexa é um botão morto.
#[test]
fn every_lab_control_moves_the_study() {
    let controls = [
        ("LAB_VARIANT_NEXT", ids::LAB_VARIANT_NEXT),
        ("LAB_VARIANT_PREV", ids::LAB_VARIANT_PREV),
        ("LAB_ACCENT_CYCLE", ids::LAB_ACCENT_CYCLE),
        ("LAB_RADIUS_CYCLE", ids::LAB_RADIUS_CYCLE),
        ("LAB_DENSITY_CYCLE", ids::LAB_DENSITY_CYCLE),
        ("LAB_DECORATOR_TOGGLE", ids::LAB_DECORATOR_TOGGLE),
        ("LAB_COMPARE_TOGGLE", ids::LAB_COMPARE_TOGGLE),
    ];
    let mut dead = Vec::new();
    for (name, id) in controls {
        let mut host = MockPanelHost::with_panel::<WidgetLabPanel>();
        let mut state = WidgetLabState::default();
        let before = snapshot(&state);
        let outcome = host.apply_panel_event::<WidgetLabPanel>(&mut state, WidgetEvent::Click(id));
        let after = snapshot(&state);
        if outcome != EventOutcome::Consumed || before == after {
            dead.push(name);
        }
    }
    assert!(
        dead.is_empty(),
        "chips da bancada que NAO mexem no estudo (botao morto): {dead:?}\n\
         \u{26a0} Um chip mudo num painel de comparacao faz comparar duas vezes o mesmo desenho."
    );
}

/// ⚠️ **O controlo positivo do gate acima.** Sem ele, um `snapshot` que devolvesse sempre o mesmo
/// tuplo (ou um `apply_event` que devolvesse `Consumed` para tudo) faria a lista de mortos ficar
/// vazia por vácuo. Um id que a bancada não conhece tem de ser IGNORADO e não mover nada.
#[test]
fn an_id_the_bench_does_not_know_moves_nothing() {
    let mut host = MockPanelHost::with_panel::<WidgetLabPanel>();
    let mut state = WidgetLabState::default();
    let before = snapshot(&state);
    let outcome =
        host.apply_panel_event::<WidgetLabPanel>(&mut state, WidgetEvent::Click(ids::GAL_CLOSE));
    assert_eq!(
        outcome,
        EventOutcome::Ignored,
        "a bancada consumiu um id que nao e' dela — o `else` final do event.rs caiu"
    );
    assert_eq!(
        before,
        snapshot(&state),
        "um id estranho mexeu no estudo — a sonda do gate irmao esta' a medir ruido"
    );
}

/// A linha do menu *Window* abre a bancada.
#[test]
fn the_window_menu_row_opens_the_bench() {
    let mut host = MockPanelHost::with_panel::<WidgetLabPanel>();
    let mut state = WidgetLabState::default();
    assert!(
        !host.panel_visible(WidgetLabPanel::ID),
        "pre-condicao: a bancada nasce fechada"
    );
    let outcome = host.apply_panel_event::<WidgetLabPanel>(
        &mut state,
        WidgetEvent::Click(ids::TOPBAR_WIDGET_LAB),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert!(
        host.panel_visible(WidgetLabPanel::ID),
        "a linha do menu foi consumida e a bancada nao abriu — a costura esta' morta"
    );
}

/// E o X fecha-a.
#[test]
fn lab_close_click_flips_visibility_off() {
    let mut host = MockPanelHost::with_panel::<WidgetLabPanel>();
    let mut state = WidgetLabState::default();
    host.set_panel_visible(WidgetLabPanel::ID, true);
    let outcome =
        host.apply_panel_event::<WidgetLabPanel>(&mut state, WidgetEvent::Click(ids::LAB_CLOSE));
    assert_eq!(outcome, EventOutcome::Consumed);
    assert!(!host.panel_visible(WidgetLabPanel::ID));
}

/// ⭐ **A bancada nasce a mostrar a coluna de animação e o «hoje».**
///
/// ⚠️ Não é gosto: as duas são decisões do Enio (2026-09-01), e um default que as contradiz
/// obriga-o a repô-las a cada abertura — que é como uma decisão se perde sem ninguém a revogar.
#[test]
fn the_bench_opens_honouring_the_decisions_already_taken() {
    let s = WidgetLabState::default();
    assert!(
        s.decorator,
        "a coluna de animacao nasce DESLIGADA — contradiz «em todas as propriedades animaveis»"
    );
    assert!(
        s.compare,
        "a comparacao com o widget de hoje nasce DESLIGADA — sem ela nao se ve' se melhoramos"
    );
}

/// ⭐⭐⭐ **A caixa VIVA arrasta mesmo** — e este gate é a **prova medida** que a entrada dela no
/// `NO_CONSUMER_PENDING` do `the_painted_control_reaches_a_consumer` exige.
///
/// ⚠️ **Por que aquela régua não a vê.** Ela procura términos POSITIVOS — `id == ids::X`, um braço
/// de `match`, uma chave de tabela. A caixa viva não tem nenhum **de propósito**: ela é registada
/// como [`InteractiveState::Slider`] e quem a move é o despacho de ponteiro GENÉRICO
/// (`interaction/dispatch/pointer_*`), que nunca nomeia um id — é isso que faz o gesto da bancada
/// ser o gesto do produto em vez de uma imitação. *Um consumidor genérico lê-se exactamente como
/// consumidor nenhum.*
///
/// ⇒ o término existe, é o `paint.rs` a ler `slider_visual` e a pintar o preenchimento; o que falta
/// à régua é ver POR DENTRO do despacho. Esta é a mesma família do `HIER_SEARCH` que já está lá.
#[test]
fn the_live_box_actually_drags() {
    use bumpalo::Bump;
    use ph2d_editor_core::interaction::{HitIndex, WidgetStore, dispatch_pointer};
    use ph2d_editor_core::widget::SliderState;
    use ph2d_editor_core::zones::Rect;
    use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};

    let mut store = WidgetStore::with_capacity(16);
    // ⚠️ Pelo `populate` do painel, não por um `register` escrito aqui: o que se quer provar é que
    // **o registo do produto** produz uma caixa arrastável, não que um slider qualquer arrasta.
    <WidgetLabPanel as Panel>::populate(&mut store);

    let mut hits = HitIndex::new();
    hits.register(ids::LAB_LIVE_BOX, Rect::new(0.0, 0.0, 100.0, 20.0));
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        PointerEvent {
            x: 75.0,
            y: 10.0,
            pressure: 1.0,
            kind: PointerKind::Down,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        },
        &arena,
    );

    let (state, v) = store
        .slider(ids::LAB_LIVE_BOX)
        .expect("a caixa viva nao esta' registada como Slider — o `populate` do painel mudou");
    assert_eq!(
        state,
        SliderState::Dragging,
        "premir sobre a caixa viva nao a po^s a arrastar"
    );
    assert!(
        (v - 0.75).abs() < 0.01,
        "premir a 75% da largura devia po^r o valor em 0,75; deu {v} — a caixa viva esta' MORTA \
         e a bancada mede um desenho que nao responde ao dedo"
    );
}
