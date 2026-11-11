//! B40.3: o recorte do rascunho da máscara que o re-carimbo de uma figura repõe.

use super::super::{PaintMode, PainterTool, Region};
use std::sync::Arc;

impl PainterTool {
    /// B40.3: the scratch under `rect` while a Mask-mode shape preview owns it (`None` otherwise).
    pub(in crate::tool::paint) fn save_mask_region(&self, rect: &Region) -> Option<Vec<u8>> {
        if !matches!(self.paint.paint_mode, PaintMode::Mask) || !self.mask_scratch_active() {
            return None;
        }
        let stride = self.source_size.0 as usize * 4;
        let rw = rect.w as usize * 4;
        let s = &self.paint.mask_scratch_rgba;
        Some(
            (0..rect.h as usize)
                .flat_map(|r| {
                    let a = (rect.y as usize + r) * stride + rect.x as usize * 4;
                    s[a..a + rw].iter().copied()
                })
                .collect(),
        )
    }

    /// Put back what [`Self::save_mask_region`] took (the shape's previous frame leaves no trail).
    pub(in crate::tool::paint) fn restore_mask_region(
        &mut self,
        rect: &Region,
        saved: Option<&[u8]>,
    ) {
        let Some(px) = saved else { return };
        if !self.mask_scratch_active() {
            return;
        }
        let stride = self.source_size.0 as usize * 4;
        let rw = rect.w as usize * 4;
        let s = Arc::make_mut(&mut self.paint.mask_scratch_rgba);
        for r in 0..rect.h as usize {
            let a = (rect.y as usize + r) * stride + rect.x as usize * 4;
            s[a..a + rw].copy_from_slice(&px[r * rw..(r + 1) * rw]);
        }
        self.bump_mask_scratch_gen();
        self.mark_dirty(*rect);
    }
}

#[cfg(test)]
thread_local! {
    /// Oráculo da prancha B40b: a figura no modo Mask pelo caminho de antes (sem o rascunho).
    pub(in crate::tool::paint) static FORMA_NO_MASK_ANTIGA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// `true` só sob o oráculo de teste; no produto é sempre `false`.
pub(in crate::tool::paint) fn forma_no_mask_antiga() -> bool {
    #[cfg(test)]
    return FORMA_NO_MASK_ANTIGA.get();
    #[cfg(not(test))]
    false
}
