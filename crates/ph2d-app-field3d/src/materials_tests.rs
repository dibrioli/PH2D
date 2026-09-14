//! Os gates da tabela de materiais. Ver [`super`].

use super::{Table, surface_of};
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_ecs::FieldMaterial;

/// Duas esferas bem separadas, em união — cada uma tem uma região da tela que é só dela.
fn two_balls() -> FieldDoc {
    let leaf = |x: f32| Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.25 }),
        mods: Vec::new(),
        verb: None,
    };
    FieldDoc::new(
        vec![
            leaf(-0.4),
            leaf(0.4),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(2),
    )
    .expect("duas esferas")
}

fn a_world() -> (ph2d_ecs::SimWorld, bevy_ecs::entity::Entity) {
    let mut sim = ph2d_ecs::SimWorld::new();
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &two_balls(), "peça");
    (sim, root)
}

fn leaves_of(
    world: &bevy_ecs::world::World,
    root: bevy_ecs::entity::Entity,
) -> Vec<bevy_ecs::entity::Entity> {
    world
        .get::<bevy_ecs::hierarchy::Children>(root)
        .expect("a raiz tem filhos")
        .iter()
        .copied()
        .collect()
}

/// ⭐⭐⭐ **O DEFAULT do componente É o da nodedef** — e este gate é a ponte entre as duas crates.
///
/// ⛔⛔ **A `ph2d-field-ecs` NÃO depende da `ph2d-material`**, de propósito: o material é um
/// componente de cena e a lei do OpenPBR é uma crate sem dependências nenhumas. O preço dessa
/// fronteira é o `Default` estar escrito duas vezes — e é **este** gate que impede as duas cópias de
/// divergirem. *Uma constante escrita em dois sítios ainda não é uma constante.*
///
/// **Mutação que deve sangrar:** mexer num dos dois `Default`.
#[test]
fn the_default_material_is_the_one_the_nodedef_declares() {
    let nosso = surface_of(FieldMaterial::default());
    let nodedef = ph2d_material::OpenPbr::default().prepare();
    // A superfície não é comparável campo a campo de fora, então compara-se o que ela FAZ: a luz
    // que devolve. ⚠️ Três normais, para um acerto por acaso numa delas não passar.
    for n in [[0.0, 0.0, 1.0], [0.6, 0.0, 0.8], [0.0, 0.8, 0.6]] {
        let v = [0.0, 0.0, 1.0];
        let l = [0.3, 0.6, 0.74];
        assert_eq!(
            nosso.direct(n, v, l, [3.0; 3]),
            nodedef.direct(n, v, l, [3.0; 3]),
            "o `FieldMaterial::default()` deixou de ser o material da nodedef (normal {n:?})"
        );
    }
}

/// ⭐⭐⭐ **CADA FOLHA LEVA O SEU MATERIAL** — e a tabela sabe qual é qual.
///
/// ⚠️ **O ponto é posto sobre cada esfera**, que é como ele chega do traçado: no meio de uma união
/// qualquer das duas é plausível, e um gate que apontasse ali passaria com a resposta errada.
#[test]
fn each_leaf_wears_its_own_material() {
    let (mut sim, root) = a_world();
    let folhas = leaves_of(sim.world(), root);
    assert_eq!(folhas.len(), 2);
    // A da esquerda fica VERMELHA; a da direita continua no material de omissão.
    ph2d_field_ecs::set_param(
        sim.world_mut(),
        folhas[0],
        ph2d_field::Param::Material(1),
        0.0,
    )
    .expect("o verde");
    ph2d_field_ecs::set_param(
        sim.world_mut(),
        folhas[0],
        ph2d_field::Param::Material(2),
        0.0,
    )
    .expect("o azul");

    let t = Table::build(sim.world(), root, 0.8, 480.0);
    assert_eq!(t.surfaces.len(), 2, "uma superfície por folha");
    let owners = t.owners.as_ref().expect("duas folhas pedem um dono");
    // O pólo de cada esfera: só ela existe ali.
    assert_eq!(owners.at([-0.4, 0.0, 0.25]), Some(0), "a esquerda");
    assert_eq!(owners.at([0.4, 0.0, 0.25]), Some(1), "a direita");
    // E as duas superfícies devolvem luz DIFERENTE — é isso que o artista vê.
    let (n, v, l) = ([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.3, 0.6, 0.74]);
    assert_ne!(
        t.surfaces[0].direct(n, v, l, [3.0; 3]),
        t.surfaces[1].direct(n, v, l, [3.0; 3]),
        "as duas folhas têm materiais diferentes e devolvem a MESMA luz — a tabela não os separou"
    );
}

/// ⭐⭐ **Arrastar um número NÃO recompila a geometria** — é a razão de a tabela ter duas metades.
///
/// **Mutação que deve sangrar:** fazer o `refresh_authored` devolver `false` sempre (o slider de cor
/// deixa de ter efeito), ou `true` sempre (o quadro re-traça para sempre).
#[test]
fn changing_a_number_refreshes_the_surfaces_and_not_the_geometry() {
    let (mut sim, root) = a_world();
    let folhas = leaves_of(sim.world(), root);
    let mut t = Table::build(sim.world(), root, 0.8, 480.0);
    let antes = t.surfaces[0].direct([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.3, 0.6, 0.74], [3.0; 3]);

    assert!(
        !t.refresh_authored(sim.world(), root),
        "nada mudou e a tabela disse que sim — o quadro re-traçaria para sempre"
    );
    ph2d_field_ecs::set_param(
        sim.world_mut(),
        folhas[0],
        ph2d_field::Param::Material(3),
        0.9,
    )
    .expect("a rugosidade");
    assert!(
        t.refresh_authored(sim.world(), root),
        "a rugosidade mudou e a tabela não deu por isso — o slider fica sem efeito"
    );
    assert_ne!(
        t.surfaces[0].direct([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.3, 0.6, 0.74], [3.0; 3]),
        antes,
        "a superfície não seguiu o número"
    );
    // ⭐ **E a geometria não foi tocada:** o dono continua a responder o mesmo.
    assert_eq!(
        t.owners.as_ref().expect("dois donos").at([-0.4, 0.0, 0.25]),
        Some(0)
    );
}

/// Uma peça de UMA folha não constrói dono nenhum — não perguntar é o custo zero.
#[test]
fn a_single_leaf_asks_nobody_who_it_belongs_to() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: 0.3 },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("uma esfera");
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
    let t = Table::build(sim.world(), root, 0.8, 480.0);
    assert!(
        t.owners.is_none(),
        "uma peça de uma folha construiu um resolvedor de donos — é trabalho por uma pergunta que \
         não existe"
    );
    assert_eq!(t.surfaces.len(), 1);
}
