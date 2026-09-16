//! **§5 9-Slice** — a seção que a spec
//! ([`03_inspector_secoes.md`](../../../../docs/Sprite_projeto/03_inspector_secoes.md) §3.5)
//! declarou em 2026-05 e que ninguém construiu: até 2026-08-21 `git grep -c SliceNine` dava
//! **0** em todo o repositório.
//!
//! # Duas decisões de desenho, e as razões
//!
//! 1. **Sem componente, a seção mostra UM botão e mais nada.** Não pinta bordas a zero: ausência
//!    de autoria não é «bordas a zero», e mostrar zeros seria afirmar um valor que não existe.
//!    ⚠️ E anexar é **inerte** (`SliceNine::INERT` = Simple + bordas 0): um botão que abre uma
//!    seção não pode ser uma edição destrutiva disfarçada.
//! 2. **Os oito modos por-região são uma GRELHA 3×3 que cicla ao clique**, não oito dropdowns.
//!    A spec pedia dropdowns; oito de quatro opções são 32 alvos e ~8 linhas num painel estreito.
//!    A grelha ocupa três linhas, tem oito alvos — e **parece a coisa que edita**. O miolo não é
//!    clicável: ele obedece a `Fill Center`, que é uma lei própria.

use super::*;
use ph2d_editor_core::screens::hero::InspectorSliceInfo;
use ph2d_editor_core::widget::SectionFold;
use ph2d_editor_core::widget::{SegmentedAdaptive, SegmentedOption, paint_segmented_adaptive};
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;

/// Os nomes que esta secção pinta à esquerda — a fonte da coluna dela.
const NOMES: [&str; 6] = [
    "panel.inspector.slice.tile_mode",
    "panel.inspector.slice.borders_l_t_px",
    "panel.inspector.slice.borders_r_b_px",
    "panel.inspector.slice.size_x_y_m_0",
    // ⭐ As linhas de MARCAR partilham a coluna — logo o nome mais largo pode ser o de uma delas.
    "panel.inspector.slice.enable_9_slice",
    "panel.inspector.slice.fill_center",
];
/// Quantas componentes tem a linha que esta secção não quer ver quebrar.
const CAMPOS: usize = 2;

/// ⭐⭐ **A COLUNA DESTA SECÇÃO — uma só, medida sobre TODOS os nomes que ela pinta.**
///
/// ⛔ Report do dono, 2026-09-15: *«a caixa recua quando na verdade o nome deveria criar as
/// colunas»*. Ver [`ph2d_editor_core::property_row::Seccao`]. ⚠️ Entram os nomes das linhas de
/// campo **e** os das linhas cujo controlo é construído à mão (o segmentado): *uma secção em que
/// metade das linhas mede e a outra metade não é uma secção com duas colunas.*
fn seccao(text_system: &mut TextSystem) -> ph2d_editor_core::property_row::Seccao {
    ph2d_editor_core::property_row::Seccao::medida(text_system, CAMPOS, &NOMES.map(tr))
}

/// Rótulos do Tile Mode global, tags `0..=1` de `SliceTileMode`.
pub const TILE_MODE_LABELS: [TextKey; 2] = [
    TextKey::new("panel.inspector.slice.continuous"),
    TextKey::new("panel.inspector.slice.whole"),
];

/// A dica do Tile Mode: é aqui que mora a cura da emenda rente à borda.
const WHOLE_HINT: TextKey = TextKey::new("panel.inspector.slice.whole_entire_tiles_so_the");
/// **Nenhum segmento aceso** — a afordância de divergência numa seleção múltipla.
const NOTHING_LIT: usize = usize::MAX;

const FIELD_H: f32 = 24.0; // LITERAL-PX-OK: altura de campo do Inspector
/// Passo de scrub do tamanho alvo, em metros. Uma medida do MUNDO, não do desenho: um passo de
/// um token de espaçamento não teria significado num campo em metros.
const SIZE_STEP: f64 = 0.1; // LITERAL-PX-OK: passo de scrub em metros

/// As duas linhas que só existem em `Tiled`: o Tile Mode global e, dentro de `Adaptive`, o
/// limiar de ladrilho. Devolve o `y` seguinte.
///
/// ⚠️ **Sai daqui por CAP de função** (200): com elas inline a `paint_slice_section` media 211.
/// *A cura de um teto estourado é o corte.*
#[allow(clippy::too_many_arguments)]
fn tiled_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSliceInfo,
) -> f32 {
    let sec = seccao(text_system);
    let mut cur_y = y;

    // ⭐⭐ **O nome à ESQUERDA, como as caixas numéricas desta mesma secção** (2026-09-15) — deixá-lo
    //    por cima aqui e ao lado três linhas abaixo **é** a queixa do dono, dentro de uma secção só.
    let tm_row = super::rows::property_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        cur_y,
        FIELD_H,
        tr("panel.inspector.slice.tile_mode"),
        sec,
    );
    let tm = SegmentedAdaptive::new(
        core_ids::INSP_LIVE_SLICE_SECTION,
        tr("panel.inspector.slice.tile_mode"),
        ids::INSP_SLICE_TILE_MODE
            .iter()
            .zip(TILE_MODE_LABELS)
            .map(|(&id, label)| SegmentedOption::new(id, label.tr()))
            .collect(),
    )
    .selected(if info.mixed.tile_mode {
        NOTHING_LIT
    } else {
        usize::from(info.tile_mode_tag).min(TILE_MODE_LABELS.len() - 1)
    });
    cur_y += paint_segmented_adaptive(
        &tm,
        tm_row.control,
        scene,
        text_system,
        theme,
        store,
        hit_index,
    )
    .max(FIELD_H)
        + Spacing::Sm.px();
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, tm_row.dot);
    // ⚠️ **A emenda rente ao canto tem NOME aqui** (smoke do Enio, 2026-08-22). Ela não é um
    // defeito de textura: é o último ladrilho cortado a meio, e o utilizador não tem como
    // adivinhar que o remédio se chama «Whole». Uma linha diz onde ele está.
    cur_y + hint(scene, text_system, theme, x, w, cur_y, WHOLE_HINT.tr())
}

/// Uma dica de rodapé, na cor terciária. **Devolve a altura que ela de facto ocupou.**
///
/// ⚠️ **É `paint_text_block` e não `paint_text` de propósito, e o motivo é um defeito que este
/// ficheiro cometeu** (smoke do Enio, 2026-08-22): estas dicas **quebram em duas linhas** num
/// painel estreito, e avançar `label_font` por elas escrevia o rótulo seguinte por cima. O
/// doc-comment do `paint_text_block` já descrevia exatamente este acidente, vindo do painel de
/// física — *quem empilha texto de comprimento variável tem de PERGUNTAR ao pintor quanto ele
/// gastou, nunca estimar*.
#[allow(clippy::too_many_arguments)]
pub(super) fn hint(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    text: &str,
) -> f32 {
    let h = paint_text_block(
        text_system,
        scene,
        text,
        x,
        y,
        TypeToken::Sm.px(),
        w,
        resolve(ColorToken::Text3, theme),
    );
    h + Spacing::Sm.px()
}

/// Pinta a §5 e devolve o `y` a seguir a ela.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_slice_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSliceInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let color_id = core_ids::INSP_LIVE_SLICE_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: acento neutro por omissão
    let header = section_header(store, core_ids::INSP_LIVE_SLICE_SECTION, "9-Slice").color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_SLICE_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;

    // ⚠️ **UMA CAIXA, e ela é a única porta.** Antes disto havia duas — um segmentado
    // `Simple`/`9-Slice` e um botão `+ Add 9-Slice` — a dizer a mesma coisa por caminhos
    // diferentes (Enio, 2026-08-22). *Duas portas para o mesmo estado é como as duas divergem*, e
    // um controlo de dois estados disfarçado de escolha entre modos é a mesma afordância a mentir
    // que os cantos da grelha tinham.
    //
    // Ligá-la sem componente ANEXA-O — e continua inerte, porque as bordas nascem a zero e bordas
    // a zero colapsam no sprite de sempre. Desligá-la GUARDA os valores: uma caixa que perdesse
    // dados ao desmarcar não seria uma caixa.
    let cb_h = ph2d_tokens::ROW_H_PX; // ⛔ era `18.0`, o MESMO literal em TREZE sitios: a linha de marcar e' uma linha de propriedade, e a altura dela e' a do app (report do dono 2026-09-15: a marca enchia a caixa toda)
    let on = info.present && info.draw_mode_tag == 1;
    let en_value = if info.mixed.enabled {
        CheckboxValue::Indeterminate
    } else if on {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    };
    let en_rect = Rect::new(x, cur_y, w, cb_h);
    hit_index.register(ids::INSP_SLICE_ENABLE, en_rect);
    paint_checkbox(
        &Checkbox::new(
            ids::INSP_SLICE_ENABLE,
            tr("panel.inspector.slice.enable_9_slice"),
        )
        .visual(store.checkbox_visual(ids::INSP_SLICE_ENABLE))
        .value(en_value)
        .seccao(seccao(text_system)),
        en_rect,
        scene,
        text_system,
        theme,
    );
    cur_y += cb_h + ph2d_tokens::control_gap_px();

    // Desligado: a seção não tem mais nada a mostrar. O «Remove» só aparece quando há de facto
    // autoria guardada para remover — sem componente, não há.
    if !on {
        return fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX);
    }

    let sec = seccao(text_system);
    // Bordas, em pixels da fonte, na ordem do array: [L, T] e depois [R, B].
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.slice.borders_l_t_px"),
        &[ids::INSP_SLICE_BORDER[0], ids::INSP_SLICE_BORDER[1]],
        1.0,
        Some(ph2d_editor_core::widget::Unit::Px),
        sec,
    );
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.slice.borders_r_b_px"),
        &[ids::INSP_SLICE_BORDER[2], ids::INSP_SLICE_BORDER[3]],
        1.0,
        Some(ph2d_editor_core::widget::Unit::Px),
        sec,
    );
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.slice.size_x_y_m_0"),
        &[ids::INSP_SLICE_SIZE[0], ids::INSP_SLICE_SIZE[1]],
        SIZE_STEP,
        Some(ph2d_editor_core::widget::Unit::Meters),
        sec,
    );

    // Fill Center.
    let (_, fc_value) = store
        .checkbox(ids::INSP_SLICE_FILL_CENTER)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Checked));
    let fc_rect = Rect::new(x, cur_y, w, cb_h);
    hit_index.register(ids::INSP_SLICE_FILL_CENTER, fc_rect);
    paint_checkbox(
        &Checkbox::new(
            ids::INSP_SLICE_FILL_CENTER,
            tr("panel.inspector.slice.fill_center"),
        )
        .visual(store.checkbox_visual(ids::INSP_SLICE_FILL_CENTER))
        .value(fc_value)
        .seccao(sec),
        fc_rect,
        scene,
        text_system,
        theme,
    );
    cur_y += cb_h + ph2d_tokens::control_gap_px();

    // A grelha por-região e o Tile Mode valem nos DOIS modos de nove quads.
    //
    // ⚠️ **O Tile Mode esteve escondido em `Sliced` e isso era falso.** Ele governa a contagem
    // de ladrilhos de *qualquer região que repita* — e uma célula posta em `R`/`M` repete
    // também em `Sliced` (é a lei do `tile_count`: `Tiled` só acrescenta que o `S` passa a
    // repetir). Escondê-lo ali punha o artista a mexer numa célula cujo resultado ele não podia
    // afinar, que é a mesma classe de mentira do controlo pintado sobre um modo que o ignora.
    cur_y = super::slice_grid::region_grid(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
    );
    cur_y = tiled_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
    );
    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// As listas de rótulos e os arrays de ids têm de ter o mesmo comprimento — senão um modo
    /// fica sem botão (inalcançável) ou um botão fica sem rótulo (pinta vazio).
    ///
    /// ⚠️ **A comparação contra as variantes do MOTOR não pode viver aqui:** este painel é chrome
    /// e não depende do `ph2d-ecs`, de propósito (a mesma razão do `FILTER_LABELS` da §9). Quem
    /// vê as duas metades é a shell, e o gate está lá:
    /// `shells/desktop/tests/it/the_slice_section_offers_every_mode_the_engine_has.rs`.
    #[test]
    fn every_id_array_has_a_label_for_each_slot() {
        assert_eq!(ids::INSP_SLICE_TILE_MODE.len(), TILE_MODE_LABELS.len());
        // ⚠️ A contagem de regiões contra o MOTOR vive no gate da shell — aqui só a grelha.
        assert_eq!(
            ids::INSP_SLICE_REGION.len(),
            8,
            "a moldura 3x3 tem oito celulas"
        );
        assert_eq!(
            ids::INSP_SLICE_BORDER.len(),
            4,
            "as bordas sao [l, t, r, b]"
        );
    }

    /// A grelha REAL cobre a moldura 3×3 menos o miolo, sem repetir célula.
    #[test]
    fn the_painted_grid_is_the_ring_around_the_centre() {
        let mut seen = super::super::slice_grid::REGION_CELLS.to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 8, "duas celulas pintadas no mesmo sitio");
        assert!(!seen.contains(&(1, 1)), "o miolo nao e' uma das oito");
    }
}
