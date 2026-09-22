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
use ph2d_i18n::tr;
use ph2d_i18n::tr_with;

const BTN_H: f32 = 30.0; // LITERAL-PX-OK: altura de botão do Inspector, igual à das irmãs
/// A linha de uma lista é a linha do app — pela porta, nunca por um literal que coincide.
const ROW_H: f32 = ph2d_tokens::ROW_H_PX;

/// **O que esta linha FAZ, numa frase** — `batida → Toggle Visibility · Piscante`.
fn summary(row: &InspectorActionRow, verbs: &[String]) -> String {
    let verbo = verbs.get(row.verb_tag as usize).map_or("?", String::as_str);
    let alvo = if row.target_is_self() {
        tr("panel.inspector.actions.this_object")
    } else {
        row.target.as_str()
    };
    if row.never_fires() {
        tr_with(
            "panel.inspector.actions.never_fires",
            &[("verbo", &verbo), ("alvo", &alvo)],
        )
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
        .zip(crate::ids::INSP_ACTION_ROW.iter())
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
    let can_add = info.rows.len() < crate::ids::INSP_ACTION_ROW.len();
    let can_remove = !info.rows.is_empty();
    let n = usize::from(can_add) + usize::from(can_remove);
    if n == 0 {
        return y;
    }
    // ⭐ A fileira mede as PALAVRAS, e só as que de facto vão ser pintadas: com `+` escondido no
    //    tecto, a fileira tem UMA peça e o rótulo dela é o único a medir.
    let rotulos: Vec<&str> = [
        (can_add, tr("panel.inspector.actions.plus_add_action")),
        (can_remove, tr("panel.inspector.actions.x_remove_action")),
    ]
    .into_iter()
    .filter_map(|(ativo, l)| ativo.then_some(l))
    .collect();
    let seg = ph2d_editor_core::widget::segment_rects_for(
        Rect::new(x, y, w, BTN_H),
        &rotulos,
        ph2d_editor_core::widget::button_label_font(),
        text_system,
    );
    let mut cell = 0usize;
    if can_add {
        let (rect, group) = seg[cell];
        cell += 1;
        hit_index.register(ids::INSP_ACTION_ADD, rect);
        paint_button(
            &Button::new(
                ids::INSP_ACTION_ADD,
                tr("panel.inspector.actions.plus_add_action"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_ACTION_ADD))
            .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    if can_remove {
        let (rect, group) = seg[cell];
        hit_index.register(ids::INSP_ACTION_REMOVE, rect);
        paint_button(
            &Button::new(
                ids::INSP_ACTION_REMOVE,
                tr("panel.inspector.actions.x_remove_action"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_ACTION_REMOVE))
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
    crate::ids::INSP_ACTION_VERB
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
/// O `segment_rects_for` reparte a largura do painel pelas `N` palavras, e esta secção existe para CRESCER — o
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
    hit_index.register(ids::INSP_ACTION_VERB_PICK, rect);
    let open = matches!(
        store.get(ids::INSP_ACTION_VERB_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let mut dd = Dropdown::new(ids::INSP_ACTION_VERB_PICK, "", verb_options(labels))
        .open(open)
        .visual(store.dropdown_visual(ids::INSP_ACTION_VERB_PICK));
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

/// ⭐⭐⭐ **A QUEM esta acção acerta** (TOP-20 #9, W3b) — o segmentado `Name | Tag` e, por baixo, o
/// controlo do modo escolhido. Devolve o `y` seguinte.
///
/// # ⚠️ Um modo, UM controlo
///
/// O campo do nome e a caixa da tag **nunca aparecem os dois**: com os dois à vista, *«a quem?»*
/// teria duas respostas escritas ao mesmo tempo e o artista não saberia qual manda — que é
/// exactamente o que o doc do `SignalTarget` recusa no modelo. O segmentado escolhe; o outro sai.
///
/// # ⛔ E as duas formas de «não acerta em ninguém» são DITAS, cada uma com a sua frase
///
/// | estado | o que se passa | o que o artista faz |
/// |---|---|---|
/// | por tag, **sem tag escolhida** | a linha está por acabar | escolher uma na caixa |
/// | por tag, **tag apagada** | alguém apagou a tag no painel *Tags* | escolher outra |
///
/// *As duas alcançam ninguém e são histórias diferentes; uma frase só para as duas mandaria o
/// artista procurar a tag que ele nunca escolheu.*
#[allow(clippy::too_many_arguments)]
fn target_rows(
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
    // ⚠️ **A coluna do nome é da SECÇÃO** — desde 2026-09-22 as linhas de TEXTO também têm nome
    //    (report do dono), logo ela mede-se sobre eles.
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[tr("panel.inspector.actions.target_label")],
    );
    let por_tag = row.target_is_tag();
    let (seg_w, seg_dot) = ph2d_editor_core::widget::form_row_columns(x, w, y, ROW_H);
    let seg_h = paint_segmented_group_adaptive(
        Rect::new(x, y, seg_w, ROW_H),
        &[
            (
                tr("panel.inspector.actions.name"),
                !por_tag && !row.target_is_other(),
                ids::INSP_ACTION_BY_NAME,
            ),
            (
                tr("panel.inspector.actions.tag"),
                por_tag,
                ids::INSP_ACTION_BY_TAG,
            ),
            // ⭐⭐⭐ **Quem BATEU** (suplente #24) — o outro lado do contacto que publicou o sinal.
            (
                tr("panel.inspector.actions.who_hit"),
                row.target_is_other(),
                ids::INSP_ACTION_BY_OTHER,
            ),
        ],
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, seg_dot);
    // ⚠️ **Pela PORTA, como a cauda do irmão `from_row`** — ver o comentário de lá. Este sítio era
    // invisível ao censo por ser uma INSTRUÇÃO e não uma cauda (*um censo que conhece uma forma da
    // mesma pergunta é cego às outras*, que é o que aquele ficheiro narra sobre si mesmo).
    let mut cur_y = y + seg_h + ph2d_tokens::control_gap_px();

    // ⭐⭐ **O modo «quem bateu» NÃO tem controlo por baixo, e isso é a lei do modo**: o alvo sai do
    // sinal e não de nada que o artista escreva. *Um campo aqui seria um controlo morto.*
    if row.target_is_other() {
        let font = TypeToken::Sm.px();
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.actions.who_hit_hint"),
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        return cur_y + font + ph2d_tokens::control_gap_px();
    }

    if !por_tag {
        return super::anim_rows::text_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.actions.target_label"),
            ids::INSP_ACTION_TARGET,
            TextInput::new(ids::INSP_ACTION_TARGET, "")
                .placeholder(tr("panel.inspector.actions.target_empty_this_object")),
            seccao,
        );
    }

    cur_y = tag_pick_row(
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
    let font = TypeToken::Sm.px();
    let (aviso, cor) = if row.target_tag_unset() {
        (
            tr("panel.inspector.actions.no_tag_chosen_u_this_action_reaches_nobody"),
            ColorToken::Text3,
        )
    } else if row.target_tag_missing() {
        (
            tr("panel.inspector.actions.that_tag_was_deleted_u_this_action_reaches_nobody"),
            ColorToken::Warn,
        )
    } else {
        return cur_y;
    };
    paint_text(
        text_system,
        scene,
        aviso,
        x,
        cur_y,
        font,
        w,
        resolve(cor, theme),
    );
    cur_y + font + ph2d_tokens::control_gap_px()
}

/// A caixa de escolha da tag alvo. Devolve o `y` seguinte.
///
/// ⚠️ **Ela oferece a árvore INTEIRA**, ao contrário da secção *Tags* — ali a lista tira as que o
/// objecto já tem (escolhê-las seria um gesto recusado); aqui uma acção pode apontar a qualquer
/// tag, incluindo uma que o próprio objecto carregue.
#[allow(clippy::too_many_arguments)]
fn tag_pick_row(
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
    let (control_w, dot) = ph2d_editor_core::widget::form_row_columns(x, w, y, ROW_H);
    let rect = Rect::new(x, y, control_w, ROW_H);
    hit_index.register(ids::INSP_ACTION_TAG_PICK, rect);
    let open = matches!(
        store.get(ids::INSP_ACTION_TAG_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let mut dd = Dropdown::new(ids::INSP_ACTION_TAG_PICK, "", tag_options())
        .placeholder(ph2d_i18n::tr("panel.tags.pick"))
        .open(open)
        .visual(store.dropdown_visual(ids::INSP_ACTION_TAG_PICK));
    if let Some(t) = row.target_tag.filter(|t| *t != 0) {
        dd.select(t);
    }
    paint_dropdown_chip(&dd, rect, scene, text_system, theme);
    if open {
        crate::state_popovers::set_pending_action_tag_dd(Some(rect));
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    y + ph2d_tokens::row_pitch_px()
}

/// **As opções: a árvore inteira do projecto**, indentada pela profundidade.
///
/// ⚠️ `pub(crate)` porque o passe diferido a re-deriva — a lei do popover.
pub(crate) fn tag_options() -> Vec<DropdownOption<u64>> {
    crate::state::current_tag_tree()
        .iter()
        .zip(ids::INSP_ACTION_TAG_OPT.iter())
        .map(|(row, &id)| {
            let recuo = "    ".repeat(row.depth);
            DropdownOption::new(id, row.id, format!("{recuo}{}", row.label))
        })
        .collect()
}

#[path = "actions_editor.rs"]
mod irmao;
pub(crate) use irmao::*;
