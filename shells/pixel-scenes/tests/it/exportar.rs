//! **File ▸ Export e Import até ao disco e ao documento** (P9b), sobre a cena 20: o comando do menu ou
//! o atalho → o quadro da família → a janela e o seu verbo → bytes no disco, ou um objecto novo. O
//! diálogo nativo responde por `responde_o_proximo`; a janela em si tem os testes dela.

use std::path::PathBuf;

use ph2d_app_host::modal::responde_o_proximo;
use ph2d_app_pixel::{PixelApp, input, mode, object_add, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent};
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{PixelFileAction as A, PixelRequest as R};
use ph2d_editor_core::screens::hero::{HeroScreen, PANEL_PIXEL_EXPORT};
use ph2d_editor_core::{ToastQueue, ids};
use ph2d_panel_pixel_export::canal::{self, Pedido};
use ph2d_pixel_io::exporta::{Export, ExportAs, ExportFolha};
use ph2d_pixel_io::render;
use ph2d_pixel_io::saida::Formato;
use ph2d_pixel_io::sheet::{OpcoesFolha, Tipo};
use ph2d_render::Camera2d;
use winit::keyboard::KeyCode;

use crate::comum::{PPM, WINDOW, spawn_root};

struct Palco {
    app: PixelApp,
    sim: SimWorld,
    hero: HeroScreen,
    toasts: ToastQueue,
    bits: u64,
    key: u32,
    pasta: PathBuf,
}

impl Palco {
    fn novo(nome: &str) -> Self {
        ph2d_editor_core::test_support::ensure_panel_registry();
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(ph2d_app_pixel::smoke::stage(
            &mut app, &mut sim, 20, spawn_root
        ));
        let (bits, key) = mode::sprites(sim.world())[0];
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        hero.gizmo.mode.enter(bits, ObjectMode::Draw);
        let mut cam = Camera2d::default();
        app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
        let pasta = std::env::temp_dir().join(format!("ph2d_cena20_{nome}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&pasta);
        std::fs::create_dir_all(&pasta).expect("pasta");
        let mut p = Self {
            app,
            sim,
            hero,
            toasts: ToastQueue::new(),
            bits,
            key,
            pasta,
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

    /// Os quadros até o trabalho de fora da tela acabar.
    fn espera(&mut self) {
        for _ in 0..2000 {
            self.quadro();
            if !self.app.exportando() {
                self.quadro();
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("o export não acabou");
    }

    fn aberta(&self) -> bool {
        self.hero.panel_visibility.get(PANEL_PIXEL_EXPORT).copied() == Some(true)
    }

    fn menu_visivel(&self, id: ph2d_editor_core::NodeId) -> bool {
        !self
            .hero
            .store
            .menu_row_is_hidden(ContextMenuKind::MenuBarFile, id)
    }

    fn tecla(&mut self, code: KeyCode, ctrl: bool, shift: bool, alt: bool) {
        let m = ph2d_app_pixel::compose::Mods { alt, ctrl, shift };
        let a = input::key_with(&mut self.app, self.sim.world(), Some(self.bits), code, m);
        assert!(a.is_some(), "{code:?} é da família");
        self.quadro();
    }
}

fn gif() -> Export {
    Export::Ficheiro(ExportAs {
        pedido: render::Pedido {
            escala: 1.0,
            ..render::Pedido::default()
        },
        formato: Formato::Gif,
        repeat: 0,
    })
}

/// As linhas só existem em Draw; o Repeat fica apagado até haver o que repetir.
#[test]
fn o_menu_file_so_mostra_as_linhas_em_draw() {
    let mut p = Palco::novo("menu");
    let linhas = [
        ids::CTX_MENU_PIXEL_OPEN,
        ids::CTX_MENU_PIXEL_IMPORT_SHEET,
        ids::CTX_MENU_PIXEL_EXPORT_AS,
        ids::CTX_MENU_PIXEL_EXPORT_SHEET,
        ids::CTX_MENU_PIXEL_REPEAT_EXPORT,
    ];
    assert!(linhas.iter().all(|&l| p.menu_visivel(l)), "em Draw");
    assert!(
        p.hero
            .store
            .menu_row_is_unavailable(ids::CTX_MENU_PIXEL_REPEAT_EXPORT)
    );
    assert!(
        p.hero
            .apply_event(WidgetEvent::Click(ids::CTX_MENU_PIXEL_REPEAT_EXPORT))
    );
    assert_eq!(
        p.hero.store.take_pixel_file_request(),
        None,
        "apagada não age"
    );
    let _ = p.hero.gizmo.mode.leave();
    p.quadro();
    assert!(linhas.iter().all(|&l| !p.menu_visivel(l)), "fora do Draw");
    assert!(p.menu_visivel(ids::CTX_MENU_EXPORT_SVG), "as outras ficam");
}

/// Cada linha do menu abre a sua janela pelo clique real.
#[test]
fn o_clique_no_menu_abre_a_janela() {
    let mut p = Palco::novo("clique");
    assert!(!p.aberta());
    assert!(
        p.hero
            .apply_event(WidgetEvent::Click(ids::CTX_MENU_PIXEL_EXPORT_AS))
    );
    p.quadro();
    assert!(p.aberta(), "Export As");
    p.hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, false);
    assert!(
        p.hero
            .apply_event(WidgetEvent::Click(ids::CTX_MENU_PIXEL_EXPORT_SHEET))
    );
    p.quadro();
    assert!(p.aberta(), "Export Sprite Sheet");
}

/// Os atalhos do Aseprite: `Ctrl+E`, `Ctrl+Alt+Shift+S`, `Ctrl+Shift+X`; `Ctrl+I` pede o ficheiro.
/// Os vizinhos (`Ctrl+Alt+I` Sprite Size, `Ctrl+Shift+I` Inverse) não abrem nada aqui.
#[test]
fn os_atalhos_do_aseprite() {
    let mut p = Palco::novo("atalhos");
    p.tecla(KeyCode::KeyE, true, false, false);
    assert!(p.aberta(), "Ctrl+E");
    p.hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, false);
    p.tecla(KeyCode::KeyS, true, true, true);
    assert!(p.aberta(), "Ctrl+Alt+Shift+S");
    p.hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, false);
    p.tecla(KeyCode::KeyI, true, true, false);
    assert!(!p.aberta(), "Ctrl+Shift+I é o Inverse");
    p.tecla(KeyCode::KeyI, true, false, true);
    assert!(!p.aberta(), "Ctrl+Alt+I é o Sprite Size");
    let folha = p.pasta.join("folha.png");
    write_folha(&folha);
    responde_o_proximo(folha);
    p.tecla(KeyCode::KeyI, true, false, false);
    assert!(p.aberta(), "Ctrl+I");
}

/// Uma folha 3 × 2 de células 4 × 4, cada célula com o seu tom (a grelha vê-se nela).
fn write_folha(c: &std::path::Path) {
    let (w, h) = (12_u32, 8_u32);
    let mut px = vec![0_u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            // Um furo em cada célula deixa as linhas e as colunas vazias que a sugestão lê.
            if x % 4 == 3 || y % 4 == 3 {
                continue;
            }
            let i = ((y * w + x) * 4) as usize;
            let tom = (x / 4 + 3 * (y / 4)) as u8 * 40 + 20;
            px[i..i + 4].copy_from_slice(&[tom, 200, 100, 255]);
        }
    }
    let b =
        ph2d_imageio_png::encode_pixel(w, h, &ph2d_imageio_png::PixelPng::Rgba, &px).expect("png");
    std::fs::write(c, b).expect("grava");
}

/// **O Export e o Repeat, até aos bytes.** O caminho e as opções ficam no sprite, e o Repeat regrava
/// sem perguntar nada. *Mutação: não `lembrar` ⇒ o Repeat fica apagado e o 2.º bloco reprova.*
#[test]
fn export_as_grava_e_o_repeat_regrava_sem_perguntar() {
    let mut p = Palco::novo("repeat");
    p.app.pede(R::File(A::ExportAs));
    p.quadro();
    assert!(p.aberta());
    let c = p.pasta.join("bola.gif");
    responde_o_proximo(c.clone());
    canal::pede(Pedido::Exportar(gif()));
    p.quadro();
    assert!(!p.aberta(), "o Export fecha a janela");
    p.espera();
    let b = std::fs::read(&c).expect("o ficheiro saiu");
    assert_eq!(&b[..6], b"GIF89a");
    assert!(
        !p.hero
            .store
            .menu_row_is_unavailable(ids::CTX_MENU_PIXEL_REPEAT_EXPORT),
        "o Repeat acende"
    );
    std::fs::remove_file(&c).expect("apaga");
    p.tecla(KeyCode::KeyX, true, true, false);
    p.espera();
    assert_eq!(std::fs::read(&c).expect("voltou"), b, "os mesmos bytes");
}

/// Um verbo que não se pode cumprir (o Import sem imagem) deixa a janela aberta e não cria nada.
#[test]
fn verbo_sem_o_que_cumprir_deixa_a_janela_aberta() {
    let mut p = Palco::novo("sem_imagem");
    p.app.pede(R::File(A::ExportAs));
    p.quadro();
    canal::pede(Pedido::Importar(ph2d_pixel_io::abrir::Grelha {
        x: 0,
        y: 0,
        w: 4,
        h: 4,
        gap_x: 0,
        gap_y: 0,
        por_colunas: false,
    }));
    p.quadro();
    assert!(p.aberta());
    assert_eq!(p.app.docs().count(), 1);
}

/// A folha sai com o `.json` ao lado, e o que está na tela é o que sai.
#[test]
fn a_folha_ao_vivo_e_a_folha_do_disco_sao_a_mesma() {
    let mut p = Palco::novo("folha");
    p.app.pede(R::File(A::ExportSheet));
    p.quadro();
    let o = OpcoesFolha {
        tipo: Tipo::Horizontal,
        border: 2,
        ..OpcoesFolha::default()
    };
    canal::pede_folha(Some(o.clone()));
    p.espera();
    let i = canal::imagem().expect("a prévia");
    let s = p.app.doc(p.key).expect("doc").sprite.clone();
    let esperada = ph2d_pixel_io::sheet::montar(&s, &o, "", "png");
    assert_eq!((i.w, i.h), (esperada.w, esperada.h));
    assert_eq!(*i.rgba, esperada.rgba);
    assert_eq!(i.entradas, 16);
    // Mudar a opção remonta, sem fechar a janela.
    let o2 = OpcoesFolha {
        border: 5,
        ..o.clone()
    };
    canal::pede_folha(Some(o2));
    p.espera();
    assert_eq!(canal::imagem().expect("a prévia").h, esperada.h + 6);
    let seq = canal::imagem().expect("a prévia").seq;
    p.quadro();
    let agora = canal::imagem().expect("a prévia").seq;
    assert_eq!(agora, seq, "nada mudou: nada remonta");
    // Desenhar com a janela aberta remonta: a chave leva o carimbo do documento.
    let d = p.app.doc(p.key).expect("doc");
    let (layer, frame) = (d.layer, d.frame);
    let r = ph2d_pixel_core::Rect::new(0, 0, 1, 1);
    let patch = ph2d_pixel_core::Patch {
        layer,
        frame,
        bbox: Some(r),
        rect: r,
        pixels: vec![9, 9, 9, 255],
    };
    p.app
        .commit(p.key, ph2d_pixel_core::Op::Patch(patch))
        .expect("pinta");
    canal::pede_folha(Some(OpcoesFolha {
        border: 5,
        tipo: Tipo::Horizontal,
        ..OpcoesFolha::default()
    }));
    p.espera();
    assert_ne!(
        canal::imagem().expect("a prévia").seq,
        seq,
        "o desenho muda a folha"
    );

    let c = p.pasta.join("folha.png");
    responde_o_proximo(c.clone());
    canal::pede(Pedido::Exportar(Export::Folha(ExportFolha {
        opcoes: o,
        json: Some(ph2d_pixel_io::sheet::OpcoesJson::default()),
    })));
    p.quadro();
    p.espera();
    let (w, h, _, _) =
        ph2d_imageio_png::decode_pixel(&std::fs::read(&c).expect("png")).expect("lê");
    assert_eq!((w, h), (esperada.w, esperada.h));
    assert!(p.pasta.join("folha.json").exists());
}

/// **Abrir cria um Pixel Sprite NOVO**, com o que o `.ase` trazia, e pede o Draw dele; o antigo fica.
#[test]
fn open_de_um_ase_nasce_um_sprite_novo() {
    let mut p = Palco::novo("open");
    let antes = p.app.doc(p.key).expect("doc").sprite.clone();
    let c = p.pasta.join("bola.ase");
    let (bytes, _) = ph2d_pixel_io::ase::escrever(&antes).expect("grava");
    std::fs::write(&c, bytes).expect("grava");
    responde_o_proximo(c);
    p.app.pede(R::File(A::Open));
    p.quadro();
    let bits = object_add::born_from_paste(&mut p.sim, &mut p.app, spawn_root)
        .expect("há nascimento")
        .expect("nasceu");
    assert_ne!(bits, p.bits);
    let sprites = mode::sprites(p.sim.world());
    assert_eq!(sprites.len(), 2);
    let novo = sprites.iter().find(|(b, _)| *b == bits).expect("objecto").1;
    let s = &p.app.doc(novo).expect("doc").sprite;
    assert_eq!(s.frames().len(), antes.frames().len());
    assert_eq!(s.tags().len(), antes.tags().len());
    assert_eq!(
        p.app.doc(p.key).expect("doc").sprite,
        antes,
        "o antigo não muda"
    );
}

/// Ficheiro que não se lê: um aviso, nada de sprite novo.
#[test]
fn open_de_um_ficheiro_ruim_avisa() {
    let mut p = Palco::novo("ruim");
    let c = p.pasta.join("ruim.ase");
    std::fs::write(&c, b"isto nao e um ase").expect("grava");
    responde_o_proximo(c);
    p.app.pede(R::File(A::Open));
    p.quadro();
    assert!(object_add::born_from_paste(&mut p.sim, &mut p.app, spawn_root).is_none());
    assert_eq!(p.toasts.len(), 1, "o artista é avisado");
}

/// **O Import:** o ficheiro primeiro, a grelha sugerida na janela, e o verbo cria o sprite recortado.
#[test]
fn import_sprite_sheet_recorta_pela_grelha() {
    let mut p = Palco::novo("import");
    let c = p.pasta.join("walk.png");
    write_folha(&c);
    responde_o_proximo(c);
    p.app.pede(R::File(A::ImportSheet));
    p.quadro();
    assert!(p.aberta());
    let i = canal::imagem().expect("a imagem");
    assert_eq!((i.w, i.h), (12, 8));
    let g = i.sugestao.expect("a sugestão");
    assert_eq!((g.w, g.h), (4, 4));
    canal::pede(Pedido::Importar(g));
    p.quadro();
    assert!(!p.aberta(), "o Import fecha a janela");
    let bits = object_add::born_from_paste(&mut p.sim, &mut p.app, spawn_root)
        .expect("há nascimento")
        .expect("nasceu");
    let novo = mode::sprites(p.sim.world())
        .into_iter()
        .find(|(b, _)| *b == bits)
        .expect("objecto")
        .1;
    let s = &p.app.doc(novo).expect("doc").sprite;
    assert_eq!((s.width(), s.height(), s.frames().len()), (4, 4, 6));
    let segundo = ph2d_pixel_io::frame_rgba(s, s.frames()[1].id);
    assert_eq!(segundo[0], 60, "o 2.º quadro é a 2.ª célula");
}

/// O que o `.ase` traz e o documento não guarda chega ao artista em aviso, junto do «abriu».
#[test]
fn o_aviso_de_abrir_chega_ao_artista() {
    let frases =
        ph2d_panel_pixel_export::textos::do_aberto("x", vec![ph2d_pixel_io::ase::Aviso::Slices(2)]);
    assert_eq!(frases.len(), 2);
    assert!(frases[1].1.contains('2'), "{:?}", frases[1]);
}
