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
    let frases = ph2d_panel_pixel_export::textos::do_aberto(
        "x",
        vec![ph2d_pixel_io::ase::Aviso::TerrenosFicam],
    );
    assert_eq!(frases.len(), 2);
    assert!(!frases[1].1.is_empty(), "{:?}", frases[1]);
}

/// **Exporta a folha e volta a importá-la: os tempos e as tags são os mesmos** (smoke do dono 05/11:
/// «uma só tem keys de 100 ms»). A cena 20 tem quadros de 120 e de 480 ms. *Mutação: ignorar o `.json`
/// ⇒ tudo 100 ms, vermelho.*
#[test]
fn a_folha_importada_tem_os_tempos_da_exportada() {
    let mut p = Palco::novo("tempos");
    let original = p.app.doc(p.key).expect("doc").sprite.clone();
    assert!(
        original.frames().iter().any(|f| f.duration_ms != 100),
        "vácuo: a cena tem de ter tempos diferentes"
    );
    let c = p.pasta.join("bola.png");
    p.app.pede(R::File(A::ExportSheet));
    p.quadro();
    responde_o_proximo(c.clone());
    canal::pede(Pedido::Exportar(Export::Folha(ExportFolha {
        opcoes: OpcoesFolha::default(),
        json: Some(ph2d_pixel_io::sheet::OpcoesJson {
            tags: true,
            ..Default::default()
        }),
    })));
    p.quadro();
    p.espera();

    responde_o_proximo(c);
    p.app.pede(R::File(A::ImportSheet));
    p.quadro();
    let i = canal::imagem().expect("a imagem");
    assert!(i.com_dados, "o .json ao lado");
    canal::pede(Pedido::Importar(ph2d_pixel_io::abrir::Grelha {
        x: 0,
        y: 0,
        w: 64,
        h: 32,
        gap_x: 0,
        gap_y: 0,
        por_colunas: false,
    }));
    p.quadro();
    let bits = object_add::born_from_paste(&mut p.sim, &mut p.app, spawn_root)
        .expect("há nascimento")
        .expect("nasceu");
    let novo = mode::sprites(p.sim.world())
        .into_iter()
        .find(|(b, _)| *b == bits)
        .expect("objecto")
        .1;
    let s = &p.app.doc(novo).expect("doc").sprite;
    let ms = |x: &ph2d_pixel_core::Sprite| -> Vec<u32> {
        x.frames().iter().map(|f| f.duration_ms).collect()
    };
    assert_eq!(ms(s), ms(&original));
    let tags = |x: &ph2d_pixel_core::Sprite| -> Vec<_> {
        x.tags()
            .iter()
            .map(|t| (t.name.clone(), t.from, t.to, t.direction, t.repeat, t.color))
            .collect()
    };
    assert_eq!(tags(s), tags(&original));
}

/// O File ▸ Import até à janela, e a resposta dela.
fn responde(p: &mut Palco, c: Vec<PathBuf>, pixel: bool) {
    p.hero.store.ask_import_as(c);
    p.quadro();
    assert!(p.aberta(), "a janela abre");
    canal::pede(Pedido::ImportarComo { pixel });
    p.quadro();
    assert!(!p.aberta(), "a resposta fecha a janela");
}

/// **A janela do File ▸ Import:** «Pixel Sprite» nasce um por ficheiro (e os PNG viram uma sequência);
/// «Image» deixa os ficheiros para a shell importar como imagem do Painter, sem nascer sprite.
#[test]
fn a_janela_do_import_decide_o_tipo_do_objeto() {
    let mut p = Palco::novo("pergunta");
    let s = p.app.doc(p.key).expect("doc").sprite.clone();
    let (ase, _) = ph2d_pixel_io::ase::escrever(&s).expect("grava");
    let (a, b) = (p.pasta.join("a.ase"), p.pasta.join("b.ase"));
    std::fs::write(&a, &ase).expect("grava");
    std::fs::write(&b, &ase).expect("grava");
    let (x, y) = (p.pasta.join("w1.png"), p.pasta.join("w2.png"));
    write_folha(&x);
    write_folha(&y);
    assert!(ph2d_app_pixel::exportar::le_todos(&[a.clone(), x.clone()]));
    assert!(!ph2d_app_pixel::exportar::le_todos(&[p
        .pasta
        .join("c.svg")]));

    let sprites = |p: &Palco| mode::sprites(p.sim.world()).len();
    let nasce = |p: &mut Palco| {
        while object_add::born_from_paste(&mut p.sim, &mut p.app, spawn_root).is_some() {}
    };
    let antes = sprites(&p);
    p.hero.store.ask_import_as(vec![a.clone()]);
    p.quadro();
    let f = canal::pergunta().expect("o que a janela mostra");
    assert_eq!(f.nome, "a.ase");
    assert_eq!((f.lados, f.quadros), ((s.width(), s.height()), 16));
    assert_eq!((f.camadas, f.tags), (s.layers().len(), s.tags().len()));
    assert!(f.pixel_de_fabrica, "um .ase parte de Pixel Sprite");
    assert_eq!(
        canal::imagem().map(|i| (i.w, i.h)),
        Some((s.width(), s.height()))
    );
    p.hero.panel_visibility.insert(PANEL_PIXEL_EXPORT, false);
    p.quadro();
    assert_eq!(canal::pergunta(), None, "Cancel não deixa nada para trás");
    nasce(&mut p);
    assert_eq!(sprites(&p), antes, "Cancel não importa");

    responde(&mut p, vec![a.clone(), b], true);
    nasce(&mut p);
    assert_eq!(sprites(&p), antes + 2, "um por .ase");

    responde(&mut p, vec![x.clone(), y], true);
    nasce(&mut p);
    assert_eq!(sprites(&p), antes + 3, "os PNG, uma sequência");

    p.hero.store.ask_import_as(vec![x]);
    p.quadro();
    assert!(
        !canal::pergunta().expect("janela").pixel_de_fabrica,
        "um PNG parte de imagem"
    );
    canal::pede(Pedido::ImportarComo { pixel: false });
    p.quadro();
    nasce(&mut p);
    assert_eq!(sprites(&p), antes + 3, "a imagem não é da família");
    assert!(
        p.hero.store.take_import_as_image().is_some(),
        "volta à shell"
    );
}

/// A janela abre sem sprite em Draw (o 1.º ficheiro do projecto) e o Pixel Sprite nasce na mesma.
#[test]
fn a_janela_do_import_funciona_fora_do_draw() {
    let mut p = Palco::novo("fora");
    let c = p.pasta.join("w.png");
    write_folha(&c);
    let _ = p.hero.gizmo.mode.leave();
    p.quadro();
    responde(&mut p, vec![c], true);
    assert!(object_add::born_from_paste(&mut p.sim, &mut p.app, spawn_root).is_some());
}

/// Um ficheiro que não se lê diz porquê na janela, e parte de imagem.
#[test]
fn a_janela_do_import_diz_porque_nao_le() {
    let mut p = Palco::novo("ruim2");
    let c = p.pasta.join("r.ase");
    std::fs::write(&c, b"nao").expect("grava");
    p.hero.store.ask_import_as(vec![c]);
    p.quadro();
    let f = canal::pergunta().expect("janela");
    assert!(f.erro.is_some());
    assert_eq!(canal::imagem(), None);
}

/// A folha a montar quando o Import abre não troca a imagem dele.
#[test]
fn a_previa_atrasada_nao_troca_a_imagem_do_import() {
    let mut p = Palco::novo("atrasada");
    let c = p.pasta.join("w.png");
    write_folha(&c);
    p.app.pede(R::File(A::ExportSheet));
    p.quadro();
    canal::pede_folha(Some(OpcoesFolha::default()));
    p.quadro();
    assert!(p.app.exportando(), "a folha está a montar");
    responde_o_proximo(c);
    p.app.pede(R::File(A::ImportSheet));
    p.quadro();
    canal::pede_folha(None);
    p.espera();
    assert_eq!(canal::imagem().map(|i| (i.w, i.h)), Some((12, 8)));
}

/// O Repeat acende assim que o export começa.
#[test]
fn o_repeat_acende_quando_o_export_comeca() {
    let mut p = Palco::novo("acende");
    p.app.pede(R::File(A::ExportAs));
    p.quadro();
    responde_o_proximo(p.pasta.join("a.gif"));
    canal::pede(Pedido::Exportar(gif()));
    p.quadro();
    p.quadro();
    assert!(
        !p.hero
            .store
            .menu_row_is_unavailable(ids::CTX_MENU_PIXEL_REPEAT_EXPORT)
    );
    p.espera();
}

/// P14e-2 (4): **Open de um `.ase` com `ph2d.collider`** nasce o objecto COM a filha que segue a
/// slice, no MESMO `born_from_paste` (um quadro = um passo do desfazer do mundo); a marca sai do
/// documento ao ler; regravar pelo conjunto do mundo dá UMA marca.
#[test]
fn open_de_um_ase_com_collider_nasce_a_filha() {
    let mut p = Palco::novo("open_collider");
    // (d) A régua do PROJECTO (50 px/m, não a de omissão 100): o `deriva` a escreve.
    p.hero.project.pixels_per_meter = 50.0;
    ph2d_app_pixel::deriva(&p.app, p.sim.world_mut(), Some(&p.hero));
    let s = ph2d_pixel_cena::fixture::heroi_p14();
    let corpo = ph2d_pixel_cena::fixture::body(&s);
    let marcado = ph2d_pixel_cena::com_colliders(&s, &std::collections::BTreeSet::from([corpo.0]));
    let c = p.pasta.join("heroi.ase");
    std::fs::write(&c, ph2d_pixel_io::ase::escrever(&marcado).expect("grava").0).expect("grava");
    responde_o_proximo(c);
    p.app.pede(R::File(A::Open));
    p.quadro();
    let bits = object_add::born_from_paste(&mut p.sim, &mut p.app, spawn_root)
        .expect("há nascimento")
        .expect("nasceu");
    let e = ph2d_ecs::Entity::try_from_bits(bits).expect("entidade");
    let key = p
        .sim
        .world()
        .get::<ph2d_ecs::PixelSprite>(e)
        .expect("pixel")
        .0;
    let doc = p.app.doc(key).expect("doc").sprite.clone();
    let id = doc
        .slices()
        .iter()
        .find(|x| x.name == "body")
        .expect("body")
        .id;
    let filha = ph2d_pixel_cena::collider_da_slice(p.sim.world(), e, id)
        .expect("a filha nasce com o objecto");
    // A forma da filha na régua do projecto: a do Add Collider com 1/50 m por píxel.
    let mut w = SimWorld::new();
    let o = w.world_mut().spawn(ph2d_ecs::Transform::IDENTITY).id();
    let q = doc.frames()[0].id;
    let f2 = ph2d_pixel_cena::adiciona_collider_da_slice(w.world_mut(), o, id, &doc, q, 1.0 / 50.0)
        .expect("molde");
    let forma = |w: &ph2d_ecs::World, f| {
        let c = w.get::<ph2d_physics_ecs::Collider>(f).expect("collider");
        (c.shape, c.offset)
    };
    assert_eq!(
        forma(p.sim.world(), filha),
        forma(w.world(), f2),
        "régua do projecto"
    );
    assert!(
        !ph2d_pixel_cena::slice_e_collider(doc.slice(id).expect("body")),
        "a marca sai do documento"
    );
    let mundo = ph2d_pixel_cena::slices_com_collider(p.sim.world_mut(), key);
    assert_eq!(mundo.len(), 1);
    let (b, _) = ph2d_pixel_io::ase::escrever(&ph2d_pixel_cena::com_colliders(&doc, &mundo))
        .expect("regrava");
    let (_, _, de_novo) = ph2d_pixel_io::ase::ler_com_colliders(&b).expect("relê");
    assert_eq!(de_novo, mundo, "regravar não duplica a marca");
}

/// P14e-2 (5): **Export e Repeat Last Export gravam a marca que vem do MUNDO** (`a_gravar`): o
/// herói aberto com o collider em `body` sai no `.ase` com `ph2d.collider`, pelo clique do Export e
/// pela tecla do Repeat. *Mutação: cada caminho a gravar o documento cru ⇒ vermelho.*
#[test]
fn export_e_repeat_gravam_o_collider_do_mundo() {
    let mut p = Palco::novo("export_collider");
    let s = ph2d_pixel_cena::fixture::heroi_p14();
    let corpo = ph2d_pixel_cena::fixture::body(&s);
    let marcado = ph2d_pixel_cena::com_colliders(&s, &std::collections::BTreeSet::from([corpo.0]));
    let c = p.pasta.join("heroi_in.ase");
    std::fs::write(&c, ph2d_pixel_io::ase::escrever(&marcado).expect("grava").0).expect("grava");
    responde_o_proximo(c);
    p.app.pede(R::File(A::Open));
    p.quadro();
    let bits = object_add::born_from_paste(&mut p.sim, &mut p.app, spawn_root)
        .expect("há nascimento")
        .expect("nasceu");
    let e = ph2d_ecs::Entity::try_from_bits(bits).expect("entidade");
    p.key = p
        .sim
        .world()
        .get::<ph2d_ecs::PixelSprite>(e)
        .expect("pixel")
        .0;
    p.bits = bits;
    p.hero.gizmo.mode.enter(bits, ObjectMode::Draw);
    let mut cam = Camera2d::default();
    p.app
        .frame(&mut p.sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
    ph2d_app_pixel::deriva(&p.app, p.sim.world_mut(), Some(&p.hero));
    p.quadro();
    let marca = |b: &[u8]| {
        let (r, _, col) = ph2d_pixel_io::ase::ler_com_colliders(b).expect("relê");
        let id = r
            .slices()
            .iter()
            .find(|x| x.name == "body")
            .expect("body")
            .id;
        col.contains(&id.0)
    };
    p.app.pede(R::File(A::ExportAs));
    p.quadro();
    let saida = p.pasta.join("heroi_out.ase");
    responde_o_proximo(saida.clone());
    canal::pede(Pedido::Exportar(Export::Ficheiro(ExportAs {
        pedido: render::Pedido {
            escala: 1.0,
            ..render::Pedido::default()
        },
        formato: Formato::Ase,
        repeat: 0,
    })));
    p.quadro();
    p.espera();
    assert!(
        marca(&std::fs::read(&saida).expect("o Export gravou")),
        "o Export leva a marca"
    );
    std::fs::remove_file(&saida).expect("apaga");
    p.tecla(KeyCode::KeyX, true, true, false);
    p.espera();
    assert!(
        marca(&std::fs::read(&saida).expect("o Repeat gravou")),
        "o Repeat leva a marca"
    );
}
