//! ⭐ **DE FORMA PARA ÁRVORE** — o despacho que dá a cada [`Primitive`] a fórmula dela.
//!
//! # Por que ele saiu do `lib.rs`
//!
//! O `lib.rs` desta crate responde por *como uma ÁRVORE se compila* (a dobra sobre a arena, as
//! poses, os modificadores, as regiões); este responde por *qual fórmula cada FORMA usa*. A W106
//! acrescentou catorze primitivas e o arquivo chegou ao tecto de **700** do gate de LOC.
//!
//! ⚠️ **Partir para irmão, nunca uma entrada na allowlist.**
//!
//! ⚠️ **É aqui que o `f32` do documento vira `f64` da árvore**, num sítio só. O documento guarda
//! `f32` (é o que uma cena ECS transporta) e a avaliação corre em `f64`; uma conversão espalhada
//! por catorze sítios seria catorze oportunidades de alguém guardar o resultado errado.

use fidget::context::Tree;
use ph2d_field::Primitive;

use crate::{ops, ops_plates, ops_solids, profile};

pub(crate) fn primitive(p: &Primitive) -> Tree {
    match *p {
        Primitive::Box {
            half,
            round,
            chamfer,
        } => ops::sd_box(
            [f64::from(half[0]), f64::from(half[1]), f64::from(half[2])],
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Sphere { radius } => ops::sd_sphere(f64::from(radius)),
        Primitive::Cylinder {
            radius,
            half_height,
            round,
            chamfer,
        } => ops::sd_cylinder(
            f64::from(radius),
            f64::from(half_height),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Torus { major, minor } => ops::sd_torus(f64::from(major), f64::from(minor)),
        Primitive::Extrude {
            ref profile,
            half_height,
            round,
            chamfer,
        } => profile::sd_extrude(
            profile,
            f64::from(half_height),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Revolve { ref profile } => profile::sd_revolve(profile),
        Primitive::Cone {
            bottom,
            top,
            half_height,
            round,
            chamfer,
        } => ops::sd_cone(
            f64::from(bottom),
            f64::from(top),
            f64::from(half_height),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Capsule {
            radius,
            half_height,
        } => ops::sd_capsule(f64::from(radius), f64::from(half_height)),
        Primitive::Prism {
            sides,
            bottom,
            top,
            half_height,
            round,
            chamfer,
        } => ops::sd_prism(
            sides,
            f64::from(bottom),
            f64::from(top),
            f64::from(half_height),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Wedge {
            half,
            round,
            chamfer,
        } => ops::sd_wedge(half.map(f64::from), f64::from(round), f64::from(chamfer)),
        Primitive::TorusArc {
            major,
            minor,
            angle,
            round,
            chamfer,
        } => ops::sd_torus_arc(
            f64::from(major),
            f64::from(minor),
            f64::from(angle),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Star {
            points,
            outer,
            inner,
            half_height,
            round,
            chamfer,
        } => ops::sd_star(
            points,
            f64::from(outer),
            f64::from(inner),
            f64::from(half_height),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::BoxFrame {
            half,
            thickness,
            round,
            chamfer,
        } => ops::sd_box_frame(
            half.map(f64::from),
            f64::from(thickness),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Ellipsoid { radii } => ops::sd_ellipsoid(radii.map(f64::from)),
        // ─────────────────────────── W106 ───────────────────────────
        // ⚠️ **Tudo entra em `f64`** — o documento guarda `f32` e a árvore é avaliada em `f64`; a
        // conversão vive aqui, num sítio, como para todas as outras.
        Primitive::Octahedron {
            radius,
            round,
            chamfer,
        } => ops_solids::sd_octahedron(f64::from(radius), f64::from(round), f64::from(chamfer)),
        Primitive::RoundCone {
            bottom,
            top,
            half_height,
        } => ops_solids::sd_round_cone(f64::from(bottom), f64::from(top), f64::from(half_height)),
        Primitive::CutSphere {
            radius,
            cut,
            round,
            chamfer,
        } => ops_solids::sd_cut_sphere(
            f64::from(radius),
            f64::from(cut),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::HollowDome {
            radius,
            cut,
            thickness,
            round,
            chamfer,
        } => ops_solids::sd_cut_hollow_sphere(
            f64::from(radius),
            f64::from(cut),
            f64::from(thickness),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Link {
            major,
            minor,
            length,
        } => ops_solids::sd_link(f64::from(major), f64::from(minor), f64::from(length)),
        Primitive::SolidAngle {
            radius,
            angle,
            round,
            chamfer,
        } => ops_solids::sd_solid_angle(
            f64::from(radius),
            f64::from(angle),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Gear {
            teeth,
            root,
            outer,
            tooth,
            half_height,
            round,
            chamfer,
        } => ops_plates::sd_gear(
            teeth,
            f64::from(root),
            f64::from(outer),
            f64::from(tooth),
            f64::from(half_height),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Cross {
            arm,
            width,
            half_height,
            round,
            chamfer,
        } => ops_plates::sd_cross(
            f64::from(arm),
            f64::from(width),
            f64::from(half_height),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Heart {
            size,
            half_height,
            round,
            chamfer,
        } => ops_plates::sd_heart(
            f64::from(size),
            f64::from(half_height),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Moon {
            radius,
            bite,
            offset,
            half_height,
            round,
            chamfer,
        } => ops_plates::sd_moon(
            f64::from(radius),
            f64::from(bite),
            f64::from(offset),
            f64::from(half_height),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Drop {
            radius,
            height,
            half_height,
            round,
            chamfer,
        } => ops_plates::sd_drop(
            f64::from(radius),
            f64::from(height),
            f64::from(half_height),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Pie {
            radius,
            angle,
            half_height,
            round,
            chamfer,
        } => ops_plates::sd_pie(
            f64::from(radius),
            f64::from(angle),
            f64::from(half_height),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Trapezoid {
            bottom,
            top,
            half_width,
            half_height,
            round,
            chamfer,
        } => ops_plates::sd_trapezoid(
            f64::from(bottom),
            f64::from(top),
            f64::from(half_width),
            f64::from(half_height),
            f64::from(round),
            f64::from(chamfer),
        ),
        Primitive::Vesica {
            radius,
            offset,
            half_height,
            round,
            chamfer,
        } => ops_plates::sd_vesica(
            f64::from(radius),
            f64::from(offset),
            f64::from(half_height),
            f64::from(round),
            f64::from(chamfer),
        ),
    }
}
