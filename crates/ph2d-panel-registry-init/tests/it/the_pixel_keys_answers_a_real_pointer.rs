//! **A janela KEYBOARD SHORTCUTS responde a um PONTEIRO real** (P12a): a tela inteira pintada, com
//! todos os painéis registados; `Add` arma a captura, o acorde chega pelo canal (como a família o
//! manda), e só o `Confirm` junta a tecla — o mapa novo sai em `toma_keymap`. A tecla de outra
//! acção pergunta e só o `Replace` a tira.

use bumpalo::Bump;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, PANEL_PIXEL_KEYS, paint_hero_screen,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_keys::ids;
use ph2d_pixel_keys::{Chord, Context, PixelAction as A};
use winit::keyboard::KeyCode as K;

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
fn add_captura_e_confirm_juntam_a_tecla_e_a_tomada_pergunta() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let _ = ph2d_panel_pixel_keys::toma_keymap();
    // O ficheiro do utilizador vai para uma pasta do teste: nunca o `~/.ph2d` do dono.
    let pasta = std::env::temp_dir().join(format!("ph2d-pixel-keys-{}", std::process::id()));
    std::fs::create_dir_all(&pasta).expect("pasta do teste");
    let ficheiro = pasta.join("pixel-keys.txt");
    ph2d_panel_pixel_keys::canal::usa_ficheiro(Some(ficheiro.clone()));
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = HeroScreen::new(NodeId(1));
    // Aberta como a família a abre (a `PixelApp` lê `pede_abrir` e acende a visibilidade): o
    // pedido é do processo, e os outros testes pintam heróis em paralelo.
    hero.panel_visibility.insert(PANEL_PIXEL_KEYS, true);
    assenta(&mut hero, &mut ts, 60);
    // `Export As` é a primeira linha do primeiro cartão de secção: o Add dela está à vista.
    clica(&mut hero, ids::add(A::ExportAs), "Add de Export As");
    assenta(&mut hero, &mut ts, 1);
    assert!(ph2d_panel_pixel_keys::a_capturar(), "a captura armou");
    let f12 = Chord::new(K::F12);
    assert!(ph2d_panel_pixel_keys::oferece(f12));
    assenta(&mut hero, &mut ts, 2);
    assert!(
        ph2d_panel_pixel_keys::toma_keymap().is_none(),
        "premir não é confirmar"
    );
    clica(&mut hero, ids::CONFIRM, "Confirm");
    let k = ph2d_panel_pixel_keys::toma_keymap().expect("o mapa novo vai à família");
    assert!(k.keys_of(A::ExportAs).contains(&(f12, Context::Normal)));
    assert_eq!(
        std::fs::read_to_string(&ficheiro).expect("gravou ao mudar"),
        k.to_text(),
        "o ficheiro é o delta"
    );
    assenta(&mut hero, &mut ts, 2);
    clica(&mut hero, ids::EXPORT, "Export");
    // A família abre o diálogo do app e devolve o caminho; a janela grava no quadro seguinte.
    let troca = pasta.join("pixel-keys-export.txt");
    assert_eq!(
        ph2d_panel_pixel_keys::toma_troca(),
        Some(ph2d_panel_pixel_keys::Troca::Export)
    );
    ph2d_panel_pixel_keys::entrega_troca(ph2d_panel_pixel_keys::Troca::Export, troca.clone());
    assenta(&mut hero, &mut ts, 1);
    assert_eq!(std::fs::read_to_string(&troca).ok(), Some(k.to_text()));
    assert!(!ph2d_panel_pixel_keys::a_capturar());

    // `B` é do lápis: pergunta; Replace tira-a.
    assenta(&mut hero, &mut ts, 2);
    clica(&mut hero, ids::add(A::ExportAs), "Add de Export As");
    assenta(&mut hero, &mut ts, 1);
    assert!(ph2d_panel_pixel_keys::oferece(Chord::new(K::KeyB)));
    assenta(&mut hero, &mut ts, 2);
    clica(&mut hero, ids::CONFIRM, "Confirm");
    assert!(
        ph2d_panel_pixel_keys::toma_keymap().is_none(),
        "nunca rouba em silêncio"
    );
    assenta(&mut hero, &mut ts, 2);
    clica(&mut hero, ids::REPLACE, "Replace");
    let k = ph2d_panel_pixel_keys::toma_keymap().expect("o Replace muda o mapa");
    assert_eq!(
        k.resolve(Chord::new(K::KeyB), Context::Normal),
        Some(A::ExportAs)
    );

    // Repõe o ficheiro do utilizador (o teste corre com um HOME de teste).
    assenta(&mut hero, &mut ts, 2);
    clica(&mut hero, ids::RESET_ALL, "Reset All");
    assenta(&mut hero, &mut ts, 2);
    clica(&mut hero, ids::CONFIRM, "Confirm do Reset All");
    let k = ph2d_panel_pixel_keys::toma_keymap().expect("o Reset All muda o mapa");
    assert_eq!(k, ph2d_pixel_keys::Keymap::factory());

    // Import SOMA o ficheiro exportado (Aseprite R6): o F12 volta.
    assenta(&mut hero, &mut ts, 2);
    clica(&mut hero, ids::IMPORT, "Import");
    assert_eq!(
        ph2d_panel_pixel_keys::toma_troca(),
        Some(ph2d_panel_pixel_keys::Troca::Import)
    );
    ph2d_panel_pixel_keys::entrega_troca(ph2d_panel_pixel_keys::Troca::Import, troca);
    assenta(&mut hero, &mut ts, 1);
    let k = ph2d_panel_pixel_keys::toma_keymap().expect("o Import muda o mapa");
    assert!(k.keys_of(A::ExportAs).contains(&(f12, Context::Normal)));
    // A busca atravessa as secções: escrever no campo deixa só as acções que casam. (No MESMO
    // teste: o pedido de abrir é do processo, e dois heróis em paralelo disputá-lo-iam.)
    assenta(&mut hero, &mut ts, 2);
    clica(&mut hero, ids::SEARCH, "a busca");
    let arena = Bump::new();
    let ev: Vec<_> = "outline"
        .chars()
        .flat_map(|c| hero.handle_text_input(c, &arena).to_vec())
        .collect();
    for e in ev {
        hero.apply_event(e);
    }
    assenta(&mut hero, &mut ts, 2);
    assert!(
        hero.hit_index.rect_for(ids::add(A::Outline)).is_some(),
        "Outline casa"
    );
    assert!(
        hero.hit_index.rect_for(ids::add(A::ExportAs)).is_none(),
        "Export As não casa"
    );
    ph2d_panel_pixel_keys::canal::usa_ficheiro(None);
    let _ = std::fs::remove_dir_all(&pasta);
}
