//! Gates das LEGENDAS da fila de ferramentas — a palavra curta por cima de cada chip.

use super::menu_bar::tests::em_todo_estilo;
use super::tool_bar::{bar_rail, tool_bar_h};
use crate::interaction::WidgetStore;
use crate::widget::{RailAxis, RailButtonSize, ToolRailEntry, paint_tool_rail_axis};
use crate::zones::Rect;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

fn legenda(e: &ToolRailEntry) -> Option<&str> {
    match e {
        ToolRailEntry::Icon { sub, .. }
        | ToolRailEntry::Compound { sub, .. }
        | ToolRailEntry::Swatch { sub, .. }
        | ToolRailEntry::Glyph { sub, .. } => Some(sub.as_str()).filter(|s| !s.is_empty()),
        ToolRailEntry::Divider => None,
    }
}

/// ⭐⭐ **Nenhuma legenda da fila sai cortada — em nenhuma fonte, peso, tamanho de texto, nitidez
/// ou tamanho de botão** (2026-10-02).
///
/// ⛔ O defeito (foto do primeiro passo desta wave, tamanho de fábrica): `MOVE` -> `M…`, `SCALE` ->
/// `S…`, `UNDO` -> `U…`; no `Large`, `MOVE` -> `…`. A legenda é uma palavra SOLTA por cima do chip,
/// e era medida como texto DENTRO de uma caixa com borda — o respiro de `2·Md` comia `16` dos `36`.
///
/// A régua é o que foi PINTADO (o instrumento das elisões), sobre a fila real: as quatro
/// composições (Painter × Image Tools), cada uma desenhada como o quadro a desenha.
#[test]
fn nenhuma_legenda_da_fila_de_ferramentas_e_cortada() {
    let acusados = em_todo_estilo(|ts| {
        let mut fora = Vec::new();
        for size in [
            RailButtonSize::Small,
            RailButtonSize::Medium,
            RailButtonSize::Large,
        ] {
            let mut store = WidgetStore::default();
            store.set_rail_button_size(size);
            for painter in [false, true] {
                for image_tools in [false, true] {
                    let rail = bar_rail(&store, painter, image_tools);
                    let legendas: Vec<&str> = rail.entries.iter().filter_map(legenda).collect();
                    assert!(
                        !legendas.is_empty(),
                        "a fila não tem legendas — fixtura vazia"
                    );
                    let faixa = Rect::new(0.0, 0.0, 4000.0, tool_bar_h(size, 1));
                    let mut scene = VectorScene::new();
                    let (_, medidos) = crate::text_elide::elisao::medindo(|| {
                        paint_tool_rail_axis(
                            &rail,
                            faixa,
                            &mut scene,
                            ts,
                            Theme::default(),
                            &store,
                            &|_| None,
                            false,
                            RailAxis::Horizontal,
                        );
                    });
                    fora.extend(
                        medidos
                            .iter()
                            .filter(|m| legendas.contains(&m.texto.as_str()) && !m.coube())
                            .map(|m| {
                                format!(
                                    "{size:?} painter={painter} image={image_tools}: «{}» -> «{}» ({:.1} px)",
                                    m.texto, m.pintado, m.largura
                                )
                            }),
                    );
                }
            }
        }
        fora
    });
    assert!(
        acusados.is_empty(),
        "legendas cortadas ({}):\n  {}",
        acusados.len(),
        acusados
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
