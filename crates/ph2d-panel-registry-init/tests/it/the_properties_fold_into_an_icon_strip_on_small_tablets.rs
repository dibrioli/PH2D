//! ⭐⭐ **O ③ recolhe em faixa de ícones no iPad 11" e no mini** (D10, F5 — 08/10).
//!
//! Com o registo real: abaixo da janela de referência (o iPad 12,9" dos tokens) o painel de
//! propriedades nasce recolhido UMA vez, a faixa mostra um ícone por painel que reabrir devolve, a
//! área de desenho não passa por baixo dela, e tocar num ícone reabre a coluna com aquele painel à
//! frente. Na referência e no desktop nada muda (a alça de 6 px).

use ph2d_editor_core::screens::dock_seam::{DOCK_SEAM_PX, closed_reserve_px};
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroScreen, dock_columns, dock_reopen, paint_hero_screen,
    slot_tabs,
};
use ph2d_editor_core::screens::layout::DockSide;
use ph2d_editor_core::zones::Rect;

/// Os três alvos do `medicoes/06` e a janela do dono.
const IPAD_11: Rect = Rect::new(0.0, 0.0, 1194.0, 834.0);
const IPAD_MINI: Rect = Rect::new(0.0, 0.0, 1133.0, 744.0);
const DONO: Rect = Rect::new(0.0, 0.0, 1930.0, 1040.0);

fn pinta(hero: &mut HeroScreen, vp: Rect, ts: &mut ph2d_text::TextSystem) {
    let mut scene = ph2d_vector::VectorScene::new();
    paint_hero_screen(hero, vp, &mut scene, ts);
}

fn hero_em(vp: Rect, ts: &mut ph2d_text::TextSystem) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    pinta(&mut hero, vp, ts);
    pinta(&mut hero, vp, ts);
    hero
}

/// ⭐⭐ **No iPad 11 e no mini: recolhido, em faixa, sem tapar a área — e o toque escolhe.**
#[test]
fn on_small_tablets_the_properties_start_as_an_icon_strip() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for vp in [IPAD_11, IPAD_MINI] {
        let mut hero = hero_em(vp, &mut ts);
        assert!(
            dock_columns::is_closed(&hero, DockSide::Right),
            "{vp:?}: o ③ não nasceu recolhido"
        );
        let layout = hero.last_layout.expect("layout");
        let strip = layout.dock_reopen(DockSide::Right);
        assert!(
            (strip.w - dock_reopen::strip_px()).abs() < 0.5,
            "{vp:?}: a alça tem {} px, não a faixa",
            strip.w
        );
        let area = layout.draw_area;
        assert!(
            area.x + area.w <= strip.x + 0.5,
            "{vp:?}: a área de desenho passa por baixo da faixa ({area:?} × {strip:?})"
        );
        let icons = dock_reopen::strip_icons(&hero, &layout, DockSide::Right);
        let reopens = dock_columns::reopens(&hero, DockSide::Right);
        assert_eq!(icons.len(), reopens.len(), "{vp:?}: um ícone por painel");
        assert!(!icons.is_empty(), "{vp:?}: a faixa está vazia");
        let (pick, _, r) = *icons.last().expect("um ícone");
        let p = [r.x + r.w * 0.5, r.y + r.h * 0.5];
        assert!(dock_reopen::open_from_strip(
            &mut hero,
            &layout,
            DockSide::Right,
            p
        ));
        assert!(!dock_columns::is_closed(&hero, DockSide::Right));
        let front = ph2d_editor_core::screens::slot::Slot::ALL
            .into_iter()
            .filter_map(|s| slot_tabs::chosen(&hero, s))
            .any(|n| n == pick);
        assert!(front, "{vp:?}: o ícone tocado não veio à frente");
        // UMA vez por sessão: aberto pelo artista, o quadro seguinte não o recolhe outra vez.
        pinta(&mut hero, vp, &mut ts);
        assert!(
            !dock_columns::is_closed(&hero, DockSide::Right),
            "{vp:?}: recolheu de novo"
        );
    }
}

/// **Na referência e no desktop do dono nada muda** — o controlo: a alça continua de 6 px e o ③
/// continua aberto.
#[test]
fn at_the_reference_and_on_the_desktop_nothing_changes() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for vp in [Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H), DONO] {
        let hero = hero_em(vp, &mut ts);
        assert!(
            !dock_columns::is_closed(&hero, DockSide::Right),
            "{vp:?}: recolheu"
        );
        assert!((closed_reserve_px(vp.w) - DOCK_SEAM_PX).abs() < f32::EPSILON);
    }
    assert!(
        closed_reserve_px(IPAD_11.w) > DOCK_SEAM_PX,
        "o controlo do controlo"
    );
}
