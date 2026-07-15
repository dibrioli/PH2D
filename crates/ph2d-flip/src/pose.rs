//! A **pose de uma chave** (W7.2 → W7.5): um afim 2×3, no MESMO layout do `Xform` do
//! shell (`[a, b, c, d, tx, ty]`, com `apply(p) = [a·x + c·y + tx, b·x + d·y + ty]`).
//!
//! Por que afim e não só `Vec2`: a pose nasceu como translação (mover uma instância), mas
//! o passo seguinte é **girar/escalar** essa instância pelo gizmo. Guardar já como afim
//! evita rebumpar o schema quando o gizmo chegar — a translação de hoje é a identidade com
//! `(tx, ty)`, byte-idêntica ao `Vec2` de antes. A COMPOSIÇÃO de rotação/escala (que precisa
//! de multiplicação de afim e de um pivô) mora no shell, que já tem o `Xform`; aqui a `Pose`
//! só ARMAZENA os coeficientes e faz as operações triviais (translação, aplicar em ponto).
//!
//! Identidade = a arte onde foi desenhada (o caminho comum não paga nada).

use ph2d_core::Vec2;
use serde::{Deserialize, Serialize};

/// Afim 2×3 da pose de uma chave (`[a, b, c, d, tx, ty]`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pose(pub [f32; 6]);

impl Default for Pose {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Pose {
    /// A pose neutra — a arte onde foi desenhada.
    pub const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    /// Uma pose de translação pura (o que a pose foi até o gizmo de rotate/escala).
    #[must_use]
    pub fn from_translation(t: Vec2) -> Self {
        Self([1.0, 0.0, 0.0, 1.0, t.x, t.y])
    }

    /// Constrói a partir dos coeficientes crus (a ponte com o `Xform` do shell).
    #[must_use]
    pub fn from_coeffs(c: [f32; 6]) -> Self {
        Self(c)
    }

    /// Os coeficientes crus (para o shell montar o `Xform`).
    #[must_use]
    pub fn coeffs(&self) -> [f32; 6] {
        self.0
    }

    /// É a pose neutra? (o caminho comum — o render/hit não pagam nada).
    #[must_use]
    pub fn is_identity(&self) -> bool {
        self.0 == Self::IDENTITY.0
    }

    /// A translação da pose (`(tx, ty)`).
    #[must_use]
    pub fn translation(&self) -> Vec2 {
        Vec2::new(self.0[4], self.0[5])
    }

    /// **Pós-translada** por `d` — compõe uma translação no espaço EXTERNO (do objeto), que
    /// é o que o move de instância faz: o delta do arrasto desloca a arte posada inteira.
    /// Numa pose de translação pura reduz a `offset += d`, byte-idêntico ao pré-afim.
    pub fn translate(&mut self, d: Vec2) {
        self.0[4] += d.x;
        self.0[5] += d.y;
    }

    /// Afim·ponto — leva um ponto da geometria LOCAL ao lugar em que a chave o mostra.
    /// MESMA convenção do `Xform::apply` do shell (o par render/hit não pode divergir).
    #[must_use]
    pub fn apply(&self, p: Vec2) -> Vec2 {
        Vec2::new(
            self.0[0] * p.x + self.0[2] * p.y + self.0[4],
            self.0[1] * p.x + self.0[3] * p.y + self.0[5],
        )
    }

    /// Interpolação componente-a-componente (o tween). Crua, mas correta para a translação;
    /// rot/escala intermediárias saem de um lerp de matriz (aceitável para inbetweens).
    #[must_use]
    pub fn lerp(&self, other: &Pose, t: f32) -> Pose {
        Pose(std::array::from_fn(|i| {
            self.0[i] + (other.0[i] - self.0[i]) * t
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A identidade é a arte onde foi desenhada — aplica sem mover, e se declara neutra.
    #[test]
    fn identity_is_the_neutral_pose() {
        let p = Pose::IDENTITY;
        assert!(p.is_identity());
        assert_eq!(p.apply(Vec2::new(3.0, -7.0)), Vec2::new(3.0, -7.0));
        assert_eq!(p.translation(), Vec2::ZERO);
    }

    /// 🔴 **Uma pose de translação pura reduz a `offset += d`** — o caminho de hoje, byte a
    /// byte (o afim é um superconjunto, não uma mudança de comportamento). Mutação que
    /// sangra: pós-transladar em `[0]/[1]` (a base linear) em vez de `[4]/[5]`.
    #[test]
    fn a_translation_pose_matches_the_old_offset_math() {
        let mut p = Pose::from_translation(Vec2::new(10.0, 20.0));
        p.translate(Vec2::new(3.0, -5.0));
        assert_eq!(p.translation(), Vec2::new(13.0, 15.0));
        // E aplica como translação: o ponto anda o (tx,ty), sem rodar/escalar.
        assert_eq!(p.apply(Vec2::new(1.0, 1.0)), Vec2::new(14.0, 16.0));
    }

    /// A `apply` da `Pose` usa a MESMA convenção do `Xform` do shell (`a·x + c·y + tx`) — é
    /// o que impede o `posed_bbox` (aqui) de divergir do render (lá).
    #[test]
    fn apply_matches_the_xform_convention() {
        // Afim arbitrário (girado/escalado): [a,b,c,d,tx,ty].
        let p = Pose([0.6, 0.8, -0.8, 0.6, 30.0, -12.0]);
        let q = p.apply(Vec2::new(10.0, 5.0));
        // a·x + c·y + tx, b·x + d·y + ty.
        assert!((q.x - (0.6 * 10.0 - 0.8 * 5.0 + 30.0)).abs() < 1e-5);
        assert!((q.y - (0.8 * 10.0 + 0.6 * 5.0 - 12.0)).abs() < 1e-5);
    }
}
