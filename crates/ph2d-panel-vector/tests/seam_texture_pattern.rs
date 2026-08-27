//! Seam do **TEXTURE PATTERN** (plano 33, W5) — o chip *Tile* está vivo sob o ponteiro.
//!
//! O gesto é REAL (Down+Up sobre o rectângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: ⚠️ o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — é a lacuna que já deixou 36 células da matriz de física e dez chips do Painter
//! *pintados, hit-registrados e mortos sob o ponteiro*. Foi também o 2.º report do Enio sobre os
//! chips da booleana em 26/08: **um controlo nunca pintado e um morto sob o dedo dão o MESMO
//! report.**
//!
//! As duas metades de cada gate são independentes: sair do `populate_ops` mata a primeira (o
//! ponteiro não vira Click), sair do `event_clicks` mata a segunda (o Click não chega ao bus).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{FillKind, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids, state};
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

fn click_reaches_bus(id: ph2d_a11y::NodeId, what: &str) {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "o ponteiro sobre {what} nao virou Click - ele esta' desenhado e nao existe para o \
         dispatcher (falta o `register` no populate_ops)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id
        )),
        "o Click de {what} nao chegou ao bus - ele acende sob o mouse e nao faz nada (falta a \
         linha na allowlist do event_clicks)"
    );
}

/// **O chip `Tile` está vivo sob o ponteiro e chega ao bus.**
#[test]
fn the_pattern_chip_is_reachable_and_reaches_the_bus() {
    state::set_current_fill(Some(FillKind::Solid), None);
    click_reaches_bus(ids::VECTOR_FILL_KIND_PATTERN, "o chip Tile");
}

/// ⚠️ **A fileira inteira continua viva** — acrescentar o 5.º chip encolheu TODOS os outros (de
/// `58,5` para `45,6` px, medido), e um rectângulo mal calculado mata o vizinho sem mudar o
/// desenho de nada.
#[test]
fn adding_the_fifth_chip_left_the_other_four_alive() {
    state::set_current_fill(Some(FillKind::Solid), None);
    for (id, what) in [
        (ids::VECTOR_FILL_KIND_SOLID, "o chip Solid"),
        (ids::VECTOR_FILL_KIND_LINEAR, "o chip Linear"),
        (ids::VECTOR_FILL_KIND_RADIAL, "o chip Radial"),
        (ids::VECTOR_FILL_KIND_MULTI, "o chip Multi"),
    ] {
        click_reaches_bus(id, what);
    }
}

/// ⚠️ **Os cinco chips não se sobrepõem** — em NENHUM dos dois eixos.
///
/// ⚠️⚠️ **A 1.ª redacção deste gate comparava só o eixo X, e reprovou o produto certo.** A fileira
/// passou a usar o `paint_segmented_group_adaptive`, que **REFLUI** quando os chips não cabem: dois
/// chips em LINHAS diferentes partilham a mesma faixa de `x` por construção, e isso não é
/// sobreposição nenhuma. *Uma régua de um eixo só chama de colisão o que é apenas uma quebra de
/// linha.*
#[test]
fn the_five_chips_do_not_overlap() {
    state::set_current_fill(Some(FillKind::Solid), None);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let mut rects: Vec<(&str, Rect)> = Vec::new();
    for (id, name) in [
        (ids::VECTOR_FILL_KIND_SOLID, "Solid"),
        (ids::VECTOR_FILL_KIND_LINEAR, "Linear"),
        (ids::VECTOR_FILL_KIND_RADIAL, "Radial"),
        (ids::VECTOR_FILL_KIND_MULTI, "Multi"),
        (ids::VECTOR_FILL_KIND_PATTERN, "Tile"),
    ] {
        let r = host
            .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .expect("chip pintado");
        rects.push((name, r));
    }
    for i in 0..rects.len() {
        for j in (i + 1)..rects.len() {
            let (an, a) = rects[i];
            let (bn, b) = rects[j];
            let x = a.x < b.x + b.w - 1e-3 && b.x < a.x + a.w - 1e-3;
            let y = a.y < b.y + b.h - 1e-3 && b.y < a.y + a.h - 1e-3;
            assert!(
                !(x && y),
                "os chips `{an}` e `{bn}` sobrepoem-se: {a:?} e {b:?}"
            );
        }
    }
}

/// ⭐ **Sonda: quantas LINHAS a fileira de tipo de preenchimento ocupa.** Ela reflui quando não
/// cabe, e saber em que ponto isso acontece é o que decide se cabe um 6.º chip.
#[test]
#[ignore = "sonda: imprime a disposicao, nao afirma nada"]
fn measure_the_fill_kind_row_layout() {
    state::set_current_fill(Some(FillKind::Solid), None);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    for (id, name) in [
        (ids::VECTOR_FILL_KIND_SOLID, "Solid"),
        (ids::VECTOR_FILL_KIND_LINEAR, "Linear"),
        (ids::VECTOR_FILL_KIND_RADIAL, "Radial"),
        (ids::VECTOR_FILL_KIND_MULTI, "Multi"),
        (ids::VECTOR_FILL_KIND_PATTERN, "Tile"),
    ] {
        let r = host
            .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .expect("chip pintado");
        println!(
            "{name:>7}: x {:.1} y {:.1} w {:.1} h {:.1}",
            r.x, r.y, r.w, r.h
        );
    }
}
