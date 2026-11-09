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
use ph2d_pixel_core::{Forma, Sprite, TERRAIN_TILES, TileRef, derived_is_consistent};
use ph2d_pixel_tiles::desenho::Modo;
use ph2d_pixel_tiles::exemplo::{LADO, MAPA, celulas_ilhas};
use ph2d_pixel_tiles::mao::tileset_de;
use ph2d_pixel_tiles::mapa::Mapa;
use ph2d_pixel_tiles::terreno;
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
        Self::cena(21)
    }

    fn cena(n: u32) -> Self {
        let _ = canal::take_pedidos();
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(ph2d_app_pixel::smoke::stage(
            &mut app, &mut sim, n, spawn_root
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

    /// Um quadro do canvas: a imagem composta como o artista a vê.
    fn desenha(&mut self) {
        let mut scene = ph2d_vector::VectorScene::new();
        ph2d_app_pixel::draw::art(
            &mut self.app,
            self.sim.world(),
            ph2d_vector::Affine::IDENTITY,
            PPM,
            Some(self.bits),
            &[self.bits],
            true,
            &mut scene,
        );
        self.quadro();
    }

    /// A cor no canvas do píxel `(3, 3)` da célula `(c, r)`.
    fn vista(&self, c: i32, r: i32) -> [u8; 4] {
        let l = i32::from(LADO);
        self.app
            .canvas_rgba(self.key, (c * l + 3) as u32, (r * l + 3) as u32)
            .expect("o canvas já foi composto")
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

    /// O clique no píxel do meio da célula, pela forma da grelha.
    fn meio(&self, c: i32, r: i32) -> (f32, f32) {
        let d = self.app.doc(self.key).expect("doc");
        let set = tileset_de(&d.sprite, d.layer).expect("tileset");
        let (x, y) = Mapa::de(&d.sprite, d.layer, d.frame, set).origem(c, r);
        let (mx, my) = (i32::from(set.tile_w) / 2, i32::from(set.tile_h) / 2);
        at(&self.app, &self.sim, &self.cam, self.bits, x + mx, y + my)
    }

    fn prime_meio(&mut self, c: i32, r: i32, b: Button, m: Mods) -> Press {
        let p = self.meio(c, r);
        let cv = canvas(&self.sim, &self.cam, self.bits);
        input::press(&mut self.app, &cv, p, b, m)
    }

    fn move_meio(&mut self, c: i32, r: i32) {
        let p = self.meio(c, r);
        let cv = canvas(&self.sim, &self.cam, self.bits);
        input::moved(&mut self.app, &cv, p, Mods::default());
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
    assert!(
        canal::contexto().com_sprite,
        "com o sprite em Draw a janela não manda abrir um"
    );
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

/// O dono, 06/11: «as mudanças nos tiles acontecem com um delay de um traço». O canvas é a imagem
/// composta, não o documento: é ela que se confere.
#[test]
fn o_traco_em_manual_aparece_em_todas_as_copias_durante_e_depois() {
    let mut p = Palco::novo();
    p.pede(Pedido::Modo(Modo::Manual));
    p.app.fg = [1, 2, 3, 255];
    p.desenha();
    let (fica, copia) = ((0, 3), (5, 3));
    let (l, antes) = (usize::from(LADO), p.vista(copia.0, copia.1));
    assert_eq!(p.vista(fica.0, fica.1), antes, "a mesma relva");
    let referencia = p.s().clone();

    assert_eq!(p.prime(fica.0, fica.1), Press::Done);
    p.desenha();
    assert_eq!(p.vista(fica.0, fica.1), [1, 2, 3, 255]);
    assert_eq!(p.vista(copia.0, copia.1), [1, 2, 3, 255], "a meio do traço");
    assert_eq!(p.s(), &referencia, "o documento só muda ao largar");
    p.quadro();
    let img = &canal::contexto().imagens[1];
    let i = (3 * l + 3) * 4;
    assert_eq!(img[i..i + 4], [1, 2, 3, 255], "o cartão também acompanha");
    p.move_para(1, 3);
    p.quadro();
    let img = &canal::contexto().imagens[1];
    assert_eq!(
        img[i..i + 4],
        [1, 2, 3, 255],
        "o cartão não volta atrás entre um quadro e outro do traço"
    );

    let _ = input::release(&mut p.app);
    p.desenha();
    assert_eq!(
        p.vista(copia.0, copia.1),
        [1, 2, 3, 255],
        "depois de largar"
    );
    assert!(p.app.undo(p.key));
    p.desenha();
    assert_eq!(p.vista(copia.0, copia.1), antes, "o desfazer devolve");
}

/// Manual numa célula vazia não muda nada: o canvas não pode mostrar uma tinta que não vai ficar.
#[test]
fn a_tinta_que_nao_vai_ficar_nao_se_mostra() {
    let mut p = Palco::novo();
    p.pede(Pedido::Modo(Modo::Manual));
    p.app.fg = [1, 2, 3, 255];
    p.desenha();
    let antes = p.vista(0, 0);
    assert_eq!(p.prime(0, 0), Press::Done);
    p.desenha();
    assert_eq!(p.vista(0, 0), antes);
    let _ = input::release(&mut p.app);
    p.desenha();
    assert_eq!(p.vista(0, 0), antes);
}

/// O dono, 06/11: «New Tilemap Layer apagou os tiles já existentes». A camada nova traz um tileset
/// só dela; a antiga fica como estava, no canvas e no cartão ao voltar a ela.
#[test]
fn a_camada_nova_nao_mexe_na_antiga_e_o_cartao_segue_a_camada_activa() {
    let mut p = Palco::novo();
    p.desenha();
    let velha = p.app.doc(p.key).expect("doc").layer;
    let (arte, n) = (p.vista(0, 3), canal::contexto().imagens.len());
    assert_eq!(n, 4);
    p.app.pede(PixelRequest::LayerAction(A::NewTilemap));
    p.quadro();
    p.pede(Pedido::NovaCamada {
        conjunto: None,
        w: 16,
        h: 16,
        forma: Forma::Square,
    });
    p.desenha();
    assert_eq!(p.vista(0, 3), arte, "o desenho continua no canvas");
    assert_eq!(
        canal::contexto().imagens.len(),
        1,
        "a camada nova está vazia"
    );

    p.app.pede(PixelRequest::LayerSelect(velha.0));
    p.quadro();
    assert_eq!(
        canal::contexto().imagens.len(),
        4,
        "os tiles voltam ao cartão"
    );
}

/// O dono, 06/11: «os tiles devem estar disponíveis para qualquer camada desde sempre». Duas camadas
/// sobre o mesmo tileset: uma mão só.
fn duas_camadas() -> (Palco, ph2d_pixel_core::LayerId, ph2d_pixel_core::LayerId) {
    let mut p = Palco::novo();
    let velha = p.app.doc(p.key).expect("doc").layer;
    let conjunto = canal::contexto().padrao;
    assert!(conjunto.is_some(), "a janela traz o tileset da camada");
    p.app.pede(PixelRequest::LayerAction(A::NewTilemap));
    p.quadro();
    p.pede(Pedido::NovaCamada {
        conjunto,
        w: 16,
        h: 16,
        forma: Forma::Square,
    });
    let nova = p.app.doc(p.key).expect("doc").layer;
    assert_ne!(nova, velha);
    (p, velha, nova)
}

#[test]
fn a_camada_nova_desenha_com_os_tiles_que_ja_existem() {
    let (mut p, velha, nova) = duas_camadas();
    let s = p.s();
    assert_eq!(
        s.layer(nova).unwrap().tileset(),
        s.layer(velha).unwrap().tileset()
    );
    assert_eq!(
        canal::contexto().imagens.len(),
        4,
        "o cartão traz os mesmos tiles"
    );
    assert_eq!(
        p.tile_em(0, 0),
        TileRef::EMPTY,
        "a camada nasce sem nada desenhado"
    );
    let n = p.n_tiles();
    p.pede(Pedido::Alvo(Alvo::Tiles));
    p.pede(Pedido::Tile(2));
    assert_eq!(p.prime(0, 0), Press::Done);
    let _ = input::release(&mut p.app);
    assert_eq!(p.tile_em(0, 0).id(), 2);
    assert_eq!(p.n_tiles(), n, "carimbar não acrescenta tiles");
    assert!(derived_is_consistent(p.s()));
    let passos = p.passos();
    assert!(p.app.undo(p.key));
    assert!(p.app.undo(p.key));
    assert!(p.s().layer(nova).is_none(), "um passo tira a camada");
    assert!(p.s().layer(velha).is_some());
    assert_eq!(p.passos() + 2, passos);
    assert!(derived_is_consistent(p.s()));
}

#[test]
fn editar_um_tile_repartido_muda_todas_as_camadas() {
    let (mut p, _, nova) = duas_camadas();
    p.pede(Pedido::Alvo(Alvo::Tiles));
    p.pede(Pedido::Tile(1));
    assert_eq!(p.prime(0, 0), Press::Done);
    let _ = input::release(&mut p.app);
    p.pede(Pedido::Alvo(Alvo::Pixels));
    p.pede(Pedido::Modo(Modo::Manual));
    p.app.fg = [1, 2, 3, 255];
    p.desenha();
    assert_eq!(p.prime(0, 0), Press::Done);
    p.desenha();
    assert_eq!(
        p.vista(5, 3),
        [1, 2, 3, 255],
        "a relva da outra camada, a meio do traço"
    );
    let _ = input::release(&mut p.app);
    p.desenha();
    assert_eq!(p.vista(5, 3), [1, 2, 3, 255], "e depois de largar");
    assert_eq!(p.app.doc(p.key).unwrap().layer, nova);
    assert!(derived_is_consistent(p.s()));
}

#[test]
fn os_modos_e_apagar_um_tile_contam_todas_as_camadas() {
    let (mut p, velha, _) = duas_camadas();
    // O tile 3 está só na linha de cima da camada antiga; aqui fica a seta usada por mais uma camada.
    p.pede(Pedido::Alvo(Alvo::Tiles));
    p.pede(Pedido::Tile(3));
    assert_eq!(p.prime(2, 2), Press::Done);
    let _ = input::release(&mut p.app);
    p.pede(Pedido::Alvo(Alvo::Pixels));
    p.pede(Pedido::Modo(Modo::Auto));
    p.app.fg = [1, 2, 3, 255];
    let n = p.n_tiles();
    assert_eq!(p.prime(2, 2), Press::Done);
    let _ = input::release(&mut p.app);
    assert_eq!(
        p.n_tiles(),
        n + 1,
        "repartido com a outra camada: o Auto faz um tile novo"
    );
    assert!(p.app.undo(p.key));

    p.pede(Pedido::Tile(1));
    p.pede(Pedido::ApagaTile);
    assert!(derived_is_consistent(p.s()));
    assert_eq!(
        p.tile_em(2, 2).id(),
        2,
        "a seta desce um número nesta camada"
    );
    p.app.pede(PixelRequest::LayerSelect(velha.0));
    p.quadro();
    assert_eq!(
        p.tile_em(0, 3),
        TileRef::EMPTY,
        "a relva sai da outra camada"
    );
    assert_eq!(p.tile_em(2, 0).id(), 2, "e a seta desce lá também");
}

#[test]
fn a_camada_repartida_sobrevive_a_gravar_e_a_mover_o_cel() {
    let (mut p, velha, nova) = duas_camadas();
    p.pede(Pedido::Alvo(Alvo::Tiles));
    p.pede(Pedido::Tile(2));
    assert_eq!(p.prime(0, 0), Press::Done);
    let _ = input::release(&mut p.app);
    let s = p.s().clone();
    let lib = std::collections::BTreeMap::from([(1_u64, s.clone())]);
    let bytes = ph2d_pixel_core::codec::write_library(&lib);
    let lido = ph2d_pixel_core::codec::read_library(&bytes).expect("lê");
    let lido = &lido[&1];
    assert_eq!(lido.tilesets().len(), 1, "o mesmo tileset, gravado uma vez");
    assert_eq!(
        lido.layer(nova).unwrap().tileset(),
        lido.layer(velha).unwrap().tileset()
    );
    assert!(derived_is_consistent(lido));

    let (ase, _) = ph2d_pixel_io::ase::escrever(&s).expect("grava o .ase");
    let (lido, _) = ph2d_pixel_io::ase::ler(&ase).expect("lê o .ase");
    assert_eq!(lido.tilesets().len(), 1, "no .ase também é um só");
    let com_mapa: Vec<_> = lido.layers().iter().filter_map(|l| l.tileset()).collect();
    assert_eq!(com_mapa.len(), 2);
    assert_eq!(com_mapa[0], com_mapa[1]);
    assert!(derived_is_consistent(&lido));

    let f = s.frames()[0].id;
    let mut m = s.clone();
    let movido =
        ph2d_pixel_core::frames::move_cels_grow(&mut m, &[((nova, f), (velha, f))], &[], true);
    assert!(
        movido.is_some(),
        "mover entre camadas do mesmo tileset é permitido"
    );
}

/// Converter sobre o tileset que já existe: o pedaço igual usa o tile, o composto fica igual.
#[test]
fn converter_reaproveita_os_tiles_e_so_acrescenta_os_novos() {
    let converte = |flips: bool| {
        let mut p = Palco::novo();
        p.desenha();
        let conjunto = canal::contexto().padrao;
        let (arte, seta) = (p.vista(0, 3), p.vista(4, 0));
        p.app.pede(PixelRequest::LayerAction(A::ConvertToImage));
        p.quadro();
        assert_eq!(
            canal::contexto().padrao,
            conjunto,
            "o tileset fica oferecido"
        );
        p.app.pede(PixelRequest::LayerAction(A::ConvertToTilemap));
        p.quadro();
        p.pede(Pedido::Converte {
            conjunto,
            w: 16,
            h: 16,
            flips,
        });
        p.desenha();
        assert_eq!((p.vista(0, 3), p.vista(4, 0)), (arte, seta));
        assert!(derived_is_consistent(p.s()));
        p
    };
    let p = converte(true);
    assert_eq!(p.n_tiles(), 4, "com flips, a seta virada usa a seta");
    assert_eq!(p.tile_em(0, 3).id(), 1);
    assert!(p.tile_em(4, 0).flip_x());
    let mut p = converte(false);
    assert_eq!(p.n_tiles(), 5, "sem flips, a seta virada é um tile novo");
    assert_eq!(p.tile_em(2, 0).id(), 3);
    assert_eq!(p.tile_em(4, 0).id(), 4);
    let antes = p.n_tiles();
    assert!(p.app.undo(p.key));
    assert_eq!(
        p.s().tilesets()[0].tiles.len(),
        antes - 1,
        "o desfazer repõe o tileset"
    );
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
    assert!(p.tile_em(4, 0).flip_x(), "com os flips");
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
    p.pede(Pedido::NovaCamada {
        conjunto: None,
        w: 8,
        h: 12,
        forma: Forma::Square,
    });
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
        conjunto: None,
        w: 16,
        h: 16,
        flips: false,
    });
    let d = p.app.doc(p.key).expect("doc");
    assert!(tileset_de(&d.sprite, d.layer).is_some());
}

/// Os comandos que eram mudos numa camada de tilemap dizem porquê, e o documento fica igual.
/// Fora do múltiplo do tile, o canvas anda como nos oráculos (M9), e o desenho vai com ele.
#[test]
fn o_canvas_fora_do_multiplo_nao_recusa() {
    let mut p = Palco::novo();
    p.desenha();
    let (w, arte) = (p.s().width(), p.vista(0, 3));
    p.app.aplica_dialogo(
        ph2d_editor_core::pixel_panel::PixelDialogChoice::CanvasSize {
            left: 16,
            top: 0,
            right: 1,
            bottom: 0,
        },
    );
    assert_eq!(p.s().width(), w + 17);
    assert_eq!(p.avisos(), 0);
    p.desenha();
    assert_eq!(p.vista(1, 3), arte);
    assert!(p.app.undo(p.key));

    p.app.aplica_dialogo(
        ph2d_editor_core::pixel_panel::PixelDialogChoice::CanvasSize {
            left: 1,
            top: 0,
            right: 0,
            bottom: 0,
        },
    );
    assert_eq!(p.s().width(), w + 1);
    assert!(derived_is_consistent(p.s()));
    p.app.pede(PixelRequest::Sprite(Sp::Trim));
    assert_eq!(p.avisos(), 0, "o Trim corta sem recusar");
    assert_eq!(p.s().width(), w, "a coluna vazia sai");
    assert!(derived_is_consistent(p.s()));
}

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

/// A casa de terreno da célula, ou `None` se ela não é de terreno.
fn casa_em(p: &Palco, c: i32, r: i32) -> Option<u32> {
    let d = p.app.doc(p.key).expect("doc");
    let set = tileset_de(&d.sprite, d.layer).expect("tileset");
    let id = p.tile_em(c, r).id();
    let k = set.terrains.first()?;
    (id >= k.first && id < k.first + TERRAIN_TILES).then(|| id - k.first)
}

#[test]
fn a_cena_22_abre_com_as_ilhas_e_o_pincel_de_terreno() {
    let p = Palco::cena(22);
    let x = canal::contexto();
    assert_eq!(p.mao().alvo, Alvo::Terreno);
    assert_eq!(x.terrenos, ["Terrain 1"]);
    assert_eq!(x.forma, Forma::Square);
    assert!(p.hero.panel_visibility.get(PANEL_PIXEL_TILES).copied() == Some(true));
    assert!(derived_is_consistent(p.s()));
    for (c, r) in celulas_ilhas() {
        assert!(casa_em(&p, c, r).is_some(), "({c},{r}) sem terreno");
    }
    assert_eq!(casa_em(&p, 0, 0), None);
}

/// O gesto real: arrastar o lápis em Terrain resolve as células e as vizinhas, num só desfazer; o botão
/// de trás e a borracha tiram a área.
#[test]
fn pintar_terreno_arrastando_resolve_as_vizinhas_e_vale_um_passo() {
    let mut p = Palco::cena(22);
    let (antes, s0) = (p.passos(), p.s().clone());
    assert_eq!(casa_em(&p, 1, 7), None);
    assert_eq!(
        p.prime_meio(1, 7, Button::Primary, Mods::default()),
        Press::Done
    );
    p.move_meio(1, 9);
    let _ = input::release(&mut p.app);
    assert_eq!(p.passos(), antes + 1, "um arrasto é um passo");
    for r in 7..=9 {
        assert!(casa_em(&p, 1, r).is_some(), "a célula (1,{r}) do arrasto");
    }
    // Só sul: a primeira célula do corredor tem o braço de baixo.
    let k = terreno::casa(terreno::S);
    assert_eq!(casa_em(&p, 1, 7), Some(k));
    assert!(derived_is_consistent(p.s()));
    assert_eq!(
        p.prime_meio(1, 8, Button::Secondary, Mods::default()),
        Press::Done
    );
    let _ = input::release(&mut p.app);
    assert_eq!(casa_em(&p, 1, 8), None);
    assert_eq!(
        casa_em(&p, 1, 7),
        Some(terreno::casa(0)),
        "a vizinha resolve-se de novo"
    );
    p.app.tool = Tool::Eraser;
    p.prime_meio(1, 9, Button::Primary, Mods::default());
    let _ = input::release(&mut p.app);
    assert_eq!(casa_em(&p, 1, 9), None);
    assert!(p.app.undo(p.key) && p.app.undo(p.key) && p.app.undo(p.key));
    assert_eq!(p.s(), &s0, "desfazer volta ao byte");
}

#[test]
fn novo_terreno_nasce_com_o_guia_e_o_gabarito_abre_numa_camada() {
    let mut p = Palco::novo();
    assert_eq!(canal::contexto().terrenos.len(), 0);
    p.pede(Pedido::Alvo(Alvo::Terreno));
    p.app.tool = Tool::Pencil;
    assert_eq!(
        p.prime_meio(0, 0, Button::Primary, Mods::default()),
        Press::Done
    );
    assert_eq!(p.avisos(), 1, "sem terreno, diz porquê");
    let (n0, passos) = (p.n_tiles(), p.passos());
    p.pede(Pedido::NovoTerreno);
    assert_eq!(canal::contexto().terrenos, ["Terrain 1"]);
    assert_eq!(p.n_tiles(), n0 + TERRAIN_TILES as usize);
    assert_eq!(p.passos(), passos + 1);
    let img = canal::contexto().imagens;
    assert!(
        img[n0 + 46].iter().any(|b| *b != 0),
        "o miolo tem o guia pintado"
    );
    let camada = p.app.doc(p.key).unwrap().layer;
    let w = p.s().width();
    p.pede(Pedido::Gabarito);
    let d = p.app.doc(p.key).unwrap();
    assert_ne!(d.layer, camada, "a camada nova fica activa");
    assert_eq!(p.passos(), passos + 2);
    assert!(
        p.s().width() >= 8 * u32::from(LADO).max(w / 8),
        "o canvas cresceu para caber"
    );
    assert!(casa_em(&p, 7, 5).is_some() || p.tile_em(6, 5).id() != 0);
    assert!(
        p.app.undo(p.key),
        "um passo tira a camada e devolve o canvas"
    );
    assert_eq!(p.s().width(), w);
    assert_eq!(
        p.app.doc(p.key).unwrap().layer,
        camada,
        "volta à camada de tilemap"
    );
    p.pede(Pedido::ApagaTerreno);
    assert_eq!(p.n_tiles(), n0);
    assert_eq!(canal::contexto().terrenos.len(), 0);
}

#[test]
fn o_conta_gotas_em_terrain_fica_com_o_terreno_da_celula() {
    let mut p = Palco::cena(22);
    p.pede(Pedido::NovoTerreno);
    assert_eq!(p.mao().terreno, 1, "o terreno novo fica na mão");
    let alt = Mods {
        alt: true,
        ..Mods::default()
    };
    let (c, r) = celulas_ilhas()[0];
    let antes = p.s().clone();
    assert_eq!(p.prime_meio(c, r, Button::Primary, alt), Press::Done);
    let _ = input::release(&mut p.app);
    p.quadro();
    assert_eq!(p.mao().terreno, 0);
    assert_eq!(p.s(), &antes, "o conta-gotas não escreve");
}

/// Mudar a forma pelo cartão: em iso o clique acerta a célula do losango; em hex um tileset com terreno
/// recusa, com a frase, e a mão não fica num alvo que a forma não tem.
#[test]
fn a_forma_da_grelha_muda_o_clique_e_o_terreno() {
    let mut p = Palco::novo();
    let (s0, passos) = (p.s().clone(), p.passos());
    p.pede(Pedido::Forma(Forma::Iso));
    assert_eq!(canal::contexto().forma, Forma::Iso);
    assert_eq!(p.passos(), passos + 1);
    assert!(derived_is_consistent(p.s()));
    p.pede(Pedido::Alvo(Alvo::Tiles));
    p.pede(Pedido::Tile(2));
    p.prime_meio(8, 2, Button::Primary, Mods::default());
    let _ = input::release(&mut p.app);
    assert_eq!(p.tile_em(8, 2).id(), 2, "o clique acerta o losango");
    assert!(p.app.undo(p.key) && p.app.undo(p.key));
    assert_eq!(p.s(), &s0, "desfazer volta à grelha quadrada, ao byte");
    p.quadro();
    p.pede(Pedido::Forma(Forma::Hex));
    p.pede(Pedido::Alvo(Alvo::Terreno));
    assert_eq!(
        p.mao().alvo,
        Alvo::Tiles,
        "a mão não fica num alvo que a forma não tem"
    );
    let n = p.n_tiles();
    p.pede(Pedido::NovoTerreno);
    assert_eq!(p.n_tiles(), n, "hex não tem gabarito");
    assert_eq!(p.avisos(), 1);
    assert!(p.app.undo(p.key));
    p.quadro();
    p.pede(Pedido::NovoTerreno);
    p.pede(Pedido::Forma(Forma::Hex));
    assert_eq!(
        canal::contexto().forma,
        Forma::Square,
        "com terreno, hex recusa"
    );
    assert_eq!(p.avisos(), 1);
}

/// A janela New Tilemap Layer cria a camada na forma pedida, com um tileset novo.
#[test]
fn a_camada_nova_nasce_na_forma_pedida() {
    let mut p = Palco::novo();
    let conjuntos = p.s().tilesets().len();
    p.pede(Pedido::NovaCamada {
        conjunto: None,
        w: 32,
        h: 16,
        forma: Forma::Iso,
    });
    assert_eq!(p.s().tilesets().len(), conjuntos + 1);
    assert_eq!(canal::contexto().forma, Forma::Iso);
    assert_eq!(p.mao().alvo, Alvo::Pixels);
}
