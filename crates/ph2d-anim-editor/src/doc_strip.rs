//! **A tira de quadros**: onde um quadro largado entra, o arrasto que rola a tira, e a MARCA DE INSERÇÃO (dono 08/11):
//! um espaço quadriculado entre dois quadros, com «INSERT HERE» na vertical; as células escolhidas na folha entram ALI,
//! e a marca anda para depois delas. Agarra-se e arrasta-se, ou clica-se num vão da tira.

use super::{AnimDoc, Drag, FrameDrag};
use ph2d_editor_core::AnimFrameRef;
use ph2d_editor_core::zones::Rect;

impl AnimDoc {
    /// ⭐ **A lei única do marcador e da largada**: o intervalo (`0..=n`) onde um quadro largado em
    /// `x` entra — antes do primeiro quadro pintado cujo centro está à direita de `x`.
    #[must_use]
    pub fn drop_gap(&self, x: f32) -> usize {
        self.layout
            .frames
            .iter()
            .filter(|f| f.0.x + f.0.w / 2.0 <= x)
            .count()
    }

    /// A tira rola por um tique, se a mão de um arrasto de quadro está na borda dela.
    pub(crate) fn scroll_at_edge(&mut self, strip: Rect, step: f32, margin: f32) {
        if let Some(d) = self.frame_drag() {
            let x = d.at.0;
            if x > strip.x + strip.w - margin {
                self.strip_scroll += step;
            } else if x < strip.x + margin {
                self.strip_scroll -= step;
            }
        }
    }

    /// A faixa rolou por baixo da mão: o intervalo do arrasto é o que ela aponta agora.
    pub(crate) fn drag_points_at(&mut self, frames: &[(Rect, Rect)]) {
        if let Drag::Frame {
            moved: true,
            gap,
            at,
            ..
        } = &mut self.drag
        {
            *gap = frames
                .iter()
                .filter(|f| f.0.x + f.0.w / 2.0 <= at.0)
                .count();
        }
    }

    /// O arrasto de quadro em curso, se já passou o limiar.
    #[must_use]
    pub fn frame_drag(&self) -> Option<FrameDrag> {
        match self.drag {
            Drag::Frame {
                from,
                x0,
                y0,
                moved: true,
                gap,
                at,
            } => Some(FrameDrag {
                from,
                gap,
                grab: (x0, y0),
                at,
            }),
            _ => None,
        }
    }
}

/// `base` com `added` metido no intervalo `at` (preso a `0..=base.len()`).
#[must_use]
pub fn spliced(base: &[AnimFrameRef], added: &[AnimFrameRef], at: usize) -> Vec<AnimFrameRef> {
    let at = at.min(base.len());
    let mut v = base[..at].to_vec();
    v.extend_from_slice(added);
    v.extend_from_slice(&base[at..]);
    v
}

impl AnimDoc {
    /// Onde a marca de inserção está pintada: com células a entrar, logo depois delas; senão a escolhida, ou o fim.
    #[must_use]
    pub fn playhead_gap(&self) -> usize {
        match &self.drag {
            Drag::Cells { base, added, at } => (*at).min(base.len()) + added.len(),
            // Presa aos quadros que a tira mostra (smoke do dono 09/11: presa a outra conta, ficava no começo).
            _ => {
                let n = self.shown_frames().len();
                self.insert_at.unwrap_or(n).min(n)
            }
        }
    }

    /// O quadro `gone` saiu: a marca que estava depois dele recua um.
    pub(crate) fn playhead_follow_delete(&mut self, gone: usize) {
        if let Some(p) = self.insert_at.as_mut()
            && *p > gone
        {
            *p -= 1;
        }
    }

    /// A marca de inserção na tira, como a disposição a pôs (vazia sem quadros).
    #[must_use]
    pub fn playhead_rect(&self) -> Rect {
        self.layout.playhead
    }
}
