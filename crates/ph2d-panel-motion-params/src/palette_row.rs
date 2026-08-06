//! The **Palette** editor row — an ordered list of colours, with no length limit.
//!
//! The colour sibling of [`crate::gradient_row`], and deliberately SMALLER than it: a
//! palette has no positions and no interpolation, so it has no bar, no draggable markers
//! and no interp button. What is left is the part that matters — a strip of **OKLCH
//! swatches** (`register_picker_swatch`, the canonical colour UI, the mirror of
//! [`crate::snapshot::ColorRow`]) plus `+` / `−`.
//!
//! ⚠️ **The strip WRAPS, and that is what makes "no limit" true on screen** (Enio: *"color
//! array poderia ter quantas cores o usuário quisesse, tire os limites"*). A fixed row of
//! swatches would have re-imposed a cap the moment the eleventh colour ran off the edge —
//! so the row's HEIGHT is a function of how many colours there are, which is why
//! [`paint_palette_row`] returns the height it used instead of a constant.
//!
//! The artist never sees the string ([`ph2d_color::palette_text`]).

use crate::snapshot::{PaletteRow, param_pal_add_id, param_pal_remove_id, param_pal_swatch_id};
use ph2d_color::srgb::linear_to_srgb_byte;
use ph2d_color::{DEFAULT_PALETTE_FALLBACK, parse_palette};
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, paint_text_centered, resolve};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const SWATCH: f32 = 22.0; // LITERAL-PX-OK: one palette swatch, square
const BTN_W: f32 = 22.0; // LITERAL-PX-OK: +/- button width

/// The colours a row shows: the authored palette, or the node's factory one when the
/// artist has not authored yet.
///
/// ⚠️ **The same fallback the NODE uses**, and that is the point — a swatch strip that
/// disagreed with the tint on screen would be a second answer to *what colours is this
/// node cycling?*. A malformed string falls back too, because `parse_palette` refuses
/// rather than silently shortening.
fn working(value: &str) -> Vec<[f32; 4]> {
    parse_palette(value)
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| DEFAULT_PALETTE_FALLBACK.to_vec())
}

fn swatch_srgb(c: [f32; 4]) -> [u8; 4] {
    [
        linear_to_srgb_byte(c[0]),
        linear_to_srgb_byte(c[1]),
        linear_to_srgb_byte(c[2]),
        255,
    ]
}

/// How many swatches fit on one line of width `w`. At least one — a row narrower than a
/// single swatch still has to draw something, and one-per-line is the honest degenerate.
pub(crate) fn per_line(w: f32) -> usize {
    let gap = Spacing::Xs.px();
    (((w + gap) / (SWATCH + gap)) as usize).max(1)
}

/// Paint the row and collect its store registrations. Returns the height used.
///
/// The arg list mirrors [`crate::gradient_row::paint_gradient_row`] one-for-one: the two
/// are called from the same dispatch arm shape, and a different signature here would be a
/// second convention for the same job.
#[expect(clippy::too_many_arguments, reason = "mirrors the gradient row's paint door")]
pub(crate) fn paint_palette_row(
    row: &PaletteRow,
    slot: usize,
    x: f32,
    w: f32,
    y: f32,
    label_font: f32,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    out: &mut crate::gradient_row::ColourRowWidgets,
) -> f32 {
    let gap = Spacing::Xs.px();
    let colors = working(&row.value);

    // ── Header: label (left) + + / − (right) ──
    paint_text(
        text_system,
        scene,
        &row.label,
        x,
        y + (ROW_H_PX - label_font) * 0.5,
        label_font,
        w - BTN_W * 2.0 - gap * 2.0, // LITERAL-PX-OK: CONTAGEM (2 vaos), nao medida
        resolve(ColorToken::Text2, theme),
    );
    let rem = Rect::new(x + w - BTN_W, y, BTN_W, ROW_H_PX);
    let add = Rect::new(rem.x - BTN_W - gap, y, BTN_W, ROW_H_PX);
    for (brect, label, id) in [
        (add, "+", param_pal_add_id(slot)),
        (rem, "\u{2212}", param_pal_remove_id(slot)),
    ] {
        fill_rounded_rect(
            scene,
            brect,
            Radius::Sm.px(),
            resolve(ColorToken::Bg2, theme),
        );
        paint_text_centered(
            text_system,
            scene,
            label,
            brect,
            TypeToken::Base.px(),
            resolve(ColorToken::Text1, theme),
        );
        hit_index.register(id, brect);
        out.buttons.push(id);
    }

    // ── The strip, WRAPPED. The height follows the count; nothing here caps it. ──
    let cols = per_line(w);
    let mut used = ROW_H_PX + gap;
    for (i, c) in colors.iter().enumerate() {
        let (line, col) = (i / cols, i % cols);
        #[expect(
            clippy::cast_precision_loss,
            reason = "swatch grid indices; a palette long enough to lose precision here \
                      would not fit on any screen"
        )]
        let r = Rect::new(
            x + col as f32 * (SWATCH + gap),
            y + used + line as f32 * (SWATCH + gap),
            SWATCH,
            SWATCH,
        );
        let id = param_pal_swatch_id(row.name, i);
        hit_index.register(id, r);
        out.swatches.push((id, swatch_srgb(*c)));
    }
    let lines = colors.len().div_ceil(cols);
    #[expect(
        clippy::cast_precision_loss,
        reason = "a line count; see the swatch-grid note above"
    )]
    {
        used += lines as f32 * (SWATCH + gap);
    }
    used
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The strip wraps, so the row has no length limit of its own.** A fixed row would
    /// have re-imposed the cap this wave removed the moment a colour ran off the edge.
    #[test]
    fn the_strip_wraps_instead_of_running_off_the_edge() {
        let narrow = per_line(60.0);
        assert!(
            (2..20).contains(&narrow),
            "a narrow row fits few: {narrow}"
        );
        assert!(per_line(600.0) > narrow, "a wide row fits more");
        assert_eq!(per_line(1.0), 1, "narrower than one swatch still draws one");
    }

    /// **A row with more colours is TALLER.** This is the executable form of "no limit":
    /// the answer to *where does the eleventh swatch go?* is "the next line", not
    /// "nowhere".
    #[test]
    fn the_row_grows_with_the_palette() {
        let h = |n: usize| {
            let pal: Vec<[f32; 4]> = (0..n).map(|_| [1.0, 0.0, 0.0, 1.0]).collect();
            let row = PaletteRow {
                name: "palette",
                label: "Palette".into(),
                value: ph2d_color::serialize_palette(&pal),
            };
            let mut hit = HitIndex::default();
            let mut scene = VectorScene::new();
            let mut text = TextSystem::without_system_fonts();
            let mut out = crate::gradient_row::ColourRowWidgets::new();
            paint_palette_row(
                &row,
                0,
                0.0,
                120.0,
                0.0,
                12.0,
                &mut hit,
                &mut scene,
                &mut text,
                Theme::default(),
                &mut out,
            )
        };
        let (four, forty) = (h(4), h(40));
        assert!(
            forty > four * 3.0,
            "forty colours must be much taller than four: {four} vs {forty}"
        );
    }

    /// The strip shows what the NODE cycles: an unauthored palette falls back to the same
    /// factory list the node uses, so the swatches never describe colours nobody paints.
    #[test]
    fn an_unauthored_row_shows_the_nodes_own_default() {
        assert_eq!(working(""), DEFAULT_PALETTE_FALLBACK.to_vec());
        assert_eq!(working("p1"), DEFAULT_PALETTE_FALLBACK.to_vec());
        let one = vec![[0.25, 0.5, 0.75, 1.0]];
        assert_eq!(working(&ph2d_color::serialize_palette(&one)), one);
    }

    /// **O `+` ACRESCENTA uma cor, sem teto — e o `−` para em UMA.**
    ///
    /// O gate do cook (na shell) prova que qualquer comprimento cicla; este prova que o
    /// artista CHEGA lá, que é a metade que falta quando um modelo sem limite fica atrás
    /// de uma UI que não o alcança. O piso de um existe porque uma paleta vazia deixaria o
    /// nó sem nada para ciclar e a faixa sem nada em que clicar de volta.
    #[test]
    fn the_buttons_grow_the_palette_without_a_cap_and_stop_at_one() {
        use crate::snapshot::{ParamsSnapshot, param_pal_add_id, param_pal_remove_id};

        let mut value = String::new(); // não-autorada: as quatro de fábrica
        let click = |value: &str, id| {
            let snap = ParamsSnapshot {
                node: 1,
                title: "motion.color_array".into(),
                rows: vec![crate::snapshot::ParamRow::Palette(PaletteRow {
                    name: "palette",
                    label: "Palette".into(),
                    value: value.to_string(),
                })],
            };
            let _ = crate::events::on_click(id, &snap);
            crate::drain_param_intents()
                .into_iter()
                .find_map(|i| match i {
                    crate::MotionParamIntent::SetTextParam { value, .. } => Some(value),
                    _ => None,
                })
        };
        // Nove cliques no `+` a partir de quatro ⇒ TREZE, muito além do cap de quatro.
        for _ in 0..9 {
            value = click(&value, param_pal_add_id(0)).expect("the + emits a palette");
        }
        assert_eq!(
            parse_palette(&value).expect("well formed").len(),
            13,
            "nine clicks on `+` reach thirteen — the old cap was four"
        );
        // E o `−` desce até UMA e para.
        for _ in 0..20 {
            value = click(&value, param_pal_remove_id(0)).expect("the - emits a palette");
        }
        assert_eq!(
            parse_palette(&value).expect("well formed").len(),
            1,
            "the floor is one colour, never zero"
        );
    }
}
