//! **A CÂMERA do traçador** — irmã de [`super`], cortada por RESPONSABILIDADE quando o teto de LOC
//! de 700 disparou.
//!
//! ⚠️ O corte é de **arquivo**, nunca de superfície: `Orbit` continua a ser
//! `ph2d_field_render::Orbit` para todo o mundo (o `pub use` no pai), como o `sculpt3d_rulers` fez
//! com as réguas dele. Um corte que mudasse caminhos seria uma migração disfarçada de arrumação.
//!
//! ⚠️ **A aritmética de quaternion NÃO mora aqui** — ela vive em [`ph2d_field::xform`], junto do
//! tipo cuja rotação ela compõe. Havia uma cópia local dela neste arquivo até 20/08, e uma segunda
//! resposta para *"qual é a rotação resultante?"* é a forma normal de duas metades do mesmo módulo
//! divergirem sem que nada fique vermelho.

use ph2d_field::xform::{dot, quat_axis_angle, quat_mul, quat_normalize, quat_rotate};

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

/// **O mapeamento pixel ↔ plano da câmera**, e ele é o **único** neste módulo.
///
/// ⭐ A marcha de raios constrói os raios a partir desta conta, e o gizmo projeta as alças com ela.
/// Duas cópias seriam duas respostas para *"onde este ponto cai na tela?"* — e a divergência não
/// apareceria como erro, apareceria como uma alça que se agarra meio pixel ao lado da superfície
/// que ela diz mover. É a mesma razão pela qual o shell guarda o `last_canvas` em vez de repetir a
/// aritmética do layout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Screen {
    w: f32,
    h: f32,
    /// Metade do lado **menor**, em pixels — é ele que fixa a escala, para o quadro não deformar.
    half: f32,
    half_extent: f32,
}

impl Screen {
    #[must_use]
    pub fn new(w: u32, h: u32, half_extent: f32) -> Self {
        Self {
            w: w as f32,
            h: h as f32,
            half: (w.min(h) as f32) * 0.5,
            half_extent,
        }
    }

    #[must_use]
    pub fn width(self) -> f32 {
        self.w
    }

    #[must_use]
    pub fn height(self) -> f32 {
        self.h
    }

    /// Pixel → plano da câmera, em unidades de **mundo**.
    #[must_use]
    pub fn plane_at(self, x: f32, y: f32) -> (f32, f32) {
        (
            (x - self.w * 0.5) / self.half * self.half_extent,
            -(y - self.h * 0.5) / self.half * self.half_extent,
        )
    }

    /// A inversa exacta de [`Screen::plane_at`]. O gate `a_pixel_survives_the_round_trip` prende as
    /// duas — uma inversa escrita à mão é onde um sinal trocado sobrevive anos.
    #[must_use]
    pub fn pixel_of(self, u: f32, v: f32) -> (f32, f32) {
        (
            u / self.half_extent * self.half + self.w * 0.5,
            -v / self.half_extent * self.half + self.h * 0.5,
        )
    }

    /// Quantos **pixels** mede uma unidade de mundo neste enquadramento. É o número que converte um
    /// tamanho de alça (que é de tela) num tamanho de gizmo (que é de mundo).
    #[must_use]
    pub fn px_per_world(self) -> f32 {
        self.half / self.half_extent
    }
}

impl Orbit {
    /// **Onde um ponto do mundo cai na tela**, e a que profundidade.
    ///
    /// Devolve `(pixel, profundidade)`, com a profundidade a crescer **na direção do observador** —
    /// é ela que resolve qual de duas alças sobrepostas está à frente.
    ///
    /// ⚠️ Ortográfica: não há divisão por `w`, e por isso não há ponto que a projeção não saiba
    /// responder. Quando a perspectiva entrar (item aberto), é **aqui** que ela entra — num sítio.
    #[must_use]
    pub fn project(&self, p: [f32; 3], screen: Screen) -> ([f32; 2], f32) {
        let (right, up, fwd) = self.basis();
        let v = [
            p[0] - self.target[0],
            p[1] - self.target[1],
            p[2] - self.target[2],
        ];
        let (x, y) = screen.pixel_of(dot(v, right), dot(v, up));
        ([x, y], dot(v, fwd))
    }

    /// O raio que sai de um pixel: `(origem, direção)`. A origem fica no plano da câmera que passa
    /// pelo alvo, o que a torna útil para intersectar com um plano de arrasto sem inventar um
    /// afastamento.
    #[must_use]
    pub fn ray(&self, x: f32, y: f32, screen: Screen) -> ([f32; 3], [f32; 3]) {
        let (right, up, fwd) = self.basis();
        let (u, v) = screen.plane_at(x, y);
        let o = [
            self.target[0] + right[0] * u + up[0] * v,
            self.target[1] + right[1] * u + up[1] * v,
            self.target[2] + right[2] * u + up[2] * v,
        ];
        (o, [-fwd[0], -fwd[1], -fwd[2]])
    }
}
