//! **A cena 7 — as camadas que SE VÊEM** (P3). Um Pixel Sprite RGBA 190 × 96 em Draw:
//!
//! - em cima, o degradê da cena 4 na **Background** e, por cima dele, o grupo recolhido
//!   **«19 modes»** com dezanove faixas de 10 colunas — a MESMA tinta (laranja em cima, azul em
//!   baixo, com um cinza a meio) em cada uma, cada faixa na sua camada e com o seu modo, na ordem do
//!   menu do Aseprite: a diferença entre os modos lê-se lado a lado (o teste prova que as 19 faixas
//!   compostas são duas a duas diferentes);
//! - em baixo, da esquerda para a direita: um quadrado branco a **50 %** de opacidade, uma estrela
//!   numa camada **escondida** (o olho acende-a), uma camada **trancada** (pintar nela recusa e diz
//!   porquê), uma **referência** (vê-se; o Flatten deixa-a de fora) e um **grupo** aberto com duas
//!   formas (o olho do grupo apaga as duas).
//!
//! A camada activa é a do quadrado a 50 % (o campo Opacity mostra 128).

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::blend::BlendMode;
use ph2d_pixel_core::layers::{self, Place};
use ph2d_pixel_core::{ColorMode, LayerId, LayerKind, NewSprite, Op, Patch, Rect, Sprite};

use crate::PixelApp;

/// O tamanho da cena 7.
pub const SIZE: (u32, u32) = (190, 96);
/// A largura de cada faixa de modo e a altura da banda de cima.
pub const STRIPE_W: u32 = 10;
pub const BAND_H: u32 = 48;

/// A tinta das faixas na linha `y` da banda: laranja → cinza → azul, opaca.
#[must_use]
pub fn stripe_rgba(y: u32) -> [u8; 4] {
    let t = y as f32 / (BAND_H - 1) as f32;
    let (a, m, b) = ([255.0, 170.0, 60.0], [128.0, 128.0, 128.0], [40.0, 90.0, 230.0]);
    let mix = |p: [f32; 3], q: [f32; 3], u: f32| [0, 1, 2].map(|k| p[k] + (q[k] - p[k]) * u);
    let c = if t < 0.5 {
        mix(a, m, t * 2.0)
    } else {
        mix(m, b, t * 2.0 - 1.0)
    };
    [c[0].round() as u8, c[1].round() as u8, c[2].round() as u8, 255]
}

/// O retângulo da faixa do modo `i` (0 = Normal … 18 = Divide).
#[must_use]
pub fn stripe_rect(i: usize) -> Rect {
    Rect::new(i as i32 * STRIPE_W as i32, 0, STRIPE_W, BAND_H)
}

/// As peças de baixo: (nome, retângulo).
pub const OPACITY_BOX: Rect = Rect::new(4, 54, 30, 36);
pub const HIDDEN_STAR: Rect = Rect::new(44, 54, 30, 36);
pub const LOCKED_BOX: Rect = Rect::new(84, 54, 30, 36);
pub const REFERENCE_BOX: Rect = Rect::new(124, 54, 28, 36);
pub const GROUP_A: Rect = Rect::new(158, 54, 14, 20);
pub const GROUP_B: Rect = Rect::new(166, 66, 20, 24);

fn put(s: &mut Sprite, key: u32, app: &mut PixelApp, layer: LayerId, r: Rect, f: impl Fn(i32, i32) -> [u8; 4]) -> Option<()> {
    let mut pixels = Vec::with_capacity(r.area() * 4);
    for y in r.y..r.y + r.h as i32 {
        for x in r.x..r.x + r.w as i32 {
            pixels.extend_from_slice(&s.value_of_rgba(f(x, y)).to_le_bytes());
        }
    }
    let frame = s.frames()[0].id;
    app.commit(key, Op::Patch(Patch { layer, frame, bbox: Some(r), rect: r, pixels })).ok()?;
    *s = app.docs.get(&key)?.sprite.clone();
    Some(())
}

/// Uma camada nova acima de `above` com o nome dado; devolve o id.
fn nova(app: &mut PixelApp, key: u32, above: LayerId, name: &str, group: bool) -> Option<LayerId> {
    let doc = app.docs.get_mut(&key)?;
    let (op, id) = if group {
        layers::new_group(&mut doc.sprite, above)?
    } else {
        layers::new_layer(&mut doc.sprite, above)?
    };
    app.commit(key, op).ok()?;
    let s = &app.docs.get(&key)?.sprite;
    let op = layers::edit(s, id, |l| name.clone_into(&mut l.name))?;
    app.commit(key, op).ok()?;
    Some(id)
}

fn edita(app: &mut PixelApp, key: u32, id: LayerId, f: impl FnOnce(&mut ph2d_pixel_core::Layer)) -> Option<()> {
    // sem mudança (o Normal de uma camada que já é Normal) não há passo
    match layers::edit(&app.docs.get(&key)?.sprite, id, f) {
        Some(op) => app.commit(key, op).ok(),
        None => Some(()),
    }
}

/// Põe `id` no topo do grupo `g` (nada a fazer se já é filha dele: nasceu acima de uma irmã).
fn entra(app: &mut PixelApp, key: u32, id: LayerId, g: LayerId) -> Option<()> {
    let s = &app.docs.get(&key)?.sprite;
    if s.layer(id)?.parent == Some(g) {
        return Some(());
    }
    let op = layers::move_to(s, id, Place::Into(g))?;
    app.commit(key, op).ok()
}

/// A estrela de cinco pontas inscrita em `r` (dentro = amarela).
fn estrela(r: Rect, x: i32, y: i32) -> [u8; 4] {
    let (cx, cy) = (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0);
    let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
    let raio = (dx * dx + dy * dy).sqrt();
    let a = dy.atan2(dx) + std::f32::consts::FRAC_PI_2;
    let k = (a * 5.0 / std::f32::consts::TAU).rem_euclid(1.0);
    let borda = r.w.min(r.h) as f32 / 2.0 * (0.45 + 0.55 * (1.0 - (k - 0.5).abs() * 2.0));
    if raio <= borda { [250, 220, 40, 255] } else { [0; 4] }
}

pub(crate) fn stage(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u32> {
    let (w, h) = SIZE;
    let spec = NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Rgba,
        background: ph2d_pixel_core::Background::White,
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let mut s = app.docs.get(&key)?.sprite.clone();
    let bg = s.layers()[0].id;
    let (gw, gh) = crate::smoke::GRADIENT_SIZE;
    put(&mut s, key, app, bg, Rect::new(0, 0, w, BAND_H), |x, y| {
        crate::smoke::gradient_rgba(x as u32 * gw / w, y as u32 * gh / BAND_H)
    })?;
    let grupo = nova(app, key, bg, "19 modes", true)?;
    let mut abaixo = grupo;
    for (i, m) in BlendMode::ALL.iter().enumerate() {
        let nome = ph2d_i18n::tr(crate::camadas::blend_of(*m).label_key());
        let id = nova(app, key, abaixo, nome, false)?;
        entra(app, key, id, grupo)?;
        s = app.docs.get(&key)?.sprite.clone();
        put(&mut s, key, app, id, stripe_rect(i), |_, y| stripe_rgba(y as u32))?;
        edita(app, key, id, |l| l.blend = *m)?;
        abaixo = id;
    }
    edita(app, key, grupo, |l| l.kind = LayerKind::Group { collapsed: true })?;
    // a banda de baixo
    let opa = nova(app, key, grupo, "Opacity 50%", false)?;
    put(&mut s, key, app, opa, OPACITY_BOX, |_, _| [255, 255, 255, 255])?;
    edita(app, key, opa, |l| l.opacity = 128)?;
    let esc = nova(app, key, opa, "Hidden star", false)?;
    put(&mut s, key, app, esc, HIDDEN_STAR, |x, y| estrela(HIDDEN_STAR, x, y))?;
    edita(app, key, esc, |l| l.visible = false)?;
    let tr = nova(app, key, esc, "Locked", false)?;
    put(&mut s, key, app, tr, LOCKED_BOX, |x, y| {
        if (x + y) % 6 < 3 { [200, 60, 60, 255] } else { [120, 30, 30, 255] }
    })?;
    edita(app, key, tr, |l| l.locked = true)?;
    let rf = nova(app, key, tr, "Reference", false)?;
    put(&mut s, key, app, rf, REFERENCE_BOX, |_, y| {
        let v = (y - REFERENCE_BOX.y) as u8 * 5;
        [60 + v, 200 - v, 255, 255]
    })?;
    edita(app, key, rf, |l| l.reference = true)?;
    let g2 = nova(app, key, rf, "Group", true)?;
    let ga = nova(app, key, g2, "Circle", false)?;
    entra(app, key, ga, g2)?;
    put(&mut s, key, app, ga, GROUP_A, |x, y| {
        let (dx, dy) = (x - 165, y - 64);
        if dx * dx + dy * dy <= 49 { [40, 200, 90, 255] } else { [0; 4] }
    })?;
    let gb = nova(app, key, ga, "Square", false)?;
    entra(app, key, gb, g2)?;
    put(&mut s, key, app, gb, GROUP_B, |_, _| [240, 140, 30, 255])?;
    let doc = app.docs.get_mut(&key)?;
    doc.layer = opa;
    app.tool = crate::Tool::Pencil;
    app.fg = [0, 0, 0, 255];
    app.bg = [255, 255, 255, 255];
    Some(key)
}
