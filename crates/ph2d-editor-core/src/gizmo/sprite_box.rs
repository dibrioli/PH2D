//! **A caixa do gizmo de uma SPRITE** — da afim de mundo que o motor DESENHA (`T·R·Sk·S`) à
//! [`GizmoView`]: a caixa no espaço sem skew (`T·R·S`) e o skew em [`GizmoView::model`]
//! ([`super::skew_about`]). Sem skew, a conta de sempre e `model` identidade, bit a bit.
//!
//! ⚠️ Um pai de escala NÃO uniforme não cisalha o filho: o motor compõe somando rotações e
//! multiplicando escalas (ADR-0025 emenda 1 §2.2.1). Só o skew faz da sprite um paralelogramo
//! (doc ObjectsAndComponents 01 §15, onda 5 item a).

use super::model::{MODEL_IDENTITY, model_apply, model_inverse, skew_about};
use super::{GizmoCamera, GizmoDragKind, GizmoDragState, GizmoTarget, GizmoView, compose_snapshot};

/// A geometria de uma caixa de sprite — os campos de mundo da [`GizmoView`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SpriteBox {
    pub bbox_min_world: [f32; 2],
    pub bbox_max_world: [f32; 2],
    pub pivot_world: [f32; 2],
    pub rotation: f32,
    pub model: [f32; 6],
}

/// A caixa de uma sprite desenhada pela afim `affine` (`[a, b, c, d, tx, ty]`) com o skew de mundo
/// `skew`, a âncora efectiva `eff_anchor` e a meia-extensão `half` (intrínsecas, da
/// `sheet_lattice::gizmo_box`).
#[must_use]
pub fn sprite_box(
    affine: [f32; 6],
    skew: [f32; 2],
    eff_anchor: [f32; 2],
    half: [f32; 2],
) -> SpriteBox {
    let p = [affine[4], affine[5]];
    // Sem skew o `model` é a identidade e as colunas ficam as do motor. Com skew, `R·S = L⁻¹·A`:
    // o ângulo da 1.ª coluna é `θ + atan(tan k_y)`.
    let (model, cols) = if skew == [0.0, 0.0] {
        (MODEL_IDENTITY, [affine[0], affine[1], affine[2], affine[3]])
    } else {
        let theta = affine[1].atan2(affine[0]) - libm::atanf(libm::tanf(skew[1]));
        let m = skew_about(p, theta, skew);
        let inv = model_inverse(&[m[0], m[1], m[2], m[3], 0.0, 0.0]).unwrap_or(MODEL_IDENTITY);
        let c0 = model_apply(&inv, [affine[0], affine[1]]);
        let c1 = model_apply(&inv, [affine[2], affine[3]]);
        (m, [c0[0], c0[1], c1[0], c1[1]])
    };
    let [col0_x, col0_y, col1_x, col1_y] = cols;
    let scale_x = (col0_x * col0_x + col0_y * col0_y).sqrt();
    let scale_y = (col1_x * col1_x + col1_y * col1_y).sqrt();
    let rotation = col0_y.atan2(col0_x);
    let half_w = half[0] * scale_x;
    let half_h = half[1] * scale_y;
    let ax = eff_anchor[0] * scale_x;
    let ay = eff_anchor[1] * scale_y;
    // T1.3.5 cross-OS bit-identical.
    let (sin_r, cos_r) = libm::sincosf(rotation);
    let cx = p[0] + ax * cos_r - ay * sin_r;
    let cy = p[1] + ax * sin_r + ay * cos_r;
    SpriteBox {
        bbox_min_world: [cx - half_w, cy - half_h],
        bbox_max_world: [cx + half_w, cy + half_h],
        pivot_world: p,
        rotation,
        model,
    }
}

/// O `model` com que um arrasto de sprite lê o cursor: o skew em torno da pose de PARTIDA. Só
/// escalar e mover o pivô o pedem — transladar e rodar são rígidos sobre o pintado, e a conta de
/// mundo já é exacta; o gizmo GLOBAL é de mundo. Skew nulo ⇒ identidade.
#[must_use]
pub fn drag_skew_model(drag: &GizmoDragState, skew: [f32; 2]) -> [f32; 6] {
    let individual = matches!(
        drag.target,
        GizmoTarget::PrimaryIndividual | GizmoTarget::ExtraIndividual(_)
    );
    let pede = matches!(
        drag.kind,
        GizmoDragKind::ScaleCorner { .. }
            | GizmoDragKind::ScaleEdge { .. }
            | GizmoDragKind::MovePivot
    );
    if !(individual && pede) {
        return MODEL_IDENTITY;
    }
    let mundo = compose_snapshot(drag.parent_world, drag.start_transform);
    skew_about(mundo.translation, mundo.rotation, skew)
}

/// O arrasto visto do espaço sem skew: o cursor de agora e o do pen-down pela inversa de `model`.
/// O pivô já lá está — sai da pose TRS de partida (`anchor_pivot_world`, `live_anchor`).
#[must_use]
pub fn drag_in_skew_frame(
    drag: GizmoDragState,
    model: &[f32; 6],
    cam: &GizmoCamera,
) -> GizmoDragState {
    let Some(inv) = model_inverse(model).filter(|_| *model != MODEL_IDENTITY) else {
        return drag;
    };
    GizmoDragState {
        cursor_screen: cam.screen_in_model(model, drag.cursor_screen),
        start_cursor_world: model_apply(&inv, drag.start_cursor_world),
        ..drag
    }
}

impl GizmoView {
    /// A caixa alinhada aos eixos de MUNDO que envolve os quatro cantos PINTADOS — rodados por
    /// `rotation` em torno do centro e levados por `model`. A união do gizmo global lia os campos
    /// crus e deixava de fora os cantos de uma vista rodada. Sem rotação nem `model` ⇒ os campos
    /// intactos, o caminho de antes.
    #[must_use]
    pub fn world_aabb(&self) -> ([f32; 2], [f32; 2]) {
        if self.model == MODEL_IDENTITY && self.rotation == 0.0 {
            return (self.bbox_min_world, self.bbox_max_world);
        }
        let (lo, hi) = (self.bbox_min_world, self.bbox_max_world);
        let c = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
        let h = [(hi[0] - lo[0]) * 0.5, (hi[1] - lo[1]) * 0.5];
        let (s, co) = libm::sincosf(self.rotation);
        let mut min = [f32::INFINITY; 2];
        let mut max = [f32::NEG_INFINITY; 2];
        for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            let (lx, ly) = (sx * h[0], sy * h[1]);
            let w = model_apply(
                &self.model,
                [c[0] + lx * co - ly * s, c[1] + lx * s + ly * co],
            );
            for k in 0..2 {
                min[k] = min[k].min(w[k]);
                max[k] = max[k].max(w[k]);
            }
        }
        (min, max)
    }
}

#[cfg(test)]
#[path = "sprite_box_tests.rs"]
mod tests;
