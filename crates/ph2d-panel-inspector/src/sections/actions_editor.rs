//! **O EDITOR de uma acção** — as rows do verbo, do alvo e da tag.
//!
//! ⚠️ **Irmão por tecto de LOC** (600 por ficheiro de painel): a migração do HR-15 de
//! 2026-09-16 alongou cada rótulo (`tr("chave")` no lugar do literal) e o ficheiro passou
//! o tecto. O corte é por RESPONSABILIDADE, que é o que o tecto pede.

use super::*;

/// ⭐⭐ **A coluna do nome do editor de uma acção — UMA, para as SETE linhas.**
///
/// ⛔⛔ Até 2026-09-23 havia duas medidas (o editor com `On`/`Argument`, o alvo só com `Target`) e
/// quatro linhas SEM NOME nenhum — os dois segmentados (*a quem* · *de quem*) e as duas caixas
/// (*o verbo* · *a tag*) pintavam o controlo desde a borda do conteúdo. Com o nome ao lado (a porta
/// [`ph2d_editor_core::property_row::paint_choice_row`]) todas entram na mesma coluna, senão cada
/// uma arranca num `x` diferente.
pub(super) fn seccao_da_acao(
    text_system: &mut TextSystem,
) -> ph2d_editor_core::property_row::Seccao {
    ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.actions.on_label"),
            tr("panel.inspector.actions.target_by"),
            tr("panel.inspector.actions.target_label"),
            tr("panel.inspector.actions.tag"),
            tr("panel.inspector.actions.source"),
            tr("panel.inspector.actions.do_label"),
            tr("panel.inspector.actions.arg_label"),
        ],
    )
}

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
    (linha, row): (usize, &InspectorActionRow),
    labels: &[String],
) -> f32 {
    let seccao = seccao_da_acao(text_system);
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
            seccao,
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
        seccao,
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
        seccao,
    );
    // ⚠️ **O campo do parâmetro só existe onde o verbo o LÊ.** Mostrá-lo sempre seria um controlo
    // morto em três dos cinco verbos — a família que a caça de 30/08 mediu.
    // ⭐ Com escolhas no snapshot (o *Play Clip* com UM alvo) o parâmetro é um ▾, nunca texto.
    if let (true, Some(nomes)) = (row.uses_arg, row.arg_options.as_deref()) {
        cur_y = clip_row(
            scene,
            text_system,
            theme,
            (hit_index, store),
            (x, w, cur_y),
            (linha, row, nomes),
            seccao,
        );
    } else if row.uses_arg {
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
                .placeholder(tr(dica_do_parametro(row.arg_hint))),
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

/// **As opções do ▾ do *Play Clip*** — uma por nome da lista do snapshot, na ordem dela.
///
/// ⚠️ O valor é a POSIÇÃO, e o despacho lê o nome da MESMA lista por ela (`event_action`). `zip`
/// com os ids: uma lista maior que o tecto perde as excedentes em vez de as pintar sem id.
pub(crate) fn clip_options(nomes: &[String]) -> Vec<DropdownOption<usize>> {
    ids::INSP_ACTION_CLIP
        .iter()
        .zip(nomes)
        .enumerate()
        .map(|(i, (&id, nome))| DropdownOption::new(id, i, nome.clone()))
        .collect()
}

/// ⭐ **O parâmetro do *Play Clip* como ▾** (onda 5, item c) — no molde do seletor do verbo.
///
/// | estado | chip | linha por baixo |
/// |---|---|---|
/// | nome na lista | o nome | — |
/// | nome vazio | a dica do parâmetro | — |
/// | nome FORA da lista | o nome, como dica | WARN: o clipe não existe na Animação do alvo |
/// | lista vazia | desabilitado, sem hit | WARN: o alvo não tem Animação |
///
/// ⚠️ O nome fora da lista vai na dica (`Text3`) e o tom de aviso na linha: o chip partilhado não
/// tem um estado de aviso, e a dica já diz «isto não é uma escolha da lista».
fn clip_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    (hit_index, store): (&mut HitIndex, &WidgetStore),
    (x, w, y): (f32, f32, f32),
    (linha, row, nomes): (usize, &InspectorActionRow, &[String]),
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let rotulo = ph2d_editor_core::property_row::paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H,
        tr("panel.inspector.actions.arg_label"),
        seccao,
    );
    let (rect, dot) = (rotulo.control, rotulo.dot);
    let vazia = nomes.is_empty();
    let atual = nomes.iter().position(|n| *n == row.arg);
    let open = !vazia
        && matches!(
            store.get(ids::INSP_ACTION_CLIP_PICK),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
    let dica = if row.arg.is_empty() {
        tr(dica_do_parametro(row.arg_hint)).to_string()
    } else {
        row.arg.clone()
    };
    let mut dd = Dropdown::new(ids::INSP_ACTION_CLIP_PICK, "", clip_options(nomes))
        .placeholder(dica)
        .open(open)
        .visual(store.dropdown_visual(ids::INSP_ACTION_CLIP_PICK));
    if vazia {
        dd = dd.state(ph2d_editor_core::widget::DropdownState::Disabled);
    } else {
        hit_index.register(ids::INSP_ACTION_CLIP_PICK, rect);
    }
    if let Some(i) = atual {
        dd.select(i);
    }
    paint_dropdown_chip(&dd, rect, scene, text_system, theme);
    if open {
        crate::state_popovers::set_pending_action_clip_dd(Some((linha, rect)));
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    let cur_y = y + ph2d_tokens::row_pitch_px();
    let aviso = if vazia {
        tr("panel.inspector.actions.target_has_no_animation").to_string()
    } else if atual.is_none() && !row.arg.trim().is_empty() {
        tr_with(
            "panel.inspector.actions.clip_not_in_target_animation",
            &[("clip", &row.arg)],
        )
    } else {
        return cur_y;
    };
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        &aviso,
        x,
        cur_y,
        font,
        w,
        resolve(ColorToken::Warn, theme),
    );
    cur_y + font + ph2d_tokens::control_gap_px()
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
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let so_meu = row.from_is_myself();
    ph2d_editor_core::property_row::paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("panel.inspector.actions.source"),
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
        seccao,
    )
    // ⚠️ **A cauda sai da PORTA** (`control_gap_px`), e não de um degrau escrito aqui: *o que fica
    // depois de um bloco é UMA resposta*, e o gate `the_tail_of_a_block_is_one_answer` apanhou esta
    // linha na primeira corrida. ⭐ O irmão `target_rows` escrevia `Spacing::Xs` a meio de uma
    // instrução — invisível àquele censo — e passou a ler a mesma porta: os dois segmentados vivem
    // na MESMA coluna, a vinte pixels um do outro, e dois vãos diferentes ali leem-se como defeito.
    // ⭐ Desde 2026-09-23 a cauda é da porta da ESCOLHA, que é a mesma resposta.
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
    let title = if info.rows.is_empty() {
        String::from(tr("panel.inspector.actions.signal_actions"))
    } else {
        tr_with(
            "panel.inspector.actions.title_count",
            &[("n", &info.rows.len())],
        )
    };
    let header = section_header(store, core_ids::INSP_LIVE_ACTION_SECTION, &title);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
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
            (selected, row),
            &info.verb_labels,
        );
    }
    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}

/// ⭐⭐ **A dica do campo do parâmetro, pelo que ele É** (plano 28, W2b).
///
/// ⛔ Nasceu de uma FOTO: a linha `Damage` pintava *«timer name (empty = all)»* no campo da
/// quantidade — e a do contador tinha a mesma dica errada desde o #20.
pub(crate) const fn dica_do_parametro(
    hint: ph2d_editor_core::screens::hero::ActionArgHint,
) -> &'static str {
    use ph2d_editor_core::screens::hero::ActionArgHint;
    match hint {
        ActionArgHint::TimerName => "panel.inspector.actions.timer_name_empty_all",
        ActionArgHint::Count => "panel.inspector.actions.count_empty_one",
        ActionArgHint::Amount => "panel.inspector.actions.amount_of_life",
        ActionArgHint::ObjectName => "panel.inspector.actions.object_name_empty_own",
        ActionArgHint::ClipName => "panel.inspector.actions.clip_name",
    }
}

#[cfg(test)]
mod dica_tests {
    use super::dica_do_parametro;
    use ph2d_editor_core::screens::hero::ActionArgHint;

    /// ⭐ **Cada espécie de parâmetro tem a SUA dica, e ela existe na tabela de textos.**
    #[test]
    fn cada_parametro_tem_a_sua_dica() {
        let chaves = [
            dica_do_parametro(ActionArgHint::TimerName),
            dica_do_parametro(ActionArgHint::Count),
            dica_do_parametro(ActionArgHint::Amount),
            dica_do_parametro(ActionArgHint::ObjectName),
            dica_do_parametro(ActionArgHint::ClipName),
        ];
        for (i, a) in chaves.iter().enumerate() {
            assert_ne!(
                ph2d_i18n::tr(a),
                *a,
                "a dica «{a}» não existe na tabela de textos"
            );
            for b in &chaves[i + 1..] {
                assert_ne!(a, b, "duas espécies de parâmetro com a mesma dica");
            }
        }
    }
}
