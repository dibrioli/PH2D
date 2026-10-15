//! **Os AVISOS da secção SCRIPT** — a metade que responde a *«anexei um script e não acontece
//! nada»*.
//!
//! ⚠️ **Irmão por tecto de LOC** (600 por ficheiro de painel), e o corte é por RESPONSABILIDADE,
//! que é o que o tecto pede: ali mora *o que a secção CONTROLA* (as propriedades declaradas, o
//! ficheiro, o botão), aqui *o que ela DIZ quando alguma coisa está no caminho*. ⭐ O precedente é
//! o [`super::particles_avisos`], cortado da irmã pela mesma razão.
//!
//! ⛔ O corte foi forçado pela wave das alturas (2026-09-22): ao passar `FIELD_H`/`BTN_H` para as
//! portas `ALTURA_DE_CAMPO`/`ALTURA_DE_CAMPO`, os nomes mais longos fizeram o `rustfmt` quebrar
//! duas chamadas e o ficheiro passou o tecto por **uma** linha. *A cura de um tecto é sempre o
//! corte, nunca uma entrada no `FILE_OVERAGE_OK`.*

use super::*;
use ph2d_editor_core::script_edits::{
    InspectorScriptError, InspectorScriptInfo, InspectorScriptStatus,
};
use ph2d_i18n::{tr, tr_with};

/// O erro do Luau sobe um degrau sobre os outros avisos (`Sm`), por pedido do dono (15/10).
const ERRO: TypeToken = TypeToken::Base;

/// **A frase de um erro** — com a LINHA deste script, quando o Luau a disse.
fn erro(e: &InspectorScriptError) -> String {
    match e.line {
        Some(line) => tr_with(
            "panel.inspector.script.at_line",
            &[("line", &line), ("msg", &e.message)],
        ),
        None => e.message.clone(),
    }
}

/// ⭐ (CodeEditor E7) **O símbolo culpado num chip** — nunca cru entre aspas no meio da frase
/// (retorno do dono: o `'='` confunde). Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn simbolo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    sym: &str,
) -> f32 {
    crate::censo_dos_avisos::regista(sym);
    let font = ERRO.px();
    let rotulo = tr("panel.inspector.script.symbol");
    let lw = ph2d_editor_core::paint::label_column_width(text_system, font, [rotulo]);
    let pad = Spacing::Xs.px();
    let h = font + pad * 2.0;
    paint_text(
        text_system,
        scene,
        rotulo,
        x,
        y + pad,
        font,
        lw,
        resolve(ColorToken::Text2, theme),
    );
    let cw = ph2d_editor_core::paint::label_column_width(text_system, font, [sym])
        + Spacing::Md.px() * 2.0;
    let chip = Rect::new(x + lw + Spacing::Sm.px(), y, cw.min((w - lw).max(0.0)), h);
    fill_rounded_rect(
        scene,
        chip,
        Radius::Xs.px(),
        resolve(ColorToken::DangerSoft, theme),
    );
    ph2d_editor_core::paint::paint_text_centered(
        text_system,
        scene,
        sym,
        chip,
        font,
        resolve(ColorToken::Danger, theme),
    );
    y + h + ph2d_tokens::control_gap_px()
}

/// Os avisos do ficheiro e da corrida. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn avisos(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorScriptInfo,
) -> f32 {
    let mut y = y;
    // ⚠️ **Os avisos vêm ANTES dos números**, pela razão da secção do som: quem não vê nada a mexer
    // não quer afinar um número — quer saber porquê.
    match &info.status {
        InspectorScriptStatus::NoFile => {
            y = super::rows::aviso(
                scene,
                text_system,
                theme,
                x,
                w,
                y,
                tr("panel.inspector.script.no_script_file_yet_u_use_browse_to_pick_a_luau_file"),
                ColorToken::Text3,
            );
        }
        InspectorScriptStatus::Unavailable => {
            y = super::rows::aviso(
                scene,
                text_system,
                theme,
                x,
                w,
                y,
                tr("panel.inspector.script.scripting_is_not_available_in_this_session"),
                ColorToken::Danger,
            );
        }
        InspectorScriptStatus::Loading => {
            y = super::rows::aviso(
                scene,
                text_system,
                theme,
                x,
                w,
                y,
                tr("panel.inspector.script.reading_the_file_u"),
                ColorToken::Text3,
            );
        }
        InspectorScriptStatus::Missing => {
            y = super::rows::aviso(
                scene,
                text_system,
                theme,
                x,
                w,
                y,
                tr("panel.inspector.script.that_file_is_gone_u_pick_it_again"),
                ColorToken::Danger,
            );
        }
        InspectorScriptStatus::Unimported(path) => {
            y = super::rows::aviso(
                scene,
                text_system,
                theme,
                x,
                w,
                y,
                &tr_with(
                    "panel.inspector.script.file_not_found_browse_to_import",
                    &[("path", path)],
                ),
                ColorToken::Danger,
            );
        }
        InspectorScriptStatus::Broken(e) | InspectorScriptStatus::BrokenRunsPrevious(e) => {
            // ⭐ (E6/E7) Quebrado e PARADO, ou quebrado com a versão anterior a correr.
            let chave = if matches!(info.status, InspectorScriptStatus::Broken(_)) {
                "panel.inspector.script.the_script_has_an_error"
            } else {
                "panel.inspector.script.broken_runs_previous"
            };
            y = super::rows::aviso_em(
                scene,
                text_system,
                theme,
                x,
                w,
                y,
                &tr_with(chave, &[("msg", &erro(e))]),
                ColorToken::Danger,
                ERRO,
            );
            if let Some(s) = &e.symbol {
                y = simbolo(scene, text_system, theme, x, w, y, s);
            }
        }
        InspectorScriptStatus::Ready => {
            if info.props.is_empty() {
                y = super::rows::aviso(
                    scene,
                    text_system,
                    theme,
                    x,
                    w,
                    y,
                    tr("panel.inspector.script.this_script_offers_no_properties"),
                    ColorToken::Text3,
                );
            }
        }
    }
    if let Some(e) = &info.failure {
        y = super::rows::aviso_em(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            &tr_with(
                "panel.inspector.script.stopped_fix_and_save",
                &[("msg", &erro(e))],
            ),
            ColorToken::Danger,
            ERRO,
        );
        if let Some(s) = &e.symbol {
            y = simbolo(scene, text_system, theme, x, w, y, s);
        }
    }
    if info.kept > 0 {
        y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            &tr_with(
                "panel.inspector.script.values_kept_until_reload",
                &[("n", &info.kept)],
            ),
            ColorToken::Text3,
        );
    }
    if info.also_physics {
        y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            tr("panel.inspector.script.this_object_is_also_moved_by_physics_u_the_two_fight"),
            ColorToken::Warn,
        );
    }
    if !info.clock_playing {
        y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            tr(
                "panel.inspector.script.the_clock_is_stopped_u_scripts_only_run_while_the_scene_plays",
            ),
            ColorToken::Text3,
        );
    }
    y
}
