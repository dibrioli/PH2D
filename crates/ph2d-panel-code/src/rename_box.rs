//! ⭐ **A caixinha do rename** (CodeEditor Onda 2, I10): por baixo do nome (por cima se não couber),
//! o campo com o nome novo e uma linha por baixo — a ajuda («Enter to rename…») ou, a vermelho, porque
//! o nome não serve. É a do VS Code 1.142.0 (`docs/CodeEditor/ferramentas/medicoes_onda2/
//! I10_rename_e_undo.md` §2), sobre o `Popover` do design system e o campo da barra de busca.

use crate::chrome::{Ui, font, text_field};
use crate::frame::{COMPLETION_MAX_CHARS, CodeFrame};
use crate::palette::Palette;
use crate::view::CodeView;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_editor_core::paint::paint_text;
use ph2d_editor_core::widget::{Popover, paint_popover};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Spacing;
use ph2d_vector::VectorScene;

/// A largura mínima da caixinha, em colunas da mono (cabe a ajuda numa linha).
const RENAME_MIN_CHARS: usize = 32;

/// A distância da id da caixinha ao `Document` (o bloco da aba; a lista usa `1 << 31`).
const A11Y_RENAME: u64 = (1 << 31) + (1 << 30);

impl CodeView {
    pub(crate) fn paint_rename(
        &mut self,
        frame: &CodeFrame<'_>,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        pal: &Palette,
    ) {
        self.rename_box = None;
        let Some(r) = &frame.rename else { return };
        let Some(at) = self.caret_rect(frame.buffer, r.word.start) else {
            return;
        };
        let tr = self.text_rect();
        let pad = Spacing::Xs.px();
        let line = font() + 2.0 * Spacing::Sm.px();
        let note = r.error.unwrap_or(r.hint);
        let chars = ph2d_editor_core::math::safe_clamp_ord(
            note.chars().count(),
            RENAME_MIN_CHARS,
            COMPLETION_MAX_CHARS,
        );
        let w = (chars as f32 * self.m.advance + 2.0 * pad).min(tr.w);
        let h = r.field.map_or(0.0, |_| line + pad) + line + 2.0 * pad;
        let x = (at.x - pad).min(tr.x + tr.w - w).max(tr.x);
        let below = at.y + at.h;
        let y = if below + h <= tr.y + tr.h || at.y - h < tr.y {
            below
        } else {
            at.y - h
        };
        let rect = Rect::new(x, y, w, h);
        paint_popover(&Popover::new(NodeId(0)), rect, scene, pal.theme);
        let ui = Ui::new(pal.theme);
        let mut y0 = y + pad;
        if let Some(buf) = r.field {
            text_field(
                scene,
                ts,
                buf,
                "",
                Rect::new(x + pad, y0, w - 2.0 * pad, line),
                true,
                &ui,
            );
            y0 += line + pad;
        }
        let ink = if r.error.is_some() {
            pal.error
        } else {
            ui.text2
        };
        let f = font();
        paint_text(
            ts,
            scene,
            note,
            x + pad,
            y0 + (line - f) * 0.5,
            f,
            w - 2.0 * pad,
            ink,
        );
        self.rename_box = Some((rect, note.to_owned(), r.field.map(|b| b.text().to_string())));
    }

    /// A caixa da caixinha do rename do último quadro.
    #[must_use]
    pub fn rename_rect(&self) -> Option<Rect> {
        self.rename_box.as_ref().map(|(r, ..)| *r)
    }

    /// O ponto está dentro da caixinha do rename do último quadro.
    #[must_use]
    pub fn rename_contains(&self, x: f32, y: f32) -> bool {
        self.rename_box
            .as_ref()
            .is_some_and(|(r, ..)| r.contains(x, y))
    }

    /// O campo da caixinha para o leitor de ecrã: um `TextInput` filho do `Document`, com o nome
    /// escrito no valor e a ajuda (ou a razão) no rótulo.
    pub(crate) fn rename_a11y(&self, doc: NodeId) -> Option<(NodeId, Node)> {
        let (rect, note, field) = self.rename_box.as_ref()?;
        let id = NodeId(doc.0 + A11Y_RENAME);
        let b = NodeBuilder::new(if field.is_some() {
            Role::TextInput
        } else {
            Role::Tooltip
        })
        .label(ph2d_i18n::tr("code.a11y.rename").replace("{note}", note))
        .bounds(
            f64::from(rect.x),
            f64::from(rect.y),
            f64::from(rect.w),
            f64::from(rect.h),
        );
        let b = match field {
            Some(v) => b.value(v.clone()),
            None => b,
        };
        Some((id, b.build()))
    }
}
