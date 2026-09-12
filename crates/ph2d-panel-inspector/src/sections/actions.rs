//! ⭐⭐⭐ **A secção SIGNAL ACTIONS** — onde o artista escreve *«quando `X` chegar, faz `Y` em `Z`»*
//! (TOP-20 #5, W3).
//!
//! # ⚠️ Ela nasce COM painel, e isso é a lição da wave anterior
//!
//! O `Timers` shipou anexável e sem linha de edição, e o report do dono foi *«timer sumiu do modal
//! de componente»* — o componente **estava** na paleta; o que não existia era o que acontece depois
//! de o anexar. *Um componente anexável sem painel é indistinguível de um que não foi anexado.*
//!
//! # A forma é a da secção TIMERS, e de propósito
//!
//! Lista + **um** editor. Quatro campos por linha (sinal · alvo · verbo · parâmetro) desenhados em
//! cada linha custariam `4 × 16 = 64` ids e uma coluna que não cabe. ⚠️ E a linha aberta **não vai
//! ao barramento**: qual acção se edita é um facto da UI.
//!
//! # ⚠️ O que o painel DIZ que a lista sozinha não diria
//!
//! - **`never fires`**, em WARN, quando o nome do sinal está vazio. É a causa nº 1 de *«não
//!   acontece nada»*, e ela é invisível numa lista que mostra o verbo.
//! - **`(this object)`** quando o alvo é vazio — o caso comum, escrito como escolha e não como
//!   espaço em branco.
//! - **O campo do parâmetro só aparece no verbo que o LÊ** (`uses_arg`, derivado do verbo na
//!   shell): mostrá-lo sempre seria um controlo morto em três dos cinco verbos.

use super::*;
use ph2d_editor_core::screens::hero::{InspectorActionInfo, InspectorActionRow};
use ph2d_editor_core::widget::SectionFold;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};

const BTN_H: f32 = 30.0; // LITERAL-PX-OK: altura de botão do Inspector, igual à das irmãs
/// A linha de uma lista é a linha do app — pela porta, nunca por um literal que coincide.
const ROW_H: f32 = ph2d_tokens::ROW_H_PX;

/// **O que esta linha FAZ, numa frase** — `batida → Toggle Visibility · Piscante`.
fn summary(row: &InspectorActionRow, verbs: &[String]) -> String {
    let verbo = verbs.get(row.verb_tag as usize).map_or("?", String::as_str);
    let alvo = if row.target_is_self() {
        "(this object)"
    } else {
        row.target.as_str()
    };
    if row.never_fires() {
        format!("never fires \u{b7} {verbo} \u{b7} {alvo}")
    } else {
        format!("{} \u{2192} {verbo} \u{b7} {alvo}", row.on)
    }
}

/// A lista das acções. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn list(
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
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    for (i, (row, &id)) in info
        .rows
        .iter()
        .zip(ids::INSP_ACTION_ROW.iter())
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
            if i == selected {
                ph2d_editor_core::widget::RowHighlight::Selected
            } else if store.hover_live(id) > 0.0 {
                ph2d_editor_core::widget::RowHighlight::Hovered
            } else {
                ph2d_editor_core::widget::RowHighlight::None
            },
            0.0,
        );
        // ⚠️ **Uma linha que nunca dispara escreve-se em WARN** — a causa nº 1 de «não acontece
        // nada», e invisível num resumo que mostre só o verbo.
        let color = if row.never_fires() {
            resolve(ColorToken::Warn, theme)
        } else if i == selected {
            resolve(ColorToken::Text1, theme)
        } else {
            resolve(ColorToken::Text2, theme)
        };
        paint_text(
            text_system,
            scene,
            &summary(row, &info.verb_labels),
            x + Spacing::Sm.px(),
            cur_y + (ROW_H - font) * 0.5,
            font,
            w - Spacing::Sm.px(),
            color,
        );
        cur_y += ROW_H;
    }
    cur_y + ph2d_tokens::control_gap_px()
}

/// Os dois botões da lista, **pela porta do grupo**. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn buttons(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorActionInfo,
) -> f32 {
    let can_add = info.rows.len() < ids::INSP_ACTION_ROW.len();
    let can_remove = !info.rows.is_empty();
    let n = usize::from(can_add) + usize::from(can_remove);
    if n == 0 {
        return y;
    }
    let seg = ph2d_editor_core::widget::segment_rects(Rect::new(x, y, w, BTN_H), n);
    let mut cell = 0usize;
    if can_add {
        let (rect, group) = seg[cell];
        cell += 1;
        hit_index.register(crate::ids::INSP_ACTION_ADD, rect);
        paint_button(
            &Button::new(crate::ids::INSP_ACTION_ADD, "+ Add Action")
                .kind(ButtonKind::Default)
                .visual(store.button_visual(crate::ids::INSP_ACTION_ADD))
                .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    if can_remove {
        let (rect, group) = seg[cell];
        hit_index.register(crate::ids::INSP_ACTION_REMOVE, rect);
        paint_button(
            &Button::new(crate::ids::INSP_ACTION_REMOVE, "x Remove Action")
                .kind(ButtonKind::Default)
                .visual(store.button_visual(crate::ids::INSP_ACTION_REMOVE))
                .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    y + BTN_H + ph2d_tokens::control_gap_px()
}

/// **As opções do seletor do verbo** — uma por entrada de `SignalVerb::ALL`, na ordem dele.
///
/// ⚠️ **O valor da opção é a TAG, e a tag é a posição** — a mesma lei que o array de ids declara e
/// que o despacho lê com `position()`. ⚠️ `zip` com os rótulos do snapshot: uma lista de rótulos
/// mais curta perde as excedentes em vez de as pintar sem nome.
pub(crate) fn verb_options(labels: &[String]) -> Vec<DropdownOption<u8>> {
    ids::INSP_ACTION_VERB
        .iter()
        .enumerate()
        .zip(labels.iter())
        .map(|((i, &id), label)| {
            DropdownOption::new(id, u8::try_from(i).unwrap_or(0), label.clone())
        })
        .collect()
}

/// **O seletor do verbo** — um chip com a lista.
///
/// ⚠️ **Era uma fileira de cinco botões até 2026-09-09** (*«as actions deveriam ficar num dropdown
/// e não em muitos botões»*, report do dono). E a fileira não era só ruidosa: ela **escala mal**.
/// O `segment_rects` reparte a largura do painel por `N`, e esta secção existe para CRESCER — o
/// doc do `SignalVerb` já nomeia os verbos que faltam (som, animação, spawn) —, logo o sexto verbo
/// entregaria rótulos cortados numa coluna estreita. *Um chip mostra UM nome, inteiro.*
///
/// ⚠️ **A escolha vem do SNAPSHOT e o `open` vem do store** — ler a escolha do store faria o chip
/// mostrar o verbo da acção anterior depois de trocar de linha na lista (a lei que a §12 paga).
#[allow(clippy::too_many_arguments)]
fn verb_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    labels: &[String],
    sel: u8,
) -> f32 {
    let (control_w, dot) = ph2d_editor_core::widget::form_row_columns(x, w, y, ROW_H_PX);
    let rect = Rect::new(x, y, control_w, ROW_H_PX);
    hit_index.register(crate::ids::INSP_ACTION_VERB_PICK, rect);
    let open = matches!(
        store.get(crate::ids::INSP_ACTION_VERB_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let mut dd = Dropdown::new(crate::ids::INSP_ACTION_VERB_PICK, "", verb_options(labels))
        .open(open)
        .visual(store.dropdown_visual(crate::ids::INSP_ACTION_VERB_PICK));
    dd.select(sel);
    paint_dropdown_chip(&dd, rect, scene, text_system, theme);
    // ⚠️ **O popover NÃO se pinta aqui** — ele sairia debaixo da secção seguinte. O rect vai ao
    // slot e o passe diferido do painel desenha-o por cima de tudo.
    if open {
        crate::state_popovers::set_pending_action_dd(Some((sel, rect)));
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    y + ph2d_tokens::row_pitch_px()
}

/// O editor da acção aberta. Devolve o `y` seguinte.
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
    row: &InspectorActionRow,
    labels: &[String],
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
        crate::ids::INSP_ACTION_ON,
        TextInput::new(crate::ids::INSP_ACTION_ON, "").placeholder("on signal\u{2026}"),
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
        crate::ids::INSP_ACTION_TARGET,
        TextInput::new(crate::ids::INSP_ACTION_TARGET, "")
            .placeholder("target (empty = this object)"),
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
            crate::ids::INSP_ACTION_ARG,
            TextInput::new(crate::ids::INSP_ACTION_ARG, "").placeholder("timer name (empty = all)"),
        );
    }
    // ⚠️⚠️ **A LINHA QUE RESPONDE AO «não acontece nada».**
    if row.never_fires() {
        let font = TypeToken::Sm.px();
        paint_text(
            text_system,
            scene,
            "This action never runs: it has no signal name.",
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
    let color_id = ids::INSP_LIVE_ACTION_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: acento neutro por omissão
    let title = if info.rows.is_empty() {
        String::from("Signal Actions")
    } else {
        format!("Signal Actions  ({})", info.rows.len())
    };
    let header = section_header(store, ids::INSP_LIVE_ACTION_SECTION, &title).color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    let Some(fold) = SectionFold::begin(
        store,
        ids::INSP_LIVE_ACTION_SECTION,
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
            "Multiple selected \u{b7} action edits apply to the active object only.",
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
            "No actions yet.",
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
