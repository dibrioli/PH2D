//! ⭐⭐⭐ **A secção TIMERS** — o painel do primeiro relógio autorável do produto (TOP-20 #2, W3).
//!
//! # Porque ela existe, e o que a ausência dela custava
//!
//! O `Timers` shipou anexável pela paleta e **sem uma linha de edição**: a duração, o nome do sinal
//! e o `autostart` só existiam por código. Medido no smoke de 2026-09-08, o dono anexou-o e **nada
//! apareceu no Inspector** — que é indistinguível de o componente não ter sido anexado. *Um
//! componente anexável sem painel lê-se como um componente partido.*
//!
//! # ⚠️ A lista escolhe, e UM editor mostra os campos — nunca cinco controlos por linha
//!
//! Um `Timers` guarda até [`ph2d_ecs::TIMERS_MAX`] timers de cinco campos. Cinco controlos por
//! linha custariam 80 ids e uma coluna que não cabe na largura do Inspector. ⇒ o molde é o da §11.
//!
//! ⚠️ **Mas a linha aberta NÃO vai ao barramento**, e aqui o precedente certo é a §12: qual timer
//! se edita é um facto da UI. Na §11 a linha aberta **é** a animação que toca — estado da cena. Um
//! `Timers` não tem «o timer actual»: os N correm todos ao mesmo tempo.
//!
//! # ⛔ Não há «+ Add Timers component» aqui
//!
//! É o ADR-0166: *o Inspector mostra o que o objecto TEM, e um componente anexa-se pela paleta*.
//! Um segundo caminho de anexação seria a segunda resposta à mesma pergunta — e a que o artista
//! encontra primeiro é a que envelhece. ⇒ a shell só publica este snapshot para quem tem `Timers`.
//!
//! # ⚠️ O que esta secção NÃO mostra
//!
//! O relógio a andar. Ele vive no `TimerRuntime`, que não é componente registado (é isso que
//! impede um passo de undo por quadro) — e um painel que o mostrasse repintaria a 60 Hz e
//! convidaria o pedido seguinte: poder mexer nele.

use super::*;
use ph2d_editor_core::screens::hero::{InspectorTimerInfo, InspectorTimerRow};
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::tr;
use ph2d_i18n::tr_with;

const BTN_H: f32 = 30.0; // LITERAL-PX-OK: altura de botão do Inspector, igual à da §11
const CHECK_H: f32 = 18.0; // LITERAL-PX-OK: altura visual do Checkbox, igual à da §11
/// A linha de uma lista é a linha do app — pela porta, nunca por um literal que coincide.
const ROW_H: f32 = ph2d_tokens::ROW_H_PX;

/// **O que esta linha FAZ, numa frase** — a mesma ideia do resumo `2-5 · Forward` da §11.
///
/// ⚠️ Ele diz o **período**, se **repete** e para onde **fala**, porque são exactamente as três
/// perguntas que o artista tem com a lista fechada. E diz *«mute»* em vez de deixar o espaço em
/// branco: um timer sem nome de sinal cumpre o período e cala-se, e isso é uma escolha legítima
/// que tem de se ler como escolha.
fn summary(row: &InspectorTimerRow) -> String {
    let repeat = if row.repeat {
        tr("panel.inspector.timers.repeats")
    } else {
        tr("panel.inspector.timers.once")
    };
    if row.is_mute() {
        tr_with(
            "panel.inspector.timers.summary_mute",
            &[
                ("s", &format!("{:.2}", row.duration_s)),
                ("repeat", &repeat),
            ],
        )
    } else {
        format!(
            "{:.2}s \u{b7} {repeat} \u{b7} \u{2192} {}",
            row.duration_s, row.signal
        )
    }
}

/// A lista dos timers. Devolve o `y` seguinte.
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
    info: &InspectorTimerInfo,
    selected: usize,
) -> f32 {
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    // ⚠️ **`zip` com o array de ids**: uma lista com mais timers do que ids (impossível enquanto o
    // gate `the_timer_row_ids_cover_the_model_cap` viver) perde os excedentes em vez de os pintar
    // uns sobre os outros.
    for (i, (row, &id)) in info
        .rows
        .iter()
        .zip(crate::ids::INSP_TIMER_ROW.iter())
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
        // ⚠️ **O `Hovered` está aqui de propósito, e a §11 não o tem.** Uma linha que só acende ao
        // ser escolhida não diz que é clicável — e o defeito lê-se exactamente como um controlo
        // morto sob o dedo, que é a família que a caça de 30/08 mediu.
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
        // ⚠️ **Um timer que nunca dispara escreve-se em WARN**, e a razão é o report que abriu esta
        // wave: *«nada acontece»* é o sintoma mais caro deste componente, e as duas causas
        // autoráveis dele (duração `0`, `autostart` desligado) são invisíveis num resumo.
        let color = if row.will_ever_fire() {
            if i == selected {
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
            &row.name,
            x + Spacing::Sm.px(),
            cur_y + (ROW_H - font) * 0.5,
            font,
            w,
            color,
        );
        let sum = summary(row);
        let sum_x = x + w * 0.5;
        paint_text(
            text_system,
            scene,
            &sum,
            sum_x,
            cur_y + (ROW_H - font) * 0.5,
            font,
            w * 0.5 - Spacing::Sm.px(),
            resolve(ColorToken::Text3, theme),
        );
        cur_y += ROW_H;
    }
    // ⚠️ **A cauda da lista vem da PORTA** — o que separa dois controlos de uma secção é uma
    // pergunta que a casa já respondeu, e escrevê-la aqui seria a segunda resposta (há gate).
    cur_y + ph2d_tokens::control_gap_px()
}

/// Os dois botões da lista, lado a lado. Devolve o `y` seguinte.
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
    info: &InspectorTimerInfo,
) -> f32 {
    // ⚠️ **O `+` desaparece no tecto**, e não fica cinzento a mentir: o modelo não aceita mais, e
    // um botão que aceita o clique e não faz nada é o defeito que a DIRETIVA §2 nomeia.
    let can_add = info.rows.len() < crate::ids::INSP_TIMER_ROW.len();
    let can_remove = !info.rows.is_empty();
    // ⭐⭐ **`+ Add | x Remove` é UM par**, e por isso passa pela porta do grupo — a mesma pergunta
    // (*o que fazer com a lista*), a mesma fileira, o traço de um pixel entre as duas peças.
    // ⛔ Dispô-las à mão com um vão pintaria duas peças separadas onde a casa junta, e há gate
    // (`no_panel_lays_a_button_row_out_by_hand`).
    let n = usize::from(can_add) + usize::from(can_remove);
    if n == 0 {
        return y;
    }
    let seg = ph2d_editor_core::widget::segment_rects(Rect::new(x, y, w, BTN_H), n);
    let mut cell = 0usize;
    if can_add {
        let (rect, group) = seg[cell];
        cell += 1;
        hit_index.register(ids::INSP_TIMER_ADD, rect);
        paint_button(
            &Button::new(
                ids::INSP_TIMER_ADD,
                tr("panel.inspector.timers.plus_add_timer"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_TIMER_ADD))
            .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    if can_remove {
        let (rect, group) = seg[cell];
        hit_index.register(ids::INSP_TIMER_REMOVE, rect);
        paint_button(
            &Button::new(
                ids::INSP_TIMER_REMOVE,
                tr("panel.inspector.timers.x_remove_timer"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_TIMER_REMOVE))
            .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    y + BTN_H + ph2d_tokens::control_gap_px()
}

/// O editor do timer aberto. Devolve o `y` seguinte.
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
    row: &InspectorTimerRow,
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
        ids::INSP_TIMER_NAME,
        TextInput::new(ids::INSP_TIMER_NAME, "")
            .placeholder(tr("panel.inspector.timers.timer_name")),
    );

    // ⚠️ **SEGUNDOS, e o passo é 0,1** — a unidade do artista. O componente guarda microssegundos
    // porque o tique é de passo fixo; a conversão vive nas duas pontas do canal e em mais lado
    // nenhum.
    cur_y = super::anchors::field_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.timers.duration_seconds"),
        &[ids::INSP_TIMER_DURATION],
        0.1, // LITERAL-PX-OK: passo de scrub em SEGUNDOS, não em pixels
    );

    let half = (w - Spacing::Sm.px()) * 0.5;
    for (i, (id, label, on)) in [
        (
            ids::INSP_TIMER_REPEAT,
            tr("panel.inspector.timers.repeat"),
            row.repeat,
        ),
        (
            ids::INSP_TIMER_AUTOSTART,
            tr("panel.inspector.timers.autostart"),
            row.autostart,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let rect = Rect::new(
            x + (half + Spacing::Sm.px()) * i as f32,
            cur_y,
            half,
            CHECK_H,
        );
        hit_index.register(id, rect);
        // ⚠️ **O valor vem do SNAPSHOT**, nunca do store: ler dali faria a caixa sobreviver à troca
        // de objecto — a lei que a §11 escreveu para o `Playing`.
        paint_checkbox(
            &Checkbox::new(id, label)
                .visual(store.checkbox_visual(id))
                .value(if on {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                }),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    cur_y += CHECK_H + ph2d_tokens::control_gap_px();

    cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        ids::INSP_TIMER_SIGNAL,
        TextInput::new(ids::INSP_TIMER_SIGNAL, "")
            .placeholder(tr("panel.inspector.timers.signal_name_empty_mute")),
    );

    // ⚠️⚠️ **A LINHA QUE RESPONDE AO «nada acontece».** As três causas autoráveis do silêncio são
    // invisíveis a olho — e cada uma delas já produziu um report. Ela nomeia a que está em vigor.
    let aviso = if row.duration_s <= 0.0 {
        Some(tr("panel.inspector.timers.this_timer_never_fires_duration"))
    } else if !row.autostart {
        Some(tr(
            "panel.inspector.timers.this_timer_never_starts_autostart",
        ))
    } else if row.is_mute() {
        Some(tr("panel.inspector.timers.this_timer_runs_but_says"))
    } else {
        None
    };
    if let Some(msg) = aviso {
        let font = TypeToken::Sm.px();
        paint_text(
            text_system,
            scene,
            msg,
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
pub(crate) fn paint_timer_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorTimerInfo,
    selected: usize,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let color_id = core_ids::INSP_LIVE_TIMER_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: acento neutro por omissão
    let title = if info.rows.is_empty() {
        String::from(tr("panel.inspector.timers.timers"))
    } else {
        tr_with(
            "panel.inspector.timers.title_count",
            &[("n", &info.rows.len())],
        )
    };
    let header = section_header(store, core_ids::INSP_LIVE_TIMER_SECTION, &title).color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_TIMER_SECTION,
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

    // ⚠️ **A SELEÇÃO MÚLTIPLA tem de se dizer** — a lei da §11, e pela mesma razão: o índice que
    // uma edição carrega só significa alguma coisa na lista da primária. Vem antes de tudo,
    // porque *«em quem é que isto pega?»* é anterior a qualquer controlo.
    if info.selected_count > 1 {
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.timers.multiple_selected_timer_edits_apply"),
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
            tr("panel.inspector.timers.no_timers_yet"),
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
        );
    }
    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}
