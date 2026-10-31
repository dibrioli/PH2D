//! 📷 As fotos da cena 31 que pedem cliques (o fotógrafo da janela real não clica): o menu de um
//! Kanban nos Assets, a pergunta de apagar e um Kanban largado no Quadro. O ecrã é o hero com o
//! registo de painéis da app (os Assets pintam-se) e o índice dos documentos publicado como a shell.

use super::*;
use ph2d_asset_index::{AssetEntry, AssetIndex, AssetRef};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids::{CTX_MENU_ASSET_DELETE, asset_cell_id};
use ph2d_editor_core::interaction::drag_payload::DragPayload;
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_text::TextSystem;

const DIR: &str = "ph2d_d17_fotos";

/// A cena 31 com os painéis da app e os documentos nos Assets (o `push_documents` da shell).
fn staged() -> (HeroScreen, TextSystem) {
    ph2d_panel_registry_init::register_all_panels();
    let mut h = HeroScreen::new(NodeId(1));
    scene_d17_assets(&mut h);
    let mut index = AssetIndex::new();
    for (doc, name) in h.documents.asset_listing() {
        let key = match doc {
            DragPayload::Kanban { id } => AssetRef::Kanban { id },
            DragPayload::Board { id } => AssetRef::Board { id },
            _ => continue,
        };
        let mut e = AssetEntry::new(key, name);
        e.detail = tr(key.kind().label_key()).to_string();
        index.push(e);
    }
    ph2d_panel_asset_browser::set_current_index(index);
    let mut ts = TextSystem::without_system_fonts();
    for _ in 0..30 {
        frame(&mut h, &mut ts);
    }
    (h, ts)
}

fn frame(h: &mut HeroScreen, ts: &mut TextSystem) {
    let now = h.store.clock_ns().max(1_000_000_000_000) + 16_666_667;
    h.store.note_clock(now);
    ph2d_board_render::install_kanbans(h.documents.kanbans_shared());
    ph2d_editor_core::screens::hero::document_tabs::follow_surfaces(h);
    let vp = ph2d_editor_core::zones::Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    paint_hero_screen(h, vp, &mut ph2d_vector::VectorScene::new(), ts);
}

/// O meio do alvo `id` que o pintor registou.
fn centre_of(h: &HeroScreen, id: NodeId) -> (f32, f32) {
    let (mut sx, mut sy, mut n) = (0.0, 0.0, 0.0);
    let mut y = 0.0;
    while y < HERO_VIEWPORT_H {
        let mut x = 0.0;
        while x < HERO_VIEWPORT_W {
            if h.hit_index.hit(x, y) == Some(id) {
                (sx, sy, n) = (sx + x, sy + y, n + 1.0);
            }
            x += 2.0;
        }
        y += 2.0;
    }
    assert!(n > 0.0, "{id:?} não está pintado");
    (sx / n, sy / n)
}

/// Down + Up pelo despacho do hero, com o botão `button`.
fn click(h: &mut HeroScreen, id: NodeId, button: PointerButton) {
    let (x, y) = centre_of(h, id);
    let arena = bumpalo::Bump::new();
    let mut events = Vec::new();
    for kind in [PointerKind::Down, PointerKind::Up] {
        let e = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button,
            timestamp_ns: 0,
        };
        events.extend_from_slice(h.handle_pointer(e, &arena));
    }
    for e in events {
        h.apply_event(e);
    }
}

/// A célula dos Assets que pinta o Kanban «Short film — production» (está no Quadro).
fn film_cell(h: &HeroScreen) -> NodeId {
    let film = h
        .documents
        .asset_listing()
        .into_iter()
        .find_map(|(p, n)| match p {
            DragPayload::Kanban { id } if n == "Short film \u{2014} production" => Some(id),
            _ => None,
        })
        .expect("o Kanban da cena 30");
    let i = (0..64)
        .find(|&i| {
            ph2d_panel_asset_browser::state::probe_painted_at(i)
                == Some(AssetRef::Kanban { id: film })
        })
        .expect("o cartão pintado");
    asset_cell_id(i)
}

/// 📷 O menu do cartão (Open · Delete…) e a pergunta de apagar, pelo botão direito e pelo clique.
#[test]
#[ignore = "fotos: precisa da placa"]
fn foto_d17_menu_and_delete_question() {
    let mut shots = crate::smoke::shots::Shots::new(DIR);
    let (mut h, mut ts) = staged();
    let cell = film_cell(&h);
    click(&mut h, cell, PointerButton::Secondary);
    frame(&mut h, &mut ts);
    assert!(
        matches!(
            h.store.context_menu().map(|r| r.kind),
            Some(ph2d_editor_core::interaction::ContextMenuKind::AssetDocCard { .. })
        ),
        "{:?}",
        h.store.context_menu()
    );
    shots.shot(&mut h, "menu_do_kanban_nos_assets");
    click(&mut h, CTX_MENU_ASSET_DELETE, PointerButton::Primary);
    frame(&mut h, &mut ts);
    assert!(
        matches!(
            h.store.context_menu().map(|r| r.kind),
            Some(ph2d_editor_core::interaction::ContextMenuKind::ConfirmDeleteKanban { .. })
        ),
        "{:?}",
        h.store.context_menu()
    );
    shots.shot(&mut h, "apagar_kanban_pergunta");
}

/// 📷 Vista afastada (zoom 0,5), `place` larga no canvas vazio por cima do Kanban da cena 30, pinta até
/// as imagens assentarem (o desenho descodifica fora do fio) e fotografa.
fn shot_dropped(name: &str, place: impl FnOnce(&mut HeroScreen, [f32; 2])) {
    let mut shots = crate::smoke::shots::Shots::new(DIR);
    let (mut h, mut ts) = staged();
    crate::images::install();
    h.documents.active_board_mut().unwrap().camera.zoom = 0.5;
    frame(&mut h, &mut ts);
    place(&mut h, [HERO_VIEWPORT_W * 0.5, HERO_VIEWPORT_H * 0.17]);
    let t0 = std::time::Instant::now();
    while h.documents.images_pending() && t0.elapsed().as_secs() < 20 {
        frame(&mut h, &mut ts);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    for _ in 0..30 {
        frame(&mut h, &mut ts);
    }
    shots.shot(&mut h, name);
}

/// 📷 O Kanban vazio largado no canvas do Quadro (a ponta da queda: `link::place_at`).
#[test]
#[ignore = "fotos: precisa da placa"]
fn foto_d17_kanban_dropped_on_the_board() {
    shot_dropped("kanban_largado_no_quadro", |h, at| {
        let empty = h
            .documents
            .asset_listing()
            .into_iter()
            .find_map(|(p, n)| match p {
                DragPayload::Kanban { id } if n == tr("board.smoke.d17.empty_kanban") => Some(id),
                _ => None,
            });
        let link = ph2d_editor_core::screens::hero::board_video::kanban::link::place_at;
        assert!(link(h, empty.unwrap(), at.map(f64::from)).is_some());
    });
}

/// 📷 A «Clouds» dos Assets largada no canvas vazio do «Production board», seleccionada.
#[test]
#[ignore = "fotos: precisa da placa"]
fn foto_d17_image_dropped_on_the_board() {
    shot_dropped("imagem_largada_no_quadro", |h, at| {
        let (db, clouds) = factory_clouds();
        assert!(crate::images::drop_asset(h, &db, clouds, at).is_none());
    });
}

/// A arte de fábrica no `AssetDb` como a app a regista, e o blake3 da «Clouds».
fn factory_clouds() -> (ph2d_asset::AssetDb, [u8; 32]) {
    let (db, mut atlas) = (ph2d_asset::AssetDb::new(), Default::default());
    ph2d_render::factory_images::register(&db, &mut atlas);
    (db, *atlas[&ph2d_render::CLOUDS_TILE_KEY].as_bytes())
}

fn images(h: &HeroScreen) -> Vec<ph2d_board_model::Element> {
    let live = h.documents.active_board().unwrap().doc.live();
    live.filter(|e| e.image().is_some()).cloned().collect()
}

/// ⭐ D17 — a «Clouds» dos Assets largada no Quadro: UMA imagem com o meio no ponto, os píxeis do asset
/// (PNG embutido) no tamanho dele, seleccionada (a barra dela pinta-se); `Ctrl+Z` tira-a e o asset fica.
/// Sem píxeis no `AssetDb`, avisa e nada entra.
#[test]
fn asset_image_dropped_on_the_board_lands_selected_and_undoes() {
    use ph2d_editor_core::screens::hero::{board_bar, board_keys};
    let (mut h, mut ts) = staged();
    let (db, clouds) = factory_clouds();
    let corners = board_bar::node(board_bar::Item::Round(true));
    let bar = |h: &HeroScreen| {
        (0..400 * 400)
            .any(|i| h.hit_index.hit((i % 400 * 4) as f32, (i / 400 * 4) as f32) == Some(corners))
    };
    let before = images(&h);
    assert!(!bar(&h));
    let missing = crate::images::drop_asset(&mut h, &ph2d_asset::AssetDb::new(), [3; 32], [9.0; 2]);
    assert!(missing.is_some_and(|t| t.message == tr("board.images.asset_missing")));
    let at = [HERO_VIEWPORT_W * 0.5, HERO_VIEWPORT_H * 0.2];
    assert!(crate::images::drop_asset(&mut h, &db, clouds, at).is_none());
    let mut new = images(&h);
    new.retain(|e| !before.iter().any(|o| o.id == e.id));
    assert_eq!(new.len(), 1, "UMA imagem");
    let (e, c, b) = (&new[0], h.last_canvas, h.documents.active_board().unwrap());
    let s = b
        .camera
        .to_screen([c.x, c.y, c.w, c.h].map(f64::from), e.center());
    assert!(
        (0..2).all(|i| (s[i] - f64::from(at[i])).abs() < 1e-6),
        "{s:?}"
    );
    let id = ph2d_asset::AssetId::from_digest(clouds);
    let asset = db.get(&id).unwrap();
    let (w, hh, px) = asset.image_rgba8().unwrap();
    let file = b.file(e.image().unwrap().file).expect("os bytes no quadro");
    assert_eq!((file.size, file.mime()), ([w, hh], Some("image/png")));
    assert_eq!(
        crate::images::decode(&file.bytes.0, None).unwrap().rgba,
        *px
    );
    assert!((e.w / e.h - f64::from(w) / f64::from(hh)).abs() < 1e-6);
    frame(&mut h, &mut ts);
    assert!(bar(&h), "a barra da imagem largada não está à vista");
    let ctrl = ph2d_host::Modifiers {
        ctrl: true,
        ..Default::default()
    };
    let z = board_keys::BoardKey::Char('z');
    assert!(board_keys::key(&mut h, &mut ts, z, ctrl, true, None, None).consumed);
    assert_eq!(images(&h).len(), before.len(), "Ctrl+Z tira-a");
    assert!(db.get(&id).is_some(), "o asset fica");
}
