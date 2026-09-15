//! **UMA LINHA da secção Transform** — rótulo à esquerda, e um ou dois campos numéricos.
//!
//! ⚠️ Este ficheiro nasceu de um **tecto de LOC por função**: a `paint_transform_section` passou a
//! folga dela quando a coluna de animação entrou (2026-09-03). ⛔ A casa não aumenta a folga —
//! *«allowances shrink, they never grow»* —, corta. E o corte é o que a própria mensagem do gate
//! sugere: *o desenho de uma linha* sai da *orquestração das linhas*.
//!
//! ⭐ Os catorze valores que a closure capturava viraram o [`RowStyle`]: eles são **a geometria da
//! secção**, calculada uma vez e igual para as quatro linhas — e enquanto viviam como capturas,
//! nada dizia isso.

use super::*;

/// A geometria partilhada por TODA linha desta secção — calculada uma vez, lida por cada linha.
///
/// ⭐⭐⭐ **Encolheu de 16 para 10 campos em 2026-09-15** (ordem do dono: *«a mesma formatação do
/// Position X/Y que fez para Anchor vou querer para todo o Transform»*). Os SEIS que saíram —
/// `row_w`, `section_narrow`, `two_chip_w`, `label_above_gap`, `label_col_w` e o `chip_metrics` que
/// os derivava — eram a **lei adaptativa própria** desta secção: ela decidia sozinha entre *nome ao
/// lado* e *nome por cima*, com a sua própria aritmética.
///
/// ⛔ Medido no mesmo dia: o limiar dela era `interior ≥ 360 px` ⇒ **painel ≥ 380**, e o dock do
/// dono está em `273,3`. *A secção estava no modo «nome por cima» em TODA largura que ele usa* —
/// que é exactamente o que ele fotografou em 14 de Setembro.
pub(super) struct RowStyle<'a> {
    pub x: f32,
    /// A largura INTEIRA da secção. A coluna de animação e as colunas da linha saem daqui.
    pub w: f32,
    pub field_h: f32,
    pub label_font: f32,
    pub label_color: ph2d_vector::Color,
    pub theme: Theme,
    pub store: &'a WidgetStore,
    pub col_gap: f32,
    pub axis_col_w: f32,
    pub tag_box_gap: f32,
    /// O corpo de letra da etiqueta de eixo (`X` / `Y`).
    pub axis_label_font: f32,
}

/// Pinta uma linha e devolve a altura que ela usou.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_row(
    st: &RowStyle<'_>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    row_y: f32,
    row_label: &str,
    left_id: NodeId,
    left_tag: &str,
    left_color: ColorToken,
    left_step: f64,
    right: Option<(NodeId, &str, ColorToken, f64)>,
    // ⭐ **A UNIDADE dos dois chips** (2026-09-15, ordem do dono: *«todos na caixa»*) — a mesma nos
    //    dois, porque um par `X`/`Y` mede a mesma grandeza nos dois.
    //
    // ⚠️ **Aqui ela substitui um rótulo que dizia a RÉGUA activa** (`Position (m)` ⇄
    //    `Position (px)`, pelo `DisplayUnit`): o campo passa a mostrar `12,5 m`, que diz a mesma
    //    coisa **e** o número, no sítio onde o artista olha.
    unit: Option<ph2d_editor_core::widget::Unit>,
) -> f32 {
    // ⭐⭐⭐ **A LINHA DE PROPRIEDADE, a mesma das Âncoras** — ordem do dono, 2026-09-15: *«a mesma
    // formatação do Position X/Y que fez para Anchor vou querer para todo o Transform»*.
    //
    // ⛔ O que saiu daqui foi a lei adaptativa PRÓPRIA desta secção (ver o doc do [`RowStyle`]):
    // ela punha o nome por cima abaixo de um painel de `380`, e o dock do dono está em `273,3`.
    let row = ph2d_editor_core::widget::property_row_columns(st.x, st.w, row_y, st.field_h);
    ph2d_editor_core::widget::paint_property_label(
        text_system,
        scene,
        row_label,
        row.label.x,
        row.label.y + (row.label.h - st.label_font) * 0.5,
        st.label_font,
        row.label.w,
        st.label_color,
    );

    // ⭐⭐ **A letra do eixo é parte da CÉLULA, não do rótulo da linha** — daí o `lead`. O piso
    // continua a ser o da CAIXA (`NUMBER_INPUT_MIN_W_PX`), e a caixa mede `célula − lead`.
    let lead = st.axis_col_w + st.tag_box_gap;
    let componentes: [Option<(NodeId, &str, ColorToken, f64)>; 2] =
        [Some((left_id, left_tag, left_color, left_step)), right];
    let n = componentes.iter().filter(|c| c.is_some()).count();
    let (por_linha, linhas, celula_w) =
        ph2d_editor_core::widget::property_fields_layout(row.control.w, n, st.col_gap, lead);
    let passo = ph2d_tokens::row_pitch_px();
    let caixa_w = (celula_w - lead).max(1.0);

    // ⭐⭐⭐ **O ALINHAMENTO da linha de UM campo cai de graça** — feedback do dono, 2026-05-24:
    // *«a caixa única de Rotation deve se alinhar à caixa de X à esquerda e à direita»*.
    //
    // ⚠️ **Antes era uma FÓRMULA escrita à mão** (`2·two_chip_w + col_gap + axis_col_w +
    // tag_box_gap`), e o comentário dela registava que já tinha estado ERRADA — faltava-lhe o
    // `axis_col_w + tag_box_gap` e a Rotation acabava aquém da borda direita do `Y`. Hoje não há
    // fórmula: com `n = 1` a célula **é** a coluna do controlo, que é exactamente o vão de `X` a
    // `Y` quando eles estão lado a lado, e exactamente a caixa de `X` quando eles empilham.
    // *Uma propriedade que se obtém por construção não pode ficar errada.*
    for (i, comp) in componentes.iter().flatten().enumerate() {
        let (id, tag, cor, passo_scrub) = *comp;
        let cx = row.control.x + (celula_w + st.col_gap) * (i % por_linha) as f32;
        let cy = row.control.y + passo * (i / por_linha) as f32;
        paint_text(
            text_system,
            scene,
            tag,
            cx,
            cy + (st.field_h - st.axis_label_font) * 0.5,
            st.axis_label_font,
            st.axis_col_w,
            resolve(cor, st.theme),
        );
        let rect = Rect::new(cx + lead, cy, caixa_w, st.field_h);
        hit_index.register(id, rect);
        let (state, value, buffer, caret, anchor) = read_number_input(st.store, id);
        let input = NumberInput::new(id, "", value)
            .step(passo_scrub)
            .visual((state, st.store.hover_live(id)))
            .suffix(unit.map(ph2d_editor_core::widget::Unit::suffix));
        paint_number_input_with_buffer(
            &input,
            Some(buffer),
            caret,
            anchor,
            rect,
            scene,
            text_system,
            st.theme,
        );
    }
    // ⚠️⚠️ **A altura devolvida é a do CONTEÚDO, não o passo inteiro** — o chamador desta secção
    // soma o `row_gap` a seguir (`cur_y += h + row_gap`), ao contrário do `rows::fields_row`, cujo
    // chamador escreve `cur_y = fields_row(…)`. Com `passo × linhas` o Transform ficaria com um vão
    // a mais por linha do que as Âncoras, e o dono pediu **a mesma formatação**.
    // ⇒ `field_h + (linhas − 1) × passo`, que somado ao `row_gap` dá exactamente `passo × linhas`.
    let total_h = st.field_h + (linhas as f32 - 1.0) * passo;

    // O ponto da coluna de animação — **um por LINHA**, não por campo: a propriedade é a
    // linha, e um par X/Y é uma propriedade com duas componentes.
    ph2d_editor_core::widget::paint_decorator_dot(scene, st.theme, row.dot);
    total_h
}
