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
/// registados: um painel que consumisse o id calaria o clique, como já aconteceu com uma paleta
/// aberta por cima de um painel (`a_paleta_fecha_ao_escolher_por_cima_de_um_painel`, na
/// `ph2d-panel-registry-init`).
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

    let mut others: Vec<(String, NodeId)> = Vec::new();
    for e in ph2d_app_components::test_support::registo().iter() {
        others.push((
            e.canonical_name.to_string(),
            ph2d_app_components::component_palette::item_id(e.canonical_name),
        ));
    }
    let models = [
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
    let wanted = [
        ph2d_editor_core::object_add::EMPTY,
        ph2d_editor_core::object_add::IMAGE,
        ph2d_app_vec::object_add::VECTOR,
        ph2d_app_text::TEXT,
        ph2d_app_flip::object_add::FLIP,
        ph2d_app_components::object_add::CAMERA,
        ph2d_app_light2d::object_add::LUZ,
        ph2d_app_light2d::object_add::LUZ_AMBIENTE,
    ];
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

/// ⭐⭐ **Criar pelo menu e desfazer devolve o projecto ao BIT** — o vazio, um objecto de jogo e um
/// desenho Flip (o desenho vetorial não cria nada no menu: nasce no 1.º traço, spec/06 F3), pela porta de cada família e pelo `ProjectState` que o
/// `post_frame_undo` usa (o mundo, a cena vetorial e o documento Flip juntos).
///
/// (Mutação: o `restore` devolver uma `VecScene` vazia ⇒ a forma que já existia some ⇒ RED.)
#[test]
fn creating_then_undoing_returns_the_project_to_the_bit() {
    let reg = ph2d_app_components::test_support::registo();
    for which in ["empty", "camera", "flip", "luz"] {
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
            "luz" => Ok(ph2d_app_light2d::object_add::add(
                ph2d_app_light2d::object_add::LUZ,
                &mut sim,
            )
            .expect("é da luz")),
            _ => ph2d_app_flip::object_add::add(
                ph2d_app_flip::object_add::FLIP,
                &mut sim,
                &mut flip,
                &mut fstate,
            )
            .expect("é do Flip")
            .map_err(String::from),
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

/// ⭐⭐ **G8 — mexer na Intensidade e desfazer devolve o projecto ao BIT** (Luz 2D, W4): a edição
/// do Inspector pela porta que a shell drena, e o `ProjectState` que o `post_frame_undo` usa.
#[test]
fn editing_a_light_then_undoing_returns_the_project_to_the_bit() {
    use ph2d_editor_core::luz_edits::LuzFieldEdit;
    let reg = ph2d_app_components::test_support::registo();
    let (vec_scene, flip) = (ph2d_vec_scene::VecScene::new(), ph2d_flip::FlipDoc::new());
    let mut sim = ph2d_ecs::SimWorld::new();
    let bits = ph2d_app_light2d::object_add::add(ph2d_app_light2d::object_add::LUZ, &mut sim)
        .expect("é da luz");
    let before = capture(&mut sim, &vec_scene, &flip, &reg);
    let edit = [(bits, LuzFieldEdit::Intensidade(1.5))];
    assert!(ph2d_app_light2d::inspector::apply_all(&mut sim, &edit));
    let after = capture(&mut sim, &vec_scene, &flip, &reg);
    assert_ne!(
        after, before,
        "a edição não chegou ao projecto que o undo vê"
    );
    let (rvec, _vmap, rflip, _fmap) = before.restore(&mut sim, &reg);
    assert_eq!(
        capture(&mut sim, &rvec, &rflip, &reg),
        before,
        "o Ctrl+Z não voltou ao bit"
    );
    let w = sim.world_mut();
    let mut q = w.query::<&ph2d_light2d_ecs::Luz>();
    let luz = *q.single(w).expect("uma luz depois do undo");
    assert_eq!(luz.intensidade, std::f32::consts::TAU);
}

/// O que fica SELECCIONADO depois de criar `entry` pela porta da família (a que a fase chama) —
/// mais o que o artista selecciona a seguir num modo da família (o osso do Pose). Para a imagem,
/// o pixel sprite e o desenho vetorial: o que o diálogo/1.º traço faz nascer.
fn seleccionados_ao_nascer(entry: ph2d_editor_core::object_add::AddEntry) -> Vec<u64> {
    use ph2d_editor_core::object_add as core;
    let reg = ph2d_app_components::test_support::registo();
    let seeds = ph2d_app_physics::physics_seed::COMPONENT_SEEDS;
    let raiz = ph2d_app_components::object_add::spawn_empty_root;
    let mut sim = ph2d_ecs::SimWorld::new();
    let ok = |r: Result<u64, &str>| r.unwrap_or_else(|e| panic!("{}: {e}", entry.key.key()));
    let mut bits = if entry == core::EMPTY {
        vec![raiz(&mut sim, "Empty")]
    } else if entry == core::IMAGE {
        // O OK do diálogo New Image: `spawn_blank_canvas` → `spawn_sprite` (o atlas é da placa).
        let src = ph2d_image_import::PackedSource::Atlas { cell_idx: 0 };
        let c = ph2d_core::Vec2::new(0.0, 0.0);
        vec![ph2d_image_import::spawn_sprite(&mut sim, src, c, [1.0, 1.0], "Image").1]
    } else if entry == ph2d_app_vec::object_add::VECTOR || entry == ph2d_app_text::TEXT {
        let mut vec = ph2d_app_vec::state::VecState::default();
        let text = ph2d_app_text::TextAdd::default().add(entry, &sim, None, [0.0; 2], || ());
        assert!(ph2d_app_vec::object_add::add(entry, &mut vec) || text);
        // O 1.º traço (ou a palavra do texto): uma forma na cena; a ponte de sempre dá-lhe o objecto.
        let mut scene = ph2d_vec_scene::VecScene::new();
        let id = scene.push_path(ph2d_vec_scene::rectangle([0.0, 0.0], [2.0, 2.0]));
        ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut vec.entities);
        vec![*vec.entities.get(&id).expect("a forma tem objecto")]
    } else if entry == ph2d_app_pixel::object_add::PIXEL_SPRITE {
        let mut app = ph2d_app_pixel::PixelApp::default();
        let mut store = ph2d_editor_core::interaction::WidgetStore::default();
        assert!(ph2d_app_pixel::object_add::open_dialog(entry, &mut store));
        let r = ph2d_app_pixel::object_add::add(entry, &mut sim, &mut app, raiz);
        vec![ok(r.expect("é do pixel"))]
    } else if entry == ph2d_app_flip::object_add::FLIP {
        let (mut doc, mut st) = Default::default();
        let r = ph2d_app_flip::object_add::add(entry, &mut sim, &mut doc, &mut st);
        vec![ok(r.expect("é do Flip"))]
    } else if entry == ph2d_app_skeleton::object_add::SKELETON {
        let mut st = ph2d_app_skeleton::state::SkeletonState::default();
        let r = ph2d_app_skeleton::object_add::add(entry, &mut sim, &mut st);
        vec![ok(r.expect("é do esqueleto"))]
    } else if entry == ph2d_app_motion::motion_objeto::object_add::MOTION {
        let r = ph2d_app_motion::motion_objeto::object_add::add(entry, &mut sim, raiz);
        vec![ok(r.expect("é do Motion"))]
    } else if entry == ph2d_app_macros::object_add::MACRO {
        let r = ph2d_app_macros::object_add::add(entry, &mut sim, raiz);
        vec![ok(r.expect("é da Macro"))]
    } else if let Some(b) = ph2d_app_light2d::object_add::add(entry, &mut sim) {
        vec![b]
    } else {
        let anexa = |s: &mut ph2d_ecs::SimWorld, b, n: &str| {
            ph2d_app_components::component_attach::attach_by_name(s, &reg, seeds, b, n)
        };
        let portas = ph2d_app_game::object_add::Portas {
            raiz,
            anexa: &anexa,
        };
        let r = ph2d_app_game::object_add::add(entry, &mut sim, portas)
            .or_else(|| ph2d_app_components::object_add::add(entry, &mut sim, &reg, seeds))
            .unwrap_or_else(|| panic!("{}: tipo do Add sem linha neste gate", entry.key.key()));
        vec![r.unwrap_or_else(|e| panic!("{}: {e}", entry.key.key()))]
    };
    // O Pose selecciona o OSSO: ele também é um objecto do Inspector.
    let w = sim.world_mut();
    let mut q = w.query_filtered::<ph2d_ecs::Entity, ph2d_ecs::With<ph2d_skeleton_ecs::Bone>>();
    bits.extend(q.iter(w).map(ph2d_ecs::Entity::to_bits));
    bits.into_iter()
        .map(|b| assert_ordering(&mut sim, &reg, entry, b))
        .collect()
}

/// A §7 existe para `b` e o `Z Index` que o artista escreve CHEGA.
fn assert_ordering(
    sim: &mut ph2d_ecs::SimWorld,
    reg: &ph2d_ecs::scene::ComponentRegistry,
    entry: ph2d_editor_core::object_add::AddEntry,
    b: u64,
) -> u64 {
    use crate::render_loop::inspector_presence_probe::ordering_z_index;
    let k = entry.key.key();
    // `None` = `—` (vem da árvore); a Background Layer nasce com o seu −1 autorado (atrás).
    let z = ordering_z_index(sim.world(), b);
    assert!(
        z.is_some(),
        "{k}: o Inspector deste tipo NÃO tem a secção Ordering"
    );
    let q = ph2d_ecs::scene::EditorCommandQueue::new();
    let edit = ph2d_editor_core::OrderingFieldEdit::ZIndex(Some(7));
    ph2d_inspector_ordering::apply_ordering_edit(sim, b, edit, &q, reg);
    ph2d_ecs::scene::apply_editor_commands(sim.world_mut(), &q, reg).expect("aplica");
    assert_eq!(
        ordering_z_index(sim.world(), b),
        Some(Some(7)),
        "{k}: o Z Index escrito no Inspector não chegou ao objecto"
    );
    b
}

/// ⭐⭐ **Todo tipo do Add tem o Z Index no Inspector** (dono, 01/11; plano 19 E5) — cada entrada
/// compilada criada pela porta da SUA família, a entidade que fica seleccionada (e o osso do
/// Pose), a §7 presente e o `Z Index` escrito a chegar. ⚠️ Os modos (Draw/Edit/Pose)
/// não escondem a §7: o snapshot lê só `hero.gizmo.selection`, e o modo exige-a = o objecto
/// (`ObjectModeState::still_holds`).
///
/// (Mutação, 01/11: o `build_ordering_info` devolver `None` a quem tem `Macro` ⇒ RED.)
#[test]
fn a_seccao_ordering_existe_em_todo_tipo_do_add() {
    let mut vistos = 0;
    for entry in FAMILIES.iter().flat_map(|f| f.iter()) {
        let sel = seleccionados_ao_nascer(*entry);
        assert!(!sel.is_empty(), "{}: nada seleccionado", entry.key.key());
        vistos += 1;
    }
    // Controlo positivo: a varredura viu o menu inteiro (20 tipos em 01/11).
    assert!(vistos >= 20, "só {vistos} tipos do Add varridos");
}

/// `Add ▸ Sprite` abre a escolha da imagem daquele objecto; os outros objectos do jogo, não.
#[test]
fn so_a_sprite_nasce_a_pedir_a_imagem() {
    use super::pede_a_imagem;
    use ph2d_editor_core::action_bus::{ActionBus, EditorAction};
    let mut bus = ActionBus::default();
    pede_a_imagem(ph2d_app_game::object_add::SPRITE, 7, &mut bus);
    assert!(matches!(
        bus.drain().collect::<Vec<_>>().as_slice(),
        [EditorAction::OpenImagePicker { entity_bits: 7 }]
    ));
    pede_a_imagem(ph2d_app_game::object_add::ANIMATED_SPRITE, 7, &mut bus);
    assert_eq!(bus.drain().count(), 0);
}
