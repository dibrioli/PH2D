//! ⭐ **O gizmo 3D de mover** — as três setas, os três quadrados de plano e o disco de vista.
//!
//! Enio, 2026-08-19: *"Não há gzimo 3d para mover os objetos. Precisamos de uma como o do blender."*
//!
//! # O que este arquivo é, e o que ele NÃO é
//!
//! É **lei pura**: projeção, apontar e arrastar, sem `App`, sem ponteiro e sem GPU. Tudo o que aqui
//! entra sai de dois números — a âncora no mundo e a câmera — e por isso todo gesto é gateável sem
//! abrir janela nenhuma. A pintura e a ligação ao ponteiro são os arquivos irmãos
//! ([`crate::field3d_gizmo_paint`], [`crate::field3d_input`]).
//!
//! ⚠️ **Mora no SHELL**, e é a mesma razão do [ADR-0150] que já manda na navegação: uma janela 3D
//! não pode obrigar a mexer no `Tool=12`, que está **congelado**.
//!
//! # ⭐ A projeção é a MESMA do traçador
//!
//! [`ph2d_field_render::Screen`] e [`ph2d_field_render::Orbit::project`] são a conta que a marcha de
//! raios usa para construir os raios. Uma segunda cópia dela aqui divergiria meio pixel, e o sintoma
//! seria uma seta que **agarra ao lado da superfície que ela diz mover** — o tipo de defeito que
//! ninguém chama de bug de projeção. O gate `a_point_projects_where_the_march_actually_hits_it`
//! prende as duas metades.
//!
//! # Os eixos são os do MUNDO
//!
//! Como o default do Blender ("Global"). O nó pode estar rodado — os cilindros da cena 1 estão — e
//! nesse caso mover *ao longo do próprio eixo dele* é uma segunda orientação, que o Blender expõe
//! num seletor. Ela é item ABERTO, e não uma omissão: escolher a orientação é decisão de produto, e
//! entregar só a local seria escolher por quem não pediu.
//!
//! [ADR-0150]: ../../../docs/architecture/decisions/0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md

use ph2d_field_render::{Orbit, Screen};

/// **O comprimento do braço, EM PIXELS** — o gizmo tem tamanho de tela constante, como o do Blender.
///
/// ⚠️ Constante na tela e não no mundo, de propósito: um gizmo de tamanho de mundo fixo fica maior
/// do que a janela ao aproximar e some ao afastar, e é a mesma peça que se está a manipular nos dois
/// casos. O comprimento em mundo sai daqui dividido por [`Screen::px_per_world`].
pub(crate) const ARM_PX: f32 = 90.0;

/// A folga no centro. Nada é desenhado nem apontável dentro dela — é ela que separa as três setas
/// umas das outras e do disco de vista.
pub(crate) const INNER_PX: f32 = 15.0;

/// O raio de agarre: a que distância do traço um clique ainda é daquela alça.
pub(crate) const GRAB_PX: f32 = 9.0;

/// Comprimento e meia-largura da ponta da seta.
pub(crate) const HEAD_PX: f32 = 17.0;
pub(crate) const HEAD_HALF_W_PX: f32 = 5.5;

/// Espessura do traço da haste.
pub(crate) const SHAFT_HALF_W_PX: f32 = 1.3;

/// Onde fica o quadrado de plano, em fração do braço, e o lado dele.
pub(crate) const PLANE_AT: f32 = 0.38;
pub(crate) const PLANE_SIDE: f32 = 0.22;

/// ⚠️ **O comprimento projetado abaixo do qual uma seta deixa de ser uma alça** — e o número é
/// **derivado**, não escolhido.
///
/// Uma seta apontada para o observador projeta-se curta. A partir de certo ponto a região que a
/// agarra deixa de ser distinguível do centro: a haste começa em [`INNER_PX`] e o agarre tem
/// [`GRAB_PX`] de raio dos dois lados, então uma haste mais curta do que `INNER_PX + 2·GRAB_PX`
/// **não tem um único pixel que seja só dela**. Aí ela não é um controle — é uma lotaria entre três.
///
/// Escondê-la é o que o Blender faz, e o efeito colateral é bom: com a seta escondida sobra o
/// quadrado de plano perpendicular a ela, que é exatamente o gesto que aquele enquadramento pede.
pub(crate) const MIN_ARM_PX: f32 = INNER_PX + 2.0 * GRAB_PX;

/// A alça agarrada. `usize` é o índice do eixo: 0 = X, 1 = Y, 2 = Z.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Handle {
    /// Mover ao longo de um eixo.
    Axis(usize),
    /// Mover no plano **perpendicular** a este eixo (o quadrado XY é `Plane(2)`).
    Plane(usize),
    /// Mover no plano da tela.
    View,
}

/// **Onde o gizmo está**, no mundo. Publicado pela ponte com a cena, que é quem tem o mundo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Anchor {
    /// A entidade que ele move — a identidade viaja com a âncora, senão o arrasto teria de a
    /// procurar outra vez e podia achar outra.
    pub(crate) entity: u64,
    pub(crate) origin: [f32; 3],
}

/// A forma de uma alça já projetada — em **pixels**, pronta a pintar e a apontar.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Shape {
    /// Haste (do centro para fora) + ponta.
    Arrow { from: [f32; 2], to: [f32; 2] },
    /// Quadrilátero: os quatro cantos, já projetados.
    Quad([[f32; 2]; 4]),
    /// Disco no centro.
    Disc { center: [f32; 2], radius: f32 },
}

/// Uma alça pronta. `live = false` ⇒ **nem pintada nem apontável** neste enquadramento.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Projected {
    pub(crate) handle: Handle,
    pub(crate) shape: Shape,
    pub(crate) live: bool,
}

/// Os três eixos do mundo.
const WORLD_AXES: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// **Projeta o gizmo inteiro.** A ordem é a de apontar: do centro para fora.
///
/// ⚠️ **A ordem é load-bearing** — [`pick`] devolve a primeira que casa, e o disco de vista está por
/// dentro da folga onde as setas não começam. Sem esta ordem, apontar o centro escolheria um eixo à
/// sorte.
pub(crate) fn project(anchor: Anchor, cam: &Orbit, screen: Screen) -> Vec<Projected> {
    let px_per_world = screen.px_per_world().max(f32::MIN_POSITIVE);
    let arm = ARM_PX / px_per_world;
    let (o2, _) = cam.project(anchor.origin, screen);

    let mut out = vec![Projected {
        handle: Handle::View,
        shape: Shape::Disc {
            center: o2,
            radius: INNER_PX,
        },
        // O plano da tela nunca fica de perfil consigo mesmo: esta alça é a única que não pode
        // degenerar, e é por isso que ela é a rede de segurança do enquadramento difícil.
        live: true,
    }];

    for n in 0..3 {
        let (u, v) = ((n + 1) % 3, (n + 2) % 3);
        let corner = |a: f32, b: f32| -> [f32; 2] {
            let mut p = anchor.origin;
            for k in 0..3 {
                p[k] += WORLD_AXES[u][k] * a * arm + WORLD_AXES[v][k] * b * arm;
            }
            cam.project(p, screen).0
        };
        let (lo, hi) = (PLANE_AT, PLANE_AT + PLANE_SIDE);
        let quad = [
            corner(lo, lo),
            corner(hi, lo),
            corner(hi, hi),
            corner(lo, hi),
        ];
        // ⚠️ **De perfil, um quadrado é um traço.** A pergunta certa não é a área: é se ele ainda é
        // largo o bastante para se apontar — o lado mais estreito tem de passar do raio de agarre.
        let narrow = (0..4)
            .map(|i| dist(quad[i], quad[(i + 1) % 4]))
            .fold(f32::INFINITY, f32::min);
        out.push(Projected {
            handle: Handle::Plane(n),
            shape: Shape::Quad(quad),
            live: narrow >= GRAB_PX,
        });
    }

    for (n, axis) in WORLD_AXES.iter().enumerate() {
        let tip = {
            let mut p = anchor.origin;
            for k in 0..3 {
                p[k] += axis[k] * arm;
            }
            cam.project(p, screen).0
        };
        let len = dist(o2, tip);
        out.push(Projected {
            handle: Handle::Axis(n),
            shape: Shape::Arrow { from: o2, to: tip },
            live: len >= MIN_ARM_PX,
        });
    }
    out
}

/// **De quem é este ponto?** — `None` quando nenhuma alça o reclama.
pub(crate) fn pick(projected: &[Projected], p: [f32; 2]) -> Option<Handle> {
    projected
        .iter()
        .find(|h| h.live && hits(h.shape, p))
        .map(|h| h.handle)
}

fn hits(shape: Shape, p: [f32; 2]) -> bool {
    match shape {
        Shape::Disc { center, radius } => dist(center, p) <= radius,
        Shape::Quad(q) => point_in_quad(q, p),
        // ⚠️ A haste começa DEPOIS da folga: sem isto as três setas disputariam o centro com o
        // disco, e qual ganha dependeria da ordem da lista em vez da geometria.
        Shape::Arrow { from, to } => {
            let d = [to[0] - from[0], to[1] - from[1]];
            let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
            if len <= INNER_PX {
                return false;
            }
            let u = [d[0] / len, d[1] / len];
            let start = [from[0] + u[0] * INNER_PX, from[1] + u[1] * INNER_PX];
            dist_to_segment(start, to, p) <= GRAB_PX
        }
    }
}

/// ⭐ **O arrasto**: quanto o nó anda no MUNDO quando o ponteiro vai de `from_px` a `to_px`.
///
/// Devolve `[0; 3]` quando a alça não é utilizável neste enquadramento — a mesma condição que
/// [`project`] usa para a esconder, porque uma alça invisível não pode arrastar.
pub(crate) fn drag(
    handle: Handle,
    anchor: Anchor,
    cam: &Orbit,
    screen: Screen,
    from_px: [f32; 2],
    to_px: [f32; 2],
) -> [f32; 3] {
    let px_per_world = screen.px_per_world().max(f32::MIN_POSITIVE);
    let arm = ARM_PX / px_per_world;
    match handle {
        // A conta é uma projeção escalar: `d` é quanto o braço inteiro mede na tela, e a fração do
        // movimento do rato ao longo dele é a fração do braço que a peça anda.
        //
        // ⚠️ **Sem divisão por zero possível**: `dot(d,d)` só é nulo quando o eixo aponta ao
        // observador, e aí a alça já não está viva.
        Handle::Axis(n) => {
            let (o2, _) = cam.project(anchor.origin, screen);
            let tip = {
                let mut p = anchor.origin;
                for k in 0..3 {
                    p[k] += WORLD_AXES[n][k] * arm;
                }
                cam.project(p, screen).0
            };
            let d = [tip[0] - o2[0], tip[1] - o2[1]];
            let dd = d[0].mul_add(d[0], d[1] * d[1]);
            if dd < MIN_ARM_PX * MIN_ARM_PX {
                return [0.0; 3];
            }
            let m = [to_px[0] - from_px[0], to_px[1] - from_px[1]];
            let t = m[0].mul_add(d[0], m[1] * d[1]) / dd * arm;
            [
                WORLD_AXES[n][0] * t,
                WORLD_AXES[n][1] * t,
                WORLD_AXES[n][2] * t,
            ]
        }
        // Num plano, o deslocamento é a diferença entre dois pontos do plano — cada um o encontro do
        // raio do cursor com ele. É a mesma conta do gizmo 2D, com o plano a vir do mundo.
        Handle::Plane(n) => plane_delta(WORLD_AXES[n], anchor.origin, cam, screen, from_px, to_px),
        // O plano da tela: a normal é a direção da vista, e o denominador vale 1 — nunca degenera.
        Handle::View => {
            let (_, _, fwd) = cam.basis();
            plane_delta(fwd, anchor.origin, cam, screen, from_px, to_px)
        }
    }
}

fn plane_delta(
    normal: [f32; 3],
    origin: [f32; 3],
    cam: &Orbit,
    screen: Screen,
    from_px: [f32; 2],
    to_px: [f32; 2],
) -> [f32; 3] {
    let a = ray_plane(cam, screen, from_px, origin, normal);
    let b = ray_plane(cam, screen, to_px, origin, normal);
    match (a, b) {
        (Some(a), Some(b)) => [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
        _ => [0.0; 3],
    }
}

/// Onde o raio de um pixel encontra o plano que passa por `p0` com normal `n`. `None` de perfil.
fn ray_plane(
    cam: &Orbit,
    screen: Screen,
    px: [f32; 2],
    p0: [f32; 3],
    n: [f32; 3],
) -> Option<[f32; 3]> {
    let (o, dir) = cam.ray(px[0], px[1], screen);
    let denom = ph2d_field::xform::dot(dir, n);
    // ⚠️ O limiar não é folclore: abaixo dele um pixel de rato vale um salto arbitrário no plano, e
    // o gesto deixa de ser manipulação para ser sorteio. É a mesma razão de `MIN_ARM_PX`.
    if denom.abs() < 1.0e-3 {
        return None;
    }
    let t = ph2d_field::xform::dot([p0[0] - o[0], p0[1] - o[1], p0[2] - o[2]], n) / denom;
    Some([o[0] + dir[0] * t, o[1] + dir[1] * t, o[2] + dir[2] * t])
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn dist_to_segment(a: [f32; 2], b: [f32; 2], p: [f32; 2]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let dd = d[0].mul_add(d[0], d[1] * d[1]);
    if dd <= f32::MIN_POSITIVE {
        return dist(a, p);
    }
    let t = ((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / dd;
    let t = t.clamp(0.0, 1.0);
    dist([a[0] + d[0] * t, a[1] + d[1] * t], p)
}

/// ⚠️ **Por produto vetorial, e não por «está dentro da caixa»**: o quadrilátero é um quadrado do
/// MUNDO já projetado, então ele é um losango qualquer na tela. Um teste de caixa alinhada
/// reclamaria pixels que não são dele — e como as três alças de plano se tocam nos cantos, o gesto
/// escolheria a errada exatamente onde a diferença importa.
fn point_in_quad(q: [[f32; 2]; 4], p: [f32; 2]) -> bool {
    let mut positive = false;
    let mut negative = false;
    for i in 0..4 {
        let a = q[i];
        let b = q[(i + 1) % 4];
        let cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        if cross > 0.0 {
            positive = true;
        }
        if cross < 0.0 {
            negative = true;
        }
    }
    !(positive && negative)
}

#[cfg(test)]
#[path = "field3d_gizmo_tests.rs"]
mod tests;
