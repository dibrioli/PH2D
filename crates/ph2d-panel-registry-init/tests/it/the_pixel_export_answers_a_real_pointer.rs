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

/// Abre *File* e carrega na cascata `sub` (Import ▸ / Export ▸) pelo ponteiro.
fn abre_file_sub(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem, sub: NodeId) {
    hero.store.open_context_menu(ContextMenuRequest {
        x: 40.0,
        y: 40.0,
        kind: ContextMenuKind::MenuBarFile,
    });
    pinta(hero, ts);
    clica(hero, sub, "File ▸ cascata");
    let aberto = hero.store.context_menu().map(|r| r.kind);
    assert!(
        matches!(
            aberto,
            Some(ContextMenuKind::MenuBarFileImport | ContextMenuKind::MenuBarFileExport)
        ),
        "a cascata não abriu o submenu: {aberto:?}"
    );
    pinta(hero, ts);
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
        abre_file_sub(&mut hero, &mut ts, ids::MENUBAR_FILE_EXPORT_MENU);
        clica(&mut hero, ids::CTX_MENU_EXPORT_SVG, "Export SVG (controlo)");
        assert!(hero.file_menu.export_svg);
    }
    // ⭐ Desde 12/11 (dono) as linhas moram em File ▸ Import e File ▸ Export: o ponteiro abre File,
    //    carrega na cascata, e só então na folha.
    let (imp, exp) = (ids::MENUBAR_FILE_IMPORT_MENU, ids::MENUBAR_FILE_EXPORT_MENU);
    for (sub, id, nome, a) in [
        (imp, ids::CTX_MENU_PIXEL_OPEN, "Open", PixelFileAction::Open),
        (
            imp,
            ids::CTX_MENU_PIXEL_IMPORT_SHEET,
            "Import",
            PixelFileAction::ImportSheet,
        ),
        (
            exp,
            ids::CTX_MENU_PIXEL_EXPORT_AS,
            "Export As",
            PixelFileAction::ExportAs,
        ),
        (
            exp,
            ids::CTX_MENU_PIXEL_EXPORT_SHEET,
            "Export Sheet",
            PixelFileAction::ExportSheet,
        ),
        (
            exp,
            ids::CTX_MENU_PIXEL_EXPORT_TILESET,
            "Export Tileset",
            PixelFileAction::ExportTileset,
        ),
        (
            exp,
            ids::CTX_MENU_PIXEL_REPEAT_EXPORT,
            "Repeat",
            PixelFileAction::RepeatExport,
        ),
    ] {
        let mut hero = HeroScreen::new(NodeId(1));
        pinta(&mut hero, &mut ts);
        hero.store.set_pixel_file_menu(Some(true));
        abre_file_sub(&mut hero, &mut ts, sub);
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
        ..canal::Contexto::default()
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

/// A janela do File ▸ Import pelo ponteiro: um `.ase` parte de Pixel Sprite, a outra escolha muda a
/// resposta, e o Cancel não pede nada.
#[test]
fn a_janela_do_import_responde_ao_ponteiro() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = HeroScreen::new(NodeId(1));
    pinta(&mut hero, &mut ts);
    canal::publish_imagem(Some(canal::Imagem {
        rgba: std::sync::Arc::new(vec![255; 16 * 8 * 4]),
        w: 16,
        h: 8,
        seq: 1,
        lados_reais: (16, 8),
        entradas: 0,
        sugestao: None,
        com_dados: false,
    }));
    canal::publish_pergunta(Some(canal::Pergunta {
        nome: "a.ase".to_owned(),
        ficheiros: 1,
        lados: (16, 8),
        quadros: 3,
        camadas: 2,
        tags: 1,
        pixel_de_fabrica: true,
        erro: None,
    }));
    canal::abrir(Tela::ImportComo);
    hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, true);
    assenta(&mut hero, &mut ts);
    assert!(
        hero.hit_index.rect_for(e::EXPORT_RESET).is_none(),
        "nada a repor"
    );
    clica(&mut hero, e::OK, "Import");
    assert_eq!(
        canal::take_pedido(),
        Some(Pedido::ImportarComo { pixel: true })
    );
    clica(&mut hero, e::AS_IMAGE, "Image");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::OK, "Import");
    assert_eq!(
        canal::take_pedido(),
        Some(Pedido::ImportarComo { pixel: false })
    );
    clica(&mut hero, e::AS_PIXEL, "Pixel Sprite");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::CANCEL, "Cancel");
    assert_eq!(canal::take_pedido(), None);
    assert_eq!(hero.panel_visibility.get(PANEL_PIXEL_EXPORT), Some(&false));
}

/// A janela Export Tileset e as caixas da grelha na folha, por ponteiro real, até ao verbo.
#[test]
fn o_export_tileset_e_as_caixas_da_grelha_respondem() {
    use ph2d_pixel_core::TilesetId;
    use ph2d_pixel_io::exporta::Export;
    use ph2d_pixel_io::tiles_saida::{COLUNAS_INICIAIS, ExportTiles};
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = HeroScreen::new(NodeId(1));
    pinta(&mut hero, &mut ts);
    canal::publish_contexto(canal::Contexto {
        lados: (16, 8),
        quadros: 1,
        tilesets: vec![(7, 3, 16, 16), (9, 5, 32, 16)],
        ..canal::Contexto::default()
    });
    canal::abrir(Tela::Tileset);
    hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, true);
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::tileset_option(0), "o tileset 1");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::OK, "Export");
    assert_eq!(
        canal::take_pedido(),
        Some(Pedido::Exportar(Export::Tiles(ExportTiles {
            conjunto: TilesetId(7),
            colunas: COLUNAS_INICIAIS,
        })))
    );
    canal::abrir(Tela::Folha);
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::SPLIT_GRID, "Split grid");
    assenta(&mut hero, &mut ts);
    assert!(
        hero.hit_index.rect_for(e::TRIM_BY_GRID).is_none(),
        "sem recorte não se oferece"
    );
    clica(&mut hero, e::TRIM_CELS, "Trim cels");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::TRIM_BY_GRID, "Trim by grid");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::OK, "Export");
    match canal::take_pedido() {
        Some(Pedido::Exportar(Export::Folha(x))) => {
            assert!(x.opcoes.split_grid && x.opcoes.trim_by_grid);
        }
        o => panic!("{o:?}"),
    }
    // Sem tileset no sprite, as caixas da grelha não se oferecem.
    canal::publish_contexto(canal::Contexto::default());
    assenta(&mut hero, &mut ts);
    assert!(hero.hit_index.rect_for(e::SPLIT_GRID).is_none());
}

/// `Split slices` e `Area` só com slices; `Area` lista o canvas e as slices com gémeas, e a escolha
/// chega ao pedido. A slice que some volta a área ao canvas.
#[test]
fn split_slices_e_area_so_com_slices_e_chegam_ao_pedido() {
    use ph2d_pixel_core::SliceId;
    use ph2d_pixel_io::exporta::Export;
    use ph2d_pixel_io::render::Area;
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = HeroScreen::new(NodeId(1));
    pinta(&mut hero, &mut ts);
    let contexto = |slices: Vec<(SliceId, String)>| canal::Contexto {
        lados: (16, 8),
        quadros: 3,
        slices,
        ..canal::Contexto::default()
    };
    canal::publish_contexto(contexto(Vec::new()));
    canal::abrir(Tela::ExportAs);
    hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, true);
    assenta(&mut hero, &mut ts);
    assert!(
        hero.hit_index.rect_for(e::AREA).is_none(),
        "sem slices, sem Area"
    );
    canal::abrir(Tela::Folha);
    assenta(&mut hero, &mut ts);
    assert!(hero.hit_index.rect_for(e::SPLIT_SLICES).is_none());

    let gemeas = vec![
        (SliceId(5), "a".to_owned()),
        (SliceId(7), "a".to_owned()),
        (SliceId(6), "b".to_owned()),
    ];
    canal::publish_contexto(contexto(gemeas));
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::SPLIT_SLICES, "Split slices");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::OK, "Export da folha");
    match canal::take_pedido() {
        Some(Pedido::Exportar(Export::Folha(x))) => assert!(x.opcoes.split_slices),
        o => panic!("{o:?}"),
    }

    canal::abrir(Tela::ExportAs);
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::AREA, "a caixa Area");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::area_option(1), "a segunda gémea");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::OK, "Export");
    match canal::take_pedido() {
        Some(Pedido::Exportar(Export::Ficheiro(x))) => {
            assert_eq!(x.pedido.area, Area::Slice(SliceId(7)));
        }
        o => panic!("{o:?}"),
    }

    canal::abrir(Tela::ExportAs);
    canal::publish_contexto(contexto(vec![(SliceId(5), "a".to_owned())]));
    assenta(&mut hero, &mut ts);
    // Outro controlo no meio: dois cliques seguidos no mesmo botão seriam um duplo clique.
    clica(&mut hero, e::FORMATS[0], "PNG");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::OK, "Export");
    match canal::take_pedido() {
        Some(Pedido::Exportar(Export::Ficheiro(x))) => assert_eq!(x.pedido.area, Area::Canvas),
        o => panic!("{o:?}"),
    }
}

/// P14d: Output ▸ **Godot SpriteFrames (.tres)** e **Normal map sheet** respondem ao ponteiro e chegam
/// ao pedido; a caixa Normal map sheet só aparece quando o sprite tem camada Normal Map.
#[test]
fn normal_map_sheet_e_godot_chegam_ao_pedido() {
    use ph2d_pixel_io::exporta::Export;
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = HeroScreen::new(NodeId(1));
    pinta(&mut hero, &mut ts);
    let contexto = |tem_normal| canal::Contexto {
        lados: (16, 8),
        quadros: 3,
        tem_normal,
        ..canal::Contexto::default()
    };
    canal::publish_contexto(contexto(false));
    canal::abrir(Tela::Folha);
    hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, true);
    assenta(&mut hero, &mut ts);
    assert!(
        hero.hit_index.rect_for(e::NORMAL_SHEET).is_none(),
        "sem camada Normal Map, sem Normal map sheet"
    );
    canal::publish_contexto(contexto(true));
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::NORMAL_SHEET, "Normal map sheet");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::GODOT, "Godot SpriteFrames (.tres)");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, e::OK, "Export da folha");
    match canal::take_pedido() {
        Some(Pedido::Exportar(Export::Folha(x))) => {
            assert!(x.opcoes.normal, "Normal map sheet no pedido");
            assert!(x.opcoes.godot, "Godot no pedido");
        }
        o => panic!("{o:?}"),
    }
}
