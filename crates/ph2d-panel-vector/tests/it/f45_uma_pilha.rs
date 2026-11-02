//! ⭐⭐ **F45 — Fill & Stroke numa só pilha, e o arrasto pelo ⚙** (dono 01/11: *«a fila de
//! sobreposição funcione na ordem em que são colocadas […] clicar e arrastar o ícone de config deve
//! ser usado para mudar a ordem»*).
//!
//! O arrasto corre pelo ponteiro REAL (Down → Move → pintura → Up), como no app: a pintura do meio
//! é a que abre o vão, põe a linha a voar e fixa a queda que o largar aplica.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{self, Appearance, PaintRow, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_tool_vector::ids as tv;
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vec_scene::{Place, StackItem};

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

fn camada(is_fill: bool) -> PaintRow {
    PaintRow {
        align: ph2d_vec_scene::StrokeAlign::Centre,
        ramp: None,
        is_fill,
        color: [0x33, 0x66, 0xCC, 0xFF],
        width: 2.0,
        enabled: true,
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        offset: [0.0, 0.0],
        dilate: 0.0,
        dilate_join: 1,
        mixed: Default::default(),
    }
}

/// Uma forma com uma camada de traço e a pilha em `order` (do chão ao topo).
fn forma(order: Vec<StackItem>) {
    state::set_current_appearance(Some(Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers: vec![camada(false)],
        order,
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
    }));
    state::set_base_paints(Some([state::BasePaint::Shown; 2]));
    state::set_stroke_present(Some(true));
    state::set_current_fill(Some(ph2d_panel_vector::FillKind::Solid), None);
}

fn limpa() {
    state::set_current_appearance(None);
    state::set_base_paints(None);
    state::set_stroke_present(None);
    state::set_current_fill(None, None);
    state::close_open_layer();
    let _ = state::take_paint_move();
}

fn y_de(h: &mut MockPanelHost, id: NodeId) -> f32 {
    h.painted_rect::<VectorPanel>(&mut VectorPanelState, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{id:?} não foi pintado"))
        .y
}

fn ponteiro(h: &mut MockPanelHost, kind: PointerKind, x: f32, y: f32, t: u128) {
    let evs = h.dispatch_pointer_event(PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: t,
    });
    for ev in evs {
        h.apply_panel_event::<VectorPanel>(&mut VectorPanelState, ev);
    }
}

/// ⭐ **A ordem publicada manda na lista**: com o preenchimento de base no TOPO da pilha, a linha dele
/// é a de cima — por cima da camada de traço e do traço de base. *Mutação: ignorar `order` ⇒ o Fill
/// fica em baixo.*
#[test]
fn a_fill_on_top_of_the_stack_is_the_top_row() {
    forma(vec![
        StackItem::BaseStroke,
        StackItem::Entry(0),
        StackItem::BaseFill,
    ]);
    let mut h = crate::host_aberto();
    let fill = y_de(&mut h, tv::VECTOR_FILL_SWATCH);
    let camada = y_de(&mut h, ids::vector_paint_swatch_id(0));
    let traco = y_de(&mut h, tv::VECTOR_STROKE_SWATCH);
    assert!(fill < camada && camada < traco, "{fill} {camada} {traco}");
    limpa();
}

/// ⭐⭐ **Arrastar o ⚙ do preenchimento de base para o topo**: a meio, a linha sai do lugar (voa) e as
/// outras abrem o vão; ao largar, o painel pede o verbo `Move` com «Fill por cima da camada».
/// *Mutação: não registar os ⚙ na família ⇒ não há arrasto; trocar Above/Below ⇒ a queda errada.*
#[test]
fn dragging_the_gear_reorders_the_stack() {
    forma(Vec::new());
    let mut h = crate::host_aberto();
    // a 1.ª pintura publica os ⚙ arrastáveis (o passe diferido)
    let gear = h
        .painted_rect::<VectorPanel>(&mut VectorPanelState, VIEWPORT, ids::VECTOR_FILL_ROW)
        .expect("o ⚙ do Fill de base");
    let topo = y_de(&mut h, ids::vector_paint_row_id(0));
    let (x, y) = (gear.x + gear.w * 0.5, gear.y + gear.h * 0.5);
    ponteiro(&mut h, PointerKind::Down, x, y, 1);
    ponteiro(&mut h, PointerKind::Move, x, topo + 1.0, 2);
    let _ = h.paint::<VectorPanel>(&mut VectorPanelState, VIEWPORT);
    assert!(
        h.painted_rect::<VectorPanel>(&mut VectorPanelState, VIEWPORT, ids::VECTOR_FILL_ROW)
            .is_none(),
        "a linha agarrada voa: o ⚙ dela sai do lugar"
    );
    ponteiro(&mut h, PointerKind::Up, x, topo + 1.0, 3);
    let pediu = h.drained_actions().into_iter().any(|a| {
        matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if id == ids::VECTOR_PAINT_MOVE
        )
    });
    assert!(pediu, "o largar não pediu o verbo Move");
    assert_eq!(
        state::take_paint_move(),
        Some((StackItem::BaseFill, Place::Above(StackItem::Entry(0))))
    );
    limpa();
}

/// O clique PARADO no ⚙ continua a abrir os ajustes (não é arrasto).
#[test]
fn a_still_click_on_the_gear_still_opens_it() {
    forma(Vec::new());
    let mut h = crate::host_aberto();
    crate::clica(&mut h, ids::VECTOR_FILL_ROW);
    assert_eq!(h.dropdown_is_open(ids::VECTOR_FILL_ROW), Some(true));
    assert_eq!(state::take_paint_move(), None);
    limpa();
}

/// O mesmo gesto para BAIXO: o Fill do meio largado sobre a linha de baixo (a 80 % dela) cai por
/// baixo do traço de base — a queda segue a ranhura sob o cursor.
#[test]
fn dragging_down_lands_below_the_row_under_the_cursor() {
    forma(vec![
        StackItem::BaseStroke,
        StackItem::BaseFill,
        StackItem::Entry(0),
    ]);
    let mut h = crate::host_aberto();
    let gear = h
        .painted_rect::<VectorPanel>(&mut VectorPanelState, VIEWPORT, ids::VECTOR_FILL_ROW)
        .expect("o ⚙ do Fill de base");
    let baixo = h
        .painted_rect::<VectorPanel>(&mut VectorPanelState, VIEWPORT, ids::VECTOR_STROKE_SETTINGS)
        .expect("o ⚙ do traço de base");
    let x = gear.x + gear.w * 0.5;
    let alvo = baixo.y + baixo.h * 0.8;
    ponteiro(&mut h, PointerKind::Down, x, gear.y + gear.h * 0.5, 1);
    ponteiro(&mut h, PointerKind::Move, x, alvo, 2);
    let _ = h.paint::<VectorPanel>(&mut VectorPanelState, VIEWPORT);
    ponteiro(&mut h, PointerKind::Up, x, alvo, 3);
    assert_eq!(
        state::take_paint_move(),
        Some((StackItem::BaseFill, Place::Below(StackItem::BaseStroke)))
    );
    limpa();
}

/// ⭐ **Arrastar a camada aberta fecha o painel dela** — a pilha renumera, e um `Dropdown` aberto
/// passaria a editar outra camada. A lei mora no voo: a linha agarrada sai da lista, o passe
/// diferido não a vê e fecha-a (`paint_side`). Medido no fecho: um `close_layers` a mais no largar
/// não mudava nada (o teste passava sem ele) — por isso não existe.
#[test]
fn dropping_closes_the_open_layer_panel() {
    forma(Vec::new());
    let mut h = crate::host_com(&[ids::vector_paint_row_id(0)]);
    assert_eq!(h.dropdown_is_open(ids::vector_paint_row_id(0)), Some(true));
    // o ⚙ da PRÓPRIA camada aberta (premi-lo não é «clique fora»), levado para o fundo
    let gear = h
        .painted_rect::<VectorPanel>(&mut VectorPanelState, VIEWPORT, ids::vector_paint_row_id(0))
        .expect("o ⚙ da camada 0");
    let fundo = y_de(&mut h, ids::VECTOR_FILL_ROW);
    let (x, y) = (gear.x + gear.w * 0.5, gear.y + gear.h * 0.5);
    ponteiro(&mut h, PointerKind::Down, x, y, 1);
    ponteiro(&mut h, PointerKind::Move, x, fundo + 1.0, 2);
    let _ = h.paint::<VectorPanel>(&mut VectorPanelState, VIEWPORT);
    ponteiro(&mut h, PointerKind::Up, x, fundo + 1.0, 3);
    assert_eq!(
        state::take_paint_move(),
        Some((StackItem::Entry(0), Place::Below(StackItem::BaseFill)))
    );
    assert_eq!(h.dropdown_is_open(ids::vector_paint_row_id(0)), Some(false));
    limpa();
}
