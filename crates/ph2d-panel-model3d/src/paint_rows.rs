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

use ph2d_editor_core::paint::{paint_text_elided, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::panel_chrome::paint_segmented_group_adaptive;
use ph2d_editor_core::widget::{NUMBER_INPUT_MIN_W_PX, paint_slider_with_chip_layout_adaptive};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, TypeToken};

use crate::paint::{STEPS_ACROSS_THE_RANGE, TETO_DIGITAVEL, bound_is_wall, decimals_for_step};
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
    // ⭐⭐⭐ **A NOTA DO SUJEITO vem ANTES da linha** — ver [`ParamRow::subject`]. Ela diz sobre o
    // quê a linha escreve quando isso não é óbvio (uma escrita que espalha), e vem primeiro pela
    // mesma razão da fileira do verbo: *um controlo sem sujeito lê-se ao contrário.*
    let y = match &row.subject {
        Some(nota) => crate::paint::paint_note(ctx, nota, x, w, y),
        None => y,
    };
    // ⭐⭐⭐ **UMA COR NÃO É UM NÚMERO** — ver [`ParamRow::swatch`] (Enio, 2026-09-14). Mesma lei da
    // escolha logo abaixo: substitui o controle, não o acompanha.
    //
    // ⚠️ **O CAMPO da cor sai do próprio `param` da linha**, e não de um índice guardado ao lado da
    // amostra: a âncora **é** o primeiro canal, e duas respostas à mesma pergunta divergem no dia em
    // que uma terceira cor entrar. ⛔ E uma amostra sobre um param que **não** é material cai para o
    // controlo normal — inexprimível hoje, e melhor do que uma cor inventada.
    //
    // ⛔⛔ **Ela vem ANTES do teste de `live`, e a ordem é a ordem do report** (Enio, 2026-09-14:
    // *«não devem desaparecer, mas apenas serem inativados»*): uma amostra TRAVADA continua a ser
    // uma **amostra**, apagada. Depois do teste de `live` ela caía no [`paint_fact`] e o artista via
    // um **número** (`0,8`) onde estava uma cor — *a linha deixava de saltar de sítio e passava a
    // saltar de ESPÉCIE, que é a mesma queixa noutra escala.*
    //
    // ⚠️⚠️ **A condição era `Param::Material(campo)` e passou a ser a AMOSTRA** (14/09, a luz): com
    // o filtro pela família, uma linha com amostra de qualquer outra — a cor de uma lâmpada — caía
    // no slider e pintava o canal **vermelho** como um número. *Quem decide que isto é uma cor é o
    // `swatch`, e perguntar duas vezes deixa as duas respostas divergirem.*
    if let Some(rgb) = row.swatch {
        return paint_swatch(ctx, row, rgb, x, w, y);
    }
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
    let slider = crate::ids::model3d_radius_slider(slot);
    let chip = crate::ids::model3d_radius_chip(slot);
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
        ph2d_editor_core::widget::property_label_col_w(x, w),
        NUMBER_INPUT_MIN_W_PX,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y + used + ph2d_tokens::control_gap_px()
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
/// ⚠️ **A goteira é a MESMA do slider** (a porta `property_label_col_w`), e isso não é estética: o olho percorre
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
    // ⛔ **Corta, nunca quebra** — ver a nota do [`paint_fact`]. Os rótulos de escolha deste painel
    // são curtos (`Axis`), mas o defeito não é do comprimento de hoje: é do pintor.
    ph2d_editor_core::widget::paint_property_label(
        ctx.text_system,
        ctx.scene,
        tr(row.key),
        x,
        baseline,
        font,
        ph2d_editor_core::widget::property_label_col_w(x, w),
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
                crate::ids::model3d_choice_button(slot, i as u32),
            )
        })
        .collect();
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_segmented_group_adaptive(
        Rect::new(
            x + ph2d_editor_core::widget::property_label_col_w(x, w),
            y,
            (w - ph2d_editor_core::widget::property_label_col_w(x, w)).max(0.0),
            ROW_H_PX,
        ),
        &labels,
        ctx.scene,
        ctx.text_system,
        theme,
        store,
        hit_index,
    );
    y + used.max(ROW_H_PX) + ph2d_tokens::control_gap_px()
}

/// ⭐⭐⭐ **A LINHA-AMOSTRA** — o rótulo à esquerda, e na goteira do valor a cor que a peça tem
/// (Enio, 2026-09-14: *«em vez de 3 sliders de RGB, deveríamos ter uma caixa seletora de cor»*).
///
/// # ⭐ O selector não se constrói aqui: ele já existe, e é UM
///
/// A casa tem **um** selector de cor (o OKLCH com roda, canais RGB/HSV/OKLCH, hexadecimal, paletas
/// e conta-gotas), e um painel entra nele por **duas** linhas: registar o id como amostra
/// ([`WidgetStore::register_picker_swatch`]) e manter a cor dela em dia. O `Down` genérico do
/// `pointer_down` faz o resto — é o mesmo caminho do traço do Flip, do preenchimento do vetor e da
/// tinta do Painter. *Um segundo selector neste painel seria uma segunda resposta à mesma pergunta,
/// e a que envelhece.*
///
/// # ⚠️⚠️ QUEM MANDA NA COR MUDA CONFORME O SELECTOR ESTÁ ABERTO — e é aí que mora o defeito
///
/// | o selector está… | quem é a verdade | o que esta função faz |
/// |---|---|---|
/// | **fechado** | o **documento** | semeia a amostra com a cor do nó, todo quadro |
/// | **aberto nesta amostra** | o **selector** | lê o que ele escreveu e pede a edição |
///
/// ⛔ **Semear nos dois casos apagaria a escolha debaixo do dedo:** o `hero` espelha o valor vivo do
/// selector para `widget_color(id)` **antes** de os painéis pintarem, e um `set_widget_color` aqui
/// por cima devolveria a cor velha a cada quadro — a roda mover-se-ia e a cor não. *O mesmo par de
/// metades que o `brush_color_readback` do Painter já tem escrito.*
///
/// ⚠️ **E o «mudou?» pergunta-se em sRGB8** — ver [`ParamRow::swatch`]. Sem essa comparação a
/// função pediria uma edição **por quadro** enquanto o selector estivesse aberto: um passo de undo
/// por quadro, sobre uma cor que ninguém mexeu.
fn paint_swatch(ctx: &mut PaintCtx, row: &ParamRow, rgb: [u8; 3], x: f32, w: f32, y: f32) -> f32 {
    use ph2d_editor_core::widget::{ColorSwatch, SwatchSize, paint_color_swatch};

    let theme = ctx.host.theme();
    let font = TypeToken::Sm.px();
    let dim = resolve(ColorToken::Text2, theme);
    let baseline = y + (ROW_H_PX - font) * 0.5;
    // ⭐⭐ **A MESMA CAIXA ÚNICA das outras linhas** (report do Enio com foto, 14/09): rótulo à
    // esquerda, **amostra na coluna da direita** — e o rótulo CORTA em vez de quebrar.
    //
    // ⛔ A amostra ocupava a goteira inteira (`w − 72 ≈ 230 px`) e o rótulo vivia nos `72` da
    // esquerda: `Emission Color` não cabia lá e virava **duas linhas**, por cima da linha seguinte.
    // *Uma amostra tão larga lê-se como um campo de texto, e a coluna dos valores deixava de estar
    // alinhada com a dos números.*
    let coluna = ph2d_editor_core::widget::property_label_col_w(x, w).min(w);
    let rotulo_w = (w - coluna).max(0.0);
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        tr(row.key),
        x,
        baseline,
        font,
        rotulo_w,
        dim,
    );

    // ⛔⛔ **O id vem da ENTIDADE, e é o único deste painel que vem** — ver
    // [`crate::ids::model3d_color_swatch`]. Com o id da posição, escolher outra forma com o
    // selector aberto escreveria a cor da anterior na nova, em silêncio.
    //
    // ⚠️ **O `campo` é o índice DENTRO da família**, e as duas famílias que abrem cor — o material e
    // a luz — nunca coexistem na mesma entidade (o `params_of` responde uma OU a outra), logo o par
    // `(entidade, índice)` continua a ser único. *Se um dia uma entidade tiver as duas, este id tem
    // de crescer — e o sintoma seria o selector aberto numa a responder pela outra.*
    let campo = match row.param {
        ph2d_field::Param::Material(k) | ph2d_field::Param::Light(k) => k,
        // Uma amostra sobre um param sem índice não existe hoje; `0` é a resposta estável.
        _ => 0,
    };
    let id = crate::ids::model3d_color_swatch(row.entity, campo);
    // ⭐⭐⭐ **UMA AMOSTRA TRAVADA NÃO ABRE O SELECTOR, e nem sequer se REGISTA** — ordem do Enio
    // (14/09): ela fica **visível e inactiva**.
    //
    // ⛔⛔ **As três metades têm de sair juntas, e cada uma sozinha é um defeito diferente:** sem o
    // `register_picker_swatch` o selector não a reconhece; sem o `hit_index` ela não é clicável;
    // sem o `SwatchState::Disabled` ela **parece** clicável. *Uma amostra que parece viva e não
    // responde é o controlo morto na forma que o artista mais rapidamente lê como avaria.*
    if !row.live {
        return paint_dead_swatch(ctx, row, id, rgb, x, w, y);
    }
    let aberto = {
        let store = ctx.host.store_mut();
        store.register_picker_swatch(id);
        let aberto = store.picker_target() == Some(id);
        if aberto {
            // O selector é o dono: lê-se dele, e pede-se a edição só quando a cor de facto mudou.
            if let Some(escolhida) = store.widget_color(id) {
                let nova = [escolhida[0], escolhida[1], escolhida[2]];
                if nova != rgb {
                    crate::state::push_intent(crate::state::ModelIntent::SetColor {
                        entity: row.entity,
                        anchor: row.param,
                        srgb: nova,
                    });
                }
            }
        } else {
            // ⚠️ **Opaca**: o material não tem alfa, e um `255` inventado aqui é o único honesto —
            // ver [`ParamRow::swatch`]. Com outro valor a amostra pintaria o xadrez da
            // transparência sobre uma peça que é sólida.
            store.set_widget_color(id, [rgb[0], rgb[1], rgb[2], 255]);
        }
        aberto
    };

    // ⚠️ A altura é a da linha, para o painel não saltar de tamanho entre uma linha e a vizinha.
    let gutter = Rect::new(x + rotulo_w, y, coluna, ROW_H_PX);
    let swatch = ColorSwatch::new(id, tr(row.key), [rgb[0], rgb[1], rgb[2], 255])
        .size(SwatchSize::Md)
        // ⭐ **Aberto ⇒ focado**, que é o anel que a família moderna traça: com o selector a flutuar
        // sobre o canvas, é este anel que diz **qual** amostra ele está a editar.
        .state(if aberto {
            ph2d_editor_core::widget::SwatchState::Focused
        } else {
            ph2d_editor_core::widget::SwatchState::Normal
        });
    paint_color_swatch(&swatch, gutter, ctx.scene, theme);
    // ⚠️ **Sem isto a amostra é decoração**: quem decide que o `Down` abre o selector é o
    // `pointer_down`, e ele só vê o que o índice de acerto reclamou.
    ctx.host.hit_index_mut().register(id, gutter);
    y + ROW_H_PX + ph2d_tokens::control_gap_px()
}

fn paint_fact(ctx: &mut PaintCtx, row: &ParamRow, x: f32, w: f32, y: f32) -> f32 {
    let font = TypeToken::Sm.px();
    let theme = ctx.host.theme();
    let dim = resolve(ColorToken::Text2, theme);
    let baseline = y + (ROW_H_PX - font) * 0.5;
    // ⭐⭐⭐ **A CAIXA ÚNICA, também aqui** (report do Enio com foto, 2026-09-14: *«widgets sobrepostos
    // embolados, mas espaçados»*) — rótulo à ESQUERDA, valor à DIREITA, como na linha viva.
    //
    // ⛔⛔ **Ela punha o rótulo numa coluna de largura FIXA e o valor a seguir, e isso são DOIS
    // defeitos.** O primeiro é geométrico: a linha viva deixou de ter coluna externa de rótulo em
    // 2026-09-02 (*«a caixa única: rótulo à esquerda DENTRO, valor à direita DENTRO»*), e o
    // `label_w` daquele pintor é **ignorado de propósito** desde então — logo a linha travada era a
    // única do painel ainda desenhada com o modelo de três colunas. O segundo é o que a foto mostra.
    let valor = format!("{:.d$}", f64::from(row.value), d = decimals_for_step(1.0));
    let valor_w = ph2d_editor_core::widget::property_label_col_w(x, w).min(w);
    // ⛔⛔ **`paint_text_elided` e NUNCA `paint_text_block`** — o doc do primeiro nomeia este defeito
    // à letra: *«`paint_text` trata `max_width` como orçamento de QUEBRA, então um rótulo um pixel
    // largo demais vira duas linhas em silêncio e transborda para a linha de baixo»*. Com `Coat
    // Roughness` e `Emission Color` a não caberem em `72 px`, a segunda linha caía **por cima** da
    // linha seguinte, que é exactamente o que a foto do dono mostra.
    ph2d_editor_core::widget::paint_property_label(
        ctx.text_system,
        ctx.scene,
        tr(row.key),
        x,
        baseline,
        font,
        (w - valor_w).max(0.0),
        dim,
    );
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &valor,
        x + (w - valor_w).max(0.0),
        baseline,
        font,
        valor_w,
        dim,
    );
    y + ph2d_tokens::row_pitch_px()
}

/// ⭐⭐⭐ **UMA AMOSTRA TRAVADA** — a cor continua à vista, e o gesto não existe.
///
/// Ordem do Enio (2026-09-14): *«os slideres que só aparecem sob uma condição específica não devem
/// desaparecer, mas apenas serem inativados, mas sempre visíveis»*. Numa linha de cor isso quer dizer
/// **a amostra apagada**, e não o número que ela dobra — ver o doc do [`paint_row`].
///
/// ⛔ **Ela não regista NADA** (nem o `register_picker_swatch`, nem o `hit_index`): é a mesma lei do
/// [`paint_fact`], que é o irmão desta função para as linhas que são um número.
fn paint_dead_swatch(
    ctx: &mut PaintCtx,
    row: &ParamRow,
    id: ph2d_a11y::NodeId,
    rgb: [u8; 3],
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    use ph2d_editor_core::widget::{ColorSwatch, SwatchSize, SwatchState, paint_color_swatch};

    let theme = ctx.host.theme();
    let font = TypeToken::Sm.px();
    let dim = resolve(ColorToken::Text2, theme);
    let baseline = y + (ROW_H_PX - font) * 0.5;
    // ⭐⭐ **A MESMA CAIXA ÚNICA das outras linhas** (report do Enio com foto, 14/09): rótulo à
    // esquerda, **amostra na coluna da direita** — e o rótulo CORTA em vez de quebrar.
    //
    // ⛔ A amostra ocupava a goteira inteira (`w − 72 ≈ 230 px`) e o rótulo vivia nos `72` da
    // esquerda: `Emission Color` não cabia lá e virava **duas linhas**, por cima da linha seguinte.
    // *Uma amostra tão larga lê-se como um campo de texto, e a coluna dos valores deixava de estar
    // alinhada com a dos números.*
    let coluna = ph2d_editor_core::widget::property_label_col_w(x, w).min(w);
    let rotulo_w = (w - coluna).max(0.0);
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        tr(row.key),
        x,
        baseline,
        font,
        rotulo_w,
        dim,
    );
    let gutter = Rect::new(x + rotulo_w, y, coluna, ROW_H_PX);
    // ⚠️ **O id é o VERDADEIRO, e ele não é registado em lado nenhum** — o widget precisa de um, e
    // inventar outro faria duas identidades para a mesma amostra no dia em que ela acordasse.
    let swatch = ColorSwatch::new(id, tr(row.key), [rgb[0], rgb[1], rgb[2], 255])
        .size(SwatchSize::Md)
        .state(SwatchState::Disabled);
    paint_color_swatch(&swatch, gutter, ctx.scene, theme);
    y + ph2d_tokens::row_pitch_px()
}
