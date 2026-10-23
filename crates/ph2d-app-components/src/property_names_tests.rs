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
        assert!(
            codes.insert(d.code),
            "dois componentes chamam-se '{}'",
            d.code
        );
        assert_eq!(
            d.code,
            snake(ph2d_i18n::tr(d.display_key)),
            "{}",
            d.canonical_name
        );
        let mut nomes = BTreeSet::new();
        for f in d.fields {
            if f.name.is_empty() {
                // ⏳ Sem nome só fica uma chave oculta (`.keyed(k)` + `hiding`), ou uma chave que um
                // IRMÃO com nome já cobre (o «Source Name» do `UiLabel` vive dentro do enum `source`).
                let oculta = !f.key.is_empty() && d.hidden_keys.iter().any(|(k, _)| *k == f.key);
                let do_irmao = !f.key.is_empty()
                    && d.fields
                        .iter()
                        .any(|g| !g.name.is_empty() && g.serde_key() == f.key);
                assert!(
                    oculta || do_irmao,
                    "{} campo {} sem nome nem chave oculta ou nomeada por um irmão",
                    d.code,
                    f.field_id
                );
                continue;
            }
            assert!(nomes.insert(f.name), "{}.{} repetido", d.code, f.name);
            assert_eq!(
                f.name,
                snake(ph2d_i18n::tr(f.label_key)),
                "{}.{}",
                d.code,
                f.name
            );
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
        for f in d.fields.iter().filter(|f| !f.name.is_empty()) {
            let nome = property::full_name(d.code, f.name);
            let v = property::get(
                &reg,
                &world,
                e,
                &nome,
                ph2d_ecs::scene::property_ref::RefNames::default(),
            )
            .unwrap_or_else(|err| panic!("{nome}: {err}"));
            property::set(
                &reg,
                &mut world,
                e,
                &nome,
                v,
                ph2d_ecs::scene::property_ref::RefNames::default(),
            )
            .unwrap_or_else(|err| panic!("{nome}: {err}"));
            campos += 1;
        }
        assert_eq!(
            antes,
            (entry.serialize)(&world, e).expect("bytes"),
            "{}",
            d.code
        );
    }
    assert!(campos >= 32, "{campos}");
}

#[test]
fn escrever_um_campo_muda_so_esse() {
    let reg = registo();
    let mut world = bevy_ecs::world::World::new();
    let e = world.spawn(ph2d_physics_ecs::Health::default()).id();
    property::set(
        &reg,
        &mut world,
        e,
        "health.max",
        serde_json::json!(50.0),
        ph2d_ecs::scene::property_ref::RefNames::default(),
    )
    .unwrap();
    property::set(
        &reg,
        &mut world,
        e,
        "health.seed",
        serde_json::json!(7.0),
        ph2d_ecs::scene::property_ref::RefNames::default(),
    )
    .unwrap();
    let h = world.get::<ph2d_physics_ecs::Health>(e).unwrap();
    assert_eq!((h.max, h.seed), (50.0, 7));
    assert_eq!(h.start, ph2d_physics_ecs::Health::default().start);
    assert_eq!(
        property::get(
            &reg,
            &world,
            e,
            "health.max",
            ph2d_ecs::scene::property_ref::RefNames::default()
        )
        .unwrap(),
        serde_json::json!(50.0)
    );
}

#[test]
fn um_nome_ou_um_valor_errado_diz_porque() {
    let reg = registo();
    let mut world = bevy_ecs::world::World::new();
    let e = world.spawn(ph2d_physics_ecs::Health::default()).id();
    let mut erro = |n: &str, v| {
        property::set(
            &reg,
            &mut world,
            e,
            n,
            v,
            ph2d_ecs::scene::property_ref::RefNames::default(),
        )
        .unwrap_err()
    };
    assert_eq!(
        erro("max", serde_json::json!(1)),
        PropertyError::NotQualified("max".into())
    );
    assert_eq!(
        erro("vida.max", serde_json::json!(1)),
        PropertyError::UnknownComponent("vida".into())
    );
    assert!(matches!(
        erro("health.maxx", serde_json::json!(1)),
        PropertyError::UnknownField { .. }
    ));
    assert!(matches!(
        erro("health.max", serde_json::json!("abc")),
        PropertyError::WrongType { .. }
    ));
    assert!(matches!(
        erro("health.seed", serde_json::json!(3.7)),
        PropertyError::WrongType { .. }
    ));
    assert!(matches!(
        erro("game_camera.height", serde_json::json!(2.0)),
        PropertyError::Missing { .. }
    ));
    // Nada do que falhou chegou ao componente.
    assert_eq!(
        world.get::<ph2d_physics_ecs::Health>(e),
        Some(&ph2d_physics_ecs::Health::default())
    );
}

/// ⭐ Toda ESCOLHA, toda caixa e todo booleano invertido vai e volta pelo nome que o script usa: o json
/// de cada opção é um valor que o componente aceita (o `ler_e_escrever` só prova a opção de fábrica),
/// e o nome é `snake`.
#[test]
fn toda_vista_vai_e_volta_pelo_valor_do_cartao() {
    use ph2d_component_desc::View;
    use serde_json::json;
    let reg = registo();
    let mut world = bevy_ecs::world::World::new();
    let mut vistas = 0;
    for entry in reg.iter() {
        let (Some(d), Some(insert_default)) = (entry.desc, entry.insert_default) else {
            continue;
        };
        if d.code.is_empty() {
            continue;
        }
        let e = world.spawn_empty().id();
        insert_default(&mut world, e).expect("default");
        for f in d.fields.iter().filter(|f| !f.name.is_empty()) {
            let nome = property::full_name(d.code, f.name);
            let valores: Vec<serde_json::Value> = match f.view {
                View::Options(opts) => opts
                    .iter()
                    .map(|(n, _)| {
                        let snake = n
                            .chars()
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
                        assert!(snake, "{nome}: a opção '{n}' não é snake");
                        json!(n)
                    })
                    .collect(),
                View::Toggle(..) | View::Not => vec![json!(true), json!(false)],
                _ => continue,
            };
            for v in valores {
                property::set(
                    &reg,
                    &mut world,
                    e,
                    &nome,
                    v.clone(),
                    ph2d_ecs::scene::property_ref::RefNames::default(),
                )
                .unwrap_or_else(|err| panic!("{nome} = {v}: {err}"));
                assert_eq!(
                    property::get(
                        &reg,
                        &world,
                        e,
                        &nome,
                        ph2d_ecs::scene::property_ref::RefNames::default()
                    )
                    .unwrap(),
                    v,
                    "{nome}"
                );
                vistas += 1;
            }
        }
    }
    assert!(vistas >= 100, "{vistas}");
}
