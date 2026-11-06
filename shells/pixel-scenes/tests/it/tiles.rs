//! **A tela dos tilemaps pelo gesto real** (P10b): o evento entra por onde o artista entra e o efeito
//! confere-se no documento.

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, KeyAnswer, Press};
use ph2d_app_pixel::{PixelApp, Tool, mode, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{PixelLayerAction as A, PixelRequest, PixelSpriteAction as Sp};
use ph2d_editor_core::screens::hero::{HeroScreen, PANEL_PIXEL_TILES};
use ph2d_panel_pixel_tiles::canal::{self, Alvo, Mao, Pedido};
use ph2d_pixel_core::{Sprite, TileRef, derived_is_consistent};
use ph2d_pixel_tiles::desenho::Modo;
use ph2d_pixel_tiles::exemplo::{LADO, MAPA};
use ph2d_pixel_tiles::mao::tileset_de;
use ph2d_pixel_tiles::mapa::Mapa;
use ph2d_render::Camera2d;
use winit::keyboard::KeyCode;

use crate::edit::{PPM, WINDOW, at, canvas, spawn_root};

struct Palco {
    app: PixelApp,
    sim: SimWorld,
    hero: HeroScreen,
    toasts: ToastQueue,
    cam: Camera2d,
    bits: u64,
    key: u32,
}

impl Palco {
    fn novo() -> Self {
        let _ = canal::take_pedidos();
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(ph2d_app_pixel::smoke::stage(
            &mut app, &mut sim, 21, spawn_root
        ));
        let (bits, key) = mode::sprites(sim.world())[0];
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        hero.gizmo.mode.enter(bits, ObjectMode::Draw);
        let mut cam = Camera2d::default();
        app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
        let mut p = Self {
            app,
            sim,
            hero,
            toasts: ToastQueue::new(),
            cam,
            bits,
            key,
        };
        p.quadro();
        p
    }

    fn quadro(&mut self) {
        panel::frame(
            &mut self.app,
            self.sim.world(),
            &mut self.hero,
            &mut self.toasts,
        );
    }

    fn pede(&mut self, p: Pedido) {
        canal::pede(p);
        self.quadro();
    }

    fn s(&self) -> &Sprite {
        &self.app.doc(self.key).expect("doc").sprite
    }

    fn mao(&self) -> Mao {
        canal::contexto().mao.expect("camada de tilemap activa")
    }

    fn passos(&self) -> usize {
        self.app.doc(self.key).expect("doc").history.undo_len()
    }

    fn n_tiles(&self) -> usize {
        let d = self.app.doc(self.key).expect("doc");
        tileset_de(&d.sprite, d.layer).expect("tileset").tiles.len()
    }

    fn tile_em(&self, c: i32, r: i32) -> TileRef {
        let d = self.app.doc(self.key).expect("doc");
        let set = tileset_de(&d.sprite, d.layer).expect("tileset");
        Mapa::de(&d.sprite, d.layer, d.frame, set).get(c, r)
    }

    fn ponto(&self, c: i32, r: i32) -> (f32, f32) {
        let l = i32::from(LADO);
        at(
            &self.app,
            &self.sim,
            &self.cam,
            self.bits,
            c * l + 3,
            r * l + 3,
        )
    }

    fn prime(&mut self, c: i32, r: i32) -> Press {
        let p = self.ponto(c, r);
        let cv = canvas(&self.sim, &self.cam, self.bits);
        input::press(&mut self.app, &cv, p, Button::Primary, Mods::default())
    }

    fn move_para(&mut self, c: i32, r: i32) {
        let p = self.ponto(c, r);
        let cv = canvas(&self.sim, &self.cam, self.bits);
        input::moved(&mut self.app, &cv, p, Mods::default());
    }

    fn espaco(&mut self, held: bool) {
        assert!(input::space(
            &mut self.app,
            self.sim.world(),
            Some(self.bits),
            held
        ));
    }

    fn tecla(&mut self, code: KeyCode) -> Option<KeyAnswer> {
        input::key(
            &mut self.app,
            self.sim.world(),
            Some(self.bits),
            code,
            false,
            false,
        )
    }

    fn avisos(&mut self) -> usize {
        self.quadro();
        let n = self.toasts.len();
        self.toasts = ToastQueue::new();
        n
    }
}

#[test]
fn a_cena_21_abre_com_o_mapa_desenhado_e_o_painel_aberto() {
    let p = Palco::novo();
    assert_eq!(p.n_tiles(), 4, "o vazio e três tiles");
    assert_eq!(p.tile_em(2, 0), TileRef::new(3, false, false, false));
    assert_eq!(
        p.tile_em(4, 0),
        TileRef::new(3, true, false, false),
        "a seta virada"
    );
    assert_eq!(p.tile_em(0, 0), TileRef::EMPTY);
    assert_eq!(p.hero.panel_visibility.get(PANEL_PIXEL_TILES), Some(&true));
    assert_eq!(p.mao().modo, Modo::Auto, "o modo inicial é o Auto");
    assert!(derived_is_consistent(p.s()));
    assert_eq!(MAPA.len(), 4);
}

#[test]
fn draw_tiles_arrasta_sem_saltar_celulas_e_vale_um_passo_de_desfazer() {
    let mut p = Palco::novo();
    p.pede(Pedido::Alvo(Alvo::Tiles));
    p.pede(Pedido::Tile(2));
    p.pede(Pedido::Flip(canal::Eixo::Y));
    let (antes, mapa) = (p.passos(), p.s().clone());
    assert_eq!(p.prime(0, 0), Press::Done);
    p.move_para(3, 0);
    assert!(p.app.stroking());
    let _ = input::release(&mut p.app);
    assert!(!p.app.stroking());
    let t = TileRef::new(2, false, true, false);
    for c in 0..=3 {
        assert_eq!(p.tile_em(c, 0), t, "a célula {c} do arrasto");
    }
    assert_eq!(p.passos(), antes + 1, "um arrasto é um passo");
    assert!(derived_is_consistent(p.s()));
    assert!(p.app.undo(p.key));
    assert_eq!(p.s(), &mapa, "desfazer volta ao byte");
}

#[test]
fn a_borracha_em_draw_tiles_esvazia_e_o_esc_larga_o_arrasto() {
    let mut p = Palco::novo();
    p.pede(Pedido::Alvo(Alvo::Tiles));
    p.app.tool = Tool::Eraser;
    let antes = p.s().clone();
    p.prime(0, 2);
    assert_eq!(p.tile_em(0, 2), TileRef::EMPTY);
    assert!(p.tecla(KeyCode::Escape).is_some());
    assert_eq!(p.s(), &antes, "o Esc tira o passo");
}

#[test]
fn outra_ferramenta_em_draw_tiles_diz_porque() {
    let mut p = Palco::novo();
    p.pede(Pedido::Alvo(Alvo::Tiles));
    p.app.tool = Tool::Bucket;
    let antes = p.s().clone();
    p.prime(0, 0);
    let _ = input::release(&mut p.app);
    assert_eq!(p.s(), &antes);
    assert_eq!(p.avisos(), 1);
}

#[test]
fn o_alt_clique_pega_o_tile_com_os_flips() {
    let mut p = Palco::novo();
    p.pede(Pedido::Alvo(Alvo::Tiles));
    let pt = p.ponto(4, 0);
    let cv = canvas(&p.sim, &p.cam, p.bits);
    let m = Mods {
        alt: true,
        ..Mods::default()
    };
    input::press(&mut p.app, &cv, pt, Button::Primary, m);
    let _ = input::release(&mut p.app);
    p.quadro();
    assert_eq!(p.mao().tile_ref(), TileRef::new(3, true, false, false));
}

/// O lápis em Pixels: a mesma célula, nos três modos, com o resultado que o modo promete.
#[test]
fn o_lapis_em_pixels_obedece_ao_modo_pelo_gesto_real() {
    let traco = |modo: Modo| {
        let mut p = Palco::novo();
        p.pede(Pedido::Modo(modo));
        p.app.fg = [1, 2, 3, 255];
        assert_eq!(p.prime(0, 3), Press::Done);
        let _ = input::release(&mut p.app);
        assert!(derived_is_consistent(p.s()), "{modo:?}");
        p
    };
    // O tile 1 está em várias células: Manual muda-o em todas; Auto e Stack dão um tile novo.
    let m = traco(Modo::Manual);
    assert_eq!(m.n_tiles(), 4);
    assert_eq!(m.tile_em(0, 3).id(), 1);
    for modo in [Modo::Auto, Modo::Stack] {
        let a = traco(modo);
        assert_eq!(a.n_tiles(), 5, "{modo:?}");
        assert_eq!(a.tile_em(0, 3).id(), 4, "{modo:?}");
        assert_eq!(a.tile_em(1, 3).id(), 1, "a vizinha fica, {modo:?}");
    }
}

#[test]
fn space_tab_e_space_123_nao_partem_o_mover_da_vista() {
    let mut p = Palco::novo();
    p.espaco(true);
    assert_eq!(p.tecla(KeyCode::Tab), Some(KeyAnswer::Quiet));
    p.quadro();
    assert_eq!(p.mao().alvo, Alvo::Tiles);
    for (k, modo) in [
        (KeyCode::Digit1, Modo::Manual),
        (KeyCode::Digit3, Modo::Stack),
        (KeyCode::Digit2, Modo::Auto),
    ] {
        assert_eq!(p.tecla(k), Some(KeyAnswer::Quiet));
        p.quadro();
        assert_eq!(p.mao().modo, modo);
    }
    let antes = p.passos();
    assert_eq!(p.prime(0, 0), Press::Pan, "o Espaço continua a ser a mão");
    assert_eq!(p.passos(), antes);
    let _ = input::release(&mut p.app);
    p.espaco(false);
    assert_ne!(p.prime(0, 0), Press::Pan, "ao largar, volta a desenhar");
}

#[test]
fn os_atalhos_fora_de_um_tilemap_dizem_porque() {
    let mut p = Palco::novo();
    p.app.pede(PixelRequest::LayerAction(A::ConvertToImage));
    p.quadro();
    p.espaco(true);
    assert!(matches!(p.tecla(KeyCode::Tab), Some(KeyAnswer::Toast(_))));
    assert_eq!(p.hero.panel_visibility.get(PANEL_PIXEL_TILES), Some(&false));
}

#[test]
fn os_cartoes_escrevem_no_documento() {
    let mut p = Palco::novo();
    p.pede(Pedido::NovoTile);
    assert_eq!(p.n_tiles(), 5);
    assert_eq!(p.mao().tile, 4, "o tile novo fica na mão");
    assert!(p.app.undo(p.key));
    p.quadro();
    assert_eq!(p.n_tiles(), 4);
    assert_eq!(p.mao().tile, 3, "a mão não aponta para um tile que saiu");
    p.pede(Pedido::Tile(2));
    let antes = p.s().clone();
    p.pede(Pedido::ApagaTile);
    assert_eq!(p.n_tiles(), 3);
    assert_eq!(p.tile_em(0, 1), TileRef::EMPTY, "as células dele esvaziam");
    assert_eq!(p.tile_em(4, 0).id(), 2, "o de cima desce um número");
    assert_eq!(p.tile_em(4, 0).flip_x(), true, "com os flips");
    assert!(derived_is_consistent(p.s()));
    assert!(p.app.undo(p.key));
    assert_eq!(p.s(), &antes);
    p.pede(Pedido::Tile(0));
    p.pede(Pedido::ApagaTile);
    assert_eq!(p.n_tiles(), 4, "o vazio não se apaga");
    assert_eq!(p.avisos(), 1);
}

#[test]
fn a_camada_nova_e_o_convert_passam_pela_janela() {
    let mut p = Palco::novo();
    let camadas = p.s().layers().len();
    p.app.pede(PixelRequest::LayerAction(A::NewTilemap));
    p.quadro();
    assert_eq!(p.hero.panel_visibility.get(PANEL_PIXEL_TILES), Some(&true));
    assert_eq!(p.s().layers().len(), camadas, "só abre a janela");
    p.pede(Pedido::NovaCamada { w: 8, h: 12 });
    assert_eq!(p.s().layers().len(), camadas + 1);
    let d = p.app.doc(p.key).expect("doc");
    let set = tileset_de(&d.sprite, d.layer).expect("a camada nova fica activa");
    assert_eq!((set.tile_w, set.tile_h), (8, 12));
    assert!(
        d.sprite
            .layer(d.layer)
            .expect("camada")
            .name
            .starts_with("Tilemap")
    );

    p.app.pede(PixelRequest::LayerAction(A::ConvertToImage));
    p.quadro();
    let d = p.app.doc(p.key).expect("doc");
    assert!(tileset_de(&d.sprite, d.layer).is_none());
    assert_eq!(p.hero.panel_visibility.get(PANEL_PIXEL_TILES), Some(&false));
    p.app.pede(PixelRequest::LayerAction(A::ConvertToTilemap));
    p.quadro();
    assert_eq!(p.hero.panel_visibility.get(PANEL_PIXEL_TILES), Some(&true));
    p.pede(Pedido::Fechou);
    assert_eq!(p.hero.panel_visibility.get(PANEL_PIXEL_TILES), Some(&false));
    p.app.pede(PixelRequest::LayerAction(A::ConvertToTilemap));
    p.quadro();
    p.pede(Pedido::Converte {
        w: 16,
        h: 16,
        flips: false,
    });
    let d = p.app.doc(p.key).expect("doc");
    assert!(tileset_de(&d.sprite, d.layer).is_some());
}

/// Os comandos que eram mudos numa camada de tilemap dizem porquê, e o documento fica igual.
#[test]
fn nenhum_comando_do_menu_sprite_recusa_calado() {
    for a in [
        Sp::RotateCanvas180,
        Sp::RotateCanvas90Cw,
        Sp::RotateCanvas90Ccw,
        Sp::FlipCanvasH,
        Sp::FlipCanvasV,
        Sp::SpriteSize,
    ] {
        let mut p = Palco::novo();
        let antes = p.s().clone();
        p.app.pede(PixelRequest::Sprite(a));
        assert_eq!(p.avisos(), 1, "{a:?}");
        assert_eq!(p.s(), &antes, "{a:?}");
    }
}
