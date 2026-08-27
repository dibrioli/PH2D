//! ⭐ **Os gates da porta de INSTANCIAR** (ADR-0164 / F4.2) — incluindo o smoke-gate 1 do plano.
//!
//! ⚠️ **O oráculo é a CENA, nunca «o remap correu»**: o que tem de ser verdade é que o braço de
//! cada instância fica preso ao eixo DELA depois de 120 tiques de gravidade. Um gate que
//! contasse remapeamentos ficaria verde sobre uma tabela que ninguém chama.

use crate::instance_smoke::spawn_master;
use ph2d_ecs::{
    Children, Entity, InstanceOf, MasterPiece, MasterRoot, Name, SimWorld, StableId, Transform,
};
use ph2d_physics_ecs::{PhysicsBridge, PhysicsJoint};

/// ⚠️ **Sem documentos vetoriais** — estes gates não têm arte vetorial (os que têm vivem em
/// `crate::instance_docs`). O par vazio existe para a assinatura da porta, que desde a F4.6 clona
/// os documentos possuídos junto com os bytes.
fn instantiate(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    master: Entity,
    parent: Option<Entity>,
) -> Result<Entity, super::Refusal> {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    super::instantiate_master(
        sim,
        r,
        master,
        parent,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
}

/// ⚠️ **Sem documentos vetoriais** — o ragdoll é feito de sprites. Ver `crate::instance_docs`.
fn ragdoll(sim: &mut SimWorld, r: &ph2d_ecs::scene::ComponentRegistry) -> (Entity, Vec<Entity>) {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    crate::instance_smoke::spawn_ragdoll_scene(
        sim,
        r,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
}

/// Irmã de [`instantiate`], pela mesma razão.
fn duplicate(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    src: Entity,
) -> Option<Entity> {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    super::duplicate_subtree(
        sim,
        r,
        src,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
}

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// A pose de mundo de uma entidade — o que o artista vê, e não o local.
fn world_at(sim: &SimWorld, e: Entity) -> ph2d_core::Vec2 {
    ph2d_ecs::world_transform(sim.world(), e)
        .expect("a peca existe")
        .translation
}

/// Os descendentes de `root` com um nome dado — a peça de cada instância.
fn piece(sim: &SimWorld, root: Entity, name: &str) -> Entity {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) && e != root {
            return e;
        }
        if let Some(kids) = sim.world().get::<Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    panic!("a instancia nao tem peca chamada {name:?}");
}

/// ⭐⭐ **O SMOKE-GATE 1 DO PLANO: cada junta prende os corpos DELA.**
///
/// Três instâncias do mesmo ragdoll, 120 tiques de gravidade. Cada braço tem de continuar a
/// `ARM` do eixo da PRÓPRIA instância — que é o que um pino faz.
///
/// ⛔ Sem o remap as três juntas nomeiam os corpos do MESTRE, que a F4.1 tirou do solver: as
/// juntas não prendem nada e os braços caem. **É por isso que a régua é a distância ao eixo, e
/// não «o braço mexeu-se»** — um braço solto também se mexe, e cai.
///
/// (Mutação: apagar a chamada ao `remap_object_refs` na porta ⇒ RED nomeando a distância.)
#[test]
fn the_instance_joint_binds_the_instances_own_bodies() {
    let mut sim = SimWorld::new();
    let r = reg();
    let (_master, roots) = ragdoll(&mut sim, &r);
    assert_eq!(roots.len(), 3, "as tres instancias tem de nascer");
    for (i, x) in [-2.4_f32, 0.0, 2.4].into_iter().enumerate() {
        sim.world_mut()
            .entity_mut(roots[i])
            .insert(Transform::from_translation(ph2d_core::Vec2::new(x, 1.2)));
    }

    let mut bridge = PhysicsBridge::new();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, true, t);
    }

    for (i, &root) in roots.iter().enumerate() {
        let hub = world_at(&sim, piece(&sim, root, "Hub"));
        let arm = world_at(&sim, piece(&sim, root, "Arm"));
        let d = (arm - hub).length();
        assert!(
            (d - 0.9).abs() < 0.05,
            "a instancia {} tem o braco a {d:.3} do eixo dela (o pino manda 0.900) — \
             a junta prendeu no MESTRE, nao nos corpos dela",
            i + 1
        );
        assert!(
            arm.y < hub.y,
            "a instancia {} nao balancou: o braco continua ao lado do eixo, \
             entao ela nao esta' a simular",
            i + 1
        );
    }
    // ⚠️ **O controle da separação**: os três braços têm de estar em sítios DIFERENTES. Se
    // todos convergissem, o que se veria era um ragdoll só, e as réguas acima passariam.
    let xs: Vec<f32> = roots
        .iter()
        .map(|&r| world_at(&sim, piece(&sim, r, "Arm")).x)
        .collect();
    assert!(
        (xs[0] - xs[1]).abs() > 1.0 && (xs[1] - xs[2]).abs() > 1.0,
        "os tres bracos convergiram ({xs:?}) — as instancias partilham corpos"
    );
}

/// ⚠️ **A RECEITA continua parada** enquanto as instâncias caem — a metade da F4.1 que esta
/// cena tem de manter viva.
#[test]
fn the_recipe_does_not_move_while_the_instances_do() {
    let mut sim = SimWorld::new();
    let r = reg();
    let (master, roots) = ragdoll(&mut sim, &r);
    let before = world_at(&sim, piece(&sim, master, "Arm"));
    let mut bridge = PhysicsBridge::new();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, true, t);
    }
    let after = world_at(&sim, piece(&sim, master, "Arm"));
    assert!(
        (after - before).length() < 1.0e-5,
        "a receita mexeu-se ({before:?} -> {after:?})"
    );
    // Controle positivo: as instâncias mexeram-se, senão o gate acima é sobre um mundo parado.
    let moved = world_at(&sim, piece(&sim, roots[0], "Arm"));
    assert!(moved.y < 1.2, "a instancia tambem nao simulou ({moved:?})");
}

/// ⚠️⚠️ **A instância aponta para o MESTRE, nunca para si própria.**
///
/// O mapa de identidade contém `mestre → cópia do mestre` (tem de conter). Inserir o
/// [`InstanceOf`] antes do remap fá-lo-ia ser reescrito para a identidade da própria cópia — e o
/// sync da F4.3 leria *"o mestre sou eu"* e nunca mais propagaria nada, calado.
///
/// (Mutação: mover a inserção do `InstanceOf` para antes do `remap_object_refs` ⇒ RED.)
#[test]
fn the_instance_points_at_the_master_not_at_itself() {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = spawn_master(&mut sim);
    let inst = instantiate(&mut sim, &r, master, None).expect("instancia");
    let master_id = sim.world().get::<StableId>(master).expect("id").0;
    let own_id = sim.world().get::<StableId>(inst).expect("id").0;
    let link = sim.world().get::<InstanceOf>(inst).expect("elo").master;
    assert_ne!(link, own_id, "a instancia diz-se instancia de SI PROPRIA");
    assert_eq!(link, master_id, "o elo nao aponta para o mestre");
}

/// ⚠️ **A instância NÃO é um mestre** — com o marcador ela nascia inerte (F4.1), e o artista
/// veria três ragdolls no lugar certo e nenhum a cair.
#[test]
fn an_instance_is_not_a_master() {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = spawn_master(&mut sim);
    let inst = instantiate(&mut sim, &r, master, None).expect("instancia");
    assert!(sim.world().get::<MasterRoot>(inst).is_none());
    let arm = piece(&sim, inst, "Arm");
    assert!(
        sim.world().get::<MasterPiece>(arm).is_none(),
        "a peca da instancia ficou marcada como peca de mestre — ela nunca simularia"
    );
    // E o mestre continua a ser um, com as peças dele marcadas.
    assert!(sim.world().get::<MasterRoot>(master).is_some());
    assert!(
        sim.world()
            .get::<MasterPiece>(piece(&sim, master, "Arm"))
            .is_some()
    );
}

/// **A instância recebe nome PRÓPRIO** — a Hierarquia mostra três linhas distinguíveis.
#[test]
fn each_instance_gets_its_own_name() {
    let mut sim = SimWorld::new();
    let r = reg();
    let (master, roots) = ragdoll(&mut sim, &r);
    let base = sim.world().get::<Name>(master).expect("nome").0.clone();
    let mut names: Vec<String> = roots
        .iter()
        .map(|&e| sim.world().get::<Name>(e).expect("nome").0.clone())
        .collect();
    names.push(base);
    names.sort();
    names.dedup();
    assert_eq!(names.len(), 4, "duas linhas da Hierarquia tem o mesmo nome");
}

/// ⛔ **Instanciar o que não é mestre é RECUSADO** — ver o doc-comment da porta.
#[test]
fn only_a_master_can_be_instantiated() {
    let mut sim = SimWorld::new();
    let r = reg();
    let plain = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Crate")))
        .id();
    assert_eq!(
        instantiate(&mut sim, &r, plain, None),
        Err(super::Refusal::NotAMaster)
    );
}

/// ⚠️ **Uma referência para FORA do mestre continua a apontar para fora.**
///
/// O ragdoll pendurado num gancho da CENA continua pendurado nesse gancho depois de
/// instanciado — as três instâncias partilham o gancho, que é o que o artista autorou.
#[test]
fn a_reference_out_of_the_master_still_points_out() {
    let mut sim = SimWorld::new();
    let r = reg();
    let hook = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Hook")))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let hook_id = sim.world().get::<StableId>(hook).expect("id").0;
    let master = spawn_master(&mut sim);
    let pin = piece(&sim, master, "Pin");
    sim.world_mut()
        .entity_mut(pin)
        .get_mut::<PhysicsJoint>()
        .expect("junta")
        .body_a = hook_id;

    let inst = instantiate(&mut sim, &r, master, None).expect("instancia");
    let copied_pin = piece(&sim, inst, "Pin");
    let j = sim.world().get::<PhysicsJoint>(copied_pin).expect("junta");
    assert_eq!(
        j.body_a, hook_id,
        "a ponta que apontava para a CENA foi reescrita"
    );
    let arm_id = sim
        .world()
        .get::<StableId>(piece(&sim, inst, "Arm"))
        .expect("id")
        .0;
    assert_eq!(j.body_b, arm_id, "a ponta INTERNA nao foi remapeada");
}

/// ⚠️ **Gate estrutural: a cópia profunda tem UM chamador no produto.**
///
/// [`ph2d_ecs::deep_copy_subtree`] copia bytes e não remapeia referência nenhuma — usá-la
/// diretamente dá uma cópia cujas juntas prendem no original, e o defeito é mudo. A porta que
/// compõe as duas metades é esta; qualquer outro chamador de produto é um segundo caminho por
/// onde o mesmo defeito volta.
#[test]
fn only_the_instantiate_door_calls_the_deep_copy() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut callers: Vec<String> = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("ler src") {
            let p = entry.expect("entrada").path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            // Os testes podem chamá-la à vontade: eles é que provam o que ela faz sozinha.
            if name.ends_with("_tests.rs") {
                continue;
            }
            let text = std::fs::read_to_string(&p).unwrap_or_default();
            if text.contains("deep_copy_subtree(") {
                callers.push(name.to_string());
            }
        }
    }
    callers.sort();
    assert_eq!(
        callers,
        vec!["instantiate.rs".to_string()],
        "a copia profunda ganhou um chamador de produto fora da porta: {callers:?}"
    );
}

/// ⭐⭐ **DUPLICAR traz o RIG INTEIRO, e a cópia pendura-se no pino DELA.**
///
/// ⛔ O que estava na Hierarquia antes desta fatia copiava quatro componentes e **nenhum filho**:
/// duplicar este rig dava uma linha vazia — o `piece()` abaixo entra em pânico a dizer que não há
/// peça nenhuma, que é exatamente a régua.
///
/// (Mutação: voltar a `spawn` só `Transform`+`Sprite`+`Name` ⇒ RED no `piece`.)
#[test]
fn duplicating_a_rig_brings_the_whole_subtree_and_its_own_pin() {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = spawn_master(&mut sim);
    let inst = instantiate(&mut sim, &r, master, None).expect("instancia");
    sim.world_mut()
        .entity_mut(inst)
        .insert(Transform::from_translation(ph2d_core::Vec2::new(-2.0, 1.2)));

    let copy = duplicate(&mut sim, &r, inst).expect("duplicado");
    sim.world_mut()
        .entity_mut(copy)
        .insert(Transform::from_translation(ph2d_core::Vec2::new(2.0, 1.2)));

    let mut bridge = PhysicsBridge::new();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, true, t);
    }
    for (what, root) in [("o original", inst), ("a copia", copy)] {
        let hub = world_at(&sim, piece(&sim, root, "Hub"));
        let arm = world_at(&sim, piece(&sim, root, "Arm"));
        let d = (arm - hub).length();
        assert!(
            (d - 0.9).abs() < 0.05,
            "{what} tem o braco a {d:.3} do eixo dele — a junta prendeu no rig errado"
        );
        assert!(arm.y < hub.y, "{what} nao balancou");
    }
    let ax = world_at(&sim, piece(&sim, inst, "Arm")).x;
    let bx = world_at(&sim, piece(&sim, copy, "Arm")).x;
    assert!(
        (ax - bx).abs() > 1.0,
        "os dois bracos convergiram ({ax:.3} / {bx:.3}) — eles partilham corpos"
    );
}

/// ⚠️ **Duplicar uma INSTÂNCIA dá outra instância do MESMO mestre** — o elo aponta para fora do
/// que se copiou, então o remap (com razão) não lhe toca.
#[test]
fn duplicating_an_instance_keeps_it_an_instance_of_the_same_master() {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = spawn_master(&mut sim);
    let inst = instantiate(&mut sim, &r, master, None).expect("instancia");
    let copy = duplicate(&mut sim, &r, inst).expect("duplicado");
    let a = sim.world().get::<InstanceOf>(inst).expect("elo").master;
    let b = sim
        .world()
        .get::<InstanceOf>(copy)
        .expect("elo da copia")
        .master;
    assert_eq!(a, b, "a copia da instancia ficou ligada a outro mestre");
    assert_eq!(
        a,
        sim.world().get::<StableId>(master).expect("id").0,
        "e o mestre nao e' o mestre"
    );
}

/// ⚠️ **Duplicar um MESTRE dá outro mestre — e as peças dele nascem INERTES no mesmo quadro.**
///
/// Sem o `assign_master_pieces` na porta, a receita copiada simularia até ao próximo passe da
/// ponte: um ragdoll da biblioteca a cair meio metro e a parar.
#[test]
fn duplicating_a_master_gives_a_master_whose_pieces_are_already_inert() {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = spawn_master(&mut sim);
    let copy = duplicate(&mut sim, &r, master).expect("duplicado");
    assert!(sim.world().get::<MasterRoot>(copy).is_some());
    assert!(
        sim.world()
            .get::<MasterPiece>(piece(&sim, copy, "Arm"))
            .is_some(),
        "a peca da receita copiada nao esta' marcada — ela simularia"
    );
}

/// **A cópia fica ao lado da fonte** (mesmo pai), e não na raiz da cena.
#[test]
fn the_duplicate_lands_beside_its_source() {
    let mut sim = SimWorld::new();
    let r = reg();
    let host = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Host")))
        .id();
    let src = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Crate"),
            ph2d_ecs::ChildOf(host),
        ))
        .id();
    let copy = duplicate(&mut sim, &r, src).expect("duplicado");
    assert_eq!(
        sim.world().get::<ph2d_ecs::ChildOf>(copy).map(|c| c.0),
        Some(host),
        "a copia saiu de baixo do pai da fonte"
    );
}

/// ⛔⛔ **Instanciar DENTRO do próprio mestre é recusado** — senão a receita passa a conter uma
/// instância de si mesma, o sync propaga o mestre para dentro do mestre (que cresce a cada
/// quadro), e a cópia profunda seguinte copia a cópia.
///
/// ⚠️ **A recusa é no GESTO, e não um tecto de profundidade**: um limite numérico transformaria um
/// erro de autoria numa contagem, e o artista veria a árvore crescer até um número que ninguém lhe
/// explicou.
///
/// ⚠️ E as duas recusas são **distinguíveis**, para que o gesto (F4.5) possa dizer frases
/// diferentes — *duas recusas que devolvem o mesmo `None` produzem o mesmo aviso inútil*.
#[test]
fn instantiating_inside_the_master_itself_is_refused() {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = spawn_master(&mut sim);
    let inside = piece(&sim, master, "Arm");
    assert_eq!(
        instantiate(&mut sim, &r, master, Some(inside)),
        Err(super::Refusal::WouldNestInItself),
        "a instancia aterrou DENTRO da propria receita"
    );
    // A própria raiz do mestre é o caso de bordo do mesmo laço.
    assert_eq!(
        instantiate(&mut sim, &r, master, Some(master)),
        Err(super::Refusal::WouldNestInItself)
    );
    // E o controle positivo: FORA do mestre continua a funcionar.
    let host = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Host")))
        .id();
    assert!(instantiate(&mut sim, &r, master, Some(host)).is_ok());
}
