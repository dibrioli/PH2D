use super::*;
use crate::registo_de_teste as registo;
use ph2d_physics_ecs::{LockRotation, PlatformPlayer};

fn nasce(entry: AddEntry, sim: &mut SimWorld) -> Entity {
    let bits = add(
        entry,
        sim,
        &registo(),
        ph2d_app_physics::physics_seed::COMPONENT_SEEDS,
    )
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
            "object_add.game.collision_shape" => (false, false, None, true, false),
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
        assert_eq!(tag.per_frame_ms, t.per_frame_ms, "{}", t.name);
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
    let anims = w.get::<CharacterAnimations>(e).expect("ligação");
    assert_eq!(anims, &CharacterAnimations::default());
}

/// Uma entrada de outra família não é desta: `None`, e nada nasce.
#[test]
fn entrada_alheia_nao_cria_nada() {
    let mut sim = SimWorld::new();
    let antes = sim.world().entities().len();
    assert!(
        add(
            ph2d_editor_core::object_add::EMPTY,
            &mut sim,
            &registo(),
            &[]
        )
        .is_none()
    );
    assert_eq!(sim.world().entities().len(), antes);
}
