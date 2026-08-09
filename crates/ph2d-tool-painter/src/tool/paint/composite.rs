//! **Composite Brush** — run Brush + Smear + Blur together as a reorderable 3-layer stack.
//!
//! An upgrade to the Brush tool (a panel checkbox, not a rail tool): when on, one stroke applies all
//! three operations per dab, each with its own Strength. The three layers occupy FIXED positions
//! numbered 1 (top) · 2 · 3 (bottom); the tool at each position is reordered with the panel's up/down
//! buttons. The stroke runs the stack **bottom → top** (position 3 first), so each operation processes
//! the canvas as modified by the one below it — e.g. Brush(3) → Smear(2) → Blur(1) paints, then smears
//! that, then blurs the result; Blur(3) → Smear(2) → Brush(1) blurs the canvas, smears it, then paints
//! clean strokes on top (untouched by the blur/smear below). Split from `paint.rs` for the LOC cap.
//!
//! Only the Brush operation paints colour; the shared brush parameters (Size / Shape / Grain / Falloff
//! / Tiling / Jitter / Symmetry / Stroke) drive all three ops' dab geometry, while the colour-family
//! parameters (Color / Blend / ramps / Randomize / Accumulate) only affect the Brush layer — so the
//! panel keeps every control visible in composite mode (see `BrushSettings::paints_no_color`).

use super::PaintMode;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::Dab;

/// One of the three composite operations. Wire discriminant (`to_u8`) travels in the panel snapshot so
/// the panel can label each row; the panel maps it back to a name.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum CompositeOp {
    Brush,
    Smear,
    Blur,
}

impl CompositeOp {
    /// Wire discriminant for the panel snapshot (`0` Brush · `1` Smear · `2` Blur).
    pub(crate) fn to_u8(self) -> u8 {
        match self {
            Self::Brush => 0,
            Self::Smear => 1,
            Self::Blur => 2,
        }
    }
}

/// One composite stack layer: which operation sits here + its Strength (`0..1`; `0` = the layer is a
/// no-op and is skipped). Stored in a fixed `[_; 3]` in display order (index 0 = layer 1 = top).
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct CompositeLayer {
    pub op: CompositeOp,
    pub strength: f32,
}

impl PainterTool {
    /// Whether the composite stack drives this stroke: the checkbox is on AND the active operation is
    /// the plain Brush (not a Smear/Blur/Eraser rail tool — composite is a Brush-tool upgrade).
    pub(crate) fn composite_active(&self) -> bool {
        self.paint.composite_enabled
            && matches!(self.paint.paint_mode, PaintMode::Paint)
            && !self.paint.eraser
    }

    /// Toggle the Composite Brush on/off (panel checkbox). Plain state — touches no pixels.
    pub fn toggle_composite(&mut self) {
        self.paint.composite_enabled = !self.paint.composite_enabled;
    }

    /// The Composite Brush enable flag (the panel snapshot mirrors it to show the card + hide Strength).
    #[must_use]
    pub fn composite_enabled(&self) -> bool {
        self.paint.composite_enabled
    }

    /// Set the Strength (`0..1`) of the composite layer at `pos` (`0` = layer 1 … `2` = layer 3).
    pub fn set_composite_layer_strength(&mut self, pos: usize, t: f32) {
        if pos < 3 {
            self.paint.composite[pos].strength = t.clamp(0.0, 1.0);
        }
    }

    /// Move the layer at `pos` one position UP (toward layer 1 / top) — swaps the tool with its upper
    /// neighbour. The position NUMBERS stay fixed; only which tool sits where changes. No-op at the top.
    pub fn move_composite_layer_up(&mut self, pos: usize) {
        if (1..3).contains(&pos) {
            self.paint.composite.swap(pos, pos - 1);
        }
    }

    /// Move the layer at `pos` one position DOWN (toward layer 3 / bottom). No-op at the bottom.
    pub fn move_composite_layer_down(&mut self, pos: usize) {
        if pos < 2 {
            self.paint.composite.swap(pos, pos + 1);
        }
    }

    /// The per-position operation discriminants `[layer1, layer2, layer3]` — for the panel snapshot.
    pub(crate) fn composite_ops_u8(&self) -> [u8; 3] {
        [
            self.paint.composite[0].op.to_u8(),
            self.paint.composite[1].op.to_u8(),
            self.paint.composite[2].op.to_u8(),
        ]
    }

    /// The per-position layer Strengths `[layer1, layer2, layer3]` — for the panel snapshot.
    pub(crate) fn composite_strengths(&self) -> [f32; 3] {
        [
            self.paint.composite[0].strength,
            self.paint.composite[1].strength,
            self.paint.composite[2].strength,
        ]
    }

    /// Route the Composite-card panel events (enable checkbox, per-position reorder buttons, per-position
    /// Strength sliders). Returns `true` iff consumed — chained ahead of the big `handle_panel_event` match.
    pub(crate) fn route_composite_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        match event {
            PanelEvent::Click(id) => {
                if *id == core_ids::PAINTER_BRUSH_COMPOSITE_ENABLE {
                    self.toggle_composite();
                    return true;
                }
                if let Some(p) = core_ids::PAINTER_BRUSH_COMPOSITE_UP
                    .iter()
                    .position(|x| x == id)
                {
                    self.move_composite_layer_up(p);
                    return true;
                }
                if let Some(p) = core_ids::PAINTER_BRUSH_COMPOSITE_DOWN
                    .iter()
                    .position(|x| x == id)
                {
                    self.move_composite_layer_down(p);
                    return true;
                }
                false
            }
            PanelEvent::SetValue(id, v) => {
                if let Some(p) = core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH
                    .iter()
                    .position(|x| x == id)
                {
                    self.set_composite_layer_strength(p, *v as f32);
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    /// Stamp a dab batch through the composite stack (bottom → top). Each operation reuses its own route
    /// (`stamp_dabs_inner` / `stamp_dabs_smear` / `stamp_dabs_blur`) with the layer's Strength swapped
    /// into the brush spec; a zero-Strength layer is skipped. The per-op routes each read/write the
    /// canvas in place, so an upper op sees the lower op's result (the "affects the combination" order).
    pub(super) fn stamp_dabs_composite(&mut self, dabs: &[Dab]) {
        if dabs.is_empty() {
            return;
        }
        let (w, h) = self.source_size;
        let tiling = self.paint.tiling;
        let tiled = tiling[0] || tiling[1];
        let saved_strength = self.paint.brush.strength;
        // Bottom (position 2 / layer 3) → top (position 0 / layer 1).
        for pos in (0..3).rev() {
            let layer = self.paint.composite[pos];
            if layer.strength <= 0.0 {
                continue;
            }
            self.paint.brush.strength = layer.strength;
            match layer.op {
                CompositeOp::Brush => {
                    // The brush route expects already-tiled dabs (the Smear/Blur routes tile internally).
                    let wrapped =
                        tiled.then(|| super::tiling::tiled_dabs(dabs, self.source_size, tiling));
                    let d: &[Dab] = wrapped.as_deref().unwrap_or(dabs);
                    self.lay_into_smear_base(|t| t.stamp_dabs_inner(d));
                    self.stamp_dabs_inner(d);
                }
                CompositeOp::Smear => self.stamp_dabs_smear(dabs, w, h),
                CompositeOp::Blur => {
                    // ⚠️ O Blur precisa da MESMA porta que o Brush, pelo mesmo motivo: ele escreve só o
                    // canvas, e o render do smear do batch seguinte reescreve a região a partir de
                    // `pre` — que nunca viu o blur. O resultado era o blur DESFEITO dentro da região
                    // renderizada e vivo fora dela, com a união de rects como fronteira.
                    self.lay_into_smear_base(|t| t.stamp_dabs_blur(dabs, w, h));
                    self.stamp_dabs_blur(dabs, w, h);
                }
            }
        }
        self.paint.brush.strength = saved_strength;
    }
}

impl PainterTool {
    /// Lay the Brush layer's deposit into the smear session's frozen source **by the same door that lays
    /// it on the canvas** — the plane is swapped into `canvas_rgba` for the stamp and swapped back.
    ///
    /// ⚠️ **Sem dobra nenhuma a pilha não pinta mais que uma mancha** (Enio 2026-08-09): desde que o smear
    /// virou CAMPO, uma esfregada *acumula um mapa de deslocamento e resolve UMA vez a partir dos pixels
    /// congelados no pen-down* — a lei que matou o filamento —, enquanto o composite promete o oposto,
    /// *cada operação processa o canvas como a de baixo o deixou*, que é por BATCH. O render de smear do
    /// batch seguinte reescrevia a região a partir de uma base que nunca vira o Brush (108 de 141 colunas).
    ///
    /// ⚠️ **TRÊS dobras reconstruídas de FORA do depósito foram construídas, e cada uma falhou de um jeito
    /// — a terceira é a razão desta existir:** copiar a REGIÃO do canvas dá 141 colunas mas escreve a bbox
    /// do batch na FONTE (⇒ a escada axis-aligned que o smoke fotografou, e o smear já feito volta para
    /// dentro ⇒ as estrias) · SOMAR o delta do Brush dá **131** (sobre pixel já esfregado o incremento é
    /// pequeno ⇒ perde tinta) · recuperar `a` de `after = before·(1−a) + C·a` dá **108**, zero em toda
    /// parte, porque a cor e o espaço com que o depósito compõe não são `brush.color` em sRGB de 8 bits.
    ///
    /// ⇒ **Só quem deposita sabe `(C, a)` por texel.** Trocar o plano para dentro do canvas durante o
    /// stamp é o padrão que este repo já usa duas vezes (o scratch da máscara · o plano `free` do gate de
    /// proteção) e dá o resultado **exato por construção**: a fonte recebe a MESMA composição, delimitada
    /// pelo **falloff do dab** — nenhuma borda de retângulo pode nascer, porque não existe retângulo em
    /// parte alguma da operação.
    ///
    /// ⚠️ **`DrawTo::Color` no passe da fonte, e é obrigatório:** sem ele o segundo depósito acumularia o
    /// envelope de relevo uma segunda vez, e o CORPO da tinta passaria a ser função de haver uma sessão de
    /// smear viva — o relevo dependendo de qual camada está na pilha.
    ///
    /// ⚠️ **O Blur usa a MESMA porta** — toda camada da pilha que NÃO é o smear precisa dela, senão o
    /// render do smear desfaz o trabalho dela dentro da região que re-resolve.
    ///
    /// **Mutação que must bleed:** apagar a chamada ⇒ 108 de 141.
    fn lay_into_smear_base(&mut self, op: impl FnOnce(&mut Self)) {
        if !self.paint.warp.active || self.paint.warp.pre.len() != self.canvas_rgba.len() {
            return;
        }
        // ⚠️ Pela PORTA, não por `mem::replace` cru: é ela que chama `toggle_foreign_plane`, e sem isso
        // todo `fork_canvas` das rotas captura os bytes da FONTE achando que são a tela — e *a primeira
        // captura de cada tile é a que vale*, então a poluição do journal é permanente.
        let mut plane = std::mem::take(&mut self.paint.warp.pre);
        super::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut plane,
            &self.undo.write_state,
        );
        // ⚠️ Este passe roda o depósito uma SEGUNDA vez no mesmo batch, e os estados que ele consome
        // são por-TRAÇO, não por-passe. Sem os salvar aqui:
        // • `stroke_mask` é o cap de Accumulate — com Strength < 1 o passe da fonte leva a cobertura ao
        //   teto e o passe do canvas deposita **ZERO**, e a tinta só reaparece onde o smear a traz de
        //   volta da fonte: dentro de um retângulo, com fronteira axis-aligned;
        // • `tex_rng` é um stream CONSUMIDO, não copiado — sem salvá-lo a fonte recebe uma realização
        //   de Grain/Random/Randomize e o canvas recebe a SEGUINTE, quebrando a promessa desta função.
        let saved_mask = self.paint.stroke_mask.clone();
        let saved_rng = self.paint.tex_rng;
        let saved_draw = self.paint.brush.impasto_draw_to;
        self.paint.brush.impasto_draw_to = ph2d_painter_brush::DrawTo::Color;
        op(self);
        self.paint.brush.impasto_draw_to = saved_draw;
        self.paint.tex_rng = saved_rng;
        self.paint.stroke_mask = saved_mask;
        super::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut plane,
            &self.undo.write_state,
        );
        self.paint.warp.pre = plane;
    }
}
