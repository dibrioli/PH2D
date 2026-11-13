//! B20.11: o overlay da cena emite as formigas em traço de 1 px de ECRÃ (a régua das bordas e do passo
//! vive em `ph2d-tool-painter` `selection_formigas_ecra_tests`); aqui só a costura do afim da sprite.

use super::draw_selection_overlay_no_passo;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite};
use ph2d_tool_painter::PainterTool;
use ph2d_vector::VectorScene;

#[test]
fn as_formigas_saem_na_cena_com_1_px_de_ecra_a_zoom_baixo() {
    const N: u32 = 400;
    let janela = WindowSize {
        width: 480,
        height: 480,
    };
    let mut sim = SimWorld::default();
    let sprite = Sprite::atlas(0, [4.0, 4.0], [1.0; 4]);
    let e = sim.world_mut().spawn((Transform::IDENTITY, sprite)).id();
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.selection = Some(e.to_bits());
    let camera = Camera2d::new([0.0, 0.0], 4.0 * 480.0 / (0.37 * N as f32));
    let mut p = PainterTool::default();
    p.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    p.set_rect_selection(60, 50, 250, 210);
    let mut vs = VectorScene::new();
    draw_selection_overlay_no_passo(&p, &hero, &sim, &camera, janela, &mut vs, 3);
    let (larguras, pontos) = vs.probe_strokes();
    assert!(
        !larguras.is_empty() && larguras.iter().all(|&l| l == 1.0),
        "1 px de ecrã: {larguras:?}"
    );
    // Os 4 cantos da caixa pelo afim da sprite, no centro do pixel de ecrã.
    let tr = ph2d_ecs::world_transform(sim.world(), e).expect("pose");
    let afim =
        ph2d_sprite_screen::sprite_image_to_screen_affine(N, N, tr, sprite, None, &camera, janela);
    for (x, y) in [(60.0, 50.0), (310.0, 50.0), (310.0, 260.0), (60.0, 260.0)] {
        let s = afim * ph2d_vector::Point::new(x, y);
        let (sx, sy) = ((s.x.floor() + 0.5) as f32, (s.y.floor() + 0.5) as f32);
        assert!(
            pontos
                .iter()
                .any(|q| (q[0] - sx).abs() < 0.01 && (q[1] - sy).abs() < 0.01),
            "canto ({x}, {y}) → ({sx}, {sy}) ausente"
        );
    }
}
