//! **O rascunho é a PRÓPRIA água** — a autoria dos métodos que re-carimbam a figura inteira
//! (Line · Curve · Ellipse · Polygon · Free Hand · Drag Dot · Anchored) em modo Wet Paint.
//!
//! ## O que isto substitui, e por quê
//!
//! O doc 21 desenhou a autoria como um **preview flat ESTÁTICO pelo pipeline digital**: o esboço era
//! pintado pelo roteador de cor comum, o lote exato ficava num stash (`pending_deposit`) e só o gesto
//! de commit o replayava para dentro do fluido — *"o esboço derrete"*. Enio, 2026-08-07: *"o traço de
//! wet paint nos strokes vivos antes da simulação ainda é o digital normal. Deve ser o traço do
//! próprio Wet painter."* O smoke decidiu o item que o CLAUDE.md listava como aberto.
//!
//! ## A dança, e por que ela é a MESMA que o canvas já faz
//!
//! Um método de re-stamp mostra só a última posição, então ele *restaura o pristino e re-carimba a
//! figura inteira* a cada quadro ([`PainterTool::stamp_drag_preview`]). A água não é idempotente, então
//! depositar por quadro empilharia — a cura é a dança que o canvas já executa, uma camada abaixo:
//!
//! ```text
//!   restaura o recorte do quadro anterior  →  recorta a janela nova  →  deposita  →  composita
//! ```
//!
//! O recorte é [`ph2d_wet_paint::grid::snapshot_grid_region`] (a metade REGIONAL do snapshot de folha
//! inteira, que a 4096² seriam centenas de MB por quadro).
//!
//! ## As três coisas que fazem isto funcionar
//!
//! 1. **A sim SEGURA sob a mão** ([`PainterTool::wet_authoring_hold`]) — já era assim. Sem o hold a água
//!    do rascunho FLUIRIA para fora da janela recortada e o restore deixaria rastro.
//! 2. **O depósito é o do commit**, pela mesma porta (`deposit_pass` ⇒ [`PainterTool::wet_owns_the_dabs`]),
//!    então Tiling, Symmetry, a silhueta do pincel, a pressão e as lanes chegam ao fluido exatamente
//!    como chegariam no commit — *o que se autora É o que fica*, sem nada re-derivado depois.
//! 3. **O commit não deposita nada**: ele apenas PARA de restaurar. Não há esboço para derreter,
//!    porque nunca houve esboço.

use super::*;

/// Quantas células de folga o recorte ganha além da caixa dos dabs — o depósito escreve a silhueta do
/// dab e o vizinho que a interpolação lê, e um recorte apertado deixaria uma orla que o restore não
/// alcança (o rastro de um texel que o §13.12 já pagou no canvas).
const PATCH_PAD_CELLS: i32 = 3;

impl PainterTool {
    /// `true` quando o rascunho de autoria deve ser a própria água.
    ///
    /// ⚠️ **Os métodos INCREMENTAIS não entram**: neles cada Move deposita dabs permanentes ao longo do
    /// caminho — não há re-stamp, logo não há o que desfazer, e eles já iam direto ao fluido
    /// ([`PainterTool::wet_owns_the_dabs`]). O eraser também não: uma borracha plana é a sua própria
    /// edição, e é foreign para a água por desenho (W2.6).
    pub(in crate::tool::paint) fn wet_preview_owns_the_gesture(&self) -> bool {
        matches!(self.paint.paint_mode, PaintMode::WetPaint)
            && !self.paint.eraser
            && !self.paint.brush.stroke_method.is_incremental()
            && self.paint.wetpaint.armed
    }

    /// A janela do recorte, em CÉLULAS de interior, para um lote de dabs — `None` quando o lote não
    /// alcança a folha.
    fn wet_patch_window(&self, dabs: &[Dab]) -> Option<(i32, i32, i32, i32)> {
        let region = self.dab_batch_region(dabs)?;
        let sess = self.paint.wetpaint.session.as_ref()?;
        let r = sess.ratio;
        let x0 =
            super::grid_map::px_to_cell(f64::from(region.x), r).floor() as i32 - PATCH_PAD_CELLS;
        let y0 =
            super::grid_map::px_to_cell(f64::from(region.y), r).floor() as i32 - PATCH_PAD_CELLS;
        let x1 = super::grid_map::px_to_cell(f64::from(region.x + region.w), r).ceil() as i32
            + PATCH_PAD_CELLS;
        let y1 = super::grid_map::px_to_cell(f64::from(region.y + region.h), r).ceil() as i32
            + PATCH_PAD_CELLS;
        Some((x0, y0, x1, y1))
    }

    /// Restaura o recorte vivo (se houver) e marca a janela suja, para o composite re-derivar os pixels
    /// dela a partir do grid que voltou. A porta ÚNICA do desfazer — o quadro seguinte, o Esc e a morte
    /// do gesto passam todos por aqui.
    /// O desfazer COMPLETO de um rascunho: o recorte volta ao grid **e a tela volta com ele** — sem o
    /// composite o pixel continua mostrando a água que já não existe (o Esc que "não devolvia o
    /// canvas", pego pelo gate `esc_returns_the_water_alive`).
    pub(in crate::tool::paint) fn wet_peel_preview(&mut self) {
        if self.wet_restore_preview_patch() {
            self.wetpaint_composite();
            self.wetpaint_rearm_after_own_write();
        }
        self.paint.wetpaint.pending_deposit.clear();
    }

    pub(in crate::tool::paint) fn wet_restore_preview_patch(&mut self) -> bool {
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return false;
        };
        let Some(patch) = sess.preview.take() else {
            return false;
        };
        sess.bring_home();
        let (x0, y0, x1, y1) = patch.rect();
        let layer = sess.engine.active_layer;
        let grid = &mut sess.engine.layers[layer].grid;
        ph2d_wet_paint::grid::restore_grid_region(grid, &patch);
        sess.engine.merge_dirty(x0, y0, x1, y1);
        true
    }

    /// O quadro de autoria: restaura o anterior, recorta a janela nova, deposita o lote no fluido e
    /// composita. Substitui a dança flat do pipeline digital para os métodos de re-stamp.
    pub(in crate::tool::paint) fn wet_stamp_drag_preview(&mut self, dabs: &[Dab]) {
        // A sessão TEM de existir antes do recorte: sem ela o depósito a criaria lazily e o primeiro
        // quadro do gesto ficaria sem recorte — o único quadro que o restore do quadro seguinte não
        // desfaria, e o rastro que ele deixa é permanente.
        self.ensure_wet_session();
        self.wet_restore_preview_patch();
        let Some((x0, y0, x1, y1)) = self.wet_patch_window(dabs) else {
            // Nada a depositar (lote vazio / fora da folha) — o restore acima já devolveu o pristino,
            // e sem janela não há recorte novo a guardar.
            self.wetpaint_composite();
            self.wetpaint_rearm_after_own_write();
            return;
        };
        if let Some(sess) = self.paint.wetpaint.session.as_mut() {
            sess.bring_home();
            let layer = sess.engine.active_layer;
            let grid = &sess.engine.layers[layer].grid;
            sess.preview = ph2d_wet_paint::grid::snapshot_grid_region(grid, x0, y0, x1, y1);
        }
        // O MESMO depósito do commit, pela mesma porta: `deposit_pass` faz o `wet_owns_the_dabs`
        // devolver `true` para um lote não-incremental, e daí Tiling / Symmetry / silhueta / pressão
        // chegam ao fluido exatamente como chegariam no commit.
        self.paint.wetpaint.deposit_pass = true;
        self.stamp_dabs(dabs);
        self.paint.wetpaint.deposit_pass = false;
        // ⚠️ **Fecha o traço do motor a cada quadro**, como o commit faz — um quadro de autoria é um
        // depósito COMPLETO da figura, não um pedaço de um traço contínuo. Deixá-lo aberto trava o
        // `hand_off_sim` (que recusa com `stroke_open`) e a pausa da autoria vira PERMANENTE: a água
        // nunca mais volta a correr depois do Apply. Achado pelo gate `the_authoring_pause_stops_the_
        // sim_and_not_merely_the_display`, cuja segunda metade existe exatamente para isso.
        self.wetpaint_stroke_end();
        self.wetpaint_composite();
        self.wetpaint_rearm_after_own_write();
        // ⚠️ **O stash SOBREVIVE, e não é redundância.** O recorte responde *"desfaça o rascunho"*; o
        // stash responde *"deposite ISTO de novo"* — e são gestos diferentes: o primeiro Apply & Keep
        // fixa o rascunho que já está na água (não deposita nada), e o SEGUNDO aperto, sem rascunho
        // vivo, cai na porta de commit do doc 21 e carimba a figura outra vez. É o que *"Apply & Keep
        // deposita POR aperto"* significa, e sem ele o segundo aperto seria um no-op silencioso.
        self.paint.wetpaint.pending_deposit.clear();
        self.paint.wetpaint.pending_deposit.extend_from_slice(dabs);
    }

    /// `true` enquanto um recorte de autoria está vivo — a pergunta *"a mão está sobre a água?"* na
    /// forma que sobrevive ao `drag_preview` do canvas ter deixado de existir neste modo.
    pub(in crate::tool::paint) fn wet_preview_is_live(&self) -> bool {
        self.paint
            .wetpaint
            .session
            .as_ref()
            .is_some_and(|s| s.preview.is_some())
    }

    /// O commit: o depósito já ESTÁ na água, então commitar é apenas **parar de restaurar**.
    ///
    /// ⚠️ É aqui que a sessão volta a andar: soltar o recorte faz o `wet_authoring_hold` devolver
    /// `false` e o tick volta a pedir passos ao worker.
    pub(in crate::tool::paint) fn wet_commit_preview(&mut self) {
        if let Some(sess) = self.paint.wetpaint.session.as_mut() {
            sess.preview = None;
        }
    }
}
