//! ⭐ **F22b — o OPACITY da barra de controlo lê a forma.** O campo (`VECTOR_OBJ_OPACITY_NUM`, o
//! mesmo do Appearance) nunca era semeado do documento: mostrava o 100 de fábrica ao lado de um
//! Appearance que dizia 30 (achado pela foto da cena 37).

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::panel::{ErasedPanel, Panel, PanelRegistry};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
use ph2d_editor_core::screens::paint_hero_screen;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::{self, Appearance};
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_text::TextSystem;
use ph2d_vec_scene::BlendMode;
use ph2d_vector::VectorScene;
use std::sync::Once;

fn hero() -> HeroScreen {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<VectorPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(VectorPanel::ID, true);
    hero
}

fn quadro(hero: &mut HeroScreen, opacidade: f32) {
    state::set_current_appearance(Some(Appearance {
        opacity: opacidade,
        blend: BlendMode::Normal,
        layers: Vec::new(),
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
    }));
    paint_hero_screen(
        hero,
        Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H),
        &mut VectorScene::new(),
        &mut TextSystem::without_system_fonts(),
    );
}

#[test]
fn a_opacidade_da_barra_le_a_forma() {
    let mut hero = hero();
    for (a, esperado) in [(0.3, 30.0), (0.75, 75.0)] {
        quadro(&mut hero, a);
        let v = hero
            .store
            .number_value(ids::VECTOR_OBJ_OPACITY_NUM)
            .expect("o campo está registado");
        assert!(
            (v - esperado).abs() < 1e-3,
            "a forma está a {esperado} % e o campo diz {v}"
        );
    }
    state::set_current_appearance(None);
}
