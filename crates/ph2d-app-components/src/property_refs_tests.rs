//! ⭐ **As referências lêem-se e escrevem-se pelo NOME do alvo** (doc CodeEditor 08 §5, d4): o objecto,
//! a tag, a camada — e o campo de variante de um enum com dados (o alvo do agente, o «Where» da
//! fábrica), sobre o registo do produto.

use crate::component_registry_for_tests::registo;
use ph2d_ecs::scene::property::{self, PropertyError};
use ph2d_ecs::scene::property_ref::RefNames;
use ph2d_ecs::{Name, StableId};
use serde_json::{Value, json};

struct Mundo {
    reg: ph2d_ecs::scene::ComponentRegistry,
    world: bevy_ecs::world::World,
    tree: ph2d_tags::TagTree,
    e: bevy_ecs::entity::Entity,
}

fn mundo() -> Mundo {
    let mut world = bevy_ecs::world::World::new();
    world.spawn((Name::from("Caixa"), StableId(41)));
    world.spawn((Name::from("Bala"), StableId(42)));
    let e = world.spawn((Name::from("Eu"), StableId(43))).id();
    let mut tree = ph2d_tags::TagTree::new();
    tree.create("Enemy/Flying").expect("tag");
    Mundo {
        reg: registo(),
        world,
        tree,
        e,
    }
}

impl Mundo {
    fn com(&mut self, canonical: &str) -> &mut Self {
        let entry = self.reg.get_by_name(canonical).expect("registado");
        (entry.insert_default.expect("default"))(&mut self.world, self.e).expect("insere");
        self
    }
    fn names(&self) -> RefNames<'_> {
        RefNames {
            tags: Some(&self.tree),
        }
    }
    fn get(&self, n: &str) -> Value {
        property::get(&self.reg, &self.world, self.e, n, self.names())
            .unwrap_or_else(|e| panic!("{n}: {e}"))
    }
    fn set(&mut self, n: &str, v: Value) -> Result<(), PropertyError> {
        let tree = std::mem::take(&mut self.tree);
        let r = property::set(
            &self.reg,
            &mut self.world,
            self.e,
            n,
            v,
            RefNames { tags: Some(&tree) },
        );
        self.tree = tree;
        r.map(|_| ())
    }
    fn stored(&self, n: &str) -> Value {
        property::get_stored(&self.reg, &self.world, self.e, n)
            .expect("guardado")
            .0
    }
}

#[test]
fn um_objecto_le_se_e_escreve_se_pelo_nome() {
    let mut m = mundo();
    m.com("ph2d::physics::PhysicsJoint")
        .com("ph2d::ecs::Factory")
        .com("ph2d::ecs::CameraFollow");
    // Pelo hash do nome, pelo `StableId` e pelo próprio nome.
    for (n, guardado) in [
        ("joint.body_a", json!(ph2d_ecs::stable_name_id("Caixa"))),
        ("factory.recipe", json!(41)),
        ("camera_follow.target", json!("Caixa")),
    ] {
        assert_eq!(m.get(n), Value::Null, "{n}: de fábrica ninguém");
        m.set(n, json!("Caixa")).unwrap();
        assert_eq!(m.stored(n), guardado, "{n}: guarda a identidade");
        assert_eq!(m.get(n), json!("Caixa"), "{n}");
        let erro = m.set(n, json!("Caxia")).unwrap_err();
        assert!(
            matches!(&erro, PropertyError::NoTarget { wanted, .. } if wanted == "Caxia"),
            "{n}: {erro}"
        );
        assert_eq!(m.get(n), json!("Caixa"), "{n}: a gralha não muda o campo");
        m.set(n, Value::Null).unwrap();
        assert_eq!(m.get(n), Value::Null, "{n}: nil = ninguém");
        assert!(m.set(n, json!(5)).is_err(), "{n}: um número não é um nome");
    }
}

#[test]
fn uma_tag_le_se_pelo_caminho() {
    let mut m = mundo();
    m.com("ph2d::ecs::Tags").com("ph2d::physics::Collider");
    assert_eq!(m.get("tags"), json!([]));
    m.set("tags", json!(["Enemy/Flying", "enemy"])).unwrap();
    assert_eq!(
        m.get("tags"),
        json!(["Enemy", "Enemy/Flying"]),
        "na ordem da identidade"
    );
    assert!(matches!(
        m.set("tags", json!(["Boss"])),
        Err(PropertyError::NoTarget { .. })
    ));
    let muitas: Vec<Value> = (0..17).map(|_| json!("Enemy")).collect();
    assert!(matches!(
        m.set("tags", Value::Array(muitas)),
        Err(PropertyError::TooMany { max: 16, .. })
    ));
    m.set("tags", json!({})).unwrap();
    assert_eq!(m.get("tags"), json!([]), "a tabela vazia do Luau");
    // O filtro dos sinais é uma linha do colisor: lê-se `nil` (qualquer) e escrevê-lo cria-o.
    assert_eq!(m.get("signal_tag_filter.only_for_tag"), Value::Null);
    m.set("signal_tag_filter.only_for_tag", json!("Enemy"))
        .unwrap();
    assert_eq!(m.get("signal_tag_filter.only_for_tag"), json!("Enemy"));
}

#[test]
fn a_camada_le_se_pelo_nome_do_projecto() {
    let mut m = mundo();
    let camadas = ph2d_ecs::SortingLayers::canonical();
    let (id, nome) = camadas
        .iter()
        .last()
        .map(|(i, n)| (i, n.to_owned()))
        .expect("camadas");
    m.com("ph2d::ecs::SortingLayer");
    m.set("sorting_layer", json!(nome)).unwrap();
    assert_eq!(m.stored("sorting_layer"), json!(id.0));
    assert_eq!(m.get("sorting_layer"), json!(nome));
    assert!(matches!(
        m.set("sorting_layer", json!("Nenhuma")),
        Err(PropertyError::NoTarget { .. })
    ));
}

/// O alvo do agente: «Target» escolhe a variante; cada linha dela lê `nil` noutra variante, e
/// escrevê-la muda o enum. `nil` numa variante que não está em uso não o muda.
#[test]
fn o_campo_de_variante_muda_o_enum_e_le_nil_fora_dele() {
    let mut m = mundo();
    m.com("ph2d::physics::NavAgent");
    assert_eq!(m.get("nav_agent.target"), json!("none"));
    m.set("nav_agent.object", Value::Null).unwrap();
    assert_eq!(
        m.get("nav_agent.target"),
        json!("none"),
        "nil fora da variante não a escolhe"
    );
    m.set("nav_agent.object", json!("Caixa")).unwrap();
    assert_eq!(m.get("nav_agent.target"), json!("object"));
    assert_eq!(m.get("nav_agent.object"), json!("Caixa"));
    assert_eq!(m.get("nav_agent.point"), Value::Null);
    m.set("nav_agent.point", json!([2.0, -1.0])).unwrap();
    assert_eq!(m.get("nav_agent.target"), json!("point"));
    assert_eq!(m.get("nav_agent.object"), Value::Null);
    assert_eq!(m.get("nav_agent.point"), json!([2.0, -1.0]));
    m.set("nav_agent.target", json!("point")).unwrap();
    assert_eq!(
        m.get("nav_agent.point"),
        json!([2.0, -1.0]),
        "escolher a variante em uso guarda o conteúdo"
    );
    m.set("nav_agent.tag", json!("Enemy/Flying")).unwrap();
    assert_eq!(m.get("nav_agent.target"), json!("tag"));
    assert_eq!(m.get("nav_agent.tag"), json!("Enemy/Flying"));
    assert_eq!(
        m.stored("nav_agent.tag"),
        m.stored("nav_agent.target"),
        "o ledger grava o enum inteiro"
    );
}

#[test]
fn o_onde_da_fabrica_le_se_por_linha() {
    let mut m = mundo();
    m.com("ph2d::ecs::Factory");
    assert_eq!(m.get("factory.where"), json!("here"));
    assert_eq!(m.get("factory.spawn_tag"), Value::Null);
    m.set("factory.spawn_tag", json!("Enemy")).unwrap();
    assert_eq!(m.get("factory.where"), json!("at_tag"));
    assert_eq!(
        m.get("factory.pick_at_random"),
        json!(false),
        "o molde: Cycle"
    );
    m.set("factory.pick_at_random", json!(true)).unwrap();
    assert_eq!(
        m.get("factory.spawn_tag"),
        json!("Enemy"),
        "o vizinho na variante fica"
    );
    m.set("factory.area", json!([2.0, 3.0])).unwrap();
    assert_eq!(m.get("factory.where"), json!("area"));
    assert_eq!(m.get("factory.area"), json!([2.0, 3.0]));
    assert_eq!(
        m.stored("factory.area"),
        json!({"Area": {"w": 2.0, "h": 3.0}})
    );
}
