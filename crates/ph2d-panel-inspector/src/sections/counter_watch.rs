//! ⭐⭐⭐ **O que o Inspector mostra da VIGIA DO CONTADOR** — as regras *«quando o número X chegar
//! a N, diz S»*.
//!
//! # ⭐⭐ A lista + UM editor, e o CHIP para a comparação
//!
//! A forma é a do `Timers`: uma linha por regra, e um editor só para a aberta. A comparação é um
//! **chip** e não um campo de texto porque o conjunto dela é **conhecido e tem três elementos** —
//! escrever `<=` à mão seria dar ao artista uma maneira de a errar.
//!
//! # ⭐⭐⭐ E o que esta secção DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `No rules yet` | o objecto tem o componente e não vigia nada |
//! | `There is no counter called «x»` | ⭐ o nome não casa com contador nenhum da cena — **a regra nunca dispara**, e sem esta linha ela é indistinguível de uma que funciona |
//! | `This rule says nothing` | sem nome de sinal ela segue o número e cala-se |
//! | `The clock is stopped` | sem a cena a tocar, nenhuma vigia avalia |
//! | `Now: 3` | o valor VIVO do contador — leitura, nunca edição |
//!
//! ⛔ **Da mais ESPECÍFICA para a mais geral** — a lei da recusa dos pincéis: *dizer «o relógio
//! está parado» a quem escreveu o nome do contador com um `d` a mais é mandá-lo resolver a metade
//! errada.*

use super::*;
use ph2d_editor_core::counter_watch_edits::{InspectorCounterWatchInfo, InspectorWatchRow};
use ph2d_editor_core::widget::{Dropdown, DropdownOption, SectionFold, paint_dropdown_chip};
use ph2d_i18n::{tr, tr_with};

const BTN_H: f32 = 30.0; // LITERAL-PX-OK: altura de botão do Inspector, igual à das irmãs
const ROW_H: f32 = ph2d_tokens::ROW_H_PX;

/// Os três rótulos da comparação, **pela ordem do enum do motor**.
///
/// ⚠️ O índice É o `u8` que a edição carrega — reordenar isto reescreve o sentido de toda regra já
/// gravada. Há gate na shell a prender a ordem à do `ph2d_ecs::Compare`.
pub(crate) fn opcoes_de_comparacao() -> Vec<DropdownOption<usize>> {
    crate::ids::INSP_WATCH_CMP_OPT
        .iter()
        .enumerate()
        .zip([
            tr("panel.inspector.counter_watch.at_most"),
            tr("panel.inspector.counter_watch.at_least"),
            tr("panel.inspector.counter_watch.exactly"),
        ])
        .map(|((i, &id), rotulo)| DropdownOption::new(id, i, rotulo.to_owned()))
        .collect()
}

/// O símbolo da comparação, para o resumo da linha.
fn simbolo(compare: u8) -> &'static str {
    match compare {
        1 => "\u{2265}", // ≥
        2 => "=",
        _ => "\u{2264}", // ≤
    }
}

/// **O que esta regra FAZ, numa frase.**
///
/// ⚠️ Ela diz o contador, a comparação, o limiar e para onde fala — as quatro perguntas que o
/// artista tem com a lista fechada. E diz *«mute»* em vez de deixar o espaço em branco: uma regra
/// sem sinal é uma escolha legítima que tem de se ler como escolha.
fn resumo(row: &InspectorWatchRow) -> String {
    let alvo = if row.signal.trim().is_empty() {
        tr("panel.inspector.counter_watch.mute").to_owned()
    } else {
        format!("\u{2192} {}", row.signal)
    };
    format!("{} {} {alvo}", simbolo(row.compare), row.value)
}

/// A lista das regras. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn lista(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorCounterWatchInfo,
    escolhida: usize,
) -> f32 {
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    for (i, (row, &id)) in info
        .rows
        .iter()
        .zip(crate::ids::INSP_WATCH_ROW.iter())
        .enumerate()
    {
        let rect = Rect::new(x, cur_y, w, ROW_H);
        hit_index.register(id, rect);
        ph2d_editor_core::widget::paint_row_stripe(
            scene,
            rect,
            theme,
            ph2d_editor_core::widget::section_cards::CardDepth::Section.token(),
            i,
        );
        ph2d_editor_core::widget::paint_row_highlight(
            scene,
            rect,
            theme,
            if i == escolhida {
                ph2d_editor_core::widget::RowHighlight::Selected
            } else if store.hover_live(id) > 0.0 {
                ph2d_editor_core::widget::RowHighlight::Hovered
            } else {
                ph2d_editor_core::widget::RowHighlight::None
            },
            0.0,
        );
        // ⚠️⚠️ **Uma regra ÓRFÃ escreve-se em WARN na própria lista**, e não só no editor: com seis
        // regras e uma errada, o artista não abre as seis à procura da que não funciona.
        let cor = if row.counter_existe {
            if i == escolhida {
                resolve(ColorToken::Text1, theme)
            } else {
                resolve(ColorToken::Text2, theme)
            }
        } else {
            resolve(ColorToken::Warn, theme)
        };
        paint_text(
            text_system,
            scene,
            &row.counter,
            x + Spacing::Sm.px(),
            cur_y + (ROW_H - font) * 0.5,
            font,
            w,
            cor,
        );
        paint_text(
            text_system,
            scene,
            &resumo(row),
            x + w * 0.5,
            cur_y + (ROW_H - font) * 0.5,
            font,
            w * 0.5 - Spacing::Sm.px(),
            resolve(ColorToken::Text3, theme),
        );
        cur_y += ROW_H;
    }
    cur_y + ph2d_tokens::control_gap_px()
}

/// Os dois botões da lista. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn botoes(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorCounterWatchInfo,
) -> f32 {
    // ⚠️ **O `+` DESAPARECE no tecto** e não fica cinzento a mentir — a lei das irmãs.
    let pode_juntar = info.rows.len() < crate::ids::INSP_WATCH_ROW.len();
    let pode_tirar = !info.rows.is_empty();
    let n = usize::from(pode_juntar) + usize::from(pode_tirar);
    if n == 0 {
        return y;
    }
    let seg = ph2d_editor_core::widget::segment_rects(Rect::new(x, y, w, BTN_H), n);
    let mut cell = 0usize;
    if pode_juntar {
        let (rect, group) = seg[cell];
        cell += 1;
        hit_index.register(ids::INSP_WATCH_ADD, rect);
        paint_button(
            &Button::new(
                ids::INSP_WATCH_ADD,
                tr("panel.inspector.counter_watch.plus_add_rule"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_WATCH_ADD))
            .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    if pode_tirar {
        let (rect, group) = seg[cell];
        hit_index.register(ids::INSP_WATCH_REMOVE, rect);
        paint_button(
            &Button::new(
                ids::INSP_WATCH_REMOVE,
                tr("panel.inspector.counter_watch.x_remove_rule"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_WATCH_REMOVE))
            .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    y + BTN_H + ph2d_tokens::control_gap_px()
}

/// A linha do chip da comparação. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn linha_da_comparacao(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &InspectorWatchRow,
) -> f32 {
    let rotulo = tr("panel.inspector.counter_watch.compare");
    let seccao = ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &[rotulo]);
    let linha = ph2d_editor_core::property_row::paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H,
        rotulo,
        seccao,
    );
    hit_index.register(crate::ids::INSP_WATCH_CMP_PICK, linha.control);
    let aberto = matches!(
        store.get(crate::ids::INSP_WATCH_CMP_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let mut dd = Dropdown::new(crate::ids::INSP_WATCH_CMP_PICK, "", opcoes_de_comparacao())
        .open(aberto)
        .visual(store.dropdown_visual(crate::ids::INSP_WATCH_CMP_PICK));
    // ⚠️ **A escolha vem do SNAPSHOT e o `open` vem do store** — ler a escolha do store faria o
    // chip mostrar a comparação da regra ANTERIOR depois de trocar de linha.
    dd.select(usize::from(row.compare));
    paint_dropdown_chip(&dd, linha.control, scene, text_system, theme);
    if aberto {
        crate::state_popovers::set_pending_watch_dd(Some((row.compare, linha.control)));
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, linha.dot);
    y + ph2d_tokens::row_pitch_px()
}

/// O editor da regra aberta. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn editor(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorCounterWatchInfo,
    row: &InspectorWatchRow,
) -> f32 {
    let mut cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_WATCH_COUNTER,
        TextInput::new(ids::INSP_WATCH_COUNTER, "")
            .placeholder(tr("panel.inspector.counter_watch.counter_name")),
    );
    cur_y = linha_da_comparacao(
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
    let sec = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[tr("panel.inspector.counter_watch.value")],
    );
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.counter_watch.value"),
        &[ids::INSP_WATCH_VALUE],
        1.0, // LITERAL-PX-OK: passo de scrub em UNIDADES do contador, não em pixels
        None,
        sec,
    );
    cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        ids::INSP_WATCH_SIGNAL,
        TextInput::new(ids::INSP_WATCH_SIGNAL, "")
            .placeholder(tr("panel.inspector.counter_watch.signal_name_empty_mute")),
    );
    cur_y = ph2d_editor_core::property_row::paint_check_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &[(
            ids::INSP_WATCH_ONCE,
            tr("panel.inspector.counter_watch.only_once"),
            row.once,
        )],
        sec,
    );
    avisos(scene, text_system, theme, x, w, cur_y, info, row)
}

/// ⚠️⚠️ **A LINHA QUE RESPONDE AO «nada acontece»** — da mais específica para a mais geral.
#[allow(clippy::too_many_arguments)]
fn avisos(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorCounterWatchInfo,
    row: &InspectorWatchRow,
) -> f32 {
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    let (msg, cor) = if row.counter.trim().is_empty() {
        (
            Some(tr("panel.inspector.counter_watch.no_counter_named_yet").to_owned()),
            ColorToken::Warn,
        )
    } else if !row.counter_existe {
        (
            Some(tr_with(
                "panel.inspector.counter_watch.there_is_no_counter_called",
                &[("name", &row.counter.trim())],
            )),
            ColorToken::Warn,
        )
    } else if row.signal.trim().is_empty() {
        (
            Some(tr("panel.inspector.counter_watch.this_rule_says_nothing").to_owned()),
            ColorToken::Warn,
        )
    } else if !info.clock_playing {
        (
            Some(tr("panel.inspector.counter_watch.the_clock_is_stopped").to_owned()),
            ColorToken::Text3,
        )
    } else if let Some(v) = row.valor_vivo {
        (
            Some(tr_with(
                "panel.inspector.counter_watch.now_value",
                &[("v", &v)],
            )),
            ColorToken::Text3,
        )
    } else {
        (None, ColorToken::Text3)
    };
    if let Some(m) = msg {
        paint_text(
            text_system,
            scene,
            &m,
            x,
            cur_y,
            font,
            w,
            resolve(cor, theme),
        );
        cur_y += font + ph2d_tokens::control_gap_px();
    }
    cur_y
}

/// A secção inteira — cabeçalho, dobra e corpo. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_counter_watch_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorCounterWatchInfo,
    escolhida: usize,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let color_id = core_ids::INSP_LIVE_WATCH_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: acento neutro por omissão
    // ⚠️ **O título CONTA as órfãs**, e é a única coisa que se vê com a secção dobrada — que é
    // exactamente o estado em que uma regra partida passa despercebida.
    let orfas = info.orfas();
    let titulo = if info.rows.is_empty() {
        String::from(tr("panel.inspector.counter_watch.counter_watch"))
    } else if orfas > 0 {
        tr_with(
            "panel.inspector.counter_watch.title_count_broken",
            &[("n", &info.rows.len()), ("broken", &orfas)],
        )
    } else {
        tr_with(
            "panel.inspector.counter_watch.title_count",
            &[("n", &info.rows.len())],
        )
    };
    let header = section_header(store, core_ids::INSP_LIVE_WATCH_SECTION, &titulo).color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect) {
        hit_index.register(color_id, circle);
    }
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_WATCH_SECTION,
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

    if info.selected_count > 1 {
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.counter_watch.multiple_selected_edits_apply"),
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Warn, theme),
        );
        cur_y += font + ph2d_tokens::control_gap_px();
    }

    if info.rows.is_empty() {
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.counter_watch.no_rules_yet"),
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        cur_y += font + ph2d_tokens::control_gap_px();
    } else {
        cur_y = lista(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            info,
            escolhida,
        );
    }
    cur_y = botoes(
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
    if let Some(row) = info.rows.get(escolhida) {
        cur_y = editor(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            info,
            row,
        );
    }
    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}
