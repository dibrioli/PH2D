//! ⭐⭐⭐ **As secções da escultura abrem o menu de TEMA e ARRASTAM pela pega** — o gesto REAL, pelo
//! despachante, com os rects que a pintura registou (ordem do dono, 2026-09-30: *«siga com os
//! outros painéis»*). Irmão do gate do Painter e do Vector; aqui as espécies são DUAS — a `Tool`
//! FIXA (muda de tema e não tem pega) e as seis MÓVEIS —, e o painel tem a SUA dobra (o clique
//! esquerdo no título), que tem de continuar a dobrar.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_sculpt3d::{
    Sculpt3dPanel, Sculpt3dPanelState, Sculpt3dSnapshot, Sculpt3dUi, drain_intents, ids,
    set_current_sculpt3d,
};
use ph2d_ui_testkit::MockPanelHost;

const SEC: u128 = 1_000_000_000;

/// Alto de propósito: o painel mede mais de dois mil píxeis aberto, e o clique é recortado pelo
/// corpo — uma secção abaixo da dobra não registaria nada e passaria calada.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

/// As secções que se arrastam, pela ordem natural.
const MOVEIS: [NodeId; 6] = [
    ids::SCULPT3D_SEC_BRUSH,
    ids::SCULPT3D_SEC_SYMMETRY,
    ids::SCULPT3D_SEC_TOPOLOGY,
    ids::SCULPT3D_SEC_SHADING,
    ids::SCULPT3D_SEC_SCENE,
    ids::SCULPT3D_SEC_BAKE,
];

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

fn publica() {
    set_current_sculpt3d(Some(Sculpt3dSnapshot {
        ui: Sculpt3dUi::default(),
        has_bake_target: true,
        model_span: 2.0,
        ..Sculpt3dSnapshot::default()
    }));
    let _ = drain_intents();
}

fn pintado() -> (MockPanelHost, Sculpt3dPanelState, Vec<(NodeId, Rect)>) {
    publica();
    let mut host = MockPanelHost::with_panel::<Sculpt3dPanel>();
    let mut st = Sculpt3dPanelState;
    let rects = host.paint::<Sculpt3dPanel>(&mut st, VIEWPORT);
    (host, st, rects)
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .rev()
        .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// As móveis à vista, pela ordem pintada.
fn moveis_a_vista(rects: &[(NodeId, Rect)]) -> Vec<(NodeId, Rect)> {
    let mut v: Vec<(NodeId, Rect)> = MOVEIS
        .iter()
        .filter_map(|&id| rect_of(rects, id).map(|r| (id, r)))
        .collect();
    v.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));
    v
}

/// ⚠️ Num ponto do título à ESQUERDA da pega — ela é outro alvo.
fn no_titulo(cab: Rect) -> (f32, f32) {
    (cab.x + cab.w * 0.25, cab.y + cab.h * 0.5)
}

fn botao_direito(host: &mut MockPanelHost, cab: Rect) -> Option<ContextMenuKind> {
    let (x, y) = no_titulo(cab);
    let _ = host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Secondary,
        x,
        y,
        SEC,
    ));
    host.store().context_menu().map(|m| m.kind)
}

/// ⭐⭐ **O botão direito no título de uma secção que se ARRASTA abre o menu de tema dela, e ela tem
/// pega.** *Mutação: o `header` a registar pelo `hit_index.register` de antes ⇒ o menu aberto não é
/// o de tema e a pega não existe no índice.*
#[test]
fn uma_seccao_movel_tem_menu_de_tema_e_pega() {
    let (mut host, _, rects) = pintado();
    let vista = moveis_a_vista(&rects);
    assert_eq!(
        vista.len(),
        MOVEIS.len(),
        "fixtura: as seis secções móveis têm de estar à vista (pintou {})",
        vista.len()
    );
    for (alvo, _) in &vista {
        assert!(
            rect_of(&rects, ph2d_editor_core::ids::grip_de(*alvo)).is_some(),
            "a secção móvel {alvo:?} não registou a pega"
        );
    }
    let (alvo, cab) = vista[1];
    let menu = botao_direito(&mut host, cab);
    assert!(
        matches!(menu, Some(ContextMenuKind::SectionOutline { section }) if section == alvo),
        "o botão direito no título abriu {menu:?} em vez do menu de tema da secção"
    );
}

/// ⭐⭐ **A `Tool` é FIXA: muda de tema e não tem pega.** O controlo é a móvel logo abaixo, que tem
/// as duas coisas. *Mutação: a `Tool` nas `SECCOES_MOVEIS` ⇒ ganha pega.*
#[test]
fn a_ferramenta_e_fixa_tem_tema_e_nao_tem_pega() {
    let (mut host, _, rects) = pintado();
    let cab = rect_of(&rects, ids::SCULPT3D_SEC_TOOL).expect("fixtura: a `Tool` é pintada");
    assert!(
        rect_of(
            &rects,
            ph2d_editor_core::ids::grip_de(ids::SCULPT3D_SEC_TOOL)
        )
        .is_none(),
        "a `Tool` ganhou uma pega — ela é FIXA"
    );
    let primeira_movel = moveis_a_vista(&rects)[0].1;
    assert!(
        cab.y < primeira_movel.y,
        "a `Tool` FIXA tem de ficar acima de toda secção que se arrasta"
    );
    let menu = botao_direito(&mut host, cab);
    assert!(
        matches!(menu, Some(ContextMenuKind::SectionOutline { section })
            if section == ids::SCULPT3D_SEC_TOOL),
        "o botão direito no título da `Tool` abriu {menu:?} em vez do menu de tema"
    );
}

/// ⭐⭐ **Arrastar a pega da segunda móvel para cima da primeira põe-na à frente** — e a pintura
/// seguinte segue a ordem gravada. *Mutação: o `PlanoCtx::corre` pela ordem natural ⇒ a segunda
/// continua atrás.*
#[test]
fn arrastar_a_pega_poe_a_seccao_onde_se_larga() {
    let (mut host, mut st, rects) = pintado();
    let vista = moveis_a_vista(&rects);
    assert!(vista.len() >= 2, "fixtura: duas secções móveis à vista");
    let (primeira, cab1) = vista[0];
    let (segunda, _) = vista[1];
    let pega =
        rect_of(&rects, ph2d_editor_core::ids::grip_de(segunda)).expect("a segunda móvel tem pega");
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
    publica();
    let depois = moveis_a_vista(&host.paint::<Sculpt3dPanel>(&mut st, VIEWPORT));
    let pos = |id: NodeId| depois.iter().position(|(n, _)| *n == id);
    assert!(
        pos(segunda) < pos(primeira),
        "a pintura seguinte não seguiu a ordem gravada ({depois:?})"
    );
}

/// ⭐⭐ **O clique ESQUERDO no título continua a dobrar a secção** — a dobra é deste painel (o
/// `event` despacha-a pelo id do cabeçalho), e o livro do quadro regista o título sob o MESMO id.
/// *Mutação: o título registado só como pega ⇒ o clique não chega ao `toggle_collapsed`.*
#[test]
fn o_clique_esquerdo_no_titulo_ainda_dobra() {
    let (mut host, mut st, rects) = pintado();
    let (alvo, cab) = moveis_a_vista(&rects)[1];
    assert!(!host.store().is_collapsed(alvo), "fixtura: nasce aberta");
    let (x, y) = no_titulo(cab);
    for ev in host.click_at(x, y) {
        let _ = host.apply_panel_event::<Sculpt3dPanel>(&mut st, ev);
    }
    assert!(
        host.store().is_collapsed(alvo),
        "o clique esquerdo no título deixou de dobrar a secção"
    );
}
