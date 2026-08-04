//! Seam dos **COMPONENTES** (plano UI/UX W5) — os quatro verbos estão vivos sob o MOUSE, chegam
//! ao barramento, e cada um aparece **só onde faz sentido**.
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — a lacuna que já deixou as 36 células da matriz de física e os dez chips de ferramenta
//! do Painter *pintados, hit-registrados e mortos sob o ponteiro*.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{ComponentState, InstancePiece, VectorPanelState};
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

fn clear() {
    state::set_component_state(None);
    // ⚠️ As peças são thread-local como o resto: sem esta linha um teste que publica uma lista
    // envenena os seguintes, e o gate da AUSÊNCIA ficaria verde por herdar o estado do vizinho.
    state::set_instance_pieces(Vec::new(), 0);
}

/// Publica `n` peças, a primeira delas com cor própria.
fn pieces(n: usize, beyond: usize) {
    let rows = (0..n)
        .map(|i| InstancePiece {
            name: format!("Piece {i}"),
            colour: [10, 20, 30, 255],
            visible: true,
            overridden: i == 0,
        })
        .collect();
    state::set_instance_pieces(rows, beyond);
}

/// Uma forma comum (só o *Create* faz sentido).
fn plain() -> ComponentState {
    ComponentState::default()
}

/// Um mestre (só o *Place*).
fn main_shape() -> ComponentState {
    ComponentState {
        is_main: true,
        ..ComponentState::default()
    }
}

/// Uma instância COM overrides (Detach + Reset).
fn instance_with_overrides() -> ComponentState {
    ComponentState {
        is_instance: true,
        has_overrides: true,
        ..ComponentState::default()
    }
}

fn rect_under(st: ComponentState, id: ph2d_a11y::NodeId) -> Option<Rect> {
    state::set_component_state(Some(st));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
}

/// Clica de verdade no widget `id` e exige que o Click chegue ao barramento.
fn click_reaches_bus(st: ComponentState, id: ph2d_a11y::NodeId, what: &str) {
    state::set_component_state(Some(st));
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

/// **Os QUATRO verbos estão vivos e chegam ao bus**, cada um no estado em que é oferecido.
#[test]
fn all_four_component_verbs_are_reachable_and_reach_the_bus() {
    clear();
    click_reaches_bus(plain(), ids::VECTOR_COMPONENT_CREATE, "Create Component");
    click_reaches_bus(main_shape(), ids::VECTOR_COMPONENT_PLACE, "Place Instance");
    click_reaches_bus(
        instance_with_overrides(),
        ids::VECTOR_COMPONENT_DETACH,
        "Detach Instance",
    );
    click_reaches_bus(
        instance_with_overrides(),
        ids::VECTOR_COMPONENT_RESET,
        "Reset Overrides",
    );
    clear();
}

/// **Cada verbo aparece SÓ onde faz sentido** — a metade da AUSÊNCIA.
///
/// ⚠️ Sem ela, o gate acima ficaria verde sobre uma seção que pinta os quatro botões sempre, três
/// deles inertes — que é o botão-morto que este repo persegue e que ensina o artista a duvidar
/// dos outros.
#[test]
fn each_verb_appears_only_where_it_makes_sense() {
    clear();
    // Uma forma comum: Create sim, os outros três não.
    assert!(rect_under(plain(), ids::VECTOR_COMPONENT_CREATE).is_some());
    for id in [
        ids::VECTOR_COMPONENT_PLACE,
        ids::VECTOR_COMPONENT_DETACH,
        ids::VECTOR_COMPONENT_RESET,
    ] {
        assert!(
            rect_under(plain(), id).is_none(),
            "um verbo de instância foi oferecido sobre uma forma comum"
        );
    }
    // Um mestre: Place sim, Create não (ele já é um).
    assert!(rect_under(main_shape(), ids::VECTOR_COMPONENT_PLACE).is_some());
    assert!(rect_under(main_shape(), ids::VECTOR_COMPONENT_CREATE).is_none());
    // Uma instância LIMPA: Detach sim, Reset não — um reset que não reseta nada é um clique que
    // não faz nada, e o artista não tem como o saber antes de o dar.
    let clean = ComponentState {
        is_instance: true,
        ..ComponentState::default()
    };
    assert!(rect_under(clean, ids::VECTOR_COMPONENT_DETACH).is_some());
    assert!(rect_under(clean, ids::VECTOR_COMPONENT_RESET).is_none());
    clear();
}

/// **Os DOIS verbos da W5b chegam ao bus** — *Update Main* e *Swap Main*.
#[test]
fn update_main_and_swap_are_reachable_and_reach_the_bus() {
    clear();
    click_reaches_bus(
        instance_with_overrides(),
        ids::VECTOR_COMPONENT_UPDATE_MAIN,
        "Update Main",
    );
    // ⚠️ O *Swap* é oferecido a uma instância LIMPA também: trocar de mestre não pede diferença
    // nenhuma, e gateá-lo em `has_overrides` tê-lo-ia deixado inalcançável no caso comum.
    let clean = ComponentState {
        is_instance: true,
        ..ComponentState::default()
    };
    click_reaches_bus(clean, ids::VECTOR_COMPONENT_SWAP, "Swap Main");
    clear();
}

/// **A LINHA de peça está viva sob o mouse e chega ao bus** — a porta do override (W5b).
///
/// ⚠️ É o gate que a W5a não podia ter: `OverrideSlot` existia com gates e **nada no editor podia
/// produzir um**. Se este ficar vermelho, o *Reset Overrides* volta a ser um botão que nunca pode
/// ser preciso.
#[test]
fn the_piece_switch_is_alive_and_reaches_the_bus() {
    clear();
    pieces(2, 0);
    click_reaches_bus(
        ComponentState {
            is_instance: true,
            ..ComponentState::default()
        },
        ids::vector_instance_piece_show_id(1),
        "o interruptor da 2a peca",
    );
    clear();
}

/// **A swatch de uma peça é alvo de PICKER, não botão** — um id só tem um tipo no store, e
/// registá-la como botão faria o Down acender o widget e **nunca abrir o OKLCH**.
#[test]
fn the_piece_colour_swatch_is_painted_and_is_a_picker_target() {
    clear();
    pieces(1, 0);
    state::set_component_state(Some(ComponentState {
        is_instance: true,
        ..ComponentState::default()
    }));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let id = ids::vector_instance_piece_colour_id(0);
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .is_some(),
        "a swatch da peca tem de ser pintada com area clicavel"
    );
    assert!(
        host.store().is_picker_swatch(id),
        "a swatch tem de estar no conjunto de picker (senao o Down nao abre o OKLCH)"
    );
    clear();
}

/// **Sem peças publicadas não há linhas** — a metade da AUSÊNCIA.
///
/// ⚠️ Sem ela o gate acima ficaria verde sobre um painel que pinta o TETO inteiro de linhas, com
/// interruptores de peças que não existem.
#[test]
fn a_main_without_pieces_paints_no_piece_rows() {
    clear();
    state::set_component_state(Some(ComponentState {
        is_instance: true,
        ..ComponentState::default()
    }));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    for row in 0..ids::MAX_INSTANCE_PIECES {
        assert!(
            host.painted_rect::<VectorPanel>(
                &mut st,
                VIEWPORT,
                ids::vector_instance_piece_show_id(row)
            )
            .is_none(),
            "a linha {row} foi pintada sem peca publicada"
        );
    }
    clear();
}

/// **Só as peças publicadas ganham linha** — o teto regista, o corpo pinta a contagem VIVA.
#[test]
fn only_the_published_pieces_get_a_row() {
    clear();
    pieces(2, 0);
    let st_c = ComponentState {
        is_instance: true,
        ..ComponentState::default()
    };
    assert!(rect_under(st_c, ids::vector_instance_piece_show_id(1)).is_some());
    assert!(
        rect_under(st_c, ids::vector_instance_piece_show_id(2)).is_none(),
        "uma linha foi pintada para uma peca que nao existe"
    );
    clear();
}

/// **Sem estado publicado a seção não existe.**
#[test]
fn the_section_is_not_painted_without_a_selection() {
    clear();
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    for id in [
        ids::VECTOR_SECTION_COMPONENT,
        ids::VECTOR_COMPONENT_CREATE,
        ids::VECTOR_COMPONENT_PLACE,
    ] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                .is_none(),
            "a secao de componente foi pintada sem selecao"
        );
    }
}
