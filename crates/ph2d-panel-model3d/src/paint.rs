//! O corpo do painel. Percorre o **retrato** que o shell publicou — a mesma lista que o `event`
//! consulta —, então o que é pintado e o que despacha não podem discordar.

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{paint_text_block, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::paint_segmented_group_adaptive;
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::widget::{
    MODEL3D_SCROLLBAR_ID, paint_scrollbar, scrollbar_is_needed, scrollbar_thumb_rect,
    scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_field::Bound;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

use crate::state::{self, Model3dPanelState};
use crate::{Model3dPanel, populate::MAX_MODES, populate::MAX_ROWS};

/// ⭐⭐ **O tecto de uma linha SEM parede, para efeito de DIGITAÇÃO.**
///
/// ⚠️ Ele não é uma parede nova: é um número grande o bastante para nunca ser a resposta a um gesto
/// real, e finito para o campo continuar a ter uma faixa (o stepper e o piso saem dela). *Um tecto
/// que só existe para não haver tecto tem de dizê-lo.*
pub(crate) const TETO_DIGITAVEL: f64 = 1.0e6;

/// A goteira do rótulo. Os rótulos aqui são o **tipo do nó** ("Union", "Cylinder"), não uma frase.
pub(crate) const LABEL_COL_W: f32 = 72.0; // LITERAL-PX-OK: panel grid metric (label gutter width)

/// Em quantos passos o arrasto atravessa o curso de uma linha.
///
/// ⚠️ **Em centésimos do CURSO, e não num passo absoluto**: um passo fixo seria grosseiro num filete
/// de 0,02 e fino demais num ângulo de 360 de curso. ⭐ É daqui que sai o passo, e é o passo que
/// decide quantas casas a linha mostra ([`decimals_for_step`]) — os dois números têm de vir da mesma
/// fonte, senão a tela deixa de acompanhar o arrasto sem que nada pareça errado.
pub(crate) const STEPS_ACROSS_THE_RANGE: f32 = 100.0; // LITERAL-PX-OK: granularidade de arrasto, não métrica de design

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
pub(crate) fn decimals_for_step(step: f32) -> usize {
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
    {}

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

    // ⭐⭐⭐ **O CORPO ROLA** (report do Enio, 2026-08-27: *«o painel 3d Model precisa de scroll e
    // barra de scroll»*).
    //
    // ⛔ **Ele já recortava e nunca rolava, que é a pior das três formas:** um painel sem recorte
    // desenha por cima do título e vê-se; um que recorta e rola funciona; **um que recorta e não
    // rola esconde os controles e não diz nada.** O rodapé e as fileiras de parâmetros de um
    // documento com vários nós ficavam inalcançáveis, sem sinal nenhum de que existiam.
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll = ctx.host.store().panel_scroll(ids::MODEL3D_PANEL);
    ctx.scene.push_clip(&rect_to_vello(body_rect));
    // ⚠️ **UMA BANDA, DOIS CONSUMIDORES** — a lei que o painel do Motion pagou. O `push_clip` da
    // cena recorta o **DESENHO**; sem o gémeo no `HitIndex`, uma fileira rolada para cima continua
    // **registada** onde ninguém a vê, e o hit-rect dela sobe para a faixa do TÍTULO. *Enquanto
    // nada rolava isto era inofensivo por aritmética; ligar a rolagem é o dia em que passa a
    // morder.*
    ctx.host.hit_index_mut().push_clip(body_rect);
    let mut y = body_top - scroll;
    // ⭐ **O seletor do verbo, no topo** — é a porta que se encontra sem saber que existe. As
    // teclas `G`/`R`/`S` são a outra, e são para quem já sabe.
    y = paint_chips(ctx, &snapshot.modes, ids::model3d_mode_button, x, w, y);
    // ⭐ **Em que eixos** — do mundo, ou do próprio objeto. Fica por baixo do verbo porque ele
    // qualifica o verbo: um referencial sem verbo não quer dizer nada.
    y = paint_chips(ctx, &snapshot.frames, ids::model3d_frame_button, x, w, y);
    // ⭐⭐ **O que o LAÇO faz ao que apanha** (W112), logo abaixo do referencial — os dois
    // qualificam um GESTO do canvas, e não a forma escolhida. ⚠️ A nota vem antes porque «Add» e
    // «Subtract» sozinhos não dizem *a quê*: um chip sem sujeito lê-se ao contrário.
    if !snapshot.selects.is_empty() {
        y = paint_note(ctx, tr("panel.model3d.select.title"), x, w, y);
        y = paint_chips(ctx, &snapshot.selects, ids::model3d_select_button, x, w, y);
    }
    // ⭐ **Criar e combinar** — sem estes dois, o módulo edita a cena que veio pronta e mais nada.
    y = paint_chips(ctx, &snapshot.adds, ids::model3d_add_button, x, w, y);
    y = paint_chips(ctx, &snapshot.ops, ids::model3d_op_button, x, w, y);
    // ⭐⭐⭐ **O VERBO DESTA FORMA**, logo abaixo da operação do grupo — porque é ela que ele
    // qualifica: *o grupo diz o padrão, a forma diz se o segue*.
    //
    // ⚠️ **A nota vem ANTES dos chips e NOMEIA a forma**, e não é cosmética: tocar um filho pode
    // acender o grupo inteiro no canvas, e sem o nome o artista escolhe o verbo sem saber de qual
    // das formas o painel fala. É a cura que o vetorial pagou em 2026-08-22, e a razão de a fileira
    // e o nome viajarem juntos no retrato.
    if let Some(subject) = &snapshot.verb_subject {
        y = paint_note(
            ctx,
            &format!("{}: {subject}", tr("panel.model3d.verb_of")),
            x,
            w,
            y,
        );
        y = paint_chips(ctx, &snapshot.verbs, ids::model3d_verb_button, x, w, y);
    }
    // ⭐⭐⭐ **O CARÁTER da mistura** (W99), logo abaixo — porque ele qualifica a junta que a fileira
    // de cima escolheu. ⚠️ Ela é **independente** do sujeito nomeado acima: uma OPERAÇÃO tem
    // carácter e não tem verbo, então a fileira aparece ali sozinha.
    y = paint_chips(
        ctx,
        &snapshot.characters,
        ids::model3d_character_button,
        x,
        w,
        y,
    );
    // ⭐ **O que se faz À forma depois de ela existir** — a casca e o afastamento, os dois verbos em
    // que a tese do módulo mais aparece (ver `ph2d_field::mods`).
    y = paint_chips(ctx, &snapshot.mods, ids::model3d_mod_button, x, w, y);
    y = paint_chips(ctx, &snapshot.acts, ids::model3d_act_button, x, w, y);
    // ⭐⭐ **A CÂMERA passa a ser alcançável** (W47) — as seis vistas nomeadas, a lente e o
    // enquadrar. ⚠️ Até aqui os três gestos existiam **só como teclas** (`Numpad1/3/7`, `Numpad5`,
    // `Home`), isto é, para quem já sabia que existem — e as vistas nem sequer existiam.
    //
    // ⚠️ Elas ficam **depois** das ações e antes da exportação de propósito: são sobre *olhar*, e o
    // que está acima é sobre *fazer*. O último bloco continua a ser a porta de saída.
    y = paint_chips(ctx, &snapshot.views, ids::model3d_view_button, x, w, y);
    y = paint_chips(ctx, &snapshot.camera, ids::model3d_camera_button, x, w, y);
    // ⭐ **A porta de SAÍDA**, no fim: é o último gesto de uma peça, e é a primeira vez que o módulo
    // troca resolução infinita por um número de triângulos (ver `crate::field3d_export` no shell).
    y = paint_chips(ctx, &snapshot.exports, ids::model3d_export_button, x, w, y);
    // ⭐ **ESTÁ ISOLADO, e quem o diz é a VISTA** (W44) — logo abaixo dos controles e acima dos
    // números, porque é uma afirmação sobre *o que se está a ver*, não sobre o que está escolhido.
    //
    // ⚠️ **Independente da seleção**, e é essa a correção: o único sinal anterior era o `active` do
    // chip *Isolate*, que compara o nó isolado com o **escolhido** — escolher outra coisa apagava-o,
    // e com a raiz escolhida a fileira inteira desaparece. *Um estado da vista não se anuncia por um
    // controle da seleção.*
    if let Some(name) = &snapshot.isolated {
        y = paint_note(
            ctx,
            &format!("{}: {name}", tr("panel.model3d.isolated")),
            x,
            w,
            y,
        );
    }
    if snapshot.rows.is_empty() {
        y = paint_note(ctx, tr("panel.model3d.empty"), x, w, y);
    }
    // ⚠️ **Corta na família, e o rodapé DIZ que cortou** — ver `MAX_ROWS`. Uma linha além dela
    // ficaria sem controle registado: pintada e morta sob o rato, que é a falha de paridade de
    // fiação na sua forma mais cara.
    for (slot, row) in snapshot.rows.iter().enumerate().take(MAX_ROWS) {
        // ⭐⭐⭐ **O CABEÇALHO DA SECÇÃO** (report do Enio, 2026-08-30) — ver `ParamRow::section`.
        if let Some(key) = row.section {
            y = paint_section(ctx, tr(key), x, w, y);
        }
        y = crate::paint_rows::paint_row(ctx, row, slot as u32, x, w, y);
    }
    y = paint_footer(ctx, &snapshot, x, w, y);
    // ⚠️ O `+ scroll` desfaz o deslocamento: a altura do conteúdo é do CONTEÚDO, e não de onde ele
    // calhou de ser desenhado. Sem ele o `max_scroll` encolheria a cada rolagem e o painel
    // empurraria o artista de volta para o topo.
    let content_h = y + scroll - body_top + PANEL_HEAD_PAD;
    state::set_last_content_h(content_h);
    ctx.scene.pop_layer();
    // ⚠️ O `pop` vem ANTES da barra, e de propósito: o polegar vive no corpo mas **não rola com
    // ele** — recortá-lo pela mesma banda seria correcto hoje e uma armadilha no dia em que ele
    // saísse um pixel.
    ctx.host.hit_index_mut().pop_clip();
    paint_scroll_chrome(ctx, body_rect, content_h, body_h, scroll, theme);
}

/// Desenha a barra de rolagem e **publica** o par `content_h`/`visible_h` que o dispatch da roda
/// consome.
///
/// ⚠️ **Publicar é a metade que não se vê e sem a qual a roda não faz nada:** o `dispatch_wheel`
/// deriva o `max_scroll` desses dois números, então um painel que recorta, desloca e desenha o
/// polegar — mas não publica — rola com o polegar e fica **inerte na roda**.
fn paint_scroll_chrome(
    ctx: &mut PaintCtx,
    body_rect: Rect,
    content_h: f32,
    body_h: f32,
    scroll: f32,
    theme: ph2d_tokens::Theme,
) {
    if scrollbar_is_needed(content_h, body_h) {
        let track = scrollbar_track_rect(body_rect);
        let thumb = scrollbar_thumb_rect(track, scroll, content_h, body_h);
        paint_scrollbar(
            body_rect,
            scroll,
            content_h,
            body_h,
            ctx.host.store().scrollbar_visual(MODEL3D_SCROLLBAR_ID),
            ctx.scene,
            theme,
        );
        ctx.host
            .hit_index_mut()
            .register(MODEL3D_SCROLLBAR_ID, thumb);
    }
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::MODEL3D_PANEL, content_h);
    store.set_panel_visible_h(ids::MODEL3D_PANEL, body_h);
    // ⚠️ O clamp existe porque o conteúdo ENCOLHE: apagar um nó na Hierarquia tira fileiras, e um
    // painel rolado até ao fim abriria em branco.
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(ids::MODEL3D_PANEL) > max_scroll {
        store.set_panel_scroll(ids::MODEL3D_PANEL, max_scroll);
    }
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

/// Texto puro — um fato, não um controle. Sem hit-index: uma affordance que ele não pode honrar
/// seria pior do que nenhuma.
/// ⭐⭐⭐ **O cabeçalho de uma secção de números** — o nome de quem é dono das linhas abaixo.
///
/// # ⛔ O report que o obrigou
///
/// Enio, 2026-08-30: *«nada aparece torcido, apesar dos sliders terem algum efeito. O modificador
/// deveria ter sua própria seção no painel»*. As linhas de um modificador vinham **no fim da mesma
/// lista** das dimensões da forma, sem nada a separá-las — com uma casca e uma torção empilhadas o
/// painel mostrava seis números seguidos e nenhum dizia de quem era.
///
/// ⚠️ *Um controle que o artista não consegue atribuir lê-se exactamente como uma feature partida:*
/// ele arrasta o que julga ser o da torção, vê outra coisa mudar, e conclui que a torção não faz
/// nada.
///
/// ⚠️ **Não entra no índice de acerto** — é rótulo, não controle. Um cabeçalho clicável prometeria
/// recolher a secção, e isso não existe.
fn paint_section(ctx: &mut PaintCtx, text: &str, x: f32, w: f32, y: f32) -> f32 {
    let font = TypeToken::Xs.px();
    let theme = ctx.host.theme();
    // Uma folga acima separa a secção da anterior; abaixo fica colada às linhas que ela nomeia —
    // é a proximidade que diz de quem elas são, e não o texto.
    let top = y + Spacing::Sm.px();
    let used = paint_text_block(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        top,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    top + used.max(font) + Spacing::Xs.px()
}

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
/// ⭐⭐ **Ela decide se o campo numérico CLAMPA** — ver a nota no registo da faixa. Uma parede é um
/// facto da peça (um filete que não cabe); uma sugestão é o alcance do gesto que a vista escolheu, e
/// clampar a digitação nela torna um valor legítimo indigitável.
///
/// ⚠️ Falta ainda **desenhar** a diferença (uma parede e uma sugestão não se pintam igual); isso é
/// outra wave.
pub(crate) fn bound_is_wall(b: Bound) -> bool {
    matches!(b, Bound::Hard(_))
}

#[cfg(test)]
#[path = "paint_tests.rs"]
mod tests;

/// ⚠️ **Irmão de arquivo, e não uma secção do `paint_tests`:** aqueles medem o que se **lê** numa
/// linha (a formatação); estes medem a **costura da rolagem**, que é outra pergunta e tem outra
/// fixtura (um host de painel a sério).
#[cfg(test)]
#[path = "paint_scroll_tests.rs"]
mod scroll_tests;
