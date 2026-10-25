//! **A roda em Draw** (filho de [`super`] pelo tecto de LOC): a escada de zoom inteiro, o resto da
//! roda que ainda não virou degrau e o alinhamento das células a píxeis inteiros do ecrã.

use ph2d_host::WindowSize;
use ph2d_pixel_render::{px_per_cell, zoom};
use ph2d_render::Camera2d;
use ph2d_vector::Point;

use super::{key_of, to_screen};
use crate::PixelApp;
use crate::state::Doc;

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
pub(super) fn snap(
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
