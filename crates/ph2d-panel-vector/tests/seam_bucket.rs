//! ⭐⭐⭐ Seam do pill **Bucket** (plano 40) — ele está vivo sob o MOUSE, o clique chega ao bus, e
//! o modo que ele arma é o dele.
//!
//! ⚠️ **Escrito ANTES do smoke**, e a razão é o report de 2026-09-01: um verbo cujo botão não fala
//! com ninguém lê-se exactamente como um motor partido, e o `seam_weld.rs` só nasceu depois de o
//! Enio ter pago essa ambiguidade. *Um pill novo tem quatro sítios independentes (id · pintura ·
//! registo · allowlist), e três deles falham em silêncio.*

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids};
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

/// **O pill é pintado, responde ao ponteiro, e o Click chega ao barramento.**
///
/// ⚠️ As três metades são independentes: sem a entrada em `paint_modes` ele não existe; sem a linha
/// em `populate_modes` ele pinta e o ponteiro não o vê; sem a linha em `event_clicks` o Click morre
/// dentro do painel.
#[test]
fn the_bucket_pill_is_alive_and_reaches_the_bus() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_MODE_BUCKET)
        .expect("o pill Bucket nao foi PINTADO com area clicavel");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == ids::VECTOR_MODE_BUCKET)),
        "o ponteiro sobre o Bucket nao virou Click — falta o `register` no populate_modes"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut st, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == ids::VECTOR_MODE_BUCKET
        )),
        "o Click do Bucket nao chegou ao bus — falta a linha na allowlist do event_clicks"
    );
}

/// ⭐ **E o clique arma o modo do BALDE, não o do vizinho.**
///
/// ⚠️ A cadeia de guardas do `apply_panel_event` é escrita à mão, e um `id` copiado da linha de
/// cima compila: o pill do Balde acenderia o Trim, e o artista veria a ferramenta errada com o
/// nome certo.
#[test]
fn clicking_the_pill_arms_the_bucket_mode() {
    use ph2d_editor_core::tool::Tool;
    let mut tool = ph2d_tool_vector::VectorTool::default();
    tool.handle_panel_event(PanelEvent::Click(ids::VECTOR_MODE_BUCKET));
    assert_eq!(
        tool.draw_config().mode,
        ph2d_tool_vector::DrawMode::Bucket,
        "o pill do Balde armou outro modo"
    );
}
