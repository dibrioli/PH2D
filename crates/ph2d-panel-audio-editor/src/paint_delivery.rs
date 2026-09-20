//! The Delivery section of the Audio Editor panel (W6 — asset-prep).
//!
//! Two numbers decide whether an asset ships: what the player **downloads** and what
//! the engine **holds**. This section prices both before the export, not after.
//!
//! The trade it exists to make visible: **the codec moves Disk and never RAM.** A resident
//! Vorbis clip decodes to the same `f32` buffer a WAV would — compressing an asset shrinks the
//! download and buys back exactly zero memory. People assume the opposite, and a number on
//! screen is the only way to un-assume it. (ADR-0118 later found the same thing from the mixer's
//! side, and answered it with **streaming voices** — but a clip the *editor* has open is
//! resident by definition, so what this section prices is still the resident cost.)
//!
//! Which is why the **shipping targets** below are formats and not just codecs. A variant that
//! only swapped the container would print the same RAM figure three times; Mobile conforms the
//! audio (24 kHz, mono) and is the only one that buys memory back — a quarter of it
//! (`ph2d_audio_encode::platform`).
//!
//! UI-only: the panel owns the codec choice and the quality slider; the shell owns the
//! encoders, sizes every file for real (no bitrate guesses) and publishes the readouts as
//! finished strings via `delivery_state`.

use crate::paint::{ClippedHits, button, stepper_row};
use crate::{
    AEDIT_CODEC_NEXT, AEDIT_CODEC_PREV, AEDIT_EXPORT_SET, AEDIT_OGG_QUALITY, delivery_state,
};
use ph2d_editor_core::paint::{paint_text, resolve};

use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A clip eating more than this share of the audio subsystem's RAM budget (HR-13,
/// 30 MB) is worth a second look — one sound should not be most of the envelope.
const BUDGET_WARN_FRAC: f32 = 0.25; // LITERAL-PX-OK: share of a RAM budget, not a design value

/// Paint the Delivery section starting at `y`; returns the `y` below it. Everything
/// dims when there is no clip: there is nothing to price.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_delivery_section(
    y: f32,
    x: f32,
    w: f32,
    loaded: bool,
    row_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let gap = Spacing::Xs.px();
    let label_h = TypeToken::Xs.px();

    // No title row: the section header above carries the name AND the download size —
    // the number people came for, readable without unfolding the section.

    // Codec selector: ◀ | name | ▶. It drives both the readout and the Export button,
    // so there is exactly one place the codec is decided.
    // ⭐ O selector de formato é um corpo (wave 20b) — ver o irmão em `paint_fx::paint_presets`.
    let mut y = stepper_row(
        Rect::new(x, y, w, row_h),
        &delivery_state::codec_name(),
        loaded,
        AEDIT_CODEC_PREV,
        AEDIT_CODEC_NEXT,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + gap;

    // The compression control. It is **Quality** for Vorbis and **Bitrate** for Opus — the same
    // slider, driving a different knob, and it says which rather than calling a bitrate
    // "quality" (ADR-0116). On a lossless codec it is inert, and it says so by dimming rather
    // than by pretending to do something.
    let lossy = delivery_state::is_lossy();
    let live = loaded && lossy;
    // ⭐ O nome deste controlo é do MOTOR (`Quality` ou `Bitrate`, conforme o codec) — ver
    //    `editor_publish_delivery`. Ele vai para a coluna do NOME da caixa única; a coluna do
    //    valor fica com a fracção, que é o que este painel tem.
    let label = delivery_state::quality_label();
    // ⛔ Um codec sem perdas não tem qualidade que se troque: a fileira é pintada e NÃO
    //    registada — ver [`crate::fileira_de_param`], onde essa recusa é uma só para as cinco.
    y = crate::fileira_de_param::fileira_de_param(
        y,
        x,
        w,
        if label.is_empty() {
            tr("panel.audio_editor.delivery.quality")
        } else {
            &label
        },
        delivery_state::quality_norm(),
        None,
        AEDIT_OGG_QUALITY,
        live,
        scene,
        text_system,
        theme,
        hit_index,
    ) + gap;

    // What the ENGINE pays — the half of the trade the codec has no say in.
    let frac = delivery_state::budget_frac();
    let over = frac > BUDGET_WARN_FRAC;
    let ram = delivery_state::ram();
    y = text_row(
        if loaded && !ram.is_empty() {
            &ram
        } else {
            tr("panel.audio_editor.delivery.ram")
        },
        x,
        y,
        w,
        label_h,
        resolve(
            match (loaded, over) {
                (false, _) => ColorToken::Text3,
                (true, true) => ColorToken::Warn,
                (true, false) => ColorToken::Text2,
            },
            theme,
        ),
        scene,
        text_system,
    ) + gap;

    // Loop points and cue markers live in WAV chunks. Exporting to Vorbis silently
    // loses them, so the panel says it out loud while the choice can still be changed.
    if loaded && delivery_state::drops_meta() {
        y = text_row(
            tr("panel.audio_editor.delivery.drops_loop_points_and_markers"),
            x,
            y,
            w,
            label_h,
            resolve(ColorToken::Warn, theme),
            scene,
            text_system,
        ) + gap;
    }

    y = paint_shipping_targets(loaded, x, y, w, label_h, gap, scene, text_system, theme);

    // One click writes all three, each conformed to its own platform's format first.
    button(
        Rect::new(x, y, w, row_h),
        tr("panel.audio_editor.delivery.export_set"),
        loaded,
        AEDIT_EXPORT_SET,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + gap;

    // **Export Pieces** — one file per piece, in the codec priced above, named `<stem>_01..NN`:
    // exactly what the variation importer reads back as one group. Record eight footsteps in one
    // pass, split, export — and the eight assets come back as a ready-made variation container.
    //
    // It is dim until the clip is actually cut, because with one piece this is just Export.
    button(
        Rect::new(x, y, w, row_h),
        tr("panel.audio_editor.delivery.export_pieces"),
        loaded && crate::tool_state::has_cuts(),
        crate::AEDIT_EXPORT_PIECES,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y + row_h + ph2d_tokens::control_gap_px()
}

/// The download size, for the section header. Empty when there is nothing to price.
pub(crate) fn delivery_readout() -> String {
    let disk = delivery_state::disk();
    if disk.is_empty() {
        "\u{2014}".to_string()
    } else {
        disk
    }
}

/// Paint one line of body text and return the `y` **below what was actually laid out**.
///
/// `paint_text`'s `max_width` is a *wrap budget*, not a clip: a string wider than the
/// panel comes out as two rows. Advancing `y` by one line height then prints the next
/// row **on top of the wrapped one** — which is exactly what Enio's smoke caught
/// (2026-07-12), with the RAM readout and the codec warning stacked on each other.
///
/// So measure, do not assume. The strings are also kept short enough to fit on one line
/// at the panel's width, but a readout is built from numbers and a translation can be
/// longer than the English — the layout has to survive that on its own.
#[allow(clippy::too_many_arguments)]
fn text_row(
    text: &str,
    x: f32,
    y: f32,
    w: f32,
    size: f32,
    color: ph2d_vector::Color,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
) -> f32 {
    let h = text_system.layout(text, size, w).height().max(size);
    paint_text(text_system, scene, text, x, y, size, w, color);
    y + h
}

/// **Os alvos de entrega — uma linha por plataforma:** o que ela baixa, o que ela segura, e se
/// isso é uma fatia preocupante do orçamento. São os números pelos quais uma variante EXISTE, e
/// diferem entre linhas só porque cada alvo *conforma* o áudio, não apenas o contêiner (ver o doc
/// do módulo).
///
/// ⚠️ **Recebe o `y` e devolve o `y`** — a forma que o cap de função deste crate prescreve, e a
/// razão de o corte ser aqui: este bloco é a única parte do painel que percorre uma LISTA cujo
/// tamanho o painel não escolhe.
#[allow(clippy::too_many_arguments)]
fn paint_shipping_targets(
    loaded: bool,
    x: f32,
    mut y: f32,
    w: f32,
    label_h: f32,
    gap: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let targets = delivery_state::platforms();
    if loaded && !targets.is_empty() {
        for (name, cost, frac) in &targets {
            y = text_row(
                &format!("{name}  {cost}"),
                x,
                y,
                w,
                label_h,
                resolve(
                    if *frac > BUDGET_WARN_FRAC {
                        ColorToken::Warn
                    } else {
                        ColorToken::Text2
                    },
                    theme,
                ),
                scene,
                text_system,
            );
        }
        y += gap;
    }
    y
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::interaction::HitIndex;

    /// Paint the section with `ram` published and report how tall it came out.
    fn section_height(ram: &str) -> f32 {
        delivery_state::set_codec_info(4, "Ogg Vorbis", true, "Quality");
        delivery_state::set_cost("25 KB", ram, 0.07, false);
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let store = ph2d_editor_core::interaction::WidgetStore::default();
        let mut hits = HitIndex::default();
        let clip = Rect::new(0.0, 0.0, 220.0, 4_000.0);
        let mut ch = ClippedHits::new(&store, &mut hits, clip);
        paint_delivery_section(
            0.0,
            0.0,
            220.0, // the panel's real body width, near enough
            true,
            24.0,
            &mut scene,
            &mut text,
            Theme::default(),
            &mut ch,
        )
    }

    /// **The readout must make room for what it actually printed.**
    ///
    /// `paint_text`'s `max_width` is a wrap budget, not a clip: a string too wide for the
    /// panel comes back as TWO rows. The section used to advance `y` by one line height
    /// regardless, so the next line printed on top of the wrapped one — Enio's smoke
    /// showed the RAM readout and the codec warning stacked into an unreadable smear
    /// (2026-07-12).
    ///
    /// Red with a fixed advance: both strings would report the same height.
    #[test]
    fn a_readout_that_wraps_makes_room_for_its_second_line() {
        let short = section_height("RAM 2.2 MB");
        let wraps = section_height(
            "RAM 2.2 MB and then a great deal more text than could ever fit across one \
             single line of this panel at any sane width",
        );
        assert!(
            wraps > short,
            "a wrapped readout did not push the section down: {short} vs {wraps} \
             (the next line is printing on top of it)"
        );
    }
}
