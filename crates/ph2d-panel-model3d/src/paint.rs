//! O corpo do painel. Percorre o **retrato** que o shell publicou — a mesma lista que o `event`
//! consulta —, então o que é pintado e o que despacha não podem discordar.

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{paint_text_block, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::paint_segmented_group_adaptive;
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_surface, paint_panel_title,
    panel_drag_handle_rect,
};
use ph2d_editor_core::widget::{NUMBER_INPUT_MIN_W_PX, paint_slider_with_chip_layout_adaptive};
use ph2d_editor_core::zones::Rect;
use ph2d_field::Bound;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

use crate::state::{self, Model3dPanelState, ParamRow};
use crate::{Model3dPanel, populate::MAX_MODES, populate::MAX_ROWS};

/// A goteira do rótulo. Os rótulos aqui são o **tipo do nó** ("Union", "Cylinder"), não uma frase.
const LABEL_COL_W: f32 = 72.0; // LITERAL-PX-OK: panel grid metric (label gutter width)

/// Em quantos passos o arrasto atravessa o curso de uma linha.
///
/// ⚠️ **Em centésimos do CURSO, e não num passo absoluto**: um passo fixo seria grosseiro num filete
/// de 0,02 e fino demais num ângulo de 360 de curso. ⭐ É daqui que sai o passo, e é o passo que
/// decide quantas casas a linha mostra ([`decimals_for_step`]) — os dois números têm de vir da mesma
/// fonte, senão a tela deixa de acompanhar o arrasto sem que nada pareça errado.
const STEPS_ACROSS_THE_RANGE: f32 = 100.0; // LITERAL-PX-OK: granularidade de arrasto, não métrica de design

/// ⭐ **Quantas casas decimais uma linha mostra — DERIVADO do passo do arrasto dela.**
///
/// ⚠️ Isto era a constante `RADIUS_DECIMALS = 3`, escrita quando a única linha do painel era o
/// filete. Ela passou a servir cinco grandezas de escalas diferentes, e uma delas é um **ângulo**:
/// `90,000` gasta três casas a dizer nada, enquanto um filete de `0,0012` de passo precisa de mais
/// do que três para dois passos vizinhos não lerem igual.
///
/// A regra é a única que não é um palpite: **o número na tela tem de distinguir dois passos
/// consecutivos do arrasto**. Com passo `s`, isso é `ceil(−log10 s)`; a casa extra é para o que o
/// artista **digita** entre dois passos (senão escrever `45,5` mostraria `46`, e o painel mentiria
/// sobre o documento).
///
/// ⚠️ **Só a direção FINA é uma lei; a grossa é legibilidade.** Faltar uma casa faz dois passos
/// lerem igual — a tela deixa de acompanhar o arrasto, e isso tem gate. Sobrar uma casa é feio
/// (`90,000` gasta três dígitos a dizer nada) e nada mais; o gate não a defende, e dizer isto aqui é
/// mais honesto do que escrever um gate que finge medi-la.
///
/// # O teto de 6, e de que recurso ele é
///
/// É a precisão do `f32` no ponto que interessa: o ULP de uma coordenada de ordem 1 é **1,19e-7**,
/// então a sexta casa (1e-6) é a **última** que ainda distingue dois valores de verdade — a sétima
/// escreveria ruído do tipo, não da peça. Abaixo disso o passo não é representável, e um campo mais
/// largo não o traria de volta.
fn decimals_for_step(step: f32) -> usize {
    if !step.is_finite() || step <= 0.0 {
        return 1;
    }
    let needed = (-step.log10()).ceil().max(0.0) as usize + 1;
    needed.clamp(1, 6)
}

pub(crate) fn paint(_state: &mut Model3dPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(Model3dPanel::ID) {
        // Limpeza simétrica do rect: sem isto o `panel_at` continuaria a devolver este painel
        // depois de ele fechar, e a roda do canvas ficaria a rolar um painel invisível.
        ctx.host.store_mut().clear_panel_rect(ids::MODEL3D_PANEL);
        return;
    }

    let rect: Rect = ctx.layout.inspector;
    let theme = ctx.host.theme();
    let snapshot = state::current();

    ctx.host
        .store_mut()
        .set_panel_rect(ids::MODEL3D_PANEL, rect);
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    {
        let drag_rect =
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
        ctx.host
            .hit_index_mut()
            .register(ids::INSP_DRAG_HANDLE, drag_rect);
    }

    let title_size = paint_panel_title(
        rect,
        tr("panel.model3d.title"),
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        ids::MODEL3D_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let x = rect.x + PANEL_HEAD_PAD;
    let w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);

    ctx.scene
        .push_clip(&rect_to_vello(Rect::new(rect.x, body_top, rect.w, body_h)));
    let mut y = body_top;
    // ⭐ **O seletor do verbo, no topo** — é a porta que se encontra sem saber que existe. As
    // teclas `G`/`R`/`S` são a outra, e são para quem já sabe.
    y = paint_chips(ctx, &snapshot.modes, ids::model3d_mode_button, x, w, y);
    // ⭐ **Em que eixos** — do mundo, ou do próprio objeto. Fica por baixo do verbo porque ele
    // qualifica o verbo: um referencial sem verbo não quer dizer nada.
    y = paint_chips(ctx, &snapshot.frames, ids::model3d_frame_button, x, w, y);
    // ⭐ **Criar e combinar** — sem estes dois, o módulo edita a cena que veio pronta e mais nada.
    y = paint_chips(ctx, &snapshot.adds, ids::model3d_add_button, x, w, y);
    y = paint_chips(ctx, &snapshot.ops, ids::model3d_op_button, x, w, y);
    // ⭐ **O que se faz À forma depois de ela existir** — a casca e o afastamento, os dois verbos em
    // que a tese do módulo mais aparece (ver `ph2d_field::mods`).
    y = paint_chips(ctx, &snapshot.mods, ids::model3d_mod_button, x, w, y);
    y = paint_chips(ctx, &snapshot.acts, ids::model3d_act_button, x, w, y);
    if snapshot.rows.is_empty() {
        y = paint_note(ctx, tr("panel.model3d.empty"), x, w, y);
    }
    // ⚠️ **Corta na família, e o rodapé DIZ que cortou** — ver `MAX_ROWS`. Uma linha além dela
    // ficaria sem controle registado: pintada e morta sob o rato, que é a falha de paridade de
    // fiação na sua forma mais cara.
    for (slot, row) in snapshot.rows.iter().enumerate().take(MAX_ROWS) {
        y = paint_row(ctx, row, slot as u32, x, w, y);
    }
    y = paint_footer(ctx, &snapshot, x, w, y);
    state::set_last_content_h(y - body_top + PANEL_HEAD_PAD);
    ctx.scene.pop_layer();
}

/// ⭐ **Um seletor segmentado** — os verbos do gizmo, ou o referencial dos eixos. Devolve o **y
/// seguinte**.
///
/// ⚠️ **Grupo ADAPTATIVO**, e não três botões de largura fixa: num painel estreito ou num idioma
/// mais comprido, três rótulos lado a lado deixam de caber, e a versão fixa quebraria o texto
/// DENTRO do botão — o artefato que a casa já registou e curou com este widget.
fn paint_chips(
    ctx: &mut PaintCtx,
    chips: &[state::ModeChip],
    id_of: fn(u32) -> ph2d_a11y::NodeId,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if chips.is_empty() {
        return y;
    }
    let theme = ctx.host.theme();
    let labels: Vec<(&str, bool, ph2d_a11y::NodeId)> = chips
        .iter()
        .take(MAX_MODES as usize)
        .enumerate()
        .map(|(i, m)| (tr(m.key), m.active, id_of(i as u32)))
        .collect();
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_segmented_group_adaptive(
        Rect::new(x, y, w, ROW_H_PX),
        &labels,
        ctx.scene,
        ctx.text_system,
        theme,
        store,
        hit_index,
    );
    y + used + Spacing::Sm.px()
}

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
fn paint_row(ctx: &mut PaintCtx, row: &ParamRow, slot: u32, x: f32, w: f32, y: f32) -> f32 {
    // ⭐ **Uma linha que não pode agir não é pintada como se pudesse** — ver [`ParamRow::live`]. Ela
    // sai daqui como facto e **não regista nada** no índice de acerto, então não há slider a agarrar
    // nem campo a receber texto: é a mesma lei do [`paint_note`], neste mesmo arquivo.
    if !row.live {
        return paint_fact(ctx, row, x, w, y);
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
    let step = scale / STEPS_ACROSS_THE_RANGE;
    {
        let store = ctx.host.store_mut();
        store.link_slider_number_mapped(slider, chip, scale, lo);
        store.set_number_range(chip, f64::from(lo), f64::from(hi), f64::from(step));
    }

    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    // A verdade é o documento; a posição guardada é só o que o arrasto deixou para trás. Semear a
    // trilha a partir do valor todo quadro é o que mantém o controle honesto quando o raio muda de
    // outro lado — um desfazer, um arquivo aberto, uma segunda linha.
    let track = ((row.value - lo) / scale).clamp(0.0, 1.0);
    let display = f64::from(row.value);
    let decimals = decimals_for_step(step);
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
    y + ROW_H_PX + Spacing::Xs.px()
}

/// Texto puro — um fato, não um controle. Sem hit-index: uma affordance que ele não pode honrar
/// seria pior do que nenhuma.
fn paint_note(ctx: &mut PaintCtx, text: &str, x: f32, w: f32, y: f32) -> f32 {
    let font = TypeToken::Sm.px();
    let theme = ctx.host.theme();
    let used = paint_text_block(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    // O avanço é MEDIDO: uma nota quebra em duas linhas num painel estreito ou num idioma mais
    // comprido, e um avanço fixo escreveria a linha seguinte por cima dela.
    y + (ROW_H_PX - font).mul_add(0.5, used).max(ROW_H_PX) + Spacing::Xs.px()
}

/// O rodapé: quantos nós, e **quanto custou o último quadro**.
///
/// ⭐ O custo fica aqui, e não só no terminal, porque quem mexe num raio é quem paga o traçado
/// seguinte — e um painel que esconde isso deixa a lentidão parecer um defeito em vez de uma conta.
fn paint_footer(ctx: &mut PaintCtx, snap: &state::ModelSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let hidden = snap.rows.len().saturating_sub(MAX_ROWS);
    let mut line = format!(
        "{}: {} · {} {:.1} ms",
        tr("panel.model3d.nodes"),
        snap.node_count,
        tr("panel.model3d.trace_cost"),
        snap.last_trace_ms
    );
    if hidden > 0 {
        // ⛔ **Nunca em silêncio.** Se a família de ids não chegou para todos os nós, o artista tem
        // de saber quantos ficaram sem controle — senão a conclusão dele é que o modelo encolheu.
        line.push_str(&format!(" (+{hidden})"));
    }
    paint_note(ctx, &line, x, w, y + Spacing::Xs.px())
}

/// A parte de um limite que a UI precisa de saber: o topo, e se ele é parede.
///
/// ⚠️ Existe para o dia em que o painel **desenhar** a diferença (uma parede e uma sugestão não se
/// pintam igual). Hoje as duas viram a mesma faixa, e isto é o registo de que a distinção existe no
/// documento e ainda não chegou ao pixel.
#[allow(dead_code)]
fn bound_is_wall(b: Bound) -> bool {
    matches!(b, Bound::Hard(_))
}

#[cfg(test)]
#[path = "paint_tests.rs"]
mod tests;
