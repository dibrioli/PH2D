//! **A janela GRID & BACKGROUND responde a um PONTEIRO real** (P12 P): a tela inteira pintada, com
//! todos os painéis registados; a caixa `Visible` da grade muda a grade do documento (o que a
//! família recebe pelo `troca`), e `Set as Defaults` grava os valores para os documentos novos.

use bumpalo::Bump;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, PANEL_PIXEL_PREFS, paint_hero_screen,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_prefs::canal::{self, Caixa};
use ph2d_panel_pixel_prefs::ids;
use ph2d_pixel_core::PixelPrefs;
use ph2d_pixel_render::prefs_canal as fio;

fn ptr(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

fn assenta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem, n: usize) {
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    for _ in 0..n {
        let mut scene = ph2d_vector::VectorScene::new();
        ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || paint_hero_screen(hero, vp, &mut scene, ts),
        );
    }
}

fn clica(hero: &mut HeroScreen, id: NodeId, nome: &str) {
    let r = hero
        .hit_index
        .rect_for(id)
        .unwrap_or_else(|| panic!("{nome} não foi pintado"));
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let arena = Bump::new();
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    assert!(
        ev.contains(&WidgetEvent::Click(id)),
        "{nome}: o ponteiro deu {ev:?}; em ({x},{y}) acerta {:?}",
        hero.hit_index.hit_with_rect(x, y),
    );
    for e in ev {
        hero.apply_event(e);
    }
}

#[test]
fn visible_muda_a_grade_do_documento_e_set_as_defaults_grava_os_novos() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    // O ficheiro dos novos vai para uma pasta do teste: nunca o `~/.ph2d` do dono.
    let pasta = std::env::temp_dir().join(format!("ph2d-pixel-prefs-seam-{}", std::process::id()));
    let ficheiro = pasta.join("pixel-prefs.txt");
    let doc = PixelPrefs::default();
    assert_eq!(fio::troca_em(Some(doc), Some(&ficheiro)), None);
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(PANEL_PIXEL_PREFS, true);
    assenta(&mut hero, &mut ts, 60);
    clica(
        &mut hero,
        ids::caixa(Caixa::GridVisible),
        "Visible da grade",
    );
    let p = fio::troca_em(Some(doc), Some(&ficheiro)).expect("a grade mudou");
    assert!(p.grid.visible);
    assert_eq!(
        PixelPrefs {
            grid: doc.grid,
            ..p
        },
        doc,
        "só a grade mudou"
    );
    assenta(&mut hero, &mut ts, 2);
    clica(&mut hero, ids::SET_DEFAULTS, "Set as Defaults");
    assert_eq!(fio::troca_em(Some(p), Some(&ficheiro)), None);
    assert_eq!(
        ph2d_pixel_render::prefs_view::load_defaults_from(&ficheiro),
        p
    );
    let _ = std::fs::remove_dir_all(pasta);
}

#[test]
fn o_cartao_onion_skin_abre_pelo_pedido_do_onion_e_muda_o_documento() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let pasta = std::env::temp_dir().join(format!("ph2d-pixel-prefs-onion-{}", std::process::id()));
    let ficheiro = pasta.join("pixel-prefs.txt");
    let doc = PixelPrefs::default();
    let _ = fio::troca_em(Some(doc), Some(&ficheiro));
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = HeroScreen::new(NodeId(1));
    // O botão direito no botão do onion da linha do tempo pede isto (`ONION_SETTINGS`).
    fio::pede_abrir_onion();
    assenta(&mut hero, &mut ts, 60);
    assert!(hero.is_panel_visible(PANEL_PIXEL_PREFS), "a janela abriu");
    clica(&mut hero, ids::caixa(Caixa::OnionFront), "In Front");
    let p = fio::troca_em(Some(doc), Some(&ficheiro)).expect("mudou");
    assenta(&mut hero, &mut ts, 2);
    clica(
        &mut hero,
        ids::caixa(Caixa::OnionCurrentLayer),
        "Current layer only",
    );
    let q = fio::troca_em(Some(p), Some(&ficheiro)).expect("mudou");
    assert!(q.onion.front && q.onion.current_layer_only);
    assert_eq!(
        PixelPrefs {
            onion: doc.onion,
            ..q
        },
        doc,
        "só o onion mudou"
    );
    let _ = std::fs::remove_dir_all(pasta);
}

/// A amostra `Color` da grade: o ponteiro abre o seletor nela; a cor escolhida chega ao documento;
/// fechar o seletor não muda mais nada (uma mudança só).
#[test]
fn the_grid_colour_swatch_opens_the_picker_and_the_pick_changes_the_documents_grid_once() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let doc = PixelPrefs::default();
    assert_eq!(fio::troca_em(Some(doc), None), None);
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(PANEL_PIXEL_PREFS, true);
    assenta(&mut hero, &mut ts, 60);
    let amostra = ids::cor(canal::Cor::Grid);
    let r = hero
        .hit_index
        .rect_for(amostra)
        .expect("a amostra da grade");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let arena = Bump::new();
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    for e in ev {
        hero.apply_event(e);
    }
    assenta(&mut hero, &mut ts, 2);
    assert_eq!(hero.store.picker_target(), Some(amostra), "o seletor abriu");
    assert_eq!(fio::troca_em(Some(doc), None), None, "abrir não muda nada");
    let quer = [10, 200, 30, 255];
    hero.store.set_blender_value(
        ph2d_editor_core::ids::INSP_BLENDER_PICKER,
        ph2d_tokens::ColorValue::from_rgba8(quer[0], quer[1], quer[2], quer[3]),
    );
    hero.store
        .note_colour_pick(ph2d_editor_core::ids::INSP_BLENDER_PICKER);
    assenta(&mut hero, &mut ts, 3);
    let p = fio::troca_em(Some(doc), None).expect("a cor mudou");
    assert_eq!(p.grid.color, quer);
    assert_eq!(
        PixelPrefs {
            grid: doc.grid,
            ..p
        },
        doc,
        "só a grade mudou"
    );
    assenta(&mut hero, &mut ts, 2);
    hero.store.set_picker_target(None);
    assenta(&mut hero, &mut ts, 2);
    assert_eq!(fio::troca_em(Some(p), None), None, "fechar não muda mais");
}
