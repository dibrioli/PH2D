//! ⭐⭐⭐ **As secções do Tuning abrem o menu de TEMA e ARRASTAM pela pega** — o gesto REAL, pelo
//! despachante, com os rects que a pintura registou (ordem do dono, 2026-09-30: *«siga com os
//! outros painéis»*). Irmão do gate do Painter e do Vector; aqui as seis secções são MÓVEIS, e o
//! painel tem a SUA dobra (o clique esquerdo no título) — que tem de continuar a dobrar.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_wet_tuning::{WetTuningPanel, rows, set_current_brush, state};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

const SEC: u128 = 1_000_000_000;

/// Alto de propósito: o corpo pede mais de `1 300` px, e o gate quer as secções à vista (o clique
/// é recortado pelo corpo).
fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 2400.0)
}

/// As secções que se arrastam, pela ordem natural — os cinco grupos e o *Experimental*.
fn moveis() -> Vec<NodeId> {
    rows::SECTIONS
        .iter()
        .map(|s| s.header)
        .chain(std::iter::once(
            ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[5],
        ))
        .collect()
}

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

fn pinta(host: &mut MockPanelHost, st: &mut state::WetTuningPanelState) -> Vec<(NodeId, Rect)> {
    set_current_brush(Some(PainterTool::default().brush_settings()));
    host.paint::<WetTuningPanel>(st, viewport())
}

fn pintado() -> (
    MockPanelHost,
    state::WetTuningPanelState,
    Vec<(NodeId, Rect)>,
) {
    let mut host = MockPanelHost::with_panel::<WetTuningPanel>();
    host.set_panel_visible(WetTuningPanel::ID, true);
    let mut st = state::WetTuningPanelState;
    let rects = pinta(&mut host, &mut st);
    (host, st, rects)
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .rev()
        .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// As secções à vista, pela ordem pintada.
fn a_vista(rects: &[(NodeId, Rect)]) -> Vec<(NodeId, Rect)> {
    let mut v: Vec<(NodeId, Rect)> = moveis()
        .into_iter()
        .filter_map(|id| rect_of(rects, id).map(|r| (id, r)))
        .collect();
    v.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));
    v
}

/// ⚠️ Num ponto do título à ESQUERDA da pega, do repor e do olho — os três são outros alvos.
fn no_titulo(cab: Rect) -> (f32, f32) {
    (cab.x + cab.w * 0.25, cab.y + cab.h * 0.5)
}

/// ⭐⭐ **O botão direito no título de uma secção abre o menu de tema dela, e ela tem pega.**
/// *Mutação: o `header_row` a registar pelo `hit.register` de antes ⇒ o menu aberto não é o de
/// tema de secção e a pega não existe no índice.*
#[test]
fn uma_seccao_tem_menu_de_tema_e_pega() {
    let (mut host, _, rects) = pintado();
    let vista = a_vista(&rects);
    assert_eq!(
        vista.len(),
        moveis().len(),
        "fixtura: as seis secções têm de estar à vista (pintou {})",
        vista.len()
    );
    for (alvo, _) in &vista {
        assert!(
            rect_of(&rects, ph2d_editor_core::ids::grip_de(*alvo)).is_some(),
            "a secção {alvo:?} não registou a pega"
        );
    }
    // ⚠️ A do PAPER, que tem o repor E o olho no cabeçalho — o ponto do título tem de cair fora
    //    dos dois.
    let (alvo, cab) = vista[4];
    let (x, y) = no_titulo(cab);
    let _ = host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Secondary,
        x,
        y,
        SEC,
    ));
    let menu = host.store().context_menu().map(|m| m.kind);
    assert!(
        matches!(menu, Some(ContextMenuKind::SectionOutline { section }) if section == alvo),
        "o botão direito no título abriu {menu:?} em vez do menu de tema da secção"
    );
}

/// ⭐⭐ **Arrastar a pega da segunda secção para cima da primeira põe-na à frente** — e a pintura
/// seguinte segue a ordem gravada. *Mutação: o `PlanoCtx::corre` pela ordem natural ⇒ a segunda
/// continua atrás.*
#[test]
fn arrastar_a_pega_poe_a_seccao_onde_se_larga() {
    let (mut host, mut st, rects) = pintado();
    let vista = a_vista(&rects);
    assert!(vista.len() >= 2, "fixtura: duas secções à vista");
    let (primeira, cab1) = vista[0];
    let (segunda, _) = vista[1];
    let pega =
        rect_of(&rects, ph2d_editor_core::ids::grip_de(segunda)).expect("a segunda tem pega");
    let (px, py) = (pega.x + pega.w * 0.5, pega.y + pega.h * 0.5);
    let alvo_y = cab1.y + 1.0;
    for (kind, y, t) in [
        (PointerKind::Down, py, SEC),
        (PointerKind::Move, alvo_y, SEC + 1),
        (PointerKind::Up, alvo_y, SEC + 2),
    ] {
        let _ = host.dispatch_pointer_event(pointer(kind, PointerButton::Primary, px, y, t));
    }
    assert_eq!(
        host.store().section_order().first(),
        Some(&segunda),
        "a ordem gravada não pôs a secção arrastada à frente"
    );
    let depois = a_vista(&pinta(&mut host, &mut st));
    let pos = |id: NodeId| depois.iter().position(|(n, _)| *n == id);
    assert!(
        pos(segunda) < pos(primeira),
        "a pintura seguinte não seguiu a ordem gravada ({depois:?})"
    );
}

/// ⭐⭐ **O clique ESQUERDO no título continua a dobrar a secção** — a dobra é deste painel (o
/// `event` despacha-a), e o livro do quadro regista o título sob o MESMO id. *Mutação: o título
/// registado só na pega ⇒ o clique não chega ao `toggle_collapsed`.*
#[test]
fn o_clique_esquerdo_no_titulo_ainda_dobra() {
    let (mut host, mut st, rects) = pintado();
    let (alvo, cab) = a_vista(&rects)[1];
    assert!(!host.store().is_collapsed(alvo), "fixtura: nasce aberta");
    let (x, y) = no_titulo(cab);
    for ev in host.click_at(x, y) {
        let _ = host.apply_panel_event::<WetTuningPanel>(&mut st, ev);
    }
    assert!(
        host.store().is_collapsed(alvo),
        "o clique esquerdo no título deixou de dobrar a secção"
    );
    set_current_brush(None);
}
