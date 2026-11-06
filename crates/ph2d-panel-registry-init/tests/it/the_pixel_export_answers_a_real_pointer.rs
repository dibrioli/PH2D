//! ⭐⭐ **O Export do Pixel responde a um PONTEIRO real** (report do dono 05/11: «o botão não abre»):
//! a tela inteira pintada, com todos os painéis registados; Down+Up no rect pintado vira o evento que
//! o `apply_event` da tela recebe. Chamar o `apply_event` direto não prova este caminho.

use bumpalo::Bump;
use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::pixel_panel::PixelFileAction;
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, PANEL_PIXEL_EXPORT, paint_hero_screen,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId, ids};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_export::canal::{self, Pedido, Tela};
use ph2d_panel_pixel_export::ids as e;

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

fn pinta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) {
    let mut scene = ph2d_vector::VectorScene::new();
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    paint_hero_screen(hero, vp, &mut scene, ts);
}

fn clica(hero: &mut HeroScreen, id: NodeId, nome: &str) {
    let r = hero
        .hit_index
        .rect_for(id)
        .unwrap_or_else(|| panic!("{nome} não foi pintado"));
    let arena = Bump::new();
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    assert!(
        ev.contains(&WidgetEvent::Click(id)),
        "{nome}: o ponteiro deu {ev:?}, e nenhum Click dele; em ({x},{y}) rect {r:?} acerta {:?}; o painel {:?}",
        hero.hit_index.hit_with_rect(x, y),
        hero.store.panel_rect(ids::PIXEL_EXPORT_PANEL)
    );
    for x in ev {
        hero.apply_event(x);
    }
}

#[test]
fn a_linha_do_menu_file_pede_o_comando() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    {
        // O controlo: uma linha que já funcionava, pelo mesmo caminho.
        let mut hero = HeroScreen::new(NodeId(1));
        pinta(&mut hero, &mut ts);
        hero.store.set_pixel_file_menu(Some(true));
        hero.store.open_context_menu(ContextMenuRequest {
            x: 40.0,
            y: 40.0,
            kind: ContextMenuKind::MenuBarFile,
        });
        pinta(&mut hero, &mut ts);
        clica(&mut hero, ids::CTX_MENU_EXPORT_SVG, "Export SVG (controlo)");
        assert!(hero.file_menu.export_svg);
    }
    for (id, nome, a) in [
        (ids::CTX_MENU_PIXEL_OPEN, "Open", PixelFileAction::Open),
        (
            ids::CTX_MENU_PIXEL_IMPORT_SHEET,
            "Import",
            PixelFileAction::ImportSheet,
        ),
        (
            ids::CTX_MENU_PIXEL_EXPORT_AS,
            "Export As",
            PixelFileAction::ExportAs,
        ),
        (
            ids::CTX_MENU_PIXEL_EXPORT_SHEET,
            "Export Sheet",
            PixelFileAction::ExportSheet,
        ),
        (
            ids::CTX_MENU_PIXEL_REPEAT_EXPORT,
            "Repeat",
            PixelFileAction::RepeatExport,
        ),
    ] {
        let mut hero = HeroScreen::new(NodeId(1));
        pinta(&mut hero, &mut ts);
        hero.store.set_pixel_file_menu(Some(true));
        hero.store.open_context_menu(ContextMenuRequest {
            x: 40.0,
            y: 40.0,
            kind: ContextMenuKind::MenuBarFile,
        });
        pinta(&mut hero, &mut ts);
        clica(&mut hero, id, nome);
        assert_eq!(hero.store.take_pixel_file_request(), Some(a), "{nome}");
        assert!(
            hero.store.context_menu().is_none(),
            "{nome}: o menu fica aberto"
        );
    }
}

#[test]
fn o_verbo_da_janela_chega_a_familia() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = HeroScreen::new(NodeId(1));
    pinta(&mut hero, &mut ts);
    canal::abrir(Tela::ExportAs);
    hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, true);
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::FORMATS[1], "GIF");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::OK, "Export");
    let p = canal::take_pedido().expect("o Export não pediu nada à família");
    assert!(matches!(p, Pedido::Exportar(_)), "{p:?}");
}

fn assenta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) {
    // A mola dos cartões corre no relógio: com passo fixo, os rects assentam sempre igual.
    for _ in 0..60 {
        ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || pinta(hero, ts),
        );
    }
}

/// As listas suspensas, as caixas e as outras duas janelas, pelo mesmo ponteiro.
#[test]
fn as_listas_e_as_outras_janelas_respondem() {
    use ph2d_pixel_io::exporta::Export;
    use ph2d_pixel_io::render::Camadas;
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = HeroScreen::new(NodeId(1));
    pinta(&mut hero, &mut ts);
    canal::publish_contexto(canal::Contexto {
        lados: (16, 8),
        quadros: 3,
        camadas: Vec::new(),
        tags: Vec::new(),
    });
    canal::abrir(Tela::ExportAs);
    hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, true);
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::LAYERS, "a caixa Layers");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::LAYERS_ALL, "a linha All");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::OK, "Export");
    match canal::take_pedido() {
        Some(Pedido::Exportar(Export::Ficheiro(x))) => {
            assert_eq!(x.pedido.camadas, Camadas::Todas);
        }
        o => panic!("{o:?}"),
    }

    canal::abrir(Tela::Folha);
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::POW2, "Power of 2");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::OK, "Export da folha");
    match canal::take_pedido() {
        Some(Pedido::Exportar(Export::Folha(x))) => assert!(x.opcoes.potencia_de_2),
        o => panic!("{o:?}"),
    }

    canal::publish_imagem(Some(canal::Imagem {
        rgba: std::sync::Arc::new(vec![255; 64 * 32 * 4]),
        w: 64,
        h: 32,
        seq: 1,
        lados_reais: (64, 32),
        entradas: 0,
        sugestao: None,
        com_dados: false,
    }));
    canal::abrir(Tela::Import);
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::BY_COLUMNS, "By columns");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::OK, "Import");
    match canal::take_pedido() {
        Some(Pedido::Importar(g)) => assert!(g.por_colunas),
        o => panic!("{o:?}"),
    }
    clica(&mut hero, e::CANCEL, "Cancel");
    assert_eq!(hero.panel_visibility.get(PANEL_PIXEL_EXPORT), Some(&false));
}
