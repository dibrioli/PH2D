//! **A CÂMERA do traçador** — irmã de [`super`], cortada por RESPONSABILIDADE quando o teto de LOC
//! de 700 disparou.
//!
//! ⚠️ O corte é de **arquivo**, nunca de superfície: `Orbit` continua a ser
//! `ph2d_field_render::Orbit` para todo o mundo (o `pub use` no pai), como o `sculpt3d_rulers` fez
//! com as réguas dele. Um corte que mudasse caminhos seria uma migração disfarçada de arrumação.

/// A câmera. **Ortográfica**, e a orientação é um **quaternion**.
///
/// # ⭐ Por que não é `yaw`/`pitch`
///
/// Uma câmera de dois ângulos tem **polos por construção**: a elevação satura em ±90°, e a partir
/// dali arrastar na vertical não faz nada. Com o enquadramento inicial já a 30° de cima, meio
/// centímetro de rato para baixo bate na parede — e o que o artista vê é *"só roda para um lado"*
/// (Enio, 2026-08-19). A câmera da casa (`ph2d_mesh_render::camera`) tem exatamente o mesmo teto,
/// e **prende-o com um `clamp`**.
///
/// Um `clamp` é o remédio para o sintoma. A causa é a **representação**: dois ângulos não conseguem
/// exprimir uma orientação livre, então nenhum número melhor a devolve. Guardando a orientação
/// inteira, a rotação passa a ser *uma* composição de quaternions — sem polo, sem `clamp`, sem caso
/// especial, e sem o eixo vertical do mundo a decidir o que a mão pode fazer.
///
/// ⚠️ O preço é real e está aceite: **o horizonte deixa de ser fixo**. Uma câmera de dois ângulos
/// nunca inclina; esta inclina, porque é isso que *rotação livre* significa. A volta é
/// [`Orbit::from_yaw_pitch`], que é o que a tecla de repor a vista chama.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Orbit {
    /// A orientação, como quaternion `(x, y, z, w)`: leva os eixos **locais** da câmera para o
    /// mundo.
    pub rotation: [f32; 4],
    /// Quantas unidades de mundo cabem em meia altura de tela. Menor = mais perto.
    pub half_extent: f32,
    /// O ponto que fica no centro do quadro.
    pub target: [f32; 3],
}

impl Default for Orbit {
    fn default() -> Self {
        // Três-quartos, ligeiramente por cima: o ângulo em que uma aresta viva e um filete se
        // distinguem sem ambiguidade (escolhido na W0, ao olhar as imagens).
        Self {
            rotation: Self::from_yaw_pitch(0.72, 0.52).rotation,
            half_extent: 0.8,
            target: [0.0; 3],
        }
    }
}

impl Orbit {
    /// A orientação que os dois ângulos de uma câmera de prato giratório dariam.
    ///
    /// Continua a existir por duas razões, e nenhuma é nostalgia: é como se escreve um
    /// **enquadramento nomeado** (o inicial, a vista de frente, a de topo), e é o que **repõe** a
    /// vista depois de a rotação livre a ter inclinado.
    #[must_use]
    pub fn from_yaw_pitch(yaw: f32, pitch: f32) -> Self {
        // `R = Ry(yaw) · Rx(−pitch)` — a composição que reproduz exatamente a base antiga
        // (`fwd = (cos p·sin y, sin p, cos p·cos y)`), verificada por gate.
        let (sy, cy) = (yaw * 0.5).sin_cos();
        let (sp, cp) = (-pitch * 0.5).sin_cos();
        Self {
            rotation: quat_mul([0.0, sy, 0.0, cy], [sp, 0.0, 0.0, cp]),
            half_extent: 0.8,
            target: [0.0; 3],
        }
    }

    /// A base ortonormal da câmera: `(direita, cima, para-o-observador)`.
    ///
    /// ⚠️ **Projeção ortográfica**, e isso é uma escolha com data: é a que a W0 validou, e é a que
    /// o matcap pressupõe (`ph2d-mesh-render::matcap` amostra pela normal de vista, com a vista em
    /// `(0,0,1)`). Perspectiva é item ABERTO — ela muda o *feel* de um modelador e merece a sua
    /// própria comparação lado a lado, não uma troca silenciosa.
    ///
    /// ⚠️ A trigonometria daqui **não** fere o HR-5: a câmera é estado de VISTA — não entra no
    /// documento salvo, não entra no undo e não entra em hash de replay nenhum.
    #[must_use]
    pub fn basis(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let q = self.rotation;
        (
            quat_rotate(q, [1.0, 0.0, 0.0]),
            quat_rotate(q, [0.0, 1.0, 0.0]),
            quat_rotate(q, [0.0, 0.0, 1.0]),
        )
    }

    /// ⭐ **Rotação LIVRE**: gira em torno de um eixo dado nas coordenadas da **própria câmera**.
    ///
    /// É a composição pela direita (`q ⊗ Δ`), e é ela que faz a rotação ser local — o eixo é o que
    /// o gesto nomeia na tela, e não um eixo do mundo. Daí não haver polo: nenhum eixo do mundo
    /// participa da conta.
    pub fn turn_local(&mut self, axis: [f32; 3], angle: f32) {
        self.rotation = quat_normalize(quat_mul(self.rotation, quat_axis_angle(axis, angle)));
    }

    /// Gira em torno de um eixo do **mundo** (composição pela esquerda) — o prato giratório.
    pub fn turn_world(&mut self, axis: [f32; 3], angle: f32) {
        self.rotation = quat_normalize(quat_mul(quat_axis_angle(axis, angle), self.rotation));
    }
}

/// `a ⊗ b` — aplicar `b` **depois** de `a` no referencial de `a`.
fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let ([ax, ay, az, aw], [bx, by, bz, bw]) = (a, b);
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

fn quat_axis_angle(axis: [f32; 3], angle: f32) -> [f32; 4] {
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len <= 0.0 || !len.is_finite() {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let (s, c) = (angle * 0.5).sin_cos();
    [axis[0] / len * s, axis[1] / len * s, axis[2] / len * s, c]
}

/// ⚠️ **Re-normalizar a cada giro não é zelo.** Uma rotação livre é uma composição *acumulada*: um
/// arrasto longo são centenas de multiplicações, e o erro de `f32` faz a norma derivar. Um
/// quaternion que deixa de ser unitário deixa de ser uma rotação — ele passa a **escalar** a peça,
/// e o sintoma é a forma a encolher devagar enquanto se gira.
fn quat_normalize(q: [f32; 4]) -> [f32; 4] {
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if n <= 0.0 || !n.is_finite() {
        return [0.0, 0.0, 0.0, 1.0];
    }
    [q[0] / n, q[1] / n, q[2] / n, q[3] / n]
}

fn quat_rotate(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    // `v + 2·w·(u×v) + 2·u×(u×v)`, com `u` a parte vetorial — a forma sem construir a matriz.
    let u = [q[0], q[1], q[2]];
    let cross = |a: [f32; 3], b: [f32; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let t = cross(u, v);
    let tt = cross(u, t);
    [
        v[0] + 2.0 * (q[3] * t[0] + tt[0]),
        v[1] + 2.0 * (q[3] * t[1] + tt[1]),
        v[2] + 2.0 * (q[3] * t[2] + tt[2]),
    ]
}
