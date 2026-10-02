//! Gates das LEGENDAS da fila de ferramentas — a palavra curta por cima de cada chip.

use super::{bar_rail, tool_bar_h};
use crate::interaction::WidgetStore;
use crate::text_elide::em_todo_estilo;
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
                    let rail = bar_rail(&store, ts, painter, image_tools);
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

/// ⭐⭐ **Nenhuma FACE de pulldown da fila sai cortada** (2026-10-02, escolha do dono: *botão mais
/// largo*) — em todo estado (SPACE nos dois, VIEW nos três, pulldowns de área com o vocabulário do
/// 3D), fonte, peso, tamanho de texto, nitidez e tamanho de botão.
///
/// ⛔ O defeito (foto): o chip era quadrado e `Global` saía `G…`, `Selected` `S…`; no Large, `…`.
#[test]
fn nenhuma_face_de_pulldown_da_fila_e_cortada() {
    let tr = ph2d_i18n::tr;
    // Fixtura: dois pulldowns de área com as faces REAIS do modelador (as mais largas incluídas).
    let area = |face: &str, faces: &[&str]| crate::interaction::AreaMenu {
        label: tr("panel.model3d.area.view").to_owned(),
        face: tr(face).to_owned(),
        faces: faces.iter().map(|k| tr(k).to_owned()).collect(),
        rows: Vec::new(),
    };
    let acusados = em_todo_estilo(|ts| {
        let mut fora = Vec::new();
        for size in [
            RailButtonSize::Small,
            RailButtonSize::Medium,
            RailButtonSize::Large,
        ] {
            for (local, view) in [(false, 0u8), (true, 1), (false, 2)] {
                let mut store = WidgetStore::default();
                store.set_rail_button_size(size);
                store.set_tool_space_local(local);
                store.set_tool_view_mode(view);
                for face in [
                    "viewport.model3d.view.front",
                    "viewport.model3d.view.bottom",
                ] {
                    store.set_area_commands(
                        vec![
                            area(
                                face,
                                &["viewport.model3d.view.bottom", "viewport.model3d.view.user"],
                            ),
                            area(
                                "panel.model3d.shading.matcap",
                                &["panel.model3d.shading.render"],
                            ),
                        ],
                        Vec::new(),
                    );
                    let rail = bar_rail(&store, ts, false, false);
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
                    fora.extend(medidos.iter().filter(|m| !m.coube()).map(|m| {
                        format!(
                            "{size:?}: «{}» -> «{}» ({:.1} px)",
                            m.texto, m.pintado, m.largura
                        )
                    }));
                }
            }
        }
        fora
    });
    assert!(
        acusados.is_empty(),
        "faces cortadas ({}):\n  {}",
        acusados.len(),
        acusados
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
