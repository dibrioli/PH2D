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
use ph2d_editor_core::widget::{
    NUMBER_INPUT_MIN_W_PX, number_text_origin, paint_slider_with_chip_layout_adaptive,
    property_label_col_w, slider_with_chip_chip_rect, slider_with_chip_label_rect, stepper_width,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, TypeToken};

use crate::paint::{STEPS_ACROSS_THE_RANGE, TETO_DIGITAVEL, bound_is_wall, decimals_for_step};
use crate::state::ParamRow;

/// ⭐⭐⭐ **A REPARTIÇÃO DE UMA LINHA DESTE PAINEL — a MESMA nas TRÊS formas, e a da fileira VIVA.**
///
/// ⛔⛔⛔ **Auditoria de 2026-09-19, sobre o report *«widgets embolados»*: o painel tinha TRÊS
/// alinhamentos de rótulo ao mesmo tempo.** A fileira viva é a **caixa única**
/// (`paint_slider_with_chip_layout_adaptive`), que põe o nome dentro à esquerda e reserva a coluna
/// do valor pela direita; a travada e a amostra escolhiam a coluna do **formulário**
/// (`property_label_col_w`, metade da linha), que é outra grandeza — e uma que **cresce com a
/// largura**:
///
/// | painel | valor da VIVA | valor da TRAVADA (antes) | desvio |
/// |---:|---:|---:|---:|
/// | `220` | `108,00` | `118,00` | `+10,00` |
/// | `296,89` (o dono) | `184,89` | `156,45` | `−28,44` |
/// | `720` | `608,00` | `368,00` | `−240,00` |
///
/// ⚠️ **E o `16 px` que a auditoria publicou descreve o ecrã CLÁSSICO**, não o de omissão: ali a
/// caixa única não corre, o pintor lê o `label_w` que lhe passam, e a diferença entre `Lc` e
/// `w − Lc` é de facto `2 × Spacing::Md`. *Uma medição feita sobre a fórmula e não sobre o pintor
/// mede a aparência que o artista não tem.*
///
/// ⭐ **Os dois números que esta porta passa são os MESMOS que o [`paint_row`] passa ao pintor
/// vivo** — é isso, e não uma constante partilhada, que faz as três formas aterrarem no mesmo
/// sítio nas **duas** aparências.
pub(crate) fn colunas_da_fileira(x: f32, w: f32, y: f32) -> (Rect, Rect) {
    let linha = Rect::new(x, y, w, ROW_H_PX);
    let label_w = property_label_col_w(x, w);
    (
        slider_with_chip_label_rect(linha, label_w, NUMBER_INPUT_MIN_W_PX),
        slider_with_chip_chip_rect(linha, label_w, NUMBER_INPUT_MIN_W_PX),
    )
}

/// ⭐⭐⭐ **O PASSO DO ARRASTO DE UMA LINHA** — a porta, com dois leitores.
///
/// ⛔⛔ **Ela existe porque a [`paint_fact`] formatava com `decimals_for_step(1.0)`** — uma casa
/// decimal, fosse qual fosse a faixa. Medido: `0,375` vivo lia-se **`0.4`** travado, e o `Coat IOR`
/// de `1,6` da foto do dono é a prova. *Atravessar a trava mudava a PRECISÃO do número*, que é a
/// mesma família do salto de espécie que a linha-amostra já pagou.
///
/// ⚠️ **Uma linha inteira tem passo `1`**, e não um centésimo do curso: meia cópia não existe.
pub(crate) fn passo_da_linha(row: &ParamRow) -> f32 {
    if row.integral {
        1.0
    } else {
        let scale = (row.bound.value() - row.lo).max(f32::MIN_POSITIVE);
        scale / STEPS_ACROSS_THE_RANGE
    }
}

/// ⭐ **Quantas casas decimais uma linha mostra** — DERIVADO do passo dela. Ver [`passo_da_linha`].
pub(crate) fn casas_da_linha(row: &ParamRow) -> usize {
    if row.integral {
        0
    } else {
        decimals_for_step(passo_da_linha(row))
    }
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
    //
    // ⚠️⚠️ **E o ID sai da PORTA** (report de 2026-09-19) — ver [`swatch_id`]. Uma família de cor
    // que ela não conheça devolve `None` e a linha cai para o controlo normal, que é **exactamente
    // o que o parágrafo acima já prometia** e o código não fazia: ele mandava-a para a amostra com
    // um id partilhado, e cinco cores passaram a mudar juntas.
    if let Some(rgb) = row.swatch
        && let Some(id) = crate::paint_rows_swatch::swatch_id(row)
    {
        return crate::paint_rows_swatch::paint_swatch(ctx, row, id, rgb, x, w, y);
    }
    // ⭐ **Uma linha que não pode agir não é pintada como se pudesse** — ver [`ParamRow::inert`].
    // Ela sai daqui como facto e **não regista nada** no índice de acerto, então não há slider a
    // agarrar nem campo a receber texto: é a mesma lei do [`paint_note`], neste mesmo arquivo.
    //
    // ⚠️ **A RAZÃO não é pintada aqui**, e é de propósito: ela pertence à **corrida** de linhas que
    // a partilham, e só o orquestrador vê a linha anterior — ver [`razao_a_pintar`], neste mesmo
    // arquivo. *Quatro fileiras seguidas com a mesma frase por baixo de cada uma é ruído que o
    // artista aprende a ignorar, exactamente quando ela passar a ser a que importa.*
    if row.inert.is_some() {
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
    //
    // ⚠️ **Pela porta [`passo_da_linha`]** — a fileira TRAVADA formata o mesmo número e tem de ler o
    // mesmo passo; enquanto a conta viveu aqui dentro, ela lia `1.0` e mostrava uma casa só.
    let step = passo_da_linha(row);
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
        // ⭐⭐⭐ **O BALÃO** — ver [`crate::dica`]. Nos DOIS ids, porque o rato pode estar quente
        // sobre o trilho ou sobre o campo.
        crate::dica::pendura(store, row, &[slider, chip]);
    }

    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    // A verdade é o documento; a posição guardada é só o que o arrasto deixou para trás. Semear a
    // trilha a partir do valor todo quadro é o que mantém o controle honesto quando o raio muda de
    // outro lado — um desfazer, um arquivo aberto, uma segunda linha.
    let track = ((row.value - lo) / scale).clamp(0.0, 1.0);
    let display = f64::from(row.value);
    let decimals = casas_da_linha(row);
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

/// ⭐⭐⭐ **A FILEIRA DE ESCOLHA de uma linha** — o rótulo à esquerda, os botões na goteira do valor.
///
/// ⛔⛔ **O doc desta função dizia *«a goteira é a MESMA do slider (a porta `property_label_col_w`)»*
/// e isso era FALSO no ecrã de omissão**: desde a caixa única (2026-09-02) o pintor da fileira viva
/// **ignora** o `label_w` que lhe passam e reserva a coluna do valor pela direita. *Uma afirmação
/// sobre um argumento que o consumidor deita fora é um palpite com cara de medição.*
///
/// ⭐ **O que fica igual é o RÓTULO**, que é a queixa do report (*«widgets embolados»*): ele nasce
/// no mesmo `x` das outras três formas, pela porta [`colunas_da_fileira`].
///
/// ⛔ **A GOTEIRA fica DECLARADAMENTE mais larga, e o recurso está medido:** o valor de uma linha
/// viva é um campo numérico com piso de `NUMBER_INPUT_MIN_W_PX`; aqui são `n` botões que têm de
/// caber lado a lado. Medido a `296,89` (a largura do dono): a coluna do valor mede `80 px` e os
/// três chips do `Axis` pediriam `26` cada — o `paint_segmented_group_adaptive` **quebra a fileira
/// em duas linhas**, e a linha passa a ter o dobro da altura das vizinhas. *Trocar um rótulo
/// alinhado por um painel que salta de altura é a troca errada.*
///
/// ⭐ Ela reusa o `paint_segmented_group_adaptive` — o **mesmo** widget das fileiras de chips do topo
/// do painel. *Um sexto caminho de pintura neste arquivo seria um sexto sítio onde o hit-index
/// pode ficar por registar.*
fn paint_choice(ctx: &mut PaintCtx, row: &ParamRow, slot: u32, x: f32, w: f32, y: f32) -> f32 {
    let theme = ctx.host.theme();
    let font = TypeToken::Sm.px();
    let dim = resolve(ColorToken::Text2, theme);
    let baseline = y + (ROW_H_PX - font) * 0.5;
    let (nome, _) = colunas_da_fileira(x, w, y);
    let goteira_x = x + property_label_col_w(x, w);
    // ⛔ **Corta, nunca quebra** — ver a nota do [`paint_fact`]. Os rótulos de escolha deste painel
    // são curtos (`Axis`), mas o defeito não é do comprimento de hoje: é do pintor.
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        tr(row.key),
        nome.x,
        baseline,
        font,
        (goteira_x - nome.x - ph2d_tokens::Spacing::Md.px()).max(0.0),
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
    // ⭐ **O BALÃO, em cada botão da fileira** — ver [`crate::dica`]: o `hot_id` é o do botão sob o
    // rato, e uma dica pendurada num só apareceria em parte da fileira.
    //
    // ⚠️ **Antes do empréstimo do par `store`+`hit_index`**, que é partilhado e imutável no `store`.
    let botoes: Vec<ph2d_a11y::NodeId> = labels.iter().map(|(_, _, id)| *id).collect();
    crate::dica::pendura(ctx.host.store_mut(), row, &botoes);
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_segmented_group_adaptive(
        Rect::new(goteira_x, y, (x + w - goteira_x).max(0.0), ROW_H_PX),
        &labels,
        ctx.scene,
        ctx.text_system,
        theme,
        store,
        hit_index,
    );
    y + used.max(ROW_H_PX) + ph2d_tokens::control_gap_px()
}

/// ⭐⭐⭐ **A RAZÃO DE UMA LINHA APAGADA PINTA-SE UMA VEZ POR CORRIDA** — `Some` ⇒ pinta-a agora.
///
/// Decisão do dono, 2026-09-18: as fileiras que o modo em mãos não lê ficam *«à vista, apagadas»*,
/// com a razão ao lado. ⚠️ Com `Thin Walled` ligado são **quatro** fileiras seguidas com a mesma
/// razão (o raio da subsuperfície e os três canais da escala dele) — e escrevê-la quatro vezes é o
/// defeito que a família do esculpir já nomeou por escrito: *um pincel que se queixa sempre é ruído
/// que o artista aprende a ignorar, exactamente quando a queixa passar a ser verdade.*
///
/// ⭐ **A corrida é DERIVADA e não uma lista de famílias:** ela quebra sozinha quando a razão muda,
/// quando aparece uma linha viva (`inerte == None`) e quando o orquestrador pinta um cabeçalho de
/// secção — nesse caso ele repõe `ja_dita`, porque o cabeçalho separa as duas **à vista** e a razão
/// de cima deixa de estar ao lado da de baixo.
///
/// ⛔ **A comparação é por PONTEIRO-e-conteúdo (`Option<&str>`) e não por família**: duas razões
/// diferentes que calhem seguidas continuam a ser ditas as duas. *Agrupar por família seria uma
/// segunda lista ao lado da do `params_of`, e a que envelhece na primeira razão nova.*
pub(crate) fn razao_a_pintar(
    ja_dita: Option<&'static str>,
    inerte: Option<&'static str>,
) -> Option<&'static str> {
    match inerte {
        Some(k) if ja_dita != Some(k) => Some(k),
        _ => None,
    }
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
///
/// # ⛔⛔⛔ Atravessar a trava mexia o número de SÍTIO e de PRECISÃO
///
/// Auditoria de 2026-09-19. Ela pintava o rótulo encostado à **direita** de uma coluna de
/// `w − property_label_col_w(x, w)` e o valor a seguir — três consequências, todas medidas:
///
/// 1. **vão `0,00 px` por construção**, em toda a largura do dock: o `paint_property_label` encosta
///    o nome à direita da coluna dele, e o valor começava exactamente onde ela acaba — *o nome e o
///    número tocavam-se*, que é o «embolado» da foto;
/// 2. **a coluna não era a da fileira viva** (ver [`colunas_da_fileira`] para a escada);
/// 3. **`decimals_for_step(1.0)`** ⇒ uma casa decimal fosse qual fosse a faixa: `0,375` vivo lia-se
///    `0.4` travado (ver [`passo_da_linha`]).
///
/// ⭐ Hoje as três formas leem a **mesma** repartição, e o número é pousado pela **mesma porta** que
/// o campo numérico usa ([`number_text_origin`]) — *um número travado aterra no pixel em que o
/// número vivo estava*.
fn paint_fact(ctx: &mut PaintCtx, row: &ParamRow, x: f32, w: f32, y: f32) -> f32 {
    let font = TypeToken::Sm.px();
    let theme = ctx.host.theme();
    let dim = resolve(ColorToken::Text2, theme);
    let (nome, coluna) = colunas_da_fileira(x, w, y);
    // ⛔⛔ **`paint_text_elided` e NUNCA `paint_text_block`** — o doc do primeiro nomeia este defeito
    // à letra: *«`paint_text` trata `max_width` como orçamento de QUEBRA, então um rótulo um pixel
    // largo demais vira duas linhas em silêncio e transborda para a linha de baixo»*.
    //
    // ⚠️ **À ESQUERDA, e não mais encostado à direita:** a fileira viva põe o nome dentro à esquerda
    // da caixa, e era o `paint_property_label` que fazia desta a única linha do painel com o nome
    // do outro lado.
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        tr(row.key),
        nome.x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        nome.w,
        dim,
    );
    // ⚠️ **A fonte é a do CAMPO (`Xs`) e não a do rótulo (`Sm`)** — é o número vivo que esta linha
    // dobra, e medir num corpo e pintar noutro punha-o a saltar de tamanho ao travar.
    let fonte_n = TypeToken::Xs.px();
    let texto = format!("{:.d$}", f64::from(row.value), d = casas_da_linha(row));
    let medido = ctx
        .text_system
        .layout(&texto, fonte_n, f32::INFINITY)
        .width();
    let area = (coluna.w - stepper_width(coluna)).max(0.0);
    // ⚠️ **O `max` é a cerca do caso degenerado**: um número mais largo que a área é centrado pela
    // porta e começaria à ESQUERDA da coluna, por cima do nome. Aqui ele encosta e elide.
    let tx = number_text_origin(coluna, medido).max(coluna.x);
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &texto,
        tx,
        y + (ROW_H_PX - fonte_n) * 0.5,
        fonte_n,
        area,
        dim,
    );
    y + ph2d_tokens::row_pitch_px()
}

/// ⚠️ **Irmão de arquivo do `paint_tests`:** aqueles medem o que se LÊ numa linha (a formatação);
/// este mede a REPARTIÇÃO — onde o nome e o valor de cada uma das quatro formas aterram.
#[cfg(test)]
#[path = "paint_rows_colunas_tests.rs"]
mod colunas_tests;
