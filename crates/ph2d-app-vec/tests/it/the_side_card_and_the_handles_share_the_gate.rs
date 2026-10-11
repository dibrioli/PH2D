//! ⭐ **O cartão lateral e as alças nascem do MESMO portão** (F10): o MODO (o plano do overlay das
//! alças contra `bars::shape_handles_shown`) e o ALVO (os dois leem `shape_field_target`: as alças
//! directamente, o cartão pelo foco que `frame_fields` publica — `None` com o latch armado).

use ph2d_app_vec::overlay::vec_overlay_plan;
use ph2d_panel_vector::bars::shape_handles_shown;
use ph2d_tool_vector::DrawMode;
use ph2d_vec_scene::ShapeKind;

#[test]
fn the_card_and_the_handles_open_in_the_same_modes() {
    for &mode in DrawMode::ALL {
        assert_eq!(
            vec_overlay_plan(true, mode).shape_handles,
            shape_handles_shown(mode, Some(ShapeKind::Star)),
            "{mode:?}: as alças e o cartão discordam do modo"
        );
        assert!(
            !shape_handles_shown(mode, None),
            "{mode:?}: sem forma viva (ou com o latch armado) não há cartão"
        );
    }
    assert!(
        DrawMode::ALL
            .iter()
            .any(|&m| shape_handles_shown(m, Some(ShapeKind::Star))),
        "controlo: nenhum modo mostra as alças"
    );
}
