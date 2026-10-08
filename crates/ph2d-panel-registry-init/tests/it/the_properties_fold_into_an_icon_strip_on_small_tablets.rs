//! ⭐⭐ **O ③ recolhido é uma faixa de ícones no iPad 11" e no mini** (D10, F5 — 08/10).
//!
//! Com o registo real: abaixo da janela de referência (o iPad 12,9" dos tokens), recolher o painel de
//! propriedades deixa uma FAIXA FLUTUANTE de ícones dentro da área, do lado da coluna — um por
//! painel que reabrir devolve —, a alça continua de 6 px (o piso do ecrã com as colunas fechadas não
//! muda), e um clique REAL num ícone reabre a coluna com aquele painel à frente. Na referência e no
//! desktop: nenhuma faixa.

use bumpalo::Bump;
use ph2d_editor_core::screens::dock_seam::DOCK_SEAM_PX;
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroScreen, dock_columns, dock_reopen, paint_hero_screen,
    slot_tabs,
};
use ph2d_editor_core::screens::layout::DockSide;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};

/// Os dois alvos estreitos do `medicoes/06` e a janela do dono.
const IPAD_11: Rect = Rect::new(0.0, 0.0, 1194.0, 834.0);
const IPAD_MINI: Rect = Rect::new(0.0, 0.0, 1133.0, 744.0);
const DONO: Rect = Rect::new(0.0, 0.0, 1930.0, 1040.0);

fn pinta(hero: &mut HeroScreen, vp: Rect, ts: &mut ph2d_text::TextSystem) {
    let mut scene = ph2d_vector::VectorScene::new();
    paint_hero_screen(hero, vp, &mut scene, ts);
}

/// Um herói com o ③ recolhido, pintado duas vezes.
fn recolhido(vp: Rect, ts: &mut ph2d_text::TextSystem) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    pinta(&mut hero, vp, ts);
    assert!(
        !dock_columns::is_closed(&hero, DockSide::Right),
        "{vp:?}: o ③ recolheu sozinho — a coluna aberta no tablet é lei medida (09/20)"
    );
    assert!(dock_columns::close(&mut hero, DockSide::Right, None));
    pinta(&mut hero, vp, ts);
    pinta(&mut hero, vp, ts);
    hero
}

fn clica(hero: &mut HeroScreen, r: Rect) {
    let arena = Bump::new();
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    for kind in [PointerKind::Down, PointerKind::Up] {
        let ev = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        };
        let evs = ph2d_editor_core::interaction::dispatch_pointer(
            &mut hero.store,
            &hero.hit_index,
            ev,
            &arena,
        )
        .to_vec();
        for e in evs {
            hero.apply_event(e);
        }
    }
}

/// ⭐⭐ **No iPad 11 e no mini: a faixa flutua do lado do ③, sem custar área, e o clique escolhe.**
#[test]
fn on_small_tablets_the_folded_properties_become_an_icon_strip() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for vp in [IPAD_11, IPAD_MINI] {
        let mut hero = recolhido(vp, &mut ts);
        let layout = hero.last_layout.expect("layout");
        let handle = layout.dock_reopen(DockSide::Right);
        assert!(
            (handle.w - DOCK_SEAM_PX).abs() < 0.5,
            "{vp:?}: a alça passou a {} px — a faixa custaria área",
            handle.w
        );
        let icons = dock_reopen::strip_icons(&hero, &layout);
        assert_eq!(
            icons.len(),
            dock_columns::reopens(&hero, DockSide::Right).len(),
            "{vp:?}: um ícone por painel que reabrir devolve"
        );
        assert!(!icons.is_empty(), "{vp:?}: a faixa está vazia");
        let area = layout.draw_area;
        for (_, _, _, r) in &icons {
            assert!(
                r.x >= area.x + area.w * 0.5 && r.x + r.w <= area.x + area.w,
                "{vp:?}: o ícone {r:?} não flutua dentro da área, do lado do ③"
            );
        }
        let (tab, _, _, r) = *icons.last().expect("um ícone");
        let pick = slot_tabs::panel_for_tab(tab).expect("o ícone é uma aba");
        clica(&mut hero, r);
        assert!(
            !dock_columns::is_closed(&hero, DockSide::Right),
            "{vp:?}: o clique no ícone não reabriu o ③"
        );
        let front = ph2d_editor_core::screens::slot::Slot::ALL
            .into_iter()
            .filter_map(|s| slot_tabs::chosen(&hero, s))
            .any(|n| n == pick);
        assert!(front, "{vp:?}: o ícone clicado não veio à frente");
    }
}

/// **Na referência e no desktop do dono: nenhuma faixa** — o controlo (o ③ recolhido fica só a alça).
#[test]
fn at_the_reference_and_on_the_desktop_there_is_no_strip() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for vp in [Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H), DONO] {
        let hero = recolhido(vp, &mut ts);
        let layout = hero.last_layout.expect("layout");
        assert!(
            dock_reopen::strip_icons(&hero, &layout).is_empty(),
            "{vp:?}: faixa fora dos tablets estreitos"
        );
    }
}
