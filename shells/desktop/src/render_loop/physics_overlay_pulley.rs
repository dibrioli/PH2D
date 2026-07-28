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

use super::physics_overlay_joint_glyphs::{ARC_SEGS, ring_px, screen_of};
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
    // A corda. `route` devolve os trechos entre TANGENTES; entre o trecho que
    // chega e o que sai, a corda **abraça** a roda, e esse pedaço é um arco.
    //
    // ⚠️ Ligar os dois pontos de tangência por uma reta é o que a v1 fazia, com
    // um comentário afirmando que *já mostra o que importa* — e é falso na tela:
    // uma corda entre dois pontos de um círculo é uma CORDA, que atravessa o
    // disco. Nenhum número mudava (o comprimento no solver sempre incluiu o
    // arco), então o defeito só existia no desenho, que é onde o artista olha.
    let mut segs = Vec::new();
    let ends = (v.anchor_a, v.anchor_b);
    if rope_route::route(ends.0, ends.1, wheels, &mut segs).is_some() {
        span.move_to(a);
        for (i, t) in segs.iter().enumerate() {
            span.line_to(screen_of(camera, window, t.from));
            span.line_to(screen_of(camera, window, t.to));
            // O trecho `i` CHEGA na roda `i` e o trecho `i+1` LARGA dela; o que
            // há entre os dois é o enlace. A última perna termina numa âncora,
            // que não tem roda — daí o par de `get`.
            if let (Some(w), Some(next)) = (wheels.get(i), segs.get(i + 1)) {
                arc_on_wheel(w, t.to, t.dir, next.dir, camera, window, span);
            }
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

/// **O enlace:** do ponto onde a corda ENCOSTA na roda até onde ela LARGA,
/// pela superfície.
///
/// ⚠️ **O ângulo vem da MESMA `turn_angle` que o solver usa** para somar o arco
/// ao comprimento da corda — o desenho não re-deriva por que lado ela passa.
/// Uma segunda resposta desenharia um enlace de mais de meia volta pelo lado
/// curto, e a corda vista discordaria da corda que segura.
///
/// O ponto de tangência gira **junto com a direção**: se a corda vira `θ`, o
/// raio que a toca varre `θ`. É por isso que basta rodar o vetor-raio de
/// chegada — não há um segundo ângulo a computar, e o último ponto do arco
/// pousa exatamente onde o trecho seguinte começa.
///
/// Roda de raio zero é o modelo de PONTO da v1 (a âncora de regressão da wave):
/// ali não há superfície a abraçar, e a corda já vira no lugar certo.
fn arc_on_wheel(
    w: &RopeWheel,
    touch: [f32; 2],
    dir_in: [f32; 2],
    dir_out: [f32; 2],
    camera: &Camera2d,
    window: WindowSize,
    span: &mut BezPath,
) {
    if w.radius <= 0.0 {
        return;
    }
    let sweep = f64::from(rope_route::turn_angle(dir_in, dir_out, w.side));
    let step = std::f64::consts::TAU / f64::from(ARC_SEGS);
    let steps = ((sweep.abs() / step).ceil() as u32).max(1);
    let (cx, cy) = (f64::from(w.centre[0]), f64::from(w.centre[1]));
    let (rx, ry) = (f64::from(touch[0]) - cx, f64::from(touch[1]) - cy);
    for k in 1..=steps {
        let th = sweep * f64::from(k) / f64::from(steps);
        let (sin, cos) = libm::sincos(th);
        let p = [
            (cx + rx * cos - ry * sin) as f32,
            (cy + rx * sin + ry * cos) as f32,
        ];
        span.line_to(screen_of(camera, window, p));
    }
}
