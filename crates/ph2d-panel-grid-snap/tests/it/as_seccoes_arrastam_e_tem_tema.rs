//! ⭐⭐⭐ **As secções do painel da grelha abrem o menu de TEMA e ARRASTAM pela pega** — o gesto
//! REAL, pelo despachante, com os rects que a pintura registou (ordem do dono, 2026-09-30: *«siga
//! com os outros painéis»*). Irmão dos gates do Vector, do Painter e da Física; aqui as quatro
//! secções são móveis e o botão *Snap* do topo é um BLOCO (sem título, logo nem menu nem pega —
//! `crate::plano`).
//!
//! ⚠️ **O painel é FLUTUANTE e nasce com `640 px`**, mais baixo do que o corpo: o gate dá-lhe uma
//! janela alta pela porta do host (`set_grid_snap_panel_rect`), senão as secções de baixo ficam
//! fora do recorte da rolagem e não se clicam.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_grid_snap::GridSnapPanel;
use ph2d_panel_grid_snap::state::GridSnapPanelState;
use ph2d_ui_testkit::MockPanelHost;

const SEC: u128 = 1_000_000_000;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 4000.0)
}

/// As quatro secções, pela ordem natural.
const SECCOES: [NodeId; 4] = [
    ph2d_panel_grid_snap::ids::GS_SEC_KIND,
    ph2d_panel_grid_snap::ids::GS_SEC_TARGET,
    ph2d_panel_grid_snap::ids::GS_SEC_DISPLAY,
    ph2d_editor_core::grid_snap::ids::GS_INSPECT_HEADER,
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

fn pintado() -> (MockPanelHost, GridSnapPanelState, Vec<(NodeId, Rect)>) {
    let mut host = MockPanelHost::with_panel::<GridSnapPanel>();
    // Uma janela flutuante ALTA — o corpo inteiro cabe e nada fica rolado para fora.
    host.set_grid_snap_panel_rect(Some(Rect::new(8.0, 8.0, 320.0, 3000.0)));
    let mut st = GridSnapPanelState;
    let rects = host.paint::<GridSnapPanel>(&mut st, viewport());
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
fn seccoes_a_vista(rects: &[(NodeId, Rect)]) -> Vec<(NodeId, Rect)> {
    let mut v: Vec<(NodeId, Rect)> = SECCOES
        .iter()
        .filter_map(|&id| rect_of(rects, id).map(|r| (id, r)))
        .collect();
    v.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));
    v
}

fn botao_direito(host: &mut MockPanelHost, cab: Rect) -> Option<ContextMenuKind> {
    // ⚠️ Num ponto do título à ESQUERDA da pega — a pega é outro alvo.
    let (x, y) = (cab.x + cab.w * 0.25, cab.y + cab.h * 0.5);
    let _ = host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Secondary,
        x,
        y,
        SEC,
    ));
    host.store().context_menu().map(|m| m.kind)
}

/// ⭐⭐ **As quatro secções estão à vista e cada uma tem PEGA.** *Mutação: o `dobravel` a registar
/// pelo `hit.register` de antes ⇒ nenhuma pega no índice.*
#[test]
fn as_quatro_seccoes_tem_pega() {
    let (_host, _, rects) = pintado();
    let vista = seccoes_a_vista(&rects);
    assert_eq!(
        vista.len(),
        SECCOES.len(),
        "fixtura: com a janela alta as quatro secções têm de estar à vista ({vista:?})"
    );
    for (id, _) in vista {
        assert!(
            rect_of(&rects, ph2d_editor_core::ids::grip_de(id)).is_some(),
            "a secção {id:?} foi pintada e não registou a pega — não se arrasta"
        );
    }
}

/// ⭐⭐ **O botão direito no título abre o menu de tema DESSA secção.** *Mutação: o `dobravel` a
/// registar pelo `hit.register` de antes ⇒ o título deixa de estar no livro.*
#[test]
fn o_botao_direito_no_titulo_abre_o_menu_de_tema() {
    let (mut host, _, rects) = pintado();
    let vista = seccoes_a_vista(&rects);
    let (alvo, cab) = vista[1];
    let menu = botao_direito(&mut host, cab);
    assert!(
        matches!(menu, Some(ContextMenuKind::SectionOutline { section }) if section == alvo),
        "o botão direito no título abriu {menu:?} em vez do menu de tema da secção {alvo:?}"
    );
}

/// ⭐ **O botão *Snap* do topo é um BLOCO, não uma secção** — não tem pega nem abre o menu de tema
/// de secção (não há título onde o artista o escolhesse). O controlo é a secção ao lado, que tem
/// as duas coisas.
#[test]
fn o_botao_snap_e_um_bloco_sem_pega_nem_menu() {
    let (mut host, _, rects) = pintado();
    let snap = rect_of(&rects, ph2d_editor_core::grid_snap::ids::GS_SNAP_ENABLED)
        .expect("fixtura: o botão Snap é pintado");
    assert!(
        rect_of(
            &rects,
            ph2d_editor_core::ids::grip_de(ph2d_editor_core::grid_snap::ids::GS_SNAP_ENABLED)
        )
        .is_none(),
        "o botão Snap ganhou uma pega — ele é um bloco"
    );
    let menu = botao_direito(&mut host, snap);
    assert!(
        !matches!(menu, Some(ContextMenuKind::SectionOutline { .. })),
        "o botão direito no Snap abriu o menu de tema de secção ({menu:?})"
    );
    let primeira = seccoes_a_vista(&rects)[0].1;
    assert!(
        snap.y < primeira.y,
        "o bloco do Snap deixou de ficar no topo, acima das secções"
    );
}

/// ⭐⭐ **Arrastar a pega da segunda secção para cima da primeira põe-na à frente** — e a pintura
/// seguinte segue a ordem gravada. *Mutação: o plano a declarar as secções como FIXAS ⇒ a ordem
/// do artista é ignorada e a segunda continua atrás.*
#[test]
fn arrastar_a_pega_poe_a_seccao_onde_se_larga() {
    let (mut host, mut st, rects) = pintado();
    let vista = seccoes_a_vista(&rects);
    let (primeira, cab1) = vista[0];
    let (segunda, _) = vista[1];
    let pega = rect_of(&rects, ph2d_editor_core::ids::grip_de(segunda))
        .expect("a segunda secção tem pega");
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
    let depois = seccoes_a_vista(&host.paint::<GridSnapPanel>(&mut st, viewport()));
    let pos = |id: NodeId| depois.iter().position(|(n, _)| *n == id);
    assert!(
        pos(segunda) < pos(primeira),
        "a pintura seguinte não seguiu a ordem gravada ({depois:?})"
    );
}
