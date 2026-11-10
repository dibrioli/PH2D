//! ⭐ **Onde mora um cartão que flutua e se arrasta** (a Timeline nos cartões, dono 10/11).
//!
//! O lugar é um DESVIO sobre o encaixe de origem que o layout calcula: o `blender_picker_offset`
//! (posição) e o `panel_resize_delta` (tamanho) — os mesmos campos e o mesmo arrasto do título
//! (`BlenderHitKind::DragHandle`) dos painéis flutuantes de sempre (Assets, Authored). Sem desvio, o
//! cartão está na origem e o rect é o do layout, byte a byte. «Encaixar» apaga o desvio.
//!
//! O desvio grava-se em `~/.ph2d/sections.txt` pela porta de
//! [`super::section_prefs_ops::section_prefs_text`]; o formato é do editor, a shell só sabe do disco.

use super::WidgetStore;
use crate::interaction::{TIMELINE_EDGE_B, TIMELINE_EDGE_L, TIMELINE_EDGE_R, TIMELINE_EDGE_T};
use crate::zones::Rect;
use ph2d_a11y::NodeId;

/// Os cartões cujo lugar se grava, e a chave de cada um no ficheiro.
const GRAVADOS: [(NodeId, &str); 1] = [(crate::ids::TIMELINE_PANEL, "timeline_card")];

/// O desvio de um cartão: posição `(dx, dy)` e tamanho `(dw, dh)` sobre a origem.
pub type Desvio = ((f32, f32), (f32, f32));

impl WidgetStore {
    /// O desvio de `panel` sobre a origem (zeros na origem).
    #[must_use]
    pub fn floating_card_offsets(&self, panel: NodeId) -> Desvio {
        (
            self.blender_picker_offset(panel),
            self.panel_resize_delta(panel),
        )
    }

    /// `panel` saiu do lugar de origem (foi arrastado ou redimensionado)?
    #[must_use]
    pub fn floating_card_away(&self, panel: NodeId) -> bool {
        let ((dx, dy), (dw, dh)) = self.floating_card_offsets(panel);
        [dx, dy, dw, dh].iter().any(|v| v.abs() > f32::EPSILON)
    }

    /// Escreve o desvio de `panel`.
    pub fn set_floating_card_offsets(&mut self, panel: NodeId, (off, d): Desvio) {
        self.set_blender_picker_offset(panel, off.0, off.1);
        self.set_panel_resize_delta(panel, d.0, d.1);
    }

    /// ⭐ **Encaixar no lugar de origem**: o desvio sai, e o rect volta a ser o do layout.
    pub fn dock_floating_card(&mut self, panel: NodeId) {
        self.blender_picker_offset.remove(&panel);
        self.panel_resize_delta.remove(&panel);
        self.mark_floating_card_settled();
    }

    /// Um gesto sobre o cartão acabou: o lugar novo é para gravar.
    pub fn mark_floating_card_settled(&mut self) {
        self.mark_section_prefs_dirty();
    }

    /// ⭐ **O rect do cartão neste quadro**: a origem mais o desvio, o tamanho entre `min` e a janela,
    /// e a posição dentro da janela; o desvio grava-se de volta já preso (o próximo arrasto parte do
    /// que se vê, sem elástico). Na origem devolve `origin` intacto.
    pub fn place_floating_card(
        &mut self,
        panel: NodeId,
        origin: Rect,
        viewport: Rect,
        min: (f32, f32),
    ) -> Rect {
        if !self.floating_card_away(panel) {
            return origin;
        }
        let ((dx, dy), (dw, dh)) = self.floating_card_offsets(panel);
        let w = (origin.w + dw).min(viewport.w).max(min.0);
        let h = (origin.h + dh).min(viewport.h).max(min.1);
        let x = (origin.x + dx).clamp(viewport.x, (viewport.x + viewport.w - w).max(viewport.x)); // CLAMP-OK: viewport bounds, min<=max
        let y = (origin.y + dy).clamp(viewport.y, (viewport.y + viewport.h - h).max(viewport.y)); // CLAMP-OK: viewport bounds, min<=max
        let r = Rect::new(x, y, w, h);
        self.set_floating_card_offsets(
            panel,
            ((x - origin.x, y - origin.y), (w - origin.w, h - origin.h)),
        );
        r
    }

    /// ⭐ **Redimensionar por uma borda ou canto** (`edges`, os bits `TIMELINE_EDGE_*`): a borda
    /// agarrada segue o ponteiro (`dx, dy` desde o Begin, sobre o desvio `base` capturado nele) e a
    /// oposta fica parada; o tamanho nunca desce de `min` (`size0` = o tamanho visível no Begin).
    pub fn resize_floating_card(
        &mut self,
        panel: NodeId,
        base: Desvio,
        size0: (f32, f32),
        edges: u8,
        (dx, dy): (f32, f32),
        min: (f32, f32),
    ) {
        let ((mut ox, mut oy), (mut dw, mut dh)) = base;
        // Quanto a borda de cima/esquerda pode entrar antes do piso.
        let (folga_w, folga_h) = ((size0.0 - min.0).max(0.0), (size0.1 - min.1).max(0.0));
        if edges & TIMELINE_EDGE_L != 0 {
            let d = dx.min(folga_w);
            (ox, dw) = (ox + d, dw - d);
        } else if edges & TIMELINE_EDGE_R != 0 {
            dw += dx.max(-folga_w);
        }
        if edges & TIMELINE_EDGE_T != 0 {
            let d = dy.min(folga_h);
            (oy, dh) = (oy + d, dh - d);
        } else if edges & TIMELINE_EDGE_B != 0 {
            dh += dy.max(-folga_h);
        }
        self.set_floating_card_offsets(panel, ((ox, oy), (dw, dh)));
    }

    /// Os rects flutuantes que podem ASSENTAR numa coluna (as listas de cartões, D24) — sem os cartões
    /// que se arrastam livres: um cartão largado sobre a coluna não é a lista dela (senão o encaixe
    /// de origem andaria atrás dele).
    pub fn column_floating_rects(&self) -> impl Iterator<Item = Rect> + '_ {
        self.panel_rects
            .iter()
            .filter(|(id, _)| {
                self.floating_panels.contains(id) && !GRAVADOS.iter().any(|(p, _)| p == *id)
            })
            .map(|(_, r)| *r)
    }

    /// O título arrastado acabou: um cartão gravado marca o lugar novo.
    pub(super) fn floating_card_drag_ended(&mut self, panel: NodeId) {
        if GRAVADOS.iter().any(|(p, _)| *p == panel) {
            self.mark_section_prefs_dirty();
        }
    }
}

/// Escreve o desvio dos cartões gravados que saíram da origem (`timeline_card=dx,dy,dw,dh`).
pub(super) fn escreve(store: &WidgetStore, s: &mut String) {
    use std::fmt::Write as _;
    for (panel, chave) in GRAVADOS {
        if store.floating_card_away(panel) {
            let ((dx, dy), (dw, dh)) = store.floating_card_offsets(panel);
            let _ = writeln!(s, "{chave}={dx},{dy},{dw},{dh}");
        }
    }
}

/// Lê uma linha `k=v`; `true` se era de um cartão gravado (mesmo ilegível).
pub(super) fn le(store: &mut WidgetStore, k: &str, v: &str) -> bool {
    let Some((panel, _)) = GRAVADOS.iter().find(|(_, c)| *c == k) else {
        return false;
    };
    let n: Vec<f32> = v
        .split(',')
        .filter_map(|t| t.trim().parse::<f32>().ok())
        .filter(|x| x.is_finite())
        .collect();
    if let [dx, dy, dw, dh] = n[..] {
        store.set_floating_card_offsets(*panel, ((dx, dy), (dw, dh)));
    }
    true
}

#[cfg(test)]
#[path = "floating_card_ops_tests.rs"]
mod tests;
