//! ⭐⭐⭐ **O CARTÃO de instância** (ADR-0164 / F5) — *«o que esta cópia tem de diferente da
//! receita»*, no **topo** do Inspector.
//!
//! Ver [`ph2d_editor_core::screens::hero::InspectorInstanceInfo`] para o buraco que ele fecha: o
//! modelo de override existe desde a F4.4 e era **inteiramente invisível** — o artista lia-o pelo
//! COMPORTAMENTO (a receita deixou de alcançar aquela peça), nunca por um sinal.
//!
//! # ⚠️ Por que é um CARTÃO no topo, e não uma seção no fim
//!
//! A 1.ª versão era uma seção como as outras, **por último**, com o argumento de que *«ela descreve
//! a relação do objeto com a biblioteca, não uma propriedade dele»*. Enio (2026-08-27):
//! *«essas mensagens não ficariam melhor num card no topo do inspector?»* — e o argumento corta
//! para o outro lado: é **contexto sobre o que o objeto É**, logo lê-se **antes** das propriedades.
//! *Um artista que só descobre no fim do painel que está a editar uma cópia já editou.* Figma e
//! Unity põem a mesma faixa no topo, os dois.
//!
//! ⛔ **E cartão, não seção:** sem cabeçalho, sem recolher, sem âncora de nota. Ele é *chrome que
//! avisa*, e **um aviso que se pode fechar não avisa**.

use super::*;
use ph2d_editor_core::screens::hero::InspectorInstanceInfo;

/// A margem de dentro do cartão — o que separa o texto da borda dele.
const CARD_PAD: f32 = 8.0; // LITERAL-PX-OK: inset do cartão, irmão do BODY_PAD do corpo

/// ⭐⭐ **As larguras e os tamanhos do cartão, derivados UMA vez.**
///
/// ⚠️ **Ela existe porque a altura e a pintura têm de concordar sobre o orçamento de quebra.** O
/// fundo é desenhado antes do conteúdo (senão cobre-o), logo a medição é uma passagem separada — e
/// uma largura calculada duas vezes diverge no dia em que só uma delas passar a descontar um botão.
/// *Foi exactamente essa divergência que pôs o botão dos órfãos por cima do texto.*
#[derive(Copy, Clone)]
pub(crate) struct CardMetrics {
    /// A esquerda do conteúdo.
    pub(crate) tx: f32,
    /// A largura do conteúdo, sem recuo.
    pub(crate) tw: f32,
    /// A direita do conteúdo — onde o `✕` de uma linha encosta.
    pub(crate) right: f32,
    /// O orçamento de quebra de uma linha de LISTA (já sem o recuo).
    pub(crate) list_tw: f32,
    /// O de uma linha de órfão (já sem o recuo **e** sem o `✕`).
    pub(crate) orphan_tw: f32,
    pub(crate) font: f32,
    pub(crate) small: f32,
    pub(crate) line: f32,
}

/// Pinta o cartão. Devolve o `y` de baixo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_instance_card(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    info: &InspectorInstanceInfo,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let font = TypeToken::Base.px();
    let small = TypeToken::Sm.px();
    let line = font + Spacing::Xs.px();
    // ⚠️ **A altura é MEDIDA antes de pintar**, e não somada enquanto se desenha: o fundo do cartão
    // tem de ser desenhado PRIMEIRO (senão cobre o texto), e por isso ele precisa de saber onde
    // acaba. *Um fundo pintado depois do conteúdo é o conteúdo apagado.*
    let tw_probe = (w - CARD_PAD * 2.0).max(0.0);
    // ⛔⛔ **A altura das DUAS primeiras linhas é MEDIDA, não contada** (report do Enio com foto,
    // 2026-08-31: *«Card com Labels emboladas»*).
    //
    // Elas são frases, não rótulos: `Instance of "…"` e o resumo quebram quando não cabem, e a
    // conta `line * rows` assumia uma linha cada. ⇒ a proveniência desenhava duas linhas e o resumo
    // era pintado **por cima da segunda**. *Uma altura contada em linhas mente sobre todo texto que
    // pode quebrar.*
    //
    // ⛔⛔⛔ **E as linhas de LISTA passaram a ser medidas TAMBÉM** (auditoria de 2026-09-05).
    //
    // A justificação que aqui esteve — *«elas são NOMES do catálogo, curtos por construção»* — era
    // verdadeira para os componentes overridados (o mais longo do catálogo tem **20** caracteres) e
    // **falsa** para as excepções sem alvo, que entraram nesta mesma conta em 2026-09-04: o rótulo
    // delas embrulha um `Name` que o **artista escreveu**, e um `Name` não tem tecto. Medido: com
    // `Sprite — was on "Left front suspension arm assembly"` o botão dos órfãos ficava em `y = 198`
    // — **o mesmo `y` do nome curto** —, ou seja pintado por cima da 2.ª linha do texto. É a foto
    // do Enio de 2026-08-31 (*«Card com Labels emboladas»*) por outra porta.
    //
    // ⚠️ **E o argumento nunca foi só sobre a string:** embrulhar é função da LARGURA, e a largura
    // deste painel não é uma constante. *Quem empilha texto de comprimento variável tem de
    // perguntar ao pintor quanto ele gastou* — a lei já estava escrita no `paint_text_block`.
    let head_h = super::text_h(text_system, &info.provenance(), font, tw_probe, line)
        + super::text_h(text_system, &info.summary(), small, tw_probe, line);
    // ⚠️ **As fileiras da ESCADA entram na conta como as outras** — elas são botões de altura
    // fixa, e o `apply_rows` já responde se elas existem (ver o modelo: sem excepção nesta peça,
    // ou com um degrau só, não há fileira nenhuma).
    let ladder = info.apply_rows();
    let beyond = usize::from(!ladder.is_empty() && info.apply_levels_beyond > 0);
    // ⚠️ **A INDENTAÇÃO sai do orçamento de quebra** — ela entrava no `x` e não no `max_width`,
    // então uma linha embrulhada corria `Spacing::Sm` **para fora** da borda direita do cartão.
    // *Um texto recuado com o orçamento inteiro é um texto que transborda pela largura do recuo.*
    //
    // ⏳ **Correcção MEDIDA e por GATEAR, e o motivo é o instrumento:** a mutação que a desfaz
    // sobrevive a todos os gates deste cartão, porque o oráculo que eles têm é *onde o botão
    // aterra* — e o transbordo é **horizontal**. Vê-lo pedia a extensão dos glifos, que o
    // `MockPanelHost` não expõe (ele conta glifos e devolve rects, não caixas de texto). ⛔ Fica
    // escrito em vez de silencioso: *uma linha sem régua que se declara é dívida; sem a declaração
    // é uma armadilha.*
    let at = CardMetrics {
        tx: x + CARD_PAD,
        tw: (w - CARD_PAD * 2.0).max(0.0),
        right: x + w - CARD_PAD,
        list_tw: (tw_probe - Spacing::Sm.px()).max(0.0),
        // O `✕` de cada órfão é quadrado, da altura da linha — e o texto dela paga essa largura.
        orphan_tw: (tw_probe - Spacing::Sm.px() - line).max(0.0),
        font,
        small,
        line,
    };
    let list_h: f32 = info
        .overridden
        .iter()
        .map(|n| {
            super::text_h(
                text_system,
                &format!("\u{2022} {n}"),
                font,
                at.list_tw,
                line,
            )
        })
        .sum::<f32>()
        + instance_orphans::rows_height(text_system, info, font, at.orphan_tw, line);
    let fixed_rows =
        ladder.len() + beyond + instance_removed::rows(info) + instance_orphans::fixed_rows(info);
    let card_h = CARD_PAD * 2.0 + head_h + list_h + line * fixed_rows as f32;
    let card = Rect::new(x, y, w, card_h);
    fill_rounded_rect(
        scene,
        card,
        Radius::Md.px(),
        resolve(ColorToken::Bg2, theme),
    );

    let (tx, tw) = (at.tx, at.tw);
    let mut ty = y + CARD_PAD;
    // A linha de proveniência: **o que este objeto é**, e de que receita nasceu. É a única
    // superfície que o diz — a Hierarquia mostra a árvore, não o vínculo. ⚠️ A frase sai do modelo
    // (`provenance()`): *Instance* e *Variant* são estados diferentes, e escrevê-la aqui poria a
    // escolha num sítio que nenhum gate de modelo alcança.
    paint_text(
        text_system,
        scene,
        &info.provenance(),
        tx,
        ty,
        font,
        tw,
        resolve(ColorToken::Text1, theme),
    );
    // ⚠️ **O avanço é o MEDIDO** — ver a nota da altura: um `+= line` fixo aqui é exactamente o que
    // punha o resumo por cima da 2.ª linha da proveniência.
    ty += super::text_h(text_system, &info.provenance(), font, tw, line);
    paint_text(
        text_system,
        scene,
        &info.summary(),
        tx,
        ty,
        small,
        tw,
        resolve(ColorToken::Text2, theme),
    );
    ty += super::text_h(text_system, &info.summary(), small, tw, line);

    // ⚠️ **Uma linha por componente overridado, pelo NOME que o `+` usa.** Sem elas o artista sabe
    // que «alguma coisa» está diferente e não sabe o quê — que é metade do defeito.
    for name in &info.overridden {
        let text = format!("\u{2022} {name}");
        paint_text(
            text_system,
            scene,
            &text,
            tx + Spacing::Sm.px(),
            ty,
            font,
            at.list_tw,
            resolve(ColorToken::Text1, theme),
        );
        ty += super::text_h(text_system, &text, font, at.list_tw, line);
    }

    // ⭐⭐⭐ **A ESCADA do *Aplicar*** (F5 critério 4) — um botão por receita alcançável, da mais
    // **externa** para a mais **interna**.
    //
    // ⚠️ **A ordem é uma decisão de produto:** a de fora primeiro, porque é a que alcança MENOS
    // (só as cópias daquela receita) — *o gesto mais contido lê-se antes do mais amplo*. É também
    // a ordem em que o *«Apply All aplica sempre ao mais externo»* do Unity põe o caso comum na
    // primeira linha.
    //
    // ⚠️ **O rótulo sai do MODELO** (`ApplyChoice::label`), e não daqui: *Apply to* e *Apply as
    // override in* dizem coisas diferentes, e escrever a escolha num pintor poria a lei num sítio
    // que nenhum gate de modelo alcança.
    for (i, choice) in ladder.iter().enumerate() {
        // ⚠️ A tabela de ids tem tecto, e o `get` é o que impede um índice fora dela — o que sobra
        // é CONTADO na linha seguinte, nunca truncado em silêncio.
        let Some(&id) = ids::INSP_INSTANCE_APPLY_LEVEL.get(i) else {
            break;
        };
        let host = Rect::new(tx, ty, tw, line);
        hit_index.register(id, host);
        let button = Button::new(id, choice.label())
            .kind(ButtonKind::Default)
            .visual(store.button_visual(id));
        paint_button(&button, host, scene, text_system, theme);
        ty += line;
    }
    if beyond > 0 {
        paint_text(
            text_system,
            scene,
            &format!("+{} deeper", info.apply_levels_beyond),
            tx + Spacing::Sm.px(),
            ty,
            small,
            tw,
            resolve(ColorToken::Text2, theme),
        );
        ty += line;
    }

    // ⭐⭐⭐ **AS PEÇAS que esta cópia RECUSOU** (F5.10) — o bloco inteiro vive no irmão
    // [`super::instance_removed`]. ⚠️ Ele vem ANTES dos órfãos de propósito: uma peça recusada é
    // uma diferença VIVA (o artista pode desfazê-la, e a peça existe na receita), e uma excepção
    // sem alvo é um resto. *O que se pode desfazer lê-se antes do que só se pode largar.*
    ty = instance_removed::paint(scene, text_system, theme, hit_index, store, info, at, ty);

    // ⭐⭐⭐ **AS EXCEPÇÕES SEM ALVO, e o gesto de cada uma** — o bloco inteiro vive no irmão
    // [`super::instance_orphans`], cortado por assunto quando o `✕` por linha estourou o tecto de
    // 200 LOC desta função. *Um tecto paga-se com um corte.*
    let _ = instance_orphans::paint(scene, text_system, theme, hit_index, store, info, at, ty);
    y + card_h + SECTION_BOTTOM_PAD_PX
}
