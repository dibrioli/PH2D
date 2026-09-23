//! **O EDITOR de uma acção** — as rows do verbo, do alvo e da tag.
//!
//! ⚠️ **Irmão por tecto de LOC** (600 por ficheiro de painel): a migração do HR-15 de
//! 2026-09-16 alongou cada rótulo (`tr("chave")` no lugar do literal) e o ficheiro passou
//! o tecto. O corte é por RESPONSABILIDADE, que é o que o tecto pede.

use super::*;

/// O editor da acção aberta. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn editor(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &InspectorActionRow,
    labels: &[String],
) -> f32 {
    // ⚠️ **A coluna do nome é da SECÇÃO** — desde 2026-09-22 as linhas de TEXTO também têm nome
    //    (report do dono), logo ela mede-se sobre eles.
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.actions.on_label"),
            tr("panel.inspector.actions.arg_label"),
        ],
    );
    let mut cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("panel.inspector.actions.on_label"),
        ids::INSP_ACTION_ON,
        TextInput::new(ids::INSP_ACTION_ON, "")
            .placeholder(tr("panel.inspector.actions.on_signal")),
        seccao,
    );
    // ⭐⭐⭐ **A escolha de QUEM SOFRE só existe onde o verbo a LÊ** (o FIM DE JOGO, 2026-09-19) — a
    // mesma lei do `arg` uma linha abaixo, e o `Restart Run` é o primeiro verbo cujo sujeito é a
    // CORRIDA e não uma entidade. ⛔ Pintá-la ali seria uma escolha que o consumidor deita fora.
    cur_y = if row.uses_target {
        target_rows(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            row,
        )
    } else {
        cur_y
    };
    cur_y = from_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        row,
    );
    cur_y = verb_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        labels,
        row.verb_tag,
    );
    // ⚠️ **O campo do parâmetro só existe onde o verbo o LÊ.** Mostrá-lo sempre seria um controlo
    // morto em três dos cinco verbos — a família que a caça de 30/08 mediu.
    if row.uses_arg {
        cur_y = super::anim_rows::text_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.actions.arg_label"),
            ids::INSP_ACTION_ARG,
            TextInput::new(ids::INSP_ACTION_ARG, "")
                .placeholder(tr("panel.inspector.actions.timer_name_empty_all")),
            seccao,
        );
    }
    // ⚠️⚠️ **A LINHA QUE RESPONDE AO «não acontece nada».**
    if row.never_fires() {
        let font = TypeToken::Sm.px();
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.actions.this_action_never_runs_it"),
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Warn, theme),
        );
        cur_y += font + ph2d_tokens::control_gap_px();
    }
    cur_y
}

/// ⭐⭐⭐ **DE QUEM o sinal tem de vir** — a TERCEIRA pergunta de uma linha (suplente #24).
///
/// # ⚠️ Porque ela existe, com o número
///
/// Sem ela, dez inimigos com a MESMA tabela reagem todos a um golpe que acertou em **um** — medido:
/// `10` efeitos para um sinal (`mede_o_que_a_composicao_ja_da_ao_golpe`). *O sinal é um nome global,
/// e a tabela não sabia quem levou o tiro.*
///
/// ⚠️ **Ela é uma cerca de QUEM REAGE, e o segmentado do alvo é de A QUEM** — as duas ficam uma por
/// cima da outra de propósito, na ordem em que a linha se lê: *quando · de quem · a quem · o quê*.
#[allow(clippy::too_many_arguments)]
fn from_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &InspectorActionRow,
) -> f32 {
    let so_meu = row.from_is_myself();
    let (seg_w, seg_dot) = ph2d_editor_core::widget::form_row_columns(x, w, y, ROW_H_PX);
    let seg_h = paint_segmented_group_adaptive(
        Rect::new(x, y, seg_w, ROW_H_PX),
        &[
            (
                tr("panel.inspector.actions.from_anyone"),
                !so_meu,
                crate::ids::INSP_ACTION_FROM_ANYONE,
            ),
            (
                tr("panel.inspector.actions.from_myself"),
                so_meu,
                crate::ids::INSP_ACTION_FROM_MYSELF,
            ),
        ],
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, seg_dot);
    // ⚠️ **A cauda sai da PORTA** (`control_gap_px`), e não de um degrau escrito aqui: *o que fica
    // depois de um bloco é UMA resposta*, e o gate `the_tail_of_a_block_is_one_answer` apanhou esta
    // linha na primeira corrida. ⭐ O irmão `target_rows` escrevia `Spacing::Xs` a meio de uma
    // instrução — invisível àquele censo — e passou a ler a mesma porta: os dois segmentados vivem
    // na MESMA coluna, a vinte pixels um do outro, e dois vãos diferentes ali leem-se como defeito.
    y + seg_h + ph2d_tokens::control_gap_px()
}

/// A secção inteira — cabeçalho, dobra e corpo. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_action_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorActionInfo,
    selected: usize,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let color_id = core_ids::INSP_LIVE_ACTION_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: acento neutro por omissão
    let title = if info.rows.is_empty() {
        String::from(tr("panel.inspector.actions.signal_actions"))
    } else {
        tr_with(
            "panel.inspector.actions.title_count",
            &[("n", &info.rows.len())],
        )
    };
    let header = section_header(store, core_ids::INSP_LIVE_ACTION_SECTION, &title).color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_ACTION_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    let font = TypeToken::Sm.px();

    if info.rows.is_empty() {
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.actions.no_actions_yet"),
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        cur_y += font + ph2d_tokens::control_gap_px();
    } else {
        cur_y = list(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            info,
            selected,
        );
    }
    cur_y = buttons(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
    );
    if let Some(row) = info.rows.get(selected) {
        cur_y = editor(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            row,
            &info.verb_labels,
        );
    }
    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}
