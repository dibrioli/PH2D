use super::FAMILIES;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::object_add::{build, entry_of_pick};
use ph2d_editor_core::{HeroScreen, NodeId};

fn all_ids() -> Vec<NodeId> {
    FAMILIES
        .iter()
        .flat_map(|f| f.iter())
        .map(|e| e.id())
        .collect()
}

/// ⭐⭐ **O clique REAL num item do menu deixa o pick, e o dreno do menu reconhece-o** — por cada
/// entrada compilada, pela porta do quadro (`HeroScreen::apply_event`), com os painéis
/// registados: um painel que consumisse o id calaria o clique, como já aconteceu com a paleta de
/// pincéis (`a_paleta_de_pinceis_fecha_ao_escolher`).
///
/// (Mutação: o dreno da fase perguntar `entry_of_pick(&FAMILIES[..1], …)` ⇒ RED.)
#[test]
fn a_real_click_on_every_entry_reaches_the_drain() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    for entry in FAMILIES.iter().flat_map(|f| f.iter()) {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store.open_command_palette(build(FAMILIES, &|_| None));
        hero.apply_event(WidgetEvent::Click(entry.id()));
        assert!(
            !hero.store.command_palette_open(),
            "{}: o clique não fechou o menu",
            entry.key.key()
        );
        let picked = hero
            .store
            .take_command_pick_if(|id| entry_of_pick(FAMILIES, id).is_some())
            .and_then(|id| entry_of_pick(FAMILIES, id));
        assert_eq!(
            picked,
            Some(*entry),
            "{}: o pick perdeu-se",
            entry.key.key()
        );
    }
}

/// ⭐⭐ **Nenhum id do menu é de OUTRO consumidor do canal de pick** — o dreno de cada um pergunta
/// *«este id é meu?»*, e um id partilhado faria dois menus executarem o mesmo clique.
///
/// (Mutação: dar a uma entrada a chave de uma forma do *Add shape…* ⇒ RED.)
#[test]
fn no_menu_id_belongs_to_another_palette() {
    let ids = all_ids();
    let mut unique = ids.clone();
    unique.sort_by_key(|id| id.0);
    unique.dedup();
    assert_eq!(
        unique.len(),
        ids.len(),
        "duas entradas do menu com o mesmo id"
    );

    let mut others: Vec<(String, NodeId)> = ph2d_app_field3d::shapes::SHAPES
        .iter()
        .map(|s| {
            (
                s.key.to_string(),
                ph2d_app_field3d::shape_palette::item_id(s.key),
            )
        })
        .collect();
    for e in ph2d_app_components::test_support::registo().iter() {
        others.push((
            e.canonical_name.to_string(),
            ph2d_app_components::component_palette::item_id(e.canonical_name),
        ));
    }
    let models = [
        ph2d_panel_sculpt3d::brush_palette::build(),
        ph2d_editor_core::screens::hero::global_palette::build_global_model(&HeroScreen::new(
            NodeId(1),
        )),
    ];
    for m in &models {
        for item in m.groups.iter().flat_map(|g| &g.subs).flat_map(|s| &s.items) {
            others.push((item.label.clone(), item.id));
        }
    }
    for (name, id) in others {
        assert!(
            !ids.contains(&id),
            "o id de «{name}» é também um item do menu Add"
        );
    }
}

/// **Cada família compilada está no menu**, e o vazio e a imagem também — o menu é a lista de
/// tipos da engine (pedido do dono, spec/06).
#[test]
fn the_menu_offers_every_compiled_family() {
    let ids = all_ids();
    let mut wanted = vec![
        ph2d_editor_core::object_add::EMPTY,
        ph2d_editor_core::object_add::IMAGE,
        ph2d_app_vec::object_add::RECTANGLE,
        ph2d_app_flip::object_add::FLIP,
        ph2d_app_field3d::object_add::MODEL,
        ph2d_app_components::object_add::CAMERA,
    ];
    #[cfg(feature = "sculpt3d")]
    wanted.push(ph2d_app_sculpt3d::object_add::SPHERE);
    for e in wanted {
        assert!(ids.contains(&e.id()), "{} fora do menu", e.key.key());
    }
}
