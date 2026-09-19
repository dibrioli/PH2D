//! ⛔⛔⛔ **SEIS DOS SETE SELOS DESTA LISTA SAÍAM CORTADOS, e a varredura de elisões do app NÃO
//! PODIA vê-los.**
//!
//! Medido em 2026-09-19. A caixa do selo era `ICON_BTN_SIZE_PX` (`36`) — o slot do cacho de ícones
//! da direita —, e uma pílula LISA entrega ao rótulo o respiro de uma caixa de rótulo, que a `36`
//! deixa **`20,00 px`**:
//!
//! | selo | mede | orçamento antigo | saía |
//! |---|---:|---:|---|
//! | `CAM` | `25,66` | `20,00` | `CA…` |
//! | `GRP` | `22,41` | `20,00` | `GR…` |
//! | `ENT` | `22,13` | `20,00` | `EN…` |
//! | `LNK` | `22,10` | `20,00` | `LN…` |
//! | `SPR` | `21,29` | `20,00` | `SP…` |
//! | `PRF` | `20,67` | `20,00` | `PR…` |
//! | `ISO` | `18,54` | `20,00` | **cabia** |
//!
//! # ⚠️⚠️ Porque a varredura do app é CEGA a isto
//!
//! O [`nenhum_rotulo_do_app_pinta_nada`](../../../ph2d-panel-registry-init/tests/it/) pinta **cada
//! painel do registo com o estado de FÁBRICA**, e uma Hierarquia de fábrica tem uma linha (*Scene
//! Root*) sem selo nenhum. *Um censo que varre painéis vazios mede o painel vazio* — quem mede o
//! selo é quem entrega uma CENA, e é isso que este ficheiro faz.
//!
//! # ⭐ E a cura não é um número maior: é a pílula a medir a palavra
//!
//! [`Tag::natural_width`] inverte a lei que o pintor gasta, e o `ICON_BTN_SIZE_PX` fica sendo o
//! **piso** — o selo é uma das ranhuras daquele cacho, logo nunca encolhe abaixo dela e só CRESCE
//! quando a palavra pede. ⛔ Sem o piso, `ISO` passaria a ser mais estreito que os irmãos e a
//! coluna da direita ficaria irregular sem ninguém ter pedido.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::text_elide::elisao::Medido;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_hierarchy::HierarchyPanel;
use ph2d_panel_hierarchy::state::HierarchyState;
use ph2d_ui_testkit::MockPanelHost;
use std::collections::BTreeMap;

/// Os ids de uma cena viva nascem no `BASE_NODE_ID` do shell.
const FIRST_ROW: u64 = 100_000;

/// ⚠️ **Escritos à mão de propósito** — eles são o CONTROLO do vocabulário que o `badge_tone`
/// casa. Derivá-los da tabela de tons mediria a tabela contra si própria.
const SELOS: &[&str] = &[
    "ENT", "SPR", "GRP", "CAM", "LNK", "ISO", "PRF", "SUB", "EXC",
];

fn linha(nome: &str, selo: &str) -> HierarchyEntity {
    HierarchyEntity {
        name: nome.into(),
        icon: IconId::Sprite,
        indent: 0,
        badge: Some(selo.into()),
        swatch: None,
        visible: true,
        selected: false,
        hovered: false,
        muted: false,
        locked: false,
        group_locked: false,
    }
}

/// Pinta uma cena com um selo por linha e devolve o censo de elisões.
fn selos_pintados() -> Vec<Medido> {
    let ordered: Vec<ph2d_a11y::NodeId> = (0..SELOS.len())
        .map(|i| ph2d_a11y::NodeId(FIRST_ROW + i as u64))
        .collect();
    let entries: BTreeMap<_, _> = ordered
        .iter()
        .zip(SELOS)
        .map(|(id, s)| (*id, linha(&format!("Object {s}"), s)))
        .collect();
    ph2d_panel_hierarchy::state::set_live_entries(Some(entries));
    let mut host = MockPanelHost::with_panel::<HierarchyPanel>();
    host.set_hierarchy_rows(&ordered);
    let mut state = HierarchyState::default();
    let medidos =
        host.medindo_a_pintura::<HierarchyPanel>(&mut state, Rect::new(0.0, 0.0, 1366.0, 1024.0));
    ph2d_panel_hierarchy::clear_live_hierarchy();
    medidos
}

/// ⭐⭐⭐ **TODO selo desta lista é pintado INTEIRO.**
#[test]
fn todo_selo_da_hierarquia_e_pintado_inteiro() {
    let medidos = selos_pintados();
    // ⛔ **PISO DE POPULAÇÃO.** Um `for` sobre uma lista vazia passa trivialmente, e é assim que
    //    um censo cujo corpus encolheu se lê como aprovado.
    assert!(
        SELOS.len() >= 7 && medidos.len() >= SELOS.len(),
        "a fixtura tem {} selo(s) e o censo leu {} rótulo(s) — com menos, este gate mede o vazio",
        SELOS.len(),
        medidos.len()
    );
    for selo in SELOS {
        let Some(m) = medidos.iter().find(|m| m.texto == *selo) else {
            panic!(
                "⛔ o selo {selo:?} não foi pintado — sem ele este gate mede o vazio. \
                 Vistos: {:?}",
                medidos.iter().map(|m| &m.texto).collect::<Vec<_>>()
            );
        };
        assert!(
            m.coube(),
            "⛔ o selo {selo:?} saiu {:?} num orçamento de {:.2} px — a caixa dele deixou de \
             descrever a palavra que ele carrega",
            m.pintado,
            m.largura
        );
    }
}

/// ⛔ **E a caixa nunca encolhe abaixo da ranhura do cacho.**
///
/// Sem esta metade, `ISO` (`18,54`) ficaria com uma pílula mais estreita que as irmãs e a coluna
/// da direita passaria a dançar de linha para linha. *A largura natural é o PISO da palavra; o
/// slot é o piso da COLUNA.*
#[test]
fn o_selo_nunca_fica_mais_estreito_que_a_ranhura_do_cacho() {
    let medidos = selos_pintados();
    let m = medidos
        .iter()
        .find(|m| m.texto == "ISO")
        .expect("o selo mais estreito tem de ser pintado");
    // O orçamento de uma pílula lisa de `ICON_BTN_SIZE_PX` — a lei da casa, pela porta.
    let piso = ph2d_editor_core::paint::label_budget(ph2d_tokens::ICON_BTN_SIZE_PX);
    assert!(
        m.largura >= piso - 0.01,
        "o selo mais estreito recebeu {:.2} px e a ranhura do cacho dá {piso:.2} — a caixa \
         encolheu abaixo do slot em que ela vive",
        m.largura
    );
}
