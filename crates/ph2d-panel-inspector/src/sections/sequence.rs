//! ⭐⭐⭐ **O que o Inspector mostra da CUTSCENE de um objecto** (TOP-20 #19, W3).
//!
//! # ⭐⭐ Um CHIP com a lista, e não um campo de texto
//!
//! As cutscenes são os containers do documento da timeline, e o conjunto delas é **conhecido**.
//! Um campo de texto obrigaria o artista a escrever um nome que tem de casar exactamente — e
//! `"porta"` contra `"Porta"` é uma cutscene que não corre, com todos os campos certos no ecrã.
//! ⇒ o mesmo idioma do verbo do sinal e do barramento do áudio: um chip que mostra UM nome, inteiro.
//!
//! # ⭐⭐⭐ E o que esta secção DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `No cutscenes in the timeline yet` | não há o que escolher — faz-se uma em *Timeline → + Container* |
//! | `No cutscene chosen` | o objecto tem o componente e não toca nada |
//! | `That cutscene is gone` | o nome guardado já não existe, e ele **FICA** |
//! | `This object has no Timer` | sem relógio ele é inerte — o descritor exige-o |
//! | `The clock is stopped` | sem a cena a tocar, nenhum `Timer` conta |
//! | `Not running` | falta um *Start Timer* a apontar-lhe |
//! | `The timer is shorter…` | a cutscene nunca chega ao fim |
//! | `Now: 1.20 s of 2.00` | o instante VIVO — leitura, nunca edição: não é documento |
//!
//! ⛔ **A ordem é da mais ESPECÍFICA para a mais geral** — a lei que a recusa dos pincéis já
//! escreve: *dizer «o relógio está parado» a quem também não escolheu cutscene nenhuma é mandá-lo
//! resolver a metade errada*.

use super::*;
use ph2d_editor_core::sequence_edits::InspectorSequenceInfo;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, SectionFold, paint_dropdown_chip};
use ph2d_i18n::{tr, tr_with};

/// **As opções do selector** — uma por container do documento, pela ordem dele.
///
/// ⚠️ **O valor da opção é o ÍNDICE, e o índice é a posição** — a mesma lei que o array de ids
/// declara e que o despacho lê com `position()`. ⚠️ `zip` com os nomes do snapshot: uma lista mais
/// longa que o array perde as excedentes em vez de as pintar sem id (o tecto é o
/// `ph2d_timeline::MAX_CONTAINERS`, e há gate na shell a atar os dois).
pub(crate) fn opcoes(nomes: &[String]) -> Vec<DropdownOption<usize>> {
    crate::ids::INSP_SEQ_OPT
        .iter()
        .enumerate()
        .zip(nomes.iter())
        .map(|((i, &id), nome)| DropdownOption::new(id, i, nome.clone()))
        .collect()
}

/// **O que o chip mostra quando nada está escolhido** — e as três frases são diferentes de
/// propósito.
///
/// ⚠️ Um documento SEM containers, um objecto que não escolheu e um nome ÓRFÃO leem-se iguais num
/// chip vazio, e as curas são três: fazer uma cutscene · escolher uma · perceber que a que estava
/// escolhida desapareceu. *Um placeholder que diz «—» nos três casos é um controlo que não ajuda.*
pub(crate) fn placeholder(info: &InspectorSequenceInfo) -> String {
    if info.orfao() {
        return info.container.clone();
    }
    if info.nomes.is_empty() {
        return tr("panel.inspector.sequence.no_cutscenes").to_owned();
    }
    tr("panel.inspector.sequence.none_chosen").to_owned()
}

/// **O selector da cutscene** — o nome à esquerda, o chip à direita.
#[allow(clippy::too_many_arguments)]
fn linha_do_selector(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSequenceInfo,
) -> f32 {
    let rotulo = tr("panel.inspector.sequence.cutscene");
    let seccao = ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &[rotulo]);
    let row = ph2d_editor_core::property_row::paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        rotulo,
        seccao,
    );
    hit_index.register(crate::ids::INSP_SEQ_PICK, row.control);
    let aberto = matches!(
        store.get(crate::ids::INSP_SEQ_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let mut dd = Dropdown::new(crate::ids::INSP_SEQ_PICK, "", opcoes(&info.nomes))
        .open(aberto)
        .placeholder(placeholder(info))
        .visual(store.dropdown_visual(crate::ids::INSP_SEQ_PICK));
    // ⚠️ **A escolha vem do SNAPSHOT e o `open` vem do store** — ler a escolha do store faria o chip
    // mostrar a cutscene do objecto ANTERIOR depois de trocar de selecção (a lei que a §12 paga).
    if let Some(i) = info.escolhido {
        dd.select(i);
    }
    paint_dropdown_chip(&dd, row.control, scene, text_system, theme);
    // ⚠️ **O popover NÃO se pinta aqui** — ele sairia debaixo da secção seguinte. O rect vai ao slot
    // e o passe diferido do painel desenha-o por cima de tudo.
    if aberto {
        crate::state_popovers::set_pending_seq_dd(Some(row.control));
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, row.dot);
    y + ph2d_tokens::row_pitch_px()
}

/// ⭐ **O botão que LARGA a cutscene** — só existe quando há uma escolhida ou um nome órfão.
///
/// ⚠️ **Sem ele o selector é uma porta de sentido único**, e pô-lo como linha do popover faria a
/// posição de uma opção deixar de ser o índice do container.
#[allow(clippy::too_many_arguments)]
fn botao_largar(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSequenceInfo,
) -> f32 {
    if info.nome().is_none() {
        return y;
    }
    let rect = ph2d_editor_core::property_row::caixa_do_botao(
        text_system,
        x,
        w,
        y,
        tr("panel.inspector.sequence.clear"),
    );
    hit_index.register(crate::ids::INSP_SEQ_CLEAR, rect);
    paint_button(
        &Button::new(
            crate::ids::INSP_SEQ_CLEAR,
            tr("panel.inspector.sequence.clear"),
        )
        .kind(ButtonKind::Default)
        .visual(store.button_visual(crate::ids::INSP_SEQ_CLEAR)),
        rect,
        scene,
        text_system,
        theme,
    );
    ph2d_editor_core::property_row::abaixo_do_botao(rect)
}

/// **Os avisos, da razão mais ESPECÍFICA para a mais geral.** Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn avisos(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSequenceInfo,
) -> f32 {
    let mut cur_y = y;
    let mut diz = |texto: &str, token: ColorToken, cur_y: &mut f32| {
        *cur_y = super::rows::aviso(scene, text_system, theme, x, w, *cur_y, texto, token);
    };

    // 1. Não há o que escolher · 2. não escolheu · 3. escolheu e desapareceu.
    if info.nomes.is_empty() {
        diz(
            tr("panel.inspector.sequence.make_one_in_the_timeline"),
            ColorToken::Text3,
            &mut cur_y,
        );
    } else if info.orfao() {
        diz(
            &tr_with(
                "panel.inspector.sequence.gone",
                &[("v", &info.container.trim())],
            ),
            ColorToken::Warn,
            &mut cur_y,
        );
    } else if info.nome().is_none() {
        diz(
            tr("panel.inspector.sequence.nothing_plays"),
            ColorToken::Text3,
            &mut cur_y,
        );
    }

    // 3-bis. ⭐ A VISTA da timeline. ⚠️ Linha PRÓPRIA e não um braço da escada do relógio: o
    // relógio pode estar perfeito e a cutscene parada à mesma, e a cura é outra (mudar de aba).
    if !info.vista_deixa_correr {
        diz(
            tr("panel.inspector.sequence.timeline_solo"),
            ColorToken::Warn,
            &mut cur_y,
        );
    }

    // 4. O relógio. ⚠️ As três razões de ele não andar são DIFERENTES e a cura de cada uma também.
    if !info.tem_relogio {
        diz(
            tr("panel.inspector.sequence.no_timer"),
            ColorToken::Warn,
            &mut cur_y,
        );
    } else if !info.clock_playing {
        diz(
            tr("panel.inspector.sequence.clock_stopped"),
            ColorToken::Text3,
            &mut cur_y,
        );
    } else if !info.a_correr {
        diz(
            tr("panel.inspector.sequence.not_running"),
            ColorToken::Text3,
            &mut cur_y,
        );
    }

    // 5. ⭐⭐⭐ O relógio acaba antes da cutscene — ela NUNCA chega ao fim.
    if info.relogio_curto() {
        diz(
            &tr_with(
                "panel.inspector.sequence.timer_too_short",
                &[
                    ("t", &format!("{:.2}", info.duracao_do_relogio)),
                    ("c", &format!("{:.2}", info.duracao_da_cutscene)),
                ],
            ),
            ColorToken::Warn,
            &mut cur_y,
        );
    }

    // 6. O instante VIVO — só quando ela de facto corre.
    if info.a_correr && info.escolhido.is_some() {
        diz(
            &tr_with(
                "panel.inspector.sequence.now",
                &[
                    ("t", &format!("{:.2}", info.t)),
                    ("c", &format!("{:.2}", info.duracao_da_cutscene)),
                ],
            ),
            ColorToken::Text2,
            &mut cur_y,
        );
    }

    // 7. ⚠️ A secção edita o PRIMÁRIO — a lei das irmãs.
    cur_y
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_sequence_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSequenceInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_SEQ_SECTION,
        tr("panel.inspector.sequence.sequence"),
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
        ph2d_editor_core::ids::INSP_LIVE_SEQ_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    cur_y = linha_do_selector(
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
    cur_y = botao_largar(
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
    cur_y = avisos(scene, text_system, theme, x, w, cur_y, info);
    fold.finish(store, scene, hit_index, cur_y)
}
