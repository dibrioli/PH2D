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
    assert_eq!(make(&mut sim, &r, inst), Err(VerbRefusal::InsideAnInstance));
    // ⚠️ E uma PEÇA no meio da cópia também: a pergunta é sobre os ANCESTRAIS.
    assert_eq!(
        {
            let arm = piece(&sim, inst, "Arm");
            make(&mut sim, &r, arm)
        },
        Err(VerbRefusal::InsideAnInstance)
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

/// ⭐⭐⭐ **UMA CÓPIA NUNCA ATERRA EM CIMA DO QUE VEIO** (report do Enio, 2026-08-26 → 27).
///
/// Duas formas idênticas sobrepostas fazem *«mudei o mestre»* e *«mudei a cópia por cima dele»*
/// serem o mesmo gesto na tela — e foi isso que fez a propagação **parecer morta estando viva**.
///
/// ⚠️ Os **três** lados: a 1.ª cópia sai um passo, a 2.ª sai dois (cascata), e o *Criar componente*
/// **não** desloca — ali a cópia tem de ficar exactamente onde a seleção estava.
///
/// (Mutação: `cascade` não escrever a translação ⇒ RED; cascatear no `Verb::Make` ⇒ RED.)
#[test]
fn a_placed_instance_never_lands_on_top_of_what_it_came_from() {
    let mut sim = SimWorld::new();
    let r = reg();
    let mut echo = MasterEcho::default();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let at = ph2d_core::Vec2::new(2.0, -1.0);
    let src = sim
        .world_mut()
        .spawn((Transform::from_translation(at), Name::new("Badge")))
        .id();
    let step = [0.5_f32, -0.25];
    // ⚠️ **Pelo DRENO, e não pela função** — os dois verbos partilham o `place_step`, e uma
    // mutação que cascateasse o *Criar componente* passava enquanto o gate chamava
    // `make_master` directamente. *Um gate que salta o dreno não mede o verbo, mede a função.*
    {
        let (mut sc, mut mp) = crate::instance_docs::empty_docs();
        assert!(
            super::drain(
                super::Verb::Make,
                &mut sim,
                &r,
                &mut echo,
                src.to_bits(),
                &mut toasts,
                &mut crate::instance_docs::OwnedDocs {
                    vec_scene: &mut sc,
                    vec_entities: &mut mp,
                },
                step,
            ),
            "o *Criar componente* nao fez nada"
        );
    }
    let master = src;
    let mut place = |sim: &mut SimWorld, echo: &mut MasterEcho| {
        let (mut sc, mut mp) = crate::instance_docs::empty_docs();
        super::drain(
            super::Verb::Place,
            sim,
            &r,
            echo,
            master.to_bits(),
            &mut toasts,
            &mut crate::instance_docs::OwnedDocs {
                vec_scene: &mut sc,
                vec_entities: &mut mp,
            },
            step,
        )
    };
    assert!(place(&mut sim, &mut echo), "o *Instantiate* nao fez nada");
    assert!(
        place(&mut sim, &mut echo),
        "o 2o *Instantiate* nao fez nada"
    );

    // As poses das instâncias, sem a que o *Criar componente* deixou no lugar.
    let master_id = sim.world().get::<ph2d_ecs::StableId>(master).expect("id").0;
    let mut poses: Vec<(f32, f32)> = {
        // ⚠️ Só a RAIZ de uma instância tem o elo a apontar para o `master_id`; as peças apontam
        // para as peças do mestre. Não é preciso um segundo componente para as distinguir.
        let mut q = sim.world_mut().query::<(&InstanceOf, &Transform)>();
        q.iter(sim.world())
            .filter(|(link, _)| link.master == master_id)
            .map(|(_, t)| (t.translation.x, t.translation.y))
            .collect()
    };
    poses.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert_eq!(poses.len(), 3, "esperavam-se TRES copias: {poses:?}");
    for (i, (x, y)) in poses.iter().enumerate() {
        let want = (at.x + step[0] * i as f32, at.y + step[1] * i as f32);
        assert!(
            (x - want.0).abs() < 1e-4 && (y - want.1).abs() < 1e-4,
            "a copia {i} aterrou em {:?} e devia aterrar em {want:?} — a cascata (a copia 0 e' a \
             do *Criar componente*, que NAO desloca)",
            (x, y)
        );
    }
}
