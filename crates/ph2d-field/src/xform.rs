//! **A pose de um nó**, e a aritmética que a compõe.
//!
//! # Por que a aritmética de quaternion mora AQUI
//!
//! Ela mora onde mora o tipo que a usa. A rotação de um [`Xform`] **é** um quaternion, então
//! multiplicar duas poses é aritmética do documento — e uma segunda cópia dela noutra crate seria
//! duas respostas para *"qual é a pose resultante?"*, com a chance normal de divergirem numa das
//! duas.
//!
//! É o que já estava a acontecer: a câmera do traçador tinha o seu próprio `quat_mul`. Ela passou a
//! ler estas ([`quat_mul`], [`quat_rotate`], …), porque uma composição de rotações não muda de
//! resposta por ser de câmera ou de peça.

use serde::{Deserialize, Serialize};

/// Pose de um nó: translação, rotação e escala **uniforme**.
///
/// ⚠️ **É LOCAL, relativa ao pai** — a pose de mundo obtém-se compondo a cadeia
/// ([`Xform::compose`]). É a lei da casa para o vetorial, dita por extenso: *o que se vê e se
/// aponta é MUNDO; o que o documento guarda é LOCAL.*
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Xform {
    pub translation: [f32; 3],
    /// Quaternion `(x, y, z, w)`.
    pub rotation: [f32; 4],
    /// ⛔ **UNIFORME de propósito.** Escala não-uniforme **destrói a propriedade de distância**
    /// (‖∇f‖ = 1), que é a fundação de tudo neste módulo: sem ela o raio deixa de ser o raio, a
    /// casca perde a espessura e a marcha de raios atravessa a superfície. Quem quer um elipsoide
    /// usa uma primitiva de elipsoide — não uma esfera esticada (ADR-0161 §6).
    pub scale: f32,
}

impl Default for Xform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Xform {
    pub const IDENTITY: Self = Self {
        translation: [0.0; 3],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: 1.0,
    };

    #[must_use]
    pub fn at(x: f32, y: f32, z: f32) -> Self {
        Self {
            translation: [x, y, z],
            ..Self::IDENTITY
        }
    }

    /// Leva um ponto do espaço **local** deste nó para o espaço do **pai**: `t + R·(s·p)`.
    ///
    /// ⚠️ É a inversa exacta do que o avaliador faz ao campo (`p' = R⁻¹(p − t)/s`), e tem de o ser:
    /// se as duas contas discordarem, a alça do gizmo pousa num sítio e a superfície aparece noutro.
    /// O gate `the_gizmo_and_the_field_agree_on_where_a_node_is` prende as duas juntas.
    #[must_use]
    pub fn apply(self, p: [f32; 3]) -> [f32; 3] {
        let s = [p[0] * self.scale, p[1] * self.scale, p[2] * self.scale];
        let r = quat_rotate(self.rotation, s);
        [
            r[0] + self.translation[0],
            r[1] + self.translation[1],
            r[2] + self.translation[2],
        ]
    }

    /// Leva uma **direção** (sem translação) do local para o pai. Escala junto: uma seta de gizmo
    /// desenhada num nó escalado 2× tem de medir 2× no mundo, senão ela mente sobre o que arrasta.
    #[must_use]
    pub fn apply_dir(self, d: [f32; 3]) -> [f32; 3] {
        quat_rotate(
            self.rotation,
            [d[0] * self.scale, d[1] * self.scale, d[2] * self.scale],
        )
    }

    /// `self ∘ child` — a pose de `child` expressa no referencial do **pai de `self`**.
    ///
    /// ⭐ É a única porta para descer a hierarquia. `translation` compõe por [`Xform::apply`],
    /// `rotation` por [`quat_mul`], `scale` por produto — e é essa terceira que costuma faltar:
    /// sem ela um filho dentro de um pai escalado fica no sítio certo com o tamanho errado.
    #[must_use]
    pub fn compose(self, child: Self) -> Self {
        Self {
            translation: self.apply(child.translation),
            rotation: quat_normalize(quat_mul(self.rotation, child.rotation)),
            scale: self.scale * child.scale,
        }
    }
}

/// `a ⊗ b` — aplicar `b` **depois** de `a`, no referencial de `a`.
#[must_use]
pub fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let ([ax, ay, az, aw], [bx, by, bz, bw]) = (a, b);
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

/// O quaternion de uma rotação de `angle` em torno de `axis`. Eixo degenerado ⇒ identidade, que é
/// a única resposta honesta (rodar em torno de nada é não rodar).
#[must_use]
pub fn quat_axis_angle(axis: [f32; 3], angle: f32) -> [f32; 4] {
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len <= 0.0 || !len.is_finite() {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let (s, c) = (angle * 0.5).sin_cos();
    [axis[0] / len * s, axis[1] / len * s, axis[2] / len * s, c]
}

/// ⚠️ **Re-normalizar a cada composição não é zelo.** Uma orientação acumulada são centenas de
/// multiplicações, e o erro de `f32` faz a norma derivar. Um quaternion que deixa de ser unitário
/// deixa de ser uma rotação — ele passa a **escalar**, e o sintoma é a forma a encolher devagar.
#[must_use]
pub fn quat_normalize(q: [f32; 4]) -> [f32; 4] {
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if n <= 0.0 || !n.is_finite() {
        return [0.0, 0.0, 0.0, 1.0];
    }
    [q[0] / n, q[1] / n, q[2] / n, q[3] / n]
}

/// Roda um vetor: `v + 2·w·(u×v) + 2·u×(u×v)`, com `u` a parte vetorial — a forma sem construir a
/// matriz.
#[must_use]
pub fn quat_rotate(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let u = [q[0], q[1], q[2]];
    let t = cross(u, v);
    let tt = cross(u, t);
    [
        v[0] + 2.0 * (q[3] * t[0] + tt[0]),
        v[1] + 2.0 * (q[3] * t[1] + tt[1]),
        v[2] + 2.0 * (q[3] * t[2] + tt[2]),
    ]
}

#[must_use]
pub fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[must_use]
pub fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}
