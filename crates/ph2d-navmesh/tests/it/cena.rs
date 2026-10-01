//! Cenas aleatórias DETERMINÍSTICAS (um gerador congruencial — nenhuma dependência e a mesma
//! sequência em toda máquina).

use ph2d_navmesh::Shape;

pub struct Lcg(pub u64);

impl Lcg {
    pub fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) as u32
    }
    /// Um `f64` em `[a, b)`.
    pub fn range(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * (self.next_u32() as f64 / 4_294_967_296.0)
    }
}

/// Uma caixa rodada de centro `c`, meias-dimensões `h` e rotação dada por (cos, sin) com só `sqrt`.
pub fn caixa_rodada(c: [f64; 2], h: [f64; 2], cs: [f64; 2]) -> Vec<[f64; 2]> {
    let [co, si] = cs;
    [[-h[0], -h[1]], [h[0], -h[1]], [h[0], h[1]], [-h[0], h[1]]]
        .iter()
        .map(|p| [c[0] + p[0] * co - p[1] * si, c[1] + p[0] * si + p[1] * co])
        .collect()
}

/// `n` obstáculos misturados (caixas rodadas, círculos, cápsulas) numa região `w × h`.
pub fn obstaculos(rng: &mut Lcg, n: usize, w: f64, h: f64) -> Vec<Shape> {
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        let c = [rng.range(0.0, w), rng.range(0.0, h)];
        match rng.next_u32() % 4 {
            0 | 1 => {
                let hx = rng.range(0.2, 2.0);
                let hy = rng.range(0.2, 2.0);
                // Um ângulo qualquer sem trigonometria: um vector aleatório normalizado.
                let (a, b) = (rng.range(-1.0, 1.0), rng.range(-1.0, 1.0));
                let l = (a * a + b * b).sqrt().max(1e-6);
                v.push(Shape::Convex(caixa_rodada(c, [hx, hy], [a / l, b / l])));
            }
            2 => v.push(Shape::Circle {
                center: c,
                radius: rng.range(0.2, 1.5),
            }),
            _ => {
                let d = [rng.range(-2.0, 2.0), rng.range(-2.0, 2.0)];
                v.push(Shape::Capsule {
                    a: c,
                    b: [c[0] + d[0], c[1] + d[1]],
                    radius: rng.range(0.1, 0.6),
                });
            }
        }
    }
    v
}

pub fn retangulo(w: f64, h: f64) -> Vec<[f64; 2]> {
    vec![[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]]
}
