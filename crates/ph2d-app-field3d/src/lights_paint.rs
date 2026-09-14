//! ⭐⭐⭐ **A MARCA DE UMA LUZ NO CANVAS** (report do dono, 2026-09-14: *«a luz não tem seu próprio
//! gizmo»*).
//!
//! # Porque ela é obrigatória, e não decoração
//!
//! Uma luz não tem campo: o clique que marcha a peça **não a encontra**, e a wave anterior deixou-a
//! alcançável só pela Hierarquia. ⇒ *um objecto 3D que não se pode apontar no sítio onde ele está
//! não é um objecto 3D* — e era exactamente essa a metade que faltava à ordem do dono.
//!
//! ⚠️ **Onde ela é desenhada e onde ela é apanhada saem da MESMA função**
//! ([`crate::lights::marks`]). Este módulo não projecta nada: ele recebe as marcas prontas. *Uma
//! marca desenhada num sítio e apanhada noutro lê-se como «o clique não pega», e nenhum dos dois
//! lados o diagnostica sozinho.*
//!
//! # ⚠️ A cor do disco é CONTEÚDO, e a do resto é CHROME
//!
//! O miolo é pintado com a **cor da própria lâmpada** — é o que faz uma luz azul ser reconhecível
//! sem a escolher —, e essa cor vem do documento, como a amostra da linha do painel. O anel e os
//! raios são [`ColorToken`]s, como todo o resto do gizmo (HR-15).

use ph2d_tokens::{Color, ColorToken, Theme};
use ph2d_vector::{
    Affine, BezPath, Brush, Circle, Color as VelloColor, Point, Shape as _, VectorScene,
};

use crate::lights::{MARK_HALF_PX, Mark, RAY_IN_PX, RAY_OUT_PX};

/// Quantos raios a marca tem à volta.
///
/// ⚠️ **Oito, e o número não é decorativo:** é o que faz a marca ler-se como *luz* e não como um
/// ponto de pivô — a mesma silhueta que todo modelador desenha. ⛔ Menos de seis lê-se como uma
/// estrela de selecção; mais de doze vira um disco a esta escala.
const RAYS: usize = 8;

/// A espessura de um raio e do anel, em pixels.
///
/// ⚠️ **É a da haste do gizmo da casa** — a mesma razão do [`MARK_HALF_PX`]: dois chromes com
/// espessuras diferentes na mesma janela leem-se como dois sistemas.
const STROKE_HALF_PX: f32 = crate::gizmo::SHAFT_HALF_W_PX;

/// A opacidade de uma marca APAGADA.
///
/// ⚠️ **Ela não desaparece**, e é a mesma lei que o dono já deu sobre as linhas do painel (§23):
/// *«não devem desaparecer, mas apenas serem inativados, mas sempre visíveis»*. Uma luz apagada cuja
/// marca sumisse só se voltaria a acender pela Hierarquia — que é de onde esta wave a tirou.
const OFF_ALPHA: f64 = 0.35;

/// ⭐ **Pinta as marcas.** `selected` são os bits do que está escolhido — a marca dele leva a tinta
/// de selecção, como todo o resto deste app.
pub fn paint(
    scene: &mut VectorScene,
    marks: &[Mark],
    selected: Option<u64>,
    theme: Theme,
    origin: [f32; 2],
) {
    let at = Affine::translate((f64::from(origin[0]), f64::from(origin[1])));
    for m in marks {
        let escolhida = selected == Some(m.light.bits);
        let chrome = if escolhida {
            ColorToken::Accent
        } else {
            ColorToken::Text1
        }
        .resolve(theme);
        let alpha = if m.light.on { 1.0 } else { OFF_ALPHA };
        // ⭐ **A cor da lâmpada, pela MESMA porta que a amostra do painel** — o documento guarda
        // linear, e o ecrã fala sRGB. ⛔ Uma segunda travessia aqui seria a curva escrita duas vezes.
        let srgb = crate::materials::colour_srgb8(m.light.color());
        let miolo = Color {
            r: srgb[0],
            g: srgb[1],
            b: srgb[2],
            a: 255,
        };
        for k in 0..RAYS {
            let a = std::f32::consts::TAU * k as f32 / RAYS as f32;
            let (c, s) = (a.cos(), a.sin());
            fita(
                scene,
                [m.px[0] + c * RAY_IN_PX, m.px[1] + s * RAY_IN_PX],
                [m.px[0] + c * RAY_OUT_PX, m.px[1] + s * RAY_OUT_PX],
                fade(chrome, alpha),
                at,
            );
        }
        disco(scene, m.px, MARK_HALF_PX, fade(miolo, alpha), at);
        anel(scene, m.px, MARK_HALF_PX, fade(chrome, alpha), at);
    }
}

/// A mesma lei do `gizmo_paint::fade` — uma fracção da opacidade que a cor já tem.
fn fade(c: Color, a: f64) -> Color {
    Color {
        a: (f64::from(c.a) * a).round() as u8,
        ..c
    }
}

fn brush(c: Color) -> Brush {
    Brush::Solid(VelloColor::from_rgba8(c.r, c.g, c.b, c.a))
}

/// Um segmento com espessura — a mesma primitiva (e a mesma ausência de junta) do gizmo.
fn fita(scene: &mut VectorScene, a: [f32; 2], b: [f32; 2], c: Color, at: Affine) {
    let d = [b[0] - a[0], b[1] - a[1]];
    let len = d[0].hypot(d[1]);
    if len <= f32::EPSILON {
        return;
    }
    let n = [-d[1] / len * STROKE_HALF_PX, d[0] / len * STROKE_HALF_PX];
    let q = [
        [a[0] + n[0], a[1] + n[1]],
        [b[0] + n[0], b[1] + n[1]],
        [b[0] - n[0], b[1] - n[1]],
        [a[0] - n[0], a[1] - n[1]],
    ];
    let mut p = BezPath::new();
    p.move_to(Point::new(f64::from(q[0][0]), f64::from(q[0][1])));
    for v in &q[1..] {
        p.line_to(Point::new(f64::from(v[0]), f64::from(v[1])));
    }
    p.close_path();
    scene.fill_path(&p, &brush(c), at);
}

fn disco(scene: &mut VectorScene, centro: [f32; 2], r: f32, c: Color, at: Affine) {
    let p = Circle::new(
        Point::new(f64::from(centro[0]), f64::from(centro[1])),
        f64::from(r),
    )
    .to_path(0.1);
    scene.fill_path(&p, &brush(c), at);
}

/// Um anel, pela regra par-ímpar — a mesma construção do `gizmo_paint::ring`.
fn anel(scene: &mut VectorScene, centro: [f32; 2], r: f32, c: Color, at: Affine) {
    let ctr = Point::new(f64::from(centro[0]), f64::from(centro[1]));
    let mut p = Circle::new(ctr, f64::from(r + STROKE_HALF_PX)).to_path(0.1);
    // ⚠️ O furo sai da regra NonZero com o sentido CONTRÁRIO — ver o `ring` do gizmo.
    let dentro = Circle::new(ctr, f64::from(r - STROKE_HALF_PX)).to_path(0.1);
    p.extend(dentro.reverse_subpaths());
    scene.fill_path(&p, &brush(c), at);
}

#[cfg(test)]
#[path = "lights_paint_tests.rs"]
mod tests;
