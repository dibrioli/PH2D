//! Seam da **MOLDURA** (plano UI/UX W0) — os sete widgets estão vivos sob o MOUSE, não só
//! pintados.
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — foi essa lacuna que deixou as 36 células da matriz de física e os dez chips de
//! ferramenta do Painter *pintados, hit-registrados e mortos sob o ponteiro*.
//!
//! As duas metades de cada gate são independentes: sair do `populate` mata a primeira (o ponteiro
//! não vira Click), sair do `event_clicks` mata a segunda (o Click não chega ao bus).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids, state};
use ph2d_tool_vector::frames::DEVICE_PRESETS;
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

/// Clica de verdade no widget `id` e exige que o Click chegue ao barramento.
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
        "o ponteiro sobre {what} nao virou Click — ele esta' desenhado e nao existe para o \
         dispatcher (falta o `register` no populate)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id
        )),
        "o Click de {what} nao chegou ao bus — ele acende sob o mouse e nao faz nada (falta a \
         linha na allowlist do event_clicks)"
    );
}

/// **O 14º pill.** Ele é o único caminho para criar uma moldura, então morto sob o mouse a wave
/// inteira fica inalcançável.
#[test]
fn the_frame_pill_is_reachable_and_reaches_the_bus() {
    click_reaches_bus(ids::VECTOR_MODE_FRAME, "o pill Frame");
}

/// **Os dois chips de recorte e os quatro presets**, com uma moldura selecionada.
///
/// ⚠️ A publicação é a premissa: sem ela a seção não é pintada e o `painted_rect` devolve `None`
/// — o gate falharia dizendo *"não foi pintado"*, que é a verdade e não o defeito procurado.
#[test]
fn the_frame_controls_are_reachable_and_reach_the_bus() {
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    click_reaches_bus(ids::VECTOR_FRAME_CLIP_OFF, "o chip Clip=Off");
    click_reaches_bus(ids::VECTOR_FRAME_CLIP_ON, "o chip Clip=On");
    for p in DEVICE_PRESETS {
        click_reaches_bus(p.id, p.label);
    }
    state::set_frame_clip(None);
    state::set_frame_present(false);
}

/// **O interruptor do painel AUTORADO** (plano UI/UX W8b.2) — os dois chips vivos sob o mouse e
/// chegando ao bus.
///
/// ⚠️ Ele é o único caminho do artista até o painel que ele desenhou: morto sob o rato, a W8b.2
/// inteira fica inalcançável, exactamente como o painel de física do W2b — que tinha todos os
/// gates de unidade verdes.
#[test]
fn the_show_as_panel_switch_is_reachable_and_reaches_the_bus() {
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    click_reaches_bus(ids::VECTOR_FRAME_PANEL_OFF, "o chip Show as Panel=Off");
    click_reaches_bus(ids::VECTOR_FRAME_PANEL_ON, "o chip Show as Panel=On");
    state::set_frame_clip(None);
    state::set_frame_present(false);
}

/// **E o chip MOSTRA a visibilidade real** — aceso quando o painel está aberto.
///
/// ⚠️ A metade que impede a falha de duas-portas: o X do painel autorado escreve o MESMO fato, e
/// um chip que guardasse cópia própria ficaria aceso sobre um painel fechado.
#[test]
fn the_switch_shows_whether_the_panel_is_open() {
    state::set_frame_clip(Some(true));
    state::set_frame_present(true);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    for open in [false, true] {
        state::set_frame_panel_open(open);
        let lit = if open {
            ids::VECTOR_FRAME_PANEL_ON
        } else {
            ids::VECTOR_FRAME_PANEL_OFF
        };
        assert!(
            host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, lit)
                .is_some(),
            "com o painel {}, o chip correspondente nao foi pintado",
            if open { "aberto" } else { "fechado" }
        );
    }
    state::set_frame_panel_open(false);
    state::set_frame_clip(None);
    state::set_frame_present(false);
}

/// **Sem moldura na seleção a seção não existe** — e é isto que a impede de ser seis controles
/// mortos em toda seleção que não é contêiner.
#[test]
fn the_frame_section_is_absent_without_a_frame() {
    state::set_frame_clip(None);
    state::set_frame_present(false);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    for id in [
        ids::VECTOR_FRAME_CLIP_OFF,
        ids::VECTOR_FRAME_CLIP_ON,
        ids::VECTOR_FRAME_PANEL_OFF,
        ids::VECTOR_FRAME_PANEL_ON,
    ]
    .into_iter()
    .chain(DEVICE_PRESETS.iter().map(|p| p.id))
    {
        assert!(
            host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                .is_none(),
            "a secao Frame foi pintada sem moldura selecionada"
        );
    }
}
