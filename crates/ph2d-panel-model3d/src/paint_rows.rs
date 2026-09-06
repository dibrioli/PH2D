//! ⭐⭐⭐ **AS TRÊS FORMAS DE UMA LINHA** — um número, uma escolha, ou um facto.
//!
//! # Por que um módulo irmão
//!
//! O `paint.rs` é o **orquestrador**: ele decide a ordem das secções e onde cada uma começa. Estas
//! três respondem a outra pergunta — *«como se desenha UMA linha, e o que ela regista no índice de
//! acerto»* —, e as três juntas passavam o `paint.rs` dos **600** do gate de LOC dos painéis quando
//! a fileira de escolha entrou (Enio, 2026-08-31). ⛔ *Split, nunca uma entrada na allowlist.*
//!
//! ⚠️ **Neste arquivo, como no irmão, toda função de pintura devolve o Y SEGUINTE** — e a razão
//! está escrita no doc do [`paint_row`]: misturar as duas convenções foi um smoke reprovado.

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{paint_text_block, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::panel_chrome::paint_segmented_group_adaptive;
use ph2d_editor_core::widget::{NUMBER_INPUT_MIN_W_PX, paint_slider_with_chip_layout_adaptive};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

use crate::paint::{
    LABEL_COL_W, STEPS_ACROSS_THE_RANGE, TETO_DIGITAVEL, bound_is_wall, decimals_for_step,
};
use crate::state::ParamRow;

/// Uma linha: rótulo do tipo do nó + slider + campo numérico. Devolve o **y seguinte**.
///
/// ⚠️ **DEVOLVE O Y SEGUINTE, e o helper que ela chama devolve a ALTURA USADA.** As duas convenções
/// existem no mesmo repo e a confusão entre elas foi um smoke reprovado (Enio, 2026-08-19: *"o
/// painel apresenta apenas um slider"*): com `y = paint_row(...)`, a segunda linha ia parar em
/// `y = 28` **absoluto** — dentro do título, fora do recorte — e as três seguintes com ela. O painel
/// mostrava uma linha e o modelo parecia ter encolhido.
///
/// ⭐ Neste arquivo **toda** função de pintura devolve o y seguinte. Uma convenção por arquivo, dita
/// aqui: misturar as duas é como o erro entrou.
pub(crate) fn paint_row(
    ctx: &mut PaintCtx,
    row: &ParamRow,
    slot: u32,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    // ⭐ **Uma linha que não pode agir não é pintada como se pudesse** — ver [`ParamRow::live`]. Ela
    // sai daqui como facto e **não regista nada** no índice de acerto, então não há slider a agarrar
    // nem campo a receber texto: é a mesma lei do [`paint_note`], neste mesmo arquivo.
    if !row.live {
        return paint_fact(ctx, row, x, w, y);
    }
    // ⭐⭐⭐ **UMA ESCOLHA NÃO É UM SLIDER** — ver [`ParamRow::choices`] (Enio, 2026-08-31). Ela
    // substitui o controle, e não o acompanha: o mesmo facto em dois controlos é duas verdades.
    if !row.choices.is_empty() {
        return paint_choice(ctx, row, slot, x, w, y);
    }
    let theme = ctx.host.theme();
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    // ⚠️ **O id vem da POSIÇÃO da linha, nunca da entidade.** O `populate` corre antes de a peça
    // existir e cunha a família às cegas (ver `MAX_ROWS`); os bits de uma entidade não cabem numa
    // família de 64. A entidade viaja no intent, que é onde ela importa.
    let slider = ids::model3d_radius_slider(slot);
    let chip = ids::model3d_radius_chip(slot);
    // ⚠️ **DUAS pontas, e o piso não é zero em toda linha.** Uma posição vai para os dois lados da
    // origem e um ângulo para os dois lados do zero; escrever `0.0` aqui — como esta função fazia —
    // era o que tornava um número negativo indigitável, em silêncio (ver `ParamRow::lo`).
    let lo = row.lo;
    let hi = row.bound.value();
    // Uma faixa degenerada continua a ter de dar um mapeamento invertível: sem isto, `scale = 0`
    // produziria ±inf ao espelhar o campo no slider.
    let scale = (hi - lo).max(f32::MIN_POSITIVE);

    // ⚠️ **A faixa real é escrita por LINHA, todo quadro.** O teto de um raio é do nó — a caixa
    // aceita menos do que o cilindro —, e o par slider↔campo foi ligado em 0..1 no `populate`
    // justamente porque a escala não era conhecida lá.
    //
    // ⚠️ Sem `set_number_range` o campo deriva o passo do arrasto do TEXTO do buffer e escorrega
    // ~50 unidades por pixel (a nota que o painel de aquarela deixou: digitar continua a funcionar,
    // o que esconde o defeito).
    // ⚠️ **Numa linha inteira o passo é 1**, e não um centésimo do curso: meia cópia não existe, e um
    // passo fracionário faria o arrasto percorrer valores que a escrita depois arredonda — o número
    // a saltar debaixo do dedo sem que nada esteja errado.
    let step = if row.integral {
        1.0
    } else {
        scale / STEPS_ACROSS_THE_RANGE
    };
    {
        let store = ctx.host.store_mut();
        store.link_slider_number_mapped(slider, chip, scale, lo);
        // ⭐⭐⭐ **UMA PAREDE CLAMPA; UMA SUGESTÃO NÃO** (auditoria de 2026-08-30, achado F4).
        //
        // ⛔ As duas viravam a mesma faixa, e o campo numérico clampava as duas. Medido: numa linha
        // de tecto `Soft(1,0)`, digitar `5.0` escrevia **`1.0`** — e o `Soft` é, por definição, *o
        // alcance do GESTO que a vista escolheu*, não um facto da peça. ⇒ com uma peça pequena era
        // **impossível digitar** uma largura maior, e o único caminho era arrastar até ao fim e
        // esperar que o alcance crescesse — isto é, usar o laço que o report do Enio é.
        //
        // ⚠️ **Com o tecto aberto, o arrasto precisa de uma escala própria**: sem o `rate` ele seria
        // uma proporção sobre um intervalo que não termina, e um pixel andaria milhares. O `rate` é
        // o **mesmo passo** que o stepper usa, e vence o alcance no `pointer_move` — é a combinação
        // que o doc do `set_number_drag_rate` chama de certa.
        //
        // ⭐ E é aqui que a distinção do [`Bound`] ganha o consumidor que o doc do
        // [`bound_is_wall`] esperava.
        let parede = bound_is_wall(row.bound);
        let teto = if parede {
            f64::from(hi)
        } else {
            TETO_DIGITAVEL
        };
        store.set_number_range(chip, f64::from(lo), teto, f64::from(step));
        if parede {
            store.clear_number_drag_rate(chip);
        } else {
            store.set_number_drag_rate(chip, f64::from(step));
        }
        // ⭐⭐⭐ **O VALOR DO CAMPO SEMEIA-SE DO DOCUMENTO, TODO QUADRO** (auditoria de 2026-08-30,
        // achado F2) — a mesma lei que a trilha do slider já seguia, e que o campo nunca teve.
        //
        // ⛔ O `populate` regista o chip com `0.0` e nada o corrigia: a pintura só **desenha** o
        // número (recebe-o por argumento). E o arrasto do campo é **incremental** — ele lê a base do
        // store, não do documento. ⇒ o **primeiro** arrasto de qualquer campo partia de `0`:
        // medido, com o documento em `0,80`, os dois primeiros gestos despachavam `SetParam { value:
        // 0.0 }`. Numa posição isso atira o objecto para a origem; numa largura despacha um zero que
        // a porta recusa, e o número salta e volta.
        //
        // ⚠️ **A porta preserva a edição em curso**: ela não reescreve o buffer com o campo em foco
        // nem a âncora de rollback durante um arrasto (ver `set_number_value`). *Semear todo quadro
        // é o que mantém o controlo honesto quando o valor muda de outro lado* — um desfazer, um
        // ficheiro aberto, o gizmo.
        store.set_number_value(chip, f64::from(row.value));
    }

    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    // A verdade é o documento; a posição guardada é só o que o arrasto deixou para trás. Semear a
    // trilha a partir do valor todo quadro é o que mantém o controle honesto quando o raio muda de
    // outro lado — um desfazer, um arquivo aberto, uma segunda linha.
    let track = ((row.value - lo) / scale).clamp(0.0, 1.0);
    let display = f64::from(row.value);
    let decimals = if row.integral {
        0
    } else {
        decimals_for_step(step)
    };
    let text = format!("{display:.decimals$}");

    let used = paint_slider_with_chip_layout_adaptive(
        Rect::new(x, y, w, ROW_H_PX),
        tr(row.key),
        track,
        display,
        Some(&text),
        slider,
        chip,
        LABEL_COL_W,
        NUMBER_INPUT_MIN_W_PX,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y + used + Spacing::Xs.px()
}

/// ⭐ **Uma linha como FACTO**: o rótulo e o número, em texto apagado, sem controle nenhum.
///
/// ⚠️ **Nada é registado no índice de acerto**, e é essa a metade que importa: um slider desenhado
/// «desligado» mas ainda agarrável despacharia uma edição que a escrita depois recusa — e o artista
/// veria o número saltar e voltar. Aqui não há o que agarrar, e o gate
/// `an_inert_row_registers_nothing_to_click` mede exatamente isso.
///
/// ⚠️ Ela ocupa **a mesma altura** de uma linha viva: atravessar a trava não pode fazer o painel
/// saltar de tamanho debaixo do cursor.
/// ⭐⭐⭐ **A FILEIRA DE ESCOLHA de uma linha** — o rótulo à esquerda, os botões na goteira do valor.
///
/// ⚠️ **A goteira é a MESMA do slider** (`x + LABEL_COL_W`), e isso não é estética: o olho percorre
/// a coluna dos valores de cima a baixo, e uma fileira que começasse noutro sítio faria o painel
/// parecer duas listas.
///
/// ⭐ Ela reusa o `paint_segmented_group_adaptive` — o **mesmo** widget das fileiras de chips do topo
/// do painel. *Um sexto caminho de pintura neste arquivo seria um sexto sítio onde o hit-index
/// pode ficar por registar.*
fn paint_choice(ctx: &mut PaintCtx, row: &ParamRow, slot: u32, x: f32, w: f32, y: f32) -> f32 {
    let theme = ctx.host.theme();
    let font = TypeToken::Sm.px();
    let dim = resolve(ColorToken::Text2, theme);
    let baseline = y + (ROW_H_PX - font) * 0.5;
    paint_text_block(
        ctx.text_system,
        ctx.scene,
        tr(row.key),
        x,
        baseline,
        font,
        LABEL_COL_W,
        dim,
    );
    // ⚠️ **O activo lê-se do VALOR, todo quadro** — nunca de um estado guardado no painel. É a mesma
    // lei que a trilha do slider segue duas funções abaixo: a verdade é o documento, e um espelho
    // local seria a segunda resposta que diverge num desfazer.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let escolhido = row.value.round().max(0.0) as usize;
    let labels: Vec<(&str, bool, ph2d_a11y::NodeId)> = row
        .choices
        .iter()
        .take(crate::populate::MAX_CHOICES as usize)
        .enumerate()
        .map(|(i, k)| {
            (
                tr(k),
                i == escolhido,
                ids::model3d_choice_button(slot, i as u32),
            )
        })
        .collect();
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_segmented_group_adaptive(
        Rect::new(x + LABEL_COL_W, y, (w - LABEL_COL_W).max(0.0), ROW_H_PX),
        &labels,
        ctx.scene,
        ctx.text_system,
        theme,
        store,
        hit_index,
    );
    y + used.max(ROW_H_PX) + Spacing::Xs.px()
}

fn paint_fact(ctx: &mut PaintCtx, row: &ParamRow, x: f32, w: f32, y: f32) -> f32 {
    let font = TypeToken::Sm.px();
    let theme = ctx.host.theme();
    let dim = resolve(ColorToken::Text2, theme);
    let baseline = y + (ROW_H_PX - font) * 0.5;
    paint_text_block(
        ctx.text_system,
        ctx.scene,
        tr(row.key),
        x,
        baseline,
        font,
        LABEL_COL_W,
        dim,
    );
    // O número fica na goteira do valor, como numa linha viva — o olho percorre a coluna sem saltar.
    let text = format!("{:.d$}", f64::from(row.value), d = decimals_for_step(1.0));
    paint_text_block(
        ctx.text_system,
        ctx.scene,
        &text,
        x + LABEL_COL_W,
        baseline,
        font,
        (w - LABEL_COL_W).max(0.0),
        dim,
    );
    y + ph2d_tokens::row_pitch_px()
}
