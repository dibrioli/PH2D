//! Os gates do emparelhamento SEM PARENTESCO (ADR-0164 / F5, o último critério).
//!
//! ⚠️ **A fixtura tem a ordem dos irmãos TROCADA entre os dois mestres**, e isso não é decoração:
//! é o único arranjo em que os dois modos dão mapas **diferentes**. Com `Body` e `Wheel` na mesma
//! ordem dos dois lados, um gate de `ByName` ficaria verde sobre uma implementação que só sabe
//! contar índices — *duas leis que concordam na fixtura são uma lei só para quem mede*.

use ph2d_ecs::{ChildOf, Children, Entity, MasterRoot, Name, SimWorld, Transform};
use std::collections::BTreeMap;

use super::{Rematch, WhenUnrelated, rematch};

fn sid(sim: &SimWorld, e: Entity) -> u64 {
    sim.world().get::<ph2d_ecs::StableId>(e).expect("sid").0
}

fn index(sim: &mut SimWorld) -> BTreeMap<u64, Entity> {
    let mut q = sim.world_mut().query::<(Entity, &ph2d_ecs::StableId)>();
    q.iter(sim.world()).map(|(e, s)| (s.0, e)).collect()
}

fn kid(sim: &SimWorld, root: Entity, name: &str) -> Entity {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
            return e;
        }
        if let Some(kids) = sim.world().get::<Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    panic!("nao achei a peca {name:?}");
}

/// Uma receita com as peças nomeadas na ordem dada — a ORDEM de nascimento é a ordem de irmão
/// (`sibling_key` cai no índice da entidade sem `SiblingOrder`), e é ela que o modo por hierarquia
/// lê.
fn recipe(sim: &mut SimWorld, name: &str, pieces: &[&str]) -> Entity {
    let root = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(name), MasterRoot))
        .id();
    for p in pieces {
        sim.world_mut().spawn((
            Transform::IDENTITY,
            Name::new(*p),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            ChildOf(root),
        ));
    }
    root
}

fn settle(sim: &mut SimWorld) {
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
}

/// **Duas receitas SEM antepassado comum, com a ordem dos irmãos ao contrário.**
///
/// ```text
/// Car   → [0] Body   [1] Wheel
/// Truck → [0] Wheel  [1] Body
/// ```
///
/// ⇒ por NOME, `Body → Body`; por HIERARQUIA, `Body → Wheel`. As duas respostas são legítimas e
/// nenhuma contém a outra — é o par que o Unity chama `ObjectMatchMode`.
fn two_recipes() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let car = recipe(&mut sim, "Car", &["Body", "Wheel"]);
    let truck = recipe(&mut sim, "Truck", &["Wheel", "Body"]);
    settle(&mut sim);
    (sim, car, truck)
}

fn map_of(sim: &mut SimWorld, from: Entity, to: Entity, how: WhenUnrelated) -> Rematch {
    let (a, b) = (sid(sim, from), sid(sim, to));
    let by_id = index(sim);
    rematch(sim, &by_id, a, b, how).expect("o modo emparelha")
}

/// ⛔ **O caminho de omissão é a RECUSA** — a lei que separa este ficheiro de uma heurística
/// automática (plano F5: *«nunca automático»*).
///
/// ⚠️ E a metade que importa é a segunda: as duas receitas de facto **não** têm mapa derivado, senão
/// este ficheiro nem seria alcançado e o gate estaria a medir outra coisa.
///
/// (Mutação: `Refuse` a cair no braço do `CarryNothing` ⇒ RED.)
#[test]
fn without_kinship_there_is_no_derived_map_and_the_default_is_a_refusal() {
    let (mut sim, car, truck) = two_recipes();
    let (a, b) = (sid(&sim, car), sid(&sim, truck));
    assert!(
        crate::instance_variant::piece_map(&mut sim, a, b).is_none(),
        "as duas receitas nao podiam ter mapa derivado"
    );
    let by_id = index(&mut sim);
    assert!(
        rematch(&sim, &by_id, a, b, WhenUnrelated::Refuse).is_none(),
        "o modo de omissao tem de recusar"
    );
}

/// **«Não leves nada» é sobre as PEÇAS — a raiz é o objecto, e ela emparelha nos três modos.**
///
/// ⚠️ A razão não é de gosto: *a chave da raiz não tem sepultador* (ver o cabeçalho do módulo). Uma
/// chave de raiz sem imagem não fica viva nem sepultada — fica **invisível**, a bloquear a receita
/// nova para sempre.
///
/// (Mutação: `CarryNothing` a cair no `pair_up` ⇒ RED, o mapa vem com três entradas.)
#[test]
fn carrying_nothing_pairs_no_piece_and_the_root_is_not_a_piece() {
    let (mut sim, car, truck) = two_recipes();
    let (car_id, truck_id) = (sid(&sim, car), sid(&sim, truck));
    let r = map_of(&mut sim, car, truck, WhenUnrelated::CarryNothing);
    assert_eq!(
        r.map,
        BTreeMap::from([(car_id, truck_id)]),
        "so' a raiz, e nenhuma peca"
    );
    assert_eq!(r.ambiguous, 0);
}

/// ⭐⭐ **Por NOME: a peça encontra a homónima, mesmo com a ordem trocada.**
///
/// (Mutação: `step_of` a devolver o índice também no `ByName` ⇒ RED, porque a fixtura tem as
/// ordens ao contrário.)
#[test]
fn by_name_pairs_the_homonym_even_when_the_order_differs() {
    let (mut sim, car, truck) = two_recipes();
    let want = [
        (kid(&sim, car, "Body"), kid(&sim, truck, "Body")),
        (kid(&sim, car, "Wheel"), kid(&sim, truck, "Wheel")),
    ];
    let (car_id, truck_id) = (sid(&sim, car), sid(&sim, truck));
    let r = map_of(&mut sim, car, truck, WhenUnrelated::ByName);
    for (from, to) in want {
        let (f, t) = (sid(&sim, from), sid(&sim, to));
        assert_eq!(r.map.get(&f), Some(&t), "a peca nao achou a homonima");
    }
    assert_eq!(
        r.map.get(&car_id),
        Some(&truck_id),
        "a raiz emparelha com a raiz"
    );
    assert_eq!(r.map.len(), 3, "so' as duas pecas e a raiz");
}

/// ⭐⭐ **Por HIERARQUIA: a peça encontra a que ocupa o mesmo lugar, mesmo com outro nome.**
///
/// É o mapa **cruzado** do gate anterior sobre a mesma fixtura — a prova de que os dois modos são
/// duas leis, e não uma com dois nomes.
///
/// (Mutação: `step_of` a devolver o nome também no `ByHierarchy` ⇒ RED.)
#[test]
fn by_hierarchy_pairs_the_same_place_even_when_the_name_differs() {
    let (mut sim, car, truck) = two_recipes();
    let want = [
        (kid(&sim, car, "Body"), kid(&sim, truck, "Wheel")),
        (kid(&sim, car, "Wheel"), kid(&sim, truck, "Body")),
    ];
    let r = map_of(&mut sim, car, truck, WhenUnrelated::ByHierarchy);
    for (from, to) in want {
        let (f, t) = (sid(&sim, from), sid(&sim, to));
        assert_eq!(r.map.get(&f), Some(&t), "a peca nao achou o mesmo LUGAR");
    }
}

/// ⛔ **Um nome repetido não emparelha nada, e o número é DITO.**
///
/// ⚠️ Escolher *«o de `StableId` menor»* seria aplicar a excepção do artista ao braço que calhou de
/// nascer primeiro — a heurística que o plano proíbe, com cara de determinismo.
///
/// (Mutação: aceitar `from_ids[0]` sem olhar o comprimento ⇒ RED.)
#[test]
fn a_name_that_appears_twice_pairs_nothing_and_says_how_many() {
    let mut sim = SimWorld::new();
    let a = recipe(&mut sim, "A", &["Arm", "Arm", "Leg"]);
    let b = recipe(&mut sim, "B", &["Arm", "Leg"]);
    settle(&mut sim);
    let leg = sid(&sim, kid(&sim, a, "Leg"));
    let r = map_of(&mut sim, a, b, WhenUnrelated::ByName);
    assert_eq!(r.ambiguous, 1, "o `Arm` repetido tinha de ser contado");
    assert!(
        r.map.contains_key(&leg),
        "a peca sem ambiguidade continua a emparelhar"
    );
    assert_eq!(r.map.len(), 2, "a `Leg` e a raiz, e mais nada");
}

/// ⭐⭐⭐ **A LEI DO PAI: uma peça cujo pai não emparelhou também não emparelha.**
///
/// ⚠️ O `Hand` é **único dos dois lados** — uma implementação sem esta lei emparelha-o e deixa a
/// mão pendurada num pai que aquele mestre não tem, porque o passe estrutural **não muda peças de
/// pai**. O defeito seria estável e mudo.
///
/// (Mutação: aceitar todo candidato sem olhar o pai ⇒ RED.)
#[test]
fn a_piece_whose_parent_did_not_pair_does_not_pair_either() {
    let mut sim = SimWorld::new();
    let a = recipe(&mut sim, "A", &["Arm", "Arm"]);
    let b = recipe(&mut sim, "B", &["Arm"]);
    // Uma `Hand` debaixo do primeiro `Arm` de cada lado — nome único, pai ambíguo do lado de `A`.
    for root in [a, b] {
        let arm = kid(&sim, root, "Arm");
        sim.world_mut().spawn((
            Transform::IDENTITY,
            Name::new("Hand"),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            ChildOf(arm),
        ));
    }
    settle(&mut sim);
    let hand = sid(&sim, kid(&sim, a, "Hand"));
    let r = map_of(&mut sim, a, b, WhenUnrelated::ByName);
    assert!(
        !r.map.contains_key(&hand),
        "a mao emparelhou por cima de um pai que nao emparelhou"
    );
}

/// ⛔ **Sem nome não há caminho de nomes** — e a sub-árvore inteira sai com ela.
///
/// (Mutação: `step_of` a devolver `Some(String::new())` para quem não tem `Name` ⇒ RED.)
#[test]
fn a_nameless_piece_is_not_addressable_by_name_and_neither_are_its_children() {
    let mut sim = SimWorld::new();
    let a = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("A"), MasterRoot))
        .id();
    let b = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("B"), MasterRoot))
        .id();
    let mut anon = Vec::new();
    for root in [a, b] {
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, ChildOf(root)))
            .id();
        sim.world_mut().spawn((
            Transform::IDENTITY,
            Name::new("Deep"),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            ChildOf(e),
        ));
        anon.push(e);
    }
    settle(&mut sim);
    let (anon_a, deep_a) = (sid(&sim, anon[0]), sid(&sim, kid(&sim, a, "Deep")));
    let r = map_of(&mut sim, a, b, WhenUnrelated::ByName);
    assert!(
        !r.map.contains_key(&anon_a),
        "a anonima emparelhou por nome"
    );
    assert!(
        !r.map.contains_key(&deep_a),
        "a filha da anonima emparelhou sobre um caminho que nao existe"
    );
    // O controlo: por HIERARQUIA as duas emparelham, porque ali o degrau é o índice.
    let r = map_of(&mut sim, a, b, WhenUnrelated::ByHierarchy);
    assert!(r.map.contains_key(&anon_a) && r.map.contains_key(&deep_a));
}

/// ⚠️ **Injectivo** — duas peças da cópia a apontar para a mesma peça do mestre seriam dois pares a
/// ler a mesma origem, e o passe estrutural veria a segunda como *«já existe»*.
#[test]
fn the_map_never_sends_two_pieces_to_the_same_one() {
    let mut sim = SimWorld::new();
    let a = recipe(&mut sim, "A", &["X", "Y", "Z"]);
    let b = recipe(&mut sim, "B", &["X", "Y"]);
    settle(&mut sim);
    for how in [WhenUnrelated::ByName, WhenUnrelated::ByHierarchy] {
        let r = map_of(&mut sim, a, b, how);
        let mut seen: Vec<u64> = r.map.values().copied().collect();
        seen.sort_unstable();
        let n = seen.len();
        seen.dedup();
        assert_eq!(seen.len(), n, "o mapa {how:?} nao e' injectivo");
    }
}

// ── De ponta a ponta, pela porta do `swap` ───────────────────────────────────────────────────
//
// ⚠️ **O oráculo é o que a CÓPIA tem depois dos passes**, e nunca «o mapa saiu certo»: um mapa
// exacto que o `swap` não aplicasse deixaria os gates de cima todos verdes.

use crate::instance_docs::OwnedDocs;
use crate::instance_sync::{MasterEcho, sync_instances};
use ph2d_core::Vec2;
use ph2d_physics_ecs::PhysicsBridge;

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0; 4];
/// Onde o `Body` do Camião mora — o que a receita NOVA tem a dizer sobre aquela peça.
const TRUCK_BODY_X: f32 = 5.0;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// A FORMA e depois o VALOR, duas vezes — a mesma ordem do quadro, e duas voltas porque uma peça
/// materializada só forma par no laço seguinte.
fn passes(sim: &mut SimWorld, r: &ph2d_ecs::scene::ComponentRegistry, echo: &mut MasterEcho) {
    for _ in 0..2 {
        let (mut sc, mut mp) = crate::instance_docs::empty_docs();
        let mut docs = OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        };
        crate::instance_structure::reconcile(sim, r, &mut docs);
        sync_instances(sim, r, &PhysicsBridge::new(), echo, &mut docs);
    }
}

fn instantiate(
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

fn tint(sim: &SimWorld, e: Entity) -> [f32; 4] {
    sim.world()
        .get::<ph2d_render::Sprite>(e)
        .expect("sprite")
        .tint
}

fn set_tint(sim: &mut SimWorld, e: Entity, c: [f32; 4]) {
    sim.world_mut()
        .get_mut::<ph2d_render::Sprite>(e)
        .expect("sprite")
        .tint = c;
}

/// As duas receitas sem parentesco, **uma cópia do Carro na cena**, e a excepção do artista: o
/// `Body` dela pintado de vermelho.
///
/// ⚠️ O `Body` do Camião mora noutro sítio ([`TRUCK_BODY_X`]) porque a excepção vive no `Sprite` e a
/// prova de que a receita NOVA chegou tem de vir de **outro componente** — a granularidade do
/// override é o componente inteiro, então medir a cor dos dois lados responderia à mesma pergunta
/// duas vezes.
fn scene() -> (
    SimWorld,
    ph2d_ecs::scene::ComponentRegistry,
    Entity,
    Entity,
    MasterEcho,
) {
    let mut sim = SimWorld::new();
    let r = reg();
    let car = recipe(&mut sim, "Car", &["Body", "Wheel"]);
    let truck = recipe(&mut sim, "Truck", &["Wheel", "Body"]);
    let tbody = kid(&sim, truck, "Body");
    sim.world_mut()
        .entity_mut(tbody)
        .insert(Transform::from_translation(Vec2::new(TRUCK_BODY_X, 0.0)));
    settle(&mut sim);
    let copy = instantiate(&mut sim, &r, car);
    let mut echo = MasterEcho::default();
    passes(&mut sim, &r, &mut echo);
    // A excepção do artista, DEPOIS do 1.º passe: sem eco não há atribuição, e o mestre ganharia.
    let body = kid(&sim, copy, "Body");
    set_tint(&mut sim, body, RED);
    passes(&mut sim, &r, &mut echo);
    (sim, r, truck, copy, echo)
}

/// ⭐⭐⭐ **Trocar por uma receita SEM parentesco, emparelhando por nome: a excepção sobrevive E a
/// receita nova chega.**
///
/// As duas metades são precisas. Só a primeira ficaria verde sobre uma troca que não trocou nada;
/// só a segunda, sobre uma troca que deitou fora o trabalho do artista.
///
/// (Mutação: `swap` a não traduzir `inst.overrides` ⇒ o vermelho é comido ⇒ RED.)
#[test]
fn replacing_by_name_keeps_the_exception_and_lets_the_new_recipe_through() {
    let (mut sim, r, truck, copy, mut echo) = scene();
    let truck_id = sid(&sim, truck);
    let report =
        crate::instance_variant::swap(&mut sim, &mut echo, copy, truck_id, WhenUnrelated::ByName)
            .expect("o modo por nome emparelha");
    assert_eq!(report.overrides_kept, 1, "a excepcao nao foi re-chaveada");
    passes(&mut sim, &r, &mut echo);

    let body = kid(&sim, copy, "Body");
    assert_eq!(tint(&sim, body), RED, "a excepcao do artista foi comida");
    let x = sim
        .world()
        .get::<Transform>(body)
        .expect("transform")
        .translation
        .x;
    assert!(
        (x - TRUCK_BODY_X).abs() < 1e-6,
        "a receita nova nao alcancou a peca: x = {x}"
    );
}

/// ⭐⭐ **«Não leves nada» leva mesmo nada — e o que se perde fica NOMEADO na lista de excepções sem
/// alvo**, que é a metade durável do relatório (o cartão do Inspector lê-a).
///
/// (Mutação: `CarryNothing` a emparelhar as peças ⇒ o vermelho sobrevive ⇒ RED.)
#[test]
fn keeping_nothing_drops_the_exception_into_the_unused_list() {
    let (mut sim, r, truck, copy, mut echo) = scene();
    let truck_id = sid(&sim, truck);
    crate::instance_variant::swap(
        &mut sim,
        &mut echo,
        copy,
        truck_id,
        WhenUnrelated::CarryNothing,
    )
    .expect("trocar sem levar nada e' legitimo");
    passes(&mut sim, &r, &mut echo);

    assert_eq!(
        tint(&sim, kid(&sim, copy, "Body")),
        WHITE,
        "a excepcao sobreviveu a uma troca que prometeu nao levar nada"
    );
    let orphans = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(copy)
        .expect("a copia e' uma instancia")
        .orphans
        .clone();
    assert!(
        !orphans.is_empty(),
        "a excepcao evaporou em silencio em vez de ir para a lista"
    );
    assert!(
        orphans.values().any(|o| o.piece_name == "Body"),
        "a lista nao diz de que peca a excepcao era: {orphans:?}"
    );
}

/// ⚠️ **Onde o artista largou o objecto é dele** — a troca muda de que receita a cópia é, e não onde
/// ela está. A pose da raiz nunca foi do mestre (`ROOT_IS_ITS_OWN`), e o modo mais destrutivo dos
/// três é o que mais precisa de o provar.
#[test]
fn the_object_stays_where_the_artist_left_it_after_a_replace() {
    let (mut sim, r, truck, copy, mut echo) = scene();
    let at = Vec2::new(7.0, 3.0);
    sim.world_mut()
        .entity_mut(copy)
        .insert(Transform::from_translation(at));
    passes(&mut sim, &r, &mut echo);
    let truck_id = sid(&sim, truck);
    crate::instance_variant::swap(
        &mut sim,
        &mut echo,
        copy,
        truck_id,
        WhenUnrelated::CarryNothing,
    )
    .expect("trocar");
    passes(&mut sim, &r, &mut echo);
    let got = sim
        .world()
        .get::<Transform>(copy)
        .expect("transform")
        .translation;
    assert!(
        (got.x - at.x).abs() < 1e-6 && (got.y - at.y).abs() < 1e-6,
        "o objecto saltou de sitio: {got:?}"
    );
}

/// ⛔ **Sem um gesto que nomeie o modo, a porta RECUSA** — a lei *«nunca automático»* do plano F5,
/// medida onde o produto a atravessa e não só no construtor do mapa.
#[test]
fn the_door_still_refuses_when_no_mode_was_asked_for() {
    let (mut sim, _r, truck, copy, mut echo) = scene();
    let truck_id = sid(&sim, truck);
    assert_eq!(
        crate::instance_variant::swap(&mut sim, &mut echo, copy, truck_id, WhenUnrelated::Refuse),
        Err(crate::instance_variant::SwapRefusal::Unrelated)
    );
}

/// ⭐⭐⭐ **A excepção sobre o OBJECTO sobrevive à troca — e continua endereçável.**
///
/// ⚠️ Este é o gate do mecanismo que obriga a raiz a emparelhar nos três modos: *a chave da raiz não
/// tem sepultador*. Sem o par raiz→raiz, a chave fica a apontar para a raiz do mestre VELHO e as
/// duas metades quebram de uma vez — o passe deixa de a ver (logo a receita nova come a excepção do
/// artista) e o cartão deixa de a listar (logo não há gesto que lhe chegue). ⛔ *Nem viva, nem
/// sepultada: invisível.*
///
/// (Mutação: tirar o `map.insert(from, to)` do `rematch` ⇒ RED nas duas asserções.)
#[test]
fn an_exception_on_the_object_itself_survives_the_replace_and_stays_addressable() {
    let mut sim = SimWorld::new();
    let r = reg();
    let car = recipe(&mut sim, "Car", &["Body"]);
    let truck = recipe(&mut sim, "Truck", &["Body"]);
    // A raiz de uma receita também pode desenhar — e é aí que a excepção do artista mora.
    for root in [car, truck] {
        sim.world_mut()
            .entity_mut(root)
            .insert(ph2d_render::Sprite::atlas(0, [1.0, 1.0], WHITE));
    }
    settle(&mut sim);
    let copy = instantiate(&mut sim, &r, car);
    let mut echo = MasterEcho::default();
    passes(&mut sim, &r, &mut echo);
    set_tint(&mut sim, copy, RED);
    passes(&mut sim, &r, &mut echo);

    let truck_id = sid(&sim, truck);
    crate::instance_variant::swap(&mut sim, &mut echo, copy, truck_id, WhenUnrelated::ByName)
        .expect("trocar");
    passes(&mut sim, &r, &mut echo);

    assert_eq!(
        tint(&sim, copy),
        RED,
        "a excepcao sobre o proprio objecto foi comida pela receita nova"
    );
    let keys = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(copy)
        .expect("instancia")
        .overrides
        .clone();
    assert!(
        keys.iter().any(|k| k.piece == truck_id),
        "a chave ficou a apontar para a receita VELHA — invisivel ao cartao: {keys:?}"
    );
}
