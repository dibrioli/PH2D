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

    /// O intervalo `[t0, t1]` em que o raio está DENTRO da caixa, recortado a
    /// `t ≥ 0`. Método das *slabs*, com o recíproco da direção vindo pronto do
    /// chamador — numa varredura de octree a divisão é paga uma vez por raio,
    /// não uma vez por nó. Vazio (`t0 > t1`) quando não há travessia.
    ///
    /// ⚠️ **Direção com componente zero é tratada de propósito, não por acaso.**
    /// `inv` vira `±INFINITY`, e um raio que começa EXATAMENTE no plano da
    /// caixa produz `0 * INFINITY = NaN`. O `f32::max`/`min` da `std` devolvem
    /// *o outro operando* quando um é NaN, então aquele eixo simplesmente não
    /// restringe o intervalo — a resposta fica **conservadora** (a caixa pode
    /// ser visitada à toa), que é exatamente o erro que um índice espacial pode
    /// cometer sem mentir. O erro oposto — descartar a caixa — perderia
    /// geometria em silêncio.
    ///
    /// Uma pergunta, dois consumidores: quem só quer saber *se* atravessa lê
    /// [`Aabb::ray_hit`], quem ordena a travessia lê o `t0`. Duas implementações
    /// de slab divergiriam no caso do NaN, que é o único que importa.
    #[must_use]
    pub fn ray_slab(&self, origin: [f32; 3], inv_dir: [f32; 3]) -> (f32, f32) {
        let mut t0 = 0.0f32;
        let mut t1 = f32::INFINITY;
        for k in 0..3 {
            let a = (self.min[k] - origin[k]) * inv_dir[k];
            let b = (self.max[k] - origin[k]) * inv_dir[k];
            t0 = t0.max(a.min(b));
            t1 = t1.min(a.max(b));
        }
        (t0, t1)
    }

    /// O raio atravessa a caixa dentro de `[0, t_max]`?
    #[must_use]
    pub fn ray_hit(&self, origin: [f32; 3], inv_dir: [f32; 3], t_max: f32) -> bool {
        let (t0, t1) = self.ray_slab(origin, inv_dir);
        t0 <= t1.min(t_max)
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
