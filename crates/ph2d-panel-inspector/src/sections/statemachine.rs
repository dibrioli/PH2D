//! ⭐⭐⭐ **A secção STATE MACHINE** — onde o artista desenha o cérebro de um objecto (TOP-20 #15, W3).
//!
//! # A forma é a da secção SIGNAL ACTIONS, e de propósito
//!
//! **Duas listas, cada uma com `+ Add` / `x Remove` e UM editor para a linha aberta.** Três campos
//! por estado desenhados em cada linha custariam `3 × 16 = 48` ids numa coluna que mostra ~30
//! linhas; com as setas, `3 × 32 = 96` a mais. ⚠️ E a linha aberta **não vai ao barramento**: qual
//! estado se edita é um facto da UI, e publicá-lo faria um passo de undo por clique.
//!
//! # ⭐⭐ O que o painel DIZ que as listas sozinhas não diriam
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `Now: <estado>` | ⭐ **o estado CORRENTE** — sem ele o artista tem de simular a tabela de cabeça |
//! | `No states yet.` | a máquina está vazia e não pensa |
//! | `never fires` | a seta não tem nome de sinal — a causa nº 1 de *«não acontece nada»* |
//! | `dead end` | nenhuma seta SAI deste estado: quem lá entra fica |
//! | `The clock is stopped` | a ponte corre no dreno de sinais, e ele só anda a tocar |
//!
//! ⛔ **O estado corrente é só de LEITURA.** Ele é vivo; deixá-lo editar seria a segunda porta para
//! o que a ponte decide, e ela mentiria exactamente no caso em que a ponte recusasse.

use super::*;
use ph2d_editor_core::statemachine_edits::{
    InspectorStateMachineInfo, InspectorStateRow, InspectorTransitionRow,
};
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::{tr, tr_with};

const BTN_H: f32 = 30.0; // LITERAL-PX-OK: altura de botão do Inspector, igual à das irmãs
/// A linha de uma lista é a linha do app — pela porta, nunca por um literal que coincide.
const ROW_H: f32 = ph2d_tokens::ROW_H_PX;

/// **O que um estado É, numa linha** — `Fechada  →  porta_fechada`.
fn resumo_estado(r: &InspectorStateRow, i: usize) -> String {
    let nome = if r.name.is_empty() {
        tr_with("panel.inspector.statemachine.state_n", &[("i", &i)])
    } else {
        r.name.clone()
    };
    if r.on_enter.is_empty() {
        nome
    } else {
        format!("{nome}  \u{2192}  {}", r.on_enter)
    }
}

/// **O que uma seta FAZ, numa linha** — `0 \u{2014}botao\u{2192} 1`.
fn resumo_seta(t: &InspectorTransitionRow, estados: &[InspectorStateRow]) -> String {
    let nome = |i: u8| -> String {
        estados
            .get(i as usize)
            .filter(|s| !s.name.is_empty())
            .map_or_else(|| format!("#{i}"), |s| s.name.clone())
    };
    let sinal = if t.on.is_empty() {
        String::from(tr("panel.inspector.statemachine.never_fires"))
    } else {
        t.on.clone()
    };
    format!("{}  \u{2014}{sinal}\u{2192}  {}", nome(t.from), nome(t.to))
}

/// Uma lista genérica de linhas — as duas usam-na, porque são a mesma coisa.
///
/// ⚠️ **Uma função e não duas**, pela lei das portas: duas cópias divergiriam no dia em que o
/// realce ou o zebrado mudasse, e a diferença só apareceria numa screenshot.
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
    linhas: &[(String, bool)],
    ids_linha: &[ph2d_a11y::NodeId],
    selected: usize,
) -> f32 {
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    for (i, ((texto, avisa), &id)) in linhas.iter().zip(ids_linha.iter()).enumerate() {
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
        let color = if *avisa {
            resolve(ColorToken::Warn, theme)
        } else if i == selected {
            resolve(ColorToken::Text1, theme)
        } else {
            resolve(ColorToken::Text2, theme)
        };
        paint_text(
            text_system,
            scene,
            texto,
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

/// Os dois botões de uma lista, **pela porta do grupo**.
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
    add: (ph2d_a11y::NodeId, &str),
    remove: (ph2d_a11y::NodeId, &str),
    pode_add: bool,
    pode_remover: bool,
) -> f32 {
    let n = usize::from(pode_add) + usize::from(pode_remover);
    if n == 0 {
        return y;
    }
    // ⭐ A fileira mede as PALAVRAS, e só as que de facto vão ser pintadas.
    //    ⛔⛔ Medido em 2026-09-19: em partes iguais `x Remove Transition` recebia `118 px` e saía
    //    `x Remove Transi…` com a fileira a caber inteira — *uma média não é um máximo*.
    let rotulos: Vec<&str> = [(pode_add, add.1), (pode_remover, remove.1)]
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
    for (ativo, (id, rotulo)) in [(pode_add, add), (pode_remover, remove)] {
        if !ativo {
            continue;
        }
        let (rect, group) = seg[cell];
        cell += 1;
        hit_index.register(id, rect);
        paint_button(
            &Button::new(id, rotulo)
                .kind(ButtonKind::Default)
                .visual(store.button_visual(id))
                .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    y + BTN_H + ph2d_tokens::control_gap_px()
}

/// A secção inteira — cabeçalho, dobra e corpo. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_statemachine_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorStateMachineInfo,
    state_sel: usize,
    trans_sel: usize,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let color_id = core_ids::INSP_LIVE_SM_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: acento neutro por omissão
    let header = section_header(
        store,
        core_ids::INSP_LIVE_SM_SECTION,
        tr("panel.inspector.statemachine.state_machine"),
    )
    .color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_SM_SECTION,
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

    let nota = |scene: &mut VectorScene,
                text_system: &mut TextSystem,
                cur_y: &mut f32,
                texto: &str,
                token: ColorToken| {
        paint_text(
            text_system,
            scene,
            texto,
            x,
            *cur_y,
            font,
            w,
            resolve(token, theme),
        );
        *cur_y += font + ph2d_tokens::control_gap_px();
    };

    if info.selected_count > 1 {
        nota(
            scene,
            text_system,
            &mut cur_y,
            tr(
                "panel.inspector.statemachine.multiple_selected_u_edits_apply_to_the_active_object_only",
            ),
            ColorToken::Warn,
        );
    }
    // ⭐⭐⭐ **O ESTADO CORRENTE** — a razão de esta secção existir com o relógio a andar.
    if let Some(c) = info.current {
        let nome = info
            .states
            .get(c as usize)
            .filter(|s| !s.name.is_empty())
            .map_or_else(|| format!("#{c}"), |s| s.name.clone());
        nota(
            scene,
            text_system,
            &mut cur_y,
            &tr_with("panel.inspector.statemachine.now", &[("nome", &nome)]),
            ColorToken::Accent,
        );
    }
    if !info.clock_playing {
        nota(
            scene,
            text_system,
            &mut cur_y,
            tr(
                "panel.inspector.statemachine.the_clock_is_stopped_u_it_only_thinks_while_the_scene_plays",
            ),
            ColorToken::Text3,
        );
    }

    cur_y = estados(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
        state_sel,
    );
    cur_y = setas(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
        trans_sel,
    );
    // ── O INICIAL ────────────────────────────────────────────────────────────
    // ⭐⭐ **A coluna do nome é da SECÇÃO** (`line/UIUX`, 2026-09-15): esta secção nasceu
    //    contra a porta antiga (`anchors::field_row`, o nome POR CIMA do campo) e passa à
    //    única que existe. ⚠️ Os nomes são os da secção INTEIRA, inclusive os das linhas que
    //    este quadro não pinta.
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[tr("panel.inspector.statemachine.initial_state")],
    );
    if !info.states.is_empty() {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.statemachine.initial_state"),
            &[crate::ids::INSP_SM_INITIAL],
            1.0, // LITERAL-PX-OK: índice
            None,
            seccao,
        );
    }
    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}

/// **A lista de ESTADOS, os botões e o editor da linha aberta** — irmã da de baixo pelo tecto de 200 LOC por função, cortada por RESPONSABILIDADE (uma lista, um assunto).
#[allow(clippy::too_many_arguments)]
fn estados(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorStateMachineInfo,
    selected: usize,
) -> f32 {
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    let nota = |scene: &mut VectorScene,
                text_system: &mut TextSystem,
                cur_y: &mut f32,
                texto: &str,
                token: ColorToken| {
        paint_text(
            text_system,
            scene,
            texto,
            x,
            *cur_y,
            font,
            w,
            resolve(token, theme),
        );
        *cur_y += font + ph2d_tokens::control_gap_px();
    };
    // ── ESTADOS ──────────────────────────────────────────────────────────────
    if info.states.is_empty() {
        nota(
            scene,
            text_system,
            &mut cur_y,
            tr("panel.inspector.statemachine.no_states_yet"),
            ColorToken::Text3,
        );
    } else {
        let linhas: Vec<(String, bool)> = info
            .states
            .iter()
            .enumerate()
            .map(|(i, r)| (resumo_estado(r, i), !r.has_exit))
            .collect();
        cur_y = lista(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            &linhas,
            &crate::ids::INSP_SM_STATE_ROW,
            selected,
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
        (
            crate::ids::INSP_SM_STATE_ADD,
            tr("panel.inspector.statemachine.add_state"),
        ),
        (
            crate::ids::INSP_SM_STATE_REMOVE,
            tr("panel.inspector.statemachine.x_remove_state"),
        ),
        info.states.len() < crate::ids::INSP_SM_STATE_ROW.len(),
        !info.states.is_empty(),
    );
    if let Some(r) = info.states.get(selected) {
        // ⚠️⚠️ **Os três `placeholder` são LITERAIS e não um laço sobre um array**, e a diferença
        // não é estilo: a 1.ª redacção passava-os por variável, e o censo do HR-15 contou **1** de
        // **4** — *passar a string por um argumento faz a contagem cair sem tirar o literal do
        // binário*, que é a isenção silenciosa que aquele gate proíbe por escrito. Eles ficam
        // VISÍVEIS, na baseline, e caem todos quando o `t!(…)` shipar.
        cur_y = super::anim_rows::text_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            crate::ids::INSP_SM_STATE_NAME,
            TextInput::new(crate::ids::INSP_SM_STATE_NAME, "")
                .placeholder(tr("panel.inspector.statemachine.state_name_u")),
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
            crate::ids::INSP_SM_STATE_ON_ENTER,
            TextInput::new(crate::ids::INSP_SM_STATE_ON_ENTER, "")
                .placeholder(tr("panel.inspector.statemachine.on_enter_signal_name")),
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
            crate::ids::INSP_SM_STATE_ON_EXIT,
            TextInput::new(crate::ids::INSP_SM_STATE_ON_EXIT, "")
                .placeholder(tr("panel.inspector.statemachine.on_exit_signal_name")),
        );
        // ⚠️ **Um beco escreve-se em WARN** — quem lá entra fica, e isso é invisível numa lista.
        if !r.has_exit {
            nota(
                scene,
                text_system,
                &mut cur_y,
                tr("panel.inspector.statemachine.dead_end_no_transition_leaves_this_state"),
                ColorToken::Warn,
            );
        }
    }

    cur_y
}

#[path = "statemachine_setas.rs"]
mod irmao;
use irmao::*;
