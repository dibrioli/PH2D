//! InputMap onda 3 item 7: os dois editores do Sprite não rolam de lado — o deslizar não os toma.
use ph2d_app_sprite_anim::{editor::AnimDocs, region::RegionDocs};
use ph2d_editor_core::doc_surface::DocumentSurface;

#[test]
fn o_deslizar_de_lado_nao_toma_os_editores_do_sprite() {
    let fs: [&mut dyn DocumentSurface; 2] = [&mut AnimDocs::default(), &mut RegionDocs::default()];
    for f in fs {
        assert!(!f.wheel_sideways(1, 500.0, 300.0, -480.0));
    }
}
