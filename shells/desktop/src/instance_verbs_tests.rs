//! Os gates dos três verbos que fecham a tabela (ADR-0164 / F4.5).
//!
//! ⚠️ **O oráculo é o que o ARTISTA vê depois do gesto** — o que está na tela, o que a receita
//! passou a ter, o que as outras cópias receberam. Um gate que contasse chamadas ficaria verde
//! sobre um verbo que faz a coisa errada.

use super::VerbRefusal;
use crate::instance_smoke::{spawn_master, spawn_ragdoll_scene};
use crate::instance_sync::{MasterEcho, sync_instances};
use ph2d_ecs::{Children, Entity, InstanceOf, MasterRoot, Name, SimWorld, Transform, Visibility};
use ph2d_physics_ecs::PhysicsBridge;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
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
pub(super) fn ragdoll(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
) -> (Entity, Vec<Entity>) {
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

pub(super) fn piece(sim: &SimWorld, root: Entity, name: &str) -> Entity {
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

pub(super) fn tint(sim: &SimWorld, e: Entity) -> [f32; 4] {
    sim.world()
        .get::<ph2d_render::Sprite>(e)
        .expect("sprite")
        .tint
}

pub(super) fn paint(sim: &mut SimWorld, e: Entity, c: [f32; 4]) {
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
                &mut None,
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

/// ⭐⭐⭐ **A SELEÇÃO SEGUE A CÓPIA** (report do Enio, 2026-08-30).
///
/// O *Make Component* marca o objecto escolhido como RECEITA — e uma receita **não se desenha**.
/// Deixar a selecção nela punha o artista a mexer num objecto invisível: o gesto seguinte
/// (apagar, o olho, o Inspector) acertava na receita, e ele via *«ao deletar o objeto do canvas, o
/// do painel assets foi deletado»* e *«mudei o hide no objeto da cena e o objeto do painel foi
/// modificado»*.
///
/// **Mutação que deve sangrar:** escrever `master` em vez de `instance` no `select_out`.
#[test]
fn making_a_component_leaves_the_selection_on_the_copy_not_on_the_recipe() {
    let mut sim = SimWorld::new();
    let r = reg();
    let rig = plain_rig(&mut sim);
    let mut echo = MasterEcho::default();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let mut select = None;
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let ok = super::drain(
        super::Verb::Make,
        &mut sim,
        &r,
        &mut echo,
        rig.to_bits(),
        &mut toasts,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        [0.0, 0.0],
        &mut select,
    );
    assert!(ok, "o verbo recusou");
    let picked = select.expect("o verbo nao disse para onde a selecao vai");
    assert_ne!(
        picked,
        rig.to_bits(),
        "a selecao ficou na RECEITA — o artista continua a mexer no que nao ve"
    );
    let e = Entity::from_bits(picked);
    assert!(
        sim.world().get::<MasterRoot>(e).is_none(),
        "a selecao caiu num MasterRoot, que e' precisamente a receita"
    );
    assert!(
        sim.world().get::<InstanceOf>(e).is_some(),
        "a selecao tem de cair na COPIA, que e' uma instancia"
    );
}

/// ⭐⭐⭐ **`Instantiate` numa CÓPIA põe outra cópia** — report do Enio, 2026-08-31.
///
/// # ⛔⛔ Duas decisões deliberadas deste ficheiro desfaziam-se uma à outra
///
/// O *Make Prefab* move a selecção para a **cópia** de propósito (é o que o artista vê e continua a
/// editar — ver o doc do `select_out`), e o *Instantiate* pedia a **receita**. ⇒ o gesto seguinte da
/// fila recusava no caminho normal, com *«Not a prefab — pick the prefab row»*, e as duas linhas
/// lêem-se quase igual na Hierarquia (`Casa` e `Casa (1)`).
///
/// ⚠️ **O oráculo é a CONTAGEM de instâncias da receita**, e não «o verbo devolveu `true`»: um
/// verbo que criasse um objecto solto — sem `InstanceOf` — também devolveria `true`.
///
/// (Mutação: `master_subject` devolver sempre `clicked` ⇒ RED.)
#[test]
fn instantiate_on_a_copy_places_another_copy_of_its_master() {
    let mut sim = SimWorld::new();
    let r = reg();
    let mut echo = MasterEcho::default();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let master = spawn_master(&mut sim);
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let master_id = sim
        .world()
        .get::<ph2d_ecs::StableId>(master)
        .expect("stable id")
        .0;

    // A 1.ª cópia, pela receita — o estado em que o *Make Prefab* deixa o artista.
    let mut select_out = None;
    assert!(place(
        &mut sim,
        &r,
        &mut echo,
        &mut toasts,
        master,
        &mut select_out
    ));
    let copy = Entity::from_bits(select_out.expect("a copia nova"));
    let before = super::instances_of(&mut sim, master_id);

    // ⭐ E agora o gesto do artista: *Instantiate* **na cópia** em que ele ficou.
    let mut select_out = None;
    assert!(
        place(&mut sim, &r, &mut echo, &mut toasts, copy, &mut select_out),
        "o verbo recusou na copia — e' o defeito do report"
    );
    assert_eq!(
        super::instances_of(&mut sim, master_id),
        before + 1,
        "a copia nova nao ficou ligada a' MESMA receita"
    );
    let born = Entity::from_bits(select_out.expect("a 2.a copia"));
    assert_eq!(
        sim.world().get::<InstanceOf>(born).map(|l| l.master),
        Some(master_id),
        "a copia nasceu ligada a outra coisa"
    );
}

/// ⛔ **E uma linha que não é receita NEM cópia continua a recusar** — a cerca mudou de sítio, não
/// caiu.
///
/// ⚠️ Sem esta metade, um `master_subject` que devolvesse a primeira receita do mundo passaria no
/// gate acima: *uma cura só se prova com o caso em que ela NÃO pode agir*.
#[test]
fn instantiate_on_a_stranger_still_refuses() {
    let mut sim = SimWorld::new();
    let r = reg();
    let mut echo = MasterEcho::default();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let _master = spawn_master(&mut sim);
    let stranger = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Stranger")))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let mut select_out = None;
    assert!(
        !place(
            &mut sim,
            &r,
            &mut echo,
            &mut toasts,
            stranger,
            &mut select_out
        ),
        "um objecto que nao e' receita nem copia foi instanciado"
    );
    assert!(select_out.is_none());
}

/// O dreno do *Instantiate*, com os documentos vazios.
fn place(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    echo: &mut MasterEcho,
    toasts: &mut ph2d_editor::ToastQueue,
    entity: Entity,
    select_out: &mut Option<u64>,
) -> bool {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    super::drain(
        super::Verb::Place,
        sim,
        r,
        echo,
        entity.to_bits(),
        toasts,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        [1.5, 0.0],
        select_out,
    )
}

/// O dreno do *Instantiate* com o degrau ESCOLHIDO — o irmão do [`place`], para as réguas da
/// cascata, que precisam de um passo que elas próprias dizem.
fn place_with_step(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    echo: &mut MasterEcho,
    toasts: &mut ph2d_editor::ToastQueue,
    entity: Entity,
    step: [f32; 2],
) -> Option<Entity> {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut out = None;
    super::drain(
        super::Verb::Place,
        sim,
        r,
        echo,
        entity.to_bits(),
        toasts,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        step,
        &mut out,
    );
    out.map(Entity::from_bits)
}

fn pose(sim: &SimWorld, e: Entity) -> [f32; 2] {
    let t = sim.world().get::<Transform>(e).expect("pose").translation;
    [t.x, t.y]
}

/// ⭐⭐⭐ **NENHUMA CÓPIA ATERRA EM CIMA DE OUTRA** — a lei da cascata, medida no motor GERAL.
///
/// # ⛔⛔ Porque este gate nasce agora, e o que ele quase perdeu (F4.6c, 2026-09-07)
///
/// A cascata do dreno geral ([`super::cascade`]) declara, no próprio doc, *«a lei é a que o verbo
/// VETORIAL já tinha»* — e a **única régua dela vivia do outro lado**, em
/// `vec_component_edit_tests`, a exercitar o motor `VecInstance`. Apagar aquele motor sem esta
/// porta deixaria a lei **herdada por comentário e provada por ninguém**.
///
/// ⚠️ *Um censo de fatia tem de contar as RÉGUAS e não só as features* — esta linha já se queimou
/// duas vezes a contar verbos a menos, e a terceira teria sido contar gates a menos.
///
/// # O que ele afirma, e porque são DUAS metades
///
/// A **aritmética** (cada cópia a um múltiplo do degrau) e a **pilha** (duas cópias nunca
/// coincidem). A segunda diz o que o artista vê, e sobrevive a uma mudança de fórmula; a primeira
/// morde quando o degrau deixa de multiplicar. *Uma sozinha fica verde sobre metade do defeito.*
///
/// ⚠️ **O *Make* deixa a 1.ª cópia NO LUGAR da selecção**, e é por isso que a contagem dela é zero
/// — ver [`super::cascade`]. O gate parte daí, que é o estado que o gesto real produz.
#[test]
fn no_two_copies_of_a_recipe_ever_land_on_each_other() {
    let mut sim = SimWorld::new();
    let r = reg();
    let mut echo = MasterEcho::default();
    let mut toasts = ph2d_editor::ToastQueue::new();
    let rig = plain_rig(&mut sim);
    let (master, first) = make(&mut sim, &r, rig).expect("o gesto");

    let step = [3.0_f32, -3.0];
    let mut copias = vec![first];
    for _ in 0..3 {
        copias.push(
            place_with_step(&mut sim, &r, &mut echo, &mut toasts, master, step)
                .expect("o Instantiate recusou uma copia"),
        );
    }

    let base = pose(&sim, copias[0]);
    for (i, &c) in copias.iter().enumerate() {
        let p = pose(&sim, c);
        let quer = [base[0] + step[0] * i as f32, base[1] + step[1] * i as f32];
        assert!(
            (p[0] - quer[0]).abs() < 1e-4 && (p[1] - quer[1]).abs() < 1e-4,
            "a copia {i} nasceu em {p:?}, e o degrau manda {quer:?}"
        );
    }
    // A metade da PILHA, dita sem a aritmética acima: duas cópias nunca coincidem.
    for a in 0..copias.len() {
        for b in (a + 1)..copias.len() {
            let (pa, pb) = (pose(&sim, copias[a]), pose(&sim, copias[b]));
            assert!(
                (pa[0] - pb[0]).abs() > 1e-4 || (pa[1] - pb[1]).abs() > 1e-4,
                "as copias {a} e {b} pousaram no mesmo sitio ({pa:?})"
            );
        }
    }
}

/// ⭐⭐ **A CASCATA CONTA SÓ AS CÓPIAS DESTA RECEITA.**
///
/// ⚠️ Sem isto, colocar uma cópia de um botão empurraria a próxima cópia de um ícone — a folga
/// passaria a depender do que mais existe no documento, e o artista não teria como a prever.
///
/// ⚠️ **A fixtura precisa das DUAS receitas povoadas**: com uma só, *«conta as minhas»* e *«conta
/// todas»* devolvem o mesmo número e o gate ficaria verde sobre a contagem errada. É a mesma lei
/// de fixtura que o `a_shape_born_into_a_populated_scene_lands_at_the_end` paga do outro lado.
#[test]
fn the_step_of_one_recipe_does_not_count_the_copies_of_another() {
    let mut sim = SimWorld::new();
    let r = reg();
    let mut echo = MasterEcho::default();
    let mut toasts = ph2d_editor::ToastQueue::new();

    let rig_a = plain_rig(&mut sim);
    let (a_master, a_first) = make(&mut sim, &r, rig_a).expect("receita A");
    let rig_b = plain_rig(&mut sim);
    let (b_master, _) = make(&mut sim, &r, rig_b).expect("receita B");
    // A receita B leva DUAS cópias além da que o *Make* deixou.
    for _ in 0..2 {
        place_with_step(&mut sim, &r, &mut echo, &mut toasts, b_master, [1.0, -1.0])
            .expect("o Instantiate de B");
    }

    let step = [3.0_f32, -3.0];
    let base = pose(&sim, a_first);
    let a_second = place_with_step(&mut sim, &r, &mut echo, &mut toasts, a_master, step)
        .expect("o Instantiate de A");
    let p = pose(&sim, a_second);
    assert!(
        (p[0] - (base[0] + step[0])).abs() < 1e-4,
        "a 2a copia de A nasceu no degrau {:.2}, e nao no 1o — as copias de B entraram na conta",
        (p[0] - base[0]) / step[0]
    );
}
