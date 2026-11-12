//! **O CATÁLOGO real da paleta de comandos global** — o gate que só esta crate consegue correr.
//!
//! Os sete gates da `ph2d-editor-core` provam a LEI da paleta (a projeção, a execução, o que fica
//! de fora e porquê). Nenhum deles vê o **catálogo**: o registry de painéis vive nesta crate, que
//! DEPENDE da `editor-core`, então lá o grupo dos painéis nasce vazio e os gates passam sobre uma
//! lista de zero. É a mesma razão pela qual o censo do scrub mora aqui — *esta é a crate mais barata
//! que enxerga todos os painéis*.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::screens::hero::global_palette::{build_global_model, route_global_pick};

fn hero() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    HeroScreen::new(NodeId(1))
}

/// ⭐ **TODO painel registado é alcançável pela paleta, e escolhê-lo ALTERNA a visibilidade dele.**
///
/// É o comando que mais falta no app: um painel sem chip no rail e sem pill na barra (o editor de
/// áudio, o Tuning da aquarela, os params do Motion) só é alcançável por quem sabe a tecla.
///
/// *Mutação que sangra:* o `build_global_model` filtrar o registry por qualquer critério, ou o
/// `route_global_pick` deixar de reconhecer um `panel_node_id`.
#[test]
fn every_registered_panel_is_a_command_that_toggles_it() {
    let mut h = hero();
    let model = build_global_model(&h);
    let items: Vec<(String, NodeId)> = model
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .map(|it| (it.label.clone(), it.id))
        .collect();

    let mut panels: Vec<(&'static str, NodeId)> = Vec::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            panels.push((p.manifest.id, p.manifest.panel_node_id));
        }
    });
    // ⚠️ O número é MEDIDO, não escolhido: a árvore tem 23 crates de painel e o
    // `register_all_panels` deste build regista **19** — quatro ficam atrás de features. Um bar
    // acima disso reprovava produto correcto, que é como a primeira versão deste gate nasceu.
    assert!(
        panels.len() >= 15,
        "o registry tem os painéis do app; contou {}",
        panels.len()
    );
    println!("[palette-catalog] {} painéis registados", panels.len());

    for (pid, node) in &panels {
        assert!(
            items.iter().any(|(_, id)| id == node),
            "o painel {pid:?} está REGISTADO e não é alcançável pela paleta"
        );
        let before = h.is_panel_visible(pid);
        assert!(
            route_global_pick(&mut h, *node),
            "a rota não reclamou o painel {pid:?}"
        );
        assert_eq!(
            h.is_panel_visible(pid),
            !before,
            "escolher {pid:?} tem de ALTERNAR a visibilidade dele"
        );
        // Devolve o estado, para o painel seguinte partir do mesmo mundo.
        let _ = route_global_pick(&mut h, *node);
    }
}

/// ⭐ **O nome de um painel na busca é o da ABA** (dono 12/11): «Wet Paint», não «Wet Tuning».
/// Mutação: voltar a derivar o rótulo do id ⇒ «Pixel Prefs» ≠ «Grid & Background», sangra.
#[test]
fn a_panel_is_called_in_the_search_what_its_tab_says() {
    let h = hero();
    let model = build_global_model(&h);
    let labels: Vec<(NodeId, String)> = model
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .map(|it| (it.id, it.label.clone()))
        .collect();
    let mut seen = 0;
    ph2d_editor_core::panel::with_registry_opt(|reg| {
        for p in reg.panels() {
            let Some((_, label)) = labels
                .iter()
                .find(|(id, _)| *id == p.manifest.panel_node_id)
            else {
                continue;
            };
            seen += 1;
            assert_eq!(
                label,
                p.manifest.title.tr(),
                "o painel {:?} chama-se na busca outra coisa que a aba",
                p.manifest.id
            );
        }
    });
    assert!(seen >= 10, "controlo positivo: só {seen} painéis na busca");
    for tab in ["Wet Paint", "Grid & Background", "Keyboard Shortcuts"] {
        assert!(
            labels.iter().any(|(_, l)| l == tab),
            "{tab:?} devia estar na busca"
        );
    }
}
