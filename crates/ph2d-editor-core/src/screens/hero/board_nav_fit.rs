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

/// A área do quadro e o que os painéis fixos pintaram por cima dela na última pintura (ecrã).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct View {
    pub(crate) area: Rect,
    pub(crate) covers: Vec<Rect>,
}

impl View {
    /// Enquadra `bbox` (mundo) na parte livre que o deixa MAIOR.
    pub(crate) fn fit(&self, cam: &mut Camera, bbox: [f64; 4]) {
        let m = super::fit_margin();
        let free = free_of(self.area, &self.covers, bbox, m);
        cam.fit_in(area_of(self.area), area_of(free), bbox, m);
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
    View {
        area: hero.last_canvas,
        covers: hero.documents.live.nav.shown.clone(),
    }
}

/// ⭐ O rectângulo de `area` que fica livre para enquadrar `bbox` (mundo) com `margin` px: cada
/// coberta corta-o por um dos lados, e escolhe-se a combinação de lados em que `bbox` cabe MAIOR
/// depois de TODAS — numa janela estreita uma cena larga vai para baixo do cartão «Live», uma alta
/// para o lado dele, e um lado que os botões do canto comem perde para o outro.
#[must_use]
pub(crate) fn free_of(area: Rect, covers: &[Rect], bbox: [f64; 4], margin: f64) -> Rect {
    let size = |r: &Rect| r.w.max(0.0) * r.h.max(0.0);
    let (bw, bh) = ((bbox[2] - bbox[0]).max(1e-9), (bbox[3] - bbox[1]).max(1e-9));
    let zoom = |r: &Rect| {
        let w = (f64::from(r.w) - 2.0 * margin) / bw;
        let h = (f64::from(r.h) - 2.0 * margin) / bh;
        w.min(h)
    };
    let mut covers = covers.to_vec();
    covers.sort_by(|a, b| size(b).total_cmp(&size(a)));
    let mut best = (f64::NEG_INFINITY, area);
    search(area, &covers, &zoom, &mut best);
    best.1
}

/// Desce pelas cobertas, um lado de cada vez; um ramo cujo rectângulo já não bate o melhor sai cedo
/// (cortar nunca o deixa maior).
fn search(r: Rect, covers: &[Rect], zoom: &impl Fn(&Rect) -> f64, best: &mut (f64, Rect)) {
    if zoom(&r) <= best.0 {
        return;
    }
    let Some((c, rest)) = covers.split_first() else {
        if zoom(&r) > best.0 {
            *best = (zoom(&r), r);
        }
        return;
    };
    let (x1, y1) = (r.x + r.w, r.y + r.h);
    if c.x + c.w <= r.x || c.x >= x1 || c.y + c.h <= r.y || c.y >= y1 {
        search(r, rest, zoom, best);
        return;
    }
    let mut cuts: Vec<Rect> = [
        Rect::new(c.x + c.w, r.y, x1 - c.x - c.w, r.h),
        Rect::new(r.x, r.y, c.x - r.x, r.h),
        Rect::new(r.x, c.y + c.h, r.w, y1 - c.y - c.h),
        Rect::new(r.x, r.y, r.w, c.y - r.y),
    ]
    .into_iter()
    .filter(|k| k.w > 0.0 && k.h > 0.0)
    .collect();
    if cuts.is_empty() {
        search(r, rest, zoom, best);
        return;
    }
    cuts.sort_by(|a, b| zoom(b).total_cmp(&zoom(a)));
    for k in cuts {
        search(k, rest, zoom, best);
    }
}

/// ⭐ Antes de desenhar o quadro: as cobertas da pintura anterior viram a vista desta, e o
/// enquadramento que espera pela área refaz-se se ela mudou.
pub(crate) fn settle(hero: &mut HeroScreen, area: Rect) {
    let nav = &mut hero.documents.live.nav;
    nav.shown = std::mem::take(&mut nav.covers);
    let target = nav.fit_box.take();
    let pending = std::mem::take(&mut nav.fit_pending);
    let held = nav.fit_hold.take();
    let view = View {
        area,
        covers: nav.shown.clone(),
    };
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
    if let Some(bx) = hold.target.or_else(|| board.doc.extent()) {
        let m = super::fit_margin();
        let at = (area_of(area), area_of(free_of(area, &view.covers, bx, m)));
        if hold.at != Some(at) {
            view.fit(&mut board.camera, bx);
            hold.at = Some(at);
            hold.camera = board.camera;
        }
    }
    hero.documents.live.nav.fit_hold = Some(hold);
}

#[cfg(test)]
#[path = "board_nav_fit_tests.rs"]
mod tests;
