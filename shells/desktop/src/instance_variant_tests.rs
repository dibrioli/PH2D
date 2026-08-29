//! Os gates da VARIANTE e da troca (ADR-0164 / F5, critério 2).
//!
//! ⚠️ **O oráculo é o VALOR que a peça tem depois do passe**, nunca «o re-key correu»: um gate que
//! contasse chaves traduzidas ficaria verde sobre um mapa que traduz para a peça errada.

use crate::instance_docs::OwnedDocs;
use crate::instance_sync::{MasterEcho, sync_instances};
use ph2d_ecs::{ChildOf, Children, Entity, MasterRoot, Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;

use super::{SwapRefusal, piece_map, swap};

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

fn pass(sim: &mut SimWorld, r: &ph2d_ecs::scene::ComponentRegistry, echo: &mut MasterEcho) {
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
    );
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

/// A cópia vira VARIANTE: ela fica mestre **sem** perder o elo à base.
fn make_variant(sim: &mut SimWorld, copy: Entity) -> Entity {
    sim.world_mut().entity_mut(copy).insert(MasterRoot);
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    copy
}

fn sid(sim: &SimWorld, e: Entity) -> u64 {
    sim.world().get::<ph2d_ecs::StableId>(e).expect("sid").0
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

fn set_tint(sim: &mut SimWorld, e: Entity, c: [f32; 4]) {
    sim.world_mut()
        .get_mut::<ph2d_render::Sprite>(e)
        .expect("sprite")
        .tint = c;
}

fn tint(sim: &SimWorld, e: Entity) -> [f32; 4] {
    sim.world()
        .get::<ph2d_render::Sprite>(e)
        .expect("sprite")
        .tint
}

/// A base com duas peças, e a variante dela com uma excepção própria.
///
/// ⚠️ A base tem **duas** peças de propósito: com uma só, «traduziu a peça certa» e «traduziu
/// alguma peça» dão o mesmo resultado. *Uma fixtura de um elemento não pode medir um mapa.*
fn world() -> (
    SimWorld,
    ph2d_ecs::scene::ComponentRegistry,
    Entity,
    Entity,
    MasterEcho,
) {
    let mut sim = SimWorld::new();
    let r = reg();
    let base = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Base"), MasterRoot))
        .id();
    for n in ["Head", "Body"] {
        sim.world_mut().spawn((
            Transform::IDENTITY,
            Name::new(n),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            ChildOf(base),
        ));
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let copy = instantiate(&mut sim, &r, base);
    // ⚠️⚠️ **O ECO sai daqui com a fixtura, e não é conveniência.** Ele é a memória de *quem se
    // mexeu*, e um eco novo cai na regra do 1.º encontro — *«sem eco não há atribuição: o mestre
    // ganha»*. Uma fixtura que o deita fora apaga a excepção da variante **no passe seguinte**, e
    // o gate mede então um mundo que o app nunca produz. Custou dois vermelhos a descobrir.
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let variant = make_variant(&mut sim, copy);
    (sim, r, base, variant, echo)
}

/// ⭐⭐⭐ **A CADEIA: a base alcança a variante E as instâncias da variante, num passe.**
///
/// Medido por sonda em 2026-08-27 **antes** de existir uma linha deste módulo — é o achado que diz
/// que a variante derivada não custa mecanismo nenhum: ela é um mestre que também é instância, e o
/// sync já procura toda entidade com elo para um mestre vivo.
///
/// (Mutação: fazer `live_instances` exigir `MasterRoot` ausente na raiz ⇒ RED.)
#[test]
fn editing_the_base_reaches_the_variant_and_its_instances_in_one_pass() {
    let (mut sim, r, base, variant, mut echo) = world();
    let inst = instantiate(&mut sim, &r, variant);
    pass(&mut sim, &r, &mut echo);

    let head = kid(&sim, base, "Head");
    sim.world_mut()
        .get_mut::<ph2d_render::Sprite>(head)
        .expect("sprite")
        .size = [3.0, 3.0];
    pass(&mut sim, &r, &mut echo);

    for (who, e) in [("variante", variant), ("instancia da variante", inst)] {
        let got = sim
            .world()
            .get::<ph2d_render::Sprite>(kid(&sim, e, "Head"))
            .expect("sprite")
            .size;
        assert_eq!(
            got,
            [3.0, 3.0],
            "a edicao da base nao chegou a' {who} NUM passe"
        );
    }
}

/// ⭐⭐ **O mapa sai dos ELOS, não dos nomes** — a propriedade que separa este re-key do
/// `ByName` do Unity.
///
/// ⚠️ Renomear **tudo** para o mesmo nome é o controlo: com nomes iguais, qualquer heurística por
/// nome escolhe ao acaso, e este mapa continua exacto. *Um gate cujo oráculo é o nome não pode
/// distinguir os dois.*
#[test]
fn the_map_is_read_from_the_links_never_from_the_names() {
    let (mut sim, _r, base, variant, _echo) = world();
    let (h, b) = (kid(&sim, base, "Head"), kid(&sim, base, "Body"));
    let (vh, vb) = (kid(&sim, variant, "Head"), kid(&sim, variant, "Body"));
    for e in [h, b, vh, vb] {
        sim.world_mut().entity_mut(e).insert(Name::new("x"));
    }
    let (base_id, variant_id) = (sid(&sim, base), sid(&sim, variant));
    let (h_id, b_id) = (sid(&sim, h), sid(&sim, b));
    let (vh_id, vb_id) = (sid(&sim, vh), sid(&sim, vb));
    let map = piece_map(&mut sim, base_id, variant_id).expect("aparentados");
    assert_eq!(map.get(&h_id), Some(&vh_id));
    assert_eq!(map.get(&b_id), Some(&vb_id));
}

/// ⭐⭐⭐ **A troca base → variante PRESERVA a excepção do artista** — o critério 2 da F5.
///
/// A instância tem a cabeça verde (excepção dela); a variante tem o corpo vermelho (excepção da
/// variante). Depois da troca a instância tem **as duas**: a dela porque foi re-chaveada, a da
/// variante porque é o novo mestre.
///
/// (Mutação: `swap` não traduzir `inst.overrides` ⇒ a cabeça verde é comida pelo mestre ⇒ RED.)
#[test]
fn swapping_to_a_variant_keeps_the_exception_the_artist_made() {
    let (mut sim, r, base, variant, mut echo) = world();
    // A variante pinta o CORPO de vermelho.
    let vbody = kid(&sim, variant, "Body");
    set_tint(&mut sim, vbody, [1.0, 0.0, 0.0, 1.0]);
    // Uma instância da BASE, com a CABEÇA verde.
    let inst = instantiate(&mut sim, &r, base);
    pass(&mut sim, &r, &mut echo);
    let ihead = kid(&sim, inst, "Head");
    set_tint(&mut sim, ihead, [0.0, 1.0, 0.0, 1.0]);
    pass(&mut sim, &r, &mut echo);

    let variant_id = sid(&sim, variant);
    let report = swap(&mut sim, &mut echo, inst, variant_id).expect("aparentados");
    assert_eq!(
        report.overrides_kept, 1,
        "a excepcao nao sobreviveu a' troca"
    );
    pass(&mut sim, &r, &mut echo);

    assert_eq!(
        tint(&sim, kid(&sim, inst, "Head")),
        [0.0, 1.0, 0.0, 1.0],
        "a excepcao do artista foi comida pela troca"
    );
    assert_eq!(
        tint(&sim, kid(&sim, inst, "Body")),
        [1.0, 0.0, 0.0, 1.0],
        "a instancia nao passou a seguir a variante"
    );
}

/// ⭐⭐ **E a volta:** trocar de novo para a base devolve o corpo ao valor da base e **mantém** a
/// cabeça do artista. É a mesma tradução, ao contrário.
#[test]
fn swapping_back_to_the_base_keeps_it_too() {
    let (mut sim, r, base, variant, mut echo) = world();
    let vbody = kid(&sim, variant, "Body");
    set_tint(&mut sim, vbody, [1.0, 0.0, 0.0, 1.0]);
    let inst = instantiate(&mut sim, &r, variant);
    pass(&mut sim, &r, &mut echo);
    let ihead = kid(&sim, inst, "Head");
    set_tint(&mut sim, ihead, [0.0, 1.0, 0.0, 1.0]);
    pass(&mut sim, &r, &mut echo);

    let base_id = sid(&sim, base);
    swap(&mut sim, &mut echo, inst, base_id).expect("aparentados");
    pass(&mut sim, &r, &mut echo);

    assert_eq!(
        tint(&sim, kid(&sim, inst, "Head")),
        [0.0, 1.0, 0.0, 1.0],
        "a excepcao nao sobreviveu a' volta"
    );
    assert_eq!(
        tint(&sim, kid(&sim, inst, "Body")),
        [1.0; 4],
        "o corpo nao voltou ao valor da base"
    );
}

/// ⭐⭐⭐ **Uma peça que o mestre novo NÃO tem vira órfã — e volta a pegar na troca de volta.**
///
/// É o critério 3 da F5 alcançado pelo gesto do critério 2: o `swap` deixa o elo da peça a apontar
/// para o mestre velho, o passe estrutural lê isso como *«não é do mestre»*, sepulta a excepção e
/// apaga-a. Trocar de volta materializa-a e **exuma** os bytes.
///
/// ⚠️⚠️ **A direcção é a variante → base, e a 1.ª versão deste gate escolheu a impossível.**
/// Ela tirava uma peça da variante — e o passe estrutural da F5.1 **põe-na de volta**, porque uma
/// variante é uma instância da base e a regra é *«o que o mestre tem, a cópia tem»*. É a mesma
/// fronteira do Unity (uma *Prefab Variant* não apaga um objeto herdado, só o desactiva). ⇒ a peça
/// que um dos lados não tem é a que a **variante acrescentou**, e é a base que não a tem.
#[test]
fn a_piece_the_target_lacks_is_entombed_and_comes_back() {
    let (mut sim, r, base, variant, mut echo) = world();
    // A variante ACRESCENTA uma peça — o que uma variante derivada de facto pode fazer.
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Cape"),
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ChildOf(variant),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    let inst = instantiate(&mut sim, &r, variant);
    pass(&mut sim, &r, &mut echo);
    let cape = kid(&sim, inst, "Cape");
    set_tint(&mut sim, cape, [0.0, 0.0, 1.0, 1.0]);
    pass(&mut sim, &r, &mut echo);

    let (base_id, variant_id) = (sid(&sim, base), sid(&sim, variant));
    let report = swap(&mut sim, &mut echo, inst, base_id).expect("aparentados");
    assert_eq!(report.dropped, 1, "a capa devia ficar sem imagem na base");
    pass(&mut sim, &r, &mut echo);
    let orphans = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(inst)
        .map_or(0, |o| o.orphans.len());
    assert!(orphans > 0, "a excepcao da peca apagada nao foi sepultada");

    swap(&mut sim, &mut echo, inst, variant_id).expect("aparentados");
    pass(&mut sim, &r, &mut echo);
    let cape = kid(&sim, inst, "Cape");
    assert_eq!(
        tint(&sim, cape),
        [0.0, 0.0, 1.0, 1.0],
        "a excepcao nao voltou a pegar quando a peca voltou"
    );
}

/// ⛔ **Uma variante NÃO pode perder uma peça da base** — medido, e é a cerca que torna o mapa
/// total numa direcção.
///
/// Apagar a peça na variante é *«a cópia tirou uma peça que o mestre tem»*, e o passe estrutural
/// da F5.1 materializa-a de volta no quadro seguinte. É a regra do Unity dita por outro caminho:
/// lá uma *Prefab Variant* não apaga um objeto herdado. ⚠️ Sem este gate a nota vive num
/// comentário, e o próximo gate a escolher esta direcção descobre-a por um vermelho que parece
/// vir de outro sítio — que foi o que aconteceu.
#[test]
fn a_variant_cannot_lose_a_piece_of_its_base() {
    let (mut sim, r, _base, variant, mut echo) = world();
    let vbody = kid(&sim, variant, "Body");
    sim.world_mut().entity_mut(vbody).despawn();
    pass(&mut sim, &r, &mut echo);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| kid(&sim, variant, "Body")))
            .is_ok(),
        "a peca da base nao voltou a' variante — a regra da F5.1 mudou"
    );
}

/// ⭐ **Duas variantes IRMÃS traduzem-se pela base comum** — o caso 2 do §3.7, que aqui é a mesma
/// composição em vez de um mecanismo próprio.
#[test]
fn two_sibling_variants_map_through_their_common_base() {
    let (mut sim, r, base, red, mut echo) = world();
    let blue_copy = instantiate(&mut sim, &r, base);
    let blue = make_variant(&mut sim, blue_copy);
    pass(&mut sim, &r, &mut echo);

    let (red_id, blue_id) = (sid(&sim, red), sid(&sim, blue));
    let red_head = sid(&sim, kid(&sim, red, "Head"));
    let blue_head = sid(&sim, kid(&sim, blue, "Head"));
    let map = piece_map(&mut sim, red_id, blue_id).expect("irmas sao aparentadas");
    assert_eq!(
        map.get(&red_head),
        Some(&blue_head),
        "a traducao entre irmas nao passou pela base"
    );
    assert_eq!(map.len(), 3, "a raiz e as duas pecas: {map:?}");
}

/// ⛔ **Sem antepassado comum não há mapa** — e a recusa é a decisão, não uma falta.
///
/// ⚠️ Os dois mestres têm as peças com os **mesmos nomes**: é o caso em que uma heurística por
/// nome acertaria e ninguém saberia dizer se por sorte. *Nunca automático* (doc 04 §2.6).
#[test]
fn an_unrelated_master_is_refused_and_never_matched_by_name() {
    let (mut sim, _r, _base, variant, _echo) = world();
    let other = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Other"), MasterRoot))
        .id();
    for n in ["Head", "Body"] {
        sim.world_mut().spawn((
            Transform::IDENTITY,
            Name::new(n),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            ChildOf(other),
        ));
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    let (variant_id, other_id) = (sid(&sim, variant), sid(&sim, other));
    assert!(
        piece_map(&mut sim, variant_id, other_id).is_none(),
        "um mestre nao aparentado ganhou um mapa — so' pode ter vindo dos nomes"
    );
}

/// As três recusas do gesto falam, e cada uma pela sua razão.
#[test]
fn the_swap_refuses_out_loud() {
    let (mut sim, r, base, variant, mut echo) = world();
    let inst = instantiate(&mut sim, &r, base);
    let loose = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Loose")))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());

    let (variant_id, loose_id, base_id) = (sid(&sim, variant), sid(&sim, loose), sid(&sim, base));
    assert_eq!(
        swap(&mut sim, &mut echo, loose, variant_id),
        Err(SwapRefusal::NotAnInstance)
    );
    assert_eq!(
        swap(&mut sim, &mut echo, inst, loose_id),
        Err(SwapRefusal::NotAMaster)
    );
    assert_eq!(
        swap(&mut sim, &mut echo, inst, base_id),
        Err(SwapRefusal::Already)
    );
}
