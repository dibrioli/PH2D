//! **P8: a ferramenta Text pelo caminho real** — `T` escolhe-a, o clique abre a caixa na célula,
//! teclar só pré-visualiza, o 1.º Enter SOLTA o texto numa moldura (P8b: arrasta, roda, escala) e o
//! 2.º grava UM passo com as células que o rasterizador prevê, Esc não grava, e com a caixa aberta as
//! letras não trocam de ferramenta.

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, Press};
use ph2d_app_pixel::{PixelApp, Tool, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{PixelEditAction, PixelRequest};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_render::Camera2d;
use winit::keyboard::KeyCode;

use crate::comum::{at, born, canvas, value};

const CLEAR: [u8; 4] = [0, 0, 0, 0];
const RED: [u8; 4] = [255, 0, 0, 255];

/// Um pedido do painel pelo MESMO caminho da shell ([`panel::frame`], com o sprite em Draw).
fn pede(app: &mut PixelApp, sim: &SimWorld, bits: u64, r: PixelRequest) {
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.mode.enter(bits, ObjectMode::Draw);
    app.pede(r);
    panel::frame(app, sim.world(), &mut hero, &mut ToastQueue::new());
}

fn chave(sim: &SimWorld, bits: u64) -> u32 {
    input::key_of(sim.world(), bits).expect("marcador")
}

fn click(app: &mut PixelApp, sim: &SimWorld, cam: &Camera2d, bits: u64, c: (i32, i32)) {
    let p = at(app, sim, cam, bits, c.0, c.1);
    let m = Mods::default();
    assert_eq!(
        input::press(app, &canvas(sim, cam, bits), p, Button::Primary, m),
        Press::Done
    );
    input::release(app);
}

/// Uma tecla pelo MESMO caminho da shell: primeiro a caixa, depois os atalhos.
fn tecla(app: &mut PixelApp, sim: &SimWorld, bits: u64, code: KeyCode, text: Option<&str>) {
    let m = Mods::default();
    if input::text_key(app, sim.world(), Some(bits), code, m, text).is_none() {
        input::key_with(app, sim.world(), Some(bits), code, m);
    }
}

fn escreve(app: &mut PixelApp, sim: &SimWorld, bits: u64, s: &str) {
    for ch in s.chars() {
        let mut b = [0u8; 4];
        tecla(app, sim, bits, KeyCode::KeyA, Some(ch.encode_utf8(&mut b)));
    }
}

/// As células que o rasterizador prevê para `s` com a âncora `a`, na cor de frente.
fn previsto(app: &PixelApp, sim: &SimWorld, bits: u64, s: &str, a: (i32, i32)) -> Vec<(i32, i32)> {
    let d = app.doc(chave(sim, bits)).expect("doc");
    let style = ph2d_app_pixel::texto::estilo(&app.options);
    let v = ph2d_app_pixel::texto::valores(&d.sprite, (d.layer, d.frame), s, &style, a, app.fg);
    v.into_iter().map(|(c, _)| c).collect()
}

fn mostrado(app: &mut PixelApp, sim: &SimWorld, bits: u64, c: (i32, i32)) -> [u8; 4] {
    app.shown_rgba(chave(sim, bits), c.0, c.1).expect("célula")
}

#[test]
fn t_picks_the_text_tool() {
    let (mut app, sim, _cam, bits) = born();
    tecla(&mut app, &sim, bits, KeyCode::KeyT, None);
    assert_eq!(app.tool, Tool::Text);
}

/// Um quadro do painel (o `Enter`/`Esc` da moldura solta é um pedido que ele aplica).
fn quadro(app: &mut PixelApp, sim: &SimWorld, bits: u64) {
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.mode.enter(bits, ObjectMode::Draw);
    panel::frame(app, sim.world(), &mut hero, &mut ToastQueue::new());
}

/// Escreve «Ag» em (3, 2) a vermelho e dá o 1.º Enter: as células previstas e o sprite de antes.
fn solto(app: &mut PixelApp, sim: &SimWorld, cam: &Camera2d, bits: u64) -> Vec<(i32, i32)> {
    app.tool = Tool::Text;
    app.fg = RED;
    click(app, sim, cam, bits, (3, 2));
    escreve(app, sim, bits, "Ag");
    let celulas = previsto(app, sim, bits, "Ag", (3, 2));
    tecla(app, sim, bits, KeyCode::Enter, None);
    quadro(app, sim, bits);
    assert!(
        app.text_box().is_none() && app.floating(),
        "o 1.º Enter solta"
    );
    celulas
}

#[test]
fn enter_floats_the_text_without_touching_the_document() {
    let (mut app, sim, cam, bits) = born();
    let celulas = solto(&mut app, &sim, &cam, bits);
    for &c in &celulas {
        assert_eq!(value(&app, &sim, bits, c.0, c.1), CLEAR, "solto não grava");
        assert_eq!(mostrado(&mut app, &sim, bits, c), RED, "e mostra {c:?}");
    }
    assert!(!app.undo(chave(&sim, bits)), "nenhum passo");
}

/// P8c: as formigas do texto solto são SÓ o retângulo da moldura — o contorno de cada letra
/// (a máscara da cobertura) tapava as próprias letras (foto `smoke_P8_cena19_solto`).
#[test]
fn the_floating_text_ants_are_only_the_frame_rectangle() {
    let (mut app, sim, cam, bits) = born();
    let celulas = solto(&mut app, &sim, &cam, bits);
    let mk = app.floating_ants(chave(&sim, bits)).expect("formigas");
    let x0 = celulas.iter().map(|c| c.0).min().expect("tinta");
    let x1 = celulas.iter().map(|c| c.0).max().expect("tinta");
    let y0 = celulas.iter().map(|c| c.1).min().expect("tinta");
    let y1 = celulas.iter().map(|c| c.1).max().expect("tinta");
    let r = mk.bounds().expect("não vazia");
    assert!(
        r.x <= x0 && r.y <= y0 && r.x + r.w as i32 > x1 && r.y + r.h as i32 > y1,
        "o retângulo cobre a tinta: {r:?}"
    );
    assert_eq!(
        mk.cells().count(),
        (r.w * r.h) as usize,
        "um retângulo cheio, não o contorno das letras"
    );
    assert!(mk.cells().count() > celulas.len(), "mais que as letras");
}

#[test]
fn moving_the_floating_text_and_enter_stamps_it_there_in_one_step() {
    let (mut app, sim, cam, bits) = born();
    let celulas = solto(&mut app, &sim, &cam, bits);
    let n = celulas.len() as i32;
    let meio = (
        celulas.iter().map(|c| c.0).sum::<i32>() / n,
        celulas.iter().map(|c| c.1).sum::<i32>() / n,
    );
    let (dx, dy) = (5, 3);
    let c = canvas(&sim, &cam, bits);
    let de = at(&app, &sim, &cam, bits, meio.0, meio.1);
    let para = at(&app, &sim, &cam, bits, meio.0 + dx, meio.1 + dy);
    assert_eq!(
        input::press(&mut app, &c, de, Button::Primary, Mods::default()),
        Press::Done
    );
    assert!(input::moved(&mut app, &c, para, Mods::default()));
    assert!(input::release(&mut app));
    assert!(app.floating(), "arrastar não assenta");
    tecla(&mut app, &sim, bits, KeyCode::Enter, None);
    quadro(&mut app, &sim, bits);
    assert!(!app.floating());
    for &(x, y) in &celulas {
        assert_eq!(
            value(&app, &sim, bits, x + dx, y + dy),
            RED,
            "gravou no sítio novo"
        );
    }
    let fora = celulas
        .iter()
        .filter(|&&(x, y)| !celulas.contains(&(x - dx, y - dy)));
    for &(x, y) in fora {
        assert_eq!(
            value(&app, &sim, bits, x, y),
            CLEAR,
            "a âncora velha fica limpa"
        );
    }
    assert!(app.undo(chave(&sim, bits)), "um passo");
    for &(x, y) in &celulas {
        assert_eq!(value(&app, &sim, bits, x + dx, y + dy), CLEAR);
    }
    assert!(!app.undo(chave(&sim, bits)), "só um passo");
}

#[test]
fn escape_on_the_floating_text_cancels_everything() {
    let (mut app, sim, cam, bits) = born();
    let celulas = solto(&mut app, &sim, &cam, bits);
    tecla(&mut app, &sim, bits, KeyCode::Escape, None);
    quadro(&mut app, &sim, bits);
    assert!(!app.floating() && app.text_box().is_none());
    for &c in &celulas {
        assert_eq!(value(&app, &sim, bits, c.0, c.1), CLEAR);
        assert_eq!(mostrado(&mut app, &sim, bits, c), CLEAR);
    }
    assert!(!app.undo(chave(&sim, bits)), "nenhum passo");
}

#[test]
fn typing_previews_and_two_enters_stamp_one_undo_step_at_the_anchor() {
    let (mut app, sim, cam, bits) = born();
    app.tool = Tool::Text;
    app.fg = RED;
    click(&mut app, &sim, &cam, bits, (3, 2));
    let (k, a, t) = app.text_box().expect("a caixa abriu");
    assert_eq!((k, a, t), (chave(&sim, bits), (3, 2), ""));
    escreve(&mut app, &sim, bits, "Ag");
    assert_eq!(app.text_box().map(|b| b.2), Some("Ag"));
    let celulas = previsto(&app, &sim, bits, "Ag", (3, 2));
    assert!(!celulas.is_empty());
    // A tinta não começa acima nem à esquerda da âncora mais do que a fonte recua.
    assert!(celulas.iter().all(|&(_, y)| y >= 2));
    for &c in &celulas {
        assert_eq!(value(&app, &sim, bits, c.0, c.1), CLEAR, "só pré-visualiza");
        assert_eq!(
            mostrado(&mut app, &sim, bits, c),
            RED,
            "o canvas mostra {c:?}"
        );
    }
    tecla(&mut app, &sim, bits, KeyCode::Enter, None);
    assert!(app.text_box().is_none());
    tecla(&mut app, &sim, bits, KeyCode::Enter, None);
    quadro(&mut app, &sim, bits);
    for &c in &celulas {
        assert_eq!(value(&app, &sim, bits, c.0, c.1), RED, "gravou {c:?}");
    }
    assert!(app.undo(chave(&sim, bits)), "um passo");
    for &c in &celulas {
        assert_eq!(value(&app, &sim, bits, c.0, c.1), CLEAR);
    }
    assert!(!app.undo(chave(&sim, bits)), "só um passo");
}

#[test]
fn escape_closes_the_box_without_writing() {
    let (mut app, sim, cam, bits) = born();
    app.tool = Tool::Text;
    app.fg = RED;
    click(&mut app, &sim, &cam, bits, (3, 2));
    escreve(&mut app, &sim, bits, "Ag");
    let celulas = previsto(&app, &sim, bits, "Ag", (3, 2));
    tecla(&mut app, &sim, bits, KeyCode::Escape, None);
    assert!(app.text_box().is_none());
    for &c in &celulas {
        assert_eq!(value(&app, &sim, bits, c.0, c.1), CLEAR);
        assert_eq!(mostrado(&mut app, &sim, bits, c), CLEAR);
    }
    assert!(!app.undo(chave(&sim, bits)), "nenhum passo");
}

#[test]
fn letters_with_the_box_open_are_text_not_shortcuts() {
    let (mut app, sim, cam, bits) = born();
    app.tool = Tool::Text;
    click(&mut app, &sim, &cam, bits, (1, 1));
    tecla(&mut app, &sim, bits, KeyCode::KeyB, Some("b"));
    tecla(&mut app, &sim, bits, KeyCode::KeyR, Some("r"));
    tecla(&mut app, &sim, bits, KeyCode::KeyT, Some("t"));
    assert_eq!(app.tool, Tool::Text);
    assert_eq!(app.text_box().map(|b| b.2), Some("brt"));
    tecla(&mut app, &sim, bits, KeyCode::Backspace, None);
    tecla(&mut app, &sim, bits, KeyCode::ArrowLeft, None);
    tecla(&mut app, &sim, bits, KeyCode::KeyX, Some("x"));
    assert_eq!(app.text_box().map(|b| b.2), Some("bxr"));
    tecla(&mut app, &sim, bits, KeyCode::Escape, None);
    tecla(&mut app, &sim, bits, KeyCode::KeyB, None);
    assert_eq!(app.tool, Tool::Pencil, "fechada, o B volta a ser o lápis");
}

#[test]
fn edit_insert_text_opens_the_box_in_the_middle() {
    let (mut app, sim, _cam, bits) = born();
    let k = chave(&sim, bits);
    pede(
        &mut app,
        &sim,
        bits,
        PixelRequest::Edit(PixelEditAction::InsertText),
    );
    let d = app.doc(k).expect("doc");
    let meio = (d.sprite.width() as i32 / 2, d.sprite.height() as i32 / 2);
    assert_eq!(app.tool, Tool::Text);
    assert_eq!(app.text_box().map(|b| b.1), Some(meio));
}

#[test]
fn changing_the_size_with_the_box_open_redraws_the_preview() {
    let (mut app, sim, cam, bits) = born();
    app.tool = Tool::Text;
    app.fg = RED;
    click(&mut app, &sim, &cam, bits, (1, 1));
    escreve(&mut app, &sim, bits, "A");
    let pequeno = previsto(&app, &sim, bits, "A", (1, 1));
    let mut o = app.options;
    o.text_px = 32;
    pede(&mut app, &sim, bits, PixelRequest::Options(Box::new(o)));
    let grande = previsto(&app, &sim, bits, "A", (1, 1));
    assert!(grande.len() > pequeno.len());
    for &c in &grande {
        assert_eq!(
            mostrado(&mut app, &sim, bits, c),
            RED,
            "o preview cresceu {c:?}"
        );
    }
}

#[test]
fn an_outline_font_draws_and_previous_wraps_to_the_last_font() {
    let (mut app, sim, cam, bits) = born();
    let todas = ph2d_app_pixel::texto::fontes();
    let mut o = app.options;
    o.text_font = u16::MAX;
    pede(&mut app, &sim, bits, PixelRequest::Options(Box::new(o)));
    assert_eq!(
        usize::from(app.options.text_font),
        todas.len() - 1,
        "dá a volta"
    );
    // A última é a VT323, um contorno de 16 px: o preview acende células.
    let i = todas.len() - 1;
    let mut o = app.options;
    o.text_font = u16::try_from(i).unwrap_or(0);
    o.text_px = 16;
    pede(&mut app, &sim, bits, PixelRequest::Options(Box::new(o)));
    app.tool = Tool::Text;
    app.fg = RED;
    click(&mut app, &sim, &cam, bits, (1, 1));
    escreve(&mut app, &sim, bits, "A");
    let celulas = previsto(&app, &sim, bits, "A", (1, 1));
    assert!(!celulas.is_empty(), "{}", todas[i]);
    assert!(
        celulas
            .iter()
            .any(|&c| mostrado(&mut app, &sim, bits, c) != CLEAR)
    );
}

#[test]
fn the_box_frame_and_caret_follow_the_text() {
    let (mut app, sim, cam, bits) = born();
    app.tool = Tool::Text;
    click(&mut app, &sim, &cam, bits, (2, 3));
    let ((x, y, w0, h0), (cx0, cy0, _)) = app.text_box_frame().expect("moldura");
    assert_eq!((x, y, w0, cx0, cy0), (2, 3, 0, 2, 3));
    escreve(&mut app, &sim, bits, "Ag");
    let ((_, _, w1, h1), (cx1, _, alto)) = app.text_box_frame().expect("moldura");
    assert!(w1 > 0 && h1 >= h0 && alto > 0);
    assert!(cx1 > cx0, "o cursor anda com o texto");
    tecla(&mut app, &sim, bits, KeyCode::Enter, Some("\r"));
    assert!(app.text_box_frame().is_none());
    tecla(&mut app, &sim, bits, KeyCode::Escape, None);
    quadro(&mut app, &sim, bits);
    escreve_nova_linha(&mut app, &sim, &cam, bits);
}

fn escreve_nova_linha(app: &mut PixelApp, sim: &SimWorld, cam: &Camera2d, bits: u64) {
    click(app, sim, cam, bits, (2, 3));
    escreve(app, sim, bits, "A");
    let shift = Mods {
        shift: true,
        ..Mods::default()
    };
    assert!(input::text_key(app, sim.world(), Some(bits), KeyCode::Enter, shift, None).is_some());
    let ((_, _, _, h), (cx, cy, alto)) = app.text_box_frame().expect("moldura");
    assert_eq!((cx, cy), (2, 3 + alto), "Shift+Enter abre uma linha");
    assert_eq!(h, (2 * alto) as u32);
}
