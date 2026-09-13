//! The **Palette** editor row — an ordered list of colours, with no length limit.
//!
//! The colour sibling of [`crate::gradient`], and deliberately SMALLER than it: a
//! palette has no positions and no interpolation, so it has no bar, no draggable markers
//! and no interp button. What is left is the part that matters — a strip of **OKLCH
//! swatches** (`register_picker_swatch`, the canonical colour UI, the mirror of
//! a amostra de uma cor simples) plus `+` / `−`.
//!
//! ⚠️ **The strip WRAPS, and that is what makes "no limit" true on screen** (Enio: *"color
//! array poderia ter quantas cores o usuário quisesse, tire os limites"*). A fixed row of
//! swatches would have re-imposed a cap the moment the eleventh colour ran off the edge —
//! so the row's HEIGHT is a function of how many colours there are, which is why
//! [`paint_palette_row`] returns the height it used instead of a constant.
//!
//! The artist never sees the string ([`ph2d_color::palette_text`]).
//!
//! ⚠️ **A swatch é DESENHADA aqui e registrada para o picker lá** (o hospedeiro) — as
//! duas metades são perguntas diferentes (*o que aparece* × *o que o clique faz*), e a primeira
//! versão desta row tinha só a segunda: hit-rect registrado, `set_widget_color` semeado, e
//! nenhuma tinta. A faixa saía VAZIA com os dois gates da row verdes, porque eles mediam a
//! ALTURA devolvida e a aritmética de [`per_line`] — vide `every_colour_paints_a_swatch_in_its_own_colour`.

use crate::EditorKey;
use ph2d_color::srgb::linear_to_srgb_byte;
use ph2d_color::{DEFAULT_PALETTE_FALLBACK, parse_palette};
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::paint_text_elided;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_centered, resolve};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Color, VectorScene};

const SWATCH: f32 = 22.0; // LITERAL-PX-OK: one palette swatch, square
const BTN_W: f32 = 22.0; // LITERAL-PX-OK: +/- button width
const GRID_W: f32 = 1.0; // LITERAL-PX-OK: swatch border stroke width

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
pub fn per_line(w: f32, scale: f32) -> usize {
    let gap = Spacing::Xs.px() * scale;
    (((w + gap) / (SWATCH * scale + gap)) as usize).max(1)
}

/// Paint the row and collect its store registrations. Returns the height used.
///
/// The arg list mirrors [`crate::gradient::paint`] one-for-one: the two
/// are called from the same dispatch arm shape, and a different signature here would be a
/// second convention for the same job.
#[expect(
    clippy::too_many_arguments,
    reason = "mirrors the gradient row's paint door"
)]
pub fn paint(
    label: &str,
    value: &str,
    key: EditorKey<'_>,
    x: f32,
    w: f32,
    y: f32,
    label_font: f32,
    // `scale`: quanto a superfície do hospedeiro tem de folga — ver o doc de [`height`].
    scale: f32,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    out: &mut crate::gradient::ColourRowWidgets,
) -> f32 {
    let gap = Spacing::Xs.px() * scale;
    // ⚠️ **Uma multiplicação, um sítio.**
    let (swatch, btn_w, row_h) = (SWATCH * scale, BTN_W * scale, ROW_H_PX * scale);
    let colors = working(value);

    // ── Header: label (left) + + / − (right) ──
    paint_text_elided(
        text_system,
        scene,
        label,
        x,
        y + (row_h - label_font) * 0.5,
        label_font,
        w - btn_w * 2.0 - gap * 2.0, // LITERAL-PX-OK: CONTAGEM (2 vaos), nao medida
        resolve(ColorToken::Text2, theme),
    );
    let rem = Rect::new(x + w - btn_w, y, btn_w, row_h);
    let add = Rect::new(rem.x - btn_w - gap, y, btn_w, row_h);
    for (brect, label, id) in [
        (add, "+", key.sub("add")),
        (rem, "\u{2212}", key.sub("remove")),
    ] {
        fill_rounded_rect(
            scene,
            brect,
            ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
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
    let cols = per_line(w, scale);
    // O topo da tira: o cabeçalho já foi desenhado.
    let used = ph2d_tokens::row_pitch_px() * scale;
    for (i, c) in colors.iter().enumerate() {
        let (line, col) = (i / cols, i % cols);
        #[expect(
            clippy::cast_precision_loss,
            reason = "swatch grid indices; a palette long enough to lose precision here \
                      would not fit on any screen"
        )]
        let r = Rect::new(
            x + col as f32 * (swatch + gap),
            y + used + line as f32 * (swatch + gap),
            swatch,
            swatch,
        );
        let id = key.swatch_id(i);
        let srgb = swatch_srgb(*c);
        fill_rounded_rect(
            scene,
            r,
            ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
            Color::from_rgba8(srgb[0], srgb[1], srgb[2], 255), // LITERAL-COLOR-OK: the palette's own colour is data, not a token (the swatch precedent)
        );
        // ⭐ Pela porta do TEMA: a amostra é plana num tema moderno.
        ph2d_editor_core::paint::stroke_frame(
            scene,
            r,
            ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
            theme,
            ph2d_tokens::visuals::Feel::Rest,
            GRID_W,
            resolve(ColorToken::TextDisabled, theme),
        );
        hit_index.register(id, r);
        out.swatches.push((id, srgb));
    }
    // Pela PORTA — ver [`height`].
    debug_assert!(
        (used + strip_h(colors.len(), cols, scale) - height(value, w, scale)).abs() < 1e-3
    );
    height(value, w, scale)
}

/// **A ALTURA que esta paleta ocupa** — cabeçalho mais a tira, que EMBRULHA: ela segue a
/// contagem de cores, e nada aqui a limita.
///
/// ⚠️ **Uma PORTA, dois leitores**: o `paint` devolve-a e o hospedeiro flutuante lê-a antes de
/// desenhar (o fundo vem primeiro na cena). *Duas contas da mesma altura seriam um fundo que
/// não cobre o que está lá dentro* — e numa paleta, que cresce, seria pior: o fundo ficaria
/// certo até a nona cor.
#[must_use]
///
/// ⭐⭐ **A ESCALA é do HOSPEDEIRO** — ver [`crate::gradient::height`]. ⚠️ Aqui ela muda também
/// **quantas cabem por linha**, e é por isso que o `per_line` a recebe: uma tira que
/// embrulhasse por uma contagem e desenhasse por outra teria amostras fora da caixa.
pub fn height(value: &str, w: f32, scale: f32) -> f32 {
    let n = working(value).len();
    ph2d_tokens::row_pitch_px() * scale + strip_h(n, per_line(w, scale), scale)
}

/// A altura da tira embrulhada, em linhas.
fn strip_h(n: usize, cols: usize, scale: f32) -> f32 {
    let gap = Spacing::Xs.px() * scale;
    #[expect(
        clippy::cast_precision_loss,
        reason = "a line count; see the swatch-grid note above"
    )]
    {
        n.div_ceil(cols.max(1)) as f32 * (SWATCH * scale + gap)
    }
}

/// ⭐⭐ **O `+` ACRESCENTA UMA COR, SEM TETO** (Enio: *«tire os limites»*).
///
/// ⚠️ **A nova COPIA a última** — um artista acrescenta uma amostra para depois a EDITAR, e uma
/// cópia vê-se onde um buraco preto é uma falha.
#[must_use]
pub fn add_color(value: &str) -> String {
    let mut colors = working(value);
    let last = *colors.last().unwrap_or(&[1.0, 1.0, 1.0, 1.0]);
    colors.push(last);
    ph2d_color::serialize_palette(&colors)
}

/// ⭐ **O `−` tira a ÚLTIMA, e para em UMA.** Uma paleta vazia deixaria o nó sem nada para
/// ciclar e a tira sem nada em que clicar de volta.
#[must_use]
pub fn remove_color(value: &str) -> String {
    let mut colors = working(value);
    if colors.len() > 1 {
        colors.pop();
    }
    ph2d_color::serialize_palette(&colors)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A escala da row do painel — o que os gates de sempre medem.
    fn painted(colors: &[[f32; 4]], w: f32) -> (f32, u32, Vec<u32>) {
        painted_at(colors, w, 1.0)
    }

    /// ⭐⭐ **A PORTA DA ALTURA DIZ O QUE O PINTOR USOU** — a igualdade que o hospedeiro
    /// flutuante depende para desenhar o fundo ANTES do conteúdo.
    ///
    /// ⚠️ Numa paleta ela não é constante: a tira EMBRULHA, então uma segunda conta ficaria
    /// certa até a linha encher. FALSIFICADO por o `height` esquecer o cabeçalho.
    #[test]
    fn the_height_door_matches_what_the_paint_used() {
        for escala in [1.0_f32, 1.3] {
            for n in [1usize, 3, 9, 17] {
                let cores: Vec<[f32; 4]> =
                    (0..n).map(|i| [i as f32 / 20.0, 0.2, 0.3, 1.0]).collect();
                let w = 184.0 * escala;
                let (usada, _, _) = painted_at(&cores, w, escala);
                let porta = height(&ph2d_color::serialize_palette(&cores), w, escala);
                assert!(
                    (usada - porta).abs() < 1e-3,
                    "a {escala}x com {n} cores o pintor usou {usada} e a porta diz {porta}"
                );
            }
        }
    }

    /// What painting a palette of `colors` into a `w`-wide row actually produced: the height
    /// it claimed, **how many paths the scene really encodes**, and the paint bytes.
    ///
    /// ⚠️ The last two are the whole point. The row's first two gates measured the returned
    /// HEIGHT and the arithmetic of [`per_line`] — both green while `paint_palette_row`
    /// reserved space for the swatches and **drew nothing in it**, which is exactly what the
    /// artist saw ([[feedback_painted_is_not_populated_paint_gate]]). A number a painter
    /// returns is not evidence that a painter painted; `Scene::encoding()` is.
    fn painted_at(colors: &[[f32; 4]], w: f32, escala: f32) -> (f32, u32, Vec<u32>) {
        let valor = ph2d_color::serialize_palette(colors);
        let mut hit = HitIndex::default();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let mut out = crate::gradient::ColourRowWidgets::new();
        let h = paint(
            "Palette",
            &valor,
            EditorKey {
                own: "teste/pal/0",
                swatch: "teste/pal_swatch/palette",
            },
            0.0,
            w,
            0.0,
            12.0,
            escala,
            &mut hit,
            &mut scene,
            &mut text,
            Theme::default(),
            &mut out,
        );
        let e = scene.inner().encoding();
        (h, e.n_paths, e.draw_data.clone())
    }

    fn reds(n: usize) -> Vec<[f32; 4]> {
        (0..n).map(|_| [1.0, 0.0, 0.0, 1.0]).collect()
    }

    /// **Cada cor DESENHA uma swatch, e a swatch usa a cor dela.**
    ///
    /// Duas metades, porque há dois modos de falha distintos e cada um passa pelo gate do
    /// outro: *nada é desenhado* (a faixa fica vazia — o defeito que o Enio fotografou) e
    /// *algo é desenhado com a tinta errada* (uma fileira de caixas cinzas, que conta
    /// caminhos igualzinho a uma paleta correta).
    ///
    /// A 2ª metade compara os bytes de tinta de duas paletas do MESMO comprimento e cores
    /// diferentes, em vez de conferir um valor codificado: ela não precisa saber como o
    /// vello empacota um `Color`, e continua valendo se ele mudar.
    #[test]
    fn every_colour_paints_a_swatch_in_its_own_colour() {
        let (_, one, _) = painted(&reds(1), 400.0);
        let (_, twelve, _) = painted(&reds(12), 400.0);
        assert!(
            twelve >= one + 11,
            "cada cor a mais tem de desenhar ao menos um caminho a mais: {one} -> {twelve}"
        );

        let a = painted(&reds(4), 400.0).2;
        let b = painted(
            &[
                [0.0, 0.2, 0.9, 1.0],
                [0.9, 0.4, 0.0, 1.0],
                [0.1, 0.8, 0.3, 1.0],
                [0.7, 0.0, 0.6, 1.0],
            ],
            400.0,
        )
        .2;
        assert_ne!(
            a, b,
            "duas paletas de mesmo comprimento e cores diferentes pintaram os MESMOS bytes \
             — as swatches não estão usando a cor da paleta"
        );
    }

    /// **The strip wraps, so the row has no length limit of its own.** A fixed row would
    /// have re-imposed the cap this wave removed the moment a colour ran off the edge.
    #[test]
    fn the_strip_wraps_instead_of_running_off_the_edge() {
        let narrow = per_line(60.0, 1.0);
        assert!((2..20).contains(&narrow), "a narrow row fits few: {narrow}");
        assert!(per_line(600.0, 1.0) > narrow, "a wide row fits more");
        assert_eq!(
            per_line(1.0, 1.0),
            1,
            "narrower than one swatch still draws one"
        );
    }

    /// **A row with more colours is TALLER.** This is the executable form of "no limit":
    /// the answer to *where does the eleventh swatch go?* is "the next line", not
    /// "nowhere".
    #[test]
    fn the_row_grows_with_the_palette() {
        let h = |n: usize| painted(&reds(n), 120.0).0;
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
}
