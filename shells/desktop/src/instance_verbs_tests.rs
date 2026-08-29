//! Os gates dos três verbos que fecham a tabela (ADR-0164 / F4.5).
//!
//! ⚠️ **O oráculo é o que o ARTISTA vê depois do gesto** — o que está na tela, o que a receita
//! passou a ter, o que as outras cópias receberam. Um gate que contasse chamadas ficaria verde
//! sobre um verbo que faz a coisa errada.

use super::{VerbRefusal, detach};
use crate::instance_smoke::{spawn_master, spawn_ragdoll_scene};
use crate::instance_sync::{MasterEcho, sync_instances};
use ph2d_ecs::{
    Children, Entity, InstanceOf, MasterRoot, Name, ObjectInstance, SimWorld, Transform, Visibility,
};
use ph2d_physics_ecs::PhysicsBridge;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// ⚠️ **Sem documentos vetoriais** — ver `crate::instance_sync_docs` para os que têm.
fn apply(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    echo: &mut MasterEcho,
    clicked: Entity,
) -> Result<usize, VerbRefusal> {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    super::apply_to_master(
        sim,
        r,
        echo,
        clicked,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
}

/// ⚠️ **Sem documentos vetoriais** — estes gates são de sprites/física. Os do documento vivem em
/// `crate::instance_sync_docs`.
fn pass(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    bridge: &PhysicsBridge,
    echo: &mut crate::instance_sync::MasterEcho,
) -> usize {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    sync_instances(
        sim,
        r,
        bridge,
        echo,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
}

/// ⚠️ **Sem documentos vetoriais** — ver `crate::instance_docs`.
fn make(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    entity: Entity,
) -> Result<(Entity, Entity), VerbRefusal> {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    super::make_master(
        sim,
        r,
        entity,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
}

/// ⚠️ **Sem documentos vetoriais** — o ragdoll é feito de sprites. Ver `crate::instance_docs`.
fn ragdoll(sim: &mut SimWorld, r: &ph2d_ecs::scene::ComponentRegistry) -> (Entity, Vec<Entity>) {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    spawn_ragdoll_scene(
        sim,
        r,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
}

/// ⚠️ **Sem documentos vetoriais** — estes gates não têm arte vetorial (os que têm vivem em
/// `instance_docs`). O par vazio existe para a assinatura da porta, que desde a F4.6 clona os
/// documentos possuídos junto com os bytes.
fn instantiate(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    master: Entity,
    parent: Option<Entity>,
) -> Result<Entity, crate::instantiate::Refusal> {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    crate::instantiate::instantiate_master(
        sim,
        r,
        master,
        parent,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
}

fn piece(sim: &SimWorld, root: Entity, name: &str) -> Entity {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
            return e;
        }
        if let Some(kids) = sim.world().get::<Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    panic!("nao ha' peca chamada {name:?}");
}

fn tint(sim: &SimWorld, e: Entity) -> [f32; 4] {
    sim.world()
        .get::<ph2d_render::Sprite>(e)
        .expect("sprite")
        .tint
}

fn paint(sim: &mut SimWorld, e: Entity, c: [f32; 4]) {
    let mut spr = sim
        .world()
        .get::<ph2d_render::Sprite>(e)
        .copied()
        .expect("sprite");
    spr.tint = c;
    sim.world_mut().entity_mut(e).insert(spr);
}

/// Uma subárvore comum na cena: um corpo com uma peça pendurada.
fn plain_rig(sim: &mut SimWorld) -> Entity {
    let root = sim
        .world_mut()
        .spawn((
            Transform::from_translation(ph2d_core::Vec2::new(3.0, 1.0)),
            Name::new("Rig"),
        ))
        .id();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Arm"),
        ph2d_render::Sprite::atlas(
            ph2d_render::WHITE_TILE_KEY,
            [1.0, 0.2],
            [0.5, 0.5, 0.5, 1.0],
        ),
        ph2d_ecs::ChildOf(root),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    root
}

// ── CRIAR COMPONENTE ───────────────────────────────────────────────────────────────────────

/// ⭐⭐ **A seleção vira RECEITA e uma INSTÂNCIA fica no lugar dela** — o gesto do Unity
/// *Create Prefab*.
///
/// (Mutação: não instanciar ⇒ o objeto some da tela, e o gate reprova nomeando a pose.)
#[test]
fn make_master_leaves_an_instance_in_its_place() {
    let mut sim = SimWorld::new();
    let r = reg();
    let rig = plain_rig(&mut sim);
    let where_it_was = sim.world().get::<Transform>(rig).expect("pose").translation;

    let (master, instance) = make(&mut sim, &r, rig).expect("o gesto");
    assert_eq!(
        master, rig,
        "a receita E' a subarvore que o artista escolheu"
    );
    assert!(sim.world().get::<MasterRoot>(master).is_some());
    assert!(sim.world().get::<InstanceOf>(instance).is_some());
    // ⚠️ E ela está no lugar porque a **cópia profunda leva o `Transform` verbatim** — não porque
    // o verbo o reescreva. A 1.ª versão reescrevia, e a prova de mutação mostrou a linha morta.
    assert_eq!(
        sim.world()
            .get::<Transform>(instance)
            .expect("pose")
            .translation,
        where_it_was,
        "a instancia nao ficou NO LUGAR da selecao"
    );
    // E ela traz a subárvore inteira.
    assert_eq!(
        sim.world()
            .get::<ph2d_render::Sprite>(piece(&sim, instance, "Arm"))
            .map(|s| s.size),
        Some([1.0, 0.2]),
        "a instancia nasceu sem a peca"
    );
}

/// ⚠️⚠️ **A RECEITA INTEIRA sai da tela, e o gesto NÃO escreve visibilidade nenhuma.**
///
/// ⛔⛔ **A 1.ª versão deste gate media a coisa errada, e passava.** Ela afirmava
/// `Visibility { hidden: true }` na RAIZ do mestre — o que era verdade — e concluía daí que *«o
/// artista não vê dois objetos empilhados»*, o que era **falso** para toda receita que fosse um
/// grupo: `Visibility` é per-entidade neste motor e não desce aos descendentes (o `sim_extract`
/// diz-o pelo nome), então as PEÇAS da receita continuavam a desenhar. *Um gate sobre o meio
/// (a marca) em vez do fim (o que se desenha) fica verde sobre o defeito que ele existe para
/// apanhar.*
///
/// ⇒ hoje a pergunta é a do EXTRACT: **toda** entidade da receita é `MasterPiece`, e nenhuma da
/// instância é. E o gesto não toca em `Visibility`, para o olho da Hierarquia não passar a mentir.
///
/// (Mutação: `assign_master_pieces` só marcar a raiz ⇒ RED na peça.)
#[test]
fn the_whole_recipe_leaves_the_canvas_and_the_instance_stays() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let rig = plain_rig(&mut sim);
    let (master, instance) = make(&mut sim, &r, rig).expect("o gesto");

    for (what, e) in [("a raiz", master), ("a peca", piece(&sim, master, "Arm"))] {
        assert!(
            sim.world().get::<ph2d_ecs::MasterPiece>(e).is_some(),
            "{what} da receita continua a desenhar — o artista ve' dois objetos empilhados"
        );
    }
    for _ in 0..3 {
        pass(&mut sim, &r, &bridge, &mut echo);
    }
    for (what, e) in [
        ("a raiz", instance),
        ("a peca", piece(&sim, instance, "Arm")),
    ] {
        assert!(
            sim.world().get::<ph2d_ecs::MasterPiece>(e).is_none(),
            "{what} da INSTANCIA foi marcada como receita — o gesto apagou da tela o que o \
             artista escolheu"
        );
        assert!(
            !sim.world().get::<Visibility>(e).is_some_and(|v| v.hidden),
            "{what} da instancia nasceu com o olho fechado"
        );
    }
    // ⚠️ E a autoria de visibilidade fica INTACTA nos dois lados: o gesto não escreve `Visibility`
    // em sítio nenhum, senão o olho da Hierarquia passaria a mostrar um estado que ninguém pediu.
    assert!(
        sim.world().get::<Visibility>(master).is_none(),
        "o gesto escreveu `Visibility` na receita — o olho da Hierarquia passa a mentir"
    );
}

/// ⛔ **Duas recusas, distinguíveis.**
#[test]
fn make_master_refuses_a_master_and_a_piece_of_an_instance() {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = spawn_master(&mut sim);
    assert_eq!(make(&mut sim, &r, master), Err(VerbRefusal::AlreadyAMaster));
    let inst = instantiate(&mut sim, &r, master, None).expect("instancia");
    // ⚠️⚠️ **A RAIZ da cópia SAIU desta lista em 2026-08-27, e a saída é a F5.**
    //
    // Ela era recusada com o mesmo `InsideAnInstance`, e o doc do verbo já dizia porquê: *«a
    // resposta certa é a da F5 (aninhamento), não um mestre a meio de uma cópia»*. Marcar a raiz
    // faz dela uma **variante** — receita das cópias dela, instância da base —, que é o critério 2
    // da F5. Quem o afirma é
    // `the_root_of_a_copy_becomes_a_variant_and_a_piece_still_cannot`, e ele mede **as duas
    // metades**, porque a cura foi estreitar a condição e não apagá-la.
    // ⚠️ E uma PEÇA no meio da cópia continua recusada: a pergunta é sobre os ANCESTRAIS.
    assert_eq!(
        {
            let arm = piece(&sim, inst, "Arm");
            make(&mut sim, &r, arm)
        },
        Err(VerbRefusal::InsideAnInstance)
    );
    // ⭐⭐ **O caso ancestral A SÉRIO** (auditoria §1.8): uma entidade **sem `InstanceOf` próprio**
    // pendurada dentro da cópia viva. Toda peça nascida da cópia profunda tem elo, então a
    // travessia ancestral nunca corria — o gate acima confirmava o caminho curto e assinava o
    // longo. É o que um *Add Child* sobre uma peça produz, e a recusa não disparava: nascia um
    // `MasterRoot` **dentro de uma instância viva**, e o pedaço dela desaparecia da tela.
    let stowaway = {
        let arm = piece(&sim, inst, "Arm");
        sim.world_mut()
            .spawn((
                Transform::IDENTITY,
                Name::new("Child"),
                ph2d_ecs::ChildOf(arm),
            ))
            .id()
    };
    assert!(
        sim.world().get::<InstanceOf>(stowaway).is_none(),
        "o controlo negativo caiu: o filho novo TEM elo, e a travessia ancestral nao seria exercida"
    );
    assert_eq!(
        make(&mut sim, &r, stowaway),
        Err(VerbRefusal::InsideAnInstance),
        "um filho acrescentado DEPOIS virou receita dentro de uma copia viva"
    );
}

/// ⭐⭐⭐ **Os DOIS verbos de instanciar, e o que os separa** (Enio, 2026-08-27).
///
/// > *«No modelo Blender há os dois modos: Duplicate e Duplicate Linked.»*
///
/// *Instantiate* dá arte **própria** (`Shift+D`); *Instantiate Linked* dá uma cópia que **divide a
/// arte** da receita (`Alt+D`). A marca é o que os dois consumidores — a tinta e o documento —
/// leem, e ela vai em **toda peça**, não só na raiz: eles têm em mão a peça que o artista tocou.
///
/// ⚠️ **Pelo DRENO, e não pela função**: é o dreno que traduz o verbo em lei, e um gate que o
/// saltasse mediria o `instantiate_master`, que já recebe a resposta pronta.
///
/// (Mutação: dar `ArtLink::Shared` aos dois verbos ⇒ RED no lado `Own`; e ao contrário ⇒ RED no
/// outro. É por isso que o gate mede os DOIS na mesma cena.)
#[test]
fn the_two_instantiate_verbs_differ_only_in_which_art_law_the_copy_follows() {
    let mut sim = SimWorld::new();
    let r = reg();
    let mut echo = MasterEcho::default();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let master = spawn_master(&mut sim);
    let mut place = |sim: &mut SimWorld, echo: &mut MasterEcho, verb: super::Verb| {
        let (mut sc, mut mp) = crate::instance_docs::empty_docs();
        assert!(
            super::drain(
                verb,
                sim,
                &r,
                echo,
                master.to_bits(),
                &mut toasts,
                &mut crate::instance_docs::OwnedDocs {
                    vec_scene: &mut sc,
                    vec_entities: &mut mp,
                },
                [0.0, 0.0],
            ),
            "o verbo {verb:?} nao fez nada"
        );
    };
    place(&mut sim, &mut echo, super::Verb::Place);
    place(&mut sim, &mut echo, super::Verb::PlaceLinked);

    let master_id = sim.world().get::<ph2d_ecs::StableId>(master).expect("id").0;
    let mut roots: Vec<Entity> = {
        let mut q = sim.world_mut().query::<(Entity, &InstanceOf)>();
        q.iter(sim.world())
            .filter(|(_, l)| l.master == master_id)
            .map(|(e, _)| e)
            .collect()
    };
    roots.sort();
    assert_eq!(roots.len(), 2, "os dois verbos nao deixaram duas copias");
    let linked: Vec<bool> = roots
        .iter()
        .map(|&e| sim.world().get::<ph2d_ecs::LinkedArt>(e).is_some())
        .collect();
    assert_eq!(
        linked.iter().filter(|l| **l).count(),
        1,
        "as duas copias seguem a MESMA lei ({linked:?}) — os dois itens do menu fazem o mesmo"
    );
    // ⚠️ E a marca vai em toda PEÇA, senão a tinta e o documento — que recebem a peça, nunca a
    // raiz — leriam a ausência e a cópia ligada comportava-se como uma normal.
    let ligada = roots[usize::from(linked[1])];
    assert!(
        sim.world()
            .get::<ph2d_ecs::LinkedArt>(piece(&sim, ligada, "Arm"))
            .is_some(),
        "a peca da copia ligada nao tem a marca — so' a raiz a tem, e ninguem le' a raiz"
    );
}

/// ⛔⛔ **Dentro de outra RECEITA, também não** — auditoria §1.1, e é a porta cujo dano **não** se
/// cura sozinha no quadro seguinte.
///
/// `master_root_of` pára na raiz MAIS PRÓXIMA, então um `MasterRoot` aninhado **encurta a
/// sub-árvore de edição**: seleccionar a receita exterior deixa de acender o que está debaixo da
/// interior, e a instância irmã fica invisível **mesmo com a receita seleccionada**.
///
/// (Mutação: apagar a guarda ⇒ RED; e o `Err` distingue-se de `InsideAnInstance`, senão o toast
/// diria a frase errada sobre a coisa errada.)
#[test]
fn make_master_refuses_inside_another_component() {
    let mut sim = SimWorld::new();
    let r = reg();
    let outer = spawn_master(&mut sim);
    let inner = piece(&sim, outer, "Arm");
    assert_eq!(
        make(&mut sim, &r, inner),
        Err(VerbRefusal::InsideAMaster),
        "uma peca da receita virou receita — a instancia irma fica invisivel para sempre"
    );
    // Controlo POSITIVO: fora da receita, a MESMA sub-árvore é aceite.
    sim.world_mut()
        .entity_mut(inner)
        .remove::<ph2d_ecs::ChildOf>();
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    assert!(
        make(&mut sim, &r, inner).is_ok(),
        "a guarda recusa tambem fora de uma receita — ela nao mede o aninhamento"
    );
}

// ── DESTACAR ───────────────────────────────────────────────────────────────────────────────

/// ⭐ **Destacar corta o vínculo e não muda mais nada** — os objetos continuam iguais, só deixam
/// de seguir a receita.
///
/// (Mutação: não remover o `InstanceOf` das PEÇAS ⇒ o sync continua a alcançá-las e o gate
/// reprova quando a receita muda.)
#[test]
fn detaching_stops_the_following_and_changes_nothing_else() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let (master, roots) = ragdoll(&mut sim, &r);
    pass(&mut sim, &r, &bridge, &mut echo);

    let mine = piece(&sim, roots[0], "Arm");
    let before = tint(&sim, mine);
    // O gesto, feito a partir de uma PEÇA — e ele solta a instância INTEIRA.
    assert_eq!(detach(&mut sim, mine), Ok(4));
    assert!(sim.world().get::<InstanceOf>(roots[0]).is_none());
    assert!(sim.world().get::<InstanceOf>(mine).is_none());
    assert_eq!(tint(&sim, mine), before, "destacar mudou o que se ve'");

    // A receita muda; a solta não ouve mais, e as outras duas ouvem.
    let master_arm = piece(&sim, master, "Arm");
    paint(&mut sim, master_arm, [0.9, 0.9, 0.1, 1.0]);
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        tint(&sim, mine),
        before,
        "a solta continuou a seguir a receita"
    );
    assert_eq!(
        tint(&sim, piece(&sim, roots[1], "Arm")),
        [0.9, 0.9, 0.1, 1.0],
        "as outras deixaram de seguir — destacar UMA soltou todas"
    );
}

/// ⛔ **Destacar o que não é instância é recusado** (a receita não é cópia de ninguém).
#[test]
fn detaching_something_that_is_not_an_instance_is_refused() {
    let mut sim = SimWorld::new();
    let master = spawn_master(&mut sim);
    assert_eq!(detach(&mut sim, master), Err(VerbRefusal::NotAnInstance));
}

// ── APLICAR AO MESTRE ──────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **APLICAR promove a excepção a padrão** — o valor entra na receita, e as OUTRAS cópias
/// recebem-no.
///
/// ⚠️ **É a régua inteira do verbo**: se ele só apagasse a excepção, a cópia voltaria à cor antiga
/// (isso é o *Redefinir*); se só escrevesse na receita sem limpar a chave, a cópia continuaria
/// surda e a diferença voltaria no gesto seguinte.
///
/// (Mutação: trocar o `insert_from_bytes` no mestre por um no-op ⇒ as outras não recebem nada.)
#[test]
fn applying_an_override_makes_it_the_recipe_for_everyone() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let (master, roots) = ragdoll(&mut sim, &r);
    pass(&mut sim, &r, &bridge, &mut echo);

    let mine = piece(&sim, roots[0], "Arm");
    paint(&mut sim, mine, [0.1, 0.2, 0.9, 1.0]);
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        sim.world()
            .get::<ObjectInstance>(roots[0])
            .map_or(0, |o| o.overrides.len()),
        1,
        "a fixtura tem de conter a excepcao, senao o gate nao mede nada"
    );

    assert_eq!(apply(&mut sim, &r, &mut echo, mine), Ok(1));
    assert_eq!(
        tint(&sim, piece(&sim, master, "Arm")),
        [0.1, 0.2, 0.9, 1.0],
        "o valor nao chegou a' RECEITA"
    );
    assert_eq!(
        sim.world()
            .get::<ObjectInstance>(roots[0])
            .map_or(0, |o| o.overrides.len()),
        0,
        "a excepcao sobreviveu ao Apply — a copia fica surda ao proprio valor que promoveu"
    );

    pass(&mut sim, &r, &bridge, &mut echo);
    for (i, &root) in roots.iter().enumerate() {
        assert_eq!(
            tint(&sim, piece(&sim, root, "Arm")),
            [0.1, 0.2, 0.9, 1.0],
            "a instancia {} nao recebeu o valor promovido",
            i + 1
        );
    }
}

/// ⚠️ **O ESCOPO é o que se clicou** — numa peça, só a excepção dela.
#[test]
fn applying_from_a_piece_promotes_only_that_piece() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let (master, roots) = ragdoll(&mut sim, &r);
    pass(&mut sim, &r, &bridge, &mut echo);

    let arm = piece(&sim, roots[0], "Arm");
    let hub = piece(&sim, roots[0], "Hub");
    let hub_before = tint(&sim, piece(&sim, master, "Hub"));
    paint(&mut sim, arm, [0.1, 0.2, 0.9, 1.0]);
    paint(&mut sim, hub, [0.9, 0.1, 0.9, 1.0]);
    pass(&mut sim, &r, &bridge, &mut echo);

    assert_eq!(apply(&mut sim, &r, &mut echo, arm), Ok(1));
    assert_eq!(
        tint(&sim, piece(&sim, master, "Hub")),
        hub_before,
        "aplicar o BRACO promoveu tambem o eixo — o escopo esta' errado"
    );
    assert_eq!(
        sim.world()
            .get::<ObjectInstance>(roots[0])
            .map_or(0, |o| o.overrides.len()),
        1,
        "a excepcao do eixo tinha de ficar"
    );
}

/// **Sem excepção nenhuma o verbo responde ZERO** — e não um erro: o artista clicou no sítio certo.
#[test]
fn applying_with_nothing_overridden_answers_zero() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let (_master, roots) = ragdoll(&mut sim, &r);
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(apply(&mut sim, &r, &mut echo, roots[0]), Ok(0));
}
