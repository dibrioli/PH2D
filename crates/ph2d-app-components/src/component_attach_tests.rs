//! **A SEQUÊNCIA do `+`** (ADR-0166 / F3) — a quarta pergunta da costura, a que as outras três não
//! implicam.
//!
//! O gate do painel prova que o clique chega ao barramento; o do widget prova que a caixa é
//! registada; o do chrome prova que ela vira. **Nenhum dos três prova que o gesto INTEIRO leva a
//! algum lugar** — que abrir a paleta sobre este objeto oferece o que ele pode receber, que virar a
//! caixa revela o inaplicável **sem apagar o que foi escrito na busca**, e que escolher um item
//! deixa o componente na cena.

use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor_core::{HeroScreen, NodeId};

fn registry() -> ph2d_ecs::scene::ComponentRegistry {
    crate::component_registry_for_tests::registo()
}

/// Uma sprite selecionável.
fn image(sim: &mut SimWorld) -> u64 {
    sim.world_mut()
        .spawn((
            Transform::IDENTITY,
            ph2d_ecs::Name::new("Image"),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ))
        .id()
        .to_bits()
}

/// ⭐⭐ **Um objecto VAZIO — e ele é o sujeito de dois destes gates desde 2026-09-14.**
///
/// ⚠️ **A fixtura era uma SPRITE, e a varredura completa do catálogo dissolveu a premissa dela:**
/// o *Show all* revela *«o que não é para este tipo de objecto»*, e para uma imagem essa população
/// ERA a família do vetor. Com as catorze do vetor a saírem da paleta (todas têm porta no painel
/// do vetor ou no gesto que as cria), uma imagem passou a ter **zero** inaplicáveis — `26` de `26`
/// ofertas —, e os dois gates reprovaram a dizer *«o Show all não revelou nada»* sobre um produto
/// correcto.
///
/// ⛔ **A cura não é baixar a asserção: é medir o fenómeno onde ele existe.** Num objecto sem
/// sprite as ofertas `IMAGE`-only (Blend Mode · Texture Filter · 9-Slice · …) continuam
/// inaplicáveis, que é exactamente o que estes dois gates provam. *Um gate cujo sujeito deixou de
/// ter o fenómeno não afirma nada — troca-se o SUJEITO, nunca a barra.*
fn vazio(sim: &mut SimWorld) -> u64 {
    sim.world_mut()
        .spawn((Transform::IDENTITY, ph2d_ecs::Name::new("Object")))
        .id()
        .to_bits()
}

fn labels(hero: &HeroScreen) -> Vec<String> {
    hero.store
        .command_palette_model()
        .map(|m| {
            m.groups
                .iter()
                .flat_map(|g| &g.subs)
                .flat_map(|s| &s.items)
                .map(|i| i.label.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// ⭐ **O gesto inteiro: `+` → paleta → *Show all* → o inaplicável aparece, com a razão.**
///
/// ⚠️ **E a BUSCA sobrevive à caixa.** Ligar *Show all* depois de escrever «slice» não pode apagar
/// o que foi escrito — seria um controlo a desfazer o trabalho do controlo do lado. É por isso que
/// a reconstrução usa `set_command_palette_model` e não `open_command_palette`.
#[test]
fn the_whole_gesture_opens_filters_and_reveals() {
    let mut sim = SimWorld::new();
    let bits = vazio(&mut sim);
    let reg = registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let mut target: Option<u64> = None;

    crate::component_attach::open_palette_if_asked(
        &mut hero,
        &sim,
        &reg,
        Some(bits),
        &mut target,
        &|_| false,
    );
    assert_eq!(target, Some(bits), "a paleta tem de lembrar quem pediu");
    assert!(hero.store.command_palette_open());
    let closed = labels(&hero);
    assert!(
        !closed.iter().any(|l| l.starts_with("9-Slice")),
        "um objecto SEM sprite nao pode oferecer 9-Slice com a caixa desligada: {closed:?}"
    );
    assert!(
        !closed
            .iter()
            .any(|l| l.contains("not for this object type")),
        "com a caixa DESLIGADA o inaplicavel nao aparece"
    );

    // O artista escreve, e depois liga a caixa.
    hero.store.command_palette_push_char('s');
    hero.store.command_palette_push_char('l');
    hero.store.flip_command_palette_toggle();
    crate::component_attach::refresh_palette_on_toggle(&mut hero, &sim, &reg, target, &|_| false);

    assert_eq!(
        hero.store.command_palette_query(),
        "sl",
        "ligar a caixa apagou o que o artista escreveu"
    );
    let open = labels(&hero);
    assert!(
        open.iter().any(|l| l.contains("not for this object type")),
        "o Show all tem de REVELAR o inaplicavel, com a razao: {open:?}"
    );
    // ⭐ E o que ele revela é NOMEÁVEL: o 9-Slice, que a metade de cima provou estar escondido.
    assert!(
        open.iter()
            .any(|l| l.starts_with("9-Slice") && l.contains("not for this object type")),
        "o 9-Slice tem de reaparecer sob o Show all, COM a razao: {open:?}"
    );
    assert!(
        open.len() > closed.len(),
        "o Show all nao revelou nada ({} -> {})",
        closed.len(),
        open.len()
    );
}

/// ⚠️ **A caixa é uma PORTA DE DOIS SENTIDOS** — virá-la outra vez volta a esconder.
///
/// Um controlo que só funciona uma vez é a mesma classe do botão morto: o artista liga *Show all*
/// para procurar, e depois quer a lista curta de volta.
#[test]
fn the_box_goes_both_ways() {
    let mut sim = SimWorld::new();
    let bits = vazio(&mut sim);
    let reg = registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let mut target: Option<u64> = None;
    crate::component_attach::open_palette_if_asked(
        &mut hero,
        &sim,
        &reg,
        Some(bits),
        &mut target,
        &|_| false,
    );
    let closed = labels(&hero).len();

    hero.store.flip_command_palette_toggle();
    crate::component_attach::refresh_palette_on_toggle(&mut hero, &sim, &reg, target, &|_| false);
    let opened = labels(&hero).len();
    assert!(opened > closed);

    hero.store.flip_command_palette_toggle();
    crate::component_attach::refresh_palette_on_toggle(&mut hero, &sim, &reg, target, &|_| false);
    assert_eq!(labels(&hero).len(), closed, "a caixa nao voltou atras");
}

/// ⚠️ **Um pick que não é MEU fica onde está.** A metade de ausência do dreno condicional: sem ela,
/// este consumidor engoliria o pick da biblioteca de nós e o sintoma seria *«às vezes não faz
/// nada»* — um defeito que depende da ordem dos drenos, que ninguém lê.
#[test]
fn a_pick_that_is_not_mine_is_left_alone() {
    let mut sim = SimWorld::new();
    let bits = image(&mut sim);
    let reg = registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let mut target: Option<u64> = None;
    crate::component_attach::open_palette_if_asked(
        &mut hero,
        &sim,
        &reg,
        Some(bits),
        &mut target,
        &|_| false,
    );

    let foreign = ph2d_tool_registry::hash_node_id("motion.boids");
    hero.store.set_command_pick(foreign);
    assert!(
        crate::component_attach::route_pick(&mut hero, &mut target).is_none(),
        "o dreno reclamou um id que nao e' dele"
    );
    assert_eq!(
        hero.store.take_command_pick(),
        Some(foreign),
        "o pick alheio tem de ficar no canal para quem o sabe executar"
    );
}

/// ⭐ **«Animation» na paleta** (doc ObjectsAndComponents 01 §6): oferecida a quem não a tem, à
/// cabeça da família dela; o pick dela é da Timeline (a shell), nunca do anexar ao mundo.
#[test]
fn a_animacao_e_oferecida_a_quem_nao_a_tem_e_o_pick_e_da_timeline() {
    let mut sim = SimWorld::new();
    let bits = image(&mut sim);
    let reg = registry();
    let animation = ph2d_i18n::tr("app.components.component_palette.animation").to_string();
    let mut hero = HeroScreen::new(NodeId(1));
    let mut target: Option<u64> = None;
    crate::component_attach::open_palette_if_asked(
        &mut hero,
        &sim,
        &reg,
        Some(bits),
        &mut target,
        &|_| true,
    );
    assert!(
        !labels(&hero).contains(&animation),
        "quem já a tem não a vê"
    );

    let mut hero = HeroScreen::new(NodeId(1));
    crate::component_attach::open_palette_if_asked(
        &mut hero,
        &sim,
        &reg,
        Some(bits),
        &mut target,
        &|_| false,
    );
    let model = hero
        .store
        .command_palette_model()
        .expect("a paleta abriu")
        .clone();
    let group = model
        .groups
        .iter()
        .find(|g| {
            g.subs
                .iter()
                .any(|s| s.items.iter().any(|i| i.label == animation))
        })
        .expect("«Animation» não foi oferecida");
    assert_eq!(
        group.subs[0].items[0].label, animation,
        "à cabeça da família"
    );

    hero.store
        .set_command_pick(crate::component_palette::ANIMATION_ITEM);
    assert!(
        crate::component_attach::route_pick(&mut hero, &mut target).is_none(),
        "o anexar ao mundo reclamou um item que não é componente do mundo"
    );
    assert_eq!(
        crate::component_attach::take_animation_pick(&mut hero, &mut target),
        Some(bits)
    );
    assert_eq!(target, None, "o alvo é consumido como no pick normal");
}

/// Os bytes de `name` em `bits` (`None` = ausente).
fn bytes(
    sim: &SimWorld,
    reg: &ph2d_ecs::scene::ComponentRegistry,
    bits: u64,
    name: &str,
) -> Option<Vec<u8>> {
    let e = reg.get_by_id(ph2d_ecs::scene::stable_type_id(name))?;
    (e.serialize)(sim.world(), ph2d_ecs::Entity::from_bits(bits)).ok()?
}

/// A semente do corpo da composição (`ph2d_app_physics::physics_seed`), aqui sem a família: a forma nasce
/// numa filha «Collision Shape» (opção B, ADR-0199). Idempotente, como a real.
fn semente_do_corpo(sim: &mut SimWorld, bits: u64) {
    let e = ph2d_ecs::Entity::from_bits(bits);
    if !ph2d_physics_ecs::tem_forma(sim.world(), e) {
        let caixa = ph2d_physics_ecs::ColliderShape::Cuboid {
            half_x: 0.5,
            half_y: 0.5,
        };
        ph2d_physics_ecs::nasce_com_forma(sim.world_mut(), e, caixa);
    }
}

const SEMENTES: crate::component_seed::SeedTable<'static> =
    &[("ph2d::physics::RigidBody", semente_do_corpo)];

/// ⭐ **T60 — o ↺ do cartão Physics Body** (opção B): o corpo volta ao que o «+» anexa; a forma é a
/// filha (outro objecto, com o seu cartão) e fica — o Reset não cria segunda; nenhuma forma no corpo;
/// um componente de outro cartão fica.
#[test]
fn resetting_the_body_puts_body_shape_and_zones_back_as_attached() {
    use ph2d_physics_ecs::{BodyKind, Collider, Health, RigidBody};
    const BODY: &str = "ph2d::physics::RigidBody";
    let reg = registry();
    let mut sim = SimWorld::new();
    let bits = image(&mut sim);
    crate::component_attach::attach_by_name(&mut sim, &reg, SEMENTES, bits, BODY).expect("anexa");
    let e = ph2d_ecs::Entity::from_bits(bits);
    let corpo = bytes(&sim, &reg, bits, BODY);
    assert!(sim.world().get::<Collider>(e).is_none(), "a forma não vai no corpo");
    let forma = ph2d_physics_ecs::main_shape(sim.world(), e).expect("a forma nasce na filha");
    assert_ne!(forma, e);
    sim.world_mut().get_mut::<RigidBody>(e).expect("corpo").kind = BodyKind::Static;
    sim.world_mut()
        .get_mut::<Collider>(forma)
        .expect("forma")
        .density = 9.0;
    sim.world_mut().entity_mut(e).insert(Health::default());
    let vida = bytes(&sim, &reg, bits, "ph2d::physics::Health");
    crate::component_attach::reset_by_name(&mut sim, &reg, SEMENTES, bits, BODY).expect("repõe");
    assert_eq!(
        bytes(&sim, &reg, bits, BODY),
        corpo,
        "o corpo volta ao de fábrica"
    );
    assert!(sim.world().get::<Collider>(e).is_none(), "nem o Reset põe a forma no corpo");
    assert_eq!(
        ph2d_physics_ecs::main_shape(sim.world(), e),
        Some(forma),
        "a filha fica, e o Reset não cria segunda"
    );
    assert_eq!(
        sim.world().get::<Collider>(forma).map(|c| c.density),
        Some(9.0),
        "a forma tem o seu cartão: o ↺ do corpo não a toca"
    );
    assert_eq!(
        bytes(&sim, &reg, bits, "ph2d::physics::Health"),
        vida,
        "outro cartão fica"
    );
}

/// ⭐ **T60 — o ↺ do cartão Platform Player** repõe o Player e deixa o corpo de que ele depende
/// (que tem o seu cartão).
#[test]
fn resetting_the_player_leaves_the_body_it_needs() {
    use ph2d_physics_ecs::{BodyKind, PlatformPlayer, RigidBody};
    const PLAYER: &str = "ph2d::physics::PlatformPlayer";
    let reg = registry();
    let mut sim = SimWorld::new();
    let bits = image(&mut sim);
    crate::component_attach::attach_by_name(&mut sim, &reg, SEMENTES, bits, PLAYER).expect("anexa");
    let player = bytes(&sim, &reg, bits, PLAYER);
    let e = ph2d_ecs::Entity::from_bits(bits);
    sim.world_mut()
        .get_mut::<PlatformPlayer>(e)
        .expect("player")
        .float_height = 7.0;
    sim.world_mut()
        .get_mut::<RigidBody>(e)
        .expect("corpo")
        .kind = BodyKind::Static;
    crate::component_attach::reset_by_name(&mut sim, &reg, SEMENTES, bits, PLAYER).expect("repõe");
    assert_eq!(
        bytes(&sim, &reg, bits, PLAYER),
        player,
        "o Player volta ao de fábrica"
    );
    assert_eq!(
        sim.world().get::<RigidBody>(e).map(|c| c.kind),
        Some(BodyKind::Static),
        "o corpo fica"
    );
}
