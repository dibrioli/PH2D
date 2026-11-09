//! **Arrastar a letra de um campo** (dono 07/11): a lei é a do campo de cartão
//! ([`ph2d_editor_core::interaction::field_scrub`]); aqui, o que cada campo do editor arrasta e escreve.

use super::{AnimDoc, Drag, Field, Layout, hit};
use ph2d_ecs::{FRAME_MS_MAX, FRAME_MS_MIN, fps_of_ms};
use ph2d_editor_core::AnimFieldEdit;
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::zones::Rect;

/// O multiplicador do quadro anda isto por pixel de scrub (os inteiros andam 1).
const FACTOR_STEP: f64 = 0.1; // LITERAL-PX-OK: passo de valor

impl AnimDoc {
    /// O que um valor escreve num campo de número (escrito ou arrastado).
    pub(super) fn number_edit(&self, field: &Field, v: f64) -> Option<AnimFieldEdit> {
        let row = self.row();
        let (c, r) = (self.snap.hframes, self.snap.vframes);
        match *field {
            Field::Fps => Some(AnimFieldEdit::FrameMs(row?, ph2d_ecs::ms_of_fps(v as f32))),
            Field::FrameMs(i) => {
                let ms = self.ms_of_factor(v as f32)?;
                Some(AnimFieldEdit::FrameMsAt(row?, i as u32, ms))
            }
            Field::Cols => Some(AnimFieldEdit::Grid(v.round() as u32, r)),
            Field::Rows => Some(AnimFieldEdit::Grid(c, v.round() as u32)),
            Field::Name(_) | Field::Signal(_) => None,
        }
    }

    /// O valor, o passo por pixel e o intervalo do modelo de um campo com letra.
    pub(super) fn scrub_law(&self, field: &Field) -> Option<(f64, f64, (f64, f64))> {
        let side = (1.0, f64::from(crate::GRID_SIDE_MAX));
        let row = self
            .snap
            .info
            .as_ref()
            .and_then(|i| i.rows.get(self.selected));
        let (lo, hi) = (f64::from(FRAME_MS_MIN), f64::from(FRAME_MS_MAX));
        match *field {
            Field::Fps => {
                let fps = |m: u32| f64::from(fps_of_ms(m));
                Some((
                    fps(row?.frame_ms),
                    1.0,
                    (fps(FRAME_MS_MAX), fps(FRAME_MS_MIN)),
                ))
            }
            Field::FrameMs(i) => {
                let m = f64::from(row?.frame_ms.max(1));
                Some((
                    f64::from(self.frame_factor_at(i)),
                    FACTOR_STEP,
                    (lo / m, hi / m),
                ))
            }
            Field::Cols => Some((f64::from(self.snap.hframes), 1.0, side)),
            Field::Rows => Some((f64::from(self.snap.vframes), 1.0, side)),
            Field::Name(_) | Field::Signal(_) => None,
        }
    }

    /// O campo cuja letra está sob `(x, y)`, se o campo aceita valor agora.
    pub(super) fn scrub_target(&self, x: f32, y: f32) -> Option<Field> {
        let l = &self.layout;
        let on = |r: Rect| hit(Layout::handle(r), x, y);
        let field = if on(l.fps) {
            Field::Fps
        } else if on(l.this_frame) {
            Field::FrameMs(self.marked_frame()?)
        } else if on(l.cols) {
            Field::Cols
        } else if on(l.rows) {
            Field::Rows
        } else {
            return None;
        };
        Some(field)
    }

    /// A seta ↔ sobre uma letra que arrasta, e durante o arrasto (`None` = a shell decide).
    #[must_use]
    pub fn pointer_look(&self, x: f32, y: f32) -> Option<PointerLook> {
        if self.align.picking {
            return Some(PointerLook::Pick);
        }
        let pega = self.scrub_target(x, y).and_then(|f| self.scrub_law(&f));
        (matches!(self.drag, Drag::Scrub(..)) || pega.is_some()).then_some(PointerLook::ResizeEw)
    }

    /// O Esc a meio do scrub: o valor do Down volta. `false` = não havia scrub.
    pub fn cancel_scrub(&mut self) -> bool {
        if !matches!(self.drag, Drag::Scrub(..)) {
            return false;
        }
        let Drag::Scrub(f, s, _) = std::mem::take(&mut self.drag) else {
            return false;
        };
        if s.moved() {
            self.pending.extend(self.number_edit(&f, s.start));
        }
        true
    }
}
