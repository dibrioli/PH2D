use ph2d_app_game::object_add::*;
use ph2d_ecs::{Entity, SimWorld, SpriteAnimations, SpriteAnimator, SpriteGrid};
use ph2d_editor_core::object_add::AddEntry;
use ph2d_physics_ecs::{BodyKind, CharacterAnimations, Collider, ColliderShape, RigidBody};
use ph2d_physics_ecs::{LockRotation, PlatformPlayer};
use ph2d_render::{HERO_SHEET_TAGS, Sprite};

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
            w.get::<Collider>(e).is_some(),
            w.get::<PlatformPlayer>(e).is_some() && w.get::<CharacterAnimations>(e).is_some(),
        );
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
        assert_eq!(
            w.get::<ph2d_ecs::Name>(e).map(|n| n.as_str().to_owned()),
            Some(entry.key.tr().to_owned()),
            "o objecto chama-se como a entrada"
        );
    }
}

/// O bloco estático: a semente copia o tamanho da sprite para a colisão (a mesma caixa nos dois).
#[test]
fn o_bloco_estatico_bate_onde_se_ve() {
    let mut sim = SimWorld::new();
    let e = nasce(STATIC_BODY, &mut sim);
    let c = sim.world().get::<Collider>(e).expect("colisão");
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
    assert_eq!(w.get::<Collider>(e).map(|c| c.shape), Some(CAPSULA));
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
    assert_eq!(w.get::<Collider>(e).map(|c| c.friction), Some(0.0));
    assert_eq!(
        w.get::<ph2d_physics_ecs::MaterialCombine>(e)
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

/// ⭐ **Add ▸ Collision Shape, os três casos** (W3): sem seleção é um objecto na raiz com Static próprio;
/// com um corpo seleccionado é FILHA dele e mais uma forma dele (sem corpo próprio, «Part of»); com um
/// objecto sem corpo seleccionado é filha dele e nasce com o seu Static. Nunca componente do seleccionado.
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
    assert_eq!((pai(solta), corpo(solta)), (None, Some(BodyKind::Static)));
    assert_eq!((pai(peca), corpo(peca)), (Some(chao), None));
    assert_eq!(
        ph2d_physics_ecs::owner_body(w, peca),
        Some(chao),
        "é mais uma forma do chão"
    );
    assert_eq!(
        (pai(propria), corpo(propria)),
        (Some(vazio), Some(BodyKind::Static))
    );
    for e in [peca, propria] {
        assert!(w.get::<Collider>(e).is_some());
        let t = w.get::<ph2d_ecs::Transform>(e).expect("pose");
        assert_eq!(
            t.translation,
            ph2d_ecs::Transform::IDENTITY.translation,
            "offset local zero"
        );
    }
    assert_eq!(corpo(vazio), None, "o seleccionado não ganha componente");
    assert!(w.get::<Collider>(chao).is_some() && w.get::<Collider>(vazio).is_none());
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
