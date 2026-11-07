//! **A janela Tilemap do Pixel responde a um PONTEIRO real:** a tela inteira pintada, com todos os
//! painéis registados; Down+Up no rect pintado vira o pedido que a família recebe. Chamar o
//! `apply_event` direto não prova este caminho, nem que o clique deixa de cair no canvas.

use std::sync::Arc;

use bumpalo::Bump;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, PANEL_PIXEL_TILES, paint_hero_screen,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId, ids as nucleo};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_tiles::canal::{self, Alvo, Conjunto, Contexto, Mao, Pedido, Tela};
use ph2d_panel_pixel_tiles::ids;
use ph2d_pixel_core::Forma;

const TILE: u16 = 4;
const JANELA: NodeId = nucleo::PIXEL_TILES_PANEL;

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

fn assenta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) {
    // A mola dos cartões corre no relógio: com passo fixo, os rects assentam sempre igual.
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    for _ in 0..60 {
        let mut scene = ph2d_vector::VectorScene::new();
        ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || paint_hero_screen(hero, vp, &mut scene, ts),
        );
    }
}

fn centro(hero: &HeroScreen, id: NodeId, nome: &str) -> (f32, f32) {
    let r = hero
        .hit_index
        .rect_for(id)
        .unwrap_or_else(|| panic!("{nome} não foi pintado"));
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

fn clica(hero: &mut HeroScreen, id: NodeId, nome: &str) {
    let (x, y) = centro(hero, id, nome);
    let arena = Bump::new();
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    assert!(
        ev.contains(&WidgetEvent::Click(id)),
        "{nome}: o ponteiro deu {ev:?}, e nenhum Click dele; em ({x},{y}) acerta {:?}; a janela {:?}",
        hero.hit_index.hit_with_rect(x, y),
        hero.store.panel_rect(JANELA)
    );
    for e in ev {
        hero.apply_event(e);
    }
}

fn arrasta(hero: &mut HeroScreen, de: (f32, f32), passo: (f32, f32)) {
    let arena = Bump::new();
    let (x, y) = (de.0 + passo.0, de.1 + passo.1);
    let _ = hero.handle_pointer(ptr(PointerKind::Down, de.0, de.1), &arena);
    let _ = hero.handle_pointer(ptr(PointerKind::Move, x, y), &arena);
    let _ = hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena);
}

fn com_tileset() -> Contexto {
    let px = vec![255_u8; usize::from(TILE) * usize::from(TILE) * 4];
    Contexto {
        mao: Some(Mao::default()),
        tile_w: TILE,
        tile_h: TILE,
        imagens: Arc::new(vec![px; 3]),
        ..Contexto::default()
    }
}

fn hero_com_a_janela(ts: &mut ph2d_text::TextSystem, x: Contexto) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let _ = canal::take_pedidos();
    canal::publish_contexto(x);
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(PANEL_PIXEL_TILES, true);
    assenta(&mut hero, ts);
    hero
}

#[test]
fn o_alvo_e_o_tile_chegam_a_familia() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero_com_a_janela(&mut ts, com_tileset());
    clica(&mut hero, ids::TARGET_TILES, "Tiles");
    assert_eq!(canal::take_pedidos(), [Pedido::Alvo(Alvo::Tiles)]);
    clica(&mut hero, ids::tile(2), "o tile 2");
    assert_eq!(canal::take_pedidos(), [Pedido::Tile(2)]);
}

/// Dois cliques depressa no mesmo botão são dois gestos: o despacho troca o segundo por um
/// `DoubleClick`, e nenhum destes controlos distingue os dois.
#[test]
fn a_mao_rapida_nao_perde_o_segundo_clique() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero_com_a_janela(&mut ts, com_tileset());
    let (x, y) = centro(&hero, ids::ADD_TILE, "Add");
    let arena = Bump::new();
    for kind in [
        PointerKind::Down,
        PointerKind::Up,
        PointerKind::Down,
        PointerKind::Up,
    ] {
        let ev = hero.handle_pointer(ptr(kind, x, y), &arena).to_vec();
        for e in ev {
            hero.apply_event(e);
        }
    }
    assert_eq!(canal::take_pedidos(), [Pedido::NovoTile, Pedido::NovoTile]);
}

/// A janela flutua sobre o desenho, e dentro dela o ponteiro é dela: é por este juiz que a shell
/// decide o que entrega à ferramenta. Fechada, o mesmo ponto volta a ser do canvas.
#[test]
fn o_clique_dentro_da_janela_nao_cai_no_canvas() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero_com_a_janela(&mut ts, com_tileset());
    let r = hero
        .store
        .panel_rect(JANELA)
        .expect("a janela não publicou o rect");
    let l = hero.last_layout.expect("houve um quadro");
    assert!(
        l.draw_area.x <= r.x && l.draw_area.x + l.draw_area.w >= r.x + r.w,
        "a janela {r:?} não flutua sobre o desenho {:?}",
        l.draw_area
    );
    let (x, y) = centro(&hero, ids::tile(2), "o tile 2");
    assert!(r.contains(x, y), "o tile 2 está fora da janela {r:?}");
    assert_eq!(hero.chrome_panel_at(x, y), Some(JANELA));
    clica(&mut hero, ids::tile(2), "o tile 2");
    assert_eq!(canal::take_pedidos(), [Pedido::Tile(2)]);

    hero.panel_visibility.insert(PANEL_PIXEL_TILES, false);
    assenta(&mut hero, &mut ts);
    assert_eq!(
        hero.store.panel_rect(JANELA),
        None,
        "fechada, ainda publica"
    );
    assert_eq!(
        hero.chrome_panel_at(x, y),
        None,
        "o controlo: sem a janela, o ponto é do canvas"
    );
}

/// O título leva a janela e os alvos dela; o clique acerta no sítio novo.
#[test]
fn arrastar_o_titulo_move_a_janela() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero_com_a_janela(&mut ts, com_tileset());
    let antes = hero.store.panel_rect(JANELA).expect("a janela");
    let tile_antes = centro(&hero, ids::tile(1), "o tile 1");
    let pega = centro(&hero, ids::TILES_DRAG, "a alça");
    let passo = (-60.0, 30.0);
    arrasta(&mut hero, pega, passo);
    assenta(&mut hero, &mut ts);
    let depois = hero.store.panel_rect(JANELA).expect("a janela");
    assert_eq!(
        (depois.x - antes.x, depois.y - antes.y),
        passo,
        "a janela foi de {antes:?} para {depois:?}"
    );
    let tile = centro(&hero, ids::tile(1), "o tile 1");
    assert_eq!(
        (tile.0 - tile_antes.0, tile.1 - tile_antes.1),
        passo,
        "o tile ficou para trás"
    );
    clica(&mut hero, ids::tile(1), "o tile 1 no sítio novo");
    assert_eq!(canal::take_pedidos(), [Pedido::Tile(1)]);
}

/// A janela de criar, com e sem tileset activo (a camada normal é quem a pede sem nenhum): o verbo
/// responde, a faixa dele é da janela, e o título também a arrasta.
#[test]
fn a_janela_de_criar_pede_a_camada() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for x in [com_tileset(), Contexto::default()] {
        let mut hero = hero_com_a_janela(&mut ts, x);
        canal::abrir(Tela::NovaCamada);
        hero.panel_visibility.insert(PANEL_PIXEL_TILES, true);
        assenta(&mut hero, &mut ts);
        let (x, y) = centro(&hero, ids::TILES_OK, "o verbo");
        assert_eq!(
            hero.chrome_panel_at(x, y),
            Some(JANELA),
            "o verbo no canvas"
        );
        let antes = centro(&hero, ids::TILES_OK, "o verbo");
        let pega = centro(&hero, ids::TILES_DRAG, "a alça");
        arrasta(&mut hero, pega, (40.0, 20.0));
        assenta(&mut hero, &mut ts);
        let depois = centro(&hero, ids::TILES_OK, "o verbo");
        assert_eq!((depois.0 - antes.0, depois.1 - antes.1), (40.0, 20.0));
        clica(&mut hero, ids::TILES_OK, "o verbo");
        assert_eq!(
            canal::take_pedidos(),
            [Pedido::NovaCamada {
                conjunto: None,
                w: 16,
                h: 16,
                forma: Forma::Square,
            }]
        );
    }
}

/// O sprite já tem tilesets: a janela traz um escolhido, e o «novo» é que pede o tamanho.
#[test]
fn a_janela_oferece_os_tilesets_que_ja_existem() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let c = |id, tile_w, tile_h, tiles| Conjunto {
        id,
        tile_w,
        tile_h,
        tiles,
    };
    let x = Contexto {
        conjuntos: vec![c(7, 16, 16, 3), c(9, 8, 8, 0)],
        padrao: Some(7),
        ..com_tileset()
    };
    let mut hero = hero_com_a_janela(&mut ts, x);
    canal::abrir(Tela::NovaCamada);
    assenta(&mut hero, &mut ts);
    assert!(
        hero.hit_index.rect_for(ids::TILE_W).is_none(),
        "com um tileset escolhido, o tamanho é o dele"
    );
    clica(&mut hero, ids::TILES_OK, "o verbo");
    let pedido = |conjunto, w, h| {
        [Pedido::NovaCamada {
            conjunto,
            w,
            h,
            forma: Forma::Square,
        }]
    };
    assert_eq!(canal::take_pedidos(), pedido(Some(7), 16, 16));

    canal::abrir(Tela::NovaCamada);
    assenta(&mut hero, &mut ts);
    clica(&mut hero, ids::conjunto(1), "o outro tileset");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, ids::TILES_OK, "o verbo");
    assert_eq!(canal::take_pedidos(), pedido(Some(9), 8, 8));

    canal::abrir(Tela::NovaCamada);
    assenta(&mut hero, &mut ts);
    clica(&mut hero, ids::NEW_SET, "New tileset");
    assenta(&mut hero, &mut ts);
    assert!(
        hero.hit_index.rect_for(ids::TILE_W).is_some(),
        "o novo pede o tamanho"
    );
    clica(&mut hero, ids::TILES_OK, "o verbo");
    assert_eq!(canal::take_pedidos(), pedido(None, 16, 16));
}

/// O cartão Terrain e a forma da grelha, por ponteiro real, até ao pedido que a família recebe.
#[test]
fn o_terreno_e_a_forma_chegam_a_familia() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let x = Contexto {
        terrenos: vec!["Terrain 1".to_owned(), "Terrain 2".to_owned()],
        ..com_tileset()
    };
    let mut hero = hero_com_a_janela(&mut ts, x);
    clica(&mut hero, ids::TARGET_TERRAIN, "Terrain");
    assert_eq!(canal::take_pedidos(), [Pedido::Alvo(Alvo::Terreno)]);
    clica(&mut hero, ids::terreno(1), "o terreno 2");
    assert_eq!(canal::take_pedidos(), [Pedido::Terreno(1)]);
    clica(&mut hero, ids::NEW_TERRAIN, "New Terrain");
    assert_eq!(canal::take_pedidos(), [Pedido::NovoTerreno]);
    clica(&mut hero, ids::TERRAIN_TEMPLATE, "Template Layer");
    assert_eq!(canal::take_pedidos(), [Pedido::Gabarito]);
    clica(&mut hero, ids::DELETE_TERRAIN, "Delete Terrain");
    assert_eq!(canal::take_pedidos(), [Pedido::ApagaTerreno]);
    clica(&mut hero, ids::SHAPE_ISO, "Iso");
    assert_eq!(canal::take_pedidos(), [Pedido::Forma(Forma::Iso)]);
}

/// Em hex não há gabarito: a tela nem o oferece, e o terreno não aparece como destino.
#[test]
fn em_hex_o_cartao_terrain_nao_aparece() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let x = Contexto {
        forma: Forma::Hex,
        ..com_tileset()
    };
    let hero = hero_com_a_janela(&mut ts, x);
    assert!(hero.hit_index.rect_for(ids::TARGET_TERRAIN).is_none());
    assert!(hero.hit_index.rect_for(ids::NEW_TERRAIN).is_none());
    assert!(hero.hit_index.rect_for(ids::SHAPE_HEX).is_some());
}

/// A janela New Tilemap Layer leva a forma escolhida; a de converter não a oferece.
#[test]
fn a_janela_de_criar_escolhe_a_forma() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero_com_a_janela(&mut ts, Contexto::default());
    hero.panel_visibility.insert(PANEL_PIXEL_TILES, true);
    canal::abrir(Tela::NovaCamada);
    assenta(&mut hero, &mut ts);
    clica(&mut hero, ids::SHAPE_HEX, "Hex");
    assenta(&mut hero, &mut ts);
    clica(&mut hero, ids::TILES_OK, "Create");
    let [Pedido::NovaCamada { forma, .. }] = canal::take_pedidos()[..] else {
        panic!("sem o pedido da camada nova");
    };
    assert_eq!(forma, Forma::Hex);
    hero.panel_visibility.insert(PANEL_PIXEL_TILES, true);
    canal::abrir(Tela::Converter);
    assenta(&mut hero, &mut ts);
    assert!(hero.hit_index.rect_for(ids::TILES_OK).is_some());
    assert!(hero.hit_index.rect_for(ids::SHAPE_HEX).is_none());
}
