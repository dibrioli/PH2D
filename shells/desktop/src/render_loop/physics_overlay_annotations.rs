//! **As ANOTAÇÕES do overlay de física** — as marcas desenhadas POR CIMA do contorno.
//!
//! Um contorno diz *onde está o collider*; estas dizem o que ele não consegue mostrar: a
//! seta amarela de um lançamento armado (que o corpo parado em t=0 não revela), a seta
//! laranja de uma zona de força (*para que lado sopra?*) e o glifo violeta de uma zona de
//! torque (*para que lado gira?*). Todas são construídas em espaço de TELA e em pixels
//! constantes — uma força não é um comprimento, um torque não tem "longe".
//!
//! Separadas do [`super::physics_overlay`] pelo cap de 600 LOC do shell (a wave do frame,
//! W-AreaFrame, levou o arquivo a 605), pelo mesmo corte de responsabilidade que já separa
//! os gates: contorno de um lado, anotação do outro. Os gates destas vivem em
//! `physics_overlay_annotation_tests.rs`.

use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::{BezPath, Point};

/// Chords in the torque glyph's arc — enough that 270° reads as a smooth curve.
const TORQUE_ARC_SEGS: u32 = 24;

/// The **initial-velocity arrow** — a launch the artist armed but cannot see,
/// because a still body at t=0 is not moving. Yellow, a hue no collider or joint
/// uses. Only linear velocity is drawn (an angular spin has no direction to point).
pub(super) const VELOCITY_RGBA: [f32; 4] = [0.96, 0.92, 0.26, 0.95]; // LITERAL-COLOR-OK: overlay de velocidade inicial

/// How far ahead the arrow reaches, in seconds of travel: the tip is where the
/// body would be a quarter-second after launch. A time, not a raw length, so the
/// arrow scales with the world like the collider does — faster is longer.
const ARROW_SECONDS: f32 = 0.25;

/// Arrowhead length, screen px — chrome, constant size like the outline stroke.
const ARROW_HEAD_PX: f64 = 9.0; // LITERAL-PX-OK: chrome de overlay

/// An arrow for one **velocity** at one world point, in screen pixels, or `None` if
/// it is (near) zero — a still body draws no arrow.
///
/// Built in screen space on purpose (the module's rule — see the header): the
/// shaft is the world vector projected through the camera, so its length
/// tracks speed and zoom, but the arrowhead is a constant-size screen ornament,
/// so it never balloons the way a world-space stroke width would.
///
/// Two callers: the authored launch, and the force zone below — which converts its
/// newtons into the velocity they would give a reference body, so both arrows are
/// read against the SAME ruler instead of inventing a second one.
pub(super) fn velocity_arrow(
    cx: f32,
    cy: f32,
    linvel: [f32; 2],
    camera: &Camera2d,
    window: WindowSize,
) -> Option<BezPath> {
    if linvel[0].hypot(linvel[1]) < 1e-3 {
        return None;
    }
    let to_screen = |wx: f32, wy: f32| {
        let (sx, sy) = camera.world_to_screen([wx, wy], window);
        Point::new(f64::from(sx), f64::from(sy))
    };
    let tail = to_screen(cx, cy);
    let tip = to_screen(
        cx + linvel[0] * ARROW_SECONDS,
        cy + linvel[1] * ARROW_SECONDS,
    );
    let (dx, dy) = (tip.x - tail.x, tip.y - tail.y);
    let len = dx.hypot(dy);

    let mut path = BezPath::new();
    path.move_to(tail);
    path.line_to(tip);
    if len > 1e-6 {
        // Head: two barbs from the tip, back along the shaft rotated ±25°.
        let (bx, by) = (-dx / len, -dy / len);
        let (s, c) = (25.0_f64).to_radians().sin_cos();
        for sign in [1.0, -1.0] {
            let (rx, ry) = (bx * c - by * (sign * s), bx * (sign * s) + by * c);
            path.move_to(tip);
            path.line_to(Point::new(
                tip.x + rx * ARROW_HEAD_PX,
                tip.y + ry * ARROW_HEAD_PX,
            ));
        }
    }
    Some(path)
}

/// The force-zone arrow — **orange**, a hue no collider, joint or launch uses.
pub(super) const EFFECTOR_RGBA: [f32; 4] = [1.0, 0.62, 0.16, 0.95]; // LITERAL-COLOR-OK: overlay de campo de forca

/// The mass a force is drawn AGAINST, in kg.
///
/// A force is not a length, so any arrow for it is a statement about SOME body — that
/// is what "resisted by mass" means, and it is the feature, not a wart. One kilogram
/// is the honest reference: at 1 kg the newtons ARE the acceleration, so the arrow
/// reads directly off the number in the row.
const EFFECTOR_REFERENCE_KG: f32 = 1.0;

/// The arrow for one force zone's push. `None` for a zone that pushes nothing.
///
/// The force is converted into the velocity it would give the reference body in
/// [`ARROW_SECONDS`], and then drawn by the SAME function the launch arrow uses —
/// so the two are read against one ruler. A second length scale would be a second
/// answer to "how long is a strong one".
pub(super) fn effector_arrow(
    cx: f32,
    cy: f32,
    force: [f32; 2],
    camera: &Camera2d,
    window: WindowSize,
) -> Option<BezPath> {
    let k = ARROW_SECONDS / EFFECTOR_REFERENCE_KG;
    velocity_arrow(cx, cy, [force[0] * k, force[1] * k], camera, window)
}

/// O anel do **falloff** — o mesmo laranja da seta de força, apagado: ele não é um objeto
/// novo, é a *mesma* força vista a meio caminho da borda.
pub(super) const FALLOFF_RGBA: [f32; 4] = [1.0, 0.62, 0.16, 0.45]; // LITERAL-COLOR-OK: overlay de falloff

/// A fração da zona onde o anel do falloff é desenhado — **metade**.
///
/// Não é um número escolhido por gosto: a régua do falloff
/// (`ShapeDesc::radial_fraction`) é invariante sob mapas lineares, então o conjunto
/// `t = 0.5` **é** a própria silhueta encolhida à metade em torno do centro. O anel é
/// portanto a curva de nível exata, desenhada pela MESMA `collider_outline` que traça o
/// contorno — nenhuma geometria segunda, e nada para divergir do que o solver mede.
///
/// Ele anuncia que o empurrão DESVANECE e mostra onde fica o meio do caminho; o *quanto*
/// é a row `Falloff` da §11. Um anel cuja posição codificasse o valor teria de sumir para
/// falloff < 0.5 (onde a força nunca chega à metade), que é pior que uma marca fixa.
pub(super) const FALLOFF_RING: f32 = 0.5;

/// The torque-zone glyph — **violet**, a hue no collider, joint, launch or force uses.
pub(super) const TORQUE_RGBA: [f32; 4] = [0.66, 0.44, 0.98, 0.95]; // LITERAL-COLOR-OK: overlay de torque de area

/// Radius of the spin glyph, in screen pixels — a constant-size ornament like the
/// arrowhead, so it reads the same at any zoom (a torque is not a length, so nothing
/// about it scales with the world).
const TORQUE_GLYPH_PX: f64 = 16.0; // LITERAL-PX-OK: chrome de overlay

/// The spin glyph for one torque zone — a 270° arc with an arrowhead on the end,
/// **which way does this area spin, and how hard**. `None` for a zone that spins nothing.
///
/// The SIGN is the direction: `> 0` is rapier's positive (world counter-clockwise)
/// angular convention, `< 0` clockwise — so the artist sees the whirlpool's handedness,
/// exactly as the force arrow shows the wind's direction. It is a screen ornament, drawn
/// at the zone's center in constant pixels: a torque is not a length, and a spin has no
/// "far". The arc is a polyline of [`TORQUE_ARC_SEGS`] chords; the arrowhead is two barbs
/// off the terminal, tangent to the sweep.
///
/// ⚠️ The arc is built on a screen BASIS derived from the camera — `û` = where world +x
/// lands on screen, `ŵ` = world +y — not on hand-flipped screen angles. So a positive
/// (world-CCW) torque draws the direction a body would VISIBLY turn under this camera,
/// whatever y-flip or rotation it carries: the glyph cannot disagree with the sim it
/// annotates the way a hard-coded "CCW = up-left" would the moment the view rotated.
pub(super) fn torque_glyph(
    cx: f32,
    cy: f32,
    torque: f32,
    camera: &Camera2d,
    window: WindowSize,
) -> Option<BezPath> {
    if torque == 0.0 {
        return None;
    }
    let to_screen = |wx: f32, wy: f32| {
        let (sx, sy) = camera.world_to_screen([wx, wy], window);
        (f64::from(sx), f64::from(sy))
    };
    // The screen basis: û is world +x on screen, ŵ is world +y, each normalised so the
    // arc is a true circle of TORQUE_GLYPH_PX radius. A world-angle `ang` then lands at
    // `centre + R·(cos·û + sin·ŵ)`, and increasing `ang` traces world-CCW ON SCREEN.
    let (c0x, c0y) = to_screen(cx, cy);
    let (px, py) = to_screen(cx + 1.0, cy);
    let (qx, qy) = to_screen(cx, cy + 1.0);
    let norm = |vx: f64, vy: f64| {
        let l = vx.hypot(vy);
        if l < 1e-9 {
            (0.0, 0.0)
        } else {
            (vx / l, vy / l)
        }
    };
    let (ux, uy) = norm(px - c0x, py - c0y);
    let (wx, wy) = norm(qx - c0x, qy - c0y);
    let at = |ang: f64| {
        Point::new(
            c0x + TORQUE_GLYPH_PX * (ang.cos() * ux + ang.sin() * wx),
            c0y + TORQUE_GLYPH_PX * (ang.cos() * uy + ang.sin() * wy),
        )
    };
    // Positive torque sweeps in the +angle (world-CCW) direction; negative flips it. The
    // 270° gap sits opposite the arrowhead so the mouth of the arc is a stable, readable
    // notch.
    let dir = if torque > 0.0 { 1.0_f64 } else { -1.0 };
    let span = 270.0_f64.to_radians();
    let a0 = 135.0_f64.to_radians();
    let mut path = BezPath::new();
    for i in 0..=TORQUE_ARC_SEGS {
        let t = f64::from(i) / f64::from(TORQUE_ARC_SEGS);
        let p = at(a0 + dir * span * t);
        if i == 0 {
            path.move_to(p);
        } else {
            path.line_to(p);
        }
    }
    // Arrowhead at the terminal, tangent to the sweep — computed as the chord of the last
    // segment so the barbs cannot disagree with the drawn curve.
    let tip = at(a0 + dir * span);
    let prev = at(a0 + dir * span * (f64::from(TORQUE_ARC_SEGS - 1) / f64::from(TORQUE_ARC_SEGS)));
    let (dx, dy) = (tip.x - prev.x, tip.y - prev.y);
    let len = dx.hypot(dy);
    if len > 1e-6 {
        // Two barbs from the tip, back along the tangent rotated ±30°.
        let (bx, by) = (-dx / len, -dy / len);
        let (s, c) = 30.0_f64.to_radians().sin_cos();
        for sign in [1.0, -1.0] {
            let (rx, ry) = (bx * c - by * (sign * s), bx * (sign * s) + by * c);
            path.move_to(tip);
            path.line_to(Point::new(
                tip.x + rx * ARROW_HEAD_PX,
                tip.y + ry * ARROW_HEAD_PX,
            ));
        }
    }
    Some(path)
}
