//! A IMAGEM de um ficheiro do Kanban no verso — a capa (inteira, «contain», como o modal do Trello)
//! e a miniatura de um anexo (recortada, «cover»). A descodificação é a da frente
//! ([`crate::front::cover::get`]: fora do fio do desenho, em cache por conteúdo); até chegar, nada.

use crate::front::cover::ThumbRef;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{FileId, KanbanSet};
use ph2d_vector::{ImageQuality, RoundedRect, VectorScene};

/// O destino de uma imagem de `size` em `r`, centrada: `cover` enche (recorta), senão cabe inteira.
fn fit(size: [u32; 2], r: Rect, cover: bool) -> Rect {
    #[allow(clippy::cast_precision_loss)]
    let (w, h) = (size[0].max(1) as f32, size[1].max(1) as f32);
    let s = if cover {
        (r.w / w).max(r.h / h)
    } else {
        (r.w / w).min(r.h / h)
    };
    let (dw, dh) = (w * s, h * s);
    Rect::new(r.x + (r.w - dw) / 2.0, r.y + (r.h - dh) / 2.0, dw, dh)
}

/// Pinta o ficheiro `f` em `r` (cantos `radii`: sup. esq., sup. dir., inf. dir., inf. esq.).
/// `false` = não é imagem, ou ainda não está descodificada.
pub(crate) fn draw(
    scene: &mut VectorScene,
    set: &KanbanSet,
    f: FileId,
    r: Rect,
    radii: (f32, f32, f32, f32),
    cover: bool,
) -> bool {
    let Some(ThumbRef(t)) = crate::front::cover::get(f, || set.file(f).map(|k| k.bytes.0.clone()))
    else {
        return false;
    };
    let d = fit(t.size, r, cover);
    let (a, b, c, e) = radii;
    scene.push_clip(&RoundedRect::new(
        f64::from(r.x),
        f64::from(r.y),
        f64::from(r.x + r.w),
        f64::from(r.y + r.h),
        (f64::from(a), f64::from(b), f64::from(c), f64::from(e)),
    ));
    let dest = (
        f64::from(d.x),
        f64::from(d.y),
        f64::from(d.x + d.w),
        f64::from(d.y + d.h),
    );
    scene.draw_stable_image(&t.image, dest, ImageQuality::Medium);
    scene.pop_layer();
    true
}

#[cfg(test)]
mod tests {
    use super::{Rect, fit};

    #[test]
    fn contain_fits_inside_and_cover_fills() {
        let r = Rect::new(0.0, 0.0, 200.0, 100.0);
        let c = fit([100, 100], r, false);
        assert_eq!((c.x, c.y, c.w, c.h), (50.0, 0.0, 100.0, 100.0));
        let f = fit([100, 100], r, true);
        assert_eq!((f.x, f.y, f.w, f.h), (0.0, -50.0, 200.0, 200.0));
    }
}
