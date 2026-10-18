//! ⭐ **Enquadrar o que se VÊ** (MiroClone W23 A1). O «enquadrar» mede-se na parte do quadro que os
//! painéis fixos não tapam — a barra das ferramentas, as colunas da direita, os botões do canto, o
//! HUD —, e cada um regista o rectângulo que PINTA ([`cover`]): medir e pintar leem a mesma fonte.
//! E o enquadramento de uma cena que abre REFAZ-SE enquanto ninguém mexe na vista: a janela nasce
//! 1024×768 e cresce depois da 1.ª pintura, e os painéis só se conhecem depois dela (fotos de 18/10,
//! `docs/MiroClone/handoffs/HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md` A1).

use super::super::HeroScreen;
use super::super::board_view::area_of;
use crate::documents::nav::FitHold;
use crate::zones::Rect;
use ph2d_board_model::Camera;

/// A área do quadro e a parte dela que nada tapa (ecrã, `[x, y, w, h]`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct View {
    pub(crate) area: [f64; 4],
    pub(crate) free: [f64; 4],
}

impl View {
    /// Enquadra `bbox` (mundo) na parte livre.
    pub(crate) fn fit(self, cam: &mut Camera, bbox: [f64; 4]) {
        cam.fit_in(self.area, self.free, bbox, super::fit_margin());
    }
}

/// Um painel fixo do quadro pintou `r` por cima dele.
pub(crate) fn cover(hero: &mut HeroScreen, r: Rect) {
    if r.w > 0.0 && r.h > 0.0 {
        hero.documents.live.nav.covers.push(r);
    }
}

/// A vista em que um clique ou uma tecla enquadram: a da última pintura.
pub(crate) fn view(hero: &HeroScreen) -> View {
    let area = hero.last_canvas;
    let free = hero.documents.live.nav.free.unwrap_or(area);
    View {
        area: area_of(area),
        free: area_of(free),
    }
}

/// ⭐ O rectângulo de `area` que fica livre: cada coberta (a maior primeiro) corta-o pelo lado que
/// guarda mais área.
#[must_use]
pub(crate) fn free_of(area: Rect, covers: &[Rect]) -> Rect {
    let size = |r: &Rect| r.w.max(0.0) * r.h.max(0.0);
    let mut covers = covers.to_vec();
    covers.sort_by(|a, b| size(b).total_cmp(&size(a)));
    let mut r = area;
    for c in covers {
        let (x1, y1) = (r.x + r.w, r.y + r.h);
        if c.x + c.w <= r.x || c.x >= x1 || c.y + c.h <= r.y || c.y >= y1 {
            continue;
        }
        let cuts = [
            Rect::new(c.x + c.w, r.y, x1 - c.x - c.w, r.h),
            Rect::new(r.x, r.y, c.x - r.x, r.h),
            Rect::new(r.x, c.y + c.h, r.w, y1 - c.y - c.h),
            Rect::new(r.x, r.y, r.w, c.y - r.y),
        ];
        if let Some(best) = cuts.into_iter().max_by(|a, b| size(a).total_cmp(&size(b))) {
            r = best;
        }
    }
    r
}

/// ⭐ Antes de desenhar o quadro: a parte livre desta pintura (das cobertas da anterior) e o
/// enquadramento que espera por ela.
pub(crate) fn settle(hero: &mut HeroScreen, area: Rect) {
    let nav = &mut hero.documents.live.nav;
    let covers = std::mem::take(&mut nav.covers);
    let free = free_of(area, &covers);
    nav.free = Some(free);
    let target = nav.fit_box.take();
    let pending = std::mem::take(&mut nav.fit_pending);
    let held = nav.fit_hold.take();
    let Some(board) = hero.documents.active_board_mut() else {
        return;
    };
    let mut hold = match held {
        _ if pending => FitHold {
            board: board.id,
            target,
            at: None,
            camera: board.camera,
        },
        Some(h) if h.board == board.id && h.camera == board.camera => h,
        _ => return,
    };
    let view = View {
        area: area_of(area),
        free: area_of(free),
    };
    if hold.at != Some((view.area, view.free)) {
        if let Some(bx) = hold.target.or_else(|| board.doc.extent()) {
            view.fit(&mut board.camera, bx);
        }
        hold.at = Some((view.area, view.free));
        hold.camera = board.camera;
    }
    hero.documents.live.nav.fit_hold = Some(hold);
}

#[cfg(test)]
#[path = "board_nav_fit_tests.rs"]
mod tests;
