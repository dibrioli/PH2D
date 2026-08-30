//! O corpo do painel — uma linha por token de `ColorToken::ALL`.
//!
//! ⚠️ **A tabela é a lista, e não uma cópia dela.** `paint`, `populate` e `event` percorrem
//! `ColorToken::ALL`; um token novo no design system nasce pintado, registado e vivo, e não há
//! segunda lista para driftar.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{paint_icon, paint_text, paint_text_block, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_corner_dot, paint_panel_corner_dot_bl, paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::widget::{
    Button, ButtonState, ColorSwatch, IconButtonStyle, IconGlyph, SwatchSize, TOKENS_SCROLLBAR_ID,
    paint_button, paint_color_swatch, paint_icon_button, paint_scrollbar, scrollbar_is_needed,
    scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::contrast::{failing_pairs, token_is_in_a_failing_pair};
use ph2d_tokens::num_overrides::num_overridden_count;
use ph2d_tokens::overrides::{TokenValue, color_override, overridden_count};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, StrokeToken, Theme, TypeToken};

use crate::TokensPanel;
use crate::state::{TokensPanelState, set_last_content_h, set_last_visible_h};

/// Largura do botão *Reset* de uma linha. Estreito de propósito: ele é a exceção, não a coluna.
///
/// ⚠️ `pub(crate)` porque a família numérica ([`crate::paint_num`]) desenha a MESMA coluna: as duas
/// listas partilham a régua, e dois números iguais escritos em dois sítios são dois números que
/// divergem no dia em que um deles muda.
pub(crate) const RESET_W: f32 = 48.0; // LITERAL-PX-OK: panel grid metric (per-row reset button width)
/// Lado do botão de ELO — quadrado, como todo botão de ícone compacto do app.
pub(crate) const LINK_W: f32 = 20.0; // LITERAL-PX-OK: panel grid metric (compact icon button side)
/// Lado da marca de AVISO da linha. Menor que o botão de elo de propósito: ela é um **glifo**,
/// não um alvo — não há gesto para lhe dar, então não carrega a caixa de toque de um botão.
const MARK_W: f32 = 16.0; // LITERAL-PX-OK: panel grid metric (warning glyph side)

pub(crate) fn paint(state: &mut TokensPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(TokensPanel::ID) {
        // Limpeza simétrica do rect: sem ela o `panel_at` continua a devolver TOKENS_PANEL depois
        // de fechado, e a roda do rato roda um painel que não está na tela.
        ctx.host.store_mut().clear_panel_rect(ids::TOKENS_PANEL);
        return;
    }

    let rect: Rect = ctx.layout.inspector;
    let theme = ctx.host.theme();
    ctx.host.store_mut().set_panel_rect(ids::TOKENS_PANEL, rect);

    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);
    {
        let _hit_index = ctx.host.hit_index_mut();
    }

    let title_size = paint_panel_title(
        rect,
        tr("panel.tokens.title"),
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        ids::TOKENS_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll = ctx.host.store().panel_scroll(ids::TOKENS_PANEL);

    ctx.scene.push_clip(&rect_to_vello(body_rect));
    let x = rect.x + PANEL_HEAD_PAD;
    let w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let y_after = paint_body(ctx, theme, x, w, body_top - scroll, state);
    let content_h = (y_after + scroll) - body_top + PANEL_HEAD_PAD;
    set_last_content_h(content_h);
    set_last_visible_h(body_h);
    ctx.scene.pop_layer();

    paint_scrollbar_and_publish(ctx, body_rect, content_h, body_h, scroll, theme);
}

/// O readout do modo + o *Reset All* + as DUAS listas.
fn paint_body(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    w: f32,
    mut y: f32,
    state: &TokensPanelState,
) -> f32 {
    let armed = state.armed();
    // ⚠️ A contagem soma as DUAS famílias, e é o que faz o *Reset This Mode* dizer a verdade: ele
    // devolve o modo inteiro à fábrica, e um readout que contasse só as cores ofereceria o botão
    // sobre um modo "de fábrica" que ainda carrega uma escala autorada.
    let n = overridden_count(theme) + num_overridden_count(theme);
    let font = TypeToken::Sm.px();
    // ⚠️ O modo VIGENTE é dito, e não inferido: o mesmo token tem quatro valores, e um painel que
    // não nomeia qual deles está a mostrar torna o `M` (que cicla o tema) indistinguível de um bug.
    paint_text(
        ctx.text_system,
        ctx.scene,
        &format!(
            "{}  —  {n} {}",
            theme_label(theme),
            tr("panel.tokens.authored")
        ),
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    y += ROW_H_PX + Spacing::Xs.px();

    y = paint_contrast(ctx, theme, x, w, y);

    // ⚠️ Só com o que resetar. Um *Reset All* sobre um modo de fábrica é um clique que não faz
    // nada, e o artista não tem como o saber antes de o dar.
    if n > 0 {
        y = command(
            ctx,
            ids::TOKENS_RESET_ALL,
            tr("panel.tokens.reset_all"),
            x,
            w,
            y,
        ) + Spacing::Xs.px();
    }

    // **O interop DTCG** (plano UI/UX W9) — o par vive ao lado do *Reset This Mode* porque tem o
    // MESMO escopo: uma operação sobre o modo inteiro, as duas famílias juntas.
    //
    // ⚠️ **Sem o `if n > 0` do vizinho, e a assimetria é a decisão.** Um *Reset* de um modo de
    // fábrica é um clique que não faz nada; um EXPORT de um modo de fábrica é o design system
    // inteiro, que é precisamente o que alguém quer levar para outra ferramenta. Condicioná-lo ao
    // que o artista autorou daria um arquivo vazio no caso mais comum.
    y = command_pair(
        ctx,
        (ids::TOKENS_DTCG_EXPORT, tr("panel.tokens.dtcg.export")),
        (ids::TOKENS_DTCG_IMPORT, tr("panel.tokens.dtcg.import")),
        x,
        w,
        y,
    ) + Spacing::Sm.px();

    for (row, &token) in ColorToken::ALL.iter().enumerate() {
        y = paint_token_row(
            ctx,
            theme,
            row,
            token,
            Rect::new(x, y, w, ROW_H_PX),
            armed == Some(row),
        );
    }
    crate::paint_num::paint_numeric_family(ctx, theme, x, w, y, state.armed_num(), state.fx_open())
}

/// **O READOUT DE CONTRASTE** — os pares que um valor AUTORADO deixou abaixo da WCAG, neste modo.
///
/// # Por que ele existe, se o `tokens.json` já é validado nos gates
///
/// Porque a checagem de compilação é **estruturalmente cega ao que o artista escreve**: um teste
/// de unidade corre com a camada de override VAZIA, então ele afirma sempre a tabela de FÁBRICA.
/// O gate `contrast_tests::the_compile_time_check_cannot_see_an_authored_break` **mede esse
/// alcance** em vez de o afirmar — se ele um dia falhar, este readout tornou-se redundante.
///
/// # ⚠️ Nada aqui é interativo, e isso é decisão
///
/// O bloco NOMEIA o par, a razão medida, a barra e o critério. Consertar é escolher outra cor,
/// que é exactamente o gesto que a linha logo abaixo já oferece — um botão *"corrigir"* teria de
/// INVENTAR uma cor, e a que ele inventasse seria a resposta de uma máquina a uma pergunta de
/// design. (A 3ª condição de UI do plano — *o clique chega ao barramento* — é **N/A aqui**, e
/// dizê-lo é mais honesto do que fabricar um botão para a satisfazer.)
///
/// ⚠️ **Só aparece com algo a reportar** — o precedente do *Reset This Mode*: um cabeçalho
/// permanente dizendo *"0 problemas"* é uma linha que o artista aprende a não ler, e no dia em que
/// ela tiver conteúdo ele já não olha para lá.
fn paint_contrast(ctx: &mut PaintCtx, theme: Theme, x: f32, w: f32, mut y: f32) -> f32 {
    let failing = failing_pairs(theme);
    if failing.is_empty() {
        return y;
    }
    // ⚠️ `Warn`, nunca `Danger`: o app **continua correcto** — o que quebrou foi uma promessa de
    // legibilidade sobre uma escolha que o artista pode desfazer num clique.
    let warn = resolve(ColorToken::Warn, theme);
    // ⚠️ `paint_text_block` (que PINTA e devolve a altura gasta), nunca um avanço fixo: estas
    // linhas têm comprimento variável e quebram, e estimar a altura é como a dica de duas linhas
    // do painel de física escreveu por cima da linha seguinte.
    y += paint_text_block(
        ctx.text_system,
        ctx.scene,
        tr("panel.tokens.contrast.title"),
        x,
        y,
        TypeToken::Sm.px(),
        w,
        warn,
    );
    let on = tr("panel.tokens.contrast.on");
    for pair in &failing {
        // ⚠️ O CRITÉRIO viaja no texto e **não** passa pela i18n: `WCAG 2.2 AA 1.4.3` é o endereço
        // de uma norma, não uma frase — traduzi-lo tornaria improcurável exactamente a coisa que o
        // artista leva para a especificação.
        let line = format!(
            "{} {on} {} - {:.1}:1, need {:.1} ({})",
            pair.fg.key(),
            pair.bg.key(),
            pair.ratio(theme),
            pair.min_ratio,
            pair.criterion,
        );
        y += paint_text_block(
            ctx.text_system,
            ctx.scene,
            &line,
            x,
            y,
            TypeToken::Xs.px(),
            w,
            warn,
        );
    }
    y + Spacing::Sm.px()
}

/// `[swatch]  chave-do-token  [→ alvo]      [⚠] [elo] [Reset]`
fn paint_token_row(
    ctx: &mut PaintCtx,
    theme: Theme,
    row: usize,
    token: ColorToken,
    // ⚠️ A CAIXA da linha, e não três escalares soltos: `x`/`w`/`y` viajavam sempre juntos e
    // sempre na mesma ordem, que é a assinatura de um tipo por nascer.
    box_: Rect,
    armed: bool,
) -> f32 {
    let (x, w, y) = (box_.x, box_.w, box_.y);
    let slot = color_override(theme, token);
    let authored = slot.is_some();
    let swatch_w = SwatchSize::Md.px();
    let font = TypeToken::Sm.px();

    let swatch_rect = Rect::new(x, y, swatch_w, ROW_H_PX);
    // ⚠️ A swatch mostra a cor EFETIVA (a que o app está a usar), nunca a de fábrica sob um token
    // autorado: uma swatch que afirmasse um valor que o desenho não usa é a rachura que a row de
    // Token do vetor já documenta.
    let colour = token.resolve(theme);
    let sw = ColorSwatch::new(
        ids::tokens_swatch_id(row),
        "Design token colour",
        [colour.r, colour.g, colour.b, colour.a],
    )
    .size(SwatchSize::Md);
    paint_color_swatch(&sw, swatch_rect, ctx.scene, theme);
    ctx.host
        .hit_index_mut()
        .register(ids::tokens_swatch_id(row), swatch_rect);

    // ⚠️ Os DOIS lados de um par falhado são marcados, e é o que a `involves` já decide: escurecer
    // o FUNDO quebra a legibilidade do TEXTO, e marcar só o texto mandaria o artista consertar o
    // token que ele não mexeu.
    let flagged = token_is_in_a_failing_pair(theme, token);
    let mark_tail = if flagged {
        MARK_W + Spacing::Xxs.px()
    } else {
        0.0
    };
    let label_x = x + swatch_w + Spacing::Sm.px();
    // ⚠️ A marca reserva a PRÓPRIA coluna em vez de flutuar sobre o rótulo: o elo e o Reset ficam
    // onde estavam (colunas estáveis entre linhas) e quem cede largura é o texto, que já é curto.
    let tail = if authored { RESET_W } else { 0.0 } + LINK_W + mark_tail + Spacing::Xs.px();
    let label_w = (w - swatch_w - Spacing::Sm.px() - tail).max(1.0);
    // A cor do rótulo DIZ se a linha está autorada — sem isso, "este está diferente da fábrica"
    // só se descobre carregando em Reset e vendo o que muda.
    let label_token = if authored {
        ColorToken::Accent
    } else {
        ColorToken::Text1
    };
    // ⚠️ Uma linha que SEGUE outra tem de DIZER quem ela segue: a swatch mostra a cor efetiva, e
    // sem o nome do alvo o artista vê um valor que não obedece ao picker sem nada explicar por quê
    // — a pior UI possível, e a razão pela qual o plano exige isto por escrito.
    let label = match slot {
        Some(TokenValue::Alias(target)) => format!("{}  -  {}", token.key(), target.key()),
        _ => token.key().to_string(),
    };
    paint_text(
        ctx.text_system,
        ctx.scene,
        &label,
        label_x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        label_w,
        resolve(label_token, theme),
    );

    // O elo é oferecido em TODA linha — qualquer token pode seguir qualquer outro, e esconder o
    // botão em linhas não-autoradas tornaria o gesto alcançável só onde ele já foi feito.
    let link_x = x + w - LINK_W - if authored { RESET_W } else { 0.0 };
    paint_link_button(ctx, theme, row, link_x, y, armed);

    if flagged {
        // A marca vive NA linha porque o resumo do topo sai de vista assim que o artista rola —
        // e a lista tem uma linha por token do design system.
        let rect = Rect::new(
            link_x - MARK_W - Spacing::Xxs.px(),
            y + (ROW_H_PX - MARK_W) * 0.5,
            MARK_W,
            MARK_W,
        );
        paint_icon(
            ctx.scene,
            IconId::Warning,
            rect,
            resolve(ColorToken::Warn, theme),
            StrokeToken::Default.px(),
        );
    }

    if authored {
        command(
            ctx,
            ids::tokens_reset_id(row),
            tr("panel.tokens.reset"),
            x + w - RESET_W,
            RESET_W,
            y,
        );
    }
    y + ROW_H_PX + Spacing::Xxs.px()
}

/// O botão de elo da linha — **Pressed enquanto armado**, para o artista ver de onde o gesto saiu.
fn paint_link_button(ctx: &mut PaintCtx, theme: Theme, row: usize, x: f32, y: f32, armed: bool) {
    let id = ids::tokens_link_id(row);
    let rect = Rect::new(x, y + (ROW_H_PX - LINK_W) * 0.5, LINK_W, LINK_W);
    let state = if armed {
        (ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)
    } else {
        ctx.host.store().button_visual(id)
    };
    paint_icon_button(
        rect,
        IconGlyph::Builtin(IconId::Link),
        IconButtonStyle::Compact,
        state,
        ctx.scene,
        theme,
    );
    ctx.host.hit_index_mut().register(id, rect);
}

/// Um botão de ação simples — o mesmo desenho do irmão no painel de física.
pub(crate) fn command(
    ctx: &mut PaintCtx,
    id: ph2d_a11y::NodeId,
    label: &str,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    command_at(ctx, id, label, Rect::new(x, y, w, ROW_H_PX));
    y + ROW_H_PX
}

/// **DOIS comandos na mesma linha** — o par de interop DTCG (plano UI/UX W9).
///
/// ⚠️ Lado a lado, e não empilhados: importar e exportar são a MESMA operação em dois sentidos,
/// e separá-los em duas linhas faria o artista procurar o segundo em vez de o ver ao lado do
/// primeiro. Os dois chamam o mesmo [`command_at`] — uma segunda cópia da pintura de um botão
/// seria a que diverge no dia em que o estilo do botão mudar.
fn command_pair(
    ctx: &mut PaintCtx,
    left: (ph2d_a11y::NodeId, &str),
    right: (ph2d_a11y::NodeId, &str),
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let gap = Spacing::Xs.px();
    let half = ((w - gap) * 0.5).max(0.0);
    command_at(ctx, left.0, left.1, Rect::new(x, y, half, ROW_H_PX));
    command_at(
        ctx,
        right.0,
        right.1,
        Rect::new(x + half + gap, y, half, ROW_H_PX),
    );
    y + ROW_H_PX
}

/// Um comando num rect dado — a pintura e o registro de hit, num sítio só.
fn command_at(ctx: &mut PaintCtx, id: ph2d_a11y::NodeId, label: &str, rect: Rect) {
    let theme = ctx.host.theme();
    let state = ctx.host.store().button_visual(id);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (_, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_button(
        &Button::new(id, label).visual(state),
        rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, rect);
}

/// O nome do modo, como o artista o conhece.
const fn theme_label(theme: Theme) -> &'static str {
    match theme {
        Theme::Forge => "Forge",
        Theme::Workshop => "Workshop",
        Theme::Sunstone => "Sunstone",
        Theme::Blueprint => "Blueprint",
    }
}

fn paint_scrollbar_and_publish(
    ctx: &mut PaintCtx,
    body_rect: Rect,
    content_h: f32,
    body_h: f32,
    scroll: f32,
    theme: Theme,
) {
    if scrollbar_is_needed(content_h, body_h) {
        let track = scrollbar_track_rect(body_rect);
        let thumb = scrollbar_thumb_rect(track, scroll, content_h, body_h);
        paint_scrollbar(
            body_rect,
            scroll,
            content_h,
            body_h,
            ctx.host.store().scrollbar_visual(TOKENS_SCROLLBAR_ID),
            ctx.scene,
            theme,
        );
        ctx.host
            .hit_index_mut()
            .register(TOKENS_SCROLLBAR_ID, thumb);
    }
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::TOKENS_PANEL, content_h);
    store.set_panel_visible_h(ids::TOKENS_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(ids::TOKENS_PANEL) > max_scroll {
        store.set_panel_scroll(ids::TOKENS_PANEL, max_scroll);
    }
}
