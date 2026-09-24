//! ⭐⭐⭐ **O que o Inspector mostra do EMISSOR DE ABANÃO** (suplente #25) — *ao ouvir o quê* este
//! objecto abana a vista.
//!
//! # ⭐⭐ A lista + UM editor, como o gatilho
//!
//! Um `ShakeEmitter` guarda até `SHAKE_EMITTERS_MAX` fontes de **cinco** campos cada, e
//! elas não cabem numa linha da coluna do Inspector — que o dono corre a `220,9` px de largura
//! (medido no `~/.ph2d/layout.txt` dele). ⇒ o molde é o do [`super::action_trigger`]: a lista
//! escolhe qual fonte está aberta, e um editor só mostra os campos dessa.
//!
//! # ⭐⭐⭐ E o aviso mais valioso desta secção aponta para OUTRO objecto
//!
//! `There is no camera that shakes` — *nada do que este componente faz é visível* sem um
//! `CameraShake` numa câmera. ⚠️ **É a metade que o artista não consegue adivinhar olhando para
//! este objecto**, porque a causa está noutro: ele ouve, a distância mede-se, e não há quem abane.
//! É a mesma forma da recusa dos pincéis de escultura (*«falta uma pilha»*, *«falta bordo»*).

use super::tween_editor::grupo;
use super::*;
use ph2d_editor_core::shake_edits::{InspectorEmitterInfo, InspectorEmitterRow};
use ph2d_editor_core::widget::{SectionFold, Unit};
use ph2d_i18n::{tr, tr_with};

const ROW_H: f32 = ph2d_tokens::ROW_H_PX;

/// **A CHAVE de cada cerca — a PORTA, lida pelo chip E por quem resumir a linha.**
///
/// ⛔⛔ **Ela existe porque o painel NÃO conhece a `ph2d-ecs`** (ADR-0029): o `SignalFrom` vive no
/// motor, e ler o `label()` dele daqui seria furar a parede que a crate inteira respeita. ⇒ a
/// ordem é a mesma **por gate na shell**, que é a única crate que vê os dois lados.
///
/// ⚠️ **O índice É o `u8` que a edição carrega** — reordenar isto reescreve o sentido de toda fonte
/// já gravada, porque aquele enum é `append-only` e a posição dele viaja no ficheiro.
#[must_use]
pub const fn chave_da_cerca(de: u8) -> &'static str {
    match de {
        1 => "panel.inspector.emitter.from_myself",
        _ => "panel.inspector.emitter.from_anyone",
    }
}

/// **O que esta fonte FAZ, numa frase** — a força e o raio externo, que são as duas perguntas que
/// sobram com a lista fechada (o nome do sinal já é a primeira coluna).
///
/// ⚠️ Ela diz *«mute»* em vez de deixar o espaço em branco: uma fonte sem sinal é uma escolha
/// legítima que tem de se ler como escolha.
fn resumo(row: &InspectorEmitterRow) -> String {
    if row.on.trim().is_empty() {
        return tr("panel.inspector.emitter.mute").to_owned();
    }
    format!("{:.2} \u{00b7} {:.0} m", row.forca, row.fora)
}

/// A lista das fontes. Devolve o `y` seguinte.
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
    info: &InspectorEmitterInfo,
    escolhida: usize,
) -> f32 {
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    for (i, (row, &id)) in info
        .rows
        .iter()
        .zip(crate::ids::INSP_EMITTER_ROW.iter())
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
        // ⚠️ **Só os RAIOS TROCADOS pintam WARN na lista.** Uma fonte sem nome é uma configuração a
        // meio (o artista está a escrevê-la), e pintá-la de vermelho aqui poria a lista a gritar
        // sobre uma linha acabada de criar.
        let cor = if row.raios_trocados() {
            resolve(ColorToken::Warn, theme)
        } else if i == escolhida {
            resolve(ColorToken::Text1, theme)
        } else {
            resolve(ColorToken::Text2, theme)
        };
        let nome = if row.on.trim().is_empty() {
            tr("panel.inspector.emitter.unnamed").to_owned()
        } else {
            row.on.clone()
        };
        paint_text(
            text_system,
            scene,
            &nome,
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
    info: &InspectorEmitterInfo,
) -> f32 {
    // ⚠️ **O `+` DESAPARECE no tecto** e não fica cinzento a mentir — a lei das irmãs.
    let pode_juntar = info.rows.len() < crate::ids::INSP_EMITTER_ROW.len();
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
        rotulos.push(tr("panel.inspector.emitter.plus_add_source"));
    }
    if pode_tirar {
        rotulos.push(tr("panel.inspector.emitter.x_remove_source"));
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
        hit_index.register(ids::INSP_EMITTER_ADD, rect);
        paint_button(
            &Button::new(
                ids::INSP_EMITTER_ADD,
                tr("panel.inspector.emitter.plus_add_source"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_EMITTER_ADD))
            .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    if pode_tirar {
        let (rect, group) = seg[cell];
        hit_index.register(ids::INSP_EMITTER_REMOVE, rect);
        paint_button(
            &Button::new(
                ids::INSP_EMITTER_REMOVE,
                tr("panel.inspector.emitter.x_remove_source"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_EMITTER_REMOVE))
            .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    y + ALTURA_DE_CAMPO + ph2d_tokens::control_gap_px()
}

/// O editor da fonte aberta. Devolve o `y` seguinte.
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
    row: &InspectorEmitterRow,
) -> f32 {
    // ⚠️ **A coluna nasce ANTES da 1.ª linha que a usa** — desde 2026-09-22 a linha de TEXTO
    //    também tem nome, e ela é a primeira da secção.
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.emitter.strength"),
            tr("panel.inspector.emitter.full_within"),
            tr("panel.inspector.emitter.nothing_beyond"),
            tr("panel.inspector.emitter.on_label"),
            // ⭐ A ESCOLHA também — com o nome ao lado (2026-09-23) ela entra na coluna.
            tr("panel.inspector.emitter.from"),
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
        tr("panel.inspector.emitter.on_label"),
        ids::INSP_EMITTER_ON,
        TextInput::new(ids::INSP_EMITTER_ON, "")
            .placeholder(tr("panel.inspector.emitter.signal_name_empty_mute")),
        seccao,
    );
    // ⭐ A cerca de quem falou.
    let cercas: Vec<&str> = (0..crate::ids::INSP_EMITTER_DE.len())
        .map(|i| tr(chave_da_cerca(u8::try_from(i).unwrap_or(0))))
        .collect();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.emitter.from"),
        &crate::ids::INSP_EMITTER_DE,
        &cercas,
        usize::from(row.de),
        seccao,
    );
    for (id, label, step, unidade) in [
        (
            crate::ids::INSP_EMITTER_FORCA,
            tr("panel.inspector.emitter.strength"),
            0.05, // LITERAL-PX-OK: passo de scrub numa FRACCAO do trauma (0..1)
            None,
        ),
        (
            crate::ids::INSP_EMITTER_DENTRO,
            tr("panel.inspector.emitter.full_within"),
            0.5,
            Some(Unit::Meters),
        ),
        (
            crate::ids::INSP_EMITTER_FORA,
            tr("panel.inspector.emitter.nothing_beyond"),
            0.5,
            Some(Unit::Meters),
        ),
    ] {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            label,
            &[id],
            step,
            unidade,
            seccao,
        );
    }
    // ⚠️ **Os raios trocados são LEGAIS e quase nunca intencionais** — a lei responde-os com um
    // corte duro, e o artista que escreveu `fora = 2` num `dentro = 5` trocou os campos.
    if row.raios_trocados() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.emitter.the_outer_radius_is_not_beyond_the_inner"),
            ColorToken::Warn,
        );
    }
    cur_y
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) fn paint_shake_emitter_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorEmitterInfo,
    escolhida: usize,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    // ⚠️ **O título CONTA as trocadas**, e é a única coisa que se vê com a secção dobrada — que é
    // exactamente o estado em que uma fonte partida passa despercebida.
    let trocadas = info.trocadas();
    let titulo = if info.rows.is_empty() {
        String::from(tr("panel.inspector.emitter.shake_emitter"))
    } else if trocadas > 0 {
        tr_with(
            "panel.inspector.emitter.title_count_broken",
            &[("n", &info.rows.len()), ("broken", &trocadas)],
        )
    } else {
        tr_with(
            "panel.inspector.emitter.title_count",
            &[("n", &info.rows.len())],
        )
    };
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_EMITTER_SECTION,
        &titulo,
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
        ph2d_editor_core::ids::INSP_LIVE_EMITTER_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    // ⭐⭐⭐ **O aviso que aponta para OUTRO objecto vem PRIMEIRO** — sem uma câmera que treme, nada
    // do que esta secção afina é visível, e afinar a força de uma fonte muda é o caminho mais longo
    // até à descoberta.
    if !info.ha_camera_que_treme {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.emitter.there_is_no_camera_that_shakes"),
            ColorToken::Warn,
        );
    } else if !info.clock_playing {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.emitter.it_shakes_while_the_clock_plays"),
            ColorToken::Text3,
        );
    }
    if info.rows.is_empty() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.emitter.no_sources_yet"),
            ColorToken::Text3,
        );
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
            row,
        );
    }
    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}
