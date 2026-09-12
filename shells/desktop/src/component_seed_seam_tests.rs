//! **As sementes REAIS de anexar um componente** — a tabela da física entregue à porta da família de
//! componentes, exactamente como o `render_loop` a entrega.
//!
//! ⭐ Estes gates moravam em `ph2d-app-components/src/component_seed.rs` e mudaram-se para a
//! composição na auditoria de arquitectura de 2026-09-12 (A1), com os MESMOS nomes: a família de
//! componentes importava as sementes da `ph2d-app-physics` pelo nome, e uma família não chama outra
//! (ADR-0075). A tabela passou a ser injectada — e o único sítio onde as duas famílias se encontram é
//! a shell. O gate do MECANISMO (*a porta só semeia o que a tabela nomeia*) ficou na crate.

use ph2d_app_components::component_seed::seed_after_attach;
use ph2d_app_physics::physics_seed::COMPONENT_SEEDS as SEEDS;
use ph2d_ecs::{SimWorld, Transform};

fn registry() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// Semeia? — derivado da tabela da física, nunca de uma segunda lista.
fn seeds(canonical_name: &str) -> bool {
    SEEDS.iter().any(|(n, _)| *n == canonical_name)
}

/// **A §14 do Inspector aparece para esta entidade?**
///
/// ⚠️ Era `crate::render_loop::inspector_presence_probe::player`, e a troca é de ENDEREÇO, não
/// de lei: aquele wrapper já chamava esta mesma função pública da `ph2d-app-physics` (o
/// ficheiro dele tem os dois estados lado a lado — o `slice` ainda aponta para um builder
/// privado do `render_loop`, e o `physics`/`player` já apontam para a crate). A sonda fica na
/// shell porque `render_loop/inspector*` é chrome PARTILHADO e sai numa wave com dono próprio;
/// esta família não pode depender dela porque **uma crate nunca pode chamar o `bin`**.
///
/// ⛔ **O helper existe em vez de o `build_player_info` ser soletrado no sítio da asserção** —
/// é a razão que o doc-comment da sonda dá para ela própria existir (*«cada um tem a sua lista
/// de argumentos … sem esta camada a lei teria de ser escrita como oito testes soltos, cada um
/// a soletrar os defaults do vizinho»*), e ela continua a valer deste lado da fronteira.
fn secao_player_aparece(sim: &SimWorld, bits: u64) -> bool {
    ph2d_app_physics::inspector::player::build_player_info(
        sim,
        bits,
        0.0,
        0.0,
        None,
        ph2d_physics_ecs::PlayerLiveness::SPRING,
    )
    .is_some()
}

/// ⭐ **Anexar é INERTE — para tudo o que não semeia.**
///
/// A metade que o plano exige (*"anexar é inerte: bytes do componente == default"*), medida
/// pela porta de produção e não por um `insert` à mão.
///
/// ⚠️ **A tabela [`SEEDS`] é a única excepção, e ela é curta de propósito:** um seed é um valor
/// que o `Default` do tipo **não pode** conhecer porque depende da entidade. Tudo o resto tem
/// de sair do registo exactamente como o tipo o define — senão o `+` passa a ser um gesto que
/// muda a cena, e a promessa *"acrescentar uma seção não mexe no teu trabalho"* cai.
#[test]
fn attaching_is_inert_for_everything_that_does_not_seed() {
    let reg = registry();
    let mut checked = 0usize;
    for d in ph2d_component_desc::all() {
        if !matches!(d.attach, ph2d_component_desc::Attach::Authored { .. })
            || seeds(d.canonical_name)
        {
            continue;
        }
        let Some(entry) = reg.get_by_id(ph2d_ecs::scene::stable_type_id(d.canonical_name)) else {
            continue;
        };
        let Some(insert) = entry.insert_default else {
            continue;
        };
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                ph2d_ecs::Name::new("Object"),
                ph2d_render::Sprite::atlas(0, [2.0, 3.0], [1.0; 4]),
            ))
            .id();
        insert(sim.world_mut(), e).expect("o ponto neutro constroi");
        let before = (entry.serialize)(sim.world(), e).expect("serializa");
        seed_after_attach(SEEDS, &mut sim, e.to_bits(), d.canonical_name);
        let after = (entry.serialize)(sim.world(), e).expect("serializa");
        assert_eq!(
            before, after,
            "anexar {} deixou de ser inerte — se isso e' intencional, o nome tem de entrar na tabela SEEDS com o porque",
            d.canonical_name
        );
        checked += 1;
    }
    assert!(
        checked > 40,
        "o censo varreu so' {checked} componentes — ele nao pode ficar verde por nao medir nada"
    );
}

/// ⭐ **O `Collider` nasce com a CAIXA DO DESENHO**, e não com a bola de meio metro do `Default`.
///
/// (Mutação: tirar o braço do `Collider` do [`seed_after_attach`] ⇒ RED, e o valor que sai é a
/// `Ball { radius: 0.5 }` — exactamente o desencontro de 2026-07-18.)
#[test]
fn the_collider_seed_takes_the_sprites_box() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            ph2d_render::Sprite::atlas(0, [2.0, 3.0], [1.0; 4]),
        ))
        .id();
    ph2d_app_components::component_attach::attach_by_name(
        &mut sim,
        &reg,
        SEEDS,
        e.to_bits(),
        "ph2d::physics::Collider",
    )
    .expect("anexa");
    let shape = sim
        .world()
        .get::<ph2d_physics_ecs::Collider>(e)
        .expect("o collider")
        .shape;
    assert_eq!(
        shape,
        ph2d_physics_ecs::ColliderShape::Cuboid {
            half_x: 1.0,
            half_y: 1.5
        },
        "o collider tem de casar com o sprite 2x3"
    );
}

/// ⚠️ **E o seed do `Collider` NÃO reescreve uma forma autorada** — a lei que o `AddShape` já
/// honrava, medida numa peça que voltava `0,10 x 0,50` com tudo zerado.
#[test]
fn the_collider_seed_never_overwrites_authored_work() {
    let mut sim = SimWorld::new();
    let authored = ph2d_physics_ecs::Collider {
        shape: ph2d_physics_ecs::ColliderShape::Cuboid {
            half_x: 0.17,
            half_y: 0.91,
        },
        density: 3.5,
        ..ph2d_physics_ecs::Collider::default()
    };
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            ph2d_render::Sprite::atlas(0, [2.0, 3.0], [1.0; 4]),
            authored,
        ))
        .id();
    seed_after_attach(SEEDS, &mut sim, e.to_bits(), "ph2d::physics::Collider");
    assert_eq!(
        sim.world().get::<ph2d_physics_ecs::Collider>(e).copied(),
        Some(authored),
        "o seed reescreveu trabalho do artista"
    );
}

/// ⭐ **O `PlatformPlayer` nasce PAIRANDO sobre o próprio collider**, e não tangente.
///
/// (Mutação: tirar o braço do player ⇒ o `float_height` fica no `0,5` do `Default`, que é
/// exactamente o piso desta cápsula: ele encosta.)
#[test]
fn the_player_seed_lifts_him_off_his_own_collider() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            ph2d_physics_ecs::RigidBody::default(),
            ph2d_physics_ecs::Collider {
                shape: ph2d_physics_ecs::ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..ph2d_physics_ecs::Collider::default()
            },
        ))
        .id();
    ph2d_app_components::component_attach::attach_by_name(
        &mut sim,
        &reg,
        SEEDS,
        e.to_bits(),
        "ph2d::physics::PlatformPlayer",
    )
    .expect("anexa");
    let p = sim
        .world()
        .get::<ph2d_physics_ecs::PlatformPlayer>(e)
        .copied()
        .expect("o player");
    let neutral = ph2d_physics_ecs::PlatformPlayer::default();
    assert!(
        p.float_height > neutral.float_height,
        "ele nasceu tangente ({} vs o neutro {})",
        p.float_height,
        neutral.float_height
    );
}

/// ⭐ **A CASCATA chega ao mundo, e o PLAYER nasce completo** (ADR-0166 / F3).
///
/// ⚠️ **É o gate que fecha o buraco que a poda abriria:** sem `requires`, anexar um
/// `PlatformPlayer` a um objeto sem corpo punha o componente lá e a §14 **não aparecia** — o
/// artista escolhia na paleta e nada acontecia, que é a doença que esta fase cura.
///
/// ⚠️ E a ordem importa: as dependências entram ANTES, senão o seed do player mediria um mundo
/// sem `Collider` e ele nasceria tangente.
#[test]
fn attaching_a_player_brings_the_body_and_the_collider() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            ph2d_ecs::Name::new("Hero"),
            ph2d_render::Sprite::atlas(0, [1.0, 2.0], [1.0; 4]),
        ))
        .id();
    ph2d_app_components::component_attach::attach_by_name(
        &mut sim,
        &reg,
        SEEDS,
        e.to_bits(),
        "ph2d::physics::PlatformPlayer",
    )
    .expect("anexa");
    assert!(
        sim.world().get::<ph2d_physics_ecs::RigidBody>(e).is_some(),
        "a cascata tem de trazer o corpo"
    );
    assert!(
        sim.world().get::<ph2d_physics_ecs::Collider>(e).is_some(),
        "e o collider, que vem por transitividade"
    );
    // ⭐ E a §14 aparece — que e' a razao de tudo isto existir.
    assert!(
        secao_player_aparece(&sim, e.to_bits()),
        "o artista anexou o player e a seccao nao apareceu"
    );
}

/// ⚠️ **Correr o seed duas vezes não move nada** — ele é idempotente por construção (`max` /
/// «só na forma ainda neutra»), e é isso que o torna seguro numa porta que alguém pode repetir.
#[test]
fn seeding_twice_changes_nothing() {
    let reg = registry();
    for (name, _) in SEEDS {
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                ph2d_render::Sprite::atlas(0, [2.0, 3.0], [1.0; 4]),
                ph2d_physics_ecs::RigidBody::default(),
                ph2d_physics_ecs::Collider {
                    shape: ph2d_physics_ecs::ColliderShape::Capsule {
                        half_height: 0.3,
                        radius: 0.2,
                    },
                    ..ph2d_physics_ecs::Collider::default()
                },
            ))
            .id();
        ph2d_app_components::component_attach::attach_by_name(
            &mut sim,
            &reg,
            SEEDS,
            e.to_bits(),
            name,
        )
        .expect("anexa");
        let entry = reg
            .get_by_id(ph2d_ecs::scene::stable_type_id(name))
            .expect("registado");
        let once = (entry.serialize)(sim.world(), e).expect("serializa");
        seed_after_attach(SEEDS, &mut sim, e.to_bits(), name);
        let twice = (entry.serialize)(sim.world(), e).expect("serializa");
        assert_eq!(once, twice, "o seed de {name} nao e' idempotente");
    }
}
