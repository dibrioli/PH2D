//! **The contact overlay** — where two bodies touch, and how hard they are pressing.
//!
//! A collider is invisible, a joint is a relationship, and a contact is *less* than
//! either: it exists only while two shapes meet, and nothing on screen says whether
//! two objects are resting on each other or merely overlapping in the artist's eye.
//! This is the visible half of the readback, and — like the sensor's magenta (W7) —
//! it is the whole reason the channel is worth having before a gameplay consumer
//! exists.
//!
//! ## The mark says two things at once
//!
//! **Where** — a small cross at the deepest contact point.
//! **How hard** — its arms grow with the load the pair is carrying, so a stack shows
//! a visible gradient: the bottom joint is the biggest mark on screen because it is
//! holding everything above it. That is not decoration; it is the same 4 : 3 : 2 : 1
//! the wrapper's gate pins, drawn.
//!
//! ⚠️ It is **not** an impact flash. The load is read after `step` returns, and by
//! then the solver has already absorbed the impact — measured, a ball landing from
//! 6 m reports the same number as one sitting still. Sizing the mark by "impact
//! strength" would be sizing it by a number that never gets big.

use ph2d_physics_ecs::BodyContact;
use ph2d_render::Camera2d;
use ph2d_vector::{BezPath, Point};

use ph2d_host::WindowSize;

/// Contacts — **white**, the one value no collider (green / cyan / purple / magenta),
/// joint (amber) or field (yellow / orange) uses. A touch is an event between two
/// things, so it belongs to neither's colour.
pub(super) const CONTACT_RGBA: [f32; 4] = [1.0, 1.0, 1.0, 0.85]; // LITERAL-COLOR-OK: overlay de contato

/// Arm length of an unloaded mark, screen px — chrome, constant like the outline
/// stroke, so a contact stays findable at any zoom.
const MARK_MIN_PX: f64 = 3.0; // LITERAL-PX-OK: chrome de overlay

/// Arm length of a fully-loaded mark, screen px.
const MARK_MAX_PX: f64 = 9.0; // LITERAL-PX-OK: chrome de overlay

/// The load, in N·s, at which a mark reaches [`MARK_MAX_PX`].
///
/// **Measured, not chosen**: a 0.5 m box of density 1 resting on the floor reports
/// ~0.0128 N·s, and a stack of four reports ~0.0511 at the bottom. `0.05` therefore
/// puts an ordinary resting body near the small end and a heavily-loaded joint at the
/// top of the range, which is the spread that makes the gradient readable. It is a
/// display ruler and nothing reads it back — a scene of much heavier bodies saturates,
/// which is honest: past a point "very loaded" is the useful reading.
const LOAD_FULL_NS: f32 = 0.05;

/// One contact mark, in screen pixels: a cross at the contact point whose arms grow
/// with the load.
///
/// Screen space on purpose (the module's rule): the POINT goes through the camera, so
/// the mark sits on the contact at any zoom, but the arms are a constant-size screen
/// ornament rather than a world length that would balloon.
fn mark(contact: &BodyContact, camera: &Camera2d, window: WindowSize) -> BezPath {
    let (sx, sy) = camera.world_to_screen(contact.point, window);
    let centre = Point::new(f64::from(sx), f64::from(sy));
    let t = f64::from((contact.impulse / LOAD_FULL_NS).clamp(0.0, 1.0));
    let arm = MARK_MIN_PX + t * (MARK_MAX_PX - MARK_MIN_PX);

    let mut path = BezPath::new();
    // A cross, not a dot: two objects resting flush produce contacts millimetres
    // apart, and a filled dot at that size is a smudge while two crossing lines
    // still read as "here, and here".
    path.move_to(Point::new(centre.x - arm, centre.y));
    path.line_to(Point::new(centre.x + arm, centre.y));
    path.move_to(Point::new(centre.x, centre.y - arm));
    path.line_to(Point::new(centre.x, centre.y + arm));
    path
}

/// Every contact mark to draw, or nothing when the overlay is off.
///
/// Pure and returned as data, like its siblings: a refusal that lives inside a paint
/// loop is not a refusal, and "did the toggle actually stop it" is then a question a
/// gate can ask.
pub(super) fn contact_marks(
    show: bool,
    contacts: &[BodyContact],
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<BezPath> {
    if !show {
        return Vec::new();
    }
    contacts.iter().map(|c| mark(c, camera, window)).collect()
}

/// A linha d'água — **ciano claro**, a cor da água, e o único traço do overlay que
/// descreve um LUGAR em vez de um corpo.
pub(super) const WATERLINE_RGBA: [f32; 4] = [0.45, 0.85, 1.0, 0.75]; // LITERAL-COLOR-OK: overlay de linha d'agua

/// Quanto a linha se estende além da poça, em px de tela — uma sobra pequena, para que
/// a superfície leia como uma superfície e não como a borda de cima do retângulo.
const WATERLINE_OVERHANG_PX: f64 = 6.0; // LITERAL-PX-OK: chrome de overlay

/// Os segmentos de linha d'água a desenhar, em pixels de tela, ou nada com o overlay
/// desligado.
///
/// ⚠️ A geometria vem PRONTA da física (`PhysicsBridge::waterlines`, que sai da mesma
/// `surface_level` do empuxo) — esta função só a projeta. Recalcular "o topo da poça"
/// aqui seria a segunda resposta para *onde está a água*, e ela discordaria da primeira
/// numa poça rotacionada, que é exatamente onde ninguém confere.
pub(super) fn waterline_marks(
    show: bool,
    lines: &[([f32; 2], [f32; 2])],
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<BezPath> {
    if !show {
        return Vec::new();
    }
    lines
        .iter()
        .map(|(a, b)| {
            let to_screen = |w: &[f32; 2]| {
                let (x, y) = camera.world_to_screen(*w, window);
                Point::new(f64::from(x), f64::from(y))
            };
            let (p, q) = (to_screen(a), to_screen(b));
            // A sobra é aplicada em ESPAÇO DE TELA, como toda ornamentação deste módulo:
            // em mundo ela mudaria de tamanho com o zoom.
            let (dx, dy) = (q.x - p.x, q.y - p.y);
            let len = dx.hypot(dy);
            let (ux, uy) = if len > 1e-6 {
                (dx / len, dy / len)
            } else {
                (0.0, 0.0)
            };
            let mut path = BezPath::new();
            path.move_to(Point::new(
                p.x - ux * WATERLINE_OVERHANG_PX,
                p.y - uy * WATERLINE_OVERHANG_PX,
            ));
            path.line_to(Point::new(
                q.x + ux * WATERLINE_OVERHANG_PX,
                q.y + uy * WATERLINE_OVERHANG_PX,
            ));
            path
        })
        .collect()
}

#[cfg(test)]
#[path = "physics_overlay_contacts_tests.rs"]
mod tests;
