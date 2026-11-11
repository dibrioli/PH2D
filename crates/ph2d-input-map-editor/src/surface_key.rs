//! A tecla física da aba de família activa (`DocumentSurface::raw_key`), depois da escuta e dos modais.

use ph2d_editor_core::HeroScreen;

/// ⭐ A tecla física (`ph2d_input::Key` normalizado) para a aba activa com o teclado, antes de todo atalho; `true` = tomada.
pub fn surface_raw_key(
    hero: &mut HeroScreen,
    code: u32,
    pressed: bool,
    text: Option<&str>,
) -> bool {
    if !ph2d_editor_core::screens::hero::surface_input::owns_the_keyboard(Some(hero)) {
        return false;
    }
    let (writes, alt) = (
        text.is_some_and(|t| t.chars().any(|c| !c.is_control())),
        hero.store.alt_held(),
    );
    hero.documents
        .active_surface_mut()
        .is_some_and(|(f, id)| f.raw_key(id, ph2d_input::Key(code), pressed, writes, alt))
}
