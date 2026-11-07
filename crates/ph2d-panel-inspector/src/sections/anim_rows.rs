//! **A BIBLIOTECA da §11** — a lista de animações e o editor da que está aberta.
//!
//! ⚠️ **Irmão de [`super::anim`] por CAP de FICHEIRO** (600), o mesmo corte que a §12 fez entre
//! [`super::anchors`] e [`super::anchor_mount_row`].
//!
//! ⚠️ **A linha selecionada É a animação que toca** — ver o doc de [`super::anim`]. Por isso ela
//! desenha-se com o mesmo realce que a §12 dá à ficha aberta, e o clique nela vai ao barramento.

use super::*;
use ph2d_editor_core::screens::hero::InspectorAnimInfo;
use ph2d_i18n::{tr, tr_with};

/// **A animação escolhida** (o grupo com o nome dela): nome, quadros, ritmo em quadros por segundo e
/// o ciclo — as mesmas leituras do editor. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_selected_editor(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
    selected: usize,
) -> f32 {
    match selected_row(info, selected) {
        Some(row) => range_and_timing_rows(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            y,
            row,
            info.cells,
        ),
        None => y,
    }
}

/// **«Advanced»** (o corpo do grupo que nasce fechado): os campos finos da animação escolhida. As
/// substituições do tocador vêm a seguir, do `anim.rs`. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_advanced(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
    selected: usize,
) -> f32 {
    match selected_row(info, selected) {
        Some(row) => editor(scene, text_system, theme, hit_index, store, x, w, y, row),
        None => y,
    }
}

pub(super) fn selected_row(
    info: &InspectorAnimInfo,
    selected: usize,
) -> Option<&ph2d_editor_core::screens::hero::InspectorAnimRow> {
    info.rows
        .get(selected.min(info.rows.len().saturating_sub(1)))
}

/// «+ Add» e «Remove» lado a lado, colados à lista que eles mudam (plano 22). Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_add_remove(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
) -> f32 {
    let half = (w - ph2d_tokens::control_gap_px()) * 0.5;
    let mut botao = |id: NodeId, key: &str, x: f32| {
        let r = Rect::new(x, y, half, ph2d_tokens::ROW_H_PX);
        hit_index.register(id, r);
        paint_button(
            &Button::new(id, tr(key))
                .kind(ButtonKind::Default)
                .visual(store.button_visual(id)),
            r,
            scene,
            text_system,
            theme,
        );
        r
    };
    let add = botao(
        ids::INSP_ANIM_ADD,
        "panel.inspector.animation.plus_add_animation",
        x,
    );
    if !info.rows.is_empty() {
        botao(
            ids::INSP_ANIM_REMOVE,
            "panel.inspector.animation.x_remove_animation",
            x + w - half,
        );
    }
    let edit_y = ph2d_editor_core::property_row::abaixo_do_botao(add);
    let rotulo = tr("panel.inspector.anim_card.open_editor");
    let edit = ph2d_editor_core::property_row::caixa_do_botao(text_system, x, w, edit_y, rotulo);
    hit_index.register(ids::INSP_ANIM_OPEN_EDITOR, edit);
    paint_button(
        &Button::new(ids::INSP_ANIM_OPEN_EDITOR, rotulo)
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_ANIM_OPEN_EDITOR)),
        edit,
        scene,
        text_system,
        theme,
    );
    ph2d_editor_core::property_row::abaixo_do_botao(edit)
}

/// **O nome, os quadros, o ritmo e o ciclo da animação escolhida.**
///
/// Numa lista de quadros a linha diz quantos o editor escolheu (report do dono 25/10); From/To moram em
/// «Advanced» (plano 22). O ritmo é em quadros por segundo e o ciclo é Repeat · Once · Ping-pong — as
/// leituras do editor; o dado continua `frame_ms` + `repeat`/`direction`.
#[allow(clippy::too_many_arguments)]
fn range_and_timing_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &ph2d_editor_core::screens::hero::InspectorAnimRow,
    cells: u32,
) -> f32 {
    let seccao = seccao_da_animacao(text_system);
    let mut cur_y = super::anim::subtitle(
        scene,
        text_system,
        theme,
        x,
        w,
        y + ph2d_tokens::control_gap_px(),
        tr("panel.inspector.anim_card.selected"),
    );
    cur_y = text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.animation.name_label"),
        ids::INSP_ANIM_NAME,
        TextInput::new(ids::INSP_ANIM_NAME, "")
            .placeholder(tr("panel.inspector.animation.animation_name")),
        seccao,
    );
    if !row.frames.is_empty() {
        let n = ph2d_editor_core::screens::hero::row_cells(row, cells).len();
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &tr_with("panel.inspector.anim_card.frames_chosen", &[("n", &n)]),
            ColorToken::Text3,
        );
    }
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.anim_card.fps"),
        &[ids::INSP_ANIM_FPS],
        1.0,
        None,
        seccao,
    );
    cur_y = super::anim_cartao::cycle_rows(
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
    // O vão entre grupos: «Who Picks» começa outro assunto.
    cur_y + ph2d_tokens::control_gap_px()
}

// ⚠️ **O passo é em MILISSEGUNDOS ou em CÉLULAS, nunca em pixels** — daí o marcador.
const MS_STEP: f64 = 10.0; // LITERAL-PX-OK: passo de scrub em MILISSEGUNDOS, não em pixels
const MS: Option<ph2d_editor_core::widget::Unit> =
    Some(ph2d_editor_core::widget::Unit::Milliseconds);
const RANGE: [(&str, NodeId, f64, Option<ph2d_editor_core::widget::Unit>); 2] = [
    (
        "panel.inspector.animation.from_cell",
        ids::INSP_ANIM_FROM,
        1.0,
        None,
    ),
    (
        "panel.inspector.animation.to_cell",
        ids::INSP_ANIM_TO,
        1.0,
        None,
    ),
];
/// Os números finos de «Advanced».
const FINE: [(&str, NodeId, f64, Option<ph2d_editor_core::widget::Unit>); 3] = [
    (
        "panel.inspector.animation.hold",
        ids::INSP_ANIM_HOLD_MS,
        MS_STEP,
        MS,
    ),
    (
        "panel.inspector.animation.repeat_delay",
        ids::INSP_ANIM_DELAY_MS,
        MS_STEP,
        MS,
    ),
    (
        "panel.inspector.anim_card.repeat_count",
        ids::INSP_ANIM_REPEAT,
        1.0,
        None,
    ),
];

/// ⭐⭐⭐ **A COLUNA DA SECÇÃO ANIMAÇÃO — uma só, para os dois blocos que a desenham.**
///
/// ⛔⛔ **Report do dono, 2026-09-15:** *«a caixa recua quando na verdade o nome deveria criar as
/// colunas»*. Esta secção pinta linhas em **dois** sítios (a velocidade e o quadro vivo em
/// `anim.rs`, o intervalo e os tempos aqui), e uma medida por sítio seria duas colunas dentro da
/// mesma secção — a doença que a [`ph2d_editor_core::property_row::Seccao`] existe para fechar.
///
/// ⚠️ **Os nomes saem das TABELAS que pintam**, nunca de uma segunda lista: acrescentar uma linha
/// ali entra aqui de graça, e uma lista à mão envelheceria no primeiro rótulo novo.
pub(super) fn seccao_da_animacao(
    text_system: &mut TextSystem,
) -> ph2d_editor_core::property_row::Seccao {
    let mut nomes: Vec<&str> = Vec::new();
    for (chave, ..) in RANGE.iter().chain(FINE.iter()) {
        nomes.push(tr(chave));
    }
    // ⚠️ As duas linhas que vivem no `anim.rs` — ver o doc acima.
    nomes.push(tr("panel.inspector.animation.speed_x"));
    nomes.push(tr("panel.inspector.animation.this_frame_ms_0_use"));
    // ⛔⛔ **E as de TEXTO e as de ESCOLHA também (2026-09-23).** O editor media uma SEGUNDA coluna
    //    (`sec_texto`, só com os três nomes de texto) — duas colunas dentro da mesma secção, e o
    //    campo de texto começava num `x` e o numérico noutro. *Uma coluna por espécie de linha é uma
    //    coluna por linha com outro nome.*
    for chave in [
        "panel.inspector.animation.name_label",
        "panel.inspector.anim_card.fps",
        "panel.inspector.anim_card.cycle",
        "panel.inspector.animation.on_finish_label",
        "panel.inspector.animation.on_loop_label",
        "panel.inspector.animation.direction",
        "panel.inspector.animation.direction_override",
        "panel.inspector.animation.loop_override",
        // O cartão (plano GameEngineFuncional 07): a grelha, «Who Picks» e os estados.
        "component.field.sprite_grid.1",
        "component.field.sprite_grid.2",
        "panel.inspector.sprite_sheet.show_sheet_on_canvas",
        "panel.inspector.anim_card.who_picks",
        "component.field.character_animations.0",
        "component.field.character_animations.1",
        "component.field.character_animations.2",
        "component.field.character_animations.3",
        "component.field.character_animations.4",
        "component.field.character_animations.5",
    ] {
        nomes.push(tr(chave));
    }
    ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &nomes)
}

/// Os campos FINOS da animação aberta («Advanced»): From/To (só sem lista de quadros), Hold, Repeat
/// delay, Repeat count, os sinais e, só com um sentido inverso gravado, a direcção.
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
    row: &ph2d_editor_core::screens::hero::InspectorAnimRow,
) -> f32 {
    // ⚠️ **A coluna do nome é da SECÇÃO** — desde 2026-09-22 as linhas de TEXTO também têm nome
    //    (report do dono), logo ela mede-se sobre eles.
    let sec_texto = seccao_da_animacao(text_system);
    let mut cur_y = y;
    // No cartão o intervalo edita-se na folha do editor (ficha 02, decisão 1).
    let range: &[_] = if row.frames.is_empty() && !crate::cards::active() {
        &RANGE
    } else {
        &[]
    };
    for &(chave, id, passo, unidade) in range.iter().chain(FINE.iter()) {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr(chave),
            &[id],
            passo,
            unidade,
            sec_texto,
        );
    }

    // ⭐ Só com um sentido INVERSO gravado (import do `.ase`, projeto antigo): o resto é o Cycle, e tocar
    //    ao contrário é a Speed negativa, como na Godot (plano 22). A linha some ao sair do inverso.
    //    ⚠️ A selecção vem do SNAPSHOT.
    if matches!(row.direction_tag, 1 | 3) {
        let rotulos = [
            tr("panel.inspector.animation.fwd"),
            tr("panel.inspector.animation.rev"),
            tr("panel.inspector.animation.pp"),
            tr("panel.inspector.animation.pp_rev"),
        ];
        let segmentos: Vec<(&str, bool, NodeId)> = ids::INSP_ANIM_DIR
            .iter()
            .zip(rotulos)
            .enumerate()
            .map(|(i, (&id, l))| (l, usize::from(row.direction_tag) == i, id))
            .collect();
        cur_y = ph2d_editor_core::property_row::paint_choice_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.animation.direction"),
            &segmentos,
            sec_texto,
        );
    }
    // **OS SINAIS** (spec §8.10) — no FIM, e depois da direção, porque eles são o que a animação
    // diz para FORA. Tudo acima descreve o que ela faz; isto descreve o que ela anuncia.
    //
    // ⚠️ **Dois campos e não um mais uma fase**: acabar e dar a volta distinguem-se por serem
    // nomes diferentes — a lei que a física já escreveu para os contatos. Vazio = calada.
    cur_y = super::rows::titulo_de_grupo(
        scene,
        text_system,
        theme,
        x,
        w,
        cur_y,
        tr("panel.inspector.animation.signals"),
    );
    cur_y = text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.animation.on_finish_label"),
        ids::INSP_ANIM_SIGNAL_FINISH,
        TextInput::new(ids::INSP_ANIM_SIGNAL_FINISH, "")
            .placeholder(tr("panel.inspector.animation.on_finish")),
        sec_texto,
    );
    cur_y = text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.animation.on_loop_label"),
        ids::INSP_ANIM_SIGNAL_LOOP,
        TextInput::new(ids::INSP_ANIM_SIGNAL_LOOP, "")
            .placeholder(tr("panel.inspector.animation.on_loop")),
        sec_texto,
    );
    super::anim_markers::marker_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        row,
        sec_texto,
    )
}

/// **Uma linha de texto da §11**, pintada do `WidgetStore` — o nome da animação e os dois nomes de
/// sinal são a MESMA coisa vista de três sítios.
///
/// ⚠️ Um helper e não três blocos iguais: o nome já vivia inline aqui, e copiá-lo duas vezes era a
/// forma de o *placeholder*, o `hover_live` ou o caret divergirem entre campos que o artista lê
/// como irmãos.
///
/// ⚠️ **O `TextInput` chega PRONTO do chamador, e o `placeholder` não é um parâmetro `&str`.** A
/// primeira versão recebia a string — e o gate do HR-15 reprovou: ele conta `.placeholder("…")` no
/// código de widget, e passar a string por um argumento tira **três** strings de UI da vista dele.
/// *Um helper que esconde literais do scanner é uma isenção silenciosa da regra que ele scanneia.*
/// ⚠️ **`pub(super)` desde 2026-09-08**: a secção TIMERS pinta o nome e o sinal com esta mesma
/// linha. Copiá-la seria a terceira resposta à pergunta *«como se desenha um campo de texto de uma
/// row do Inspector?»* — e a cópia é onde a lei do `placeholder` acima se perde.
/// ⭐⭐⭐ **UMA LINHA DE TEXTO — hoje é a porta de `ph2d-editor-core`, e ela tem NOME.**
///
/// ⛔⛔⛔ **Report do dono, 2026-09-22, com foto da `FACTORY`:** *«campos de texto difíceis de
/// saber para que servem»*. Até aqui esta função pintava a caixa à largura INTEIRA e o sentido
/// dela vivia **só no espaço reservado**, que o pintor só escreve enquanto a caixa está VAZIA.
/// Medido no mesmo dia: `53` caixas de texto do app com rótulo vazio contra `3` com rótulo.
///
/// ⚠️ **O `label` e a `seccao` não têm valor de omissão de propósito** — um `""` aqui seria o
/// defeito a voltar em silêncio, e a coluna é da SECÇÃO (§6-ter), nunca desta linha.
#[allow(clippy::too_many_arguments)]
pub(super) fn text_row<'a>(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: impl Into<ph2d_editor_core::widget::key_mark::Rotulo<'a>>,
    id: ph2d_a11y::NodeId,
    input: TextInput,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    ph2d_editor_core::property_row::paint_text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        label,
        id,
        input,
        seccao,
    )
}
