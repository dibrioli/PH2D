//! Subsection painters for the Vector Style panel: **a seção ARRANGE** (o Z-index, os quatro
//! botões de z-order, Flip e Rotate), os botões de reshape (Smooth / Sharpen / Simplify /
//! Subdivide), os seletores de Fill-type + Fill-rule e a linha Make/Release Compound. Split from
//! `paint_sections` to keep that file under the 600-LOC panel cap; it's an `impl BodyCtx` block
//! over there.
//!
//! ⚠️ A `arrange_section` mudou-se para cá quando o READOUT do Z-index (Enio, 2026-08-04) levou o
//! `paint_sections` a 602 — e o corte é por ASSUNTO, não por tamanho: este arquivo já se chamava
//! `paint_arrange`, e a seção que lhe dá o nome vivia noutro sítio.

use crate::paint_sections::BodyCtx;
use crate::state::{FillKind, PathFillRule};
use crate::{ids, state};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::panel_chrome::{
    paint_segmented_button, paint_segmented_group_adaptive,
};
use ph2d_editor_core::widget::{Button, ButtonKind, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Spacing, TypeToken};

/// Full turn in degrees (Angle slider track `0..1` maps to `0..FULL_TURN_DEG`).
const FULL_TURN_DEG: f64 = 360.0; // LITERAL-PX-OK: degrees in a full turn (math constant)

impl BodyCtx<'_> {
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
        // **O Z-INDEX GLOBAL** (Enio, 2026-08-04: *"o Z index deve ser global e sobrepõe a ordem
        // na hierarquia"*) — maior = mais à frente, a convenção do `CanvasItem.z_index` do Godot.
        // A ordem da Hierarquia só decide entre objetos que EMPATAM neste número.
        //
        // ⚠️ Ele vem ANTES dos quatro botões de propósito: é o número que manda, e eles movem a
        // forma *dentro* do que ele permite. Só é oferecido com uma forma selecionada — um campo
        // sem resposta única seria um controlo que escreve num objeto que o artista não nomeou.
        if state::z_index().is_some() {
            y = self.lone_number_row(tr("panel.vector.arrange.z"), ids::VECTOR_ARRANGE_Z, y);
        }
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
            let bstate = self.store.button_visual(*id);
            let btn = Button::new(*id, *label)
                .kind(ButtonKind::Default)
                .visual(bstate);
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

    /// A labelled segmented button row (sibling of `segmented3`).
    ///
    /// ⚠️ **Ela REFLUI**: uma fileira que não cabe quebra em linhas, em vez de espremer os
    /// rótulos uns contra os outros. A versão anterior dividia a largura por `n` e pronto, o que
    /// bastou enquanto toda fileira deste painel tinha duas a quatro opções — e o report do Enio
    /// (2026-08-02) chegou com a de CINCO: *"botões não se adaptam à largura do painel e se
    /// amontoam (veja Between e Around)"*.
    ///
    /// ⚠️ **E ela não implementa o refluxo: DELEGA.** O `paint_segmented_group_adaptive` é a
    /// resposta canónica do design system a *"como um grupo segmentado quebra"* — a mesma que o
    /// painel do Painter usa desde a lista de dez ferramentas do impasto —, e dentro de cada
    /// linha ele reparte a largura por igual, exactamente como aqui se fazia. Logo **toda fileira
    /// que já cabia é pintada igualzinha**; só a que não cabia muda. Uma segunda regra de quebra
    /// vivendo neste painel é como uma seção passa a ser medida por uma e preenchida por outra.
    pub(crate) fn segmented(
        &mut self,
        label: &str,
        opts: &[(ph2d_a11y::NodeId, &str, bool)],
        mut y: f32,
    ) -> f32 {
        let font = TypeToken::Sm.px();
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
        let segs: Vec<(&str, bool, ph2d_a11y::NodeId)> =
            opts.iter().map(|(id, lbl, on)| (*lbl, *on, *id)).collect();
        let used = paint_segmented_group_adaptive(
            Rect::new(self.inner_x, y, self.inner_w, self.row_h),
            &segs,
            self.scene,
            self.text_system,
            self.theme,
            self.store,
            self.hit_index,
        );
        y + used + self.row_gap
    }

    /// Snap em FORMAS — ajuste da ferramenta (não da seleção), sempre visível.
    /// Segurar Alt ignora o snap momentaneamente, como no Figma.
    ///
    /// A grade **não mora aqui**: o editor tem um subsistema universal de grade
    /// (nove tipos, magnetismo, subdivisões) com painel próprio, e o Vector só
    /// pergunta a ele. Ligar/desligar a grade é lá.
    /// ⚠️ As três linhas são **independentes**, e não modos de um mesmo rádio: "Shapes" ALINHA
    /// um eixo de cada vez (a lei do Figma), enquanto "Path" e "Cross" pousam o ponto num
    /// lugar. Colapsá-las num seletor obrigaria o artista a abrir mão do alinhamento para
    /// ganhar a linha, que são perguntas diferentes.
    pub(crate) fn snap_section(&mut self, y: f32) -> f32 {
        let on = state::current_snap();
        let path = state::current_snap_path();
        let cross = state::current_snap_crossings();
        let guides = state::current_snap_guides();
        let rulers = state::current_rulers();
        let (y, collapsed) =
            self.section_header(ids::VECTOR_SECTION_SNAP, tr("panel.vector.section.snap"), y);
        if collapsed {
            return y;
        }
        let y = self.segmented(
            "Shapes",
            &[
                (ids::VECTOR_SNAP_OFF, "Off", !on),
                (ids::VECTOR_SNAP_ON, "On", on),
            ],
            y,
        );
        let y = self.segmented(
            "Path",
            &[
                (ids::VECTOR_SNAP_PATH_OFF, "Off", !path),
                (ids::VECTOR_SNAP_PATH_ON, "On", path),
            ],
            y,
        );
        let y = self.segmented(
            "Cross",
            &[
                (ids::VECTOR_SNAP_CROSS_OFF, "Off", !cross),
                (ids::VECTOR_SNAP_CROSS_ON, "On", cross),
            ],
            y,
        );
        // As GUIAS (W6.2). Alinhamento como o "Shapes", com um interruptor próprio: desligar
        // o ímã das formas não deve desligar o das linhas que o artista pôs à mão.
        let y = self.segmented(
            "Guides",
            &[
                (ids::VECTOR_SNAP_GUIDES_OFF, "Off", !guides),
                (ids::VECTOR_SNAP_GUIDES_ON, "On", guides),
            ],
            y,
        );
        // ⚠️ A RÉGUA não é um ímã, e por isso fecha a seção em vez de se misturar às três
        // acima: ela decide se as faixas aparecem — e, com elas, se as guias podem ser
        // ARRASTADAS. É o *lock* que o Illustrator esconde num booleano de menu.
        self.segmented(
            "Rulers",
            &[
                (ids::VECTOR_RULERS_OFF, "Off", !rulers),
                (ids::VECTOR_RULERS_ON, "On", rulers),
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
        self.segmented(
            "Fill Rule",
            &[
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
            let bstate = self.store.button_visual(*id);
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
                let bstate = self.store.button_visual(*id);
                let btn = Button::new(*id, *label)
                    .kind(ButtonKind::Default)
                    .visual(bstate);
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
        y = self.action_button(ids::VECTOR_PATH_CLOSE, label, y);
        // **Reverse** (W4) — o sentido do caminho é um fato autorado, e até aqui não havia gesto
        // nenhum que o mudasse: o `reverse_path` existia com UM chamador interno e nenhum id.
        y = self.action_button(ids::VECTOR_PATH_REVERSE, "Reverse", y);
        // **Join** — ⚠️ só com 2+ selecionados. Com um caminho só a resposta é o `Close Path` logo
        // acima; oferecer o Join ali seria um segundo botão para a mesma pergunta, e ele
        // devolveria `false` em silêncio (a lei do botão morto).
        if state::current_selection_count() >= 2 {
            y = self.action_button(ids::VECTOR_PATH_JOIN, "Join", y);
        }
        y
    }
}
