//! **A SEMENTE do painel** — o que o `paint` escreve no `WidgetStore` antes e depois de desenhar.
//!
//! ⚠️ **Módulo irmão por RESPONSABILIDADE.** O `paint` responde *«que pixels saem?»*; estas três
//! respondem *«que estado o quadro seguinte precisa de encontrar?»* — e cada uma tem a sua
//! própria armadilha registada, que se perdia no meio de 219 linhas de desenho.

use super::{MAX_PARAM_ROWS, ParamRow, param_swatch_id, param_text_id, rows_paint};
use crate::curve_row::CurveWidgets;
use crate::gradient_row::ColourRowWidgets;
use crate::snapshot::ParamsSnapshot;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;

/// **AS SEÇÕES, semeadas ANTES do desenho que elas governam.**
///
/// O collapse genérico exige DOIS sítios: o hit-rect (no paint) e a MARCA aqui. Sem a marca o
/// cabeçalho pinta um chevron e não dobra — o título morto que o painel do Vector já pagou.
/// Semeado na fase mutável porque os títulos são do NÓ selecionado, e o `populate` (estático)
/// não os conhece.
///
/// ⚠️ **Isto vivia DEPOIS do `paint_rows`, e mudar de sítio foi uma CORREÇÃO, não arrumação:**
/// uma seção que nasce fechada era desenhada ABERTA no primeiro quadro e só fechava no
/// seguinte — um pisca visível, e o censo de altura (que pinta exactamente uma vez) media o nó
/// com a seção aberta e a acusar de estourar o dock. *Uma semente que corre depois do desenho
/// que ela governa mostra um quadro do estado errado.*
///
/// ⚠️ **`collapsed_choice` distingue «o artista não escolheu» de «ele escolheu aberto»** — sem
/// essa distinção este laço re-fecharia a gaveta a cada quadro, e o clique de quem a abriu
/// duraria um frame.
pub(crate) fn seed_sections(store: &mut WidgetStore, snap: &ParamsSnapshot) {
    for (title, _) in &snap.sections {
        let id = rows_paint::sections::section_id(title);
        store.mark_collapsible_section(id);
        if snap.folded_by_default.contains(title) && store.collapsed_choice(id).is_none() {
            store.set_collapsed(id, true);
        }
    }
}

/// ⭐⭐ **AS DICAS, antes de as rows serem pintadas.**
///
/// O `populate` não serve: ele regista os widgets de TODOS os slots antes de saber que nó está
/// seleccionado. Aqui o snapshot já existe, e o `store` mutável também.
///
/// ⚠️ **O id tem de ser o MESMO que o hover usa** — o `paint_hover_tooltip` lê
/// `tooltip_for(hot_id())`, e o `hot_id` vem do hit-index, que a row regista sob
/// `param_text_id(i)`. Uma dica noutro id seria uma dica que existe e ninguém alcança.
pub(crate) fn seed_tooltips(store: &mut WidgetStore, snap: &ParamsSnapshot) {
    // ⛔⛔ **TODOS os slots, e a AUSÊNCIA escreve-se** — o gate apanhou a 1.ª redacção,
    // que só escrevia quando havia ajuda. O painel re-semeia a cada quadro sobre um
    // POOL de ids partilhado por todos os nós: a dica do nó anterior sobrevivia e
    // pairava sobre o campo do seguinte. *Um cache por slot posicional tem de ser
    // escrito na ausência, senão ele não é um cache — é um resíduo.*
    //
    // ⚠️ Uma string vazia REMOVE (contrato do `set_tooltip`), então a ausência tem a
    // mesma porta que a presença.
    for slot in 0..MAX_PARAM_ROWS {
        let help = match snap.rows.get(slot) {
            Some(ParamRow::Text(t)) => t.help.clone().unwrap_or_default(),
            _ => String::new(),
        };
        store.set_tooltip(param_text_id(slot), help);
    }
}

/// **FASE C — os handles que só existem depois do desenho.**
///
/// O editor de curva e o de gradiente emitem por-quadro os seus `CurvePoint`/`Button`, porque a
/// posição deles é o próprio desenho; o `paint_rows` segura o `hit_index` e não pode registá-los
/// através do store imutável. Eles voltam nos dois pacotes e entram aqui.
pub(crate) fn register_interactive(
    store: &mut WidgetStore,
    snap: &ParamsSnapshot,
    curve_widgets: &CurveWidgets,
    gradient_widgets: &ColourRowWidgets,
) {
    // O collapse genérico exige DOIS sítios: o hit-rect (no paint) e a MARCA aqui.
    // Sem a marca o cabeçalho pinta um chevron e não dobra — o título morto que o
    // painel do Vector já pagou. Marcado na fase mutável porque os títulos são do NÓ
    // selecionado, e o `populate` (estático) não os conhece.
    for row in &snap.rows {
        if let ParamRow::Color(c) = row {
            store.register_picker_swatch(param_swatch_id(c.channels[0]));
        }
    }
    for &(id, parent, index, canvas) in &curve_widgets.points {
        store.register(
            id,
            InteractiveState::CurvePoint {
                parent,
                channel: 0,
                index,
                canvas,
            },
        );
    }
    for &id in &curve_widgets.buttons {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Gradient editor (doc 85): the position markers are `CurvePoint` handles (the
    // dispatch normalizes a drag off `canvas`), each stop swatch a picker swatch (a
    // Down opens the shared OKLCH picker; seeded here from the paint's srgb so it
    // opens on the stop's colour), and `+`/`−`/interp are buttons.
    for &(id, parent, index, canvas) in &gradient_widgets.markers {
        store.register(
            id,
            InteractiveState::CurvePoint {
                parent,
                channel: 0,
                index,
                canvas,
            },
        );
    }
    for &(sid, srgb) in &gradient_widgets.swatches {
        store.register_picker_swatch(sid);
        store.set_widget_color(sid, srgb);
    }
    for &id in &gradient_widgets.buttons {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}
