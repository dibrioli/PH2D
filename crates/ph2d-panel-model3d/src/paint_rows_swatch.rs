//! ⭐⭐⭐ **UMA LINHA QUE É UMA COR** — a amostra viva, a amostra travada, e o id de cada uma.
//!
//! # Por que um módulo irmão
//!
//! Cortado do [`crate::paint_rows`] em 2026-09-19 pelo tecto de LOC dos painéis, e o corte é por
//! **assunto**: ali mora o que se desenha quando a linha é um NÚMERO (um slider, uma escolha, um
//! facto), aqui o que se desenha quando ela é uma COR. ⛔ *Split, nunca uma entrada na allowlist.*
//!
//! ⚠️ **A convenção do ficheiro-mãe continua a valer:** toda função de pintura devolve o Y SEGUINTE.

use ph2d_editor_core::paint::{paint_text_elided, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, TypeToken};

use crate::paint_rows::colunas_da_fileira;
use crate::state::ParamRow;

/// ⭐⭐⭐ **O ID DE UMA LINHA-AMOSTRA — a PORTA, com dois leitores** (report do Enio, 2026-09-19:
/// *«se modifico qualquer cor em style, todas mudam ao mesmo tempo»*).
///
/// # ⛔⛔ O defeito que ela cura, e porque nenhum gate o via
///
/// O id era derivado **dentro** do [`paint_swatch`] por um `match` cujo braço final dizia, por
/// escrito, *«uma amostra sobre um param sem índice não existe hoje; `0` é a resposta estável»*.
/// Era **verdade no dia em que foi escrita** e ficou **falsa** quando a camada de estilo
/// (`docs/Render3d/11`) trouxe cinco linhas de cor cujo `entity` é `0` por desenho — *o estilo não
/// é de entidade nenhuma*. As cinco caíam no braço final e recebiam o **mesmo** id:
///
/// ```text
/// hash("model3d.color.swatch.0.0")   ← as CINCO cores do estilo
/// ```
///
/// ⇒ com o selector aberto numa delas, **as cinco** liam `picker_target() == Some(id)`, **as cinco**
/// comparavam a cor escolhida com a sua, e **as cinco** pediam a edição. Uma roda, cinco escritas.
///
/// ⚠️⚠️ **E a segunda metade estava na outra ponta:** a lista que fecha um selector órfão
/// (`crate::paint::close_a_stranded_picker`) derivava o id por um **segundo `match`**, que só
/// conhecia `Param::Material`. *Duas respostas à mesma pergunta — «qual é o id desta amostra?» —
/// divergem no dia em que uma família nova entra*, e aqui já divergiam para a **luz**, desde a wave
/// dela. ⇒ uma porta, e os dois leitores passam por ela.
///
/// # ⚠️ Porque o estilo tem espaço de nomes PRÓPRIO
///
/// [`crate::ids::model3d_color_swatch`] cunha o par `(entidade, campo)`, e o sujeito do estilo **não
/// é uma entidade** — o `0` que a fileira carrega é um sentinela que o dreno nem lê. Pendurar o
/// estilo naquele nome faria a não-colisão depender do acidente de `Entity::to_bits()` nunca valer
/// `0`; com [`crate::ids::model3d_style_swatch`] ela é **inexprimível por construção**.
///
/// # ⛔ `None` é uma família NOVA, e ela cai para o controlo normal
///
/// Uma amostra sem id não é pintada como amostra — ela cai no slider, que é o que o doc do
/// [`crate::paint_rows::paint_row`] já prometia e o código não fazia. *Visível e diferente é um
/// defeito que se lê; um id partilhado em silêncio é o que este report custou.*
#[must_use]
pub fn swatch_id(row: &ParamRow) -> Option<ph2d_a11y::NodeId> {
    match row.param {
        // ⚠️ As duas famílias de ENTIDADE nunca coexistem no mesmo nó (o `params_of` responde uma
        // OU a outra), logo o par `(entidade, índice)` continua único entre elas.
        ph2d_field::Param::Material(k) | ph2d_field::Param::Light(k) => {
            Some(crate::ids::model3d_color_swatch(row.entity, k))
        }
        // ⭐ **O estilo é da CENA** — espaço de nomes próprio, e o `slot` é a posição na arrumação,
        // que é o que torna as cinco cores cinco controlos.
        ph2d_field::Param::Style(slot) => Some(crate::ids::model3d_style_swatch(slot)),
        // ⛔ **`None` e nunca um id inventado.** Era aqui que estava o defeito: um `0` «estável»
        // dava a TODAS as famílias novas o mesmo controlo. Uma família sem id cai para o slider —
        // visível, diferente, e legível como uma falta.
        _ => None,
    }
}

/// O rótulo de uma linha-amostra, com a **mesma** repartição da fileira viva. Devolve a goteira.
///
/// ⛔⛔ **A goteira era a coluna do RÓTULO da porta do formulário** (`property_label_col_w`), e essa
/// grandeza CRESCE com a linha: medido em 2026-09-19, a amostra media `84 px` num painel de `220` e
/// **`334 px`** num de `720` — *uma caixa de cor que ocupa metade do painel lê-se como um campo de
/// texto*, e o alinhamento dela com a fileira viva (cuja coluna de valor é fixa) derivava com a
/// largura. Ver [`colunas_da_fileira`].
fn rotulo_e_goteira(ctx: &mut PaintCtx, row: &ParamRow, x: f32, w: f32, y: f32) -> Rect {
    let theme = ctx.host.theme();
    let font = TypeToken::Sm.px();
    let dim = resolve(ColorToken::Text2, theme);
    let (nome, goteira) = colunas_da_fileira(x, w, y);
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
    goteira
}

/// ⭐⭐⭐ **A LINHA-AMOSTRA** — o rótulo à esquerda, e na goteira do valor a cor que a peça tem
/// (Enio, 2026-09-14: *«em vez de 3 sliders de RGB, deveríamos ter uma caixa seletora de cor»*).
///
/// # ⭐ O selector não se constrói aqui: ele já existe, e é UM
///
/// A casa tem **um** selector de cor (o OKLCH com roda, canais RGB/HSV/OKLCH, hexadecimal, paletas
/// e conta-gotas), e um painel entra nele por **duas** linhas: registar o id como amostra
/// (`WidgetStore::register_picker_swatch`) e manter a cor dela em dia. O `Down` genérico do
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
pub(crate) fn paint_swatch(
    ctx: &mut PaintCtx,
    row: &ParamRow,
    id: ph2d_a11y::NodeId,
    rgb: [u8; 3],
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    use ph2d_editor_core::widget::{ColorSwatch, SwatchSize, paint_color_swatch};

    let theme = ctx.host.theme();
    let gutter = rotulo_e_goteira(ctx, row, x, w, y);

    // ⛔⛔ **O id chega de FORA, pela porta [`swatch_id`]** — ele era derivado aqui dentro, e a
    // outra ponta (a lista que fecha um selector órfão) derivava-o outra vez. *Duas respostas à
    // mesma pergunta, e o report de 19/09 foi o dia em que elas divergiram.*
    //
    // ⭐⭐⭐ **UMA AMOSTRA TRAVADA NÃO ABRE O SELECTOR, e nem sequer se REGISTA** — ordem do Enio
    // (14/09): ela fica **visível e inactiva**.
    //
    // ⛔⛔ **As três metades têm de sair juntas, e cada uma sozinha é um defeito diferente:** sem o
    // `register_picker_swatch` o selector não a reconhece; sem o `hit_index` ela não é clicável;
    // sem o `SwatchState::Disabled` ela **parece** clicável. *Uma amostra que parece viva e não
    // responde é o controlo morto na forma que o artista mais rapidamente lê como avaria.*
    if row.inert.is_some() {
        return paint_dead_swatch(ctx, row, id, rgb, gutter, y);
    }
    let aberto = {
        let store = ctx.host.store_mut();
        store.register_picker_swatch(id);
        // ⭐⭐⭐ **O BALÃO desta amostra** — ver [`crate::dica`]. Um id só: a amostra é UM widget.
        crate::dica::pendura(store, row, &[id]);
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

/// ⭐⭐⭐ **UMA AMOSTRA TRAVADA** — a cor continua à vista, e o gesto não existe.
///
/// Ordem do Enio (2026-09-14): *«os slideres que só aparecem sob uma condição específica não devem
/// desaparecer, mas apenas serem inativados, mas sempre visíveis»*. Numa linha de cor isso quer dizer
/// **a amostra apagada**, e não o número que ela dobra — ver o doc do [`crate::paint_rows::paint_row`].
///
/// ⛔ **Ela não regista NADA** (nem o `register_picker_swatch`, nem o `hit_index`): é a mesma lei do
/// [`crate::paint_rows::paint_fact`], que é o irmão desta função para as linhas que são um número.
///
/// ⚠️ **Ela recebe a goteira JÁ MEDIDA** e não a re-deriva: as duas amostras têm de aterrar no
/// mesmo sítio, senão atravessar a trava faz a cor saltar de coluna.
fn paint_dead_swatch(
    ctx: &mut PaintCtx,
    row: &ParamRow,
    id: ph2d_a11y::NodeId,
    rgb: [u8; 3],
    gutter: Rect,
    y: f32,
) -> f32 {
    use ph2d_editor_core::widget::{ColorSwatch, SwatchSize, SwatchState, paint_color_swatch};

    let theme = ctx.host.theme();
    // ⚠️ **O id é o VERDADEIRO, e ele não é registado em lado nenhum** — o widget precisa de um, e
    // inventar outro faria duas identidades para a mesma amostra no dia em que ela acordasse.
    let swatch = ColorSwatch::new(id, tr(row.key), [rgb[0], rgb[1], rgb[2], 255])
        .size(SwatchSize::Md)
        .state(SwatchState::Disabled);
    paint_color_swatch(&swatch, gutter, ctx.scene, theme);
    y + ph2d_tokens::row_pitch_px()
}
