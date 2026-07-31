//! A caixa alinhada aos eixos — o tipo que o octree, o import e a consulta de
//! pincel compartilham.
//!
//! Uma caixa **vazia** é `min = +INF`, `max = -INF`: assim `expand` de um ponto
//! sobre a vazia dá a caixa degenerada correta sem caso especial, e `is_empty`
//! é uma comparação em vez de um flag que alguém esquece de manter.

/// Caixa alinhada aos eixos, em coordenadas de MUNDO (metros, Y-up — a mesma
/// convenção do `Transform` do repo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Default for Aabb {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Aabb {
    /// A caixa que não contém nada. Invertida de propósito (ver o doc do módulo).
    pub const EMPTY: Self = Self {
        min: [f32::INFINITY; 3],
        max: [f32::NEG_INFINITY; 3],
    };

    #[must_use]
    pub fn from_points(points: &[[f32; 3]]) -> Self {
        let mut b = Self::EMPTY;
        for p in points {
            b.expand_point(*p);
        }
        b
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.min[0] > self.max[0] || self.min[1] > self.max[1] || self.min[2] > self.max[2]
    }

    pub fn expand_point(&mut self, p: [f32; 3]) {
        for ((v, lo), hi) in p.iter().zip(&mut self.min).zip(&mut self.max) {
            if v < lo {
                *lo = *v;
            }
            if v > hi {
                *hi = *v;
            }
        }
    }

    pub fn expand(&mut self, other: &Self) {
        for k in 0..3 {
            if other.min[k] < self.min[k] {
                self.min[k] = other.min[k];
            }
            if other.max[k] > self.max[k] {
                self.max[k] = other.max[k];
            }
        }
    }

    #[must_use]
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    /// A maior aresta. É ela que dá a escala do modelo para os defaults de raio
    /// de pincel e de resolução de remesh — nunca um número absoluto, porque
    /// uma cabeça e um planeta chegam aqui pela mesma porta.
    #[must_use]
    pub fn longest_edge(&self) -> f32 {
        if self.is_empty() {
            return 0.0;
        }
        let d = [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ];
        d[0].max(d[1]).max(d[2])
    }

    /// A caixa toca a esfera? Distância ao ponto mais próximo da caixa, ao
    /// quadrado — sem `sqrt`, que numa varredura de octree é o custo todo.
    #[must_use]
    pub fn intersects_sphere(&self, center: [f32; 3], radius: f32) -> bool {
        let mut d2 = 0.0f32;
        for ((v, lo), hi) in center.iter().zip(&self.min).zip(&self.max) {
            if v < lo {
                let t = lo - v;
                d2 += t * t;
            } else if v > hi {
                let t = v - hi;
                d2 += t * t;
            }
        }
        d2 <= radius * radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_box_expanded_by_one_point_is_that_point() {
        let mut b = Aabb::EMPTY;
        assert!(b.is_empty());
        b.expand_point([1.0, 2.0, 3.0]);
        assert!(!b.is_empty());
        assert_eq!(b.min, [1.0, 2.0, 3.0]);
        assert_eq!(b.max, [1.0, 2.0, 3.0]);
        assert_eq!(b.center(), [1.0, 2.0, 3.0]);
        assert_eq!(b.longest_edge(), 0.0);
    }

    #[test]
    fn a_sphere_hit_is_measured_to_the_nearest_point_not_to_the_centre() {
        let b = Aabb {
            min: [0.0, 0.0, 0.0],
            max: [1.0, 1.0, 1.0],
        };
        // Encosta na face: distância exata 0.5 até a face x=1.
        assert!(b.intersects_sphere([1.5, 0.5, 0.5], 0.5));
        assert!(!b.intersects_sphere([1.5, 0.5, 0.5], 0.49));
        // Na diagonal a distância é maior que em qualquer eixo — é isto que uma
        // checagem por-eixo erraria (ela aceitaria a esfera do canto).
        let corner_d2: f32 = 3.0 * 0.5 * 0.5;
        assert!(b.intersects_sphere([1.5, 1.5, 1.5], corner_d2.sqrt() + 1e-4));
        assert!(!b.intersects_sphere([1.5, 1.5, 1.5], corner_d2.sqrt() - 1e-3));
        // Dentro é sempre um acerto, com qualquer raio.
        assert!(b.intersects_sphere([0.5, 0.5, 0.5], 0.0));
    }
}
