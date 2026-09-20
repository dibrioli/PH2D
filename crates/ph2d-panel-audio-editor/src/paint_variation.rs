//! The Variation-container section of the Audio Editor panel (W6 asset-prep).
//!
//! A set of clips the game runtime plays **one** of per trigger, chosen by a strategy
//! (Random / Sequence / Shuffle) with per-play pitch/gain jitter and per-entry weights
//! — the Wwise Random/Sequence Container. Kills the robotic repetition of footsteps /
//! gunshots. Authored here, auditioned via **Play**, and saved to a manifest file.
//!
//! UI-only: the panel paints the list + arms intents; the shell owns the
//! [`ph2d_audio_edit::VariationSet`], the decoded clips, the picker and the audition.
//! The row labels + strategy name come from `variation_state` (the shell publishes
//! labels; the panel owns the selected row and the jitter slider positions).

use crate::paint::{ClippedHits, buttons_block, stepper_row};
use crate::{
    AEDIT_VAR_ADD, AEDIT_VAR_ADD_FOLDER, AEDIT_VAR_ENABLED, AEDIT_VAR_GAIN, AEDIT_VAR_LOAD,
    AEDIT_VAR_PITCH, AEDIT_VAR_PLAY, AEDIT_VAR_REMOVE, AEDIT_VAR_ROWS, AEDIT_VAR_SAVE,
    AEDIT_VAR_STRATEGY_NEXT, AEDIT_VAR_STRATEGY_PREV, AEDIT_VAR_WEIGHT_DOWN, AEDIT_VAR_WEIGHT_UP,
    MAX_VARIATIONS, variation_state,
};
use ph2d_editor_core::paint::{paint_text, paint_text_centered, resolve};

use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken, list_row_gap_px};
use ph2d_vector::VectorScene;

/// Width of the `◀` / `▶` strategy selector arrows (matches the effects selector).
/// Height of one variation list row — **a linha do app** (wave 17). Ela escrevia `22.0` a
/// mao, que por acaso era o valor do token: uma coincidencia nao e uma derivacao, e o proximo
/// pedido de «mais compacto» deixava esta lista para tras.
const VAR_ROW_H: f32 = ROW_H_PX;

/// Paint the Variations section starting at `y`; returns the `y` below it. `row_h` is
/// the shared button row height. Play/Remove/Weight need a variation to exist.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_variation_section(
    y: f32,
    x: f32,
    w: f32,
    row_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let count = variation_state::count();
    let has_any = count > 0;

    // No title row: the section header above carries the name AND the count.

    // ⭐ O selector de estratégia é um corpo (wave 20b) — ver o irmão em `paint_fx::paint_presets`.
    let y = stepper_row(
        Rect::new(x, y, w, row_h),
        &variation_state::strategy_name(),
        true,
        AEDIT_VAR_STRATEGY_PREV,
        AEDIT_VAR_STRATEGY_NEXT,
        scene,
        text_system,
        theme,
        hit_index,
    );

    // The clip list (selectable rows).
    let mut y = y + ph2d_tokens::control_gap_px();
    y = paint_var_list(y, x, w, scene, text_system, theme, hit_index);

    // ⭐⭐⭐ **Os sete são UM corpo** (wave 20, report do dono: *«tudo o que puder ser ajuntado,
    //    ajunte»*). Eles respondem todos à mesma pergunta — *o que se faz a este conjunto?* — e
    //    estavam em **quatro** controlos separados por vãos escritos à mão, com `Add | Add Folder`
    //    a ser o único par que já falava com a porta.
    //
    // Add file | Add folder: import by convention (a folder of `name_01..NN`).
    // Enabled | Remove: as duas coisas que se fazem a UMA entrada — uma reversível, a outra não,
    // lado a lado para a diferença ser óbvia. `Enabled` tira a entrada do sorteio sem a apagar: o
    // A/B de um conjunto (calar a take de que se duvida, ouvir o conjunto sem ela, repô-la) em vez
    // de remover o ficheiro e ter de o procurar outra vez.
    // Play: audiciona a variação seguinte. Weight ÷2 | ×2: o peso da entrada escolhida.
    let on = variation_state::selected_enabled();
    let y = buttons_block(
        Rect::new(x, y, w, row_h),
        &[2, 2, 1, 2],
        &[
            (tr("panel.audio_editor.variations.add"), true, AEDIT_VAR_ADD),
            (
                tr("panel.audio_editor.variations.add_folder"),
                true,
                AEDIT_VAR_ADD_FOLDER,
            ),
            (
                if on {
                    tr("panel.audio_editor.variations.enabled")
                } else {
                    tr("panel.audio_editor.variations.disabled")
                },
                has_any,
                AEDIT_VAR_ENABLED,
            ),
            (
                tr("panel.audio_editor.variations.remove"),
                has_any,
                AEDIT_VAR_REMOVE,
            ),
            (
                tr("panel.audio_editor.variations.play_variation"),
                has_any,
                AEDIT_VAR_PLAY,
            ),
            (
                tr("panel.audio_editor.variations.weight_half"),
                has_any,
                AEDIT_VAR_WEIGHT_DOWN,
            ),
            (
                tr("panel.audio_editor.variations.weight_double"),
                has_any,
                AEDIT_VAR_WEIGHT_UP,
            ),
        ],
        scene,
        text_system,
        theme,
        hit_index,
    );
    let mut y = y + ph2d_tokens::control_gap_px();

    // Per-play jitter (container-level). Always adjustable — they are set properties.
    y = paint_jitter_slider(
        y,
        x,
        w,
        tr("panel.audio_editor.variations.pitch_jitter"),
        AEDIT_VAR_PITCH,
        variation_state::pitch_jitter_norm(),
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = paint_jitter_slider(
        y,
        x,
        w,
        tr("panel.audio_editor.variations.gain_jitter"),
        AEDIT_VAR_GAIN,
        variation_state::gain_jitter_norm(),
        scene,
        text_system,
        theme,
        hit_index,
    );

    // ⭐ Save | Load do conjunto (ficheiros de manifesto) — um par, pela porta.
    let y = buttons_block(
        Rect::new(x, y, w, row_h),
        &[2],
        &[
            (
                tr("panel.audio_editor.variations.save"),
                has_any,
                AEDIT_VAR_SAVE,
            ),
            (
                tr("panel.audio_editor.variations.load"),
                true,
                AEDIT_VAR_LOAD,
            ),
        ],
        scene,
        text_system,
        theme,
        hit_index,
    );
    y + ph2d_tokens::control_gap_px()
}

/// The variation list: one selectable row per clip (the selected row is tinted). Rows
/// beyond [`MAX_VARIATIONS`] are never published, so the fixed id array always covers
/// the list. Returns the `y` below the list.
#[allow(clippy::too_many_arguments)]
fn paint_var_list(
    mut y: f32,
    x: f32,
    w: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let names = variation_state::names();
    if names.is_empty() {
        paint_text_centered(
            text_system,
            scene,
            tr("panel.audio_editor.variations.add_clips_to_build_a_set"),
            Rect::new(x, y, w, VAR_ROW_H),
            TypeToken::Xs.px(),
            resolve(ColorToken::Text2, theme),
        );
        // ⚠️ **O MESMO fecho do braço cheio, abaixo.** Esta linha dizia `Sm` (6) e a outra `Xs`
        // (4): duas respostas à mesma pergunta — *quanto ar fica DEPOIS desta lista?* — na mesma
        // função, e a lista muda de altura conforme está vazia ou não. O número certo do fim de
        // um grupo é a pergunta aberta nº 4 do handoff (o Godot dá `base·2` = 8); ⛔ escolhê-lo
        // aqui seria adivinhá-lo para uma lista só. O que esta wave faz é pôr as duas a dizer
        // o mesmo, para que a wave que o decidir mexa num sítio.
        return y + VAR_ROW_H + Spacing::Xs.px();
    }
    let sel = variation_state::variation_sel();
    for (i, name) in names.iter().enumerate().take(MAX_VARIATIONS) {
        let rect = Rect::new(x, y, w, VAR_ROW_H);
        // ⭐⭐ Pela porta (wave 21). ⛔ **Esta era a mais desviada das quatro:** toda linha enchia
        //    `Bg3` (o repouso de um botão) e a escolhida enchia `Accent` CHEIO com texto invertido
        //    — uma lista inteira de botões, com um deles aceso. Hoje ela tem a listra da paridade
        //    por baixo e o realce por cima, como as outras quatro.
        ph2d_editor_core::widget::paint_row_stripe(
            scene,
            rect,
            theme,
            ph2d_editor_core::widget::section_cards::CardDepth::Section.token(),
            i,
        );
        let fg = if i == sel {
            ColorToken::Text1
        } else {
            ColorToken::Text2
        };
        ph2d_editor_core::widget::paint_row_highlight(
            scene,
            rect,
            theme,
            if i == sel {
                ph2d_editor_core::widget::RowHighlight::Selected
            } else {
                ph2d_editor_core::widget::RowHighlight::None
            },
            0.0,
        );
        paint_text(
            text_system,
            scene,
            name,
            x + Spacing::Sm.px(),
            y + (VAR_ROW_H - TypeToken::Xs.px()) * 0.5,
            TypeToken::Xs.px(),
            (w - Spacing::Sm.px() * 2.0).max(1.0),
            resolve(fg, theme),
        );
        hit_index.register(AEDIT_VAR_ROWS[i], rect);
        y += VAR_ROW_H + list_row_gap_px();
    }
    y + ph2d_tokens::control_gap_px()
}

/// A labelled, always-adjustable jitter slider (`0..1`; the shell maps it to a `±`
/// range). No numeric readout — it is a feel control, like the loop crossfade.
#[allow(clippy::too_many_arguments)]
fn paint_jitter_slider(
    y: f32,
    x: f32,
    w: f32,
    label: &str,
    id: ph2d_a11y::NodeId,
    value: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    // ⚠️ **A nota do cabecalho dizia *«sem leitura numerica — e um controlo de
    //    sensacao, como o crossfade do laco»*, e o que ela descrevia era a AUSENCIA de uma
    //    leitura.** A caixa unica mostra a FRACCAO, que e o que este painel tem: ela nao promete
    //    unidade nenhuma e diz onde o dedo esta. O irmao do laco mudou no mesmo commit, pela
    //    mesma razao.
    crate::fileira_de_param::fileira_de_param(
        y,
        x,
        w,
        label,
        value,
        None,
        id,
        true,
        scene,
        text_system,
        theme,
        hit_index,
    ) + ph2d_tokens::control_gap_px()
}

/// The Variations readout, for the section header — how many clips the set holds,
/// visible without unfolding it.
pub(crate) fn variation_readout() -> String {
    match variation_state::count() {
        0 => tr("panel.audio_editor.variations.no_clips").to_string(),
        1 => tr("panel.audio_editor.variations.one_clip").to_string(),
        n => ph2d_i18n::tr_with("panel.audio_editor.variations.n_clips", &[("n", &n)]),
    }
}
