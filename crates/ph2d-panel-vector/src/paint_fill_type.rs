//! Subsection painters for the Vector Style panel: os seletores de Fill-type + Fill-rule, a linha
//! Make/Release Compound e a fileira segmentada que eles partilham.
//!
//! ⚠️ Chamava-se `paint_arrange` até 15/10, quando o ARRANGE e o ALIGN saíram do painel para a
//! barra ④ (dono: *«tire a seção Arrange do painel»*; o Z é do Inspector no modo Objeto).

use crate::paint_sections::BodyCtx;
use crate::state;
use crate::state::{FillKind, PathFillRule};
use ph2d_editor_core::widget::panel_chrome::paint_segmented_group_adaptive;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::Spacing;

/// Full turn in degrees (Angle slider track `0..1` maps to `0..FULL_TURN_DEG`).
const FULL_TURN_DEG: f64 = 360.0; // LITERAL-PX-OK: degrees in a full turn (math constant)

impl BodyCtx<'_> {
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
        y: f32,
    ) -> f32 {
        self.with_rows(|r| r.segmented(label, opts, y))
    }

    /// Make / Release Compound — a 2-across row closing the Boolean section. Merging
    /// closed paths into one compound is how a hole is authored by hand (draw a
    /// contour inside another, merge, and `EvenOdd` vacates the inner region).
    pub(crate) fn compound_row(&mut self, y: f32) -> f32 {
        let gap = Spacing::Xs.px();
        let w = ((self.inner_w - gap) / 2.0).max(1.0);
        self.row2(
            w,
            gap,
            [
                (
                    ph2d_editor_core::ids::VECTOR_COMPOUND_MAKE,
                    tr("panel.vector.arrange.compound"),
                ),
                (
                    crate::ids::VECTOR_COMPOUND_RELEASE,
                    tr("panel.vector.arrange.release"),
                ),
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
            tr("panel.vector.arrange.fill_rule"),
            &[
                (
                    crate::ids::VECTOR_FILL_RULE_NONZERO,
                    tr("panel.vector.arrange.non_zero"),
                    rule == PathFillRule::NonZero,
                ),
                (
                    crate::ids::VECTOR_FILL_RULE_EVENODD,
                    tr("panel.vector.arrange.even_odd"),
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
            ph2d_tool_vector::ids::VECTOR_SECTION_FILL_TYPE,
            tr("panel.vector.section.fill_type"),
            y,
        );
        if collapsed {
            return y;
        }
        y = self.fill_rule_row(y);
        let kinds = [
            (
                ph2d_tool_vector::ids::VECTOR_FILL_KIND_SOLID,
                tr("panel.vector.arrange.solid"),
                FillKind::Solid,
            ),
            (
                ph2d_tool_vector::ids::VECTOR_FILL_KIND_LINEAR,
                tr("panel.vector.arrange.linear"),
                FillKind::Linear,
            ),
            (
                ph2d_tool_vector::ids::VECTOR_FILL_KIND_RADIAL,
                tr("panel.vector.arrange.radial"),
                FillKind::Radial,
            ),
            (
                ph2d_tool_vector::ids::VECTOR_FILL_KIND_MULTI,
                tr("panel.vector.arrange.multi"),
                FillKind::MultiPoint,
            ),
            (
                ph2d_tool_vector::ids::VECTOR_FILL_KIND_PATTERN,
                tr("panel.vector.arrange.pattern"),
                FillKind::Pattern,
            ),
        ];
        // ⭐⭐ **A fileira REFLUI, e deixou de reimplementar a aritmética de largura.**
        //
        // Ela repartia `inner_w` à mão por `n` chips — a MESMA expressão que o
        // `paint_segmented_group_adaptive` já é, e que o doc-comment do [`Self::segmented`] logo
        // acima chama de *"uma segunda regra de quebra vivendo neste painel"*. Com quatro chips
        // ninguém notava; o **quinto** (o padrão, plano 33) é o que a faz morder: cada chip novo
        // encolhe TODOS, de `58,5` para `45,6` px (medido), e o painter **não trunca** — um rótulo
        // largo demais transborda para cima do vizinho.
        //
        // ⭐⭐ **E a medição fechou o círculo.** Com a repartição à mão, os cinco chips ficavam a
        // `45,60 px` cada e `"Pattern"` media `41,37` — folga **negativa**, e o rótulo teria de ser
        // `"Tile"`. Com o refluxo, os quatro primeiros voltam aos `60,0 px` **de antes** e o quinto
        // ganha uma linha INTEIRA de `252 px`: a restrição que ditava o rótulo curto **dissolveu-se
        // ao curar a fileira**, e o nome de produto cabe. *Quem move o número que tornava algo
        // inalcançável tem de reconferir a nota.*
        //
        // Os dois gates que o medem: `every_fill_kind_chip_label_fits_its_chip` (a folga de cada
        // rótulo, com o painter REAL e o sistema de texto REAL) e `the_five_chips_do_not_overlap`.
        let segs: Vec<(&str, bool, ph2d_a11y::NodeId)> = kinds
            .iter()
            .map(|(id, label, k)| (*label, kind == *k, *id))
            .collect();
        // O losango é o da TINTA inteira (o tipo é membro dela), na 1.ª fileira do refluxo.
        let (w, dot) =
            ph2d_editor_core::widget::form_row_columns(self.inner_x, self.inner_w, y, self.row_h);
        self.decorator(dot, &[ph2d_tool_vector::ids::VECTOR_FILL_KIND_SOLID]);
        let used = paint_segmented_group_adaptive(
            Rect::new(self.inner_x, y, w, self.row_h),
            &segs,
            self.scene,
            self.text_system,
            self.theme,
            self.store,
            self.hit_index,
        );
        let gap = Spacing::Xs.px();
        y += used + self.row_gap;

        // Multi-point: Add / Remove point (drag points on-canvas to move them) +
        // Influence slider for the selected point.
        if kind == FillKind::MultiPoint {
            let two_col = ((self.inner_w - gap) / 2.0).max(1.0);
            y = self.row2(
                two_col,
                gap,
                [
                    (
                        crate::ids::VECTOR_GRAD_ADD_POINT,
                        tr("panel.vector.arrange.add_point"),
                    ),
                    (
                        crate::ids::VECTOR_GRAD_REMOVE_POINT,
                        tr("panel.vector.arrange.remove_point"),
                    ),
                ],
                y,
            );
            if let Some(inf) = state::current_grad_influence() {
                const MAX_INFLUENCE: f64 = 4.0; // LITERAL-PX-OK: IDW strength range (domain)
                let track = self
                    .store
                    .slider(crate::ids::VECTOR_GRAD_INFLUENCE)
                    .map(|(_, v)| v)
                    .unwrap_or((inf / MAX_INFLUENCE) as f32);
                let val = f64::from(track) * MAX_INFLUENCE;
                y = self.slider_row(
                    tr("panel.vector.arrange.influence"),
                    crate::ids::VECTOR_GRAD_INFLUENCE,
                    ph2d_tool_vector::ids::VECTOR_GRAD_INFLUENCE_NUM,
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
                    .slider(crate::ids::VECTOR_GRAD_JITTER)
                    .map(|(_, v)| v)
                    .unwrap_or(jit as f32);
                let val = f64::from(track);
                y = self.slider_row(
                    tr("panel.vector.arrange.jitter"),
                    crate::ids::VECTOR_GRAD_JITTER,
                    ph2d_tool_vector::ids::VECTOR_GRAD_JITTER_NUM,
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
                    (
                        crate::ids::VECTOR_GRAD_ADD_STOP,
                        tr("panel.vector.arrange.add_stop"),
                    ),
                    (
                        crate::ids::VECTOR_GRAD_REMOVE_STOP,
                        tr("panel.vector.arrange.remove_stop"),
                    ),
                ],
                y,
            );
        }
        // Angle slider (Linear only) — track 0..1 → 0..360°.
        if kind == FillKind::Linear {
            let angle = state::current_grad_angle().unwrap_or(0.0);
            let track = self
                .store
                .slider(crate::ids::VECTOR_GRAD_ANGLE)
                .map(|(_, v)| v)
                .unwrap_or((angle / FULL_TURN_DEG) as f32);
            let deg = f64::from(track) * FULL_TURN_DEG;
            y = self.slider_row(
                tr("panel.vector.arrange.angle"),
                crate::ids::VECTOR_GRAD_ANGLE,
                ph2d_tool_vector::ids::VECTOR_GRAD_ANGLE_NUM,
                track,
                deg,
                &format!("{}", deg.round() as i64),
                y,
            );
        }
        y
    }
}
