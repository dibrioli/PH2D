//! The per-row paint loop (split out of `lib.rs::paint` for the HR-18 200-LOC fn cap +
//! the 600-LOC panel-file cap). One `match` arm per [`ParamRow`] kind, each delegating to
//! the SHARED source-of-truth painters (slider/chip, swatch, checkbox, segmented button,
//! number box, text field). `super` is the crate root, so the pooled-id helpers and the
//! `normalized_track`/`row_value` mappers are in scope.

use super::curve_row::{self, CurveWidgets};
use super::gradient_row::{self, ColourRowWidgets};
use super::{
    CHANNELS_EXTRA_BASE, MAX_ENUM_OPTIONS, MAX_PARAM_ROWS, ParamRow, normalized_track,
    paint_angle_row, paint_seed_row, paint_text_row, param_checkbox_id, param_chip_id,
    param_enum_id, param_number_id, param_reroll_id, param_slider_id, param_swatch_id,
    param_text_id, row_value,
};
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::text_elide::paint_text_elided;
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
use ph2d_editor_core::widget::{
    Checkbox, CheckboxValue, ColorSwatch, DEFAULT_LABEL_W, SwatchSize, paint_checkbox,
    paint_color_swatch, paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[path = "rows_paint_editors.rs"]
mod editors;
#[path = "rows_paint_number.rs"]
mod number;
#[path = "rows_paint_reset.rs"]
mod reset;
#[path = "rows_paint_sections.rs"]
pub(crate) mod sections;
use reset::{RESET_GUTTER_W, paint_reset_button, row_is_modified};
#[path = "rows_paint_kinds.rs"]
mod kinds;
use kinds::{
    paint_channels_row, paint_color_row, paint_driven_row, paint_enum_row, paint_scalar_row,
    paint_source_row, paint_toggle_row,
};

/// **QUE WIDGET CADA ESPÉCIE DE ROW PINTA** — o `match` que o [`paint_rows`] delega.
///
/// ⚠️ Separado pelo teto de FUNÇÃO (HR-18, 200 LOC em `ph2d-panel-*`), no corte que a
/// pergunta desenha: lá fica *a travessia da lista* (a dobra que atravessa iterações, o
/// recorte, o avanço do `y`), aqui *o que uma row é*. ⚠️ E o `y` entra e SAI: uma row de
/// caixa gasta uma altura, um editor de curva gasta a sua — quem soma é o chamador.
#[allow(clippy::too_many_arguments)]
fn paint_one_row(
    row: &ParamRow,
    i: usize,
    inner_x: f32,
    inner_w: f32,
    chip_w: f32,
    y: f32,
    row_gap: f32,
    label_font: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    curve_widgets: &mut CurveWidgets,
    gradient_widgets: &mut ColourRowWidgets,
) -> f32 {
    let mut y = y;
    match row {
        // A DRIVEN param (doc 58): the wire decides the number, so there is no widget —
        // just the label and the live value. Nothing is registered, so nothing can be
        // dragged; the artist unplugs the wire to take the knob back.
        ParamRow::Scalar(row) if row.driven_by.is_some() => {
            paint_driven_row(
                row,
                inner_x,
                inner_w,
                y,
                label_font,
                scene,
                text_system,
                theme,
            );
            y += ph2d_tokens::row_pitch_px();
        }
        ParamRow::Scalar(row) => {
            y = paint_scalar_row(
                row,
                i,
                inner_x,
                inner_w,
                chip_w,
                row_gap,
                y,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
        }
        ParamRow::Color(row) => {
            y = paint_color_row(
                row,
                inner_x,
                inner_w,
                row_gap,
                y,
                label_font,
                hit_index,
                scene,
                text_system,
                theme,
            );
        }
        ParamRow::Toggle(row) => {
            y = paint_toggle_row(
                row,
                i,
                inner_x,
                inner_w,
                row_gap,
                y,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
        }
        ParamRow::Enum(row) => {
            y = paint_enum_row(
                row,
                i,
                inner_x,
                inner_w,
                row_gap,
                y,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
        }
        // ⭐ A row de TEXTO é a única que pode estar ERRADA — ela é a única com texto livre.
        // Pinta a mesma caixa das irmãs e, se houver queixa, mais UMA linha por baixo.
        ParamRow::Text(text) if text.problem.is_some() => {
            let used = number::paint_box_row(
                row,
                i,
                inner_x,
                inner_w,
                y,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            )
            .expect("o braço e a porta casam por construção");
            y += used;
            let msg = text.problem.as_deref().unwrap_or_default();
            // ⚠️ **Alinhada com o CAMPO, não com a margem** — ela fala do que está na caixa, e
            // uma linha à esquerda do rótulo leria-se como outra propriedade.
            paint_text_elided(
                text_system,
                scene,
                msg,
                inner_x + DEFAULT_LABEL_W,
                y + (ROW_H_PX - label_font) * 0.5,
                label_font,
                (inner_w - DEFAULT_LABEL_W).max(0.0),
                resolve(ColorToken::Danger, theme),
            );
            // ⚠️ **Nada é registado no `HitIndex`**: um aviso não se clica. Registá-lo poria um
            // alvo mudo por cima do campo, que é o defeito que a caça aos knobs mortos nomeia.
            y += ph2d_tokens::row_pitch_px();
        }
        ParamRow::Angle(_) | ParamRow::Seed(_) | ParamRow::Text(_) => {
            // As rows-CAIXA: uma altura de row, um `Rect` explícito.
            let used = number::paint_box_row(
                row,
                i,
                inner_x,
                inner_w,
                y,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            )
            .expect("o braço e a porta casam por construção");
            y += used + row_gap;
        }
        ParamRow::Channels(row) => {
            y = paint_channels_row(
                row,
                i,
                inner_x,
                inner_w,
                row_gap,
                y,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
        }
        ParamRow::File(row) => {
            y = kinds::paint_file_row(
                row,
                i,
                inner_x,
                inner_w,
                row_gap,
                y,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
        }
        ParamRow::Source(row) => {
            y = paint_source_row(
                row,
                i,
                inner_x,
                inner_w,
                row_gap,
                y,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
        }
        ParamRow::Curve(_) | ParamRow::Palette(_) | ParamRow::Gradient(_) => {
            // Os editores de várias linhas — cada um devolve a própria altura.
            let used = editors::paint_editor_row(
                row,
                i,
                inner_x,
                inner_w,
                y,
                label_font,
                hit_index,
                scene,
                text_system,
                theme,
                curve_widgets,
                gradient_widgets,
            )
            .expect("o braço e a porta casam por construção");
            y += used + row_gap;
        }
    }
    y
}

/// Paint each param row from `body_top` down, registering hit rects as it goes.
///
/// Devolve, além dos widgets que a fase mutável registra, **a ALTURA que de fato usou**.
///
/// ⚠️ Ela existe porque o painel **não rola**: um teto de linhas alto o bastante para caber todo
/// param do registry (`MAX_PARAM_ROWS`, medido pelo censo na shell) só é honesto se essas linhas
/// couberem na altura do inspector — senão a linha 14 deixa de ser cortada pelo `.take()` e passa
/// a ser cortada pela borda da tela, que é a MESMA invisibilidade por outra porta. Quem responde
/// *"quanto isto ocupou?"* é o pintor, uma vez; um segundo cálculo de altura ao lado dele
/// divergiria no dia em que uma linha composta mudar de tamanho.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_rows(
    rows: &[ParamRow],
    inner_x: f32,
    inner_w: f32,
    chip_w: f32,
    row_gap: f32,
    body_top: f32,
    label_font: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    modified: &std::collections::BTreeSet<String>,
    section_at: &[(String, usize)],
) -> (CurveWidgets, ColourRowWidgets, f32) {
    let mut y = body_top;
    // Toda row cede a MESMA calha à direita, tenha ou não o que reverter — largura que depende
    // do estado é rótulo que se mexe quando você toca nele (`rows_paint_reset`).
    let inner_w = (inner_w - RESET_GUTTER_W).max(0.0);
    let mut curve_widgets = CurveWidgets::new();
    let mut gradient_widgets = ColourRowWidgets::new();
    // Dobrada até o próximo cabeçalho. Uma row DOBRADA é pulada no desenho e mantém o índice
    // `i` — os eventos enumeram a MESMA lista, então pular no pintor sem pular no roteador
    // desalinharia os slots (o bug clássico de lista filtrada).
    let mut collapsed = false;
    // ⚠️ **A dobra do corpo atravessa ITERAÇÕES** (F4b), e é isso que a torna diferente de todo
    // outro painel: aqui uma seção não é um escopo léxico, ela vai de um cabeçalho até o
    // PRÓXIMO. Então a dobra vive num `Option` do laço, é FECHADA antes de o cabeçalho seguinte
    // ser pintado — senão ele sairia dentro do recorte da seção anterior — e a última é fechada
    // depois do laço.
    let mut fold: Option<sections::SectionFold> = None;
    for (i, row) in rows.iter().enumerate().take(MAX_PARAM_ROWS) {
        if sections::has_header_at(section_at, i) {
            y = turn_the_page(
                &mut fold,
                section_at,
                i,
                inner_x,
                inner_w + RESET_GUTTER_W,
                y,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
            collapsed = fold.is_none();
        }
        if collapsed {
            continue;
        }
        if row_is_modified(row, modified) {
            paint_reset_button(
                i,
                inner_x,
                inner_w + RESET_GUTTER_W,
                y,
                store,
                hit_index,
                scene,
                theme,
            );
        }
        y = paint_one_row(
            row,
            i,
            inner_x,
            inner_w,
            chip_w,
            y,
            row_gap,
            label_font,
            store,
            hit_index,
            scene,
            text_system,
            theme,
            &mut curve_widgets,
            &mut gradient_widgets,
        );
    }
    // A última seção não tem cabeçalho seguinte que a feche — o fim da lista é a fronteira dela.
    if let Some(f) = fold {
        y = f.finish(store, scene, hit_index, y);
    }
    (curve_widgets, gradient_widgets, y - body_top)
}

/// **Fecha a seção anterior e abre a seguinte** — o par que uma lista PLANA de rows exige.
///
/// ⚠️ Extraído do `paint_rows` pelo cap de fn do painel, e a ordem dentro dele é load-bearing: a
/// dobra da seção anterior FECHA antes de o cabeçalho novo ser pintado, senão ele sairia dentro
/// do recorte dela.
#[expect(
    clippy::too_many_arguments,
    reason = "espelha a porta de paint das rows deste painel"
)]
fn turn_the_page(
    fold: &mut Option<sections::SectionFold>,
    section_at: &[(String, usize)],
    i: usize,
    inner_x: f32,
    inner_w: f32,
    mut y: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    if let Some(f) = fold.take() {
        y = f.finish(store, scene, hit_index, y);
    }
    let (dy, opened) = sections::header_at(
        section_at,
        i,
        inner_x,
        inner_w,
        y,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    )
    .expect("o `has_header_at` e o `header_at` são a MESMA pergunta");
    *fold = opened;
    y + dy
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{RowDisplay, ScalarRow};
    use ph2d_editor_core::screens::layout::INSPECTOR_MAX_H;

    fn scalar(i: usize) -> ParamRow {
        ParamRow::Scalar(ScalarRow {
            name: "p",
            label: format!("Param {i}"),
            value: 0.5,
            min: 0.0,
            max: 1.0,
            hard_min: 0.0,
            hard_max: 1.0,
            step: 0.01,
            integer: false,
            driven_by: None,
            display: RowDisplay::default(),
        })
    }

    /// **Quanta folga o dock ainda tem** — a sonda que decide quando o painel precisa ROLAR.
    ///
    /// O gate irmão afirma que `MAX_PARAM_ROWS` linhas CABEM; ele é binário e não diz *por
    /// quanto*. Esta imprime o número, porque a decisão que vem a seguir (a varredura PRO do
    /// doc 88 §B3, que dá a cada nó o conjunto completo de params) é exatamente a que consome
    /// essa folga — e escolher entre *"cabe mais uma família"* e *"o painel tem de rolar"*
    /// precisa do px, não do booleano.
    #[test]
    #[ignore = "sonda de medição, não gate"]
    fn measure_the_docks_headroom() {
        let mut hit = HitIndex::default();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let store = WidgetStore::default();
        let head = 64.0; // LITERAL-PX-OK: a mesma folga de titulo que o gate irmao usa
        let mut per_row = 0.0f32;
        for n in [1usize, 8, MAX_PARAM_ROWS] {
            let rows: Vec<ParamRow> = (0..n).map(scalar).collect();
            let (_, _, used) = paint_rows(
                &rows,
                0.0,
                ph2d_editor_core::screens::layout::INSPECTOR_W,
                64.0,
                Spacing::Sm.px(),
                0.0,
                TypeToken::Base.px(),
                &store,
                &mut hit,
                &mut scene,
                &mut text,
                Theme::default(),
                &Default::default(),
                &[],
            );
            per_row = used / n as f32;
            println!(
                "{n:3} linhas: {used:7.1} px  (+{head} de titulo = {:7.1})  sobra {:7.1} de {INSPECTOR_MAX_H}",
                used + head,
                INSPECTOR_MAX_H - used - head
            );
        }
        println!("por linha: {per_row:.1} px");
        println!(
            "cabem {:.0} linhas escalares no dock inteiro",
            (INSPECTOR_MAX_H - head) / per_row
        );
    }

    /// **A altura reportada e a altura VERDADEIRA das linhas.**
    ///
    /// Este gate se chamava `a_full_panel_of_rows_fits_the_inspector` e afirmava que
    /// `MAX_PARAM_ROWS` linhas cabiam no dock — a defesa contra *"conserte o corte subindo o
    /// teto ate a ultima linha ser cortada pela borda da tela em vez do `.take()`"*. O
    /// doc-comment dele terminava dizendo: *"um dia em que ela deixe de caber e um dia em que o
    /// painel precisa ROLAR"*. Esse dia chegou (doc 88 §B3) e a rolagem existe, entao **caber
    /// deixou de ser requisito** — mante-la afirmada faria a proxima linha que nao coubesse
    /// pedir um teto MENOR, que e o oposto da cura.
    ///
    /// O que sobrevive, e agora e load-bearing, e a HONESTIDADE do numero: o scrollbar deriva
    /// `max_scroll` de `content_h - visible_h`, entao um `used` que saturasse na altura do dock
    /// convenceria o painel de que tudo cabe e o artista **perderia a cauda em silencio** — com
    /// as linhas desenhadas, o thumb ausente e a roda inerte. Por isso a afirmacao e sobre
    /// CRESCIMENTO: dobrar as linhas dobra a altura, e o dock nao entra na conta.
    #[test]
    fn the_reported_height_is_the_true_height_of_the_rows() {
        // ⚠️ O teto mora DENTRO do `paint_rows` (`.take(MAX_PARAM_ROWS)`), entao pedir mais
        // linhas nao produz mais altura — 40 e 20 medem os MESMOS 544 px. A consequencia, que
        // vale mais que este gate: com o teto em `MAX_PARAM_ROWS` o corpo mede no maximo ~544 px
        // contra um dock de 880, entao **o painel nao transborda hoje** e a rolagem esta INERTE.
        // Ela nao e enfeite: e o que remove o dock da lista de razoes para o teto nao subir.
        let rows: Vec<ParamRow> = (0..MAX_PARAM_ROWS).map(scalar).collect();
        let mut hit = HitIndex::default();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let store = WidgetStore::default();
        let (_, _, used) = paint_rows(
            &rows,
            0.0,
            ph2d_editor_core::screens::layout::INSPECTOR_W,
            64.0,
            Spacing::Sm.px(),
            0.0,
            TypeToken::Base.px(),
            &store,
            &mut hit,
            &mut scene,
            &mut text,
            Theme::default(),
            &Default::default(),
            &[],
        );
        // E a metade oposta: um painel que nao desenha nada tambem "cresce" trivialmente.
        assert!(used > 0.0, "as linhas nao ocuparam altura nenhuma");

        // Metade das linhas tem de custar (aproximadamente) metade da altura. A tolerancia e de
        // UMA linha porque o passo entre secoes nao e exatamente o passo entre linhas.
        let half: Vec<ParamRow> = (0..MAX_PARAM_ROWS / 2).map(scalar).collect();
        let mut hit2 = HitIndex::default();
        let mut scene2 = VectorScene::new();
        let mut text2 = TextSystem::without_system_fonts();
        let (_, _, half_used) = paint_rows(
            &half,
            0.0,
            ph2d_editor_core::screens::layout::INSPECTOR_W,
            64.0,
            Spacing::Sm.px(),
            0.0,
            TypeToken::Base.px(),
            &store,
            &mut hit2,
            &mut scene2,
            &mut text2,
            Theme::default(),
            &Default::default(),
            &[],
        );
        let per_row = used / MAX_PARAM_ROWS as f32;
        // ⚠️⚠️ **ESCALADO PELAS CONTAGENS REAIS, e não pelo dobro.** A 1.ª redacção comparava
        // `used` com `2 × half_used`, o que assume um tecto PAR — e ela partiu-se no primeiro
        // ÍMPAR (`24 → 25`, em 2026-08-30): `MAX_PARAM_ROWS / 2` dá `12`, o dobro é `24`, e a
        // linha que falta vale exactamente `per_row`, que é a tolerância. *A lei — a altura
        // cresce com as linhas — estava certa; a aritmética da fixtura é que tinha uma premissa
        // escondida sobre a paridade do número.*
        #[allow(clippy::cast_precision_loss)]
        let scaled = half_used * MAX_PARAM_ROWS as f32 / (MAX_PARAM_ROWS / 2) as f32;
        assert!(
            (used - scaled).abs() < per_row,
            "{MAX_PARAM_ROWS} linhas medem {used} px e {} medem {half_used}: a altura reportada \
             nao esta crescendo com as linhas, entao ela foi clampada em algum lugar — e um \
             painel que reporta menos conteudo do que desenha rola de menos e PERDE a cauda",
            MAX_PARAM_ROWS / 2
        );
        // ⭐⭐ **E O DIA CHEGOU: em 2026-08-30 o corpo passou a TRANSBORDAR o dock.**
        //
        // Esta asserção dizia *«no teto de hoje o corpo cabe no dock com folga»* e trazia ao
        // lado a sua própria data de validade: *«o dia em que ela virar vermelha é o dia em
        // que ela passa a trabalhar — e não há nada a consertar nele»*. Com `MAX_PARAM_ROWS`
        // a `30` (os cinco controlos de folha do `source.lsystem`) são **1020 px num dock de
        // 880**.
        //
        // ⇒ o que ela afirma passa a ser o **fato novo**, e ele é load-bearing: a partir daqui
        // a rolagem (doc 88 §B3) **não é mais inerte**, e é `tests_scroll` quem a defende. A
        // asserção que fica é a que impede o defeito real desta transição — *um painel que
        // reporta MENOS altura do que desenha rola de menos e perde a cauda* —, e ela é a de
        // cima, que mede a altura reportada contra as linhas.
        assert!(
            used > INSPECTOR_MAX_H,
            "{MAX_PARAM_ROWS} linhas medem {used} px e o dock tem {INSPECTOR_MAX_H}: o corpo \
             voltou a caber. Se um teto DESCEU, esta asserção e a `tests_scroll` mudam juntas \
             — a rolagem volta a ser inerte e alguém tem de o dizer aqui"
        );
    }
}
