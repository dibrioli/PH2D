//! Subsection painters for the Vector Style panel: the Smooth / Sharpen /
//! Simplify / Subdivide reshape buttons, the Fill-type + Fill-rule selectors and
//! the Make/Release Compound row. Split from `paint_sections` to keep that file
//! under the 600-LOC panel cap; it's an `impl BodyCtx` block over there.

use crate::paint_sections::BodyCtx;
use crate::state::{FillKind, PathFillRule};
use crate::{ids, state};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
use ph2d_editor_core::widget::{Button, ButtonKind, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Spacing, TypeToken};

/// Full turn in degrees (Angle slider track `0..1` maps to `0..FULL_TURN_DEG`).
const FULL_TURN_DEG: f64 = 360.0; // LITERAL-PX-OK: degrees in a full turn (math constant)

impl BodyCtx<'_> {
    /// A labelled 2-across segmented button row (sibling of `segmented3`).
    pub(crate) fn segmented2(
        &mut self,
        label: &str,
        opts: [(ph2d_a11y::NodeId, &str, bool); 2],
        mut y: f32,
    ) -> f32 {
        let font = TypeToken::Sm.px();
        let gap = Spacing::Sm.px();
        let w = ((self.inner_w - gap) / 2.0).max(1.0);
        paint_text(
            self.text_system,
            self.scene,
            label,
            self.inner_x,
            y,
            font,
            self.inner_w,
            resolve(ColorToken::Text2, self.theme),
        );
        y += font + Spacing::Xs.px();
        for (i, (id, lbl, active)) in opts.iter().enumerate() {
            let rx = self.inner_x + i as f32 * (w + gap);
            let rect = Rect::new(rx, y, w, self.row_h);
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

    /// Snap em FORMAS — ajuste da ferramenta (não da seleção), sempre visível.
    /// Segurar Alt ignora o snap momentaneamente, como no Figma.
    ///
    /// A grade **não mora aqui**: o editor tem um subsistema universal de grade
    /// (nove tipos, magnetismo, subdivisões) com painel próprio, e o Vector só
    /// pergunta a ele. Ligar/desligar a grade é lá.
    pub(crate) fn snap_section(&mut self, y: f32) -> f32 {
        let on = state::current_snap();
        let (y, collapsed) =
            self.section_header(ids::VECTOR_SECTION_SNAP, tr("panel.vector.section.snap"), y);
        if collapsed {
            return y;
        }
        self.segmented2(
            "Shapes",
            [
                (ids::VECTOR_SNAP_OFF, "Off", !on),
                (ids::VECTOR_SNAP_ON, "On", on),
            ],
            y,
        )
    }

    /// Make / Release Compound — a 2-across row closing the Boolean section. Merging
    /// closed paths into one compound is how a hole is authored by hand (draw a
    /// contour inside another, merge, and `EvenOdd` vacates the inner region).
    pub(crate) fn compound_row(&mut self, y: f32) -> f32 {
        let gap = Spacing::Sm.px();
        let w = ((self.inner_w - gap) / 2.0).max(1.0);
        self.row2(
            w,
            gap,
            [
                (ids::VECTOR_COMPOUND_MAKE, "Compound"),
                (ids::VECTOR_COMPOUND_RELEASE, "Release"),
            ],
            y,
        )
    }

    /// Fill rule of the selected path. Only shown for a COMPOUND path — with a
    /// single contour the two rules paint identically, so the row would be a no-op.
    fn fill_rule_row(&mut self, y: f32) -> f32 {
        let Some(rule) = state::current_fill_rule() else {
            return y;
        };
        self.segmented2(
            "Fill Rule",
            [
                (
                    ids::VECTOR_FILL_RULE_NONZERO,
                    "Non-Zero",
                    rule == PathFillRule::NonZero,
                ),
                (
                    ids::VECTOR_FILL_RULE_EVENODD,
                    "Even-Odd",
                    rule == PathFillRule::EvenOdd,
                ),
            ],
            y,
        )
    }

    /// Seção **FILL TYPE** — o seletor Solid / Linear / Radial / Multi + os controles de
    /// gradiente (e a regra de preenchimento do compound). Age sobre o path SELECIONADO;
    /// sem seleção (ou sem fill) a seção inteira some.
    pub(crate) fn fill_type_section(&mut self, y: f32) -> f32 {
        let Some(kind) = state::current_fill_kind() else {
            return y;
        };
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_FILL_TYPE,
            tr("panel.vector.section.fill_type"),
            y,
        );
        if collapsed {
            return y;
        }
        y = self.fill_rule_row(y);
        let kinds = [
            (ids::VECTOR_FILL_KIND_SOLID, "Solid", FillKind::Solid),
            (ids::VECTOR_FILL_KIND_LINEAR, "Linear", FillKind::Linear),
            (ids::VECTOR_FILL_KIND_RADIAL, "Radial", FillKind::Radial),
            (ids::VECTOR_FILL_KIND_MULTI, "Multi", FillKind::MultiPoint),
        ];
        let gap = Spacing::Sm.px();
        let cw = ((self.inner_w - gap * (kinds.len() as f32 - 1.0)) / kinds.len() as f32).max(1.0);
        for (i, (id, label, k)) in kinds.iter().enumerate() {
            let rx = self.inner_x + i as f32 * (cw + gap);
            let rect = Rect::new(rx, y, cw, self.row_h);
            let bstate = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
            paint_segmented_button(
                rect,
                label,
                kind == *k,
                bstate,
                self.scene,
                self.text_system,
                self.theme,
            );
            self.hit_index.register(*id, rect);
        }
        y += self.row_h + self.row_gap;

        // Multi-point: Add / Remove point (drag points on-canvas to move them) +
        // Influence slider for the selected point.
        if kind == FillKind::MultiPoint {
            let two_col = ((self.inner_w - gap) / 2.0).max(1.0);
            y = self.row2(
                two_col,
                gap,
                [
                    (ids::VECTOR_GRAD_ADD_POINT, "Add Point"),
                    (ids::VECTOR_GRAD_REMOVE_POINT, "Remove Point"),
                ],
                y,
            );
            if let Some(inf) = state::current_grad_influence() {
                const MAX_INFLUENCE: f64 = 4.0; // LITERAL-PX-OK: IDW strength range (domain)
                let track = self
                    .store
                    .slider(ids::VECTOR_GRAD_INFLUENCE)
                    .map(|(_, v)| v)
                    .unwrap_or((inf / MAX_INFLUENCE) as f32);
                let val = f64::from(track) * MAX_INFLUENCE;
                y = self.slider_row(
                    "Influence",
                    ids::VECTOR_GRAD_INFLUENCE,
                    ids::VECTOR_GRAD_INFLUENCE_NUM,
                    track,
                    val,
                    &format!("{val:.1}"),
                    y,
                );
            }
            // Jitter (per-texel grain, 0..1) for the selected point — track == value.
            if let Some(jit) = state::current_grad_jitter() {
                let track = self
                    .store
                    .slider(ids::VECTOR_GRAD_JITTER)
                    .map(|(_, v)| v)
                    .unwrap_or(jit as f32);
                let val = f64::from(track);
                y = self.slider_row(
                    "Jitter",
                    ids::VECTOR_GRAD_JITTER,
                    ids::VECTOR_GRAD_JITTER_NUM,
                    track,
                    val,
                    &format!("{val:.2}"),
                    y,
                );
            }
        }
        // Linear/Radial: Add / Remove ramp stop (drag interior stops on-canvas).
        if kind == FillKind::Linear || kind == FillKind::Radial {
            let two_col = ((self.inner_w - gap) / 2.0).max(1.0);
            y = self.row2(
                two_col,
                gap,
                [
                    (ids::VECTOR_GRAD_ADD_STOP, "Add Stop"),
                    (ids::VECTOR_GRAD_REMOVE_STOP, "Remove Stop"),
                ],
                y,
            );
        }
        // Angle slider (Linear only) — track 0..1 → 0..360°.
        if kind == FillKind::Linear {
            let angle = state::current_grad_angle().unwrap_or(0.0);
            let track = self
                .store
                .slider(ids::VECTOR_GRAD_ANGLE)
                .map(|(_, v)| v)
                .unwrap_or((angle / FULL_TURN_DEG) as f32);
            let deg = f64::from(track) * FULL_TURN_DEG;
            y = self.slider_row(
                "Angle",
                ids::VECTOR_GRAD_ANGLE,
                ids::VECTOR_GRAD_ANGLE_NUM,
                track,
                deg,
                &format!("{}", deg.round() as i64),
                y,
            );
        }
        y
    }
    /// "Align" + "Distribute" section — shown only with a multi-path OBJECT
    /// selection (≥2 for Align, ≥3 for Distribute). Two 3-col rows (X-align then
    /// Y-align) + a 2-col Distribute row. Buttons drive the shell drain.
    pub(crate) fn align_section(&mut self, y: f32) -> f32 {
        let count = state::current_selection_count();
        if count < 2 {
            return y;
        }
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_ALIGN,
            tr("panel.vector.section.align"),
            y,
        );
        if collapsed {
            return y;
        }
        let gap = Spacing::Sm.px();
        let cols = 3usize;
        let cw = ((self.inner_w - gap * (cols as f32 - 1.0)) / cols as f32).max(1.0);
        let rows = [
            [
                (ids::VECTOR_ALIGN_LEFT, "Left"),
                (ids::VECTOR_ALIGN_HCENTER, "Center"),
                (ids::VECTOR_ALIGN_RIGHT, "Right"),
            ],
            [
                (ids::VECTOR_ALIGN_TOP, "Top"),
                (ids::VECTOR_ALIGN_VCENTER, "Middle"),
                (ids::VECTOR_ALIGN_BOTTOM, "Bottom"),
            ],
        ];
        for row in rows {
            for (i, (id, label)) in row.iter().enumerate() {
                let rx = self.inner_x + i as f32 * (cw + gap);
                let rect = Rect::new(rx, y, cw, self.row_h);
                let bstate = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
                let btn = Button::new(*id, *label)
                    .kind(ButtonKind::Default)
                    .state(bstate);
                paint_button(&btn, rect, self.scene, self.text_system, self.theme);
                self.hit_index.register(*id, rect);
            }
            y += self.row_h + self.row_gap;
        }
        // Distribute needs ≥3 paths (two are always the fixed extremes).
        if count >= 3 {
            let two_col = ((self.inner_w - gap) / 2.0).max(1.0);
            y = self.row2(
                two_col,
                gap,
                [
                    (ids::VECTOR_DISTRIBUTE_H, "Dist H"),
                    (ids::VECTOR_DISTRIBUTE_V, "Dist V"),
                ],
                y,
            );
        }
        y + self.row_gap
    }

    /// Seção **PATH** — remodela o path selecionado inteiro. Smooth / Sharpen numa linha
    /// de 2 colunas; Simplify (menos pontos) / Subdivide (mais pontos) na segunda; o
    /// toggle Close/Open (rótulo vindo do `closed` publicado); e, quando a seleção tem
    /// uma forma VIVA (paramétrica / texto), o **Convert to Curves** — que é justamente
    /// o que PRODUZ um path cru editável, e por isso mora aqui.
    pub(crate) fn path_section(&mut self, y: f32) -> f32 {
        let (mut y, collapsed) =
            self.section_header(ids::VECTOR_SECTION_PATH, tr("panel.vector.section.path"), y);
        if collapsed {
            return y;
        }
        let gap = Spacing::Sm.px();
        let w = ((self.inner_w - gap) / 2.0).max(1.0);
        if state::convertible() {
            y = self.action_button(ids::VECTOR_CONVERT_TO_CURVES, "Convert to Curves", y);
        }
        y = self.row2(
            w,
            gap,
            [
                (ids::VECTOR_PATH_SMOOTH, "Smooth"),
                (ids::VECTOR_PATH_SHARPEN, "Sharpen"),
            ],
            y,
        );
        y = self.row2(
            w,
            gap,
            [
                (ids::VECTOR_PATH_SIMPLIFY, "Simplify"),
                (ids::VECTOR_PATH_SUBDIVIDE, "Subdivide"),
            ],
            y,
        );
        // Close/Open toggle — label reflects the current state (default "Close"
        // when no path is selected / not yet closed).
        let label = if state::current_path_closed() == Some(true) {
            "Open Path"
        } else {
            "Close Path"
        };
        self.action_button(ids::VECTOR_PATH_CLOSE, label, y)
    }
}
