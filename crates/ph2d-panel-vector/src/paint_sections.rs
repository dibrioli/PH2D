//! Body-section painters for the Vector Style panel, extracted from
//! [`crate::paint`] so the orchestrator fn + the file stay under the panel
//! LOC caps (600/file, 200/fn — `architecture_panel_loc_cap`).
//!
//! [`BodyCtx`] bundles the per-frame mutables (Vello scene, text shaper, widget
//! store, hit index) + the shared layout metrics; each `paint_*` method takes
//! the running `y` and returns the advanced `y`. Pure relocation of the drawing
//! calls — no behavioral change.

use crate::ids;
use crate::state;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::panel_chrome::{SECTION_LABEL_TO_CONTROL_PX, paint_segmented_button};
use ph2d_editor_core::widget::showcase::paint_section_separator;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, ColorSwatch, SectionHeader, SwatchSize, paint_button,
    paint_color_swatch, paint_section_header, paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_tool_vector::params::{
    BLEND_STEPS_DEFAULT, MORPH_T_DEFAULT, blend_steps_from_track, blend_steps_to_track,
    dash_to_slider, gap_to_slider, opacity_to_slider,
};
use ph2d_tool_vector::{StrokeCap, StrokeJoin, VectorStyleSnapshot, VertexType, px_to_slider};
use ph2d_vector::VectorScene;

/// Label column width for the Width slider row + the Stroke / Fill labels.
pub(crate) const LABEL_COL_W: f32 = 64.0; // LITERAL-PX-OK: panel grid metric (per-panel label gutter width)

/// Per-frame paint context for the Vector Style panel body — the mutable render
/// targets + the shared layout metrics. Constructed once per frame in
/// [`crate::paint`]; each section method borrows disjoint fields.
pub(crate) struct BodyCtx<'a> {
    pub scene: &'a mut VectorScene,
    pub text_system: &'a mut TextSystem,
    pub store: &'a WidgetStore,
    pub hit_index: &'a mut HitIndex,
    pub theme: Theme,
    pub inner_x: f32,
    pub inner_w: f32,
    pub row_h: f32,
    pub row_gap: f32,
    pub chip_w: f32,
    pub font: f32,
}

/// A seção **Blend** — módulo irmão (teto de 600 LOC).
#[path = "paint_blend.rs"]
mod blend;

/// A seção **Envelope** — módulo irmão (teto de 600 LOC).
#[path = "paint_envelope.rs"]
mod envelope;

/// A seção **Effects** (ADR-0132) — módulo irmão (teto de 600 LOC).
#[path = "paint_effects.rs"]
mod effects;

/// A seção **Vertex** (tipo do vértice + Delete) — módulo irmão (teto de 600 LOC).
#[path = "paint_vertex.rs"]
mod vertex;

impl BodyCtx<'_> {
    /// A full-width slider + linked value chip row; returns the advanced `y`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn slider_row(
        &mut self,
        label: &str,
        slider_id: ph2d_a11y::NodeId,
        chip_id: ph2d_a11y::NodeId,
        track: f32,
        val: f64,
        display: &str,
        y: f32,
    ) -> f32 {
        let used = paint_slider_with_chip_layout_adaptive(
            Rect::new(self.inner_x, y, self.inner_w, self.row_h),
            label,
            track,
            val,
            Some(display),
            slider_id,
            chip_id,
            LABEL_COL_W,
            self.chip_w,
            self.store,
            self.hit_index,
            self.scene,
            self.text_system,
            self.theme,
        );
        y + used + self.row_gap
    }

    /// A labelled 3-across segmented button row (Cap / Join / text Align).
    pub(crate) fn segmented3(
        &mut self,
        label: &str,
        opts: [(ph2d_a11y::NodeId, &str, bool); 3],
        mut y: f32,
    ) -> f32 {
        let sd_font = TypeToken::Sm.px();
        let sd_gap = Spacing::Sm.px();
        let sd_w = ((self.inner_w - sd_gap * 2.0) / 3.0).max(1.0); // LITERAL-PX-OK: 3-column segmented grid (button count, not a metric)
        paint_text(
            self.text_system,
            self.scene,
            label,
            self.inner_x,
            y,
            sd_font,
            self.inner_w,
            resolve(ColorToken::Text2, self.theme),
        );
        y += sd_font + Spacing::Xs.px();
        for (i, (id, lbl, active)) in opts.iter().enumerate() {
            let rx = self.inner_x + i as f32 * (sd_w + sd_gap);
            let rect = Rect::new(rx, y, sd_w, self.row_h);
            let st = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
            paint_segmented_button(
                rect,
                lbl,
                *active,
                st,
                self.scene,
                self.text_system,
                self.theme,
            );
            self.hit_index.register(*id, rect);
        }
        y + self.row_h + self.row_gap
    }

    /// O cabeçalho CANÔNICO de seção (`docs/UI_Padrao/components/section_header.md`:
    /// "TODA seção usa `paint_section_header`" · "TODA seção é colapsável"). Chevron +
    /// rótulo em MAIÚSCULAS; o clique DOBRA a seção. Retorna `(y, collapsed)` — o caller
    /// sai cedo quando dobrada.
    ///
    /// O collapse é dispatch GENÉRICO e exige **dois** sites: o `populate` marca o id
    /// (`mark_collapsible_section`) e aqui registramos o hit-rect. Sem os dois o header
    /// vira um título morto que pinta um chevron e não dobra.
    ///
    /// Substituiu os 14 `section_label` caseiros do painel (`paint_text` em Sm/Text2):
    /// eram tipograficamente idênticos aos botões abaixo deles, e é por isso que
    /// categoria e tipo de forma se confundiam.
    pub(crate) fn section_header(
        &mut self,
        id: ph2d_a11y::NodeId,
        label: &str,
        y: f32,
    ) -> (f32, bool) {
        let header_h = TypeToken::Md.px() + Spacing::Md.px();
        let collapsed = self.store.is_collapsed(id);
        let header = SectionHeader::new(id, label).collapsible(!collapsed);
        let rect = Rect::new(self.inner_x, y, self.inner_w, header_h);
        paint_section_header(&header, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
        (y + header_h + SECTION_LABEL_TO_CONTROL_PX, collapsed)
    }

    /// **A ordem do corpo.** "O que estou fazendo" (modo · forma · parâmetros da forma)
    /// vem ANTES de "com que cor" (stroke · fill) — o corpo abria em Stroke/Fill e o
    /// seletor de ferramenta aparecia depois, invertido.
    ///
    /// Cada `step` pinta a seção e, só se ela pintou, fecha com o separador canônico: as
    /// condicionais (Text, Axes, Fill Type, Transform, Vertex, Align) somem sem deixar uma
    /// linha órfã. A última (Path) fecha sem separador de rodapé.
    pub(crate) fn paint_body(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let mut y = y;
        y = self.step(y, |b, y| b.tool_section(snap, y));
        y = self.step(y, |b, y| b.shape_catalog_section(snap, y));
        y = self.step(y, |b, y| b.shape_params_section(snap, y));
        // Os parâmetros do CONECTOR selecionado — irmã da seção acima (mesma estética,
        // mesmos widgets), logo abaixo dela: as duas respondem "o que é este objeto que
        // eu selecionei, e como o afino?". Some inteira sem conector na seleção.
        y = self.step(y, Self::connector_section);
        y = self.step(y, Self::text_section);
        y = self.step(y, Self::font_section);
        y = self.step(y, Self::paragraph_section);
        y = self.step(y, Self::axes_section);
        y = self.step(y, |b, y| b.stroke_style(snap, y));
        y = self.step(y, |b, y| b.fill_style(snap, y));
        y = self.step(y, Self::fill_type_section);
        y = self.step(y, Self::snap_section);
        y = self.step(y, Self::transform_section);
        y = self.step(y, Self::vertex_section);
        y = self.step(y, Self::boolean_section);
        y = self.step(y, |b, y| b.blend_section(snap, y));
        y = self.step(y, |b, y| b.morph_section(y));
        // O Envelope fica junto dos outros deformadores não-destrutivos (Blend/Morph): os três
        // produzem geometria DERIVADA de uma relação viva, e o artista os procura no mesmo lugar.
        y = self.step(y, Self::envelope_section);
        // Effects fica logo depois dos deformadores: os três são não-destrutivos, e a
        // pilha é a generalização deles (ADR-0132).
        y = self.step(y, Self::effects_section);
        y = self.step(y, Self::align_section);
        y = self.step(y, Self::arrange_section);
        self.path_section(y)
    }

    /// A linha canônica ENTRE seções (nunca dentro de uma).
    pub(crate) fn separator(&mut self, y: f32) -> f32 {
        paint_section_separator(self.scene, self.theme, self.inner_x, self.inner_w, y)
    }

    /// Um passo do corpo: pinta a seção e, **se ela pintou** (o `y` andou), fecha com o
    /// separador. As seções condicionais (Vertex, Align, Text, Axes, Fill Type) somem
    /// sem deixar uma linha órfã boiando no painel — que é o motivo de o separador ser
    /// decidido AQUI, no orquestrador, e não dentro de cada seção.
    pub(crate) fn step(&mut self, y: f32, f: impl FnOnce(&mut Self, f32) -> f32) -> f32 {
        let after = f(self, y);
        if after > y {
            self.separator(after)
        } else {
            after
        }
    }

    /// A full-width action button (Boolean / Vertex-delete / Duplicate).
    pub(crate) fn action_button(&mut self, id: ph2d_a11y::NodeId, label: &str, y: f32) -> f32 {
        self.action_button_kind(id, label, ButtonKind::Default, y)
    }

    /// The same, but with a chosen `kind` — an **Accent** button is how a *commit* action (o
    /// "Apply" da pilha de efeitos) se destaca das edições comuns à sua volta.
    pub(crate) fn action_button_kind(
        &mut self,
        id: ph2d_a11y::NodeId,
        label: &str,
        kind: ButtonKind,
        y: f32,
    ) -> f32 {
        let rect = Rect::new(self.inner_x, y, self.inner_w, self.row_h);
        let st = self.store.button_state(id).unwrap_or(ButtonState::Normal);
        let btn = Button::new(id, label).kind(kind).state(st);
        paint_button(&btn, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
        y + self.row_h + Spacing::Xs.px()
    }

    /// Width + Stroke swatch + Stroke opacity + Cap / Join + Dash / Gap.
    pub(crate) fn stroke_style(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_STROKE,
            tr("panel.vector.section.stroke"),
            y,
        );
        if collapsed {
            return y;
        }
        // Width slider + px chip.
        let track = self
            .store
            .slider(ids::VECTOR_WIDTH)
            .map(|(_, v)| v)
            .unwrap_or_else(|| px_to_slider(snap.stroke_width_px));
        let px = self
            .store
            .number_value(ids::VECTOR_WIDTH_NUM)
            .unwrap_or(snap.stroke_width_px);
        y = self.slider_row(
            "Width",
            ids::VECTOR_WIDTH,
            ids::VECTOR_WIDTH_NUM,
            track,
            px,
            &format!("{}", px.round() as i64),
            y,
        );

        let swatch_w = SwatchSize::Md.px();
        // Stroke colour swatch.
        paint_text(
            self.text_system,
            self.scene,
            tr("panel.vector.section.stroke"),
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            LABEL_COL_W,
            resolve(ColorToken::Text1, self.theme),
        );
        let stroke_swatch_rect = Rect::new(
            self.inner_x + self.inner_w - swatch_w,
            y,
            swatch_w,
            self.row_h,
        );
        let stroke_swatch =
            ColorSwatch::new(ids::VECTOR_STROKE_SWATCH, "Stroke color", snap.stroke)
                .size(SwatchSize::Md);
        paint_color_swatch(&stroke_swatch, stroke_swatch_rect, self.scene, self.theme);
        self.hit_index
            .register(ids::VECTOR_STROKE_SWATCH, stroke_swatch_rect);
        y += self.row_h + self.row_gap;

        // Stroke Opacity slider (single source of the stroke alpha).
        let track = self
            .store
            .slider(ids::VECTOR_STROKE_OPACITY)
            .map(|(_, v)| v)
            .unwrap_or_else(|| opacity_to_slider(snap.stroke[3]));
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent for the opacity chip
        y = self.slider_row(
            "Opacity",
            ids::VECTOR_STROKE_OPACITY,
            ids::VECTOR_STROKE_OPACITY_NUM,
            track,
            pct,
            &format!("{}", pct.round() as i64),
            y,
        );

        // Cap / Join segmented rows.
        y = self.segmented3(
            "Cap",
            [
                (ids::VECTOR_CAP_BUTT, "Butt", snap.cap == StrokeCap::Butt),
                (ids::VECTOR_CAP_ROUND, "Round", snap.cap == StrokeCap::Round),
                (
                    ids::VECTOR_CAP_SQUARE,
                    "Square",
                    snap.cap == StrokeCap::Square,
                ),
            ],
            y,
        );
        y = self.segmented3(
            "Join",
            [
                (
                    ids::VECTOR_JOIN_MITER,
                    "Miter",
                    snap.join == StrokeJoin::Miter,
                ),
                (
                    ids::VECTOR_JOIN_ROUND,
                    "Round",
                    snap.join == StrokeJoin::Round,
                ),
                (
                    ids::VECTOR_JOIN_BEVEL,
                    "Bevel",
                    snap.join == StrokeJoin::Bevel,
                ),
            ],
            y,
        );

        // Dash length (multiple of width; 0 = solid).
        let track = self
            .store
            .slider(ids::VECTOR_DASH)
            .map(|(_, v)| v)
            .unwrap_or_else(|| dash_to_slider(snap.dash));
        let px = self
            .store
            .number_value(ids::VECTOR_DASH_NUM)
            .unwrap_or(snap.dash);
        y = self.slider_row(
            "Dash",
            ids::VECTOR_DASH,
            ids::VECTOR_DASH_NUM,
            track,
            px,
            &format!("{}", px.round() as i64),
            y,
        );

        // Gap length between dashes.
        let track = self
            .store
            .slider(ids::VECTOR_GAP)
            .map(|(_, v)| v)
            .unwrap_or_else(|| gap_to_slider(snap.gap));
        let px = self
            .store
            .number_value(ids::VECTOR_GAP_NUM)
            .unwrap_or(snap.gap);
        y = self.slider_row(
            "Gap",
            ids::VECTOR_GAP,
            ids::VECTOR_GAP_NUM,
            track,
            px,
            &format!("{}", px.round() as i64),
            y,
        );

        // As PONTAS do traço (Start / End). Ficam aqui, e não no catálogo de formas,
        // porque uma ponta é propriedade do stroke: herda a cor e a largura dele e gira
        // com a tangente da curva — a quarta decisão sobre como a caneta termina, ao lado
        // de cap, join e dash. Desenho em `paint_markers` (teto de LOC por arquivo).
        self.marker_rows(snap, y)
    }

    /// Fill swatch + Fill opacity (0 % = none).
    pub(crate) fn fill_style(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let (mut y, collapsed) =
            self.section_header(ids::VECTOR_SECTION_FILL, tr("panel.vector.section.fill"), y);
        if collapsed {
            return y;
        }
        let swatch_w = SwatchSize::Md.px();
        paint_text(
            self.text_system,
            self.scene,
            tr("panel.vector.section.fill"),
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            LABEL_COL_W,
            resolve(ColorToken::Text1, self.theme),
        );
        let fill_swatch_rect = Rect::new(
            self.inner_x + self.inner_w - swatch_w,
            y,
            swatch_w,
            self.row_h,
        );
        let fill_swatch =
            ColorSwatch::new(ids::VECTOR_FILL_SWATCH, "Fill color", snap.fill).size(SwatchSize::Md);
        paint_color_swatch(&fill_swatch, fill_swatch_rect, self.scene, self.theme);
        self.hit_index
            .register(ids::VECTOR_FILL_SWATCH, fill_swatch_rect);
        y += self.row_h + self.row_gap;

        let track = self
            .store
            .slider(ids::VECTOR_FILL_OPACITY)
            .map(|(_, v)| v)
            .unwrap_or_else(|| opacity_to_slider(snap.fill[3]));
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent for the opacity chip
        self.slider_row(
            "Opacity",
            ids::VECTOR_FILL_OPACITY,
            ids::VECTOR_FILL_OPACITY_NUM,
            track,
            pct,
            &format!("{}", pct.round() as i64),
            y,
        )
    }

    /// Seção **BOOLEAN** — ops N-árias sobre as regiões fechadas SELECIONADAS (a de trás
    /// é a base, a da frente doa o estilo) + a linha Compound / Release.
    pub(crate) fn boolean_section(&mut self, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_BOOLEAN,
            tr("panel.vector.section.boolean"),
            y,
        );
        if collapsed {
            return y;
        }
        for (id, label) in [
            (ids::VECTOR_BOOL_UNION, "Union"),
            (ids::VECTOR_BOOL_SUBTRACT, "Subtract"),
            (ids::VECTOR_BOOL_INTERSECT, "Intersect"),
            (ids::VECTOR_BOOL_EXCLUDE, "Exclude"),
        ] {
            y = self.action_button(id, label, y);
        }
        self.compound_row(y)
    }

    /// Seção **ARRANGE** — Duplicate + z-order (2×2) + Flip + Rotate. Age sobre o path
    /// selecionado (comandos de documento pelo dreno da shell).
    pub(crate) fn arrange_section(&mut self, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_ARRANGE,
            tr("panel.vector.section.arrange"),
            y,
        );
        if collapsed {
            return y;
        }
        y = self.action_button(ids::VECTOR_ARRANGE_DUPLICATE, "Duplicate", y);
        // Z-order: 2×2 grid — To Back | To Front · Backward | Forward.
        let zorder = [
            (ids::VECTOR_ARRANGE_TO_BACK, "To Back"),
            (ids::VECTOR_ARRANGE_TO_FRONT, "To Front"),
            (ids::VECTOR_ARRANGE_BACKWARD, "Backward"),
            (ids::VECTOR_ARRANGE_FORWARD, "Forward"),
        ];
        let z_cols = 2usize;
        let z_gap = Spacing::Sm.px();
        let z_w = ((self.inner_w - z_gap * (z_cols as f32 - 1.0)) / z_cols as f32).max(1.0);
        let z_top = y;
        for (i, (id, label)) in zorder.iter().enumerate() {
            let rx = self.inner_x + (i % z_cols) as f32 * (z_w + z_gap);
            let ry = z_top + (i / z_cols) as f32 * (self.row_h + z_gap);
            let rect = Rect::new(rx, ry, z_w, self.row_h);
            let bstate = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
            let btn = Button::new(*id, *label)
                .kind(ButtonKind::Default)
                .state(bstate);
            paint_button(&btn, rect, self.scene, self.text_system, self.theme);
            self.hit_index.register(*id, rect);
        }
        let z_rows = zorder.len().div_ceil(z_cols) as f32;
        y = z_top + z_rows * self.row_h + (z_rows - 1.0) * z_gap + self.row_gap;

        // Flip (mirror) + Rotate (90°) — each a 2-col row of action buttons.
        y = self.row2(
            z_w,
            z_gap,
            [
                (ids::VECTOR_ARRANGE_FLIP_H, "Flip H"),
                (ids::VECTOR_ARRANGE_FLIP_V, "Flip V"),
            ],
            y,
        );
        self.row2(
            z_w,
            z_gap,
            [
                (ids::VECTOR_ARRANGE_ROTATE_CW, "Rotate CW"),
                (ids::VECTOR_ARRANGE_ROTATE_CCW, "Rotate CCW"),
            ],
            y,
        )
    }

    /// A 2-column row of two half-width action buttons; returns the advanced `y`.
    pub(crate) fn row2(
        &mut self,
        w: f32,
        gap: f32,
        items: [(ph2d_a11y::NodeId, &str); 2],
        y: f32,
    ) -> f32 {
        for (i, (id, label)) in items.iter().enumerate() {
            let rx = self.inner_x + i as f32 * (w + gap);
            let rect = Rect::new(rx, y, w, self.row_h);
            let bstate = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
            let btn = Button::new(*id, *label)
                .kind(ButtonKind::Default)
                .state(bstate);
            paint_button(&btn, rect, self.scene, self.text_system, self.theme);
            self.hit_index.register(*id, rect);
        }
        y + self.row_h + self.row_gap
    }
}
