//! **A GEOMETRIA da família de LISTA** — onde cada opção foi desenhada.
//!
//! ⚠️ **Irmão do `skin.rs`, e o corte é de ASSUNTO:** ali responde-se *como um tipo APARECE* (um
//! `match`, um braço por tipo, cada um terminando no pintor nativo); aqui, *onde a opção `i`
//! caiu*. As duas perguntas crescem por motivos diferentes — a primeira com o catálogo do design
//! system, a segunda com quantas famílias têm sub-controles dentro de uma row.
//!
//! ⚠️ **`skin/`, e não `src/widget/`**: o `ph2d-widget-sync` varre os `*.rs` do TOPO daquela pasta
//! para gerar o bloco de `mod`, e um arquivo solto ali viraria um "widget" sem showcase e sem
//! a11y — a cicatriz do `command_palette`.

use super::WidgetKind;
use crate::zones::Rect;

/// **ONDE a opção `index` desta row foi desenhada** — a porta única da geometria da lista.
///
/// ⚠️ **Ela mora aqui porque é aqui que se decide COMO um tipo é desenhado.** Os três da família
/// que mostram as opções em linha dividem o retângulo em partes iguais, e é o pintor de cada um
/// que o faz (`Tabs::tab_rect`, `RadioGroup::option_rect`); um painel que recomputasse a divisão
/// seria a **segunda resposta** à mesma pergunta, e a divergência é da pior espécie — a aba acende
/// num lugar e responde noutro.
///
/// ⚠️ **`None` para o dropdown, e não por omissão:** as opções dele **não estão dentro de `host`**.
/// Elas vivem no popover, laid out por `Dropdown::option_rect_in` contra o painel que o pintor de
/// facto desenhou (que pode ter virado para cima). Devolver um retângulo aqui daria uma resposta
/// plausível e errada.
///
/// ⚠️ **E `None` também para quem não tem opções** — um controle que não é de lista não tem opção
/// `index` nenhuma, e inventar uma seria pôr retângulos de hit sobre um `Slider`.
#[must_use]
pub fn inline_option_rect(
    kind: WidgetKind,
    host: Rect,
    index: usize,
    count: usize,
) -> Option<Rect> {
    if !kind.takes_options() || kind.defers_a_popover() || index >= count {
        return None;
    }
    // As três em linha dividem a largura em partes iguais — `Tabs::tab_rect` e o
    // `RadioGroup::option_rect` HORIZONTAL são a mesma expressão, e é a que o `paint_tabs` usa
    // também para a variante segmentada (ver o braço do `SegmentedAdaptive`).
    let w = host.w / count.max(1) as f32;
    Some(Rect::new(host.x + w * index as f32, host.y, w, host.h))
}

#[cfg(test)]
#[path = "geometry_tests.rs"]
mod tests;
