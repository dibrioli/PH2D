//! **«Ignore up to» do Ray Sensor e as queixas da tag** (auditoria GEF 07 C8 e C2, dono 15/11): o nome
//! escrito, o conta-gotas no palco e o que o cartão diz. Aqui e não na `ph2d-app-components` (tecto da crate).

use ph2d_app_components::{conta_gotas, ray_inspector, ray_topo};
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, StableId};
use ph2d_editor_core::action_bus::{ActionBus, ComponentEdit, EditorAction};
use ph2d_editor_core::ray_edits::{RayFieldEdit, RayQueixa};
use ph2d_physics_ecs::{BodyKind, PlatformPlayer, RaySensor, RigidBody, SeeFilter};
use ph2d_tags::TagTree;

/// `Truck` ⊃ `Hero` (personagem) ⊃ o raio; `Crate` solta, com o mesmo nível.
fn cena() -> (SimWorld, [Entity; 4]) {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let corpo = RigidBody {
        kind: BodyKind::Static,
    };
    let truck = w.spawn((Name::new("Truck"), corpo, StableId(21))).id();
    let hero = w
        .spawn((
            Name::new("Hero"),
            corpo,
            StableId(22),
            PlatformPlayer::default(),
            ChildOf(truck),
        ))
        .id();
    let raio = w
        .spawn((Name::new("Ray Sensor"), RaySensor::default(), StableId(23), ChildOf(hero)))
        .id();
    let caixa = w.spawn((Name::new("Crate"), corpo, StableId(24))).id();
    (sim, [truck, hero, raio, caixa])
}

fn info(sim: &SimWorld, raio: Entity, tree: &TagTree) -> ph2d_editor_core::ray_edits::InspectorRayInfo {
    ray_inspector::build_ray_info(sim.world(), tree, raio.to_bits(), 1, true, &[]).expect("o raio")
}

fn escreve(sim: &mut SimWorld, raio: Entity, edit: RayFieldEdit) -> bool {
    ray_inspector::apply_ray_edit(sim.world_mut(), raio.to_bits(), &edit)
}

/// ⭐⭐ **O nome nos dois sentidos**: vazio = automático (o fantasma diz «Hero», o personagem); «Truck»
/// guarda o `StableId` dele; um nome de ninguém não escreve; «Crate» (fora da cadeia) guarda e queixa-se.
///
/// **Mutações que sangram:** `resolve` guardar 0 para um nome · a leitura não ver a cadeia (`fora` sempre falso).
#[test]
fn o_ignore_up_to_le_e_escreve_pelo_nome() {
    let (mut sim, [truck, _, raio, _]) = cena();
    let t = TagTree::new();
    let i = info(&sim, raio, &t);
    assert_eq!((i.ignore_up_to.as_str(), i.ignore_up_to_auto.as_str()), ("", "Hero"));
    assert!(escreve(&mut sim, raio, RayFieldEdit::IgnoreUpTo(" Truck ".into())));
    let r = *sim.world().get::<RaySensor>(raio).expect("raio");
    assert_eq!(r.ignore_up_to, 21, "o StableId do Truck");
    let i = info(&sim, raio, &t);
    assert_eq!((i.ignore_up_to.as_str(), i.ignore_up_to_off), ("Truck", false));
    assert_ne!(i.queixa(), Some(RayQueixa::TopoFora), "o topo certo não se queixa");
    assert!(!escreve(&mut sim, raio, RayFieldEdit::IgnoreUpTo("Nobody".into())));
    assert_eq!(sim.world().get::<RaySensor>(raio).map(|r| r.ignore_up_to), Some(21));
    assert!(escreve(&mut sim, raio, RayFieldEdit::IgnoreUpTo("Crate".into())));
    let i = info(&sim, raio, &t);
    assert!(i.ignore_up_to_off, "a Crate não está acima do raio");
    assert_eq!(i.queixa(), Some(RayQueixa::TopoFora));
    assert!(escreve(&mut sim, raio, RayFieldEdit::IgnoreUpTo(String::new())));
    assert_eq!(sim.world().get::<RaySensor>(raio).map(|r| r.ignore_up_to), Some(0));
    let _ = truck;
}

/// ⭐⭐ **O conta-gotas**: armar não toca no documento; um clique fora da cadeia (a Crate) não escreve e
/// CONTINUA armado; um clique no Truck escreve «Truck» pela porta do Inspector e desarma; o `Esc` desarma.
///
/// **Mutações que sangram:** desarmar no clique errado · aceitar o clicado sem subir à cadeia.
#[test]
fn o_conta_gotas_do_ignore_up_to_escolhe_um_pai() {
    let (mut sim, [truck, _, raio, caixa]) = cena();
    let mut bus = ActionBus::default();
    assert!(!conta_gotas::escolher(sim.world_mut(), &mut bus, &[truck.to_bits()]), "desarmado");
    assert!(!escreve(&mut sim, raio, RayFieldEdit::PickUpTo(true)), "armar não muda o documento");
    assert!(info(&sim, raio, &TagTree::new()).up_to_pick_armed);
    assert!(conta_gotas::escolher(sim.world_mut(), &mut bus, &[caixa.to_bits()]));
    assert!(conta_gotas::armado(sim.world()), "a Crate não está acima: continua armado");
    assert_eq!(bus.drain().count(), 0);
    assert!(conta_gotas::escolher(sim.world_mut(), &mut bus, &[caixa.to_bits(), truck.to_bits()]));
    assert!(!conta_gotas::armado(sim.world()));
    let esperado = EditorAction::InspectorComponentEdit {
        entity_bits: raio.to_bits(),
        edit: ComponentEdit::Ray(RayFieldEdit::IgnoreUpTo("Truck".into())),
    };
    assert_eq!(bus.drain().collect::<Vec<_>>(), vec![esperado]);
    ray_topo::arma(sim.world_mut(), Some(raio.to_bits()));
    assert!(conta_gotas::desarma(sim.world_mut()) && !conta_gotas::armado(sim.world()), "o Esc");
}

/// ⭐ **C2 — a tag por escolher e a tag apagada falam antes da camada.** «Only tag» sem tag: «Pick a tag»;
/// com uma tag que a árvore já não tem: «This tag no longer exists».
///
/// **Mutação que sangra:** `see_tag_exists` sempre verdadeiro.
#[test]
fn a_tag_do_filtro_por_escolher_ou_apagada_queixa_se() {
    let (mut sim, [_, _, raio, _]) = cena();
    let mut tree = TagTree::new();
    let enemy = tree.create("Enemy").expect("tag");
    if let Some(mut r) = sim.world_mut().get_mut::<RaySensor>(raio) {
        r.only_see = SeeFilter::OnlyTag;
    }
    assert_eq!(info(&sim, raio, &tree).queixa(), Some(RayQueixa::EscolhaATag));
    assert!(escreve(&mut sim, raio, RayFieldEdit::SeeTag(enemy.0)));
    assert_ne!(info(&sim, raio, &tree).queixa(), Some(RayQueixa::TagApagada));
    assert_eq!(info(&sim, raio, &TagTree::new()).queixa(), Some(RayQueixa::TagApagada));
}
