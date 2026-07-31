//! Gates do pick.
//!
//! O oráculo central é a **força bruta**: um raio contra TODA face, sem octree.
//! Um índice espacial só pode errar de um jeito — esquecer geometria —, e a
//! única coisa que prova que ele não esqueceu é a resposta que não usa o índice.

use super::*;
use crate::face::Face;
use crate::shapes;

/// O acerto mais próximo testando toda face, sem octree e sem poda. É o
/// oráculo: se ele e o [`Mesh::raycast`] discordarem, quem errou é o índice.
fn brute_force(mesh: &Mesh, ray: &Ray) -> Option<(u32, f32)> {
    let mut best: Option<(u32, f32)> = None;
    for (fi, f) in mesh.faces().iter().enumerate() {
        let vs = f.verts();
        let p = |k: usize| mesh.positions()[vs[k] as usize];
        let mut hit = ray_triangle(ray.origin(), ray.dir(), p(0), p(1), p(2));
        if vs.len() == 4 {
            let second = ray_triangle(ray.origin(), ray.dir(), p(0), p(2), p(3));
            hit = match (hit, second) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
        }
        if let Some(t) = hit
            && best.is_none_or(|(_, bt)| t < bt)
        {
            best = Some((fi as u32, t));
        }
    }
    best
}

/// Gerador determinístico — um `xorshift64*`. Sem dep e sem flake: a mesma
/// semente dá a mesma varredura em toda máquina, então um raio que quebrar é um
/// raio que se reproduz.
struct Rng(u64);

impl Rng {
    fn unit(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        // 24 bits de mantissa: exatamente o que um f32 representa sem arredondar.
        ((self.0 >> 40) as f32) / ((1u32 << 24) as f32)
    }

    fn signed(&mut self) -> f32 {
        self.unit() * 2.0 - 1.0
    }
}

#[test]
fn the_octree_never_loses_geometry_that_brute_force_finds() {
    let mesh = shapes::uv_sphere(24, 32, 1.0);
    let mut rng = Rng(0x2026_07_31);
    let mut hits = 0;
    let mut misses = 0;
    for _ in 0..400 {
        // Origens espalhadas numa casca bem maior que a esfera, mirando pontos
        // dentro e FORA dela — a mistura é o que garante acertos e erros.
        let origin = [rng.signed() * 4.0, rng.signed() * 4.0, rng.signed() * 4.0];
        let target = [rng.signed() * 1.6, rng.signed() * 1.6, rng.signed() * 1.6];
        let ray = Ray::new(
            origin,
            [
                target[0] - origin[0],
                target[1] - origin[1],
                target[2] - origin[2],
            ],
        );
        let want = brute_force(&mesh, &ray);
        let got = mesh.raycast(&ray).map(|h| (h.face, h.t));
        match (want, got) {
            (None, None) => misses += 1,
            (Some((wf, wt)), Some((gf, gt))) => {
                hits += 1;
                // A FACE pode empatar em aresta compartilhada; a DISTÂNCIA não.
                assert!(
                    (wt - gt).abs() <= 1e-5,
                    "octree {gf} a t={gt} vs bruto {wf} a t={wt}"
                );
            }
            (w, g) => panic!("bruto {w:?} contra octree {g:?}"),
        }
    }
    // Sem isto o teste passaria com um raio que nunca acerta nada — o vácuo que
    // deixa um gate verde sobre um índice quebrado.
    assert!(hits > 50, "poucos acertos ({hits}) para valer como oráculo");
    assert!(misses > 20, "poucos erros ({misses}); os dois lados importam");
}

#[test]
fn a_ray_reports_the_near_surface_not_the_far_one() {
    let mesh = shapes::uv_sphere(24, 32, 1.0);
    let ray = Ray::new([0.0, 0.0, 5.0], [0.0, 0.0, -1.0]);
    let hit = mesh.raycast(&ray).expect("o raio atravessa a esfera");
    // A esfera tem raio 1 e o olho está a 5: a casca da frente está a 4, a de
    // trás a 6. Pegar a de trás é a assinatura exata de uma poda invertida.
    assert!(
        (hit.t - 4.0).abs() < 0.05,
        "t={} — devia ser ~4 (casca da frente)",
        hit.t
    );
    assert!(hit.point[2] > 0.0, "o ponto caiu no hemisfério de trás");
    assert!(
        hit.normal[2] > 0.5,
        "a normal da face acertada aponta para o olho: {:?}",
        hit.normal
    );
}

#[test]
fn nothing_behind_the_eye_is_ever_picked() {
    let mesh = shapes::uv_sphere(16, 24, 1.0);
    // Olho DENTRO da esfera, mirando +Z: só a casca da frente conta; a de trás
    // está a t negativo e não é acerto nenhum.
    let ray = Ray::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    let hit = mesh.raycast(&ray).expect("de dentro ainda se acerta a casca");
    assert!(hit.t > 0.0, "t={} não pode ser negativo", hit.t);
    assert!((hit.t - 1.0).abs() < 0.06, "t={}", hit.t);
}

#[test]
fn a_ray_that_passes_beside_the_mesh_hits_nothing() {
    let mesh = shapes::uv_sphere(16, 24, 1.0);
    let ray = Ray::new([3.0, 0.0, 5.0], [0.0, 0.0, -1.0]);
    assert!(mesh.raycast(&ray).is_none());
}

#[test]
fn a_quad_is_picked_by_the_two_triangles_the_gpu_draws() {
    // Quad NÃO-PLANAR: as duas diagonais dão superfícies diferentes, então o
    // fixture distingue a triangulação `a-c` da `b-d`. Num quad plano as duas
    // concordam e o gate seria verde por vácuo.
    let positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 1.0],
    ];
    let mesh = Mesh::from_parts(positions, vec![Face::quad(0, 1, 2, 3)]).unwrap();

    // Um ponto que cai no triângulo (a,c,d) e NÃO no (a,b,c).
    let ray = Ray::new([0.2, 0.7, 5.0], [0.0, 0.0, -1.0]);
    let hit = mesh.raycast(&ray).expect("o segundo triângulo do quad existe");
    assert_eq!(hit.face, 0);
    // O plano de (a,c,d) é `x − y + z = 0`, logo `z = y − x = 0.5` no ponto.
    assert!(
        (hit.point[2] - 0.5).abs() < 1e-4,
        "z={} — a diagonal usada não é a a-c",
        hit.point[2]
    );

    // E o primeiro triângulo continua sendo acertado onde ele é quem cobre.
    let ray2 = Ray::new([0.7, 0.2, 5.0], [0.0, 0.0, -1.0]);
    let h2 = mesh.raycast(&ray2).expect("o primeiro triângulo do quad existe");
    assert!((h2.point[2]).abs() < 1e-4, "z={}", h2.point[2]);
}

#[test]
fn a_degenerate_direction_is_refused_instead_of_answering_about_another_one() {
    let mesh = shapes::uv_sphere(8, 12, 1.0);
    for dir in [[0.0, 0.0, 0.0], [f32::NAN, 0.0, 0.0], [f32::INFINITY; 3]] {
        let ray = Ray::new([0.0, 0.0, 5.0], dir);
        assert!(mesh.raycast(&ray).is_none(), "dir {dir:?} devia ser recusada");
    }
}

#[test]
fn an_axis_aligned_ray_grazing_a_box_plane_is_not_lost_to_nan() {
    // O caso que o `ray_slab` documenta: origem EXATAMENTE no plano de uma
    // caixa, com a componente daquele eixo zerada. `0 * INFINITY = NaN`, e um
    // slab escrito sem cuidado descarta o nó e perde a face.
    let mesh = shapes::cube(1.0);
    let b = mesh.bounds();
    let ray = Ray::new([b.min[0], 0.0, 5.0], [0.0, 0.0, -1.0]);
    let want = brute_force(&mesh, &ray);
    let got = mesh.raycast(&ray).map(|h| (h.face, h.t));
    assert_eq!(want.is_some(), got.is_some(), "bruto {want:?} vs índice {got:?}");
}

#[test]
fn an_empty_mesh_is_a_miss_not_a_panic() {
    let mesh = Mesh::default();
    let ray = Ray::new([0.0, 0.0, 1.0], [0.0, 0.0, -1.0]);
    assert!(mesh.raycast(&ray).is_none());
}
