//! The two labelled rows the physics sections are built from.
//!
//! They live here rather than in either section because BOTH use them: §11
//! (Physics Body) and §12 (Physics Joint) are the same shape — a label, then a
//! segmented control or a number box. A second copy would be two answers to
//! "what does a physics row look like", and they would drift by a pixel.

use super::*;
use ph2d_editor_core::widget::{SegmentedAdaptive, SegmentedOption, paint_segmented_adaptive};

/// A labelled segmented control. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
pub(super) fn seg_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    group_id: NodeId,
    option_ids: &[NodeId],
    labels: &[&str],
    selected: u8,
) -> f32 {
    // ⭐⭐ **Irmã da [`num_row`]: o rótulo fica à ESQUERDA, pela mesma porta.** Convertê-la no mesmo
    // commit não é zelo — ela e a `num_row` partilham as mesmas secções, e deixar uma à esquerda e a
    // outra por cima **é** a queixa do dono («muito ruim!») uma linha mais abaixo.
    //
    // ⚠️ **O controlo REFLUI e a coluna do rótulo não:** com três opções compridas
    // (*Dynamic · Static · Kinematic*) o grupo segmentado quebra em mais de uma fileira dentro da
    // coluna dele — e é por isso que a altura devolvida é a do que ele de facto ocupou
    // (`seg_h`), nunca `ROW_H_PX`. *O que mantém a secção legível é a coluna da esquerda estar
    // sempre no mesmo `x`; o lado direito pode crescer.*
    let row = ph2d_editor_core::widget::property_row_columns(x, w, y, ROW_H_PX);
    let label_font = TypeToken::Sm.px();
    ph2d_editor_core::widget::paint_property_label(
        text_system,
        scene,
        label,
        row.label.x,
        row.label.y + (row.label.h - label_font) * 0.5,
        label_font,
        row.label.w,
        resolve(ColorToken::Text2, theme),
    );
    let seg = SegmentedAdaptive::new(
        group_id,
        label,
        option_ids
            .iter()
            .zip(labels)
            .map(|(&id, &l)| SegmentedOption::new(id, l))
            .collect(),
    )
    .selected((selected as usize).min(labels.len() - 1));
    let seg_h = paint_segmented_adaptive(
        &seg,
        row.control,
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, row.dot);
    y + seg_h.max(ROW_H_PX) + ph2d_tokens::control_gap_px()
}

/// ⭐ **O NOME de uma linha** — hoje é a porta de `ph2d-editor-core`, aqui só com o nome curto.
///
/// ⚠️ **A implementação MUDOU-SE em 2026-09-15** (`widget::paint_property_label_row`): sete painéis
/// pintam campos e nenhum passava por aqui, porque a porta vivia dentro do Inspector. *Uma porta
/// que vive dentro de um painel é uma porta que o painel seguinte copia.*
pub(super) use ph2d_editor_core::property_row::paint_label_row as property_label_row;

/// ⭐ **A LINHA DE VÁRIAS COMPONENTES** — hoje é a porta de `ph2d-editor-core`.
///
/// ⚠️ **A implementação MUDOU-SE em 2026-09-15** (`widget::paint_property_fields_row`), pelo mesmo
/// motivo da irmã acima. Os 23 sítios do Inspector não mudaram de forma: o nome curto fica.
pub(super) use ph2d_editor_core::property_row::paint_fields_row as fields_row;

/// A altura que uma [`num_row`] consome — **uma linha**.
///
/// ⚠️ Existe porque o **card** (`card_frame`) precisa saber quanto medir ANTES
/// de as rows serem pintadas, e uma segunda cópia dessa aritmética é como uma
/// moldura passa a não caber no que ela emoldura: a próxima seção pintaria por
/// cima das caixas. Uma régua, dois consumidores — o mesmo argumento do
/// `segmented_row_counts` do Painter.
/// ⚠️ **Encolheu em 2026-09-14** — de `rótulo + caixa + respiro` para **o passo de uma linha**.
/// O rótulo deixou de ter faixa própria (ele é a coluna da esquerda, pela
/// `widget::property_row_columns`), logo uma row mede uma LINHA, como em todo painel desta casa.
pub(super) fn num_row_h() -> f32 {
    ph2d_tokens::row_pitch_px()
}

/// Quanto um card **inteiro** mede.
///
/// ⚠️ **A régua é `pub(crate)` de propósito.** O pintor tem de avançar pela
/// ALTURA DA MOLDURA, nunca pela soma das rows — as duas diferem por uma
/// respiração, e somar as rows faz a moldura do card seguinte nascer POR CIMA da
/// borda do anterior. É um defeito que só a geometria vê, então a geometria é o
/// oráculo: o gate compara onde as rows de fato caíram contra este passo.
pub(crate) fn card_h(n_rows: usize) -> f32 {
    let pad = Spacing::Sm.px();
    let title_h = TypeToken::Sm.px() + Spacing::Sm.px();
    pad + title_h + n_rows as f32 * num_row_h() + pad
}

/// O passo de um card ao próximo — a moldura mais o respiro entre elas.
pub(crate) fn card_pitch(n_rows: usize) -> f32 {
    card_h(n_rows) + Spacing::Sm.px()
}

/// Uma **moldura titulada** para um grupo de [`num_row`] — a subseção.
///
/// Devolve `(inner_x, inner_w, first_row_y, y_after_card)`: o chamador pinta as
/// rows em `[inner_x, inner_w]` a partir de `first_row_y` e segue em
/// `y_after_card`.
///
/// ⚠️ **Molde do card do Painter** (`ph2d-panel-painter-layers::card_frame`), e
/// não uma invenção: é a caixa em que este app já diz *"estes números são um
/// assunto só"*. ⚠️ **E desde 2026-09-14 a régua da linha é a MESMA dos dois lados**: o rótulo fica
/// à ESQUERDA aqui também, pela `widget::property_row_columns`. *Esta nota dizia o contrário — ela
/// é de quando o Inspector era a excepção, e o report do dono fechou essa excepção.*
#[allow(clippy::too_many_arguments)]
pub(super) fn card_frame(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    title: &str,
    n_rows: usize,
) -> (f32, f32, f32, f32) {
    let pad = Spacing::Sm.px();
    let font = TypeToken::Sm.px();
    let title_h = font + Spacing::Sm.px();
    let card_h = card_h(n_rows);
    let card = Rect::new(x, y, w, card_h);
    // ⭐ Raio e moldura pela porta do TEMA: o cartão é plano num tema moderno.
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(
        scene,
        card,
        radius,
        resolve(
            ph2d_editor_core::widget::section_cards::CardDepth::Subsection.token(),
            theme,
        ),
    );
    ph2d_editor_core::paint::stroke_frame(
        scene,
        card,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        ph2d_tokens::StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );
    paint_text(
        text_system,
        scene,
        title,
        x + pad,
        y + pad,
        font,
        w - 2.0 * pad,
        resolve(ColorToken::Text2, theme),
    );
    (
        x + pad,
        w - 2.0 * pad,
        y + pad + title_h,
        y + card_pitch(n_rows),
    )
}

/// A labelled number box, seeded from the store (which `sync` mirrors from
/// the snapshot). Returns the next `y`.
///
/// ⭐ **Sem unidade** — a forma que a esmagadora maioria das secções usa. Quem tem uma unidade
/// física chama a [`num_row_unit`], que é esta com o sufixo ao fim do campo.
#[allow(clippy::too_many_arguments)]
pub(super) fn num_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    id: NodeId,
) -> f32 {
    num_row_unit(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        label,
        id,
        None,
        None,
    )
}

/// ⭐⭐⭐ **A mesma linha, com a UNIDADE dentro do campo.**
///
/// ⛔⛔ **A unidade vivia no RÓTULO, e era o que o cortava.** Medido em 2026-09-14 com o sistema
/// de texto real, à largura de omissão do Inspector (coluna do rótulo `91,2 px`, fonte `12`):
/// **20 de 39** rótulos eram elididos, e sem a unidade **fica 1** (*«Non-Spatialized Radius»*, que
/// é um nome genuinamente longo — outra pergunta). `"Float Height (m)"` mede `92,1 px` numa coluna
/// de `91,2`: ele perdia o `(m)` **e** o `t` do *Height*.
///
/// ⚠️ **A unidade não é decoração: ela é o que o número SIGNIFICA.** Pô-la no rótulo fá-la
/// competir por espaço com o NOME; pô-la no campo põe-na ao lado do valor, que é onde ela é lida.
/// É o que o Blender, o Godot e o Illustrator fazem.
///
/// ⏳ **Dívida NOMEADA:** quando a unidade é um COMPRIMENTO, o app tem uma definição de projecto
/// (`DisplayUnit`, metros ⇄ pixels) e esta porta ainda mostra a unidade **fixa** que a tabela
/// declara. ⛔ Ligar as duas exige converter também o VALOR — mostrar `px` sobre um número em
/// metros seria trocar um rótulo comprido por um rótulo MENTIROSO. Wave própria.
#[allow(clippy::too_many_arguments)]
pub(super) fn num_row_unit(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    id: NodeId,
    unit: Option<ph2d_editor_core::widget::Unit>,
    // ⭐⭐ **O rótulo mais largo que a SECÇÃO vai pintar** — ver
    //    [`ph2d_editor_core::widget::property_label_col_w_for`]. `None` = a metade da linha.
    //
    // ⚠️ **É da SECÇÃO e não desta linha, de propósito:** uma coluna por linha seria uma coluna
    //    diferente por linha, e o que o dono pediu foi *«as labels alinhadas todas à direita»*.
    desired_label_w: Option<f32>,
) -> f32 {
    // ⭐⭐⭐ **O rótulo fica à ESQUERDA, pela porta** — report do dono, 2026-09-14, com foto da
    // secção LEG do Platform Player: *«Label acima do campo numérico! Muito ruim!»*. Até aqui esta
    // função empilhava (uma faixa só para o rótulo, o campo por baixo em largura cheia) e era a
    // única das cinco do Inspector com o campo a ocupar a linha inteira.
    //
    // ⚠️ **A altura de uma row cai de `label_h + passo` para o PASSO** (~16 px por linha), e é por
    // isso que a [`num_row_h`] encolheu no mesmo commit: a `card_h` deriva dela, e as duas têm de
    // continuar a ser a mesma resposta — senão a moldura do card nasce por cima das caixas.
    let row = ph2d_editor_core::widget::property_row_columns_for(
        x,
        w,
        y,
        ROW_H_PX,
        desired_label_w,
        None,
    );
    let label_font = TypeToken::Sm.px();
    // ⚠️ **ELIDIDO, nunca transbordado:** a coluna do rótulo é uma FRACÇÃO da linha (a coluna
    // docada é arrastável), logo um rótulo comprido numa coluna estreita passaria por cima do campo.
    ph2d_editor_core::widget::paint_property_label(
        text_system,
        scene,
        label,
        row.label.x,
        row.label.y + (row.label.h - label_font) * 0.5,
        label_font,
        row.label.w,
        resolve(ColorToken::Text2, theme),
    );
    hit_index.register(id, row.control);
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    let input = NumberInput::new(id, "", value).visual((state, store.hover_live(id)));
    // ⭐⭐ **A unidade é um SUFIXO colado ao número, dentro da caixa** — ordem do dono,
    //    2026-09-14: *«não ficou legal. Melhor junto ao número dentro da caixa»*. A 1.ª entrega
    //    punha-a num chip com fundo próprio encostado à direita, e com o campo já afundado isso
    //    lia-se como **duas** caixas.
    // ⚠️ O campo é a linha inteira nos dois casos, logo o rect registado e o pintado são o MESMO.
    paint_number_input_with_buffer(
        &input.suffix(unit.map(ph2d_editor_core::widget::Unit::suffix)),
        Some(buffer),
        caret,
        anchor,
        row.control,
        scene,
        text_system,
        theme,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, row.dot);
    y + ph2d_tokens::row_pitch_px()
}

/// ⭐⭐⭐ **O AVISO de uma secção — uma FRASE, e por isso ela QUEBRA.**
///
/// ⛔⛔⛔ **Esta porta nasceu de duas medições no mesmo dia (2026-09-19).**
///
/// **(a) Ela estava escrita SEIS vezes, byte a byte igual** — `audio` · `camera` · `factory` ·
/// `particles` · `projectile` · `topdown` —, cada cópia com um doc a dizer *«gémea da do
/// projéctil — ver a irmã»*. *Uma lei escrita em seis sítios não é uma lei.*
///
/// **(b) E as seis pintavam com a lei de um RÓTULO.** O `paint_text` **elide para uma linha**
/// desde 06/09 (report do dono: *«a palavra passa para baixo e some»*), o que está certo para o
/// nome de uma fileira e **errado para uma frase**. Medido pela varredura de elisões com o
/// Inspector armado, na coluna de `268 px`:
///
/// | frase | como saía |
/// |---|---|
/// | `These are the scene's ears — sound is heard from here.` | `…sound is hear…` |
/// | `Nothing is born from this object — put this on the recipe a Factory makes.` | `…put this o…` |
///
/// ⇒ ela passa pelo [`paint_text_block`], que existe PARA quebrar e **devolve a altura**; é a
/// mesma cura que as três frases de estado vazio pagaram em 18/09.
///
/// ⚠️ **A altura VOLTA ao chamador e é ela que empurra o resto** — sem isso a segunda linha da
/// frase é escrita por cima da fileira seguinte, que foi o defeito que a cura de 18/09 apanhou por
/// mutação sobrevivente.
///
/// ⛔ **E o censo de elisões NÃO é régua disto:** um texto que quebra não passa pela lei da
/// reticência, logo não deixa registo — *zero lê-se como aprovação*. A régua é a TINTA, no gate
/// irmão da crate `ph2d-editor-core`.
#[allow(clippy::too_many_arguments)]
/// ⚠️ **`#[track_caller]` é o que torna o censo utilizável**: sem ele todo aviso do painel é
/// registado na linha DESTE ficheiro, e a régua que diz *«esta frase saiu 21 vezes»* não sabe
/// dizer de ONDE. (A lição que o censo das elisões pagou em 19/09, com nove buscas no Inspector.)
#[track_caller]
pub(crate) fn aviso(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    texto: &str,
    token: ColorToken,
) -> f32 {
    crate::censo_dos_avisos::regista(texto);
    let font = TypeToken::Sm.px();
    let alta = ph2d_editor_core::paint::paint_text_block(
        text_system,
        scene,
        texto,
        x,
        y,
        font,
        w,
        resolve(token, theme),
    );
    y + alta.max(font) + ph2d_tokens::control_gap_px()
}

#[cfg(test)]
mod aviso_tests {
    use super::*;
    use ph2d_editor_core::paint::paint_text;

    /// A frase que a varredura de elisões apanhou cortada em `268 px` (secção AUDIO, 19/09).
    const FRASE: &str = "These are the scene's ears — sound is heard from here.";
    const COLUNA: f32 = 268.0;

    fn mede(prosa: bool) -> (f32, bool) {
        let mut ts = TextSystem::without_system_fonts();
        let mut cena = VectorScene::new();
        let (alta, medidos) = ph2d_editor_core::text_elide::elisao::medindo(|| {
            if prosa {
                aviso(
                    &mut cena,
                    &mut ts,
                    Theme::Dark,
                    0.0,
                    COLUNA,
                    0.0,
                    FRASE,
                    ColorToken::Text3,
                )
            } else {
                paint_text(
                    &mut ts,
                    &mut cena,
                    FRASE,
                    0.0,
                    0.0,
                    TypeToken::Sm.px(),
                    COLUNA,
                    resolve(ColorToken::Text3, Theme::Dark),
                );
                0.0
            }
        });
        let cortou = medidos.iter().any(|m| m.texto == FRASE && !m.coube());
        (alta, cortou)
    }

    /// ⭐⭐⭐ **UM AVISO DE SECÇÃO É UMA FRASE: ele QUEBRA, e o pintor de RÓTULO corta.**
    ///
    /// ⛔⛔ **As duas metades são obrigatórias, e a segunda é o controlo.** Um texto que quebra
    /// **não deixa registo na lei da reticência** — logo um aviso que simplesmente deixasse de ser
    /// pintado leria-se exactamente igual a um que coube. *Zero lê-se como aprovação* (a lei que a
    /// cura das frases de estado vazio já pagou em 18/09). ⇒ o lado que PROVA que a frase não cabe
    /// nesta coluna é o pintor de rótulo, corrido sobre o mesmo texto e a mesma largura.
    ///
    /// ⚠️ E a ALTURA é a terceira metade: ela é o que empurra a fileira seguinte. Sem ela a 2.ª
    /// linha da frase é escrita por cima do que vem abaixo — o defeito que a irmã de 18/09 só
    /// apanhou por mutação sobrevivente.
    ///
    /// (Mutação: trocar o `paint_text_block` por `paint_text` ⇒ `cortou = true` e a altura cai para
    /// uma linha, RED nas duas metades.)
    #[test]
    fn um_aviso_quebra_e_o_pintor_de_rotulo_corta() {
        let (alta, cortou_prosa) = mede(true);
        let (_, cortou_rotulo) = mede(false);
        assert!(
            cortou_rotulo,
            "o CONTROLO não reproduz o fenómeno: {FRASE:?} caberia em {COLUNA} px pintada como \
             rótulo — sem isso esta régua não afirma nada"
        );
        assert!(
            !cortou_prosa,
            "o aviso saiu CORTADO: ele é uma frase e tem de quebrar"
        );
        let uma_linha = TypeToken::Sm.px() + ph2d_tokens::control_gap_px();
        assert!(
            alta > uma_linha,
            "o aviso devolveu {alta} px, a altura de UMA linha ({uma_linha}) — quem empilha por \
             baixo vai escrever por cima da segunda linha"
        );
    }
}
