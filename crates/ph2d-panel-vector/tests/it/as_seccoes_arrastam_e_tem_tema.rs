//! ⭐⭐⭐ **As secções do painel Vector abrem o menu de TEMA e ARRASTAM pela pega** — o gesto REAL,
//! pelo despachante, com os rects que a pintura registou (ordem do dono, 2026-09-30: *«siga com os
//! outros painéis»*, depois de as duas coisas nascerem só no Inspector).
//!
//! ⚠️ Uma lei de ordem verde numa função pura não diz nada sobre as três costuras que este gate
//! percorre: o título ser um alvo do menu de SECÇÃO (e não o de notas), a pega estar viva sob o
//! rato e vencer o cabeçalho que dobra, e a ordem gravada chegar à PINTURA seguinte.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::VectorPanel;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 600.0,
    h: 2400.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, button: PointerButton, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// As secções do painel à vista, pela ordem pintada — `(id, rect do cabeçalho)`.
fn a_vista(rects: &[(NodeId, Rect)]) -> Vec<(NodeId, Rect)> {
    let mut v: Vec<(NodeId, Rect)> = rects
        .iter()
        .filter(|(id, _)| ph2d_panel_vector::ids::VECTOR_SECTIONS.contains(id))
        .copied()
        .collect();
    v.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));
    v.dedup_by_key(|(id, _)| *id);
    v
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Rect {
    rects
        .iter()
        .rev()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("{id:?} nao foi registado pela pintura"))
}

/// ⭐⭐ **O botão direito no título de uma secção do Vector abre o menu de TEMA dela** — e não o de
/// notas, que era o que abria até 2026-09-30. *Mutação: o despacho sem o `hit_index.is_section` ⇒
/// o menu aberto é o `CreateNote`.*
#[test]
fn o_botao_direito_no_titulo_abre_o_menu_de_tema_da_seccao() {
    let mut h = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let r = h.paint::<VectorPanel>(&mut st, VIEWPORT);
    let vista = a_vista(&r);
    assert!(
        vista.len() >= 2,
        "fixtura: o painel tem de pintar duas secções ou mais (pintou {})",
        vista.len()
    );
    let (alvo, cab) = vista[1];
    // ⚠️ Num ponto do título à ESQUERDA da pega — a pega também responde, mas é outro alvo.
    let (x, y) = (cab.x + cab.w * 0.25, cab.y + cab.h * 0.5);
    let _ = h.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Secondary,
        x,
        y,
        SEC,
    ));
    let menu = h.store().context_menu().map(|m| m.kind);
    assert!(
        matches!(menu, Some(ContextMenuKind::SectionOutline { section }) if section == alvo),
        "o botao direito no titulo abriu {menu:?} em vez do menu de tema da seccao"
    );
}

/// ⭐⭐ **Arrastar a pega da segunda secção para cima da primeira põe-na no topo** — e um clique
/// PARADO na pega não reordena nem dobra. *Mutações: o `RowCtx` sem o `regista_cabecalho` ⇒ a pega
/// não existe no índice; o corpo a ignorar a ordem (`corre` pela natural) ⇒ a segunda continua em
/// baixo.*
#[test]
fn arrastar_a_pega_poe_a_seccao_onde_se_larga() {
    let mut h = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let r = h.paint::<VectorPanel>(&mut st, VIEWPORT);
    let vista = a_vista(&r);
    assert!(vista.len() >= 2, "fixtura: duas secções à vista");
    let (primeira, cab1) = vista[0];
    let (segunda, _) = vista[1];
    let pega = rect_of(&r, ph2d_editor_core::ids::grip_de(segunda));
    let dobrada = h.store().is_collapsed(segunda);

    let (px, py) = centro(pega);
    let _ = h.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Primary,
        px,
        py,
        SEC,
    ));
    let _ = h.dispatch_pointer_event(pointer(
        PointerKind::Up,
        PointerButton::Primary,
        px,
        py,
        SEC + 1,
    ));
    assert!(
        h.store().section_order().is_empty(),
        "um clique parado na pega reordenou"
    );
    assert_eq!(
        h.store().is_collapsed(segunda),
        dobrada,
        "o Down na pega caiu no cabecalho e dobrou a seccao"
    );

    let alvo_y = cab1.y + 1.0;
    let _ = h.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Primary,
        px,
        py,
        2 * SEC,
    ));
    let _ = h.dispatch_pointer_event(pointer(
        PointerKind::Move,
        PointerButton::Primary,
        px,
        alvo_y,
        2 * SEC + 1,
    ));
    let _ = h.dispatch_pointer_event(pointer(
        PointerKind::Up,
        PointerButton::Primary,
        px,
        alvo_y,
        2 * SEC + 2,
    ));
    assert_eq!(
        h.store().section_order().first(),
        Some(&segunda),
        "a ordem gravada nao pos a seccao arrastada no topo"
    );
    let depois = a_vista(&h.paint::<VectorPanel>(&mut st, VIEWPORT));
    assert_eq!(
        depois.first().map(|(id, _)| *id),
        Some(segunda),
        "a pintura seguinte nao seguiu a ordem gravada"
    );
    assert!(
        depois.iter().any(|(id, _)| *id == primeira),
        "a primeira seccao desapareceu"
    );
}
