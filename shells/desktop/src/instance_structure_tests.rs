//! Os gates da FORMA de uma instância (ADR-0164 / F5.1).
//!
//! ⚠️ **Os quatro auxiliares são `pub(super)`** porque o irmão [`super::refuse_tests`] os usa — o
//! corte por assunto (imposto pelo tecto de 600 LOC) não pode duplicar a fixtura: *duas fixturas
//! para o mesmo mundo divergem no dia em que uma delas ganhar uma peça.*
//!
//! ⚠️ **O oráculo é a ÁRVORE da instância depois do passe**, e nunca «o passe correu»: um gate que
//! contasse materializações ficaria verde sobre um passe que põe a peça no pai errado.

use crate::instance_docs::OwnedDocs;
use crate::instance_sync::{MasterEcho, sync_instances};
use ph2d_ecs::{ChildOf, Children, Entity, MasterRoot, Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

pub(super) fn pass(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    echo: &mut MasterEcho,
) -> usize {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    sync_instances(
        sim,
        r,
        &PhysicsBridge::new(),
        echo,
        &mut OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
}

pub(super) fn instantiate(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    master: Entity,
) -> Entity {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    crate::instantiate::instantiate_master(
        sim,
        r,
        master,
        None,
        &mut OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou")
}

/// Uma receita de uma peça, e uma instância dela.
pub(super) fn scene() -> (SimWorld, ph2d_ecs::scene::ComponentRegistry, Entity, Entity) {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Badge"), MasterRoot))
        .id();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Box"),
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ChildOf(master),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let inst = instantiate(&mut sim, &r, master);
    (sim, r, master, inst)
}

pub(super) fn names(sim: &SimWorld, root: Entity) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root
            && let Some(n) = sim.world().get::<Name>(e)
        {
            out.push(n.0.clone());
        }
        if let Some(kids) = sim.world().get::<Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    out.sort();
    out
}

/// ⭐⭐⭐ **Acrescentar uma peça ao mestre MATERIALIZA-A em todas as cópias** — a promessa da tabela
/// do doc 04 §2.6, que nada cumpria.
///
/// ⛔ Medido por sonda em 2026-08-27: `a_inst tem 0 filho(s) depois do passe`. O laço de valores
/// percorre **pares**, e uma peça que só existe do lado do mestre não forma par nenhum — ela é
/// invisível para ele por construção. Para o artista: *«acrescentei uma peça ao componente e as
/// cópias não mudaram»*.
///
/// ⚠️ E os **dois** lados: a peça aparece **e** traz o valor do mestre no MESMO passe. Materializar
/// sem sincronizar deixaria a cópia com o valor do momento, até alguém tocar no mestre outra vez.
///
/// (Mutação: não chamar o `reconcile` no `sync_instances` ⇒ RED na ausência.)
#[test]
fn a_piece_added_to_the_master_materialises_in_every_copy() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    assert_eq!(names(&sim, inst), vec!["Box".to_string()]);

    // O gesto: o artista acrescenta uma peça à receita.
    sim.world_mut().spawn((
        Transform::from_translation(ph2d_core::Vec2::new(0.0, 2.0)),
        Name::new("Label"),
        ph2d_render::Sprite::atlas(0, [0.5, 0.2], [0.25, 0.5, 0.75, 1.0]),
        ChildOf(master),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    assert!(pass(&mut sim, &r, &mut echo) > 0, "o passe nao fez nada");
    assert_eq!(
        names(&sim, inst),
        vec!["Box".to_string(), "Label".to_string()],
        "a copia nao recebeu a peca nova"
    );
    // ⭐ E ela chega com o VALOR do mestre, no mesmo passe.
    let label = {
        let mut found = None;
        for e in sim
            .world()
            .get::<Children>(inst)
            .map(|c| c.iter().copied().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            if sim.world().get::<Name>(e).is_some_and(|n| n.0 == "Label") {
                found = Some(e);
            }
        }
        found.expect("a peca nova")
    };
    assert_eq!(
        sim.world()
            .get::<ph2d_render::Sprite>(label)
            .expect("sprite")
            .tint,
        [0.25, 0.5, 0.75, 1.0],
        "a peca nova chegou sem o valor da receita"
    );
    // ⚠️ E o passe assenta: a forma é um ponto fixo como os valores.
    assert_eq!(pass(&mut sim, &r, &mut echo), 0, "o passe nao assentou");
}

/// ⭐⭐ **E apagar uma peça do mestre TIRA-A das cópias** — a outra metade, e ela não pode ir
/// sozinha: acrescentar sem remover deixa na cena um objeto que o artista apagou da biblioteca.
///
/// (Mutação: apagar o laço das que SOBRAM ⇒ RED.)
#[test]
fn a_piece_deleted_in_the_master_leaves_every_copy() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    assert_eq!(names(&sim, inst), vec!["Box".to_string()]);

    let box_piece = sim
        .world()
        .get::<Children>(master)
        .and_then(|c| c.iter().next().copied())
        .expect("a peca do mestre");
    sim.world_mut().entity_mut(box_piece).despawn();
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    assert!(pass(&mut sim, &r, &mut echo) > 0, "o passe nao fez nada");
    assert!(
        names(&sim, inst).is_empty(),
        "a copia ficou com uma peca que a receita ja' nao tem: {:?}",
        names(&sim, inst)
    );
    assert_eq!(pass(&mut sim, &r, &mut echo), 0, "o passe nao assentou");
}

/// ⛔⛔ **O que o ARTISTA pendurou numa cópia NÃO é uma peça a mais** — ele nunca veio do mestre,
/// logo apagá-lo seria apagar trabalho que ninguém pediu.
///
/// ⚠️ É a fronteira que separa *«a forma segue a receita»* de *«a receita é dona de tudo o que está
/// aqui dentro»*. O sinal é o elo: só o que a receita deu é que a receita tira.
///
/// (Mutação: tratar uma entidade sem `InstanceOf` como sobra ⇒ RED.)
#[test]
fn what_the_artist_hung_on_a_copy_is_not_a_leftover_piece() {
    let (mut sim, r, _master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let mine = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Mine"), ChildOf(inst)))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());

    pass(&mut sim, &r, &mut echo);
    assert!(
        sim.world().get_entity(mine).is_ok(),
        "o passe apagou o que o artista pendurou na copia"
    );
}

/// ⛔ **Uma instância cujo MESTRE inteiro desapareceu fica em paz** — é a lei que já existia
/// (`a_dangling_link_is_left_alone`), e este passe não pode passar por cima dela.
///
/// ⚠️ A diferença é o SUJEITO: mestre presente e peça ausente é uma peça a mais; mestre ausente é
/// uma instância órfã, e apagá-la seria o passe a comer a cena por causa de um `Delete`.
#[test]
fn an_instance_whose_master_is_gone_keeps_its_pieces() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    for e in [master]
        .into_iter()
        .chain(
            sim.world()
                .get::<Children>(master)
                .map(|c| c.iter().copied().collect::<Vec<_>>())
                .unwrap_or_default(),
        )
        .collect::<Vec<_>>()
    {
        if let Ok(em) = sim.world_mut().get_entity_mut(e) {
            em.despawn();
        }
    }
    assert_eq!(pass(&mut sim, &r, &mut echo), 0, "o passe mexeu numa orfa");
    assert_eq!(
        names(&sim, inst),
        vec!["Box".to_string()],
        "a instancia orfa perdeu as pecas dela"
    );
}

/// ⭐⭐ **E uma peça acrescentada FUNDO aterra debaixo do pai DELA** — não na raiz da cópia.
///
/// ⛔⛔ **Este gate existe porque a mutação do irmão SOBREVIVEU.** A fixtura dele é plana (a peça
/// nova é filha da raiz), então *«pôr no pai certo»* e *«pôr na raiz»* dão o mesmo resultado — e a
/// mutação que troca um pelo outro passava. *Uma fixtura de um nível não pode medir de que nível a
/// peça é.*
///
/// (Mutação: usar `root` como `host` em vez de `have[parent_sid]` ⇒ RED.)
#[test]
fn a_piece_added_deep_lands_under_its_own_parent() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let box_master = sim
        .world()
        .get::<Children>(master)
        .and_then(|c| c.iter().next().copied())
        .expect("a peca do mestre");
    // O gesto: uma peça NETA — filha da peça, não da raiz.
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Pip"),
        ph2d_render::Sprite::atlas(0, [0.2, 0.2], [1.0; 4]),
        ChildOf(box_master),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    assert!(pass(&mut sim, &r, &mut echo) > 0, "o passe nao fez nada");

    let box_inst = sim
        .world()
        .get::<Children>(inst)
        .and_then(|c| c.iter().next().copied())
        .expect("a peca da copia");
    let under_box: Vec<String> = sim
        .world()
        .get::<Children>(box_inst)
        .map(|c| {
            c.iter()
                .filter_map(|&e| sim.world().get::<Name>(e).map(|n| n.0.clone()))
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(
        under_box,
        vec!["Pip".to_string()],
        "a peca neta nao aterrou debaixo do pai dela — a arvore da copia deixou de ser a do mestre"
    );
}

/// ⭐⭐⭐ **A EXCEPÇÃO sobrevive a apagar a peça no mestre, e VOLTA A PEGAR quando ela volta**
/// (ADR-0164 / F5.3 — *Overrides sem alvo*).
///
/// ⛔⛔ **Foi a F5.1 que criou este buraco**, e a sonda mediu-o:
///
/// ```text
/// depois da excepcao:                      overrides=1
/// depois de apagar a peca do mestre:       overrides=1  pecas na copia=[]
/// depois do undo no mestre:  tint da copia = [1,1,1,1]   ← a excepcao era [0.9,…]
/// ```
///
/// Antes da F5.1 ninguém despawnava a peça, então a excepção vivia no componente dela e o
/// re-encontro era automático. Com a peça a morrer, ficava **a chave sem o valor**: a cópia perdia
/// a excepção *e* ficava **surda à receita para sempre**, porque o passe salta o que a instância
/// possui.
///
/// ⚠️ **E é por isso que os bytes aqui NÃO contradizem a refutação da F4.4** (*«guardar bytes cria
/// duas fontes para o mesmo número»*): a peça órfã não existe, logo não há segunda fonte — há a
/// única. *A premissa da refutação era «a peça é uma entidade real», e a F5.1 tornou-a destruível.*
///
/// (Mutação: não chamar o `entomb` ⇒ RED na cor; não chamar o `exhume` ⇒ RED na cor.)
#[test]
fn an_override_outlives_its_piece_and_binds_again_when_it_returns() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let mine = [0.9, 0.1, 0.1, 1.0];
    let box_inst = sim
        .world()
        .get::<Children>(inst)
        .and_then(|c| c.iter().next().copied())
        .expect("peca da copia");
    let mut spr = sim
        .world()
        .get::<ph2d_render::Sprite>(box_inst)
        .copied()
        .expect("sprite");
    spr.tint = mine;
    sim.world_mut().entity_mut(box_inst).insert(spr);
    pass(&mut sim, &r, &mut echo);
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::ObjectInstance>(inst)
            .map_or(0, |o| o.overrides.len()),
        1,
        "a excepcao nao foi capturada — a fixtura nao contem o fenomeno"
    );

    // O gesto: o artista apaga a peça NO MESTRE.
    let box_master = sim
        .world()
        .get::<Children>(master)
        .and_then(|c| c.iter().next().copied())
        .expect("peca do mestre");
    let sid = sim
        .world()
        .get::<ph2d_ecs::StableId>(box_master)
        .expect("id")
        .0;
    let master_sprite = sim
        .world()
        .get::<ph2d_render::Sprite>(box_master)
        .copied()
        .expect("sprite do mestre");
    sim.world_mut().entity_mut(box_master).despawn();
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    pass(&mut sim, &r, &mut echo);

    assert!(names(&sim, inst).is_empty(), "a peca ficou na copia");
    let o = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(inst)
        .cloned()
        .expect("a raiz da instancia");
    assert_eq!(o.overrides.len(), 0, "a chave ficou a apontar para o nada");
    assert_eq!(
        o.orphans.len(),
        1,
        "a excepcao foi perdida em vez de guardada — o artista perde trabalho por um Delete"
    );

    // O Ctrl+Z no mestre: a peça volta com o MESMO `StableId`.
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Box"),
        master_sprite,
        ChildOf(master),
        ph2d_ecs::StableId(sid),
    ));
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    pass(&mut sim, &r, &mut echo);

    let back = sim
        .world()
        .get::<Children>(inst)
        .and_then(|c| c.iter().next().copied())
        .expect("a peca voltou");
    assert_eq!(
        sim.world()
            .get::<ph2d_render::Sprite>(back)
            .expect("sprite")
            .tint,
        mine,
        "a excepcao nao voltou a pegar — a copia ficou com o valor do mestre"
    );
    let o = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(inst)
        .cloned()
        .expect("a raiz");
    assert_eq!(o.orphans.len(), 0, "o orfao ficou la' depois de repor");
    assert_eq!(
        o.overrides.len(),
        1,
        "a excepcao nao voltou a ser uma excepcao"
    );
    assert_eq!(pass(&mut sim, &r, &mut echo), 0, "o passe nao assentou");
}

/// ⛔ **E o mestre continua sem alcançar a peça reposta** — é o que ser uma excepção quer dizer.
///
/// ⚠️ Sem esta metade, «voltar a pegar» podia significar *«a chave voltou»* com a cópia a obedecer
/// à receita na mesma — e o gate acima passaria, porque no instante em que ele mede os dois valores
/// coincidem por outra razão.
#[test]
fn the_restored_override_still_wins_against_the_master() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let mine = [0.9, 0.1, 0.1, 1.0];
    let box_inst = sim
        .world()
        .get::<Children>(inst)
        .and_then(|c| c.iter().next().copied())
        .expect("peca da copia");
    let mut spr = sim
        .world()
        .get::<ph2d_render::Sprite>(box_inst)
        .copied()
        .expect("sprite");
    spr.tint = mine;
    sim.world_mut().entity_mut(box_inst).insert(spr);
    pass(&mut sim, &r, &mut echo);

    let box_master = sim
        .world()
        .get::<Children>(master)
        .and_then(|c| c.iter().next().copied())
        .expect("peca do mestre");
    let sid = sim
        .world()
        .get::<ph2d_ecs::StableId>(box_master)
        .expect("id")
        .0;
    let mut master_sprite = sim
        .world()
        .get::<ph2d_render::Sprite>(box_master)
        .copied()
        .expect("sprite do mestre");
    sim.world_mut().entity_mut(box_master).despawn();
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    pass(&mut sim, &r, &mut echo);
    // A peça volta, e o artista muda a COR dela no mestre a seguir.
    master_sprite.tint = [0.0, 0.0, 1.0, 1.0];
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Box"),
        master_sprite,
        ChildOf(master),
        ph2d_ecs::StableId(sid),
    ));
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);

    let back = sim
        .world()
        .get::<Children>(inst)
        .and_then(|c| c.iter().next().copied())
        .expect("a peca voltou");
    assert_eq!(
        sim.world()
            .get::<ph2d_render::Sprite>(back)
            .expect("sprite")
            .tint,
        mine,
        "o mestre atropelou a excepcao reposta — ela voltou como chave e nao como lei"
    );
}

/// ⭐⭐⭐ **A excepção SEM ALVO sabe de que peça era** (F5, critério 3).
///
/// # ⚠️ Porque o NOME se guarda, e porque isso NÃO contradiz a refutação da F4.4
///
/// A refutação diz que guardar um valor cria **duas fontes** para o mesmo facto — e ela vale
/// **enquanto a peça existe**. Uma peça órfã **não existe**: o mestre apagou-a e a F5.1 tirou-a da
/// cópia a seguir. ⇒ não há segunda fonte, há a **única**. *É literalmente o mesmo argumento que
/// já justificou guardar os BYTES ali ao lado* — o nome cai na mesma categoria, e é o único sítio
/// onde ele pode ser lido depois.
///
/// ⚠️ **A janela em que ele se lê é estreita, e é por isso que o `entomb` o faz:** naquele
/// instante a peça da instância ainda está viva (o `despawn` vem a seguir) e o `Name` dela é o
/// **mesmo do mestre** (o passe propaga-o; só a RAIZ é dona do dela). Um segundo depois não há
/// onde o ir buscar.
///
/// **Mutação que deve sangrar:** o `entomb` gravar um nome vazio.
#[test]
fn an_orphan_override_remembers_the_name_of_the_piece_that_died() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);

    // A excepção do artista na peça da cópia.
    let mine = sim
        .world()
        .get::<Children>(inst)
        .and_then(|c| c.iter().next().copied())
        .expect("peca da copia");
    let mut spr = sim
        .world()
        .get::<ph2d_render::Sprite>(mine)
        .copied()
        .expect("sprite");
    spr.tint = [0.9, 0.2, 0.2, 1.0];
    sim.world_mut().entity_mut(mine).insert(spr);
    pass(&mut sim, &r, &mut echo);

    // O gesto: o artista apaga a peça NO MESTRE.
    let box_master = sim
        .world()
        .get::<Children>(master)
        .and_then(|c| c.iter().next().copied())
        .expect("peca do mestre");
    sim.world_mut().entity_mut(box_master).despawn();
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    pass(&mut sim, &r, &mut echo);

    let o = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(inst)
        .cloned()
        .expect("a raiz da instancia");
    let orphan = o.orphans.values().next().expect("uma excepcao sem alvo");
    assert_eq!(
        orphan.piece_name, "Box",
        "a excepcao nao sabe de que peca era — o painel so' pode dizer «ha' N», nunca «quais»"
    );
}
