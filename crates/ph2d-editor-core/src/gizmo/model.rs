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
