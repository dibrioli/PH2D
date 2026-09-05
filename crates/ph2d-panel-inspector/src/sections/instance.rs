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
    // ⚠️ As linhas dos componentes overridados ficam na conta: elas são NOMES do catálogo, curtos
    // por construção — e medir cada uma custaria um layout por quadro por linha.
    let head_h = super::text_h(text_system, &info.provenance(), font, tw_probe, line)
        + super::text_h(text_system, &info.summary(), small, tw_probe, line);
    // ⚠️ **As fileiras da ESCADA entram na conta como as outras** — elas são botões de altura
    // fixa, e o `apply_rows` já responde se elas existem (ver o modelo: sem excepção nesta peça,
    // ou com um degrau só, não há fileira nenhuma).
    let ladder = info.apply_rows();
    let beyond = usize::from(!ladder.is_empty() && info.apply_levels_beyond > 0);
    let rows = info.overridden.len()
        + ladder.len()
        + beyond
        + info.orphan_rows.len()
        + usize::from(info.orphans() > 0);
    let card_h = CARD_PAD * 2.0 + head_h + line * rows as f32;
    let card = Rect::new(x, y, w, card_h);
    fill_rounded_rect(
        scene,
        card,
        Radius::Md.px(),
        resolve(ColorToken::Bg2, theme),
    );

    let tx = x + CARD_PAD;
    let tw = (w - CARD_PAD * 2.0).max(0.0);
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
        paint_text(
            text_system,
            scene,
            &format!("\u{2022} {name}"),
            tx + Spacing::Sm.px(),
            ty,
            font,
            tw,
            resolve(ColorToken::Text1, theme),
        );
        ty += line;
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

    // ⭐⭐⭐ **AS EXCEPÇÕES SEM ALVO, uma por linha** (F5 critério 3) — o que ficou de uma peça que
    // o mestre apagou.
    //
    // ⚠️ **Elas vêm em `Text2`, e as vivas em `Text1`:** as de cima são o que esta peça TEM, estas
    // são o que sobrou de peças que já não existem. Pintá-las iguais faria o artista ler uma lista
    // só, e o botão logo abaixo apaga **apenas** estas.
    //
    // ⚠️ **A ordem é a do mapa, que agrupa por PEÇA por construção** (a chave ordena `piece` antes
    // de `type_id`) — é o agrupamento que torna a lista legível quando duas peças morreram.
    for row in &info.orphan_rows {
        paint_text(
            text_system,
            scene,
            &format!("\u{2022} {}", row.label()),
            tx + Spacing::Sm.px(),
            ty,
            font,
            tw,
            resolve(ColorToken::Text2, theme),
        );
        ty += line;
    }

    // ⭐ O gesto dos ÓRFÃOS — e ele **só aparece quando existem**: um botão permanentemente inerte
    // é ruído que o artista aprende a ignorar.
    if info.orphans() > 0 {
        let host = Rect::new(tx, ty, tw, line);
        hit_index.register(ids::INSP_INSTANCE_CLEAR_ORPHANS, host);
        let button = Button::new(
            ids::INSP_INSTANCE_CLEAR_ORPHANS,
            format!("Clear {} unused override(s)", info.orphans()),
        )
        .kind(ButtonKind::Default)
        .visual(store.button_visual(ids::INSP_INSTANCE_CLEAR_ORPHANS));
        paint_button(&button, host, scene, text_system, theme);
    }
    y + card_h + SECTION_BOTTOM_PAD_PX
}
