//! **O CENSO DA PALETA** (doc 89, folha 17) — quem é oferecido ao artista, e com que nome.
//!
//! A shell é o único sítio que vê as três peças: o registo inteiro
//! (`ph2d-node-registry-init`), os metadados de UI de cada tipo, e o filtro que a paleta
//! aplica. Nenhuma crate de nó vê as outras 129.
//!
//! ## O que este ficheiro mede, e por que ele nasceu
//!
//! ⚠️ **Medido em 2026-08-25: dos 130 tipos registados, TRÊS não tinham `NodeUiManifest`**
//! — e sem ele a paleta cai no **nome CRU do tipo** (`pulse.signal`) na categoria cinzenta
//! de omissão. Dois eram fixturas (o `debug.const` do W1.T3 e o `debug.wave`, o template
//! de fan-out); o terceiro era o `pulse.signal`, **um nó de artista a que ninguém deu
//! nome**.
//!
//! ⚠️ **É por isso que «não é oferecido» é um OPT-IN e não uma regra derivada da ausência
//! de metadados.** *«Não tem `NodeUiManifest`» quer dizer «é fixtura» e também «alguém
//! esqueceu», e as duas leem igual.* Com o opt-in, esquecer põe o nó na paleta com o nome
//! cru — **visível**; com a regra derivada, esquecer fá-lo-ia **desaparecer em silêncio**.

use ph2d_node_registry::NodeRegistry;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// ⭐⭐ **TODO NÓ OFERECIDO TEM UM NOME DE ARTISTA.**
///
/// FALSIFICADO por um nó novo que registe o op e esqueça o `register_ui` — hoje o
/// sintoma seria `motion.sub_uv` a aparecer na paleta em vez de *Sub UV*, cinzento entre
/// os utilitários, e nada daria erro.
#[test]
fn every_node_the_palette_offers_has_an_artist_facing_name() {
    let reg = registry();
    let naked: Vec<&str> = reg
        .manifests()
        .filter(|m| !reg.is_fixture(m.id) && reg.ui_manifest(m.id).is_none())
        .map(|m| m.name)
        .collect();
    assert!(
        naked.is_empty(),
        "estes tipos aparecem na paleta com o NOME CRU e na categoria de omissao: {naked:?} \
         — ou eles ganham `register_ui`, ou declaram-se `register_fixture`"
    );
}

/// ⭐ **AS FIXTURAS SÃO EXACTAMENTE AS DUAS, E NENHUMA DELAS É OFERECIDA.**
///
/// ⚠️ A lista é escrita aqui de propósito: declarar-se fixtura tira um nó da paleta, e é
/// uma decisão que merece um sítio onde alguém a veja. Um terceiro nome a aparecer aqui
/// é um pedido para explicar por que ele deixou de ser oferecido.
#[test]
fn the_only_fixtures_are_the_two_the_engine_needs() {
    let reg = registry();
    let mut fixtures: Vec<&str> = reg
        .manifests()
        .filter(|m| reg.is_fixture(m.id))
        .map(|m| m.name)
        .collect();
    fixtures.sort_unstable();
    assert_eq!(
        fixtures,
        ["debug.const", "debug.wave"],
        "a lista de fixturas mudou — quem entra sai da paleta do artista"
    );
}

/// **E o `pulse.signal` — o terceiro do censo — TEM nome.** Um gate pelo NOME, porque foi
/// esta linha que lho deu: sem ele, o achado volta em silêncio.
#[test]
fn the_signal_node_is_offered_by_its_artist_name() {
    let reg = registry();
    let m = reg
        .manifests()
        .find(|m| m.name == "pulse.signal")
        .expect("o `pulse.signal` existe");
    assert!(!reg.is_fixture(m.id), "ele e' um no' de artista");
    assert_eq!(
        reg.ui_manifest(m.id).map(|u| u.display_name),
        Some("Signal")
    );
}
