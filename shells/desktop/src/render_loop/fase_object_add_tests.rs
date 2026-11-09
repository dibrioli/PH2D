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

/// ⭐ **O destaque REAL ao abrir é a Sprite**, e a faixa da dica tem o que mostrar (dono 07/11).
#[test]
fn ao_abrir_a_sprite_vem_destacada() {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.open_command_palette(super::add_model(&[]));
    let sprite = ph2d_app_game::object_add::SPRITE.id();
    assert_eq!(hero.store.command_palette_highlight(), Some(sprite));
    assert_eq!(
        hero.store.command_palette_enter_target(),
        Some(sprite),
        "Enter cria a Sprite"
    );
}

/// ⭐⭐ **As 19 entradas nos quatro grupos da maquete B, na ordem dela** (Crítica `#addobject`, dono 07/11)
/// — sem Physics Body nem Ambient Light, que saíram.
///
/// (Mutação: ordenar pela ordem das famílias em vez de `AddEntry::order` ⇒ RED.)
#[test]
fn as_dezanove_entradas_nos_quatro_grupos_da_maquete() {
    let model = build(FAMILIES, &|_| None, None);
    let got: Vec<(String, Vec<String>)> = model
        .groups
        .iter()
        .map(|g| {
            let items = g
                .subs
                .iter()
                .flat_map(|s| &s.items)
                .map(|i| i.label.clone())
                .collect();
            (g.title.clone(), items)
        })
        .collect();
    let want: &[(&str, &[&str])] = &[
        (
            "Pictures",
            &[
                "Paint a new image\u{2026}",
                "Sprite",
                "Animated Sprite",
                "Pixel Sprite",
                "Background Layer",
            ],
        ),
        (
            "Drawing",
            &[
                "Vector Drawing",
                "Text",
                "Flip Drawing",
                "Skeleton",
                "Motion",
            ],
        ),
        (
            "Game",
            &[
                "Character",
                "Static Body",
                "Collision Shape",
                "Camera",
                "Sound",
                "HUD",
                "Light",
            ],
        ),
        ("Empty", &["Empty", "Macro"]),
    ];
    let want: Vec<(String, Vec<String>)> = want
        .iter()
        .map(|(g, i)| {
            (
                (*g).to_string(),
                i.iter().map(|s| (*s).to_string()).collect(),
            )
        })
        .collect();
    assert_eq!(got, want);
    assert_eq!(model.item_count(), 19);
}

/// ⭐⭐ **Toda entrada tem dica com desenho** (decisão do dono de 25/10: nome simples + explicação com
/// ilustração), e o texto existe na tabela.
///
/// (Mutação: tirar a dica de uma entrada ⇒ RED.)
#[test]
fn toda_entrada_tem_dica_com_desenho() {
    for e in FAMILIES.iter().flat_map(|f| f.iter()) {
        let tip = e.tip.unwrap_or_else(|| panic!("{} sem dica", e.key.key()));
        assert!(!tip.picture.is_empty(), "{} sem desenho", e.key.key());
        assert_ne!(
            tip.body.tr(),
            tip.body.key(),
            "{}: dica sem texto",
            e.key.key()
        );
    }
}

/// ⭐⭐ **A busca acha pela palavra de USO** (Crítica: «player» e «wall» não achavam nada).
///
/// (Mutação: `keywords` vazio no `build` ⇒ RED.)
#[test]
fn a_busca_acha_pela_palavra_de_uso() {
    use ph2d_app_components::object_add::{HUD, SOUND};
    use ph2d_app_game::object_add::{BACKGROUND_LAYER, CHARACTER, STATIC_BODY};
    use ph2d_editor_core::widget::command_palette::top_match;
    let model = build(FAMILIES, &|_| None, None);
    for (q, e) in [
        ("player", CHARACTER),
        ("hero", CHARACTER),
        ("wall", STATIC_BODY),
        ("floor", STATIC_BODY),
        ("platform", STATIC_BODY),
        ("music", SOUND),
        ("score", HUD),
        ("sky", BACKGROUND_LAYER),
        ("lamp", ph2d_app_light2d::object_add::LUZ),
        ("canvas", ph2d_editor_core::object_add::IMAGE),
        ("pixel art", ph2d_app_pixel::object_add::PIXEL_SPRITE),
        ("bones", ph2d_app_skeleton::object_add::SKELETON),
    ] {
        assert_eq!(top_match(&model, q), Some(e.id()), "«{q}»");
    }
}

/// ⭐⭐ **A busca que não acha ENSINA** — cada palavra de ensino não acha nenhum objecto (senão a frase
/// nunca se veria), e cada frase só cita componentes que o Add Component oferece, pelo nome que ele mostra.
///
/// (Mutação: uma palavra de ensino que um objecto acha, ou «Weapon Fire» na frase ⇒ RED.)
#[test]
fn cada_ensino_so_cita_o_que_existe_e_so_aparece_sem_resultado() {
    use ph2d_editor_core::widget::command_palette::{model_item_matches, teach_for};
    let model = super::add_model(&[]);
    let names: Vec<&str> = ph2d_component_desc::all()
        .map(|d| ph2d_i18n::tr(d.display_key))
        .collect();
    let cited: &[(&str, &[&str])] = &[
        ("coin", &["Collision Shape", "Counter", "Signal Actions"]),
        ("enemy", &["Physics Body", "Path Follow", "Damage"]),
        ("door", &["Collision Shape", "Signal Actions"]),
        ("trap", &["Damage"]),
        ("checkpoint", &["Collision Shape", "Signal Actions"]),
        ("bullet", &["Projectile Motion", "Lifetime", "Weapon"]),
        ("moving platform", &["Path Follow"]),
        ("health", &["Health", "Health Bar"]),
    ];
    assert_eq!(cited.len(), ph2d_app_game::object_add::TEACH.len());
    for (q, comps) in cited {
        let t = teach_for(&model, q).unwrap_or_else(|| panic!("«{q}» não ensina"));
        for c in *comps {
            assert!(names.contains(c), "«{c}» não é um componente do catálogo");
            assert!(t.text.contains(c), "a frase de «{q}» não cita {c}");
        }
    }
    let items: Vec<_> = model
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .collect();
    for t in &model.select.as_ref().expect("o menu destaca").teach {
        for w in t.words.split(',').map(str::trim) {
            let hit = items.iter().find(|i| model_item_matches(&model, w, i));
            assert!(
                hit.is_none(),
                "«{w}» acha «{}»: o ensino nunca se veria",
                hit.unwrap().label
            );
        }
    }
}

/// ⭐ **O Recent: os criados por último no topo, sem repetir, três no máximo** — e volta igual do ficheiro
/// de preferências.
///
/// (Mutação: `truncate` a 4 ⇒ RED.)
#[test]
fn o_recent_guarda_tres_sem_repetir_e_volta_do_ficheiro() {
    use ph2d_app_game::object_add::{CHARACTER, SPRITE, STATIC_BODY};
    let mut hero = HeroScreen::new(NodeId(1));
    for e in [
        SPRITE,
        STATIC_BODY,
        SPRITE,
        CHARACTER,
        ph2d_editor_core::object_add::EMPTY,
    ] {
        hero.store.push_add_recent(e.id());
    }
    let want = [
        ph2d_editor_core::object_add::EMPTY.id(),
        CHARACTER.id(),
        SPRITE.id(),
    ];
    assert_eq!(hero.store.add_recent(), want);
    let model = super::add_model(hero.store.add_recent());
    let recent: Vec<NodeId> = model.groups[0].subs[0].items.iter().map(|i| i.id).collect();
    assert_eq!(
        (model.groups[0].title.as_str(), recent.as_slice()),
        ("Recent", &want[..])
    );
    let text = ph2d_editor_core::interaction::section_prefs_text(&hero.store);
    let mut other = HeroScreen::new(NodeId(1));
    ph2d_editor_core::interaction::restore_section_prefs_text(&mut other.store, &text);
    assert_eq!(other.store.add_recent(), want);
}

/// ⭐⭐ **O clique REAL em cada item do menu deixa o pick que o dreno reconhece**, com os painéis registados.
/// (Mutação: o `apply` da paleta deixar de chamar `set_command_pick` ⇒ RED.)
#[test]
fn a_real_click_on_every_entry_reaches_the_drain() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    for entry in FAMILIES.iter().flat_map(|f| f.iter()) {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store.open_command_palette(super::add_model(&[]));
        // ⭐ Dono 07/11: um clique DESTACA (a janela fica), dois cliques escolhem e fecham.
        hero.apply_event(WidgetEvent::Click(entry.id()));
        assert!(
            hero.store.command_palette_open(),
            "{}: um clique fechou",
            entry.key.key()
        );
        assert_eq!(hero.store.command_palette_highlight(), Some(entry.id()));
        assert_eq!(
            hero.store.take_command_pick(),
            None,
            "um clique não escolhe"
        );
        hero.apply_event(WidgetEvent::DoubleClick(entry.id()));
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

/// ⭐⭐ **Criar pelo menu e desfazer devolve o projecto ao BIT** — vazio, jogo, Flip, luz e a Collision Shape
/// FILHA do seleccionado (W3), pela porta de cada família e pelo `ProjectState` do `post_frame_undo`.
#[test]
fn creating_then_undoing_returns_the_project_to_the_bit() {
    let reg = ph2d_app_components::test_support::registo();
    for which in ["empty", "camera", "flip", "luz", "forma"] {
        let mut sim = ph2d_ecs::SimWorld::new();
        let mut vec_scene = ph2d_vec_scene::VecScene::new();
        let mut flip = ph2d_flip::FlipDoc::new();
        let mut vec = ph2d_app_vec::state::VecState::default();
        let mut fstate = ph2d_app_flip::state::FlipState::default();
        // ⚠️ **O projecto NÃO começa vazio:** um undo que apagasse tudo passaria sobre um vazio.
        let pre = ph2d_app_components::object_add::spawn_empty_root(&mut sim, "Pre");
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
            "forma" => {
                let anexa = |s: &mut ph2d_ecs::SimWorld, b, n: &str| {
                    ph2d_app_components::component_attach::attach_by_name(s, &reg, &[], b, n)
                };
                let raiz = ph2d_app_components::object_add::spawn_empty_root;
                let portas = ph2d_app_game::object_add::Portas {
                    raiz,
                    anexa: &anexa,
                };
                let shape = ph2d_app_game::object_add::COLLISION_SHAPE;
                ph2d_app_game::object_add::add(shape, &mut sim, portas, Some(pre)).expect("jogo")
            }
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

/// O que fica SELECCIONADO depois de criar `entry` pela porta da família — e o osso do Pose; para a imagem,
/// o pixel sprite e o vetor, o que o diálogo/1.º traço faz nascer.
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
        let r = ph2d_app_game::object_add::add(entry, &mut sim, portas, None)
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

/// ⭐⭐ **Todo tipo do Add tem o Z Index no Inspector** (dono, 01/11; plano 19 E5): pela porta da família,
/// a entidade seleccionada (e o osso do Pose), a §7 e o `Z Index` a chegar; os modos não a escondem.
/// (Mutação, 01/11: o `build_ordering_info` devolver `None` a quem tem `Macro` ⇒ RED.)
#[test]
fn a_seccao_ordering_existe_em_todo_tipo_do_add() {
    let mut vistos = 0;
    for entry in FAMILIES.iter().flat_map(|f| f.iter()) {
        let sel = seleccionados_ao_nascer(*entry);
        assert!(!sel.is_empty(), "{}: nada seleccionado", entry.key.key());
        vistos += 1;
    }
    // Controlo positivo: a varredura viu o menu inteiro (19 tipos desde 07/11: Physics Body e Ambient saíram).
    assert!(vistos >= 19, "só {vistos} tipos do Add varridos");
}

/// `Add ▸ Sprite` e `Add ▸ Animated Sprite` abrem a escolha da imagem daquele objecto; os outros, não: o
/// Character nasce com o herói de fábrica.
#[test]
fn a_sprite_e_o_animated_nascem_a_pedir_a_imagem() {
    use super::pede_a_imagem;
    use ph2d_app_game::object_add::{ANIMATED_SPRITE, CHARACTER, SPRITE};
    use ph2d_editor_core::action_bus::{ActionBus, EditorAction};
    let mut bus = ActionBus::default();
    for entry in [SPRITE, ANIMATED_SPRITE] {
        pede_a_imagem(entry, 7, &mut bus);
        assert!(matches!(
            bus.drain().collect::<Vec<_>>().as_slice(),
            [EditorAction::OpenImagePicker { entity_bits: 7 }]
        ));
    }
    pede_a_imagem(CHARACTER, 7, &mut bus);
    assert_eq!(bus.drain().count(), 0);
}

/// ⭐⭐ **Add Object: dois cliques REAIS do rato criam e fecham** (smoke do dono 08/11: «dois cliques não
/// funcionam em nenhum dos quadros») — pelo `Down`/`Up` no rect pintado, não pelo `DoubleClick` injetado.
///
/// (Mutação: o ramo do modal no `dispatch_down` não registar o `Down` para o duplo ⇒ RED.)
#[test]
fn add_object_dois_cliques_reais_criam_e_fecham() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.open_command_palette(super::add_model(&[]));
    let alvo = ph2d_app_game::object_add::SPRITE.id();
    assert!(
        crate::component_attach_seam_tests::dois_apertos_reais(&mut hero, alvo),
        "o 1.º aperto não destacou ou fechou"
    );
    assert!(
        !hero.store.command_palette_open(),
        "dois cliques reais não fecharam o Add Object"
    );
    let picked = hero
        .store
        .take_command_pick_if(|id| entry_of_pick(FAMILIES, id).is_some())
        .and_then(|id| entry_of_pick(FAMILIES, id));
    assert_eq!(picked.map(|e| e.id()), Some(alvo), "o pick perdeu-se");
}
