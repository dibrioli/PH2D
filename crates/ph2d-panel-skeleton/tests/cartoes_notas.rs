//! ⭐⭐ Report do dono (01/11, CartoesUI): «não consigo criar as notas nas seções como antes». A
//! lista de cartões não se declarava anfitriã de notas, e o botão direito no corpo de um cartão
//! fechava o menu em vez de oferecer «Create Note».

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_skeleton::SkeletonPanel;
use ph2d_panel_skeleton::cards::CARD_BONE;
use ph2d_panel_skeleton::state::{self, SkeletonPanelState};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

#[test]
fn a_right_click_inside_a_card_creates_a_note_in_that_card() {
    state::set_current_bone(Some(ph2d_skeleton::bend::BoneSpec::straight(20.0, 1.0)));
    let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
    let mut st = SkeletonPanelState;
    // Os cartões medem-se ao pintar e assentam com mola: alguns quadros até parar.
    let mut pintados = Vec::new();
    for _ in 0..90 {
        pintados = host.paint::<SkeletonPanel>(&mut st, VIEWPORT);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let campo = pintados
        .iter()
        .find(|(id, _)| *id == ids::VECTOR_BONE_LENGTH)
        .map(|(_, r)| *r)
        .expect("o campo Length do cartão Bone foi pintado");
    // Na linha do rótulo, à esquerda do ponto: corpo do cartão, fora de qualquer widget.
    host.right_click_at(campo.x + 2.0, campo.y - 4.0);
    let menu = host
        .store()
        .context_menu()
        .expect("o botão direito abriu um menu");
    assert_eq!(
        menu.kind,
        ContextMenuKind::CreateNote {
            panel: ids::SKELETON_PANEL,
            section: Some(CARD_BONE),
        },
        "o botão direito no corpo do cartão Bone não ofereceu «Create Note» nele"
    );
    // O despacho real fecha o menu no Down sobre a linha (guarda-o como o último) antes do Click.
    host.store_mut().close_context_menu();
    assert!(ph2d_editor_core::widget::showcase::apply_showcase_event(
        host.store_mut(),
        WidgetEvent::Click(ids::CTX_MENU_CREATE_NOTE)
    ));
    let notas = host.store().notes_for_panel(ids::SKELETON_PANEL);
    assert_eq!(notas.len(), 1);
    assert_eq!(notas[0].section, Some(CARD_BONE));
    // E ela aparece: o cartão Bone cresce com mola (T41) e pinta-a (as caixas ficam clicáveis).
    let caixas = ids::note_ids(ids::SKELETON_PANEL);
    for _ in 0..90 {
        pintados = host.paint::<SkeletonPanel>(&mut st, VIEWPORT);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(
        pintados.iter().any(|(id, _)| *id == caixas.title[0]),
        "a nota nasceu e não foi pintada no cartão"
    );
    state::set_current_bone(None);
}

fn assenta(
    host: &mut MockPanelHost,
    st: &mut SkeletonPanelState,
) -> Vec<(ph2d_a11y::NodeId, Rect)> {
    let mut p = Vec::new();
    for _ in 0..90 {
        p = host.paint::<SkeletonPanel>(st, VIEWPORT);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    p
}

/// ⭐ Report do dono (01/11): o cartão Tool fechou e não reabria. Clicar no título dobra; clicar
/// outra vez desdobra — nos dois sentidos, com o cartão já assentado.
#[test]
fn a_card_title_click_folds_and_unfolds() {
    use ph2d_panel_skeleton::cards::CARD_TOOL;
    let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
    let mut st = SkeletonPanelState;
    for vez in 0..4 {
        let pintados = assenta(&mut host, &mut st);
        let pega = pintados
            .iter()
            .find(|(id, _)| *id == ph2d_editor_core::ids::grip_de(CARD_TOOL))
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("clique {vez}: o título do Tool não foi pintado"));
        let antes = host.store().is_collapsed(CARD_TOOL);
        let evs = host.click_at(pega.x + 10.0, pega.y + pega.h * 0.5);
        for ev in evs {
            host.apply_panel_event::<SkeletonPanel>(&mut st, ev);
        }
        assert_ne!(
            host.store().is_collapsed(CARD_TOOL),
            antes,
            "clique {vez}: o título do Tool não mudou a dobra (estava dobrado: {antes})"
        );
    }
}

/// O mesmo depois de ARRASTAR o Tool para baixo do Bone (o smoke do dono reordena antes).
#[test]
fn a_card_title_click_folds_and_unfolds_after_a_drag() {
    use ph2d_panel_skeleton::cards::CARD_TOOL;
    state::set_current_bone(Some(ph2d_skeleton::bend::BoneSpec::straight(20.0, 1.0)));
    let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
    let mut st = SkeletonPanelState;
    let pega_de = |p: &[(ph2d_a11y::NodeId, Rect)]| {
        p.iter()
            .find(|(id, _)| *id == ph2d_editor_core::ids::grip_de(CARD_TOOL))
            .map(|(_, r)| *r)
            .expect("o título do Tool foi pintado")
    };
    let pega = pega_de(&assenta(&mut host, &mut st));
    let (x, y) = (pega.x + 10.0, pega.y + pega.h * 0.5);
    for ev in host.drag_at(x, y, x, y + 160.0) {
        host.apply_panel_event::<SkeletonPanel>(&mut st, ev);
    }
    assert!(
        host.store().card_drag().is_none(),
        "o arrasto ficou pendurado"
    );
    for vez in 0..4 {
        let pega = pega_de(&assenta(&mut host, &mut st));
        let antes = host.store().is_collapsed(CARD_TOOL);
        for ev in host.click_at(pega.x + 10.0, pega.y + pega.h * 0.5) {
            host.apply_panel_event::<SkeletonPanel>(&mut st, ev);
        }
        assert_ne!(
            host.store().is_collapsed(CARD_TOOL),
            antes,
            "clique {vez} depois do arrasto: a dobra não mudou (estava dobrado: {antes})"
        );
    }
    state::set_current_bone(None);
}

/// ⭐ D24 (01/11): a lista FLUTUA — o canvas passa por trás dela, e a borda que a estreita (report
/// «não consegui estreitar o painel Bones») vive na beirada da LISTA, só na altura dela. Antes a
/// lista publicava a coluna inteira: a borda vivia, mas a coluna ficava reservada (fundo escuro).
#[test]
fn a_short_card_list_floats_and_keeps_its_seam_at_its_own_edge() {
    use ph2d_editor_core::screens::HeroLayout;
    use ph2d_editor_core::screens::layout::{DockSide, DockSides};
    state::set_current_bone(None);
    let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
    let mut st = SkeletonPanelState;
    assenta(&mut host, &mut st);
    let base = HeroLayout::for_viewport(VIEWPORT);
    let (esq, dir) = base.side_columns();
    let store = host.store();
    let publicado = store
        .panel_rect(ids::SKELETON_PANEL)
        .expect("o Bones publicou o rect");
    assert_eq!(
        store.docked_panel_rects().count(),
        0,
        "a lista não pode reservar a coluna"
    );
    let (lados, listas) = DockSides::from_published(esq, dir, store.docked_panel_rects())
        .with_floating(esq, dir, store.floating_panel_rects());
    assert!(
        !lados.left && !lados.right,
        "a coluna ficou reservada: {lados:?}"
    );
    assert!(
        lados.float_left || lados.float_right,
        "a lista {publicado:?} não assentou em coluna"
    );
    let lado = if lados.float_right {
        DockSide::Right
    } else {
        DockSide::Left
    };
    let mut l = HeroLayout::for_viewport_bands(
        VIEWPORT,
        false,
        ph2d_editor_core::screens::layout::ChromeBands::DEFAULT,
        ph2d_editor_core::screens::layout::CenterSplit::None,
        lados,
    );
    l.float_lists = listas;
    let costura = l.dock_seam(lado);
    assert!(costura.w > 0.0, "a borda de estreitar sumiu");
    assert_eq!((costura.y, costura.h), (publicado.y, publicado.h));
    assert!(
        l.draw_area.x + l.draw_area.w > publicado.x + publicado.w - 1.0
            || l.draw_area.x < publicado.x + 1.0,
        "o canvas não passa por trás da lista"
    );
}
