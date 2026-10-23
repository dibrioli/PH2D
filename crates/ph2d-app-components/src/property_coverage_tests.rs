//! ⭐ **Todo campo de um componente com cartão tem nome de código, ou o motivo de não ter** (doc
//! CodeEditor 08 §3.11), e **o rótulo que o nome copia é o que o CARTÃO pinta**: a chave do campo
//! no catálogo aparece no código do painel. Sem isto o nome seguia um rótulo que o artista não vê.

use crate::component_registry_for_tests::registo;
use ph2d_component_desc::WHOLE;
use std::collections::BTreeSet;

/// O texto de todas as fontes do painel do Inspector, onde os cartões pintam os rótulos.
fn fontes_dos_cartoes() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../ph2d-panel-inspector/src");
    let mut out = String::new();
    let mut pilha = vec![dir];
    while let Some(d) = pilha.pop() {
        for e in std::fs::read_dir(&d).expect("a pasta do painel").flatten() {
            let p = e.path();
            if p.is_dir() {
                pilha.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out += &std::fs::read_to_string(&p).expect("fonte");
            }
        }
    }
    assert!(
        out.len() > 100_000,
        "o painel não foi lido: {} bytes",
        out.len()
    );
    out
}

/// ⏳ O único cartão cujo título não é o nome do componente: «Render Source» × «Sprite» (o script diz
/// `sprite.*`). Decisão do dono: renomear um ou o outro.
const SPRITE: (&str, &str) = (
    "ph2d::render::Sprite",
    "panel.inspector.render_source.render_source",
);

#[test]
fn o_cartao_pinta_a_chave_do_catalogo() {
    let fontes = fontes_dos_cartoes();
    let mut faltam = Vec::new();
    let mut vistos = 0;
    for d in ph2d_component_desc::all().filter(|d| !d.code.is_empty()) {
        // Uma LINHA do cartão de outro (`row_of`: a Gravidade no corpo, o Blend na imagem) não tem
        // título próprio — o cartão do dono pinta as linhas dela, e essas conferem-se abaixo.
        let titulo = if d.canonical_name == SPRITE.0 {
            SPRITE.1
        } else {
            d.display_key
        };
        if d.row_of.is_empty() && !fontes.contains(&format!("\"{titulo}\"")) {
            faltam.push(format!("{} (título {titulo})", d.code));
        }
        if !d.row_of.is_empty() && ph2d_component_desc::desc_for(d.row_of).is_none() {
            faltam.push(format!("{}: o dono '{}' não existe", d.code, d.row_of));
        }
        for f in d.fields.iter().filter(|f| !f.name.is_empty()) {
            vistos += 1;
            if !fontes.contains(&format!("\"{}\"", f.label_key)) {
                faltam.push(format!("{}.{} ({})", d.code, f.name, f.label_key));
            }
        }
    }
    assert!(vistos >= 32, "{vistos}");
    assert!(
        faltam.is_empty(),
        "o cartão não pinta a chave do catálogo:\n  {}",
        faltam.join("\n  ")
    );
}

/// Uma instância de um componente com cartão e SEM `Default`, para o portão lhe ler as chaves.
fn amostra(
    canonical: &str,
    world: &mut bevy_ecs::world::World,
    e: bevy_ecs::entity::Entity,
) -> bool {
    let mut pos = |c: &dyn Fn(&mut bevy_ecs::world::EntityWorldMut<'_>)| {
        c(&mut world.entity_mut(e));
        true
    };
    match canonical {
        // Uma linha por tipo: `"ph2d::…" => pos(&|w| { w.insert(…); }),`
        "ph2d::render::Sprite" => pos(&|w| {
            w.insert(ph2d_render::Sprite::atlas(1, [1.0, 1.0], [1.0; 4]));
        }),
        "ph2d::physics::MassOverride" => pos(&|w| {
            w.insert(ph2d_physics_ecs::MassOverride(1.0));
        }),
        "ph2d::physics::Dominance" => pos(&|w| {
            w.insert(ph2d_physics_ecs::Dominance(1));
        }),
        _ => {
            let _ = &mut pos;
            false
        }
    }
}

/// As chaves serde de um componente, lidas do seu valor por omissão (`None` = é um newtype).
fn chaves(entry: &ph2d_ecs::scene::ComponentTypeEntry) -> Result<Option<BTreeSet<String>>, String> {
    let mut world = bevy_ecs::world::World::new();
    let e = world.spawn_empty().id();
    match entry.insert_default {
        Some(insert) => insert(&mut world, e).map_err(|err| format!("{err}"))?,
        None if amostra(entry.canonical_name, &mut world, e) => {}
        None => return Err("sem Default: dê-lhe uma linha em `amostra`".into()),
    }
    match (entry.value_of)(&world, e) {
        Ok(Some(serde_json::Value::Object(m))) => Ok(Some(m.keys().cloned().collect())),
        Ok(Some(_)) => Ok(None),
        other => Err(format!("{other:?}")),
    }
}

/// A chave serde de topo de uma chave de campo: o elemento `"axis[0]"` é da chave `axis`, e `"[0]"`
/// é do valor inteiro de um newtype de array; `"easing/family"` é da chave `easing`; o campo de
/// variante `"at:{molde}/tag"` é da chave `at`.
fn topo(k: &str) -> &str {
    let k = k.split_once(':').map_or(k, |(top, _)| top);
    let k = k.split('/').next().unwrap_or(k);
    match k.split_once('[') {
        Some(("", _)) => WHOLE,
        Some((base, _)) => base,
        None => k,
    }
}

#[test]
fn todo_campo_de_um_cartao_tem_nome_ou_motivo() {
    let reg = registo();
    let mut queixas = Vec::new();
    let mut nomeados = 0;
    for entry in reg.iter() {
        let Some(d) = entry.desc else { continue };
        if !d.may_have_section() || d.hidden.is_some() {
            continue;
        }
        let quem = d.canonical_name;
        if d.code.is_empty() {
            queixas.push(format!("{quem}: sem nome de código nem motivo"));
            continue;
        }
        let chaves = match chaves(entry) {
            Ok(c) => c,
            Err(e) => {
                queixas.push(format!("{quem}: {e}"));
                continue;
            }
        };
        let nomeadas: BTreeSet<&str> = d
            .fields
            .iter()
            .filter(|f| !f.name.is_empty())
            .map(|f| topo(f.serde_key()))
            .collect();
        let ocultas: BTreeSet<&str> = d.hidden_keys.iter().map(|(k, _)| topo(k)).collect();
        let esperadas: BTreeSet<String> =
            chaves.unwrap_or_else(|| BTreeSet::from([WHOLE.to_owned()]));
        for k in &esperadas {
            match (nomeadas.contains(k.as_str()), ocultas.contains(k.as_str())) {
                (true, false) => nomeados += 1,
                (false, false) => queixas.push(format!("{quem}: '{k}' sem nome nem motivo")),
                (true, true) => queixas.push(format!("{quem}: '{k}' tem nome E motivo")),
                (false, true) => {}
            }
        }
        for k in nomeadas.iter().chain(&ocultas) {
            if !esperadas.contains(*k) {
                queixas.push(format!("{quem}: '{k}' não é uma chave do componente"));
            }
        }
    }
    assert!(nomeados >= 32, "{nomeados}");
    assert!(
        queixas.is_empty(),
        "{} queixas:\n  {}",
        queixas.len(),
        queixas.join("\n  ")
    );
}

#[test]
fn todo_componente_com_nome_tem_a_sua_lei() {
    let leis: Vec<&str> = crate::property_laws::LEIS.iter().map(|(n, _)| *n).collect();
    let mut com_nome = BTreeSet::new();
    for d in ph2d_component_desc::all().filter(|d| !d.code.is_empty()) {
        com_nome.insert(d.canonical_name);
        let n = leis.iter().filter(|l| **l == d.canonical_name).count();
        assert_eq!(
            n, 1,
            "{}: {n} linhas em LEIS (uma, `None` se o cartão aceita tudo)",
            d.code
        );
    }
    for l in &leis {
        assert!(com_nome.contains(l), "{l} está em LEIS sem nome de código");
    }
}
