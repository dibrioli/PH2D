//! **As etiquetas de uma nota** (W5): uma fita por etiqueta no canto de baixo da nota, da cor dela e
//! com o nome na tinta que se lê nela — o desenho das «tags» do Miro nas notas (o help não o
//! descreve; ⚠️ por medir numa captura do dono). Só quando a letra da nota se lê: de longe a nota é
//! um bloco ([`crate::Detail::note_blocks`]).

use ph2d_board_model::Element;
use ph2d_text::TextSystem;
use ph2d_vector::{Affine, Brush, Rect, RoundedRect, Shape as _, VectorScene};

use crate::{TextCache, doc_color};

/// A letra de uma fita em fracção da letra da nota, e o recuo dela em fracção da sua letra.
const TAG_FONT: f64 = 0.6;
const TAG_PAD: f64 = 0.35;
/// O vão entre fitas e até à borda da nota, em fracção do lado da nota.
const TAG_GAP: f64 = 0.03;
/// O salto da chave do texto moldado de uma fita (a do texto da nota é o id do elemento).
const OWNER_SALT: u64 = 0x7a65_0000_0000_0000;

/// Pinta as fitas de `el` (uma nota) com `t` = mundo-do-elemento → ecrã.
pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    text: &mut TextCache,
    board: u64,
    el: &Element,
    note_font: f64,
    t: Affine,
) {
    let font = note_font * TAG_FONT;
    let pad = font * TAG_PAD;
    let gap = el.w.min(el.h) * TAG_GAP;
    let mut x = gap;
    for (i, tag) in el.tags().iter().enumerate() {
        let owner = (board ^ OWNER_SALT, (el.id.0 << 4) | i as u64);
        let rich = ph2d_board_model::RichText::plain(&tag.name);
        let layout = text.get_rich(ts, owner, &rich, font as f32, f32::MAX, false);
        let (w, h) = (
            f64::from(layout.width()) + 2.0 * pad,
            f64::from(layout.height()) + pad,
        );
        if x + w > el.w - gap {
            break;
        }
        let y = el.h - gap - h;
        let chip = RoundedRect::from_rect(Rect::new(x, y, x + w, y + h), h / 2.0);
        scene.fill_path(&chip.to_path(0.1), &Brush::Solid(doc_color(tag.color)), t);
        let ink = doc_color(tag.color.readable_ink());
        ph2d_board_layout::paint(
            scene,
            layout,
            &tag.name,
            t * Affine::translate((x + pad, y + pad / 2.0)),
            ink,
        );
        x += w + gap;
    }
}
