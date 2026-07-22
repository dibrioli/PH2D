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
//! ⚠️ The standing cross's size is **load**, and only load. *How hard the hit was*
//! is a separate reading on a separate mark — the begin-flash (below), whose size
//! rides the captured **impact peak** (W-ImpactForce). The two are genuinely
//! different numbers: measured, a ball landing from 6 m settles to the same load as
//! one sitting still, while its impact peak is ~200× larger. Keeping them on
//! different marks is what lets a gentle touch on a heavy stack read differently
//! from a hard slam onto bare floor.

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

/// The flash — the same white, at full opacity, because it is the same vocabulary
/// saying "this one, now".
pub(super) const CONTACT_FLASH_RGBA: [f32; 4] = [1.0, 1.0, 1.0, 1.0]; // LITERAL-COLOR-OK: flash de contato

/// How many ticks a begin-flash lives.
///
/// **A display ruler, not a knob** (the plan's rule: a number the artist cannot
/// calibrate against their art does not become surface). At the 60 Hz tick this is
/// ~100 ms — long enough to be caught reliably, and short enough that a busy stack
/// does not stay lit. One or two ticks reads as a dropped frame rather than an event,
/// which is the failure mode a single-frame flash has.
const FLASH_TICKS: u64 = 6;

/// Arm of the flash at birth and at death, screen px. It EXPANDS as it ages, which is
/// what makes it read as a burst rather than as a second, brighter contact.
///
/// ⚠️ **Derived from [`MARK_MAX_PX`], not chosen next to it** — a gate caught the
/// first version being born at 4 px, *smaller* than the 9 px cross of a fully-loaded
/// contact, so a landing into the most heavily-loaded joint in the scene would have
/// been announced by a mark hidden inside the one it was announcing. The flash has to
/// clear the biggest standing cross in the WORST case, and tying the two together is
/// what keeps that true the day the load ruler moves.
const FLASH_MIN_PX: f64 = MARK_MAX_PX + 2.0; // LITERAL-PX-OK: chrome de overlay
const FLASH_MAX_PX: f64 = FLASH_MIN_PX * 2.0; // LITERAL-PX-OK: chrome de overlay

/// How many extra screen px a full-impact hit adds to the flash's base size.
///
/// This is the visible reading of *how hard* — a hard slam flashes bigger than a
/// gentle touch. Chosen so a full-impact birth (`FLASH_MIN_PX + this` = 23 px)
/// clears a gentle flash at its OLDEST (`FLASH_MAX_PX` = 22 px), so "hard" always
/// out-reads "soft" no matter the age either is caught at.
const FLASH_IMPACT_BOOST_PX: f64 = 12.0; // LITERAL-PX-OK: chrome de overlay

/// The impact peak, in N·s, at which the flash reaches full [`FLASH_IMPACT_BOOST_PX`].
///
/// **Measured, not chosen** (`tests/measure_impact.rs`): the reference ball (r=0.3,
/// density 1) captures impact peaks of 0.58 / 1.13 / 1.59 / 2.18 / 3.89 N·s dropped
/// from 0.6 / 1.2 / 2.0 / 3.4 / 10 m, while its settled LOAD stays flat at 0.014. So
/// `2.0` puts a 1.2 m bounce mid-range and a 3.4 m slam at the top — the spread that
/// makes the impact reading legible. A display ruler: nothing reads it back, and a
/// harder-than-3.4 m hit saturates, which is honest (past a point, "very hard" is the
/// useful reading — the same stance the load ruler takes).
const IMPACT_FULL_NS: f32 = 2.0;

/// The begin-flash of one contact: a DIAGONAL cross, sized by IMPACT and expanding
/// with age.
///
/// ⚠️ **Diagonal, and not a bigger `+`, because upright arm length is already spoken
/// for** — [`mark`] sizes the `+` by the LOAD the pair carries. The flash sizes itself
/// by the IMPACT peak instead (`contact.impact`, not `contact.impulse`), so the two
/// marks answer the two different questions a contact raises: *how much is it holding*
/// and *how hard did it hit*. Rotating 45° keeps them on separate channels: for the
/// few ticks the flash lives, the two crosses read as a spark, and the `+` is left
/// saying what it always said.
///
/// Size composes the two dimensions: a floor of [`FLASH_MIN_PX`] (so even a gentle
/// touch clears the biggest standing load cross — the front-A gate), plus an impact
/// boost, plus an age expansion that makes it read as a burst.
fn flash(contact: &BodyContact, camera: &Camera2d, window: WindowSize) -> Option<BezPath> {
    // `None` age is a pair adopted at a re-baseline — it never *began* as far as the
    // simulation is concerned, so it must not flash. That distinction is the whole
    // reason `age_ticks` is an `Option` and not a number (see `BodyContact`).
    let age = contact.age_ticks?;
    if age >= FLASH_TICKS {
        return None;
    }
    let (sx, sy) = camera.world_to_screen(contact.point, window);
    let centre = Point::new(f64::from(sx), f64::from(sy));
    let impact_frac = f64::from((contact.impact / IMPACT_FULL_NS).clamp(0.0, 1.0));
    let base = FLASH_MIN_PX + impact_frac * FLASH_IMPACT_BOOST_PX;
    let t = age as f64 / FLASH_TICKS as f64;
    let arm = base + t * (FLASH_MAX_PX - FLASH_MIN_PX);
    // 45°: the half-diagonal of a square with this arm.
    let d = arm * std::f64::consts::FRAC_1_SQRT_2;

    let mut path = BezPath::new();
    path.move_to(Point::new(centre.x - d, centre.y - d));
    path.line_to(Point::new(centre.x + d, centre.y + d));
    path.move_to(Point::new(centre.x - d, centre.y + d));
    path.line_to(Point::new(centre.x + d, centre.y - d));
    Some(path)
}

/// Every begin-flash to draw, or nothing when the overlay is off. Pure and returned
/// as data, for the reason [`contact_marks`] documents.
pub(super) fn contact_flashes(
    show: bool,
    contacts: &[BodyContact],
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<BezPath> {
    if !show {
        return Vec::new();
    }
    contacts
        .iter()
        .filter_map(|c| flash(c, camera, window))
        .collect()
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
