use ph2d_app_game::object_add::*;
use ph2d_ecs::{Entity, SimWorld, SpriteAnimations, SpriteAnimator, SpriteGrid};
use ph2d_editor_core::object_add::AddEntry;
use ph2d_physics_ecs::{BodyKind, CharacterAnimations, Collider, ColliderShape, RigidBody};
use ph2d_physics_ecs::{LockRotation, PlatformPlayer};
use ph2d_render::{HERO_SHEET_TAGS, Sprite};

/// A forma principal de `e` (opção B: a filha «Collision Shape»).
fn forma(w: &ph2d_ecs::World, e: Entity) -> Option<&Collider> {
    ph2d_physics_ecs::main_shape(w, e).and_then(|f| w.get::<Collider>(f))
}

fn nasce(entry: AddEntry, sim: &mut SimWorld) -> Entity {
    let bits = crate::comp::menu(entry, sim)
        .expect("a entrada é desta família")
        .unwrap_or_else(|e| panic!("{}: {e}", entry.key.key()));
    Entity::from_bits(bits)
}

/// ⭐ **Cada objecto nasce com o seu conjunto FIXO** — escrito aqui, lado independente do `add`.
#[test]
fn cada_objecto_nasce_com_o_seu_conjunto() {
    for entry in ENTRIES {
        let mut sim = SimWorld::new();
        let e = nasce(*entry, &mut sim);
        let w = sim.world();
        let tem = (
            w.get::<Sprite>(e).is_some(),
            w.get::<SpriteAnimator>(e).is_some() && w.get::<SpriteGrid>(e).is_some(),
            w.get::<RigidBody>(e).map(|b| b.kind),
            forma(w, e).is_some(),
            w.get::<PlatformPlayer>(e).is_some() && w.get::<CharacterAnimations>(e).is_some(),
        );
        // A Collision Shape solta devolve a FORMA (o seleccionado); o conjunto é o do corpo dela.
        let corpo = ph2d_physics_ecs::owner_body(w, e).unwrap_or(e);
        let tem = if corpo == e {
            tem
        } else {
            (
                w.get::<Sprite>(corpo).is_some(),
                false,
                w.get::<RigidBody>(corpo).map(|b| b.kind),
                forma(w, corpo).is_some(),
                false,
            )
        };
        let esperado = match entry.key.key() {
            "object_add.game.sprite" => (true, false, None, false, false),
            "object_add.game.animated_sprite" => (true, true, None, false, false),
            "object_add.game.static_body" => (true, false, Some(BodyKind::Static), true, false),
            "object_add.game.character" => (true, true, Some(BodyKind::Dynamic), true, true),
            "object_add.game.collision_shape" => {
                (false, false, Some(BodyKind::Static), true, false)
            }
            "object_add.game.background_layer" => (true, false, None, false, false),
            other => panic!("entrada sem conjunto esperado: {other}"),
        };
        assert_eq!(tem, esperado, "{}", entry.key.key());
        let nome = if *entry == COLLISION_SHAPE {
            STATIC_BODY
        } else {
            *entry
        };
        assert_eq!(
            w.get::<ph2d_ecs::Name>(corpo)
                .map(|n| n.as_str().to_owned()),
            Some(nome.key.tr().to_owned()),
            "o objecto chama-se como a entrada"
        );
    }
}

/// O bloco estático: a semente copia o tamanho da sprite para a colisão (a mesma caixa nos dois).
#[test]
fn o_bloco_estatico_bate_onde_se_ve() {
    let mut sim = SimWorld::new();
    let e = nasce(STATIC_BODY, &mut sim);
    let c = forma(sim.world(), e).expect("colisão");
    assert_eq!(
        c.shape,
        ColliderShape::Cuboid {
            half_x: BLOCO[0] / 2.0,
            half_y: BLOCO[1] / 2.0
        }
    );
}

/// O personagem: cápsula do boneco, sem rodar, e as quatro animações da folha a tocar `idle`.
#[test]
fn o_personagem_tem_capsula_e_as_quatro_animacoes() {
    let mut sim = SimWorld::new();
    let e = nasce(CHARACTER, &mut sim);
    let w = sim.world();
    assert_eq!(forma(w, e).map(|c| c.shape), Some(CAPSULA));
    assert!(w.get::<LockRotation>(e).is_some(), "o personagem roda");
    let tags = w.get::<SpriteAnimations>(e).expect("animações");
    for t in HERO_SHEET_TAGS {
        let tag = tags
            .get(t.name)
            .unwrap_or_else(|| panic!("falta a animação {}", t.name));
        // Toca os tempos da folha, mas só o que difere do ritmo é próprio (senão o Frames/s não
        // move a animação — report do dono 25/10).
        let tocado: Vec<u32> = (0..t.per_frame_ms.len())
            .map(|i| (tag.frame_factor(i) * tag.frame_ms as f32).round() as u32)
            .collect();
        assert_eq!(tocado, t.per_frame_ms, "{}", t.name);
        assert!(
            !tag.per_frame_ms.contains(&tag.frame_ms),
            "{}: tempo igual ao ritmo gravado como próprio",
            t.name
        );
        assert_eq!(tag.repeat, t.once.then_some(1), "{}", t.name);
    }
    let a = w.get::<SpriteAnimator>(e).expect("animador");
    assert_eq!((a.current.as_str(), a.autoplay), ("idle", true));
    let p = w.get::<PlatformPlayer>(e).expect("player");
    assert_eq!(
        p.float_height,
        ph2d_render::HERO_SHEET_SOLE_BELOW_CENTRE,
        "paira à altura dos pés"
    );
    let min = ph2d_app_physics::inspector::player::fitted_float(Some(CAPSULA), p.max_slope_deg)
        .expect("cápsula")
        / 1.2;
    assert!(
        p.float_height > min,
        "a cápsula ({min}) não cabe acima dos pés"
    );
    assert!(
        p.config().crouch.armed(&p.config().ride),
        "nasce a saber agachar (plano 09)"
    );
    assert_eq!(p.crouch_height, AGACHADO);
    let anims = w.get::<CharacterAnimations>(e).expect("ligação");
    assert_eq!(anims, &CharacterAnimations::default());
}

/// Dono 04/11: o personagem não tem atrito contra as paredes, e o Inspector mostra os dois valores.
#[test]
fn o_personagem_nasce_sem_atrito() {
    let mut sim = SimWorld::new();
    let e = nasce(CHARACTER, &mut sim);
    let w = sim.world();
    assert_eq!(forma(w, e).map(|c| c.friction), Some(0.0));
    let f = ph2d_physics_ecs::main_shape(w, e).expect("forma");
    assert_eq!(
        w.get::<ph2d_physics_ecs::MaterialCombine>(f)
            .map(|m| m.friction),
        Some(ph2d_physics_ecs::CombineRule::Min)
    );
}

/// Uma entrada de outra família não é desta: `None`, e nada nasce.
#[test]
fn entrada_alheia_nao_cria_nada() {
    let mut sim = SimWorld::new();
    let antes = sim.world().entities().len();
    assert!(crate::comp::menu(ph2d_editor_core::object_add::EMPTY, &mut sim).is_none());
    assert_eq!(sim.world().entities().len(), antes);
}

/// A *Collision Shape* com o objecto `pai` seleccionado, pelas portas do produto.
fn forma_sob(sim: &mut SimWorld, pai: Option<Entity>) -> Entity {
    let reg = crate::comp::registo();
    let bits = crate::comp::com_portas(&reg, |p| {
        add(COLLISION_SHAPE, sim, p.menu, pai.map(Entity::to_bits))
    })
    .expect("é do jogo")
    .expect("nasce");
    Entity::from_bits(bits)
}

/// ⭐ **Add ▸ Collision Shape, os três casos** (opção B, respostas do dono 08/11): sem seleção nasce um
/// «Static Body» na raiz com a forma por filha; com um corpo seleccionado é FILHA dele (peça); com um
/// objecto sem corpo seleccionado ele VIRA Static e a forma nasce filha dele. O devolvido (o que fica
/// seleccionado) é sempre a FORMA.
#[test]
fn a_forma_nasce_filha_do_seleccionado_e_so_tem_corpo_se_nao_ha_corpo_acima() {
    let mut sim = SimWorld::new();
    let solta = forma_sob(&mut sim, None);
    let chao = nasce(STATIC_BODY, &mut sim);
    let vazio = Entity::from_bits(ph2d_app_components::object_add::spawn_empty_root(
        &mut sim, "Group",
    ));
    let peca = forma_sob(&mut sim, Some(chao));
    let propria = forma_sob(&mut sim, Some(vazio));
    let w = sim.world();
    let pai = |e| w.get::<ph2d_ecs::ChildOf>(e).map(ph2d_ecs::ChildOf::parent);
    let corpo = |e| w.get::<RigidBody>(e).map(|b| b.kind);
    let raiz = pai(solta).expect("a forma solta é filha do seu Static Body");
    assert_eq!((pai(raiz), corpo(raiz)), (None, Some(BodyKind::Static)));
    assert_eq!(
        w.get::<ph2d_ecs::Name>(raiz).map(|n| n.as_str().to_owned()),
        Some(STATIC_BODY.key.tr().to_owned())
    );
    assert!(w.get::<Sprite>(raiz).is_none(), "sem imagem");
    assert_eq!((pai(peca), corpo(peca)), (Some(chao), None));
    assert_eq!(ph2d_physics_ecs::owner_body(w, peca), Some(chao));
    assert_eq!((pai(propria), corpo(propria)), (Some(vazio), None));
    assert_eq!(
        corpo(vazio),
        Some(BodyKind::Static),
        "o seleccionado VIRA Static"
    );
    for e in [solta, peca, propria] {
        assert!(w.get::<Collider>(e).is_some() && w.get::<RigidBody>(e).is_none());
        let t = w.get::<ph2d_ecs::Transform>(e).expect("pose");
        assert_eq!(t.translation, ph2d_ecs::Transform::IDENTITY.translation);
    }
    for e in [raiz, chao, vazio] {
        assert!(w.get::<Collider>(e).is_none(), "a forma no próprio corpo");
    }
    assert_eq!(ph2d_physics_ecs::main_shape(w, vazio), Some(propria));
}

/// Nenhum corpo do mundo carrega a sua forma (opção B).
fn nenhum_corpo_com_forma(sim: &mut SimWorld, porta: &str) {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &RigidBody, &Collider)>();
    let errados: Vec<Entity> = q.iter(w).map(|(e, _, _)| e).collect();
    assert!(
        errados.is_empty(),
        "{porta}: corpo com Collider: {errados:?}"
    );
    let mut q = w.query::<(Entity, &RigidBody)>();
    let corpos: Vec<Entity> = q.iter(w).map(|(e, _)| e).collect();
    for c in corpos {
        assert!(
            ph2d_physics_ecs::main_shape(w, c).is_some(),
            "{porta}: corpo sem forma"
        );
    }
}

/// ⭐⭐ **Gate `nenhuma_porta_do_editor_poe_a_forma_no_corpo`** (opção B): cada porta do editor que cria
/// um corpo ou uma forma — o menu Add (Static Body, Character, Collision Shape nos três casos), o `+`
/// *Physics Body* e o mover que o traz, o Reset do corpo e o «Add Shape» do corpo sem forma — acaba sem
/// nenhuma entidade com `RigidBody` e `Collider` juntos.
#[test]
fn nenhuma_porta_do_editor_poe_a_forma_no_corpo() {
    let reg = crate::comp::registo();
    let seeds = ph2d_app_physics::physics_seed::COMPONENT_SEEDS;
    let attach = |sim: &mut SimWorld, e: Entity, n: &str| {
        ph2d_app_components::component_attach::attach_by_name(sim, &reg, seeds, e.to_bits(), n)
            .expect(n);
    };
    let imagem = |sim: &mut SimWorld| {
        let bits = ph2d_app_components::object_add::spawn_empty_root(sim, "Picture");
        let e = Entity::from_bits(bits);
        sim.world_mut()
            .entity_mut(e)
            .insert(Sprite::atlas(0, [4.0, 1.0], [1.0; 4]));
        e
    };
    let mut sim = SimWorld::new();
    for entry in ENTRIES {
        nasce(*entry, &mut sim);
        nenhum_corpo_com_forma(&mut sim, entry.key.key());
    }
    let chao = nasce(STATIC_BODY, &mut sim);
    forma_sob(&mut sim, Some(chao));
    let vazio = imagem(&mut sim);
    forma_sob(&mut sim, Some(vazio));
    nenhum_corpo_com_forma(&mut sim, "Add ▸ Collision Shape");

    for nome in ["ph2d::physics::RigidBody", "ph2d::physics::TopDownPlayer"] {
        let e = imagem(&mut sim);
        attach(&mut sim, e, nome);
        nenhum_corpo_com_forma(&mut sim, nome);
        let f = ph2d_physics_ecs::main_shape(sim.world(), e).expect("forma");
        assert_eq!(
            sim.world().get::<Collider>(f).map(|c| c.shape),
            Some(ColliderShape::Cuboid {
                half_x: 2.0,
                half_y: 0.5
            }),
            "{nome}: a caixa do desenho"
        );
        ph2d_app_components::component_attach::reset_by_name(
            &mut sim,
            &reg,
            seeds,
            e.to_bits(),
            "ph2d::physics::RigidBody",
        )
        .expect("reset");
        nenhum_corpo_com_forma(&mut sim, "Reset");
        let filhas = ph2d_ecs::ordered_children(sim.world(), e);
        assert_eq!(filhas, vec![f], "{nome}: o Reset fez uma segunda forma");
    }

    let e = imagem(&mut sim);
    sim.world_mut().entity_mut(e).insert(RigidBody {
        kind: BodyKind::Static,
    });
    for _ in 0..2 {
        let q = ph2d_ecs::scene::EditorCommandQueue::default();
        let edit = ph2d_editor_core::PhysicsFieldEdit::AddShape;
        ph2d_app_physics::inspector::body::apply_physics_edit(
            &mut sim,
            e.to_bits(),
            edit,
            &q,
            &reg,
        );
        ph2d_ecs::scene::apply_editor_commands(sim.world_mut(), &q, &reg).expect("commit");
    }
    nenhum_corpo_com_forma(&mut sim, "Add Shape");
    assert_eq!(
        ph2d_ecs::ordered_children(sim.world(), e).len(),
        1,
        "uma forma só"
    );

    // *Add Body* (imagem sem nada) e *Make Independent Body* (peça com imagem · forma solta).
    let add = |sim: &mut SimWorld, e: Entity| {
        let q = ph2d_ecs::scene::EditorCommandQueue::default();
        let edit = ph2d_editor_core::PhysicsFieldEdit::Add;
        ph2d_app_physics::inspector::body::apply_physics_edit(sim, e.to_bits(), edit, &q, &reg);
        ph2d_ecs::scene::apply_editor_commands(sim.world_mut(), &q, &reg).expect("commit");
    };
    let nua = imagem(&mut sim);
    add(&mut sim, nua);
    nenhum_corpo_com_forma(&mut sim, "Add Body");
    let autorada = Collider {
        shape: ColliderShape::Ball { radius: 0.3 },
        density: 3.5,
        ..Collider::default()
    };
    let peca = imagem(&mut sim);
    let w = sim.world_mut();
    w.entity_mut(peca)
        .insert((autorada, ph2d_ecs::ChildOf(chao)));
    add(&mut sim, peca);
    nenhum_corpo_com_forma(&mut sim, "Make Independent Body (imagem)");
    let f = ph2d_physics_ecs::main_shape(sim.world(), peca).expect("forma da peça");
    assert_eq!(
        sim.world().get::<Collider>(f),
        Some(&autorada),
        "a forma autorada"
    );
    let solta = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Name::new("Loose"),
            autorada,
            ph2d_ecs::Transform::from_translation(ph2d_core::Vec2::new(3.0, 1.0)),
        ))
        .id();
    add(&mut sim, solta);
    nenhum_corpo_com_forma(&mut sim, "Make Independent Body (solta)");
    let pai = sim
        .world()
        .get::<ph2d_ecs::ChildOf>(solta)
        .map(|c| c.parent());
    let corpo = pai.expect("a forma solta passou a filha do corpo novo");
    assert!(
        sim.world().get::<RigidBody>(corpo).is_some(),
        "o corpo novo"
    );
    let pose = ph2d_ecs::world_transform(sim.world(), solta).expect("pose");
    assert_eq!(
        pose.translation,
        ph2d_core::Vec2::new(3.0, 1.0),
        "a forma não saltou"
    );

    // O *Rig* da hierarquia: cada parte sem corpo vira corpo pela mesma porta.
    let tronco = imagem(&mut sim);
    let cabeca = imagem(&mut sim);
    sim.world_mut()
        .entity_mut(cabeca)
        .insert(ph2d_ecs::ChildOf(tronco));
    let plano = ph2d_app_physics::joint_rig::plan(&mut sim, &[tronco.to_bits()]);
    let q = ph2d_ecs::scene::EditorCommandQueue::default();
    let kind = ph2d_physics_ecs::JointKind::Pin;
    let out = ph2d_app_physics::joint_rig::apply(&mut sim, &plano, kind, &q, &reg);
    assert_eq!((out.bodies, out.joints), (2, 1), "o rig");
    nenhum_corpo_com_forma(&mut sim, "Rig");
}

/// A cena 2 continua a mesma: a parede invisível é PEÇA do chão da direita, sem corpo próprio.
#[test]
fn a_parede_invisivel_e_peca_do_chao() {
    let mut sim = SimWorld::new();
    crate::comp::montar(&mut sim, 2);
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &ph2d_ecs::Name)>();
    let nomes: Vec<(Entity, String)> = q.iter(w).map(|(e, n)| (e, n.as_str().to_owned())).collect();
    let achar = |n: &str| {
        nomes
            .iter()
            .find(|(_, m)| m == n)
            .map(|(e, _)| *e)
            .expect(n)
    };
    let (parede, chao) = (achar("Invisible Wall"), achar("Ground Right"));
    assert_eq!(ph2d_physics_ecs::owner_body(w, parede), Some(chao));
    assert!(w.get::<RigidBody>(parede).is_none(), "sem Static próprio");
}
