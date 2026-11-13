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
        | ToolRailEntry::Glyph { sub, .. }
        | ToolRailEntry::Number { sub, .. } => Some(sub.as_str()).filter(|s| !s.is_empty()),
        ToolRailEntry::Text { .. } | ToolRailEntry::Divider => None,
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
///
/// Dono 12/11: em cartões a fila é FINA e não tem legenda (ver
/// `a_fila_fina_nao_tem_legenda_e_nenhuma_face_e_cortada`); esta régua fica com a saída de referência
/// (`PH2D_CARDS=0`), que ainda pinta a legenda por cima do chip.
#[test]
fn nenhuma_legenda_da_fila_de_ferramentas_e_cortada() {
    crate::widget::lista_flutuante::force_cromo_on_this_thread(Some(false));
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
    crate::widget::lista_flutuante::force_cromo_on_this_thread(None);
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
/// seletor de modo), fonte, peso, tamanho de texto, nitidez e tamanho de botão.
///
/// ⛔ O defeito (foto): o chip era quadrado e `Global` saía `G…`, `Selected` `S…`; no Large, `…`.
#[test]
fn nenhuma_face_de_pulldown_da_fila_e_cortada() {
    let tr = ph2d_i18n::tr;
    // Fixtura: dois pulldowns de área com as faces REAIS do seletor de modo (todas as quatro).
    let area = |face: &str, faces: &[&str]| crate::interaction::AreaMenu {
        label: tr("object_mode.menu").to_owned(),
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
                for face in ["object_mode.object", "object_mode.pose"] {
                    store.set_area_commands(
                        vec![
                            area(face, &["object_mode.object", "object_mode.pose"]),
                            area(
                                "object_mode.edit",
                                &["object_mode.draw", "object_mode.edit"],
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

/// ⭐⭐ **Nenhuma face de pulldown da COLUNA vertical (`F9`) é cortada** — o ecrã inteiro com o
/// chrome legado, em todo estilo de texto, tamanho de botão e face. Escolha do dono (02/10):
/// *coluna mais larga* ([`crate::widget::column_width_px`]).
///
/// ⛔ O defeito (sonda): a coluna tinha `57/61/65 px` e o pulldown pedia `57,9`: `Global` saía
/// `Gl…`, `Selected` `S…`/`Sel…`, `Camera` `C…`/`Ca…`.
#[test]
fn nenhuma_face_de_pulldown_da_coluna_e_cortada() {
    let tr = ph2d_i18n::tr;
    let faces: Vec<&str> = [
        "chrome.rail.global",
        "chrome.rail.local",
        "chrome.rail.selected",
        "chrome.rail.camera",
        "chrome.rail.all",
    ]
    .map(tr)
    .to_vec();
    let acusados = em_todo_estilo(|ts| {
        let mut fora = Vec::new();
        let mut vistas = 0;
        for size in [
            RailButtonSize::Small,
            RailButtonSize::Medium,
            RailButtonSize::Large,
        ] {
            for (local, view) in [(false, 0u8), (true, 1), (false, 2)] {
                let mut hero = crate::screens::hero::HeroScreen::new(ph2d_a11y::NodeId(1));
                hero.view.legacy_chrome = true;
                // ⚠️ O hero publica o estilo DELE ao pintar: sem isto só a fábrica era varrida.
                hero.text_style = ph2d_text::active_text_style();
                hero.text_rendering = ph2d_text::active_text_rendering();
                hero.store.set_rail_button_size(size);
                hero.store.set_tool_space_local(local);
                hero.store.set_tool_view_mode(view);
                let mut scene = VectorScene::new();
                let (_, medidos) = crate::text_elide::elisao::medindo(|| {
                    crate::screens::hero::paint_hero_screen(
                        &mut hero,
                        Rect::new(0.0, 0.0, 1366.0, 768.0),
                        &mut scene,
                        ts,
                    );
                });
                // ⚠️ E o pulldown cabe DENTRO da coluna: sem isto uma coluna estreita passava, com o
                // chip a transbordar sobre o canvas (controlo de 02/10: a mutação sobreviveu).
                let coluna = hero.last_layout.expect("layout do quadro").left_rail;
                for id in [crate::ids::TOOL_SPACE, crate::ids::TOOL_HOME] {
                    let r = hero
                        .hit_index
                        .rect_for(id)
                        .expect("o pulldown foi registado");
                    if r.x + r.w > coluna.x + coluna.w {
                        fora.push(format!(
                            "{size:?}: {id:?} sai da coluna ({r:?} ⊄ {coluna:?})"
                        ));
                    }
                }
                for m in medidos.iter().filter(|m| faces.contains(&m.texto.as_str())) {
                    vistas += 1;
                    if !m.coube() {
                        fora.push(format!(
                            "{size:?}: «{}» -> «{}» ({:.1} px) {}",
                            m.texto, m.pintado, m.largura, m.onde
                        ));
                    }
                }
            }
        }
        assert!(
            vistas >= 18,
            "a régua só viu {vistas} faces — a coluna não pintou"
        );
        fora
    });
    assert!(
        acusados.is_empty(),
        "faces cortadas na coluna ({}):\n  {}",
        acusados.len(),
        acusados.join("\n  ")
    );
}

/// ⭐ **Report do dono 25/10** (Pixel Sprite em Draw sem Object · Draw no topo): a coluna de uma
/// FAMÍLIA (a do Pixel, como a do pintor) guarda o seletor de modo na fila de cima; as barras de um
/// MÓDULO (o vetor, F16) não — o controlo do mesmo gate.
#[test]
fn a_coluna_do_pixel_guarda_o_seletor_de_modo_na_fila() {
    use crate::interaction::{AreaMenu, ToolBars};
    let mut ts = ph2d_text::TextSystem::new();
    let chip = crate::ids::area_menu_button(0);
    let tem_chip = |bars: ToolBars, ts: &mut ph2d_text::TextSystem| {
        let mut store = WidgetStore::default();
        store.publish_mode_menu(Some(AreaMenu {
            label: ph2d_i18n::tr("object_mode.menu").to_owned(),
            face: ph2d_i18n::tr("object_mode.draw").to_owned(),
            faces: Vec::new(),
            rows: Vec::new(),
        }));
        store.set_tool_bars(Some(bars));
        bar_rail(&store, ts, false, false)
            .entries
            .iter()
            .any(|e| e.node_id() == Some(chip))
    };
    let pixel = ToolBars {
        pixel_rail: true,
        ..ToolBars::default()
    };
    assert!(
        tem_chip(pixel, &mut ts),
        "a coluna do Pixel escondeu o seletor"
    );
    assert!(
        !tem_chip(ToolBars::default(), &mut ts),
        "controlo: as barras de um módulo (F16) também o mostram"
    );
}

/// ⭐⭐ **Dono 12/11 (maquete C `.c15tl`): a fila em cartões é FINA** — `TOOL_ROW_H_PX` de altura,
/// nenhuma legenda pintada, nenhuma face (Global, Selected, Object Mode) cortada, em toda fonte, peso
/// e tamanho de texto; e cada botão tem nome, que é o que a dica mostra sem a legenda.
#[test]
fn a_fila_fina_nao_tem_legenda_e_nenhuma_face_e_cortada() {
    crate::widget::lista_flutuante::force_cromo_on_this_thread(Some(true));
    let acusados = em_todo_estilo(|ts| {
        let mut fora = Vec::new();
        let store = WidgetStore::default();
        for painter in [false, true] {
            for image_tools in [false, true] {
                let rail = bar_rail(&store, ts, painter, image_tools);
                if !super::fila_fina(&rail) {
                    fora.push(format!(
                        "painter={painter} image={image_tools}: a fila não é fina"
                    ));
                    continue;
                }
                let legendas: Vec<&str> = rail.entries.iter().filter_map(legenda).collect();
                for e in &rail.entries {
                    if e.node_id().is_some() && e.label().is_none_or(str::is_empty) {
                        fora.push(format!("{:?} sem nome para a dica", e.node_id()));
                    }
                }
                let faixa = Rect::new(0.0, 0.0, 4000.0, ph2d_tokens::TOOL_ROW_H_PX);
                let mut scene = VectorScene::new();
                let barra = crate::widget::cromo_barra::Barra::de(Theme::default())
                    .expect("cartões ligados nesta thread");
                let (_, medidos) = crate::text_elide::elisao::medindo(|| {
                    crate::widget::paint_fila_fina(
                        barra,
                        &rail,
                        faixa,
                        &mut scene,
                        ts,
                        Theme::default(),
                        &store,
                    );
                });
                for m in &medidos {
                    if legendas.contains(&m.texto.as_str()) && !m.texto.is_empty() {
                        fora.push(format!("legenda pintada: «{}»", m.texto));
                    }
                    if !m.coube() {
                        fora.push(format!("face cortada: «{}» -> «{}»", m.texto, m.pintado));
                    }
                }
            }
        }
        fora
    });
    crate::widget::lista_flutuante::force_cromo_on_this_thread(None);
    assert!(acusados.is_empty(), "{}", acusados.join("\n"));
}
