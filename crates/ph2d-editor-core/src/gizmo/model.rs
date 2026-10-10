//! **A afim do PAI de um gizmo** ([`super::GizmoView::model`]) — a caixa e o pivô vivem no espaço
//! local de um pai que pode ter escala NÃO uniforme, e a rotação do filho por cima dela dá um
//! CISALHAMENTO que nenhum `(centro, meia-extensão, rotação)` exprime. A pintura projecta
//! `câmara · model · p`, então a caixa mostrada é o paralelogramo exacto; o arrasto lê o cursor
//! pela inversa ([`GizmoCamera::screen_in_model`]) e corre a matemática de sempre no espaço local.
//!
//! O formato é o do `Camera2d::model` (`[a, b, c, d, tx, ty]`, colunas `(a, b)` e `(c, d)`):
//! `p' = (a·x + c·y + tx, b·x + d·y + ty)`. ⭐ **Identidade = o caminho de antes, bit a bit**: as
//! duas portas devolvem a entrada sem a tocar (`1·x + 0·y + 0` já trocaria um `-0.0` por `0.0`).

use super::camera::{GizmoCamera, world_to_screen_px};

/// A afim neutra — a de todo gizmo cujo bbox já é de mundo.
pub const MODEL_IDENTITY: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// `model · p`.
#[must_use]
pub fn model_apply(model: &[f32; 6], p: [f32; 2]) -> [f32; 2] {
    if *model == MODEL_IDENTITY {
        return p;
    }
    let m = model;
    [
        m[0] * p[0] + m[2] * p[1] + m[4],
        m[1] * p[0] + m[3] * p[1] + m[5],
    ]
}

/// A inversa, ou `None` quando a parte linear é singular (um pai de escala zero não tem espaço
/// local para onde levar o cursor).
#[must_use]
pub fn model_inverse(model: &[f32; 6]) -> Option<[f32; 6]> {
    let m = model;
    let det = m[0] * m[3] - m[2] * m[1];
    if !det.is_normal() {
        return None;
    }
    let (a, b, c, d) = (m[3] / det, -m[1] / det, -m[2] / det, m[0] / det);
    Some([a, b, c, d, -(a * m[4] + c * m[5]), -(b * m[4] + d * m[5])])
}

/// ⭐ **O SKEW de um objecto como `model`** — `L = R·Sk·R⁻¹` em torno do `pivot` de mundo. O motor
/// desenha `T·R·Sk·S` (ADR-0025 emenda 1 §2.2) e compõe pai×filho por soma de rotações e produto
/// de escalas (§2.2.1): um filho rodado sob um pai de escala não uniforme NÃO cisalha; só o skew
/// cisalha. A caixa vive no espaço sem skew (`T·R·S`) e `model` leva-a ao paralelogramo pintado.
/// Skew nulo ⇒ [`MODEL_IDENTITY`], o caminho de antes.
#[must_use]
pub fn skew_about(pivot: [f32; 2], rotation: f32, skew: [f32; 2]) -> [f32; 6] {
    if skew == [0.0, 0.0] {
        return MODEL_IDENTITY;
    }
    let (s, c) = libm::sincosf(rotation);
    let (a, b) = (libm::tanf(skew[0]), libm::tanf(skew[1]));
    // R·Sk (colunas), depois ·Rᵀ.
    let (m00, m10, m01, m11) = (c - s * b, s + c * b, c * a - s, s * a + c);
    let l = [
        m00 * c - m01 * s,
        m10 * c - m11 * s,
        m00 * s + m01 * c,
        m10 * s + m11 * c,
    ];
    [
        l[0],
        l[1],
        l[2],
        l[3],
        pivot[0] - (l[0] * pivot[0] + l[2] * pivot[1]),
        pivot[1] - (l[1] * pivot[0] + l[3] * pivot[1]),
    ]
}

/// A translação LOCAL que o motor do gizmo escreveu no espaço sem skew, levada ao pintado: o ponto
/// de mundo `pai·t` passa por `model` e volta ao pai. Escalar e mover o pivô correm sem skew com o
/// cursor pela inversa ([`GizmoCamera::screen_in_model`]); sem esta volta a caixa fixaria o canto
/// oposto do RETÂNGULO, e não o do paralelogramo. Identidade ⇒ `local` intacto.
#[must_use]
pub fn skew_translation_out(
    model: &[f32; 6],
    parent: super::TransformSnapshot,
    local: [f32; 2],
) -> [f32; 2] {
    if *model == MODEL_IDENTITY {
        return local;
    }
    let child = super::TransformSnapshot {
        translation: local,
        ..super::TransformSnapshot::IDENTITY
    };
    let u = super::compose_snapshot(parent, child).translation;
    let w = model_apply(model, u);
    let d = super::world_delta_to_local(parent, w[0] - u[0], w[1] - u[1]);
    [local[0] + d[0], local[1] + d[1]]
}

impl GizmoCamera {
    /// **O cursor visto do espaço do modelo**: o pixel que ESTA câmara leria no ponto local
    /// `model⁻¹ · mundo(px)`. Um arrasto que passa por aqui corre o motor canónico
    /// ([`super::compute_gizmo_transform`], o contador de voltas) inteiro no espaço local, sem
    /// nenhum campo novo na câmara. Identidade ou modelo singular ⇒ `px` intacto.
    #[must_use]
    pub fn screen_in_model(&self, model: &[f32; 6], px: (f32, f32)) -> (f32, f32) {
        if *model == MODEL_IDENTITY {
            return px;
        }
        let Some(inv) = model_inverse(model) else {
            return px;
        };
        let local = model_apply(&inv, self.screen_to_world(px));
        let s = world_to_screen_px(
            self.center,
            self.height_world,
            self.window_w,
            self.window_h,
            local,
        );
        (s[0], s[1])
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
