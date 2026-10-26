//! ⭐ **Os campos de quadros POR USO** (plano GEF 13 D4/D5) — popular → publicar → PINTAR → clicar, o caminho
//! da shell: uma imagem parada mostra o corte; um bloco que repete não mostra quadros (o defeito do dono:
//! «Platform High mostra H Frames 3»); uma imagem animada num bloco avisa e o botão pede «Make it Animated».

use crate::seam_render_source::atlas_sprite;
use ph2d_editor_core::InspectorImageUse;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel};
use ph2d_editor_core::screens::hero::InspectorSpriteInfo;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids::{
    INSP_IMAGE_MAKE_ANIMATED, INSP_SPRITE_FRAME, INSP_SPRITE_HFRAMES, INSP_SPRITE_VFRAMES,
};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_sprite};
use ph2d_ui_testkit::MockPanelHost;

fn com(uso: InspectorImageUse, cells: u32, grelha: [u32; 2]) -> InspectorSpriteInfo {
    InspectorSpriteInfo {
        image_use: uso,
        image_cells: cells,
        hframes: grelha[0],
        vframes: grelha[1],
        ..atlas_sprite(true)
    }
}

fn pintado(info: InspectorSpriteInfo) -> (MockPanelHost, InspectorState, Vec<ph2d_a11y::NodeId>) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_sprite(Some(info));
    host.settle_section_folds();
    let rects = host.paint::<InspectorPanel>(&mut state, Rect::new(0.0, 0.0, 320.0, 2000.0));
    (host, state, rects.into_iter().map(|(id, _)| id).collect())
}

#[test]
fn cada_uso_pinta_so_os_seus_campos() {
    let quadros = [INSP_SPRITE_HFRAMES, INSP_SPRITE_VFRAMES, INSP_SPRITE_FRAME];
    for (uso, cells, grelha, querem, botao) in [
        // Parada de uma célula: o corte da imagem, sem a célula a escolher.
        (
            InspectorImageUse::Still,
            1,
            [1, 1],
            [true, true, false],
            false,
        ),
        // Parada de uma folha: o corte e a célula.
        (
            InspectorImageUse::Still,
            4,
            [4, 1],
            [true, true, true],
            false,
        ),
        // O bloco de fábrica (Platform High): nada de quadros.
        (InspectorImageUse::Repeated, 1, [1, 1], [false; 3], false),
        // A bandeira num bloco: nada de quadros, o aviso e o botão.
        (InspectorImageUse::Repeated, 4, [4, 1], [false; 3], true),
        // Animada: a grelha é do cartão Animation.
        (InspectorImageUse::Animated, 4, [4, 1], [false; 3], false),
    ] {
        let (_, _, ids) = pintado(com(uso, cells, grelha));
        for (id, quer) in quadros.iter().zip(querem) {
            assert_eq!(ids.contains(id), quer, "{uso:?} {cells} células: {id:?}");
        }
        assert_eq!(
            ids.contains(&INSP_IMAGE_MAKE_ANIMATED),
            botao,
            "{uso:?} {cells}"
        );
    }
}

fn clica(host: &mut MockPanelHost, state: &mut InspectorState) -> bool {
    matches!(
        InspectorPanel::apply_event(state, host, WidgetEvent::Click(INSP_IMAGE_MAKE_ANIMATED)),
        EventOutcome::Consumed | EventOutcome::Observed,
    )
}

#[test]
fn make_it_animated_pede_a_accao_so_onde_o_aviso_existe() {
    let (mut host, mut state, _) = pintado(com(InspectorImageUse::Repeated, 4, [4, 1]));
    assert!(clica(&mut host, &mut state));
    assert_eq!(
        host.drained_actions(),
        vec![EditorAction::MakeImageAnimated {
            entity_bits: atlas_sprite(true).entity_bits
        }]
    );
    // CONTROLO: sem o aviso (parada), o mesmo id não age.
    let (mut host, mut state, _) = pintado(com(InspectorImageUse::Still, 4, [4, 1]));
    clica(&mut host, &mut state);
    assert!(host.drained_actions().is_empty());
}
