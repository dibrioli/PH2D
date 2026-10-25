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
use ph2d_editor_core::pixel_panel::{
    NewSpriteMode, PixelDither, PixelInk, PixelRequest, PixelSelect,
};
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
        Self::cena(3)
    }

    /// A cena `level` em Draw, com um quadro da shell já corrido.
    fn cena(level: u32) -> Self {
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(ph2d_app_pixel::smoke::stage(
            &mut app, &mut sim, level, spawn_root
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
    p.hero.store.set_color_mode_target(NewSpriteMode::Rgba);
    p.hero.store.request_color_mode();
    let info = p.quadro();
    assert_eq!(p.sprite().mode(), ColorMode::Rgba);
    assert_eq!(info.mode, NewSpriteMode::Rgba);
    assert_eq!(p.imagem(), img, "Indexed → RGB é exacto");
    assert!(p.app.undo(p.key));
    assert_eq!(p.sprite().mode(), ColorMode::Indexed);
}

/// OK com o alvo = o modo actual (o diálogo abre assim): nada muda e nenhum passo do desfazer.
#[test]
fn the_color_mode_ok_on_the_current_mode_is_no_undo_step() {
    let mut p = Palco::novo();
    let _ = p.quadro();
    let (img, passos) = (p.imagem(), p.passos());
    p.hero.store.open_color_mode_dialog(NewSpriteMode::Indexed);
    assert_eq!(
        p.hero.store.color_mode_choice().target,
        NewSpriteMode::Indexed
    );
    p.hero.store.request_color_mode();
    assert!(p.hero.store.context_menu().is_none(), "o OK fecha");
    let info = p.quadro();
    assert_eq!(info.mode, NewSpriteMode::Indexed);
    assert_eq!(p.sprite().mode(), ColorMode::Indexed);
    assert_eq!(p.imagem(), img);
    assert_eq!(p.passos(), passos, "nenhum passo do desfazer");
}

/// ⭐ **A cena 4 MOSTRA o Color Mode** (smoke do dono 25/10: na cena 3 «não percebi diferença»):
/// a imagem RGBA tem centenas de cores, cada dither dá outro plano de índices sobre a DB32, o
/// Grayscale dá cinza, e o `Ctrl+Z` volta sempre à imagem lisa. (Old 2×2 = Ordered 2×2 é a lei
/// medida no Aseprite, `ph2d_pixel_core::convert`.)
#[test]
fn scene_4_shows_every_color_mode_difference() {
    use ph2d_editor_core::ids::COLOR_MODE_DITHERS;
    use std::collections::BTreeSet;
    let mut p = Palco::cena(4);
    assert_eq!(p.sprite().mode(), ColorMode::Rgba);
    let lisa = p.imagem();
    let cores: BTreeSet<&[u8; 4]> = lisa.as_chunks::<4>().0.iter().collect();
    assert!(
        cores.len() > 500,
        "a imagem lisa tem só {} cores",
        cores.len()
    );
    let (w, h) = ph2d_app_pixel::smoke::GRADIENT_SIZE;
    let mut planos = Vec::new();
    for (d, ..) in COLOR_MODE_DITHERS {
        p.hero.store.open_color_mode_dialog(NewSpriteMode::Rgba);
        p.hero.store.set_color_mode_target(NewSpriteMode::Indexed);
        p.hero.store.set_color_mode_dither(d);
        p.hero.store.request_color_mode();
        let _ = p.quadro();
        assert_eq!(p.sprite().mode(), ColorMode::Indexed, "{d:?}");
        let plano: Vec<u32> = (0..h as i32)
            .flat_map(|y| (0..w as i32).map(move |x| (x, y)))
            .map(|(x, y)| p.indice(x, y))
            .collect();
        planos.push((d, plano));
        assert!(p.app.undo(p.key));
        assert_eq!(p.imagem(), lisa, "{d:?}: o Ctrl+Z volta à imagem lisa");
    }
    let plano = |d: PixelDither| &planos.iter().find(|(x, _)| *x == d).expect("dither").1;
    // As células que diferem entre dois planos, em ‰.
    let diferem = |a: PixelDither, b: PixelDither| {
        plano(a)
            .iter()
            .zip(plano(b))
            .filter(|(x, y)| x != y)
            .count()
            * 1000
            / plano(a).len()
    };
    let usados: BTreeSet<_> = plano(PixelDither::None).iter().collect();
    assert!(
        usados.len() >= 8,
        "sem dither: faixas de {} cores",
        usados.len()
    );
    // Medido 25/10 (‰, arredondado à dezena): None×Floyd 380 · None×Ord4 240 · Ord2×Ord4 100 · Ord4×Ord8 20 ·
    // Ord4×Old4 180 · Old4×Old8 10.
    for (a, b, min) in [
        (PixelDither::None, PixelDither::FloydSteinberg, 100),
        (PixelDither::None, PixelDither::Ordered4, 100),
        (PixelDither::Ordered2, PixelDither::Ordered4, 50),
        (PixelDither::Ordered4, PixelDither::Ordered8, 5),
        (PixelDither::Ordered4, PixelDither::Old4, 50),
        (PixelDither::Old4, PixelDither::Old8, 3),
    ] {
        let d = diferem(a, b);
        assert!(d >= min, "{a:?} × {b:?}: só {d} ‰ das células diferem");
    }
    // Grayscale: a imagem cinza.
    p.hero.store.open_color_mode_dialog(NewSpriteMode::Rgba);
    p.hero.store.set_color_mode_target(NewSpriteMode::Gray);
    p.hero.store.request_color_mode();
    let _ = p.quadro();
    assert_eq!(p.sprite().mode(), ColorMode::Gray);
    let cinza = p.imagem();
    assert!(
        cinza
            .as_chunks::<4>()
            .0
            .iter()
            .all(|c| c[0] == c[1] && c[1] == c[2])
    );
    let tons: BTreeSet<u8> = cinza.as_chunks::<4>().0.iter().map(|c| c[0]).collect();
    assert!(
        tons.len() > 32,
        "um degradê cinza, não faixas: {}",
        tons.len()
    );
}

/// As cenas das fotos: a 5 abre o diálogo (Indexed + Floyd–Steinberg) no 1.º quadro em Draw, a 6
/// já vem convertida com Ordered 4×4 como UM passo (o `Ctrl+Z` volta ao RGBA).
#[test]
fn scenes_5_and_6_stage_the_color_mode_photos() {
    use ph2d_editor_core::interaction::ContextMenuKind;
    let p = Palco::cena(5);
    assert_eq!(
        p.hero.store.context_menu().map(|r| r.kind),
        Some(ContextMenuKind::PixelColorModeDialog)
    );
    let c = p.hero.store.color_mode_choice();
    assert_eq!(
        (c.target, c.dither, p.hero.store.color_mode_current()),
        (
            NewSpriteMode::Indexed,
            PixelDither::FloydSteinberg,
            NewSpriteMode::Rgba
        )
    );
    let mut p = Palco::cena(6);
    assert_eq!(p.sprite().mode(), ColorMode::Indexed);
    assert!(p.app.undo(p.key));
    assert_eq!(p.sprite().mode(), ColorMode::Rgba);
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

/// ⭐ Arrastar uma amostra (ou a selecção) para outra célula reordena a paleta como UM passo, a
/// imagem composta fica igual ao byte, e a selecção segue as cores.
#[test]
fn dragging_a_swatch_reorders_the_palette_as_one_step_and_keeps_the_image() {
    let mut p = Palco::novo();
    let (img, pal, antes) = (p.imagem(), p.sprite().palette().to_vec(), p.passos());
    // A rampa da bola (4..=7) largada sobre a célula 20: o último dela cai lá.
    p.app.select(BALL_RAMP);
    p.pede(PixelRequest::DragSwatch { from: 4, to: 20 });
    assert_eq!(p.passos(), antes + 1, "um passo do desfazer");
    assert_eq!(p.imagem(), img, "a imagem não muda");
    let s = p.sprite().palette().to_vec();
    assert_eq!(&s[17..=20], &pal[4..=7], "o bloco junto, pela ordem dele");
    assert_eq!(s[4], pal[8], "o que estava depois desliza");
    assert_eq!(p.app.selection(), (17..=20).collect::<Vec<_>>());
    assert_eq!(p.app.fg, pal[4], "a amostra arrastada é a cor de frente");
    assert!(p.app.undo(p.key));
    assert_eq!(p.sprite().palette(), pal.as_slice());

    // Uma amostra fora da selecção move-se sozinha, para trás.
    p.app.select([1]);
    p.pede(PixelRequest::DragSwatch { from: 12, to: 2 });
    assert_eq!(p.sprite().palette()[2], pal[12]);
    assert_eq!(p.sprite().palette()[3], pal[2]);
    assert_eq!(p.app.selection(), vec![2]);
    assert_eq!(p.imagem(), img);
    assert_eq!(p.passos(), antes + 1);
}

#[test]
fn drag_permutation_lands_like_a_single_swatch_would() {
    use ph2d_app_pixel::paleta::drag_permutation;
    use ph2d_pixel_core::palette::move_permutation;
    for (from, to) in [(0, 5), (5, 0), (3, 3), (7, 9), (9, 0)] {
        assert_eq!(drag_permutation(10, &[from], to), move_permutation(10, from, to));
    }
    assert_eq!(drag_permutation(6, &[1, 3], 5), vec![0, 2, 4, 5, 1, 3]);
    assert_eq!(drag_permutation(6, &[3, 4], 0), vec![3, 4, 0, 1, 2, 5]);
    assert_eq!(drag_permutation(4, &[1], 9), vec![0, 1, 2, 3], "fora da paleta: nada");
}

/// Uma pasta só deste teste (nunca a HOME real).
fn pasta(nome: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("ph2d-pixel-{nome}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// ⭐ *Save Palette as Default*: um Pixel Sprite NOVO nasce com ela (mesmo um Indexed de fundo
/// branco, que a procura nela), e o *Load Default Palette* carrega-a.
#[test]
fn save_as_default_gives_the_next_new_sprite_that_palette() {
    let mut p = Palco::novo();
    let dir = pasta("omissao");
    p.app.paletas = Some(dir.clone());
    p.pede(PixelRequest::PaletteSize(5));
    let cinco = p.sprite().palette().to_vec();
    p.pede(PixelRequest::SaveDefault);
    assert!(dir.join(ph2d_app_pixel::paleta::DEFAULT_FILE).exists());
    assert_eq!(p.app.paleta_de_omissao(), cinco);

    let spec = ph2d_pixel_core::NewSprite {
        mode: ColorMode::Indexed,
        background: ph2d_pixel_core::Background::White,
        ..Default::default()
    };
    let k = p.app.create(spec).expect("sprite");
    let s = &p.app.doc(k).expect("doc").sprite;
    assert_eq!(s.palette(), cinco.as_slice(), "o sprite novo nasce com ela");
    let mut img = Vec::new();
    compose_rgba(s, s.frames()[0].id, None, &mut img);
    let px = [img[0], img[1], img[2], img[3]];
    assert!(cinco.contains(&px), "o fundo procurou-se na paleta dela: {px:?}");

    p.pede(PixelRequest::PaletteSize(9));
    p.pede(PixelRequest::DefaultPalette);
    assert_eq!(p.sprite().palette(), cinco.as_slice(), "o Load Default carrega a gravada");

    // Sem pasta, a de fábrica.
    p.app.paletas = None;
    let k = p.app.create(ph2d_pixel_core::NewSprite::default()).expect("sprite");
    assert_eq!(
        p.app.doc(k).expect("doc").sprite.palette(),
        ph2d_pixel_core::palette::DEFAULT.as_slice()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// *Save Palette as Preset* grava na pasta (o diálogo abre lá), a lista do painel mostra-a e
/// carregá-la é UM passo, como o *Load Palette*.
#[test]
fn a_saved_preset_is_listed_and_loads_as_one_step() {
    let mut p = Palco::novo();
    let dir = pasta("presets");
    p.app.paletas = Some(dir.clone());
    assert!(p.quadro().presets.is_empty());
    let pal = p.sprite().palette().to_vec();
    ph2d_app_host::modal::responde_o_proximo(dir.join("bola.gpl"));
    p.pede(PixelRequest::SavePreset);
    p.pede(PixelRequest::SaveDefault);
    assert_eq!(p.quadro().presets, vec!["bola".to_owned()], "a de omissão não é uma predefinição");
    p.pede(PixelRequest::PaletteSize(3));
    let antes = p.passos();
    p.pede(PixelRequest::LoadPreset("bola".to_owned()));
    assert_eq!(p.sprite().palette(), pal.as_slice());
    assert_eq!(p.passos(), antes + 1);
    let _ = std::fs::remove_dir_all(&dir);
}
