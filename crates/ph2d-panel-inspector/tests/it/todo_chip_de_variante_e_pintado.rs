//! ⭐⭐⭐ **TODO CHIP DE VARIANTE E PINTADO, mesmo quando a fileira QUEBRA.**
//!
//! ⛔⛔⛔ **Ele existe porque uma mutacao SOBREVIVEU.** A fileira de chips de um eixo passou a
//! refluir em 2026-09-19 (ela colapsava a `0,0 px` na largura em que o dono trabalha), e apagar a
//! SEGUNDA fileira do refluxo nao reprovava nada: a varredura de elisoes mede **o que foi
//! pintado**, logo um chip que deixa de ser pintado sai do censo em vez de aparecer nele.
//! *Um censo que varre menos fica verde* — a forma exacta que esta casa ja pagou com o
//! censo por prefixo de nome.
//!
//! ⚠️ **A pergunta aqui e de POPULACAO**, e nao de largura: *todas as opcoes que a seccao
//! declara chegam ao ecra?* — e a metade negativa e a que a torna honesta (numa coluna
//! larga a fileira nao quebra, e as opcoes continuam todas la).

use ph2d_editor_core::screens::hero::{InspectorPropertiesInfo, VariantAxis, VariantChoice};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_properties};
use ph2d_ui_testkit::MockPanelHost;

const RAIZ: u64 = 0xA11;

/// ⚠️ Rotulos COMPRIDOS de proposito: o que se mede e a fileira a QUEBRAR, e tres palavras
/// curtas cabem em qualquer largura.
fn info() -> InspectorPropertiesInfo {
    InspectorPropertiesInfo {
        entity_bits: 0xB0B,
        root_bits: RAIZ,
        rows: vec![VariantAxis {
            name: "material".to_string(),
            options: vec![
                VariantChoice {
                    master: RAIZ,
                    label: "brushed steel".to_string(),
                    current: true,
                },
                VariantChoice {
                    master: RAIZ + 1,
                    label: "oxidised copper".to_string(),
                    current: false,
                },
                VariantChoice {
                    master: RAIZ + 2,
                    label: "matte plastic".to_string(),
                    current: false,
                },
            ],
        }],
        beyond: 0,
        source_name: Some("Hero".to_string()),
    }
}

fn pintados(largura: f32) -> Vec<ph2d_a11y::NodeId> {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_properties(Some(info()));
    let mut st = InspectorState::default();
    let rects = h.paint::<InspectorPanel>(
        &mut st,
        Rect {
            x: 0.0,
            y: 0.0,
            w: largura,
            h: 1400.0,
        },
    );
    set_current_inspector_properties(None);
    let esperados = &ids::INSP_INSTANCE_AXIS_OPTION[0][..3];
    rects
        .iter()
        .map(|(n, _)| *n)
        .filter(|n| esperados.contains(n))
        .collect()
}

/// ⭐⭐⭐ **As tres opcoes chegam ao ecra na largura em que o dono trabalha.**
///
/// ⛔ A metade que interessa e esta: e o degrau estreito que faz a fileira quebrar, e era
/// nele que a segunda metade dela podia desaparecer em silencio.
#[test]
fn todo_chip_de_variante_e_pintado_mesmo_quando_a_fileira_quebra() {
    // A coluna docada no minimo — o `PANEL_MIN_W_PX`, que e o piso ate onde a borda encolhe.
    let estreito = pintados(ph2d_tokens::PANEL_MIN_W_PX);
    assert_eq!(
        estreito.len(),
        3,
        "a coluna estreita pintou {} de 3 chips — a fileira quebrou e a parte de baixo dela \
         sumiu: {estreito:?}",
        estreito.len()
    );
}

/// ⭐ **O CONTROLO: numa coluna larga elas tambem la estao.**
///
/// ⚠️ Sem ele, um painel que deixasse de pintar a seccao INTEIRA leria `0` nos dois casos e
/// so a metade de cima acusaria — *uma regua que so mede o caso dificil nao sabe se o facil
/// ainda acontece*.
#[test]
fn e_numa_coluna_larga_tambem() {
    let largo = pintados(640.0);
    assert_eq!(
        largo.len(),
        3,
        "a coluna larga pintou {} de 3 chips: {largo:?}",
        largo.len()
    );
}
