//! ⭐⭐⭐ **O que o Inspector mostra do GATILHO** — as linhas *«quando a tecla A for premida, diz
//! S»* (suplente #24).
//!
//! # ⭐⭐ A lista + UM editor, e o CHIP para a aresta
//!
//! A forma é a da vigia do contador: uma linha por gatilho, e um editor só para a aberta. A aresta
//! é um **chip** e não um campo de texto porque o conjunto dela é **conhecido e tem três
//! elementos** — escrever `press` à mão seria dar ao artista uma maneira de a errar.
//!
//! # ⭐⭐⭐ E o que esta secção DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `No triggers yet` | o objecto tem o componente e não ouve nada |
//! | `There is no action called «x»` | ⭐ o nome não casa com acção nenhuma do Input Map — **a linha nunca dispara**, e sem esta frase ela é indistinguível de uma que funciona |
//! | `This trigger says nothing` | sem nome de sinal ela segue a tecla e cala-se |
//! | `The clock is stopped` | ⭐⭐ sem a cena a tocar **nenhum** gatilho fala, e isso não é uma cerca de segurança: as teclas do jogo são as teclas do editor |
//!
//! ⛔ **Da mais ESPECÍFICA para a mais geral** — a lei da recusa dos pincéis: *dizer «o relógio
//! está parado» a quem escreveu `fier` em vez de `fire` é mandá-lo resolver a metade errada.*
//!
//! ⚠️⚠️ **E a frase da acção órfã é a mais valiosa das quatro**, porque o silêncio dela é DUPLO: a
//! lei cala uma acção que o mapa não conhece (senão um `Release` sobre ela dispararia em todo
//! quadro, porque `!pressed` é trivialmente verdade), e sem esta linha o artista lê esse silêncio
//! como *«o gatilho não funciona»*.

use super::*;
use ph2d_editor_core::action_trigger_edits::{
    InspectorActionTriggerInfo, InspectorTriggerRow, NoMapa,
};
use ph2d_editor_core::widget::{Dropdown, DropdownOption, SectionFold, paint_dropdown_chip};
use ph2d_i18n::{tr, tr_with};

const ROW_H: f32 = ph2d_tokens::ROW_H_PX;

/// **A CHAVE de cada aresta — a PORTA, lida pelo chip E pelo resumo da linha fechada.**
///
/// ⛔ **Ao contrário do símbolo da vigia, isto É língua** (`Press` é uma palavra, `≤` não é), logo
/// ele passa pela tabela de strings — a lei do HR-15, e o censo de dois lados recusaria tanto um
/// literal aqui como uma chave órfã lá.
///
/// ⚠️ **O índice É o `u8` que a edição carrega** — reordenar isto reescreve o sentido de todo
/// gatilho já gravado. Há gate na shell a prender a ordem à do `ph2d_ecs::ActionEdge`.
#[must_use]
pub const fn chave_da_aresta(edge: u8) -> &'static str {
    match edge {
        1 => "panel.inspector.trigger.on_release",
        2 => "panel.inspector.trigger.while_held",
        _ => "panel.inspector.trigger.on_press",
    }
}

/// As opções do chip, **pela ordem do enum do motor**, com a chave da porta acima.
#[must_use]
pub fn opcoes_da_aresta() -> Vec<DropdownOption<usize>> {
    crate::ids::INSP_TRIGGER_EDGE_OPT
        .iter()
        .enumerate()
        .map(|(i, &id)| {
            let rotulo = tr(chave_da_aresta(u8::try_from(i).unwrap_or(0)));
            DropdownOption::new(id, i, rotulo.to_owned())
        })
        .collect()
}

/// **O que esta linha FAZ, numa frase.**
///
/// ⚠️ Ela diz a aresta e para onde fala — as duas perguntas que sobram com a lista fechada (o nome
/// da acção já é a primeira coluna). E diz *«mute»* em vez de deixar o espaço em branco: um gatilho
/// sem sinal é uma escolha legítima que tem de se ler como escolha.
fn resumo(row: &InspectorTriggerRow) -> String {
    let alvo = if row.signal.trim().is_empty() {
        tr("panel.inspector.trigger.mute").to_owned()
    } else {
        format!("\u{2192} {}", row.signal)
    };
    format!("{} {alvo}", tr(chave_da_aresta(row.edge)))
}

/// A lista dos gatilhos. Devolve o `y` seguinte.
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
    info: &InspectorActionTriggerInfo,
    escolhida: usize,
) -> f32 {
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    for (i, (row, &id)) in info
        .rows
        .iter()
        .zip(crate::ids::INSP_TRIGGER_ROW.iter())
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
        // ⚠️⚠️ **Uma linha ÓRFÃ escreve-se em WARN na própria lista**, e não só no editor: com seis
        // gatilhos e um errado, o artista não abre os seis à procura do que não funciona.
        // ⚠️ **Só a DESCONHECIDA pinta WARN na lista.** Uma acção por ligar é uma configuração a
        // meio (como o sinal vazio), e pintá-la de vermelho aqui poria a lista a gritar sobre um
        // gatilho que o artista está a acabar de escrever.
        let cor = if row.no_mapa != NoMapa::Desconhecida {
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
            &row.action,
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
    info: &InspectorActionTriggerInfo,
) -> f32 {
    // ⚠️ **O `+` DESAPARECE no tecto** e não fica cinzento a mentir — a lei das irmãs.
    let pode_juntar = info.rows.len() < crate::ids::INSP_TRIGGER_ROW.len();
    let pode_tirar = !info.rows.is_empty();
    let n = usize::from(pode_juntar) + usize::from(pode_tirar);
    if n == 0 {
        return y;
    }
    // ⭐ **A fileira mede as PALAVRAS, nunca partes iguais** — `segment_rects_for` (a lei que a
    //    `line/UIUX` fechou a ZERO em 2026-09-19). *Uma média não é um máximo: a fileira pode
    //    caber inteira e cortar a peça mais larga na mesma.*
    let mut rotulos: Vec<&str> = Vec::with_capacity(n);
    if pode_juntar {
        rotulos.push(tr("panel.inspector.trigger.plus_add_trigger"));
    }
    if pode_tirar {
        rotulos.push(tr("panel.inspector.trigger.x_remove_trigger"));
    }
    let seg = ph2d_editor_core::widget::segment_rects_for(
        Rect::new(x, y, w, ALTURA_DE_CAMPO),
        &rotulos,
        ph2d_editor_core::widget::button_label_font(),
        text_system,
    );
    let mut cell = 0usize;
    if pode_juntar {
        let (rect, group) = seg[cell];
        cell += 1;
        hit_index.register(ids::INSP_TRIGGER_ADD, rect);
        paint_button(
            &Button::new(
                ids::INSP_TRIGGER_ADD,
                tr("panel.inspector.trigger.plus_add_trigger"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_TRIGGER_ADD))
            .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    if pode_tirar {
        let (rect, group) = seg[cell];
        hit_index.register(ids::INSP_TRIGGER_REMOVE, rect);
        paint_button(
            &Button::new(
                ids::INSP_TRIGGER_REMOVE,
                tr("panel.inspector.trigger.x_remove_trigger"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_TRIGGER_REMOVE))
            .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    y + ALTURA_DE_CAMPO + ph2d_tokens::control_gap_px()
}

/// A linha do chip da aresta. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn linha_da_aresta(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &InspectorTriggerRow,
) -> f32 {
    let rotulo = tr("panel.inspector.trigger.when");
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
    hit_index.register(crate::ids::INSP_TRIGGER_EDGE_PICK, linha.control);
    let aberto = matches!(
        store.get(crate::ids::INSP_TRIGGER_EDGE_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let mut dd = Dropdown::new(crate::ids::INSP_TRIGGER_EDGE_PICK, "", opcoes_da_aresta())
        .open(aberto)
        .visual(store.dropdown_visual(crate::ids::INSP_TRIGGER_EDGE_PICK));
    // ⚠️ **A escolha vem do SNAPSHOT e o `open` vem do store** — ler a escolha do store faria o
    // chip mostrar a aresta da linha ANTERIOR depois de trocar de gatilho.
    dd.select(usize::from(row.edge));
    paint_dropdown_chip(&dd, linha.control, scene, text_system, theme);
    if aberto {
        crate::state_popovers::set_pending_trigger_dd(Some((row.edge, linha.control)));
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, linha.dot);
    y + ph2d_tokens::row_pitch_px()
}

/// O editor do gatilho aberto. Devolve o `y` seguinte.
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
    info: &InspectorActionTriggerInfo,
    row: &InspectorTriggerRow,
) -> f32 {
    // ⚠️ **A coluna do nome é da SECÇÃO** — desde 2026-09-22 as linhas de TEXTO também têm nome
    //    (report do dono), logo ela mede-se sobre eles.
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.trigger.action_label"),
            tr("panel.inspector.trigger.signal_label"),
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
        tr("panel.inspector.trigger.action_label"),
        ids::INSP_TRIGGER_ACTION,
        TextInput::new(ids::INSP_TRIGGER_ACTION, "")
            .placeholder(tr("panel.inspector.trigger.action_name")),
        seccao,
    );
    cur_y = linha_da_aresta(
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
    cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.trigger.signal_label"),
        ids::INSP_TRIGGER_SIGNAL,
        TextInput::new(ids::INSP_TRIGGER_SIGNAL, "")
            .placeholder(tr("panel.inspector.trigger.signal_name_empty_mute")),
        seccao,
    );
    let cur_y = avisos(scene, text_system, theme, x, w, cur_y, info, row);
    // ⭐⭐⭐ **A CURA ao lado da QUEIXA** — e só onde ela aparece.
    botao_criar_a_accao(
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
}

/// ⭐⭐⭐ **O botão que CRIA a acção que falta**, ali mesmo.
///
/// ⚠️⚠️ **Ele só existe no estado `Desconhecida`, e as duas metades são a lei:** *uma queixa que
/// nomeia a cura e não a alcança é meia queixa* (o artista tem de descobrir sozinho a outra janela),
/// e *um botão que oferece criar uma acção que já existe é ruído* — pior, ele criaria uma segunda
/// linha com o mesmo nome. ⛔ No estado `SemTecla` a cura é **outra** (ligar uma tecla), e oferecer
/// este botão ali mandaria o artista resolver a metade errada.
#[allow(clippy::too_many_arguments)]
fn botao_criar_a_accao(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &InspectorTriggerRow,
) -> f32 {
    if row.no_mapa != NoMapa::Desconhecida || row.action.trim().is_empty() {
        return y;
    }
    let rect = Rect::new(x, y, w, BTN_H);
    hit_index.register(ids::INSP_TRIGGER_CREATE_ACTION, rect);
    paint_button(
        &Button::new(
            ids::INSP_TRIGGER_CREATE_ACTION,
            tr("panel.inspector.trigger.create_this_action"),
        )
        .kind(ButtonKind::Default)
        .visual(store.button_visual(ids::INSP_TRIGGER_CREATE_ACTION)),
        rect,
        scene,
        text_system,
        theme,
    );
    y + BTN_H + ph2d_tokens::control_gap_px()
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
    info: &InspectorActionTriggerInfo,
    row: &InspectorTriggerRow,
) -> f32 {
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    let (msg, cor) = if row.action.trim().is_empty() {
        (
            Some(tr("panel.inspector.trigger.no_action_named_yet").to_owned()),
            ColorToken::Warn,
        )
    } else if row.no_mapa == NoMapa::Desconhecida {
        (
            Some(tr_with(
                "panel.inspector.trigger.there_is_no_action_called",
                &[("name", &row.action.trim())],
            )),
            ColorToken::Warn,
        )
    } else if row.no_mapa == NoMapa::SemTecla {
        // ⭐⭐⭐ **O SEGUNDO silêncio, e ele não é um erro:** a acção existe e não tem tecla
        // nenhuma, logo as três leituras dela dão `false` e o gatilho fica exactamente tão calado
        // como com um nome errado. ⚠️ **A cura é OUTRA** (ligar uma tecla, não criar a acção), e é
        // por isso que a frase e a cor são outras.
        (
            Some(tr_with(
                "panel.inspector.trigger.the_action_has_no_key",
                &[("name", &row.action.trim())],
            )),
            ColorToken::Warn,
        )
    } else if row.signal.trim().is_empty() {
        (
            Some(tr("panel.inspector.trigger.this_trigger_says_nothing").to_owned()),
            ColorToken::Warn,
        )
    } else if !info.clock_playing {
        (
            Some(tr("panel.inspector.trigger.the_clock_is_stopped").to_owned()),
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
pub(crate) fn paint_action_trigger_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorActionTriggerInfo,
    escolhida: usize,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let color_id = core_ids::INSP_LIVE_TRIGGER_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: acento neutro por omissão
    // ⚠️ **O título CONTA as órfãs**, e é a única coisa que se vê com a secção dobrada — que é
    // exactamente o estado em que um gatilho partido passa despercebido.
    let orfas = info.orfas();
    let titulo = if info.rows.is_empty() {
        String::from(tr("panel.inspector.trigger.trigger"))
    } else if orfas > 0 {
        tr_with(
            "panel.inspector.trigger.title_count_broken",
            &[("n", &info.rows.len()), ("broken", &orfas)],
        )
    } else {
        tr_with(
            "panel.inspector.trigger.title_count",
            &[("n", &info.rows.len())],
        )
    };
    let header = section_header(store, core_ids::INSP_LIVE_TRIGGER_SECTION, &titulo).color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect) {
        hit_index.register(color_id, circle);
    }
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_TRIGGER_SECTION,
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
            tr("panel.inspector.trigger.no_triggers_yet"),
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
