//! ⭐ **O «Stroke settings»** (F44, dono 01/11) — irmão do [`super::fill_stroke`] pelo tecto de 600
//! LOC do painel, e o corte é por responsabilidade: ali as LINHAS do Inspector, aqui o corpo que o ⚙
//! abre à esquerda e que o painel do traço da barra ② pinta por baixo das mesmas linhas.

use super::*;
use ph2d_editor_core::icons::IconId;
use ph2d_tool_vector::StyleFields;

impl BodyCtx<'_> {
    /// ⭐ **O «Stroke settings»** — o painel à esquerda (pelo ⚙) e o da barra ② pintam este corpo:
    /// a tinta, Cap e Join em glifos, os tokens, o tracejado, as pontas e o perfil de largura.
    pub(crate) fn stroke_settings_body(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let mut y = y;
        // F44c: a Position do traço de base mora aqui (a de uma camada, o painel dela pinta-a).
        if state::open_layer().is_none() {
            y = self.base_position_row(snap, y);
        }
        if let Some(k) = state::stroke_paint_kind() {
            y = self.segmented(
                tr("panel.vector.stroke.type"),
                &[
                    (
                        ph2d_tool_vector::ids::VECTOR_STROKE_KIND_SOLID,
                        tr("panel.vector.stroke.solid"),
                        k == state::StrokePaintKind::Solid,
                    ),
                    (
                        ph2d_tool_vector::ids::VECTOR_STROKE_KIND_PATTERN,
                        tr("panel.vector.stroke.pattern"),
                        k == state::StrokePaintKind::Pattern,
                    ),
                    (
                        ph2d_tool_vector::ids::VECTOR_STROKE_KIND_BRUSH,
                        tr("panel.vector.stroke.brush"),
                        k == state::StrokePaintKind::Brush,
                    ),
                ],
                y,
            );
        }
        let cap = snap.agrees(StyleFields::CAP);
        y = self.with_rows(|rc| {
            rc.icon_segmented(
                tr("panel.vector.stroke.cap"),
                &[
                    (
                        ph2d_tool_vector::ids::VECTOR_CAP_BUTT,
                        IconId::CapButt,
                        cap && snap.cap == StrokeCap::Butt,
                    ),
                    (
                        ph2d_tool_vector::ids::VECTOR_CAP_ROUND,
                        IconId::CapRound,
                        cap && snap.cap == StrokeCap::Round,
                    ),
                    (
                        ph2d_tool_vector::ids::VECTOR_CAP_SQUARE,
                        IconId::CapSquare,
                        cap && snap.cap == StrokeCap::Square,
                    ),
                ],
                y,
            )
        });
        let join = snap.agrees(StyleFields::JOIN);
        y = self.with_rows(|rc| {
            rc.icon_segmented(
                tr("panel.vector.stroke.join"),
                &[
                    (
                        ph2d_tool_vector::ids::VECTOR_JOIN_MITER,
                        IconId::JoinMiter,
                        join && snap.join == StrokeJoin::Miter,
                    ),
                    (
                        ph2d_tool_vector::ids::VECTOR_JOIN_ROUND,
                        IconId::JoinRound,
                        join && snap.join == StrokeJoin::Round,
                    ),
                    (
                        ph2d_tool_vector::ids::VECTOR_JOIN_BEVEL,
                        IconId::JoinBevel,
                        join && snap.join == StrokeJoin::Bevel,
                    ),
                ],
                y,
            )
        });
        // ⭐ F47 (dono 02/11): o traço por cima dos Filtros da forma.
        if let Some(on) = state::stroke_above_effects() {
            let id = ids::VECTOR_STROKE_ABOVE_EFFECTS;
            y = self.checkbox_row(id, tr("panel.vector.stroke.above_effects"), on, y);
        }
        // Os tokens da espessura e da cor — só com traço: o token colore o traço que existe e
        // nunca inventa largura.
        if let Some(b) = state::token_bindings()
            .filter(|_| state::stroke_present() == Some(true) && state::open_layer().is_none())
        {
            y = self.token_row_named(
                "panel.vector.token.weight",
                ids::VECTOR_TOKEN_WIDTH,
                b.width.as_deref(),
                y,
            );
            y = self.token_row_named(
                "panel.vector.token.color",
                ids::VECTOR_TOKEN_STROKE,
                b.stroke.as_deref(),
                y,
            );
        }
        y = self.dash_rows(snap, y);
        y = self.marker_rows(snap, y);
        if state::current_selection_count() > 0 {
            y = self.width_preset_row(state::current_selection_profile(), y);
        }
        y
    }

    /// ⭐ **A Position do traço de base** (Inside · Center · Outside) — a fileira de glifos de duas
    /// tintas, como a de uma camada.
    fn base_position_row(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let agrees = snap.agrees(StyleFields::ALIGN);
        let a = |k: StrokeAlign| agrees && snap.align == k;
        self.with_rows(|rc| {
            rc.icon_segmented(
                tr("panel.vector.stroke.align"),
                &[
                    (
                        ph2d_tool_vector::ids::VECTOR_ALIGN_INNER,
                        IconId::StrokeInside,
                        a(StrokeAlign::Inner),
                    ),
                    (
                        ph2d_tool_vector::ids::VECTOR_ALIGN_CENTRE,
                        IconId::StrokeCenter,
                        a(StrokeAlign::Centre),
                    ),
                    (
                        ph2d_tool_vector::ids::VECTOR_ALIGN_OUTER,
                        IconId::StrokeOutside,
                        a(StrokeAlign::Outer),
                    ),
                ],
                y,
            )
        })
    }
}
