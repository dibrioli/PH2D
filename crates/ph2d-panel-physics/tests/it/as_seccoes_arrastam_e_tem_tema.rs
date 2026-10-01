//! ⭐⭐⭐ **As secções do painel de Física abrem o menu de TEMA e ARRASTAM pela pega** — o gesto
//! REAL, pelo despachante, com os rects que a pintura registou (ordem do dono, 2026-09-30: *«siga
//! com os outros painéis»*). Irmão dos gates do Vector e do Painter; aqui as nove secções são todas
//! móveis (não há fixa nem aninhada — `paint::plano`), e o que este painel acrescenta é o CENSO:
//! a lente do pintor (`state::last_painted_section_headers`) contra o livro do quadro.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_physics::state::{PhysicsPanelState, PhysicsSnapshot};
use ph2d_panel_physics::{PhysicsPanel, rows, set_current_physics, state};
use ph2d_ui_testkit::MockPanelHost;

const SEC: u128 = 1_000_000_000;

/// Alto de propósito: o painel pede ~1 200 px, e o gate quer as nove secções à vista (o clique é
/// recortado pela banda do corpo desde a porta da rolagem).
fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 700.0, 4000.0)
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

/// As secções à vista, pela ordem pintada.
fn seccoes_a_vista(rects: &[(NodeId, Rect)]) -> Vec<(NodeId, Rect)> {
    let mut v: Vec<(NodeId, Rect)> = rows::section_header_ids()
        .filter_map(|id| rect_of(rects, id).map(|r| (id, r)))
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

/// ⭐⭐ **O CENSO: toda secção que o PINTOR desenha entra no livro com PEGA.** A lente do pintor é
/// medida (o único `header()` do painel a anota), nunca uma lista escrita à mão. *Mutação: o
/// `header()` a registar pelo `hit.register` de antes ⇒ nenhuma pega no índice.*
#[test]
fn toda_seccao_pintada_entra_no_livro_com_pega() {
    let (_host, _, rects) = pintado();
    let pintadas = state::last_painted_section_headers();
    assert!(
        pintadas.len() >= rows::SECTIONS.len() + rows::HAND_PAINTED_SECTIONS.len(),
        "fixtura: a lente do pintor devolveu {} cabeçalhos — o `paint` não correu ou deixou de os \
         anotar, e este censo mediria nada",
        pintadas.len()
    );
    for id in pintadas {
        assert!(
            rect_of(&rects, ph2d_editor_core::ids::grip_de(id)).is_some(),
            "a secção {id:?} foi pintada e não registou a pega — não se arrasta"
        );
    }
}

/// ⭐⭐ **O botão direito no título abre o menu de tema DESSA secção.** *Mutação: o `header()` a
/// registar pelo `hit.register` de antes ⇒ o título deixa de estar no livro e o menu não é o de
/// tema.*
#[test]
fn o_botao_direito_no_titulo_abre_o_menu_de_tema() {
    let (mut host, _, rects) = pintado();
    let vista = seccoes_a_vista(&rects);
    assert!(
        vista.len() >= 3,
        "fixtura: o painel tem de pintar três secções ou mais (pintou {})",
        vista.len()
    );
    let (alvo, cab) = vista[1];
    let menu = botao_direito(&mut host, cab);
    assert!(
        matches!(menu, Some(ContextMenuKind::SectionOutline { section }) if section == alvo),
        "o botão direito no título abriu {menu:?} em vez do menu de tema da secção {alvo:?}"
    );
}

/// ⭐⭐ **Arrastar a pega da segunda secção para cima da primeira põe-na à frente** — e a pintura
/// seguinte segue a ordem gravada. *Mutação: o plano a pintar pela ordem natural ⇒ a segunda
/// continua atrás.*
#[test]
fn arrastar_a_pega_poe_a_seccao_onde_se_larga() {
    let (mut host, mut st, rects) = pintado();
    let vista = seccoes_a_vista(&rects);
    assert!(vista.len() >= 2, "fixtura: duas secções à vista");
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
    let depois = seccoes_a_vista(&host.paint::<PhysicsPanel>(&mut st, viewport()));
    let pos = |id: NodeId| depois.iter().position(|(n, _)| *n == id);
    assert!(
        pos(segunda) < pos(primeira),
        "a pintura seguinte não seguiu a ordem gravada ({depois:?})"
    );
}

/// ⭐ **Arrastar não é dobrar nem escrever no mundo:** a pega fica FORA do rect que dobra a secção,
/// e uma ordem nova não publica intenção nenhuma — a ordem das secções é vista, nunca documento.
#[test]
fn arrastar_nao_dobra_nem_publica_mundo() {
    let (mut host, _, rects) = pintado();
    let vista = seccoes_a_vista(&rects);
    let (primeira, cab1) = vista[0];
    let (segunda, cab2) = vista[1];
    let pega = rect_of(&rects, ph2d_editor_core::ids::grip_de(segunda)).expect("pega");
    let (px, py) = (pega.x + pega.w * 0.5, pega.y + pega.h * 0.5);
    for (kind, y, t) in [
        (PointerKind::Down, py, SEC),
        (PointerKind::Move, cab1.y + 1.0, SEC + 1),
        (PointerKind::Up, cab1.y + 1.0, SEC + 2),
    ] {
        let _ = host.dispatch_pointer_event(pointer(kind, PointerButton::Primary, px, y, t));
    }
    assert!(
        !host.store().is_collapsed(segunda) && !host.store().is_collapsed(primeira),
        "arrastar a pega dobrou uma secção ({cab2:?})"
    );
    assert!(
        ph2d_panel_physics::drain_intents().is_empty(),
        "reordenar secções publicou uma mudança de mundo"
    );
}
