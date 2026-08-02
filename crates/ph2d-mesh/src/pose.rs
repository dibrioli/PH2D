//! **ONDE uma malha está** — a pose de um objeto na cena.
//!
//! Até a W8 a cena tinha uma malha só, e por isso *"onde"* não era uma pergunta:
//! a geometria era o mundo. Com uma LISTA de objetos ela passa a ser, e a
//! resposta é este tipo — a transformação que leva o espaço **local** de uma
//! malha ao espaço de **mundo** que a câmera enxerga.
//!
//! ⚠️ **Ela é TRANSLAÇÃO + ESCALA UNIFORME, e a limitação é deliberada — não é
//! um degrau que ficou pela metade.** Rotacionar um objeto inteiro é um GESTO, e
//! esse gesto não existe (não há gizmo 3D); um campo de rotação hoje seria um
//! número que ninguém consegue autorar, guardado, multiplicado em todo frame e
//! sempre igual à identidade — a *capacidade sem porta* que este módulo já
//! recusou uma vez (a `sculpt_kernel_device` da W1). Quando o gizmo chegar, ele
//! entra **aqui dentro**: os call sites falam com [`Pose::point_to_local`] e
//! [`Pose::vector_to_local`], que é onde uma rotação pousa sem que nenhum deles
//! mude.
//!
//! ⚠️ **E a escala é UNIFORME por uma razão que atravessa o shader:** o passe da
//! malha leva a normal por `view * model` com `w = 0` e apenas normaliza
//! (`mesh.wgsl`, `canvas_normal`), o que é exato enquanto a parte linear for uma
//! **similaridade** — uma escala não-uniforme exigiria o inverso-transposto, e o
//! sintoma de esquecê-lo é uma luz sutilmente torta que ninguém sabe nomear. Uma
//! escala por-eixo é, por isso, uma decisão de renderização e não um campo a
//! mais.

use crate::aabb::Aabb;
use crate::ray::Ray;

/// O menor fator de escala que uma pose aceita.
///
/// ⚠️ **É um piso de REPRESENTAÇÃO, não de gosto:** [`Pose::point_to_local`]
/// divide pela escala, então zero manda todo ponto do mundo para o infinito e
/// negativo espelha a malha (o que, com `cull_mode: None`, sai sem sintoma na
/// tela e com a normal invertida por baixo). Um milionésimo é pequeno o
/// bastante para nenhum gesto plausível o alcançar e grande o bastante para a
/// divisão continuar finita em `f32`.
const MIN_SCALE: f32 = 1.0e-6;

/// A pose de um objeto: onde ele está e quão grande ele é.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    /// A origem do objeto, em mundo.
    pub translation: [f32; 3],
    /// O fator uniforme. Sempre `>= MIN_SCALE` — ver [`Pose::new`].
    scale: f32,
}

impl Default for Pose {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Pose {
    /// O objeto na origem, em tamanho natural. É a pose de toda malha que
    /// existia antes desta wave, e é o que torna a lista de um objeto
    /// **byte-idêntica** ao mundo em que a geometria era o mundo.
    pub const IDENTITY: Self = Self {
        translation: [0.0; 3],
        scale: 1.0,
    };

    /// Constrói, **clampando a escala** ao [`MIN_SCALE`].
    ///
    /// ⚠️ Clampar em vez de recusar é a escolha certa aqui porque o chamador é
    /// um gesto: um arrasto que passe por zero tem de continuar sendo um
    /// arrasto, e não um `Option` que ele precisa desembrulhar no meio do frame.
    /// Uma escala não-finita cai no mesmo piso (`NaN` falha toda comparação, e o
    /// `max` do Rust devolve o outro operando).
    #[must_use]
    pub fn new(translation: [f32; 3], scale: f32) -> Self {
        Self {
            translation,
            scale: if scale.is_finite() {
                scale.max(MIN_SCALE)
            } else {
                MIN_SCALE
            },
        }
    }

    /// O objeto em `translation`, em tamanho natural.
    #[must_use]
    pub fn at(translation: [f32; 3]) -> Self {
        Self::new(translation, 1.0)
    }

    /// O fator uniforme, já clampado.
    #[must_use]
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Local → mundo, para um PONTO (a translação entra).
    #[must_use]
    pub fn point_to_world(&self, p: [f32; 3]) -> [f32; 3] {
        [
            p[0] * self.scale + self.translation[0],
            p[1] * self.scale + self.translation[1],
            p[2] * self.scale + self.translation[2],
        ]
    }

    /// Mundo → local, para um PONTO.
    #[must_use]
    pub fn point_to_local(&self, p: [f32; 3]) -> [f32; 3] {
        [
            (p[0] - self.translation[0]) / self.scale,
            (p[1] - self.translation[1]) / self.scale,
            (p[2] - self.translation[2]) / self.scale,
        ]
    }

    /// Local → mundo, para um VETOR (deslocamento ou direção: a translação
    /// **não** entra).
    ///
    /// ⚠️ Um deslocamento e uma direção são o mesmo tipo aqui de propósito: sob
    /// uma similaridade a escala só muda o COMPRIMENTO, então quem quer uma
    /// direção normaliza e quem quer um deslocamento não — duas funções seriam
    /// duas respostas para uma aritmética só.
    #[must_use]
    pub fn vector_to_world(&self, v: [f32; 3]) -> [f32; 3] {
        [v[0] * self.scale, v[1] * self.scale, v[2] * self.scale]
    }

    /// Mundo → local, para um VETOR.
    #[must_use]
    pub fn vector_to_local(&self, v: [f32; 3]) -> [f32; 3] {
        [v[0] / self.scale, v[1] / self.scale, v[2] / self.scale]
    }

    /// **O raio, no espaço da malha** — a porta que impede o defeito clássico de
    /// uma cena com pose: o pincel pousar onde o objeto ESTAVA.
    ///
    /// ⚠️ **Transformar o RAIO, e não a malha, é a decisão inteira.** O caminho
    /// oposto (levar a geometria ao mundo antes de consultar) custaria uma cópia
    /// da malha por consulta e invalidaria o octree, que é construído em espaço
    /// local. Aqui a consulta continua sendo exatamente a que a W1 gateou, e o
    /// acerto volta em coordenadas locais — que são as coordenadas em que o dab
    /// escreve.
    ///
    /// ⚠️ **O `t` do acerto passa a ser uma distância LOCAL** (o [`Ray`]
    /// normaliza a direção na construção, então ele mede em unidades de local),
    /// e é por isso que comparar dois objetos pelo `t` deles seria errado sob
    /// escalas diferentes: quem compara converte o PONTO de volta ao mundo.
    #[must_use]
    pub fn ray_to_local(&self, ray: &Ray) -> Ray {
        Ray::new(
            self.point_to_local(ray.origin()),
            self.vector_to_local(ray.dir()),
        )
    }

    /// A caixa da malha, em mundo.
    ///
    /// Sem rotação a caixa transformada é exata (nenhum canto sai do eixo), o
    /// que a torna utilizável para enquadrar a câmera sem folga inventada.
    #[must_use]
    pub fn bounds_to_world(&self, b: Aabb) -> Aabb {
        if b.is_empty() {
            return b;
        }
        Aabb {
            min: self.point_to_world(b.min),
            max: self.point_to_world(b.max),
        }
    }

    /// A matriz local → mundo em **coluna-major**, do jeito que o WGSL a lê.
    ///
    /// ⚠️ A convenção é a mesma do `glam::Mat4::to_cols_array_2d` que o uniform
    /// da câmera já usa, e é ela que o `camera_uniform_layout` gateia — as duas
    /// matrizes do mesmo shader não podem discordar sobre o que é uma coluna.
    #[must_use]
    pub fn to_cols_array_2d(&self) -> [[f32; 4]; 4] {
        let s = self.scale;
        let t = self.translation;
        [
            [s, 0.0, 0.0, 0.0],
            [0.0, s, 0.0, 0.0],
            [0.0, 0.0, s, 0.0],
            [t[0], t[1], t[2], 1.0],
        ]
    }
}

#[cfg(test)]
#[path = "pose_tests.rs"]
mod tests;
