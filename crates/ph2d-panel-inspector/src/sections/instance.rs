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

/// Que fatia da largura o NOME do eixo leva.
///
/// ⚠️ **Uma fracção com tecto, e não um número fixo**: num painel estreito um rótulo fixo comeria
/// a fileira toda e os chips ficariam ilegíveis; num painel largo um rótulo proporcional afastaria
/// a pergunta das respostas. ⇒ ele cresce até caber `Size`/`State` e pára.
const AXIS_LABEL_FRACTION: f32 = 0.28; // LITERAL-PX-OK: proporção do rótulo, domínio do cartão

/// O tecto do rótulo, em px.
const AXIS_LABEL_MAX_PX: f32 = 72.0; // LITERAL-PX-OK: tecto do rótulo, domínio do cartão

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
    let variant_rows = info.axes.len();
    let rows = 2 + info.overridden.len() + usize::from(info.orphans > 0) + variant_rows;
    let card_h = CARD_PAD * 2.0 + line * rows as f32;
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
    ty += line;
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
    ty += line;

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

    // ⭐⭐⭐ **A fileira das VARIANTES** (F5, critério 2) — que versão do componente esta cópia é.
    //
    // ⚠️ **Ela só existe com DUAS ou mais**, e a decisão está a montante (o construtor devolve a
    // lista vazia): um chip único, já escolhido, não escolhe nada. *Um valor que não leva a lado
    // nenhum não é oferecido.*
    // ⭐⭐⭐ **UMA FILEIRA POR PERGUNTA** (a fatia dos eixos, 2026-08-30) — `Size`, `State`, … e no
    // modo plano uma só chamada `Variant`, que é exactamente a fileira de antes.
    //
    // ⚠️ **O NOME do eixo é pintado à esquerda da fileira**, e não é decoração: sem ele duas
    // fileiras de chips são duas listas sem pergunta. ⛔ No modo plano ele diz `Variant`, que é o
    // que a família de nomes crus de facto oferece.
    for (a, ax) in info.axes.iter().enumerate() {
        let Some(row_ids) = ids::INSP_INSTANCE_AXIS_OPTION.get(a) else {
            break;
        };
        let label_w = (tw * AXIS_LABEL_FRACTION).min(AXIS_LABEL_MAX_PX);
        paint_text(
            text_system,
            scene,
            &ax.name,
            tx,
            ty + (line - small) * 0.5,
            small,
            label_w,
            resolve(ColorToken::Text2, theme),
        );
        let chips_x = tx + label_w;
        let chips_w = (tw - label_w).max(0.0);
        let n = ax.options.len();
        let gap = Spacing::Xs.px();
        let cw = ((chips_w - gap * (n.saturating_sub(1)) as f32) / n as f32).max(0.0);
        for (i, v) in ax.options.iter().enumerate() {
            let Some(&id) = row_ids.get(i) else {
                break;
            };
            let host = Rect::new(chips_x + (cw + gap) * i as f32, ty, cw, line);
            hit_index.register(id, host);
            let button = Button::new(id, v.label.clone())
                // ⚠️ A vigente é `Accent` — é o **estado**, e não uma decoração: sem ela a fileira
                // mostra as opções e esconde a resposta.
                .kind(if v.current {
                    ButtonKind::Accent
                } else {
                    ButtonKind::Default
                })
                .visual(store.button_visual(id));
            paint_button(&button, host, scene, text_system, theme);
        }
        ty += line;
    }

    // ⭐ O gesto dos ÓRFÃOS — e ele **só aparece quando existem**: um botão permanentemente inerte
    // é ruído que o artista aprende a ignorar.
    if info.orphans > 0 {
        let host = Rect::new(tx, ty, tw, line);
        hit_index.register(ids::INSP_INSTANCE_CLEAR_ORPHANS, host);
        let button = Button::new(
            ids::INSP_INSTANCE_CLEAR_ORPHANS,
            format!("Clear {} unused override(s)", info.orphans),
        )
        .kind(ButtonKind::Default)
        .visual(store.button_visual(ids::INSP_INSTANCE_CLEAR_ORPHANS));
        paint_button(&button, host, scene, text_system, theme);
    }
    y + card_h + SECTION_BOTTOM_PAD_PX
}
