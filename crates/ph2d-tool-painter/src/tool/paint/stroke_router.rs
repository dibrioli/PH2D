//! **O que um Down sobre a tela SIGNIFICA quando há mais de uma figura** — o roteador do gesto, irmão
//! de [`super::stroke_multi`] (que é o que o conjunto É: parquear, ativar, apagar, re-carimbar).
//!
//! A decisão é uma escada de cinco degraus, e a ORDEM dela é a feature: editar a ativa · reativar uma
//! parqueada · parquear a ativa e começar outra. Foi o degrau 4 que quebrou em 2026-08-07 (*"se clicar
//! dentro de uma forma já desenhada, não aceita desenhar outra"*) — ele aceitava o INTERIOR de uma
//! caixa, e o degrau 5 virava inalcançável. Hoje ele pergunta ao [`super::stroke_outline`], a mesma
//! porta que desenha o contorno: *o que é desenhado é o que é clicável*.

use super::*;

impl PainterTool {
    /// The Down-time multi-shape decision (see [`Self::route_shape_pointer_multi`]): switch to a parked
    /// shape under the cursor, or park the active shape when the click is in empty space.
    pub(super) fn maybe_switch_or_new_shape(&mut self, pos: [f32; 2]) {
        // Never interrupt a Line that is still placing its corner points (multi-Down authoring).
        if self.paint.line.as_ref().is_some_and(|l| !l.is_editing()) {
            return;
        }
        let tol = self.paint.shape_grab_tol_px.max(6.0);
        // The active shape's centre square (its op type-square) is an edit target (tap = cycle op, drag =
        // move) — never treat it as empty space.
        if self.on_active_centre_square(pos, tol * 2.0) {
            return;
        }
        // A click that would EDIT the ACTIVE shape (grab a handle / point / gizmo, or land on its outline)
        // is left to the per-type editor — never park it. This precise per-editor test (not a coarse bbox)
        // is why grabbing a rotate ring that sits OUTSIDE the outline still counts as editing.
        if self.active_shape_hit(pos) {
            return;
        }
        // A click on a PARKED shape re-activates it (its handle is then grabbed by the following `*_down`).
        if let Some(i) = self.hit_parked_shape(pos, tol) {
            self.activate_parked_shape(i);
            return;
        }
        // Empty space with a complete editable active shape → park it; the following `*_down` starts fresh.
        if self.is_editing_shape() {
            self.park_active_shape();
        }
    }

    /// `true` when a Down at `pos` targets the ACTIVE editor (whichever slot is live) for editing — the
    /// per-type predicate; each returns `false` when its slot is empty, so exactly the live one answers.
    fn active_shape_hit(&self, pos: [f32; 2]) -> bool {
        self.curve_hit_active(pos)
            || self.ellipse_hit_active(pos)
            || self.polygon_hit_active(pos)
            || self.line_hit_active(pos)
    }

    /// Index of the topmost (last-drawn) parked shape that `pos` **alcança**, ou `None`.
    ///
    /// ⚠️ **Alcançar é encostar no que está DESENHADO**, não cair dentro de uma caixa. A versão anterior
    /// perguntava `bbox_contains` — o INTERIOR da AABB —, e com quatro círculos grandes sobrepostos a
    /// união das caixas cobre quase a tela: todo Down reativava uma figura parqueada e o gesto de
    /// *começar outra* ficava inalcançável (Enio, 2026-08-07: *"se clicar dentro de uma forma já
    /// desenhada, não aceita desenhar outra"*). Detalhe do mecanismo em [`super::stroke_outline`].
    ///
    /// São **duas** regiões, e as duas são coisas que o artista VÊ: o contorno e o quadrado central do
    /// badge (com o glifo de Operation dentro dele). Um alvo desenhado que não responde é a metade
    /// oposta do mesmo defeito.
    fn hit_parked_shape(&self, pos: [f32; 2], tol: f32) -> Option<usize> {
        let states: Vec<crate::undo::ShapeEditState> = self
            .paint
            .parked_shapes
            .iter()
            .map(|s| s.state.clone())
            .collect();
        states
            .iter()
            .enumerate()
            .rev()
            .find(|(_, st)| self.parked_shape_hit(st, pos, tol))
            .map(|(i, _)| i)
    }

    /// `true` quando `pos` encosta no contorno da figura ou no quadrado central dela. O raio do centro é
    /// `tol * 2.0`, o MESMO que o `maybe_switch_or_new_shape` passa ao `on_active_centre_square` — o
    /// quadrado da figura ativa e o da parqueada têm o mesmo tamanho na tela, então têm de ter o mesmo
    /// alcance.
    fn parked_shape_hit(&self, st: &crate::undo::ShapeEditState, pos: [f32; 2], tol: f32) -> bool {
        let Some(o) = self.shape_state_outline(st) else {
            return false;
        };
        if let Some(bb) = o.bbox() {
            let c = [(bb[0] + bb[2]) * 0.5, (bb[1] + bb[3]) * 0.5];
            let r = tol * 2.0;
            if dist2(pos, c) <= r * r {
                return true;
            }
        }
        o.hit(pos, tol)
    }
}

/// `true` when `pos` is within `band` px of the polyline `spine` (min distance to any segment).
pub(super) fn point_near_polyline(pos: [f32; 2], spine: &[[f32; 2]], band: f32) -> bool {
    min_dist2_to_polyline(pos, spine).is_some_and(|d2| d2 <= band * band)
}

/// The MIN squared distance from `pos` to the polyline `spine` (any segment), or `None` when empty. Used to
/// pick which of several curves a click is nearest to (multi-curve point insertion).
pub(super) fn min_dist2_to_polyline(pos: [f32; 2], spine: &[[f32; 2]]) -> Option<f32> {
    if spine.is_empty() {
        return None;
    }
    if spine.len() == 1 {
        return Some(dist2(pos, spine[0]));
    }
    spine
        .windows(2)
        .map(|w| dist2_point_segment(pos, w[0], w[1]))
        .fold(None, |acc, d| Some(acc.map_or(d, |a: f32| a.min(d))))
}

pub(super) fn dist2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    dx * dx + dy * dy
}

/// Squared distance from `p` to segment `a`–`b` (transcendental-free; HR-5).
fn dist2_point_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (abx, aby) = (b[0] - a[0], b[1] - a[1]);
    let len2 = abx * abx + aby * aby;
    if len2 <= f32::EPSILON {
        return dist2(p, a);
    }
    let t = (((p[0] - a[0]) * abx + (p[1] - a[1]) * aby) / len2).clamp(0.0, 1.0);
    let proj = [a[0] + t * abx, a[1] + t * aby];
    dist2(p, proj)
}
