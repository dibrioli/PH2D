//! ⭐ **O censo de ALCANCE da paleta de componentes** (ADR-0166 / plano F3), nos DOIS sentidos.
//!
//! O molde é o [`crate::field3d_reach_tests`], e a razão é a mesma: *o painel oferece exatamente o
//! que o gesto faz*. Aqui a pergunta tem duas metades, e cada uma barra um defeito diferente:
//!
//! | Sentido | O defeito que ele apanha |
//! |---|---|
//! | todo `Authored` chega à paleta de ALGUM tipo de objeto | um componente que existe, tem descritor, e **nenhuma porta o anexa** |
//! | nada além de `Authored` chega | o `+` ofereceria o que o artista não escolhe (um marcador de sistema, uma ponte) |
//!
//! ⚠️ **Ele pergunta ao registo do PRODUTO** ([`crate::init::build_component_registry`]) — um
//! `ComponentRegistry::new()` montado à mão aqui seria uma segunda lista, e o gate ficaria verde
//! sobre um registo que ninguém executa.

use ph2d_component_desc::{Attach, ObjectKind};

fn buildable_in_the_product(name: &str) -> bool {
    let reg = crate::init::build_component_registry();
    reg.get_by_id(ph2d_ecs::scene::stable_type_id(name))
        .is_some_and(|e| e.insert_default.is_some())
}

/// Os nomes canónicos que a paleta oferece a `kind`, com *Show all* LIGADO (isto é: tudo o que ela
/// consegue mostrar, aplicável ou não).
fn offered_to(kind: ObjectKind) -> Vec<&'static str> {
    let reg = crate::init::build_component_registry();
    let can_build = |n: &str| {
        reg.get_by_id(ph2d_ecs::scene::stable_type_id(n))
            .is_some_and(|e| e.insert_default.is_some())
    };
    let m = crate::component_palette::build(kind, &[], &can_build, true);
    m.groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .filter_map(|i| crate::component_palette::name_of_pick(i.id))
        .collect()
}

/// ⭐ **Todo `Authored` que o registo sabe construir chega à paleta de algum tipo de objeto.**
///
/// ⚠️ Isto é o **sentido difícil** do censo: um componente autorado, com descritor e com
/// `insert_default`, que nenhuma paleta oferece é uma feature construída e **inalcançável** — e
/// esse é exatamente o estado em que a F3 encontrou cinco delas (as cinco portas `INSP_*_ADD`).
#[test]
fn every_buildable_authored_component_is_reachable_from_some_palette() {
    let reachable: std::collections::BTreeSet<&str> = ObjectKind::ALL
        .iter()
        .flat_map(|&k| offered_to(k))
        .collect();

    let missing: Vec<&str> = ph2d_component_desc::all()
        .filter(|d| matches!(d.attach, Attach::Authored { .. }))
        .map(|d| d.canonical_name)
        .filter(|n| buildable_in_the_product(n))
        .filter(|n| !reachable.contains(n))
        .collect();

    assert!(
        missing.is_empty(),
        "componentes AUTORADOS que o registo constroi e nenhuma paleta oferece \
         (o artista nao tem como os anexar): {missing:#?}"
    );
}

/// ⚠️ **E o sentido fácil, que também tem de valer:** nada que não seja `Authored` chega à paleta.
///
/// Um `Intrinsic` chega pelo gesto que cria o objeto (a `Sprite`); um `Machinery` é posto por um
/// sistema. Oferecer qualquer um seria oferecer o que não é uma escolha do artista.
#[test]
fn nothing_but_authored_reaches_the_palette() {
    let offered: std::collections::BTreeSet<&str> = ObjectKind::ALL
        .iter()
        .flat_map(|&k| offered_to(k))
        .collect();
    let intruders: Vec<&str> = offered
        .iter()
        .copied()
        .filter(|n| {
            ph2d_component_desc::desc_for(n)
                .is_none_or(|d| !matches!(d.attach, Attach::Authored { .. }))
        })
        .collect();
    assert!(
        intruders.is_empty(),
        "a paleta ofereceu o que nao e' uma escolha do artista: {intruders:#?}"
    );
}

/// ⭐ **O CENSO IMPRESSO** — a tabela que a F3 usou para decidir a poda (peça 1).
///
/// ⚠️ Não é um gate: é o instrumento. Ele lista, por família, os `Authored` **sem `insert_default`
/// no registo do produto** — que são precisamente os que a paleta *não pode* oferecer, e portanto
/// os que ainda precisam de uma porta própria no Inspector.
///
/// `cargo test -p ph2d-host-desktop --bins the_palette_census -- --ignored --nocapture`
#[test]
#[ignore = "instrumento: imprime o censo, nao afirma nada"]
fn the_palette_census() {
    let mut without = Vec::new();
    for d in ph2d_component_desc::all() {
        if !matches!(d.attach, Attach::Authored { .. }) {
            continue;
        }
        if !buildable_in_the_product(d.canonical_name) {
            without.push((d.category, d.canonical_name, d.display_name));
        }
    }
    without.sort_by_key(|(c, n, _)| (format!("{c:?}"), *n));
    println!("\n=== AUTORADOS que o registo do produto NAO sabe construir ===");
    for (cat, name, label) in &without {
        println!("  {cat:?} · {label} ({name})");
    }
    println!("  total: {}\n", without.len());
}
