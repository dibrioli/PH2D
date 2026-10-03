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
/// ⚠️ O que ele prende é a ROTA do clique até ao pick, para cada id compilado; o dreno da fase
/// pergunta pelo mesmo `entry_of_pick(FAMILIES, …)`, e a criação de cada família tem gate na
/// crate dela.
///
/// (Mutação: o `apply` da paleta deixar de chamar `set_command_pick` ⇒ RED.)
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

fn capture(
    sim: &mut ph2d_ecs::SimWorld,
    vec: &ph2d_vec_scene::VecScene,
    flip: &ph2d_flip::FlipDoc,
    reg: &ph2d_ecs::scene::ComponentRegistry,
) -> crate::undo::ProjectState {
    crate::undo::ProjectState::capture(
        &ph2d_preview_drive::PreviewDrive::default(),
        sim,
        vec,
        flip,
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        &[],
        reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        None,
    )
}

/// ⭐⭐ **Criar pelo menu e desfazer devolve o projecto ao BIT** — o vazio, um objecto de jogo, uma
/// forma vetorial, um desenho Flip e o Model, pela porta de cada família e pelo `ProjectState`
/// que o `post_frame_undo` usa (o mundo, a cena vetorial e o documento Flip juntos).
///
/// ⚠️ A escultura fica de fora: a cena dela vive na placa e tem undo próprio (`StrokeUndo`).
///
/// (Mutação: o `restore` devolver uma `VecScene` vazia ⇒ a forma que já existia some ⇒ RED.)
#[test]
fn creating_then_undoing_returns_the_project_to_the_bit() {
    let reg = ph2d_app_components::test_support::registo();
    for which in ["empty", "camera", "vector", "flip", "model"] {
        let mut sim = ph2d_ecs::SimWorld::new();
        let mut vec_scene = ph2d_vec_scene::VecScene::new();
        let mut flip = ph2d_flip::FlipDoc::new();
        let mut vec = ph2d_app_vec::state::VecState::default();
        let mut fstate = ph2d_app_flip::state::FlipState::default();
        // ⚠️ **O projecto NÃO começa vazio:** um undo que apagasse tudo passaria sobre um vazio.
        ph2d_app_components::object_add::spawn_empty_root(&mut sim, "Pre");
        vec_scene.push_path(ph2d_vec_scene::rectangle([0.0, 0.0], [2.0, 2.0]));
        ph2d_vec_entities::entities::sync(&mut sim, &mut vec_scene, &mut vec.entities);
        let before = capture(&mut sim, &vec_scene, &flip, &reg);
        let born = match which {
            "empty" => Ok(ph2d_app_components::object_add::spawn_empty_root(
                &mut sim, "Empty",
            )),
            "camera" => ph2d_app_components::object_add::add(
                ph2d_app_components::object_add::CAMERA,
                &mut sim,
                &reg,
                &[],
            )
            .expect("é de jogo"),
            "vector" => ph2d_app_vec::object_add::add(
                ph2d_app_vec::object_add::STAR,
                &mut sim,
                &mut vec_scene,
                &mut vec,
                [0.0, 0.0],
                1.0,
                800.0,
            )
            .expect("é do vetor")
            .map_err(String::from),
            "flip" => ph2d_app_flip::object_add::add(
                ph2d_app_flip::object_add::FLIP,
                &mut sim,
                &mut flip,
                &mut fstate,
            )
            .expect("é do Flip")
            .map_err(String::from),
            _ => Ok(ph2d_app_field3d::object_add::add(&mut sim)),
        };
        born.unwrap_or_else(|e| panic!("{which}: {e}"));
        let after = capture(&mut sim, &vec_scene, &flip, &reg);
        assert_ne!(
            after, before,
            "{which}: criar não mudou o projecto que o undo vê"
        );
        let (rvec, _vmap, rflip, _fmap) = before.restore(&mut sim, &reg);
        assert_eq!(
            capture(&mut sim, &rvec, &rflip, &reg),
            before,
            "{which}: o Ctrl+Z não devolveu o projecto ao bit"
        );
    }
}
