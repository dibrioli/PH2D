//! ⭐⭐⭐ **As NOTAS e o CONTORNO vivem no painel de Física** — o report do dono (2026-10-01): *«a
//! possibilidade de criar notas deve existir em quaisquer painéis de qualquer tipo. Outline também
//! deve funcionar em qualquer painel. Em physics não está funcionando.»*
//!
//! Até aqui o painel mudava o tema e arrastava as secções, e o contorno escolhido no menu do título
//! **não se pintava** — um item de menu mudo —, e o botão direito no corpo não oferecia *Create
//! Note*. As três metades pelo gesto REAL, com os rects que a pintura registou: o botão direito
//! cria a nota NA secção sob o cursor, a nota pinta-se no fim DELA, e o contorno escolhido pinta.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids::{self as core_ids, CTX_MENU_CREATE_NOTE, CTX_MENU_OUTLINE_0};
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_physics::state::{PhysicsPanelState, PhysicsSnapshot};
use ph2d_panel_physics::{PhysicsPanel, rows, set_current_physics};
use ph2d_ui_testkit::MockPanelHost;

const P: NodeId = core_ids::PHYSICS_PANEL;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 700.0, 4000.0)
}

fn pintado() -> (MockPanelHost, PhysicsPanelState, Vec<(NodeId, Rect)>) {
    set_current_physics(Some(PhysicsSnapshot::default()));
    let _ = ph2d_panel_physics::drain_intents();
    let mut host = MockPanelHost::with_panel::<PhysicsPanel>();
    let mut st = PhysicsPanelState;
    let rects = host.paint::<PhysicsPanel>(&mut st, viewport());
    (host, st, rects)
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .rev()
        .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

fn seccoes_a_vista(rects: &[(NodeId, Rect)]) -> Vec<(NodeId, Rect)> {
    let mut v: Vec<(NodeId, Rect)> = rows::section_header_ids()
        .filter_map(|id| rect_of(rects, id).map(|r| (id, r)))
        .collect();
    v.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));
    v
}

fn botao_direito(host: &mut MockPanelHost, x: f32, y: f32) -> Option<ContextMenuKind> {
    let _ = host.dispatch_pointer_event(PointerEvent {
        kind: PointerKind::Down,
        x,
        y,
        button: PointerButton::Secondary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: 1_000_000_000,
    });
    host.store().context_menu().map(|m| m.kind)
}

/// ⭐⭐ **O botão direito no corpo de uma secção oferece *Create Note*, e a nota nasce NESSA
/// secção.** *Mutação: a porta das notas do fim do corpo sem o `mark_note_host` ⇒ o menu não abre
/// (era o report: «em physics não está funcionando»).*
#[test]
fn o_botao_direito_no_corpo_cria_uma_nota_na_seccao_sob_o_cursor() {
    let (mut host, _, rects) = pintado();
    let vista = seccoes_a_vista(&rects);
    assert!(vista.len() >= 2, "fixtura: duas secções à vista");
    let (alvo, cab) = vista[1];
    let (_, seguinte) = vista[2.min(vista.len() - 1)];
    // Entre o fim do título da 2.ª secção e o da seguinte — no corpo dela.
    let y = (cab.y + cab.h + seguinte.y) * 0.5;
    let menu = botao_direito(&mut host, viewport().w - 6.0, y);
    assert!(
        matches!(menu, Some(ContextMenuKind::CreateNote { panel, section }) if panel == P && section == Some(alvo)),
        "o botão direito no corpo da Física abriu {menu:?} em vez de criar nota na secção {alvo:?}"
    );
    assert!(host.choose_context_menu_row(CTX_MENU_CREATE_NOTE));
    let notas = host.store().notes_for_panel(P);
    assert_eq!(notas.len(), 1, "a nota não nasceu");
    assert_eq!(notas[0].section, Some(alvo), "a nota nasceu fora da secção");
}

/// ⭐⭐ **A nota pinta-se no FIM da secção dela** — entre o título dela e o da seguinte, com as
/// caixas registadas (título, corpo, pega, minimizar). *Mutação: o laço do plano sem o `cromo` ⇒
/// a nota cai no fim do corpo, abaixo de todas as secções.*
#[test]
fn a_nota_pinta_se_no_fim_da_sua_seccao() {
    let (mut host, mut st, rects) = pintado();
    let vista = seccoes_a_vista(&rects);
    let (alvo, cab) = vista[0];
    let (_, seguinte) = vista[1];
    assert_eq!(host.add_note(P, Some(alvo)), Some(0));
    let depois = host.paint::<PhysicsPanel>(&mut st, viewport());
    let caixas = core_ids::note_ids(P);
    let nota = rect_of(&depois, caixas.slot[0]).expect("a nota foi pintada");
    let seguinte_depois = seccoes_a_vista(&depois)
        .into_iter()
        .find(|(id, _)| *id == vista[1].0)
        .map(|(_, r)| r)
        .expect("a 2.ª secção continua à vista");
    assert!(
        nota.y > cab.y && nota.y + nota.h <= seguinte_depois.y,
        "a nota ({nota:?}) não ficou entre o título da secção ({cab:?}) e o da seguinte \
         ({seguinte_depois:?}; antes {seguinte:?})"
    );
    for id in [
        caixas.title[0],
        caixas.body[0],
        caixas.grip[0],
        caixas.fold[0],
    ] {
        assert!(rect_of(&depois, id).is_some(), "a nota não registou {id:?}");
    }
}

/// ⭐⭐ **O contorno escolhido no menu do título PINTA** — o item de menu mudo do report. A régua é
/// a geometria da pintura: o mesmo painel com o contorno escolhido tem MAIS segmentos de caminho.
/// *Mutação: o `cromo` do plano sem o contorno ⇒ a contagem não sobe.*
#[test]
fn o_contorno_escolhido_no_titulo_pinta_na_fisica() {
    let (mut host, mut st, rects) = pintado();
    let (alvo, cab) = seccoes_a_vista(&rects)[0];
    let (_, antes) = host.paint_and_count_geometry::<PhysicsPanel>(&mut st, viewport());
    let menu = botao_direito(&mut host, cab.x + cab.w * 0.25, cab.y + cab.h * 0.5);
    assert!(
        matches!(menu, Some(ContextMenuKind::SectionOutline { section }) if section == alvo),
        "fixtura: o botão direito no título abre o menu de tema ({menu:?})"
    );
    assert!(host.choose_context_menu_row(CTX_MENU_OUTLINE_0));
    assert_eq!(host.store().section_outline_color(alvo), Some(0));
    let (_, depois) = host.paint_and_count_geometry::<PhysicsPanel>(&mut st, viewport());
    assert!(
        depois > antes,
        "o contorno escolhido não pintou nada ({antes} → {depois} segmentos)"
    );
}
