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

    cur_y = bloco_raiz(scene, text_system, theme, hit_index, store, x, w, cur_y, i);
    cur_y = bloco_rotulo(scene, text_system, theme, hit_index, store, x, w, cur_y, i);
    cur_y = bloco_botao(scene, text_system, theme, hit_index, store, x, w, cur_y, i);
    cur_y = bloco_contador(scene, text_system, theme, hit_index, store, x, w, cur_y, i);
    cur_y
}

/// **A RAIZ** — irmão de [`corpo`] por tecto de LOC de FUNÇÃO (200).
///
/// ⚠️ O corte é por RESPONSABILIDADE e não por contagem: cada bloco é **um componente**,
/// e a `corpo` fica a ser o ÍNDICE dos quatro — que é como ela já se lia.
#[allow(clippy::too_many_arguments)]
fn bloco_raiz(
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
    if i.has_canvas {
        for n in [N::RefWidth, N::RefHeight] {
            cur_y = num_row(scene, text_system, theme, hit_index, store, x, w, cur_y, n);
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
            // ⚠️ A ORDEM é a do `ph2d_hud::Fit::ALL`, e o gate
            // `o_selector_do_fit_oferece_todos_os_modos` prende-as uma à outra: um modo novo na
            // lei sem entrada aqui existe, tem gates, e o artista **não lhe chega**.
            &[
                tr("panel.inspector.hud.fit_keep"),
                tr("panel.inspector.hud.fit_stretch"),
                tr("panel.inspector.hud.fit_expand"),
            ],
            usize::from(i.fit),
        );
        // ⭐ **A razão de o canvas não se mexer, dita em voz alta.**
        if !i.tem_camera {
            cur_y = super::rows::aviso(
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
    cur_y
}

/// **O RÓTULO** — irmão de [`corpo`] por tecto de LOC de FUNÇÃO (200).
///
/// ⚠️ O corte é por RESPONSABILIDADE e não por contagem: cada bloco é **um componente**,
/// e a `corpo` fica a ser o ÍNDICE dos quatro — que é como ela já se lia.
#[allow(clippy::too_many_arguments)]
fn bloco_rotulo(
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
    // ⭐⭐⭐ **E a secção DIZ onde está a metade que ela não mostra** (report do dono, 20/09).
    //
    // ⚠️ As linhas do canvas (`Fit`, a caixa de referência) vivem **na raiz**, e com um rótulo
    // escolhido a secção mostrava só a metade dele. *Uma secção que mostra metade e não diz onde
    // está a outra faz o artista concluir que ela não existe* — e foi exactamente o que aconteceu.
    //
    // ⚠️ Ela nomeia a RAIZ pelo nome dela, porque essa raiz **não se pega no canvas** (não tem
    // forma nenhuma): a Hierarquia é a única porta, e um aviso que não diga **qual** linha clicar
    // manda o artista procurar.
    if let Some(raiz) = i.canvas_parent.as_deref() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &ph2d_i18n::tr_with("panel.inspector.hud.rows_live_on_root", &[("raiz", &raiz)]),
            ColorToken::Text3,
        );
    }
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
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                cur_y,
                idx,
                dica,
            );
        }
        // ⭐⭐ **O que ele mostra AGORA** — ou a razão de não mostrar nada derivado.
        if i.source != 0 {
            if i.vivo.is_empty() {
                cur_y = super::rows::aviso(
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
                // ⚠️ **A composição vai por `tr_with`, nunca por `format!`**: o separador é
                // TEXTO de interface e vive na tabela (HR-15) — um `": "` no código é um
                // literal pintado que o censo apanha, e que nenhuma tradução alcança.
                let linha = ph2d_i18n::tr_with("panel.inspector.hud.showing", &[("v", &i.vivo)]);
                cur_y = super::rows::aviso(
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
    cur_y
}

/// **O BOTÃO** — irmão de [`corpo`] por tecto de LOC de FUNÇÃO (200).
///
/// ⚠️ O corte é por RESPONSABILIDADE e não por contagem: cada bloco é **um componente**,
/// e a `corpo` fica a ser o ÍNDICE dos quatro — que é como ela já se lia.
#[allow(clippy::too_many_arguments)]
fn bloco_botao(
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
            cur_y = super::rows::aviso(
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
            cur_y = super::rows::aviso(
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
    cur_y
}

/// **O CONTADOR** — irmão de [`corpo`] por tecto de LOC de FUNÇÃO (200).
///
/// ⚠️ O corte é por RESPONSABILIDADE e não por contagem: cada bloco é **um componente**,
/// e a `corpo` fica a ser o ÍNDICE dos quatro — que é como ela já se lia.
#[allow(clippy::too_many_arguments)]
fn bloco_contador(
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
        // ⭐⭐ **A caixa que faz o contador ATRAVESSAR um recomeço** — *«outra vida, mesma
        // pontuação»*. ⚠️ Ela fica **colada ao `Start`** de propósito: os dois respondem à mesma
        // pergunta (*com que valor este contador fica quando a corrida volta ao princípio?*), e
        // separá-los faria o artista ler o `Start` como a resposta inteira.
        cur_y = check_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            crate::ids::INSP_HUD_COUNTER_KEEP,
            tr("panel.inspector.hud.counter_keep"),
            i.counter_keep,
        );
        // ⚠️ **E o que ele NÃO faz, dito onde se lê a caixa:** um rebobinar do transporte repõe-no
        // à mesma. *Sem esta linha, «atravessa um recomeço» lê-se como «nunca mais volta ao
        // início», e o artista conclui que a caixa está partida ao carregar em Rewind.*
        if i.counter_keep {
            cur_y = warn(
                scene,
                text_system,
                theme,
                x,
                w,
                cur_y,
                tr("panel.inspector.hud.counter_keep_note"),
                ColorToken::Text3,
            );
        }
        // ⭐ O valor VIVO — leitura, nunca edição: ele não é documento.
        let linha = ph2d_i18n::tr_with(
            "panel.inspector.hud.counter_now",
            &[("v", &i.counter_value)],
        );
        cur_y = super::rows::aviso(
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
