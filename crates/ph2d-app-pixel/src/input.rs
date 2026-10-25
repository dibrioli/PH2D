//! **A entrada do modo Draw**: ponteiro (lápis, borracha, conta-gotas), roda (zoom inteiro) e
//! teclado (os atalhos do Aseprite). Cada função devolve `true` quando CONSUMIU o evento — a shell
//! não o passa a mais ninguém.

use ph2d_host::WindowSize;
use ph2d_pixel_core::{Sample, Stroke, pick};
use ph2d_pixel_render::{cell_at, image_to_world, px_per_cell, zoom};
use ph2d_render::Camera2d;
use ph2d_vector::{Affine, Point};
use winit::keyboard::KeyCode;

use crate::state::{Doc, Gesture};
use crate::{PixelApp, Tool};

/// O que a família precisa de ver do viewport.
#[derive(Clone, Copy)]
pub struct Canvas<'a> {
    pub world: &'a ph2d_ecs::World,
    pub camera: &'a Camera2d,
    pub window: WindowSize,
    /// Píxeis por metro do projecto (a régua que põe uma célula no mundo).
    pub ppm: f32,
    /// O objecto em Pixel Sprite ▸ Draw, se há.
    pub editing: Option<u64>,
}

/// O que um premir fez.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// Não é da família (fora de Draw).
    NotMine,
    /// Consumido: pintou, apagou ou leu uma cor.
    Done,
    /// A mão: a shell ancora o pan da câmera neste ponto.
    Pan,
}

/// O botão do ponteiro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Primary,
    Secondary,
}

/// `pixel da imagem → ecrã` do objecto `bits` com o documento `doc`.
#[must_use]
pub fn to_screen(
    world: &ph2d_ecs::World,
    bits: u64,
    doc: &Doc,
    camera: &Camera2d,
    window: WindowSize,
    ppm: f32,
) -> Option<Affine> {
    let entity = ph2d_ecs::Entity::try_from_bits(bits)?;
    let tr = ph2d_ecs::world_transform(world, entity)?;
    let img = image_to_world(
        doc.sprite.width(),
        doc.sprite.height(),
        f64::from(ppm),
        (f64::from(tr.translation.x), f64::from(tr.translation.y)),
        f64::from(tr.rotation),
        (f64::from(tr.scale.x), f64::from(tr.scale.y)),
        (f64::from(tr.skew_x), f64::from(tr.skew_y)),
    );
    Some(camera.world_to_screen_affine(window) * img)
}

/// A chave do documento do objecto `bits`.
#[must_use]
pub fn key_of(world: &ph2d_ecs::World, bits: u64) -> Option<u32> {
    let e = ph2d_ecs::Entity::try_from_bits(bits)?;
    world.get::<ph2d_ecs::PixelSprite>(e).map(|p| p.0)
}

fn cell(app: &PixelApp, c: &Canvas<'_>, bits: u64, key: u32, p: (f32, f32)) -> Option<(i32, i32)> {
    let doc = app.docs.get(&key)?;
    let m = to_screen(c.world, bits, doc, c.camera, c.window, c.ppm)?;
    cell_at(m, (f64::from(p.0), f64::from(p.1)))
}

/// Premir no canvas. Em Draw, todo clique esquerdo/direito é do sprite (não selecciona outros
/// objectos): com o lápis pinta a cor de frente (esquerdo) ou de fundo (direito); a borracha apaga
/// (direito: troca a cor de frente pela de fundo); o conta-gotas (ou `Alt`) lê a cor para a frente
/// (esquerdo) ou o fundo (direito).
pub fn press(
    app: &mut PixelApp,
    c: &Canvas<'_>,
    p: (f32, f32),
    button: Button,
    alt: bool,
) -> Press {
    let Some(bits) = c.editing else {
        return Press::NotMine;
    };
    let Some(key) = key_of(c.world, bits) else {
        return Press::NotMine;
    };
    if app.space || app.tool == Tool::Hand {
        app.panning = true;
        return Press::Pan;
    }
    let Some((x, y)) = cell(app, c, bits, key, p) else {
        return Press::Done;
    };
    let tool = if alt { Tool::Eyedropper } else { app.tool };
    let (fg, bg) = (app.fg, app.bg);
    let stamp = app.stamp();
    let Some(doc) = app.docs.get_mut(&key) else {
        return Press::NotMine;
    };
    let s = &doc.sprite;
    let (layer, frame) = (doc.layer, doc.frame);
    let stroke = match (tool, button) {
        (Tool::Hand, _) => return Press::Pan,
        (Tool::Eyedropper, _) => {
            if let Some(v) = pick(s, frame, x, y, Sample::AllLayers) {
                let rgba = s.rgba_of_value(v);
                match button {
                    Button::Primary => app.fg = rgba,
                    Button::Secondary => app.bg = rgba,
                }
            }
            return Press::Done;
        }
        (Tool::Pencil, Button::Primary) => {
            Stroke::begin(s, layer, frame, s.value_of_rgba(fg), x, y)
        }
        (Tool::Pencil, Button::Secondary) => {
            Stroke::begin(s, layer, frame, s.value_of_rgba(bg), x, y)
        }
        (Tool::Eraser, Button::Primary) => {
            Stroke::begin(s, layer, frame, erase_value(s, layer, bg), x, y)
        }
        (Tool::Eraser, Button::Secondary) => Stroke::begin_replacing(
            s,
            layer,
            frame,
            s.value_of_rgba(fg),
            s.value_of_rgba(bg),
            x,
            y,
        ),
    };
    doc.stamp = stamp;
    app.gesture = Some(Gesture {
        entity: bits,
        key,
        stroke,
    });
    Press::Done
}

/// O que a borracha deixa: transparente; na camada Background, a cor de fundo (Aseprite).
fn erase_value(s: &ph2d_pixel_core::Sprite, layer: ph2d_pixel_core::LayerId, bg: [u8; 4]) -> u32 {
    if s.layer(layer).is_some_and(|l| l.background) {
        s.value_of_rgba(bg)
    } else {
        s.transparent_value()
    }
}

/// Mover. Com um traço em curso, ele continua (consumido); sem traço, só a célula sob o cursor.
pub fn moved(app: &mut PixelApp, c: &Canvas<'_>, p: (f32, f32)) -> bool {
    let Some((bits, key)) = app.gesture.as_ref().map(|g| (g.entity, g.key)) else {
        app.hover = c.editing.and_then(|bits| {
            let key = key_of(c.world, bits)?;
            cell(app, c, bits, key, p).map(|xy| (bits, xy))
        });
        return false;
    };
    let Some((x, y)) = cell(app, c, bits, key, p) else {
        return true;
    };
    app.hover = Some((bits, (x, y)));
    let stamp = app.stamp();
    let (Some(doc), Some(g)) = (app.docs.get_mut(&key), app.gesture.as_mut()) else {
        return true;
    };
    g.stroke.to(&doc.sprite, x, y);
    doc.stamp = stamp;
    true
}

/// Largar a mão: a câmera realinha as arestas das células a píxeis inteiros. `false` = não havia
/// arrasto da mão.
pub fn release_pan(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    camera: &mut Camera2d,
    window: WindowSize,
    ppm: f32,
    editing: Option<u64>,
) -> bool {
    if !std::mem::take(&mut app.panning) {
        return false;
    }
    if let Some(bits) = editing
        && let Some(doc) = key_of(world, bits).and_then(|k| app.docs.get(&k))
    {
        snap(world, bits, doc, camera, window, ppm);
    }
    true
}

/// Largar: o traço inteiro vira UM passo do desfazer do sprite.
pub fn release(app: &mut PixelApp) -> bool {
    let Some(g) = app.gesture.take() else {
        return false;
    };
    let op = app.docs.get(&g.key).and_then(|d| g.stroke.op(&d.sprite));
    match op {
        Some(op) => {
            let _ = app.commit(g.key, op);
        }
        None => {
            // Nada mudou: a imagem volta à do documento.
            let stamp = app.stamp();
            if let Some(d) = app.docs.get_mut(&g.key) {
                d.stamp = stamp;
            }
        }
    }
    true
}

/// O que a roda mandou num evento (positivo = aproxima).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Turn {
    /// Linhas (a roda do rato): um degrau por linha inteira, como no Aseprite.
    Lines(f32),
    /// Píxeis (o trackpad manda o gesto aos bocados): um degrau quando os píxeis somados dão o
    /// factor dele pela lei da roda do viewport ([`Camera2d::wheel_px_for`]) — o gesto aproxima à
    /// MESMA velocidade que fora do Draw, em vez de um degrau por bocado.
    Pixels(f32),
}

/// Um evento da roda: onde está o cursor e quanto virou.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Notch {
    /// O ponto de ecrã que fica parado.
    pub at: (f32, f32),
    pub turn: Turn,
}

/// A roda ainda por virar degrau. Trocar de sentido esquece o resto; o fim da escada também.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Roda {
    linhas: f32,
    px: f32,
}

impl Roda {
    /// Soma `turn` e anda os degraus que ele completa a partir de `from` píxeis por célula: o
    /// degrau final, ou `None` se nenhum se completou.
    pub fn walk(&mut self, turn: Turn, from: f64) -> Option<u32> {
        const EPS: f64 = 1e-6;
        let mut at = from;
        let mut level = None;
        match turn {
            Turn::Lines(y) => {
                if y * self.linhas < 0.0 {
                    self.linhas = 0.0;
                }
                self.linhas += y;
                while self.linhas.abs() >= 1.0 {
                    let next = zoom::step(at, self.linhas > 0.0);
                    self.linhas -= self.linhas.signum();
                    level = Some(next);
                    at = f64::from(next);
                }
            }
            Turn::Pixels(y) => {
                if y * self.px < 0.0 {
                    self.px = 0.0;
                }
                self.px += y;
                loop {
                    let next = zoom::step(at, self.px > 0.0);
                    if (f64::from(next) - at).abs() < EPS {
                        self.px = 0.0;
                        break;
                    }
                    let need = Camera2d::wheel_px_for((at / f64::from(next)) as f32).abs();
                    if need <= 0.0 || self.px.abs() < need {
                        break;
                    }
                    self.px -= need.copysign(self.px);
                    level = Some(next);
                    at = f64::from(next);
                }
            }
        }
        level
    }
}

/// A roda sobre o canvas em Draw: os degraus da escada de zoom inteiro que ela completa, à volta
/// do cursor. `true` = consumida (mesmo sem degrau completo: o resto fica na [`Roda`]).
pub fn wheel(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    camera: &mut Camera2d,
    window: WindowSize,
    ppm: f32,
    editing: Option<u64>,
    notch: Notch,
) -> bool {
    let Some(bits) = editing else { return false };
    let Some(doc) = key_of(world, bits).and_then(|k| app.docs.get(&k)) else {
        return false;
    };
    let Some(m) = to_screen(world, bits, doc, camera, window, ppm) else {
        return false;
    };
    if let Some(target) = app.roda.walk(notch.turn, px_per_cell(m)) {
        set_zoom(world, bits, doc, camera, window, ppm, target, notch.at);
    }
    true
}

/// Põe o zoom em `target` píxeis por célula, com o ponto de ecrã `anchor` parado, e alinha as
/// arestas das células a píxeis inteiros do ecrã.
#[allow(clippy::too_many_arguments)]
pub(crate) fn set_zoom(
    world: &ph2d_ecs::World,
    bits: u64,
    doc: &Doc,
    camera: &mut Camera2d,
    window: WindowSize,
    ppm: f32,
    target: u32,
    anchor: (f32, f32),
) {
    let Some(m) = to_screen(world, bits, doc, camera, window, ppm) else {
        return;
    };
    let now = px_per_cell(m);
    if now <= 0.0 || !now.is_finite() {
        return;
    }
    let before = camera.screen_to_world(anchor, window);
    camera.height_world = (f64::from(camera.height_world) * now / f64::from(target)) as f32;
    let after = camera.screen_to_world(anchor, window);
    camera.center[0] += before[0] - after[0];
    camera.center[1] += before[1] - after[1];
    snap(world, bits, doc, camera, window, ppm);
}

/// Desloca a câmera para a aresta da célula (0,0) cair num píxel inteiro do ecrã.
fn snap(
    world: &ph2d_ecs::World,
    bits: u64,
    doc: &Doc,
    camera: &mut Camera2d,
    window: WindowSize,
    ppm: f32,
) {
    let Some(m) = to_screen(world, bits, doc, camera, window, ppm) else {
        return;
    };
    let p0 = m * Point::new(0.0, 0.0);
    let k = f64::from(window.height.max(1)) / f64::from(camera.height_world).max(1e-9);
    let (dx, dy) = (p0.x - p0.x.round(), p0.y - p0.y.round());
    camera.center[0] += (dx / k) as f32;
    camera.center[1] -= (dy / k) as f32;
}

/// A resposta ao teclado: consumida, e a mensagem a mostrar (se há).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAnswer {
    Quiet,
    Toast(String),
}

/// `Espaço` segurado/solto em Draw — a mão temporária. `true` = consumido.
pub fn space(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    editing: Option<u64>,
    held: bool,
) -> bool {
    if editing.and_then(|b| key_of(world, b)).is_none() {
        app.space = false;
        return false;
    }
    app.space = held;
    true
}

/// As teclas do modo Draw (só ao premir; a shell já descartou o caso de um campo de texto com o
/// foco). `None` = não é desta família.
pub fn key(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    editing: Option<u64>,
    code: KeyCode,
    ctrl: bool,
    shift: bool,
) -> Option<KeyAnswer> {
    let bits = editing?;
    let key = key_of(world, bits)?;
    let tool = |app: &mut PixelApp, t: Tool| {
        app.tool = t;
        KeyAnswer::Toast(ph2d_i18n::tr_with(
            "pixel.tool_switched",
            &[
                ("tool", &ph2d_i18n::tr(t.label_key())),
                ("key", &t.shortcut()),
            ],
        ))
    };
    Some(match (code, ctrl, shift) {
        (KeyCode::KeyB, false, false) => tool(app, Tool::Pencil),
        (KeyCode::KeyE, false, false) => tool(app, Tool::Eraser),
        (KeyCode::KeyI, false, false) => tool(app, Tool::Eyedropper),
        (KeyCode::KeyH, false, false) => tool(app, Tool::Hand),
        (KeyCode::KeyX, false, false) => {
            std::mem::swap(&mut app.fg, &mut app.bg);
            KeyAnswer::Quiet
        }
        (KeyCode::Quote, true, true) => {
            app.grid = !app.grid;
            KeyAnswer::Toast(
                ph2d_i18n::tr(if app.grid {
                    "pixel.grid_on"
                } else {
                    "pixel.grid_off"
                })
                .to_owned(),
            )
        }
        (KeyCode::KeyZ, true, false) => {
            if app.stroking() || app.undo(key) {
                KeyAnswer::Quiet
            } else {
                KeyAnswer::Toast(ph2d_i18n::tr("pixel.nothing_to_undo").to_owned())
            }
        }
        (KeyCode::KeyZ, true, true) | (KeyCode::KeyY, true, false) => {
            if app.stroking() || app.redo(key) {
                KeyAnswer::Quiet
            } else {
                KeyAnswer::Toast(ph2d_i18n::tr("pixel.nothing_to_redo").to_owned())
            }
        }
        _ => return None,
    })
}
