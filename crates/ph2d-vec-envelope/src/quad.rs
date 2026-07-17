//! O gesto **Quad / perspectiva** (ADR-0123 §4): a gaiola de 4 cantos, como um mapa [`Warp`].
//!
//! # A matemática, e de onde ela vem
//!
//! Uma homografia projetiva mapeia o quadrado unitário para um quadrilátero **em forma fechada** —
//! duas Cramer 2×2, **sem** o sistema 8×8 que a construção ingênua monta (Heckbert, *Fundamentals
//! of Texture Mapping and Image Warping*, 1989, §3.1). O mesmo mapa já vive no repo, em `f32`, no
//! nó `ph2d-node-motion-four-point-warp` — aqui ele é re-portado em **`f64`** (o pipeline vetorial é
//! todo `f64`) e privado a esta crate. Não dependemos daquele nó: a math é ~40 linhas e ele está
//! soldado ao contrato de nós.
//!
//! # Por que projetivo, e não bilinear
//!
//! Os dois concordam nos 4 cantos e **divergem no interior**: sob bilinear uma reta vira parábola;
//! sob homografia **toda reta continua reta**. É por isso que o modo "perspectiva" do CorelDRAW é
//! documentado como *"adding perspective"* — perspectiva exige o mapa projetivo. E a imagem é
//! **degenerada** (a divisão perde sentido) só na linha de fuga; a gaiola **convexa** a mantém fora
//! (ADR-0123 §5, e [`QuadWarp::is_convex`]).

use crate::Warp;

/// Piso absoluto de degenerescência. ⚠️ É **absoluto**, então tem um precipício de escala (como o
/// `ARCLEN_EPS` da `ph2d-vec-blend`): numa gaiola gigante ou minúscula ele deixa de separar o que
/// devia. A Fatia B trabalha em coordenadas de mundo do documento, onde isto é seguro; se um dia a
/// gaiola operar em unidades normalizadas, revisar.
const EPS: f64 = 1e-12;

/// Homografia projetiva 3×3 — só as 8 entradas livres (a 9ª é fixa em 1).
#[derive(Clone, Copy, Debug)]
struct Homography {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
    g: f64,
    h: f64,
}

impl Homography {
    /// O mapa fechado do quadrado unitário para `q`, com os cantos na ordem
    /// `(0,0)→q0, (1,0)→q1, (1,1)→q2, (0,1)→q3` (Heckbert §3.1).
    ///
    /// `None` quando o quad é degenerado (três cantos colineares) — o denominador de Cramer some.
    fn unit_square_to(q: &[[f64; 2]; 4]) -> Option<Self> {
        let ([x0, y0], [x1, y1], [x2, y2], [x3, y3]) = (q[0], q[1], q[2], q[3]);
        let sx = x0 - x1 + x2 - x3;
        let sy = y0 - y1 + y2 - y3;

        if sx.abs() < EPS && sy.abs() < EPS {
            // Paralelogramo ⇒ o mapa é **afim** (sem termos de perspectiva). Não é um caso especial
            // a evitar: é o certo — um paralelogramo não tem linha de fuga, e forçar perspectiva
            // aqui inventaria uma. `g = h = 0`.
            return Some(Self {
                a: x1 - x0,
                b: x3 - x0,
                c: x0,
                d: y1 - y0,
                e: y3 - y0,
                f: y0,
                g: 0.0,
                h: 0.0,
            });
        }

        let (dx1, dx2, dy1, dy2) = (x1 - x2, x3 - x2, y1 - y2, y3 - y2);
        let den = dx1 * dy2 - dx2 * dy1;
        if den.abs() < EPS {
            return None;
        }
        let g = (sx * dy2 - sy * dx2) / den;
        let h = (dx1 * sy - dy1 * sx) / den;
        Some(Self {
            a: x1 - x0 + g * x1,
            b: x3 - x0 + h * x3,
            c: x0,
            d: y1 - y0 + g * y1,
            e: y3 - y0 + h * y3,
            g,
            h,
            f: y0,
        })
    }

    /// O denominador projetivo `w = g·u + h·v + 1`. Zero na linha de fuga — a gaiola convexa o
    /// mantém `> 0` em `[0,1]²`.
    fn denom(&self, u: f64, v: f64) -> f64 {
        self.g * u + self.h * v + 1.0
    }

    fn apply(&self, u: f64, v: f64) -> [f64; 2] {
        let w = self.denom(u, v);
        [
            (self.a * u + self.b * v + self.c) / w,
            (self.d * u + self.e * v + self.f) / w,
        ]
    }

    /// A jacobiana `d(x,y)/d(u,v)` em ordem de linha, em **forma fechada**.
    ///
    /// Derivando `x = N_x/w` pela regra do quociente e reconhecendo `N_x/w = x`, sai
    /// `∂x/∂u = (a − x·g)/w` — sem diferença finita. É esta a derivada que a espinha exige exata
    /// (jacobiana inconsistente com o `apply` faz o fit **não convergir**, [`Warp::jacobian`]).
    fn jacobian(&self, u: f64, v: f64) -> [[f64; 2]; 2] {
        let w = self.denom(u, v);
        let x = (self.a * u + self.b * v + self.c) / w;
        let y = (self.d * u + self.e * v + self.f) / w;
        [
            [(self.a - x * self.g) / w, (self.b - x * self.h) / w],
            [(self.d - y * self.g) / w, (self.e - y * self.h) / w],
        ]
    }
}

/// O gesto **Quad** como [`Warp`]: normaliza o ponto do bbox-fonte para o quadrado unitário (um
/// mapa afim, `diag(1/w, 1/h)`) e aplica a homografia unit-square → cantos-destino.
///
/// **Ordem dos cantos:** `[BL, BR, TR, TL]` — os cantos do bbox-fonte na mesma ordem, em sentido
/// anti-horário a partir do inferior-esquerdo. Em **repouso** (destino = cantos do bbox), o mapa é
/// a **identidade** — e é isso que faz o envelope em repouso não mexer numa vírgula.
#[derive(Clone, Copy, Debug)]
pub struct QuadWarp {
    origin: [f64; 2],
    size: [f64; 2],
    h: Homography,
}

impl QuadWarp {
    /// `None` se o bbox-fonte for degenerado (largura ou altura ~0) ou o quad-destino for
    /// degenerado (3 cantos colineares).
    #[must_use]
    pub fn new(origin: [f64; 2], size: [f64; 2], corners: [[f64; 2]; 4]) -> Option<Self> {
        if size[0].abs() < EPS || size[1].abs() < EPS {
            return None;
        }
        Some(Self {
            origin,
            size,
            h: Homography::unit_square_to(&corners)?,
        })
    }

    /// O quad é **estritamente convexo**?
    ///
    /// É o que mantém a linha de fuga **fora** da gaiola (ADR-0123 §5): uma homografia de retângulo
    /// para quad estritamente convexo não põe o horizonte dentro do retângulo. A UI da Fatia B
    /// recusa quad não-convexo por isto — não é enfeite, é o que torna o caso degenerado
    /// **inalcançável pelo gesto**, sem clipping e sem epsilon.
    ///
    /// Testa se todas as viradas consecutivas têm o mesmo sentido (todos os produtos vetoriais o
    /// mesmo sinal). Ordem de polígono é pressuposta; cantos fora de ordem (bowtie) dão sinais
    /// mistos e são corretamente reprovados.
    #[must_use]
    pub fn is_convex(corners: &[[f64; 2]; 4]) -> bool {
        let mut sign = 0.0_f64;
        for i in 0..4 {
            let a = corners[i];
            let b = corners[(i + 1) % 4];
            let c = corners[(i + 2) % 4];
            let cross = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]);
            if cross.abs() < EPS {
                continue; // vértice colinear: não decide o sentido
            }
            if sign == 0.0 {
                sign = cross.signum();
            } else if cross.signum() != sign {
                return false;
            }
        }
        sign != 0.0
    }

    /// Coordenada normalizada `(u,v)` do ponto de mundo no quadrado unitário do bbox-fonte.
    fn uv(&self, p: [f64; 2]) -> (f64, f64) {
        (
            (p[0] - self.origin[0]) / self.size[0],
            (p[1] - self.origin[1]) / self.size[1],
        )
    }
}

impl Warp for QuadWarp {
    fn map(&self, p: [f64; 2]) -> [f64; 2] {
        let (u, v) = self.uv(p);
        self.h.apply(u, v)
    }

    fn jacobian(&self, p: [f64; 2]) -> [[f64; 2]; 2] {
        let (u, v) = self.uv(p);
        let j = self.h.jacobian(u, v);
        // Regra da cadeia com a normalização afim: `J_map = J_h · diag(1/w, 1/h)`. O bloco da
        // normalização é diagonal, então cada coluna de `J_h` divide pela dimensão dela.
        [
            [j[0][0] / self.size[0], j[0][1] / self.size[1]],
            [j[1][0] / self.size[0], j[1][1] / self.size[1]],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cantos do bbox `[BL, BR, TR, TL]`.
    fn bbox_corners(origin: [f64; 2], size: [f64; 2]) -> [[f64; 2]; 4] {
        let [ox, oy] = origin;
        let [w, h] = size;
        [[ox, oy], [ox + w, oy], [ox + w, oy + h], [ox, oy + h]]
    }

    /// **EM REPOUSO, O MAPA É A IDENTIDADE.** Destino = cantos do bbox ⇒ `map(p) == p`. É a metade
    /// "presença" do par ausência/presença: se o motor não desenhasse nada, este teste ficaria
    /// verde por vazio — mas ele afirma pontos concretos **iguais à entrada**.
    #[test]
    fn at_rest_the_quad_warp_is_the_identity() {
        let origin = [3.0, -2.0];
        let size = [10.0, 6.0];
        let w = QuadWarp::new(origin, size, bbox_corners(origin, size)).unwrap();
        for p in [[3.0, -2.0], [8.0, 1.0], [13.0, 4.0], [5.5, 0.0]] {
            let q = w.map(p);
            assert!(
                (q[0] - p[0]).abs() < 1e-12 && (q[1] - p[1]).abs() < 1e-12,
                "repouso não é identidade em {p:?}: {q:?}"
            );
        }
    }

    /// **Os 4 cantos do bbox mapeiam EXATAMENTE nos 4 cantos da gaiola.** É o que "gaiola" quer
    /// dizer, e prova de uma vez a normalização e a homografia. Exato, não aproximado.
    #[test]
    fn the_bbox_corners_map_exactly_to_the_cage_corners() {
        let origin = [0.0, 0.0];
        let size = [10.0, 6.0];
        let dest = [[1.0, 1.0], [9.0, 0.0], [8.0, 7.0], [2.0, 5.0]];
        let w = QuadWarp::new(origin, size, dest).unwrap();
        for (src, want) in bbox_corners(origin, size).into_iter().zip(dest) {
            let got = w.map(src);
            assert!(
                (got[0] - want[0]).abs() < 1e-9 && (got[1] - want[1]).abs() < 1e-9,
                "canto {src:?} devia mapear em {want:?}, deu {got:?}"
            );
        }
    }

    /// **Um paralelogramo é um mapa AFIM** — sem termos de perspectiva (`g == h == 0`), e a
    /// jacobiana é **constante** em todo o plano.
    #[test]
    fn a_parallelogram_quad_is_an_affine_map() {
        let origin = [0.0, 0.0];
        let size = [10.0, 6.0];
        // Topo cisalhado +3 em x: continua paralelogramo (lados opostos paralelos).
        let dest = [[0.0, 0.0], [10.0, 0.0], [13.0, 6.0], [3.0, 6.0]];
        let w = QuadWarp::new(origin, size, dest).unwrap();
        assert!(
            w.h.g == 0.0 && w.h.h == 0.0,
            "paralelogramo tem perspectiva: {:?}",
            w.h
        );
        let j0 = w.jacobian([1.0, 1.0]);
        let j1 = w.jacobian([9.0, 5.0]);
        assert_eq!(j0, j1, "jacobiana de mapa afim não é constante");
    }

    /// **A gaiola convexa mantém o horizonte FORA** (ausência): `w > 0` em todo `[0,1]²`, denso.
    #[test]
    fn a_convex_quad_keeps_the_horizon_out_of_the_cage() {
        // Trapézio convexo com perspectiva forte (topo bem mais estreito que a base).
        let dest = [[0.0, 0.0], [10.0, 0.0], [6.0, 6.0], [4.0, 6.0]];
        assert!(QuadWarp::is_convex(&dest), "o fixture devia ser convexo");
        let h = Homography::unit_square_to(&dest).unwrap();
        let mut min_w = f64::INFINITY;
        for i in 0..=50 {
            for j in 0..=50 {
                min_w = min_w.min(h.denom(i as f64 / 50.0, j as f64 / 50.0));
            }
        }
        assert!(
            min_w > 0.0,
            "convexo mas o horizonte entrou na gaiola: min_w={min_w:.3e}"
        );
    }

    /// **A metade PRESENÇA do par:** o mesmo amostrador, numa homografia cujo horizonte SABIDAMENTE
    /// cruza a gaiola (`g = -2` ⇒ `w = 1 - 2u = 0` em `u = 0.5`), reporta `w ≤ 0`.
    ///
    /// Sem este teste, o gate de ausência acima ficaria verde num amostrador que **sempre** devolve
    /// `w > 0` — [[feedback_absence_gate_needs_a_presence_sibling]]. Provamos que ele detecta um
    /// cruzamento quando há um.
    #[test]
    fn the_sampler_detects_a_horizon_that_crosses_the_cage() {
        let h = Homography {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
            e: 1.0,
            f: 0.0,
            g: -2.0,
            h: 0.0,
        };
        let mut min_w = f64::INFINITY;
        for i in 0..=50 {
            min_w = min_w.min(h.denom(i as f64 / 50.0, 0.0));
        }
        assert!(
            min_w <= 0.0,
            "o amostrador não viu o horizonte: min_w={min_w:.3e}"
        );
    }

    /// `is_convex` **distingue** — senão o gate de convexidade acima seria uma tautologia.
    #[test]
    fn is_convex_distinguishes_convex_from_concave() {
        let convex = [[0.0, 0.0], [10.0, 0.0], [6.0, 6.0], [4.0, 6.0]];
        // TR puxado para dentro do triângulo dos outros três ⇒ reflexo ⇒ côncavo.
        let concave = [[0.0, 0.0], [10.0, 0.0], [3.0, 3.0], [0.0, 10.0]];
        assert!(QuadWarp::is_convex(&convex), "trapézio devia ser convexo");
        assert!(
            !QuadWarp::is_convex(&concave),
            "quad reflexo passou por convexo"
        );
    }

    /// **A jacobiana fechada bate com a diferença central de `map`.** Guarda direto a consistência
    /// que a espinha exige — e é rápida, ao contrário do sintoma dela (o fit travar).
    #[test]
    fn the_closed_form_jacobian_matches_finite_difference() {
        let origin = [0.0, 0.0];
        let size = [10.0, 6.0];
        let dest = [[0.5, 0.5], [10.0, -0.5], [8.5, 6.5], [1.5, 5.5]];
        let w = QuadWarp::new(origin, size, dest).unwrap();
        let step = 1e-6;
        for p in [[2.0, 1.0], [7.0, 4.0], [5.0, 3.0], [9.0, 5.0]] {
            let j = w.jacobian(p);
            let du = {
                let a = w.map([p[0] + step, p[1]]);
                let b = w.map([p[0] - step, p[1]]);
                [(a[0] - b[0]) / (2.0 * step), (a[1] - b[1]) / (2.0 * step)]
            };
            let dv = {
                let a = w.map([p[0], p[1] + step]);
                let b = w.map([p[0], p[1] - step]);
                [(a[0] - b[0]) / (2.0 * step), (a[1] - b[1]) / (2.0 * step)]
            };
            assert!(
                (j[0][0] - du[0]).abs() < 1e-5
                    && (j[1][0] - du[1]).abs() < 1e-5
                    && (j[0][1] - dv[0]).abs() < 1e-5
                    && (j[1][1] - dv[1]).abs() < 1e-5,
                "jacobiana ≠ diferença finita em {p:?}: J={j:?} du={du:?} dv={dv:?}"
            );
        }
    }
}
