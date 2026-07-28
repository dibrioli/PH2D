//! **O desenho da POLIA** (W-Pulley W1) — a corda na superfície das roldanas e as
//! rodas no tamanho e no ângulo que elas têm.
//!
//! Irmão do [`super::physics_overlay_joints`], separado porque é o único tipo
//! cuja figura não é *duas âncoras e um glifo entre elas*: ela tem uma ROTA, com
//! quantos nós o artista quiser, e cada nó é uma roda com raio próprio. Nasceu do
//! cap de 600 LOC quando o raio chegou.

use ph2d_host::WindowSize;
use ph2d_physics_ecs::JointView;
use ph2d_physics_ecs::rope_route::{self, RopeWheel};
use ph2d_render::Camera2d;
use ph2d_vector::{BezPath, Point};

use super::physics_overlay_joint_glyphs::{ring_px, screen_of};
use super::physics_overlay_joints::JOINT_DOT_PX;

/// **A POLIA:** a corda passa na SUPERFÍCIE de cada roldana, e cada roldana é
/// uma roda desenhada no tamanho que ela tem.
///
/// ⚠️ **A rota é a MESMA função que o solver roda** (`rope_route::route`), não uma
/// re-derivação: a corda desenhada e a corda que segura são a mesma corda, ou o
/// desenho vira uma segunda opinião que ninguém consegue conferir numa
/// screenshot. É a lei que o overlay já segue para as âncoras.
///
/// Antes desta wave a corda ia até o CENTRO da roldana — o pedido (5) do artista
/// — porque uma roldana era um ponto e não tinha superfície onde passar.
///
/// O raio é desenhado no espaço do MUNDO (ele é uma medida da cena e escala com
/// o zoom, ao contrário do anel de uma âncora, que é uma marca de tela).
#[allow(clippy::too_many_arguments)]
pub(super) fn pulley_marks(
    v: &JointView,
    arena: &[RopeWheel],
    spins: &[f32],
    a: Point,
    b: Point,
    camera: &Camera2d,
    window: WindowSize,
    span: &mut BezPath,
    glyph: &mut BezPath,
) {
    let start = v.wheel_start as usize;
    let wheels = arena
        .get(start..start + v.wheel_count as usize)
        .unwrap_or(&[]);
    // As rodas, no tamanho que elas TÊM e no ÂNGULO em que elas estão.
    for (i, w) in wheels.iter().enumerate() {
        let spin = spins.get(start + i).copied().unwrap_or(0.0);
        let (sin, cos) = (libm::sinf(spin), libm::cosf(spin));
        let centre = screen_of(camera, window, w.centre);
        let rim = screen_of(
            camera,
            window,
            [w.centre[0] + w.radius * cos, w.centre[1] + w.radius * sin],
        );
        let r = (rim.x - centre.x).hypot(rim.y - centre.y);
        // Uma roldana de raio zero é um PONTO — modelo legítimo (foi o da v1), e
        // ali o anel de tela é a única coisa que se pode desenhar.
        if r > 1.0 {
            ring_px(centre, r, glyph);
            // **O raio-guia, no ângulo VIVO da roda.** Sem ele uma roda girando é
            // idêntica a uma parada — a mesma lição que o contorno de um collider
            // redondo pagou no W2a —, e sem o ÂNGULO o diâmetro não faria nada
            // visível: é `ω = s/r` que faz a roda grande girar mais devagar.
            glyph.move_to(centre);
            glyph.line_to(rim);
        } else {
            ring_px(centre, JOINT_DOT_PX * 2.0, glyph);
        }
    }
    // A corda. `route` devolve os trechos entre tangentes; os arcos são o que
    // sobra dentro de cada roda, e desenhá-los como corda RETA de tangente a
    // tangente já mostra o que importa (que ela não passa pelo centro).
    let mut segs = Vec::new();
    let ends = (v.anchor_a, v.anchor_b);
    if rope_route::route(ends.0, ends.1, wheels, &mut segs).is_some() {
        span.move_to(a);
        for t in &segs {
            span.line_to(screen_of(camera, window, t.from));
            span.line_to(screen_of(camera, window, t.to));
        }
        span.line_to(b);
    } else {
        // Rota degenerada — a mesma recusa do solver, e ali a corda de fato não
        // está segurando. Uma linha reta diz *há um vínculo aqui* sem afirmar
        // uma geometria que não existe.
        span.move_to(a);
        span.line_to(b);
    }
}
