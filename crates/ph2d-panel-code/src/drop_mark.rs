//! ⭐ **Onde um arrasto vai cair** (item 8 do CodeEditor, `docs/CodeEditor/07_arrastar_para_o_codigo.md`
//! §3.11): o cursor fino no byte da soltura e, no canto de baixo à direita do texto, o balão com as
//! teclas — o do editor do Godot 4.7.2 («Drop node path. Hold Ctrl: …»,
//! `ferramentas/oraculo_arrastar/foto_01_a_meio_do_arrasto.png`), no `Popover` do design system.

use crate::chrome::{Ui, font};
use crate::frame::CodeFrame;
use crate::palette::Palette;
use crate::view::CodeView;
use ph2d_a11y::NodeId;
use ph2d_editor_core::paint::paint_text;
use ph2d_editor_core::widget::{Popover, paint_popover};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Spacing;
use ph2d_vector::VectorScene;

/// O arrasto sobre o texto: o byte onde cairia e as linhas do balão.
pub struct DropView<'a> {
    pub at: usize,
    pub hint: &'a [String],
}

impl CodeView {
    /// O cursor da soltura (dentro do recorte do texto).
    pub(crate) fn paint_drop_caret(
        &self,
        frame: &CodeFrame<'_>,
        scene: &mut VectorScene,
        pal: &Palette,
    ) {
        let Some(d) = &frame.dropping else { return };
        if let Some(c) = self.caret_rect(frame.buffer, d.at) {
            scene.fill_rect(ph2d_editor_core::paint::rect_to_vello(c), pal.caret);
        }
    }

    /// O balão das teclas, no canto de baixo à direita do texto (por cima de tudo).
    pub(crate) fn paint_drop_hint(
        &self,
        frame: &CodeFrame<'_>,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        pal: &Palette,
    ) {
        let Some(d) = frame.dropping.as_ref().filter(|d| !d.hint.is_empty()) else {
            return;
        };
        let tr = self.text_rect();
        let (pad, f) = (Spacing::Sm.px(), font());
        let line = f + Spacing::Xs.px();
        let chars = d.hint.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let w = (chars as f32 * self.m.advance + 2.0 * pad).min(tr.w);
        let h = d.hint.len() as f32 * line + 2.0 * pad;
        let rect = Rect::new(tr.x + tr.w - w - pad, tr.y + tr.h - h - pad, w, h);
        paint_popover(&Popover::new(NodeId(0)), rect, scene, pal.theme);
        let ui = Ui::new(pal.theme);
        for (i, l) in d.hint.iter().enumerate() {
            let y = rect.y + pad + i as f32 * line;
            paint_text(ts, scene, l, rect.x + pad, y, f, w - 2.0 * pad, ui.text2);
        }
    }
}
