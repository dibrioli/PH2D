//! ⭐ **A cena 24 ensina o que acontece** (P11, `docs/PixelArt/P11_plano.md` §8) e o `Transform` sobre
//! uma cel ligada: a imagem é uma só, logo os dois quadros mudam juntos.

use ph2d_app_pixel::compose::View;
use ph2d_app_pixel::smoke_slices::{
    BOOT, BUTTON, BUTTON_BORDER, BUTTON_CENTER, COLOR_BUTTON, COLOR_HERO, FILL, FRAMES, GROUND_Y,
    HERO_PIVOT, HERO_SIZE, hero_box,
};
use ph2d_app_pixel::{Tool, mode, smoke};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::screens::hero::{HeroScreen, PANEL_PIXEL_SLICES};
use ph2d_panel_pixel_slices::canal::{self, Pedido};
use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::{CelKey, FrameId, LayerId, Op, Patch, Rect, Slice, Sprite, layers};
use ph2d_render::Camera2d;
use winit::keyboard::KeyCode;

use crate::comum::{PPM, WINDOW, spawn_root};
use crate::slices::{Mesa, NADA, SHIFT, px};

fn cena_24() -> Mesa {
    let _ = canal::take_pedidos();
    let mut app = ph2d_app_pixel::PixelApp::default();
    let mut sim = SimWorld::new();
    assert!(smoke::stage(&mut app, &mut sim, 24, spawn_root));
    let (bits, key) = mode::sprites(sim.world())[0];
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.mode.enter(bits, ObjectMode::Draw);
    let mut cam = Camera2d::default();
    app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
    let mut m = Mesa {
        app,
        sim,
        cam,
        toasts: ToastQueue::default(),
        hero,
        bits,
        key,
    };
    m.quadro();
    m
}

fn pixel(s: &Sprite, key: CelKey, x: i32, y: i32) -> [u8; 4] {
    let p = s.read_rect(key, Rect::new(x, y, 1, 1));
    [p[0], p[1], p[2], p[3]]
}

fn nomeada<'a>(s: &'a Sprite, nome: &str) -> &'a Slice {
    s.slices().iter().find(|c| c.name == nome).expect(nome)
}

#[test]
fn a_cena_24_abre_com_a_ferramenta_e_as_slices_sobre_a_arte() {
    let m = cena_24();
    let s = m.sprite();
    assert_eq!(m.app.tool, Tool::Slice, "a ferramenta na mão");
    assert_eq!(
        m.hero.panel_visibility.get(PANEL_PIXEL_SLICES),
        Some(&true),
        "a janela SLICES aberta"
    );
    assert_eq!(s.frames().len(), FRAMES as usize);
    assert_eq!(s.slices().len(), 2);
    let (button, hero) = (nomeada(s, "button"), nomeada(s, "hero"));
    assert_eq!(m.app.slices_selected(m.key), [hero.id], "abre com hero");
    assert_eq!(
        (button.user.color, hero.user.color),
        (COLOR_BUTTON, COLOR_HERO)
    );
    assert_ne!(COLOR_BUTTON, COLOR_HERO);

    let d = m.app.doc(m.key).expect("doc");
    let arte = d.layer;
    assert_eq!(s.layer(arte).expect("camada").name, "Art");
    let ids: Vec<FrameId> = s.frames().iter().map(|f| f.id).collect();

    // O botão: o miolo do 9-slice é EXATAMENTE o enchimento, em todos os quadros.
    let forma = button.shape_at(0).expect("key");
    assert_eq!(forma.bounds, BUTTON);
    assert_eq!(forma.center, Some(BUTTON_CENTER));
    assert_eq!(
        (BUTTON_CENTER.x, BUTTON_CENTER.y),
        (BUTTON_BORDER, BUTTON_BORDER)
    );
    assert_eq!(button.keys.len(), 1, "o botão não anda");
    for &f in &ids {
        for y in 0..BUTTON.h as i32 {
            for x in 0..BUTTON.w as i32 {
                let de_dentro = BUTTON_CENTER.contains(x, y);
                let c = pixel(s, (arte, f), BUTTON.x + x, BUTTON.y + y);
                assert_eq!(c == FILL, de_dentro, "({x},{y}): o miolo é o enchimento");
            }
        }
    }

    // O boneco: uma key por quadro, a caixa justa, o pivot no pé que toca o chão.
    let fundo = s.layers()[0].id;
    assert_eq!(hero.keys.len(), FRAMES as usize);
    for (k, &f) in ids.iter().enumerate() {
        let k = k as u32;
        assert!(hero.has_key_at(k), "a marca do quadro {k}");
        let forma = hero.shape_at(k).expect("key");
        assert_eq!(forma.bounds, hero_box(k));
        assert_eq!(forma.pivot, Some(HERO_PIVOT));
        let b = forma.bounds;
        let (px, py) = (b.x + HERO_PIVOT.0, b.y + HERO_PIVOT.1);
        assert_eq!(
            pixel(s, (arte, f), px, py),
            BOOT,
            "o pivot no pé, quadro {k}"
        );
        assert_eq!(py + 1, GROUND_Y, "o pé assenta no chão, quadro {k}");
        assert_ne!(
            pixel(s, (fundo, f), px, GROUND_Y)[3],
            0,
            "há chão, quadro {k}"
        );
        // Fora do botão, toda a arte do quadro está dentro da caixa, e a caixa não sobra num lado.
        let (mut x0, mut x1, mut y0, mut y1) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
        for y in (BUTTON.y + BUTTON.h as i32)..s.height() as i32 {
            for x in 0..s.width() as i32 {
                if pixel(s, (arte, f), x, y)[3] != 0 {
                    assert!(b.contains(x, y), "({x},{y}) fora de hero, quadro {k}");
                    (x0, x1, y0, y1) = (x0.min(x), x1.max(x), y0.min(y), y1.max(y));
                }
            }
        }
        assert!(x0 > b.x - 1 && x1 < b.x + HERO_SIZE.0 as i32);
        assert_eq!((y0, y1), (b.y, b.y + HERO_SIZE.1 as i32 - 1), "quadro {k}");
    }
    let (a, z) = (hero_box(0), hero_box(FRAMES - 1));
    assert!(
        z.x + z.w as i32 <= s.width() as i32 && z.x > a.x,
        "atravessa"
    );
}

/// **`Transform` sobre uma cel LIGADA:** a imagem é uma só (a lei do módulo), logo o outro quadro ligado
/// mostra o mesmo desenho; a ligação não se parte, o quadro não ligado e a outra camada ficam; a key é só
/// do quadro atual; `Ctrl+Z` volta ao byte. Separa: «partiu a ligação» (os dois divergem) · «só andou
/// um» · «escreveu duas vezes» (o desenho anda o dobro no outro quadro) · «o desfazer deixou lixo».
#[test]
fn transform_em_cel_ligada_e_uma_imagem_so_e_o_desfazer_volta_ao_byte() {
    let (mut t, liga, outra, f0, f1, f2) = ligados();
    assert!(t.tecla(KeyCode::KeyC, SHIFT));
    let quadrado = Rect::new(8, 8, 4, 4);
    t.arrasta(px(8, 8), px(11, 11), NADA);
    let id = t.sprite().slices().last().expect("nasceu").id;
    t.arrasta(px(9, 9), px(9, 9), NADA);
    assert_eq!(t.app.slices_selected(t.key), [id]);
    canal::pede(Pedido::Transform);
    t.quadro();
    let antes = t.sprite().clone();
    let passos = t.app.doc(t.key).expect("doc").history.undo_len();

    t.arrasta(px(9, 9), px(15, 9), NADA);

    let s = t.sprite();
    let novo = Rect::new(14, 8, 4, 4);
    let (a, b) = (
        s.read_rect((liga, f0), s.bounds()),
        s.read_rect((liga, f1), s.bounds()),
    );
    assert_eq!(a, b, "a mesma imagem nos dois quadros");
    assert_eq!(
        s.owner((liga, f0)),
        s.owner((liga, f1)),
        "a ligação não se parte"
    );
    assert_eq!(
        pixel(s, (liga, f1), novo.x, novo.y),
        [10, 0, 0, 255],
        "anda uma vez"
    );
    assert_eq!(
        pixel(s, (liga, f1), novo.x + 6, novo.y)[3],
        0,
        "não anda o dobro"
    );
    assert_eq!(
        pixel(s, (liga, f0), quadrado.x, quadrado.y)[3],
        0,
        "o buraco"
    );
    assert_eq!(
        pixel(s, (liga, f2), quadrado.x, quadrado.y),
        [20, 0, 0, 255],
        "o quadro por ligar fica"
    );
    assert_eq!(
        s.read_rect((outra, f0), s.bounds()),
        antes.read_rect((outra, f0), s.bounds()),
        "a outra camada fica"
    );
    assert_eq!(
        s.slice(id).expect("slice").keys.len(),
        1,
        "a key só neste quadro"
    );
    assert_eq!(
        t.app.doc(t.key).expect("doc").history.undo_len(),
        passos + 1,
        "um passo só"
    );

    assert!(t.tecla(KeyCode::KeyZ, crate::slices::CTRL));
    assert_eq!(t.sprite(), &antes, "o desfazer volta ao byte");
}

type Ligados = (Mesa, LayerId, LayerId, FrameId, FrameId, FrameId);

/// A camada ativa com os quadros 0 e 1 ligados e o 2 solto; outra camada por baixo.
fn ligados() -> Ligados {
    let mut t = Mesa::nova();
    let d = t.app.doc(t.key).expect("doc");
    let (outra, f0) = (d.layer, d.frame);
    let d = t.app.foca(t.key).expect("doc");
    let (op, liga) = layers::new_layer(&mut d.sprite, outra).expect("c");
    t.app.commit(t.key, op).expect("c");
    let quadrado = Rect::new(8, 8, 4, 4);
    let tinta = |t: &mut Mesa, l, f, c: [u8; 4]| {
        let op = Op::Patch(Patch {
            layer: l,
            frame: f,
            bbox: Some(quadrado),
            rect: quadrado,
            pixels: c.repeat(quadrado.area()),
        });
        t.app.commit(t.key, op).expect("pinta");
    };
    tinta(&mut t, outra, f0, [99, 0, 0, 255]);
    tinta(&mut t, liga, f0, [10, 0, 0, 255]);
    let d = t.app.foca(t.key).expect("doc");
    let (op, f1) = frames::new_frame(&mut d.sprite, f0, NewFrame::Copy).expect("q");
    t.app.commit(t.key, op).expect("q");
    let op = frames::link(t.sprite(), liga, &[f0, f1]).expect("liga");
    t.app.commit(t.key, op).expect("liga");
    let d = t.app.foca(t.key).expect("doc");
    let (op, f2) = frames::new_frame(&mut d.sprite, f1, NewFrame::Empty).expect("q");
    t.app.commit(t.key, op).expect("q");
    tinta(&mut t, liga, f2, [20, 0, 0, 255]);
    let d = t.app.foca(t.key).expect("doc");
    (d.layer, d.frame) = (liga, f0);
    (t, liga, outra, f0, f1, f2)
}

/// O cursor acompanha a slice: a célula sob o ponteiro anda durante o gesto, não fica no sítio do premir.
#[test]
fn o_cursor_acompanha_o_ponteiro_durante_o_gesto_da_slice() {
    let mut t = Mesa::nova();
    assert!(t.tecla(KeyCode::KeyC, SHIFT));
    t.arrasta(px(4, 4), px(14, 10), NADA);
    assert_eq!(t.app.hover_cell(), Some((14, 10)));
}

/// Com um menu aberto a família cede as teclas: a seleção da slice não paga pelo `Esc` que fecha o menu.
#[test]
fn um_menu_aberto_cede_o_teclado_da_familia() {
    let mut t = cena_24();
    let cede =
        |t: &mut Mesa| View::of(t.sim.world(), &t.hero, &mut t.cam, &mut t.toasts, WINDOW).typing;
    assert!(!cede(&mut t), "sem menu, as teclas são da família");
    t.hero.store.open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::NewSpriteDialog,
    });
    assert!(cede(&mut t), "com o menu aberto, cede");
    t.hero.store.close_context_menu();
    assert!(!cede(&mut t), "fechado, volta");
}

/// `Esc` com o menu da slice à vista fecha o menu e a seleção fica; o `Esc` seguinte limpa-a.
#[test]
fn esc_com_o_menu_da_slice_fecha_so_o_menu() {
    let mut t = cena_24();
    let sel = t.app.slices_selected(t.key).to_vec();
    assert_eq!(sel.len(), 1);
    canal::publish_menu_aberto(true);
    assert!(t.tecla(KeyCode::Escape, NADA));
    assert_eq!(t.app.slices_selected(t.key), sel, "a seleção fica");
    assert!(canal::take_fechar_menu(), "o painel fecha o menu");
    canal::publish_menu_aberto(false);
    assert!(t.tecla(KeyCode::Escape, NADA));
    assert!(t.app.slices_selected(t.key).is_empty(), "sem menu, limpa");
}
