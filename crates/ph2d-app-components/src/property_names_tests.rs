//! ⭐ **O nome de código de um campo É o rótulo do cartão** (doc CodeEditor 08 §3.1, a lei do Godot:
//! `capitalize(name)` = rótulo) e **lê-se e escreve-se sem perder nada** — sobre o registo que o
//! produto monta ([`crate::component_registry_for_tests::registo`]).

use crate::component_registry_for_tests::registo;
use ph2d_ecs::scene::property::{self, PropertyError};
use std::collections::BTreeSet;

/// O `snake` de um rótulo inglês: «Cull Mask» → `cull_mask`; a unidade do fim não entra no nome
/// («Regen / s» → `regen`, «Dodge (0–1)» → `dodge`).
pub(crate) fn snake(label: &str) -> String {
    let label = label.split(" / ").next().unwrap_or(label);
    let label = match label.rfind(" (") {
        Some(i) if label.ends_with(')') => &label[..i],
        _ => label,
    };
    label
        .replace('%', " percent")
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join("_")
}

#[test]
fn o_nome_de_codigo_e_o_rotulo() {
    let mut codes = BTreeSet::new();
    let mut vistos = 0;
    for d in ph2d_component_desc::all().filter(|d| !d.code.is_empty()) {
        assert!(codes.insert(d.code), "dois componentes chamam-se '{}'", d.code);
        assert_eq!(d.code, snake(ph2d_i18n::tr(d.display_key)), "{}", d.canonical_name);
        let mut nomes = BTreeSet::new();
        for f in d.fields {
            assert!(!f.name.is_empty(), "{} campo {} sem nome", d.code, f.field_id);
            assert!(nomes.insert(f.name), "{}.{} repetido", d.code, f.name);
            assert_eq!(f.name, snake(ph2d_i18n::tr(f.label_key)), "{}.{}", d.code, f.name);
            vistos += 1;
        }
    }
    assert!(vistos >= 32, "a Vida e a Câmera, no mínimo: {vistos}");
}

/// Todo campo nomeado de todo componente com nome: `get` acha a chave serde, e `set` do MESMO
/// valor deixa os bytes gravados iguais.
#[test]
fn ler_e_escrever_o_mesmo_valor_nao_muda_um_byte() {
    let reg = registo();
    let mut world = bevy_ecs::world::World::new();
    let mut campos = 0;
    for entry in reg.iter() {
        let (Some(d), Some(insert_default)) = (entry.desc, entry.insert_default) else {
            continue;
        };
        if d.code.is_empty() {
            continue;
        }
        let e = world.spawn_empty().id();
        insert_default(&mut world, e).expect("default");
        let antes = (entry.serialize)(&world, e).expect("bytes");
        for f in d.fields {
            let nome = format!("{}.{}", d.code, f.name);
            let v = property::get(&reg, &world, e, &nome).unwrap_or_else(|err| panic!("{nome}: {err}"));
            property::set(&reg, &mut world, e, &nome, v).unwrap_or_else(|err| panic!("{nome}: {err}"));
            campos += 1;
        }
        assert_eq!(antes, (entry.serialize)(&world, e).expect("bytes"), "{}", d.code);
    }
    assert!(campos >= 32, "{campos}");
}

#[test]
fn escrever_um_campo_muda_so_esse() {
    let reg = registo();
    let mut world = bevy_ecs::world::World::new();
    let e = world.spawn(ph2d_physics_ecs::Health::default()).id();
    property::set(&reg, &mut world, e, "health.max", serde_json::json!(50.0)).unwrap();
    property::set(&reg, &mut world, e, "health.seed", serde_json::json!(7.0)).unwrap();
    let h = world.get::<ph2d_physics_ecs::Health>(e).unwrap();
    assert_eq!((h.max, h.seed), (50.0, 7));
    assert_eq!(h.start, ph2d_physics_ecs::Health::default().start);
    assert_eq!(property::get(&reg, &world, e, "health.max").unwrap(), serde_json::json!(50.0));
}

#[test]
fn um_nome_ou_um_valor_errado_diz_porque() {
    let reg = registo();
    let mut world = bevy_ecs::world::World::new();
    let e = world.spawn(ph2d_physics_ecs::Health::default()).id();
    let mut erro = |n: &str, v| property::set(&reg, &mut world, e, n, v).unwrap_err();
    assert_eq!(erro("max", serde_json::json!(1)), PropertyError::NotQualified("max".into()));
    assert_eq!(erro("vida.max", serde_json::json!(1)), PropertyError::UnknownComponent("vida".into()));
    assert!(matches!(erro("health.maxx", serde_json::json!(1)), PropertyError::UnknownField { .. }));
    assert!(matches!(erro("health.max", serde_json::json!("abc")), PropertyError::WrongType { .. }));
    assert!(matches!(erro("health.seed", serde_json::json!(3.7)), PropertyError::WrongType { .. }));
    assert!(matches!(
        erro("game_camera.height", serde_json::json!(2.0)),
        PropertyError::Missing { .. }
    ));
    // Nada do que falhou chegou ao componente.
    assert_eq!(world.get::<ph2d_physics_ecs::Health>(e), Some(&ph2d_physics_ecs::Health::default()));
}
