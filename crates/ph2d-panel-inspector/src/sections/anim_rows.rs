//! **A BIBLIOTECA da §11** — a lista de animações e o editor da que está aberta.
//!
//! ⚠️ **Irmão de [`super::anim`] por CAP de FICHEIRO** (600), o mesmo corte que a §12 fez entre
//! [`super::anchors`] e [`super::anchor_mount_row`].
//!
//! ⚠️ **A linha selecionada É a animação que toca** — ver o doc de [`super::anim`]. Por isso ela
//! desenha-se com o mesmo realce que a §12 dá à ficha aberta, e o clique nela vai ao barramento.

use super::*;
use ph2d_editor_core::screens::hero::InspectorAnimInfo;
use ph2d_i18n::tr;

// ⭐ **A linha de uma LISTA e a linha do app** (wave 17): o `22.0` a mao coincidia com o
// token, e uma coincidencia nao segue quem mexe no token.
const ROW_H: f32 = ph2d_tokens::ROW_H_PX;
/// Onde o resumo (`2-5 · Forward`) começa, como fração da largura.
const SUMMARY_COL_FRAC: f32 = 0.52; // LITERAL-PX-OK: fração de layout

/// A lista mais o editor. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_library(
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
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    paint_text(
        text_system,
        scene,
        tr("panel.inspector.animation.this_sprite_s_animations"),
        x,
        cur_y,
        font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    cur_y += font + ph2d_tokens::control_gap_px();

    if info.rows.is_empty() {
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.animation.no_animations_yet"),
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        cur_y += font + ph2d_tokens::control_gap_px();
    } else {
        // ⚠️ `zip` com o array de ids: uma biblioteca com mais tags do que ids (impossível
        // enquanto o gate `the_anim_row_ids_cover_the_model_cap` viver) perde as excedentes em
        // vez de as pintar umas sobre as outras.
        for (i, (row, &id)) in info.rows.iter().zip(ids::INSP_ANIM_ROW.iter()).enumerate() {
            let rect = Rect::new(x, cur_y, w, ROW_H);
            hit_index.register(id, rect);
            // ⭐ A listra da linha ímpar — aqui o índice do laço É o visual (nada é saltado).
            // ⚠️ **O tom base vem da PORTA** (`CardDepth::Section.token()`), nunca do `ColorToken::Bg1`
            //    escrito à mão: é a lei da wave 13, e o censo dela apanhou esta linha na 1.ª redacção.
            //    ⛔ E é a `Section`, não a `Subsection`: a listra **não é uma superfície nova dentro do**
            //    **cartão** — ela é o próprio fundo do cartão movido 5/255, e é esse tamanho que a
            //    impede de se ler como um aninhamento.
            ph2d_editor_core::widget::paint_row_stripe(
                scene,
                rect,
                theme,
                ph2d_editor_core::widget::section_cards::CardDepth::Section.token(),
                i,
            );
            // **Duas coisas diferentes, dois realces:** a que está ABERTA no editor (fundo), e a
            // que está a TOCAR (o texto aceso). Elas são a mesma na esmagadora maioria dos
            // casos — mas não quando a que toca deixou de caber na grelha.
            let is_open = i == selected;
            let is_current = info.current == row.name;
            // ⭐ Pela porta (wave 21) — ver o irmão em `anchors.rs`.
            ph2d_editor_core::widget::paint_row_highlight(
                scene,
                rect,
                theme,
                if is_open {
                    ph2d_editor_core::widget::RowHighlight::Selected
                } else {
                    ph2d_editor_core::widget::RowHighlight::None
                },
                0.0,
            );
            let color = if is_current {
                resolve(ColorToken::Accent, theme)
            } else if is_open {
                resolve(ColorToken::Text1, theme)
            } else {
                resolve(ColorToken::Text2, theme)
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
            // ⚠️ **O resumo diz o que a linha FAZ**, e o alcance sai da grelha de hoje: uma tag
            // que deixou de caber mostra-o aqui, e não só quando alguém carrega em Play.
            let dir = ph2d_ecs_dir_label(row.direction_tag);
            let summary = if row.fits(info.cells) {
                format!(
                    "{}-{} \u{b7} {dir}",
                    row.from.min(row.to),
                    row.from.max(row.to)
                )
            } else {
                String::from(tr("panel.inspector.animation.out_of_grid"))
            };
            let summary_color = if row.fits(info.cells) {
                resolve(ColorToken::Text3, theme)
            } else {
                resolve(ColorToken::Danger, theme)
            };
            paint_text(
                text_system,
                scene,
                &summary,
                x + w * SUMMARY_COL_FRAC,
                cur_y + (ROW_H - font) * 0.5,
                font,
                w,
                summary_color,
            );
            cur_y += ROW_H + ph2d_tokens::list_row_gap_px();
        }
        cur_y += Spacing::Sm.px();
        if let Some(row) = info.rows.get(selected.min(info.rows.len() - 1)) {
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
    }

    let add = ph2d_editor_core::property_row::caixa_do_botao(
        text_system,
        x,
        w,
        cur_y,
        ALTURA_DE_BOTAO,
        tr("panel.inspector.animation.plus_add_animation"),
    );
    hit_index.register(ids::INSP_ANIM_ADD, add);
    paint_button(
        &Button::new(
            ids::INSP_ANIM_ADD,
            tr("panel.inspector.animation.plus_add_animation"),
        )
        .kind(ButtonKind::Default)
        .visual(store.button_visual(ids::INSP_ANIM_ADD)),
        add,
        scene,
        text_system,
        theme,
    );
    cur_y = ph2d_editor_core::property_row::abaixo_do_botao(add);
    if !info.rows.is_empty() {
        let rm = ph2d_editor_core::property_row::caixa_do_botao(
            text_system,
            x,
            w,
            cur_y,
            ALTURA_DE_BOTAO,
            tr("panel.inspector.animation.x_remove_animation"),
        );
        hit_index.register(ids::INSP_ANIM_REMOVE, rm);
        paint_button(
            &Button::new(
                ids::INSP_ANIM_REMOVE,
                tr("panel.inspector.animation.x_remove_animation"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_ANIM_REMOVE)),
            rm,
            scene,
            text_system,
            theme,
        );
        cur_y = ph2d_editor_core::property_row::abaixo_do_botao(rm);
    }
    cur_y + ph2d_tokens::control_gap_px()
}

/// O rótulo de uma direção, pelo tag.
///
/// ⚠️⚠️ **A redacção anterior dizia que isto ESPELHA a `ph2d_ecs::AnimDirection::label` e que «o
/// gate da shell prende os dois» — e esse gate NUNCA EXISTIU** (medido 2026-09-19: o
/// `ph2d_ecs_dir_label` é citado por este ficheiro e por uma linha de prosa, e por mais nada). É a
/// família que o `named_gates_census` da escultura curou em 13/09: *um gate citado em comentário
/// lê-se como um gate.*
///
/// ⭐ Hoje não há o que prender: a `AnimDirection::label` foi **apagada** como órfã na mesma
/// medição — ela não tinha um único consumidor de produto —, e esta é a ÚNICA cópia do
/// vocabulário. *Apagar a segunda resposta é mais forte do que gatear as duas.*
fn ph2d_ecs_dir_label(tag: u8) -> &'static str {
    match tag {
        1 => tr("panel.inspector.animation.reverse"),
        2 => tr("panel.inspector.animation.ping_pong"),
        3 => tr("panel.inspector.animation.ping_pong_rev"),
        _ => tr("panel.inspector.animation.forward"),
    }
}

/// **O INTERVALO e o RITMO de uma animação** — seis linhas guiadas por tabela.
///
/// ⚠️ **Sai do [`editor`] por TETO de função** (200), estourado em 2026-09-15 ao partir as três
/// rows de DUAS propriedades em seis de uma. ⛔ *A cura de um teto é o corte por responsabilidade,
/// nunca um número maior* (`CLAUDE.md` §5.0) — e o corte é honesto: o que sai é *«que células e a
/// que ritmo»*, contra *«que animação e como toca»* que fica no pai.
///
/// ⭐⭐ **Eram TRÊS linhas com DUAS propriedades cada** (`From / To (cell)` ·
/// `Frame ms / Repeat (0 = forever)` · `Hold ms / Repeat delay ms`). Com o rótulo POR CIMA os dois
/// campos ficavam lado a lado e o «A / B» mapeava da esquerda para a direita; com o nome AO LADO e
/// a coluna do controlo a refluí-los, **esse mapeamento desaparece** — um nome que descreve duas
/// caixas empilhadas não diz qual é qual. *A conversão forçou o corte que já devia existir.*
///
/// ⚠️ **E o `ms` saiu dos rótulos para dentro das caixas** (ordem do dono, 2026-09-15:
/// *«para manter padrão universal melhor todos na caixa»*).
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
) -> f32 {
    let seccao = seccao_da_animacao(text_system);
    let mut cur_y = y;
    for (chave, id, passo, unidade) in ANTES {
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
            seccao,
        );
    }
    // ⚠️ **Uma animação com ritmo PRÓPRIO por célula (§8.12) tem de o DIZER.** Sem esta linha o
    // campo `Frame` acima mente sobre ela — mostra a duração mais comum e nada explica por que a
    // animação não anda naquele ritmo. O Inspector não a edita (a §8.8 põe essa edição no editor de
    // timeline futuro); ele diz que ela existe, que é a diferença entre um dado e um mistério.
    if row.has_per_frame_timing() {
        // ⭐ **É uma FRASE, logo QUEBRA** — a porta é a mesma dos avisos das outras secções
        //    (`rows::aviso`, 2026-09-19). Medida na coluna de `268 px` ela saía
        //    `• this animation has per-frame timing (impor…`.
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.animation.this_animation_has_per_frame"),
            ColorToken::Warn,
        );
    }
    for (chave, id, passo, unidade) in DEPOIS {
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
            seccao,
        );
    }
    cur_y
}

// ⚠️ **O passo é em MILISSEGUNDOS ou em CÉLULAS, nunca em pixels** — daí o marcador.
const MS_STEP: f64 = 10.0; // LITERAL-PX-OK: passo de scrub em MILISSEGUNDOS, não em pixels
const MS: Option<ph2d_editor_core::widget::Unit> =
    Some(ph2d_editor_core::widget::Unit::Milliseconds);
const ANTES: [(&str, NodeId, f64, Option<ph2d_editor_core::widget::Unit>); 4] = [
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
    (
        "panel.inspector.animation.frame",
        ids::INSP_ANIM_FRAME_MS,
        1.0,
        MS,
    ),
    (
        "panel.inspector.animation.repeat_forever",
        ids::INSP_ANIM_REPEAT,
        1.0,
        None,
    ),
];
const DEPOIS: [(&str, NodeId, f64, Option<ph2d_editor_core::widget::Unit>); 2] = [
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
    for (chave, ..) in ANTES {
        nomes.push(tr(chave));
    }
    for (chave, ..) in DEPOIS {
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
        "panel.inspector.animation.on_finish_label",
        "panel.inspector.animation.on_loop_label",
        "panel.inspector.animation.direction",
        "panel.inspector.animation.direction_override",
        "panel.inspector.animation.loop_override",
    ] {
        nomes.push(tr(chave));
    }
    ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &nomes)
}

/// O editor da animação aberta.
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
        sec_texto,
    );

    cur_y = range_and_timing_rows(
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

    // ⭐⭐ **A direção é UMA escolha com NOME, pela porta** (2026-09-23). ⛔ Ela era quatro
    //    `Button` em partes IGUAIS com o nome POR CIMA — a quarta forma de «escolha com nome» deste
    //    painel —, e o `PP` era um literal ao lado de três chaves. ⚠️ A selecção vem do SNAPSHOT.
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
    let font = TypeToken::Sm.px();

    // **OS SINAIS** (spec §8.10) — no FIM, e depois da direção, porque eles são o que a animação
    // diz para FORA. Tudo acima descreve o que ela faz; isto descreve o que ela anuncia.
    //
    // ⚠️ **Dois campos e não um mais uma fase**: acabar e dar a volta distinguem-se por serem
    // nomes diferentes — a lei que a física já escreveu para os contatos. Vazio = calada.
    paint_text(
        text_system,
        scene,
        tr("panel.inspector.animation.signals_empty_silent"),
        x,
        cur_y,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    cur_y += font + ph2d_tokens::control_gap_px();
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
    text_row(
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
pub(super) fn text_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
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
