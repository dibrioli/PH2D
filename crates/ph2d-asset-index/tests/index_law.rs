//! Os gates da wave A1 — o vocabulário do índice.
//!
//! ⚠️ Cada um afirma uma LEI que uma implementação plausível quebraria, não que o código compila.

use ph2d_asset_index::CatalogScope;
use ph2d_asset_index::{AssetEntry, AssetIndex, AssetKind, AssetRef, CatalogId, Query, SortBy};

fn comp(id: u64, name: &str) -> AssetEntry {
    AssetEntry::new(AssetRef::Component { stable_id: id }, name)
}

fn tex(byte: u8, name: &str) -> AssetEntry {
    AssetEntry::new(AssetRef::Texture { asset: [byte; 32] }, name)
}

/// ⭐ **A junção é o ponto**: as duas fontes convivem numa lista só, e a família de cada entrada é
/// lida do ENDEREÇO — uma entrada não pode declarar uma família que o endereço contradiz.
#[test]
fn the_two_sources_live_in_one_list_and_the_kind_comes_from_the_address() {
    let mut ix = AssetIndex::new();
    ix.push(comp(7, "Ragdoll"));
    ix.push(tex(3, "brick.png"));
    assert_eq!(ix.len(), 2);
    let counts = ix.counts();
    assert_eq!(counts.get(&AssetKind::Component), Some(&1));
    assert_eq!(counts.get(&AssetKind::Texture), Some(&1));
    assert_eq!(
        ix.entries()[0].kind(),
        AssetKind::Component,
        "a familia sai da variante do AssetRef"
    );
}

/// ⛔ **O mesmo asset visto por dois caminhos é UMA entrada.** Uma textura usada por três sprites
/// apareceria três vezes numa grade que empilha sem chave.
#[test]
fn the_same_address_pushed_twice_is_one_entry_and_keeps_its_discovery_order() {
    let mut ix = AssetIndex::new();
    ix.push(tex(1, "a.png"));
    ix.push(comp(9, "Later"));
    let mut again = tex(1, "a.png (renamed)");
    again.detail = "512x512".into();
    ix.push(again);
    assert_eq!(ix.len(), 2, "a re-insercao substitui, nao acrescenta");
    let e = ix.get(&AssetRef::Texture { asset: [1; 32] }).unwrap();
    assert_eq!(e.detail, "512x512", "o conteudo novo venceu");
    assert_eq!(e.seq, 0, "e o lugar na ordem de descoberta e' o ORIGINAL");
}

/// A busca é sub-string sem distinguir maiúsculas, e **texto vazio é «sem filtro», nunca «nada»**.
#[test]
fn an_empty_search_means_no_filter_and_a_match_ignores_case() {
    let mut ix = AssetIndex::new();
    ix.push(comp(1, "Ragdoll"));
    ix.push(comp(2, "Crate"));
    assert_eq!(ix.query(&Query::default()).len(), 2);
    let q = Query {
        text: "RAG".into(),
        ..Default::default()
    };
    let hits = ix.query(&q);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].name, "Ragdoll");
}

/// ⚠️ **A ordem é TOTAL nos três modos.** Dois assets com o mesmo nome não podem trocar de lugar
/// entre quadros — o cartão debaixo do dedo deixaria de ser o que o artista mirou.
#[test]
fn every_sort_is_a_total_order_even_when_two_assets_share_a_name() {
    let mut ix = AssetIndex::new();
    ix.push(comp(1, "Same"));
    ix.push(tex(2, "Same"));
    ix.push(comp(3, "Same"));
    for sort in SortBy::ALL {
        let q = Query {
            sort: *sort,
            ..Default::default()
        };
        let a: Vec<u64> = ix.query(&q).iter().map(|e| e.seq).collect();
        let b: Vec<u64> = ix.query(&q).iter().map(|e| e.seq).collect();
        assert_eq!(a, b, "{sort:?} nao e' determinista");
        let mut sorted = a.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            a.len(),
            "{sort:?} perdeu ou duplicou entradas"
        );
    }
}

/// `Recent` é o **inverso** da ordem de descoberta — o último a entrar aparece primeiro.
#[test]
fn recent_puts_the_last_discovered_first() {
    let mut ix = AssetIndex::new();
    ix.push(comp(1, "First"));
    ix.push(comp(2, "Second"));
    let q = Query {
        sort: SortBy::Recent,
        ..Default::default()
    };
    assert_eq!(ix.query(&q)[0].name, "Second");
}

/// O filtro de catálogo **chega à consulta** — a terceira pergunta do knob morto (o painel escreve
/// onde · quem lê · o leitor DECIDE?). Aqui o leitor decide: a entrada sem catálogo sai.
#[test]
fn the_catalog_filter_actually_narrows_the_result() {
    let cat = CatalogId(42);
    let mut ix = AssetIndex::new();
    let mut inside = comp(1, "Inside");
    inside.catalog = Some(cat);
    ix.push(inside);
    ix.push(comp(2, "Unassigned"));
    let q = Query {
        catalog: CatalogScope::These(vec![cat]),
        ..Default::default()
    };
    let hits = ix.query(&q);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].name, "Inside");

    // ⭐ **E o terceiro estado**: *«os que não estão em catálogo nenhum»* é uma pergunta DIFERENTE
    // de *«todos»*, e sem ela um asset por arrumar fica inalcançável no dia em que existir um
    // catálogo. **Mutação que deve sangrar:** o braço `Unassigned` devolver `true`.
    let q = Query {
        catalog: CatalogScope::Unassigned,
        ..Default::default()
    };
    let hits = ix.query(&q);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].name, "Unassigned");
}

/// ⭐ **Os dois sentidos, e só um guardado.** `owners` é DERIVADO por inversão de `deps`; guardar
/// os dois seria duas respostas à mesma pergunta.
#[test]
fn owners_is_the_inversion_of_deps_and_is_never_stored() {
    let brick = AssetRef::Texture { asset: [5; 32] };
    let mut ix = AssetIndex::new();
    ix.push(tex(5, "brick.png"));
    let mut wall = comp(1, "Wall");
    wall.deps = vec![brick];
    ix.push(wall);
    let mut floor = comp(2, "Floor");
    floor.deps = vec![brick];
    ix.push(floor);

    let owners: Vec<&str> = ix.owners(&brick).iter().map(|e| e.name.as_str()).collect();
    assert_eq!(owners, vec!["Wall", "Floor"]);
    let deps: Vec<&str> = ix
        .deps(&AssetRef::Component { stable_id: 1 })
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    assert_eq!(deps, vec!["brick.png"]);
    assert!(
        ix.owners(&AssetRef::Component { stable_id: 1 }).is_empty(),
        "ninguem depende do Wall"
    );
}

/// ⛔ Uma dependência a um asset que o índice **não tem** não inventa uma entrada — ela some da
/// lista de `deps` resolvidas. *Um índice que devolve fantasmas é pior que um que devolve menos.*
#[test]
fn a_dependency_on_something_absent_resolves_to_nothing() {
    let mut ix = AssetIndex::new();
    let mut wall = comp(1, "Wall");
    wall.deps = vec![AssetRef::Texture { asset: [9; 32] }];
    ix.push(wall);
    assert!(ix.deps(&AssetRef::Component { stable_id: 1 }).is_empty());
}
