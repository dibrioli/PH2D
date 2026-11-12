//! O pintor do herói publica o preset e o tema do quadro (o teste sobe do `paint` para o `screens`: o
//! `paint` não conhece o herói, `architecture_the_foundation_modules_form_a_dag`).

use crate::paint::set_text_rendering;
use ph2d_text::TextSystem;
use ph2d_tokens::{TextRendering, Theme};
use ph2d_vector::VectorScene;

/// O pintor do herói publica o preset E o tema do quadro (`publish_text`): sem isso o chrome
/// decidiria o escurecimento pelo tema de abertura, e um tema claro alargaria a tinta branca.
#[test]
fn the_hero_painter_publishes_the_preset_and_the_theme() {
    use crate::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroScreen, paint_hero_screen};
    for (theme, mode) in [
        (Theme::Light, TextRendering::CrispPlusDarkened),
        (Theme::Forge, TextRendering::CrispHeavy),
    ] {
        crate::published::set_ui_theme(Theme::Oled);
        set_text_rendering(TextRendering::Default);
        let mut hero = HeroScreen::new(crate::NodeId(1));
        (hero.theme, hero.text_rendering) = (theme, mode);
        let vp = crate::zones::Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
        let mut ts = TextSystem::without_system_fonts();
        paint_hero_screen(&mut hero, vp, &mut VectorScene::new(), &mut ts);
        let got = (
            crate::published::ui_theme(),
            crate::published::text_rendering(),
        );
        set_text_rendering(TextRendering::default());
        assert_eq!(got, (theme, mode));
    }
}
