//! **O corpo da secção HUD** — irmão do [`super`] por CAP de FICHEIRO, o mesmo corte das irmãs.

use super::*;

/// Os quatro blocos, cada um só quando o objecto tem o componente dele.
#[allow(clippy::too_many_arguments)]
fn corpo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorHudInfo,
) -> f32 {
    let mut cur_y = y;

    // ── A RAIZ ───────────────────────────────────────────────────────────────
    if i.has_canvas {
        for n in [N::RefWidth, N::RefHeight] {
            cur_y = num_row(
                scene, text_system, theme, hit_index, store, x, w, cur_y, n,
            );
        }
        cur_y = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.hud.fit"),
            &crate::ids::INSP_HUD_FIT,
            &[
                tr("panel.inspector.hud.fit_keep"),
                tr("panel.inspector.hud.fit_stretch"),
            ],
            usize::from(i.fit),
        );
        // ⭐ **A razão de o canvas não se mexer, dita em voz alta.**
        if !i.tem_camera {
            cur_y = warn(
                scene,
                text_system,
                theme,
                x,
                w,
                cur_y,
                tr("panel.inspector.hud.no_game_camera"),
                ColorToken::Text3,
            );
        }
    }

    // ── O RÓTULO ─────────────────────────────────────────────────────────────
    if i.has_label {
        cur_y = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.hud.source"),
            &crate::ids::INSP_HUD_SOURCE,
            &[
                tr("panel.inspector.hud.source_authored"),
                tr("panel.inspector.hud.source_counter"),
                tr("panel.inspector.hud.source_timer"),
                tr("panel.inspector.hud.source_tag"),
            ],
            usize::from(i.source),
        );
        for (k, dica) in [
            (T::SourceName, tr("panel.inspector.hud.source_name")),
            (T::Prefix, tr("panel.inspector.hud.prefix")),
            (T::Suffix, tr("panel.inspector.hud.suffix")),
        ] {
            if !i.mostra_texto(k) {
                continue;
            }
            let Some(idx) = ph2d_editor_core::hud_edits::HUD_TEXTS
                .iter()
                .position(|&t| t == k)
            else {
                continue;
            };
            cur_y = txt_row(
                scene, text_system, theme, hit_index, store, x, w, cur_y, idx, dica,
            );
        }
        // ⭐⭐ **O que ele mostra AGORA** — ou a razão de não mostrar nada derivado.
        if i.source != 0 {
            if i.vivo.is_empty() {
                cur_y = warn(
                    scene,
                    text_system,
                    theme,
                    x,
                    w,
                    cur_y,
                    tr("panel.inspector.hud.source_missing"),
                    ColorToken::Text3,
                );
            } else {
                let linha = format!("{}: {}", tr("panel.inspector.hud.showing"), i.vivo);
                cur_y = warn(
                    scene,
                    text_system,
                    theme,
                    x,
                    w,
                    cur_y,
                    &linha,
                    ColorToken::Text2,
                );
            }
        }
    }

    // ── O BOTÃO ──────────────────────────────────────────────────────────────
    if i.has_button {
        if let Some(idx) = ph2d_editor_core::hud_edits::HUD_TEXTS
            .iter()
            .position(|&t| t == T::Signal)
        {
            cur_y = txt_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                cur_y,
                idx,
                tr("panel.inspector.hud.signal"),
            );
        }
        cur_y = check_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            crate::ids::INSP_HUD_DISABLED,
            tr("panel.inspector.hud.disabled"),
            i.disabled,
        );
        // ⚠️ As DUAS razões de um botão não responder, e elas são diferentes.
        if i.signal.trim().is_empty() {
            cur_y = warn(
                scene,
                text_system,
                theme,
                x,
                w,
                cur_y,
                tr("panel.inspector.hud.no_signal"),
                ColorToken::Text3,
            );
        }
        if i.disabled {
            cur_y = warn(
                scene,
                text_system,
                theme,
                x,
                w,
                cur_y,
                tr("panel.inspector.hud.button_disabled"),
                ColorToken::Text3,
            );
        }
    }

    // ── O CONTADOR ───────────────────────────────────────────────────────────
    if i.has_counter {
        if let Some(idx) = ph2d_editor_core::hud_edits::HUD_TEXTS
            .iter()
            .position(|&t| t == T::CounterName)
        {
            cur_y = txt_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                cur_y,
                idx,
                tr("panel.inspector.hud.counter_name"),
            );
        }
        cur_y = num_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            N::CounterStart,
        );
        // ⭐ O valor VIVO — leitura, nunca edição: ele não é documento.
        let linha = format!(
            "{}: {}",
            tr("panel.inspector.hud.counter_now"),
            i.counter_value
        );
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &linha,
            ColorToken::Text2,
        );
    }
    cur_y
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_hud_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorHudInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_HUD_SECTION,
        tr("panel.inspector.hud.hud"),
    );
    paint_section_header(
        &header,
        Rect::new(x, y, w, header_h),
        scene,
        text_system,
        theme,
    );
    let Some(fold) = SectionFold::begin(
        store,
        ph2d_editor_core::ids::INSP_LIVE_HUD_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let cur_y = corpo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y + header_h,
        info,
    );
    fold.finish(store, scene, hit_index, cur_y)
}
