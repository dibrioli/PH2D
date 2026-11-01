//! ⭐ **O «Stroke settings»** (F44, dono 01/11) — irmão do [`super::fill_stroke`] pelo tecto de 600
//! LOC do painel, e o corte é por responsabilidade: ali as LINHAS do Inspector, aqui o corpo que o ⚙
//! abre à esquerda e que o painel do traço da barra ② pinta por baixo das mesmas linhas.

use super::*;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::widget::Unit;
use ph2d_editor_core::widget::panel_chrome::paint_segmented_icon_group;
use ph2d_tokens::TypeToken;
use ph2d_tool_vector::StyleFields;

impl BodyCtx<'_> {
    /// ⭐ **O «Stroke settings»** — o painel à esquerda (pelo ⚙) e o da barra ② pintam este corpo:
    /// a tinta, Cap e Join em glifos, os tokens, o tracejado, as pontas e o perfil de largura.
    pub(crate) fn stroke_settings_body(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let mut y = y;
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

    /// ⭐ **A linha FIXA** do traço de base: Position (Inside · Center · Outside, em glifos) na
    /// coluna da caixa e Weight na do `%` — as mesmas colunas das linhas de cor, como no Figma.
    pub(super) fn stroke_position_row(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let font = TypeToken::Sm.px();
        let ink = resolve(ColorToken::Text2, self.theme);
        let c = self.colunas(y);
        for (key, x, w) in [
            ("panel.vector.stroke.align", c.caixa.x, c.caixa.w),
            ("panel.vector.stroke.weight", c.pct.x, c.pct.w),
        ] {
            paint_text(self.text_system, self.scene, tr(key), x, y, font, w, ink);
        }
        let y = y + font + Spacing::Xs.px();
        let c = self.colunas(y);
        // Dois canais, dois losangos, cada um colado ao controlo dele.
        self.decorator(
            c.caixa_key,
            &[
                ph2d_tool_vector::ids::VECTOR_ALIGN_INNER,
                ph2d_tool_vector::ids::VECTOR_ALIGN_CENTRE,
                ph2d_tool_vector::ids::VECTOR_ALIGN_OUTER,
            ],
        );
        self.decorator(
            c.pct_key,
            &[
                ph2d_tool_vector::ids::VECTOR_WIDTH,
                ph2d_tool_vector::ids::VECTOR_WIDTH_NUM,
            ],
        );
        let agrees = snap.agrees(StyleFields::ALIGN);
        paint_segmented_icon_group(
            c.caixa,
            &[
                (
                    IconId::StrokeInside,
                    agrees && snap.align == StrokeAlign::Inner,
                    ph2d_tool_vector::ids::VECTOR_ALIGN_INNER,
                ),
                (
                    IconId::StrokeCenter,
                    agrees && snap.align == StrokeAlign::Centre,
                    ph2d_tool_vector::ids::VECTOR_ALIGN_CENTRE,
                ),
                (
                    IconId::StrokeOutside,
                    agrees && snap.align == StrokeAlign::Outer,
                    ph2d_tool_vector::ids::VECTOR_ALIGN_OUTER,
                ),
            ],
            self.scene,
            self.text_system,
            self.theme,
            self.store,
            self.hit_index,
        );
        let wid = ph2d_tool_vector::ids::VECTOR_WIDTH_NUM;
        let px = self.store.number_value(wid).unwrap_or(snap.stroke_width_px);
        self.loose_number(wid, px, !snap.agrees(StyleFields::WIDTH), Unit::Px, c.pct);
        // O TOKEN da espessura risca o número: ele mostra a largura autorada, não a que se desenha.
        if state::token_bindings().is_some_and(|b| b.width.is_some()) {
            self.token_slash(c.pct);
        }
        y + self.row_h + self.row_gap
    }
}
