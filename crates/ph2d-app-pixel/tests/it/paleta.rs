//! **P2 pelo caminho real**: os pedidos do painel PIXEL (a paleta, a cor, a tinta) e o OK do diálogo
//! *Color Mode* chegam ao documento em Draw pelo [`panel::frame`] da shell — cada um UM passo do
//! desfazer, e reordenar a paleta NUNCA muda a imagem. Palco: a cena de smoke 3 (Indexed, DB32).

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, Press};
use ph2d_app_pixel::smoke::{BALL, BALL_RAMP, GROUND, SKY};
use ph2d_app_pixel::{PixelApp, Tool, mode, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::ToolRegistry;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{NewSpriteMode, PixelInk, PixelRequest, PixelSelect};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;
use ph2d_pixel_core::{ColorMode, compose_rgba};
use ph2d_render::Camera2d;

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
    /// A cena 3 em Draw, com um quadro da shell já corrido.
    fn novo() -> Self {
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(ph2d_app_pixel::smoke::stage(
            &mut app, &mut sim, 3, spawn_root
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

    fn quadro(&mut self) -> ph2d_editor_core::pixel_panel::PixelPanelInfo {
        panel::frame(
            &mut self.app,
            self.sim.world(),
            &mut self.hero,
            &mut self.toasts,
        )
    }

    fn pede(&mut self, r: PixelRequest) {
        self.app.pede(r);
        let _ = self.quadro();
    }

    fn sprite(&self) -> &ph2d_pixel_core::Sprite {
        &self.app.doc(self.key).expect("doc").sprite
    }

    fn passos(&self) -> usize {
        self.app.doc(self.key).expect("doc").history.undo_len()
    }

    /// A imagem composta, RGBA.
    fn imagem(&self) -> Vec<u8> {
        let s = self.sprite();
        let mut out = Vec::new();
        compose_rgba(s, s.frames()[0].id, None, &mut out);
        out
    }

    fn indice(&self, x: i32, y: i32) -> u32 {
        let d = self.app.doc(self.key).expect("doc");
        d.sprite.value_at((d.layer, d.frame), x, y)
    }

    fn clica(&mut self, (x, y): (i32, i32), b: Button) {
        let p = at(&self.app, &self.sim, &self.cam, self.bits, x, y);
        let c = canvas(&self.sim, &self.cam, self.bits);
        assert_eq!(
            input::press(&mut self.app, &c, p, b, Mods::default()),
            Press::Done
        );
        assert!(input::release(&mut self.app));
    }
}

#[test]
fn smoke_3_is_an_indexed_ball_with_its_ramp_selected_and_shading_in_hand_in_draw() {
    let mut p = Palco::novo();
    let s = p.sprite();
    assert_eq!(
        (s.width(), s.height(), s.mode()),
        (64, 48, ColorMode::Indexed)
    );
    assert_eq!(s.palette(), ph2d_pixel_core::palette::DEFAULT.as_slice());
    assert_eq!(p.passos(), 3, "céu, chão, bola");
    assert_eq!(p.app.selection(), BALL_RAMP);
    assert_eq!(p.app.options.ink, PixelInk::Shading);
    assert_eq!(p.app.tool, Tool::Pencil);
    let ((cx, cy), _) = BALL;
    assert!(BALL_RAMP.iter().any(|&k| p.indice(cx, cy) == k as u32));
    assert_eq!(p.indice(0, 0), SKY[0] as u32);
    assert_eq!(p.indice(0, 47), GROUND as u32);
    let info = p.quadro();
    assert_eq!(info.mode, NewSpriteMode::Indexed);
    assert_eq!(info.selected, BALL_RAMP);
    // A cena pede o Draw.
    let mut app = PixelApp::default();
    let mut sim = SimWorld::new();
    assert!(ph2d_app_pixel::smoke::stage(
        &mut app, &mut sim, 3, spawn_root
    ));
    let bits = mode::sprites(sim.world())[0].0;
    let mut tools = ToolRegistry::default();
    assert_eq!(
        mode::Family::new(&mut app, sim.world()).wants(&mut tools),
        Some((bits, ObjectMode::Draw))
    );
}

/// ⭐ O Shading anda UM degrau por traço na rampa seleccionada: esquerdo clareia (+1), direito
/// escurece (−1), parado nas pontas; o céu (fora da rampa) não muda; a borracha pinta com a Simple.
#[test]
fn shading_walks_the_selected_ramp_and_the_eraser_stays_simple() {
    let mut p = Palco::novo();
    p.app.options.size = 1;
    let ((cx, cy), _) = BALL;
    let antes = p.indice(cx, cy) as usize;
    let k = BALL_RAMP
        .iter()
        .position(|&i| i == antes)
        .expect("na rampa");
    p.clica((cx, cy), Button::Primary);
    let quer = BALL_RAMP[(k + 1).min(3)];
    assert_eq!(
        p.indice(cx, cy),
        quer as u32,
        "o esquerdo clareia um degrau"
    );
    p.clica((cx, cy), Button::Secondary);
    assert_eq!(
        p.indice(cx, cy),
        antes as u32,
        "o direito escurece um degrau"
    );
    p.clica((1, 1), Button::Primary);
    assert_eq!(p.indice(1, 1), SKY[0] as u32, "fora da rampa: intacto");
    p.app.tool = Tool::Eraser;
    p.clica((cx, cy), Button::Primary);
    let t = u32::from(p.sprite().transparent_index());
    assert_eq!(p.indice(cx, cy), t, "a borracha apaga, não sombreia");
}

/// A tinta escolhida no painel chega ao gesto: com Lock Alpha o lápis não pinta onde não há tinta.
#[test]
fn the_ink_chosen_in_the_panel_reaches_the_stroke() {
    let (mut app, sim, cam, bits) = crate::edit::born();
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.mode.enter(bits, ObjectMode::Draw);
    let mut toasts = ToastQueue::new();
    let mut o = app.options;
    o.ink = PixelInk::LockAlpha;
    app.pede(PixelRequest::Options(o));
    let _ = panel::frame(&mut app, sim.world(), &mut hero, &mut toasts);
    assert_eq!(app.options.ink, PixelInk::LockAlpha);
    let pt = at(&app, &sim, &cam, bits, 3, 3);
    let c = canvas(&sim, &cam, bits);
    assert_eq!(
        input::press(&mut app, &c, pt, Button::Primary, Mods::default()),
        Press::Done
    );
    assert!(input::release(&mut app));
    assert_eq!(crate::edit::value(&app, &sim, bits, 3, 3), [0, 0, 0, 0]);
    o.ink = PixelInk::Simple;
    app.pede(PixelRequest::Options(o));
    let _ = panel::frame(&mut app, sim.world(), &mut hero, &mut toasts);
    let c = canvas(&sim, &cam, bits);
    assert_eq!(
        input::press(&mut app, &c, pt, Button::Primary, Mods::default()),
        Press::Done
    );
    assert!(input::release(&mut app));
    assert_eq!(crate::edit::value(&app, &sim, bits, 3, 3), app.fg);
}

#[test]
fn a_palette_click_sets_the_foreground_and_shift_ctrl_shape_the_selection() {
    let mut p = Palco::novo();
    let pal = p.sprite().palette().to_vec();
    p.pede(PixelRequest::Swatch {
        index: 9,
        how: PixelSelect::Only,
    });
    assert_eq!((p.app.fg, p.app.selection()), (pal[9], vec![9]));
    p.pede(PixelRequest::Swatch {
        index: 12,
        how: PixelSelect::Range,
    });
    assert_eq!(p.app.selection(), vec![9, 10, 11, 12]);
    p.pede(PixelRequest::Swatch {
        index: 10,
        how: PixelSelect::Toggle,
    });
    assert_eq!(p.app.selection(), vec![9, 11, 12]);
    assert_eq!(
        p.app.anchor(),
        Some(10),
        "o Edit color edita a última clicada"
    );
    assert_eq!(p.passos(), 3, "seleccionar não é um passo do desfazer");
}

/// ⭐ Ordenar e mover reordenam a paleta e o desenho segue as cores: a imagem composta fica igual
/// ao byte, e o desfazer devolve a ordem.
#[test]
fn sort_and_move_keep_the_composed_image_byte_identical() {
    let mut p = Palco::novo();
    let (img, pal) = (p.imagem(), p.sprite().palette().to_vec());
    p.pede(PixelRequest::Swatch {
        index: 0,
        how: PixelSelect::Only,
    });
    p.pede(PixelRequest::Sort { descending: true });
    assert_ne!(
        p.sprite().palette(),
        pal.as_slice(),
        "a paleta inteira ordenou"
    );
    assert_eq!(p.imagem(), img, "a imagem não muda");
    assert_eq!(p.passos(), 4);
    assert!(p.app.undo(p.key));
    assert_eq!(p.sprite().palette(), pal.as_slice());

    p.app.select(BALL_RAMP);
    p.pede(PixelRequest::MoveSelection(1));
    assert_eq!(p.app.selection(), vec![5, 6, 7, 8]);
    assert_eq!(p.sprite().palette()[5], pal[4]);
    assert_eq!(p.sprite().palette()[4], pal[8]);
    assert_eq!(p.imagem(), img, "mover não muda a imagem");
    p.pede(PixelRequest::MoveSelection(-1));
    assert_eq!(p.sprite().palette(), pal.as_slice());
}

#[test]
fn palette_size_truncates_or_pads_with_black_as_one_step() {
    let mut p = Palco::novo();
    p.pede(PixelRequest::PaletteSize(40));
    let s = p.sprite();
    assert_eq!(s.palette().len(), 40);
    assert_eq!(s.palette()[39], [0, 0, 0, 255]);
    p.pede(PixelRequest::PaletteSize(8));
    assert_eq!(p.sprite().palette().len(), 8);
    assert_eq!(p.passos(), 5);
    assert!(p.app.undo(p.key) && p.app.undo(p.key));
    assert_eq!(p.sprite().palette().len(), 32);
}

#[test]
fn select_used_colors_picks_the_indices_the_drawing_uses() {
    let mut p = Palco::novo();
    p.pede(PixelRequest::SelectUsed);
    let mut quer: Vec<usize> = SKY.iter().chain(&BALL_RAMP).copied().collect();
    quer.push(GROUND);
    quer.sort_unstable();
    assert_eq!(p.app.selection(), quer);
}

#[test]
fn new_palette_from_sprite_keeps_the_image_and_is_one_step() {
    let mut p = Palco::novo();
    let img = p.imagem();
    p.pede(PixelRequest::PaletteFromSprite);
    assert_eq!(
        p.sprite().palette().len(),
        1 + SKY.len() + BALL_RAMP.len() + 1
    );
    assert_eq!(p.imagem(), img, "as células seguiram as cores");
    assert_eq!(p.passos(), 4);
    assert!(p.app.undo(p.key));
    assert_eq!(p.sprite().palette().len(), 32);
}

#[test]
fn load_default_and_save_then_load_a_palette_file() {
    let mut p = Palco::novo();
    let pal = p.sprite().palette().to_vec();
    let dir = std::env::temp_dir().join(format!("ph2d-pixel-pal-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("pasta");
    let ficheiro = dir.join("bola.gpl");
    ph2d_app_host::modal::responde_o_proximo(ficheiro.clone());
    p.pede(PixelRequest::SavePalette);
    assert!(ficheiro.exists(), "gravou");
    p.pede(PixelRequest::PaletteSize(4));
    ph2d_app_host::modal::responde_o_proximo(ficheiro.clone());
    p.pede(PixelRequest::LoadPalette);
    assert_eq!(p.sprite().palette(), pal.as_slice(), "leu as 32 de volta");
    p.pede(PixelRequest::PaletteSize(4));
    p.pede(PixelRequest::DefaultPalette);
    assert_eq!(
        p.sprite().palette(),
        ph2d_pixel_core::palette::DEFAULT.as_slice()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn replace_color_turns_the_foreground_into_the_background_in_one_step() {
    let mut p = Palco::novo();
    let pal = p.sprite().palette().to_vec();
    p.app.fg = pal[GROUND];
    p.app.bg = pal[8];
    p.pede(PixelRequest::ReplaceColor);
    assert_eq!(p.indice(0, 47), 8);
    assert_eq!(p.indice(0, 0), SKY[0] as u32, "só a cor de frente");
    assert_eq!(p.passos(), 4);
    assert!(p.app.undo(p.key));
    assert_eq!(p.indice(0, 47), GROUND as u32);
    // Nenhuma célula da cor: nenhum passo.
    p.app.fg = pal[30];
    p.pede(PixelRequest::ReplaceColor);
    assert_eq!(p.passos(), 3);
}

#[test]
fn the_color_mode_ok_converts_the_sprite_as_one_undo_step() {
    let mut p = Palco::novo();
    let img = p.imagem();
    p.hero.store.open_color_mode_dialog(NewSpriteMode::Indexed);
    assert!(p.hero.store.set_color_mode_target(NewSpriteMode::Rgba));
    p.hero.store.request_color_mode();
    let info = p.quadro();
    assert_eq!(p.sprite().mode(), ColorMode::Rgba);
    assert_eq!(info.mode, NewSpriteMode::Rgba);
    assert_eq!(p.imagem(), img, "Indexed → RGB é exacto");
    assert!(p.app.undo(p.key));
    assert_eq!(p.sprite().mode(), ColorMode::Indexed);
}

/// O *Edit color*: o seletor partilhado aberto na amostra escreve a cor da última amostra clicada,
/// e arrastar no seletor é UM passo; fechado, a amostra mostra a cor da paleta.
#[test]
fn edit_color_through_the_shared_picker_is_one_undo_step() {
    let mut p = Palco::novo();
    let id = ph2d_editor_core::ids::PIXEL_PAL_EDIT;
    p.pede(PixelRequest::Swatch {
        index: 5,
        how: PixelSelect::Only,
    });
    let pal5 = p.sprite().palette()[5];
    assert_eq!(p.hero.store.widget_color(id), Some(pal5));
    p.hero.store.open_picker_on(id);
    for c in [[10, 20, 30, 255], [40, 50, 60, 255]] {
        p.hero.store.set_widget_color(id, c);
        let _ = p.quadro();
    }
    assert_eq!(p.sprite().palette()[5], [40, 50, 60, 255]);
    assert_eq!(p.passos(), 4, "arrastar no seletor é UM passo");
    p.hero.store.set_picker_target(None);
    let _ = p.quadro();
    assert!(p.app.undo(p.key));
    let _ = p.quadro();
    assert_eq!(p.sprite().palette()[5], pal5);
    assert_eq!(p.hero.store.widget_color(id), Some(pal5));
}
