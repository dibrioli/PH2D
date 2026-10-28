//! ⭐ A cena 30 abre o quadro com o Kanban da cena 1 inteiro (nível alto) ao lado de três notas, e a
//! aba do MESMO Kanban aberta atrás; e as fotos (`#[ignore]`) nos três zooms, claro e escuro.

use super::*;
use ph2d_board_render::KanbanLod;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_text::TextSystem;

/// A shell: a cena (antes da família, como no `init.rs`), a família, e um quadro de imagem.
fn staged() -> (HeroScreen, W4Kanban, TextSystem) {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut h = HeroScreen::new(NodeId(1));
    let s = scene_w4_kanban(&mut h);
    ph2d_app_kanban::install(&mut h);
    let mut ts = TextSystem::without_system_fonts();
    for _ in 0..3 {
        frame(&mut h, &mut ts);
    }
    (h, s, ts)
}

/// Um quadro de imagem a 60 Hz (o relógio anda: as molas assentam).
fn frame(h: &mut HeroScreen, ts: &mut TextSystem) {
    let now = h.store.clock_ns().max(1_000_000_000_000) + 16_666_667;
    h.store.note_clock(now);
    ph2d_app_kanban::frame(h);
    ph2d_board_render::install_kanbans(h.documents.kanbans_shared());
    document_tabs::follow_surfaces(h);
    let vp = ph2d_editor_core::zones::Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    paint_hero_screen(h, vp, &mut ph2d_vector::VectorScene::new(), ts);
}

#[test]
fn the_w4_kanban_scene_puts_the_scene_1_kanban_beside_notes() {
    let (h, s, _) = staged();
    assert_eq!(h.documents.active(), Some(s.board), "o quadro à frente");
    let b = h.documents.active_board().unwrap();
    let k = b.doc.get(s.element).and_then(|e| e.kanban()).expect("o Kanban");
    assert_eq!(k.kanban, s.kanban);
    assert_eq!(k.view.collapsed.len(), 1, "a Review recolhida NESTA vista");
    let notes = b.doc.live().filter(|e| e.shape().is_some_and(|s| s.kind.is_note()));
    assert_eq!(notes.count(), 3, "três notas ao lado");
    assert_eq!(
        h.documents.kanbans().get(s.kanban).map(|k| k.header.name.clone()).as_deref(),
        Some("Short film \u{2014} production"),
    );
    let pl = kb::placement(&h, s.element).expect("pintado");
    assert_eq!(pl.lod, KanbanLod::High, "a zoom 1 o Kanban inteiro");
    let docs = h.documents.surfaces().find::<ph2d_app_kanban::KanbanDocs>().unwrap();
    assert_eq!(docs.open_kanbans(), vec![s.kanban], "a aba do mesmo Kanban, aberta atrás");
}

/// 📷 As fotos da cena 30: claro e escuro, nos três zooms (inteiro, médio, de longe), e a aba.
#[test]
#[ignore = "fotos: precisa da placa"]
fn photos_of_the_w4_kanban_scene() {
    let mut shots = crate::smoke::shots::Shots::new("ph2d_w4_kanban");
    for (theme, name) in [
        (ph2d_tokens::Theme::Light, "claro"),
        (ph2d_tokens::Theme::Dark, "escuro"),
    ] {
        let (mut h, s, mut ts) = staged();
        h.theme = theme;
        // O nível alto centrado no Kanban (a janela da foto é mais estreita que a do dono).
        for (zoom, z) in [(1.0, "alto"), (0.55, "medio"), (0.22, "baixo")] {
            if let Some(b) = h.documents.active_board_mut() {
                b.camera.zoom = zoom;
                if zoom == 1.0 {
                    b.camera.center_x = KANBAN[0] + KANBAN[2] / 2.0;
                }
            }
            for _ in 0..30 {
                frame(&mut h, &mut ts);
            }
            shots.shot(&mut h, &format!("cena30_{z}_{name}"));
        }
        let docs = h.documents.surfaces_mut();
        if let Some(d) = docs.find_mut::<ph2d_app_kanban::KanbanDocs>() {
            d.open(s.kanban);
        }
        for _ in 0..30 {
            frame(&mut h, &mut ts);
        }
        shots.shot(&mut h, &format!("cena30_aba_{name}"));
    }
}
