//! **Sólidos isométricos** (2.5D) — módulo irmão de [`crate::shapes`].
//!
//! Não são 3D: são polígonos planos desenhados em projeção, que é o que os apps de
//! diagrama chamam de "3D shapes". O que se espera deles é que **as arestas internas
//! existam** — sem elas o cubo é um hexágono e a pirâmide é um triângulo.
//!
//! Autorado no espaço unitário ([`crate::space`]), `v = 0` no TOPO. A versão anterior
//! nascia de cabeça para baixo (ápices apontando para o chão) e o cubo carregava uma
//! **diagonal solta** cortando uma face.
//!
//! ## Os dois parâmetros que governam a projeção
//!
//! Ninguém concorda sobre a projeção: o `cube` do OOXML e o do draw.io são **oblíquos a
//! 45°** (a face frontal é um retângulo comum); o `isoCube` do draw.io é isométrico de
//! 30°. Em vez de escolher uma, o módulo expõe a identidade que governa todas:
//!
//! ```text
//! ângulo do eixo = atan(rise · altura_px / largura_px)
//! ```
//!
//! - **`rise`** — a altura (fração) do vértice interno. É o que inclina os eixos.
//!   `rise = 0,5` numa caixa quadrada dá **26,57°** (o dimétrico 2:1 do pixel art);
//!   `rise = 0,5` numa caixa de proporção `2/√3` dá **30°** (isométrico verdadeiro).
//! - **`skew`** — reparte largura e profundidade. **Não** muda o ângulo do eixo.
//!
//! ## Sombreamento por face: pendência consciente
//!
//! O padrão-ouro (e o que o OOXML faz no `cube`) é emitir **cada face como um caminho
//! preenchível próprio** — topo claro, lateral média, frontal escura —, porque é o
//! sombreamento que faz o sólido parecer sólido; só o contorno nunca lê como volume.
//! Aqui uma forma cozinha para UM `VecPath`, então as faces saem como silhueta preenchida
//! + arestas internas em traço. Faces independentes exigem `cook` emitir várias entidades
//! (uma por face, agrupadas) — mudança de arquitetura, anotada como follow-up.

use crate::VecPath;
use crate::space::{Unit, Uv, add_sub, closed, fit, poly};

/// O vértice interno e os seis da silhueta, dados `rise` e `skew`.
///
/// A silhueta é um hexágono e o vértice interno `M = (skew, rise)` é onde as três faces se
/// encontram. **É o único ponto de onde saem arestas internas** — e são exatamente três,
/// para os vértices alternados do hexágono. Qualquer outra aresta (uma diagonal de face,
/// por exemplo) é ruído: foi o que apareceu na foto do cubo.
fn cube_frame(rise: f64, skew: f64) -> ([Uv; 6], Uv) {
    let (r, t) = (rise, skew);
    let hex = [
        (1.0 - t, 0.0),             // V0 — o topo
        (1.0, r * t),               // V1 — ombro direito
        (1.0, 1.0 - r * (1.0 - t)), // V2 — quadril direito
        (t, 1.0),                   // V3 — a base
        (0.0, 1.0 - r * t),         // V4 — quadril esquerdo
        (0.0, r * (1.0 - t)),       // V5 — ombro esquerdo
    ];
    (hex, (t, r))
}

/// **Cubo isométrico.** Silhueta hexagonal + as TRÊS arestas internas (V1–M, V3–M, V5–M),
/// que recortam as três faces (topo, esquerda, direita).
#[must_use]
pub fn iso_cube(a: [f64; 2], b: [f64; 2], rise: f64, skew: f64) -> VecPath {
    let u = Unit::of(a, b);
    let (hex, m) = cube_frame(rise.clamp(0.1, 0.9), skew.clamp(0.1, 0.9));
    let mut p = poly(&u, &hex);
    // As três arestas internas, todas incidentes em M. Uma polilinha aberta V1→M→V3 mais
    // um traço M→V5: cobre as três sem repetir nenhuma.
    add_sub(
        &mut p,
        vec![u.corner(hex[1]), u.corner(m), u.corner(hex[3])],
        false,
    );
    add_sub(&mut p, vec![u.corner(m), u.corner(hex[5])], false);
    p
}

/// **Cone.** Ápice para CIMA; base elíptica; e — o ponto que quase todo mundo erra — as
/// geratrizes são **tangentes** à elipse da base, não retas até os extremos dela.
///
/// Com o ápice em `(0,5, 0)`, a base centrada em `(0,5, 1 − lip)` de raios `(0,5, lip)` e
/// altura `h = 1 − lip`, a tangência sai de
///
/// ```text
/// φ = asin(ry / h)          →  θ = −φ  e  θ = 180° + φ
/// T = (0,5 ± rx·√(h² − ry²)/h,  (1 − lip) − ry²/h)
/// ```
///
/// e o arco da FRENTE varre `180° + 2φ` (mais que meia elipse — é justamente o excedente
/// que revela a barriga da base). Ligar o ápice aos extremos laterais `(0/1, 1 − lip)` dá
/// um cone visivelmente "quebrado" no encontro: com `lip = 0,25` o erro é de 8,3% da
/// altura. O stencil de cone do draw.io tem exatamente esse defeito.
#[must_use]
pub fn iso_cone(a: [f64; 2], b: [f64; 2], lip: f64) -> VecPath {
    let u = Unit::of(a, b);
    let ry = lip.clamp(0.05, 0.45);
    let (rx, h) = (0.5, 1.0 - ry);
    let c: Uv = (0.5, 1.0 - ry);
    let phi = (ry / h).asin().to_degrees();

    let mut verts = vec![u.corner((0.5, 0.0))]; // o ápice
    verts.extend(u.arc(c, rx, ry, -phi, 180.0 + 2.0 * phi)); // a barriga da base
    let mut p = closed(verts);
    // A boca da base (a meia-elipse de trás), que é o que dá o volume ao cone.
    add_sub(
        &mut p,
        u.arc(c, rx, ry, 180.0 + phi, 180.0 - 2.0 * phi),
        false,
    );
    fit(&u, &mut p);
    p
}

/// **Pirâmide.** Ápice para CIMA sobre o centro da base; a silhueta é um QUADRILÁTERO
/// (ápice → direita → frente → esquerda) e há **uma** aresta interna: ápice → vértice da
/// frente. O 4º vértice da base fica escondido atrás — dentro da silhueta, nunca desenhado.
#[must_use]
pub fn iso_pyramid(a: [f64; 2], b: [f64; 2], rise: f64, skew: f64) -> VecPath {
    let u = Unit::of(a, b);
    let (hex, _) = cube_frame(rise.clamp(0.1, 0.9), skew.clamp(0.1, 0.9));
    let apex: Uv = (0.5, 0.0);
    let (right, front, left) = (hex[2], hex[3], hex[4]);

    let mut p = poly(&u, &[apex, right, front, left]);
    add_sub(&mut p, vec![u.corner(apex), u.corner(front)], false);
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: [f64; 2] = [-2.0, -2.0];
    const B: [f64; 2] = [2.0, 2.0];

    /// **Os sólidos ficam de PÉ.** O ápice do cone e o da pirâmide têm de estar ACIMA de
    /// todo o resto da forma (mundo Y-para-cima). Nasceram os dois apontando para o chão.
    #[test]
    fn the_apex_points_up_not_down() {
        for (name, p) in [
            ("cone", iso_cone(A, B, 0.15)),
            ("pyramid", iso_pyramid(A, B, 0.5, 0.5)),
        ] {
            let apex = p
                .verts
                .iter()
                .max_by(|x, y| x.anchor[1].total_cmp(&y.anchor[1]))
                .expect("tem vertices");
            assert!(
                apex.anchor[0].abs() < 1e-6,
                "{name}: o apice fica no eixo, nao no canto: {:?}",
                apex.anchor
            );
            let base = p.verts_all().map(|v| v.anchor[1]).fold(f64::MAX, f64::min);
            assert!(
                apex.anchor[1] > base + 1.0,
                "{name}: o apice ({}) tem de estar bem ACIMA da base ({base})",
                apex.anchor[1]
            );
        }
    }

    /// **A aresta solta do cubo, executável.** O cubo tem 6 vértices de silhueta e as
    /// arestas internas são exatamente 3 — todas incidentes no vértice central `M`. A
    /// versão antiga desenhava uma diagonal que cortava uma face ao meio (foi o que o Enio
    /// fotografou); este teste rejeita qualquer aresta que não toque `M`.
    #[test]
    fn every_internal_edge_of_the_cube_touches_the_centre() {
        let p = iso_cube(A, B, 0.5, 0.5);
        assert_eq!(p.verts.len(), 6, "a silhueta e um hexagono");

        let u = Unit::of(A, B);
        let m = u.p((0.5, 0.5));
        let near = |q: [f64; 2]| (q[0] - m[0]).hypot(q[1] - m[1]) < 1e-9;

        let mut edges = 0;
        for c in &p.subpaths {
            for w in c.verts.windows(2) {
                edges += 1;
                assert!(
                    near(w[0].anchor) || near(w[1].anchor),
                    "aresta interna solta {:?}→{:?}: nao toca o centro {m:?}",
                    w[0].anchor,
                    w[1].anchor
                );
            }
        }
        assert_eq!(edges, 3, "sao exatamente TRES arestas internas");
    }

    /// **As geratrizes do cone são TANGENTES à base.** O teste é o discriminante: a reta do
    /// ápice ao ponto de tangência toca a elipse em UM ponto (raiz dupla). Ligar o ápice ao
    /// extremo lateral da elipse — o erro clássico — dá duas raízes e o cone sai quebrado.
    #[test]
    fn the_cone_generatrices_are_tangent_to_its_base() {
        let lip = 0.25_f64;
        let (rx, ry) = (0.5, lip);
        let h = 1.0 - ry;
        // O ponto de tangência, em espaço de autoria (base centrada em (0.5, 1−lip)).
        let tx = 0.5 + rx * (h * h - ry * ry).sqrt() / h;
        let ty = (1.0 - ry) - ry * ry / h;

        // A reta ápice→T, parametrizada; substituída na elipse, tem de dar raiz DUPLA.
        let (ax, ay) = (0.5, 0.0);
        let (dx, dy) = (tx - ax, ty - ay);
        let (ex, ey) = (0.5, 1.0 - ry); // centro da elipse
        let qa = (dx / rx).powi(2) + (dy / ry).powi(2);
        let qb = 2.0 * ((ax - ex) * dx / (rx * rx) + (ay - ey) * dy / (ry * ry));
        let qc = ((ax - ex) / rx).powi(2) + ((ay - ey) / ry).powi(2) - 1.0;
        let disc = qb * qb - 4.0 * qa * qc;
        assert!(
            disc.abs() < 1e-9,
            "discriminante {disc} != 0 — a geratriz corta a base em vez de tangenciar"
        );

        // E o erro que se comete ligando ao extremo lateral é grande o bastante para ver.
        let naive_y = 1.0 - ry; // o "equador" da elipse
        assert!(
            (naive_y - ty).abs() > 0.08,
            "o ponto ingenuo estaria a {} da tangencia — se fosse pequeno, o bug seria invisivel e o teste, inutil",
            (naive_y - ty).abs()
        );
    }

    /// Todos cabem na caixa do gesto — senão a bbox mente e o gizmo desalinha do desenho.
    #[test]
    fn every_solid_fits_inside_the_gesture_box() {
        for (name, p) in [
            ("cube", iso_cube(A, B, 0.5, 0.5)),
            ("cone", iso_cone(A, B, 0.25)),
            ("pyramid", iso_pyramid(A, B, 0.5, 0.5)),
        ] {
            for v in p.verts_all() {
                assert!(
                    v.anchor[0].abs() <= 2.0 + 1e-6 && v.anchor[1].abs() <= 2.0 + 1e-6,
                    "{name}: ancora {:?} fora da caixa",
                    v.anchor
                );
            }
        }
    }
}
