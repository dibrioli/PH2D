//! Os gates da wave A1 — o vocabulário do índice.
//!
//! ⚠️ Cada um afirma uma LEI que uma implementação plausível quebraria, não que o código compila.

use ph2d_asset_index::CatalogScope;
use ph2d_asset_index::{
    AssetEntry, AssetIndex, AssetKind, AssetRef, CatalogId, Query, Relation, SortBy,
};

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

// ── ⭐⭐ AS DUAS METADES DE UMA RELAÇÃO (plano 07 D9) ────────────────────────────────────────────

/// Uma receita que desenha duas texturas, e uma terceira textura que ninguém usa.
fn library_with_one_prefab() -> AssetIndex {
    let mut ix = AssetIndex::new();
    let a = AssetRef::Texture { asset: [1; 32] };
    let b = AssetRef::Texture { asset: [2; 32] };
    ix.push(tex(1, "bark"));
    ix.push(tex(2, "leaf"));
    ix.push(tex(3, "unrelated"));
    let mut house = comp(10, "house");
    house.deps = vec![a, b];
    ix.push(house);
    ix
}

fn names(hits: &[&AssetEntry]) -> Vec<String> {
    let mut v: Vec<String> = hits.iter().map(|e| e.name.clone()).collect();
    v.sort();
    v
}

/// ⭐ **«O que isto usa»** devolve as dependências, e só elas.
#[test]
fn the_uses_filter_shows_exactly_what_the_anchor_depends_on() {
    let ix = library_with_one_prefab();
    let q = Query {
        related: Some((AssetRef::Component { stable_id: 10 }, Relation::Uses)),
        ..Query::default()
    };
    assert_eq!(names(&ix.query(&q)), vec!["bark", "leaf"]);
}

/// ⭐ **«O que usa isto»** é o outro sentido, e ele é DERIVADO por inversão — ninguém o guarda.
#[test]
fn the_used_by_filter_shows_exactly_who_depends_on_the_anchor() {
    let ix = library_with_one_prefab();
    let q = Query {
        related: Some((AssetRef::Texture { asset: [1; 32] }, Relation::UsedBy)),
        ..Query::default()
    };
    assert_eq!(names(&ix.query(&q)), vec!["house"]);

    // E a textura que ninguém usa devolve vazio — não «todas».
    let q = Query {
        related: Some((AssetRef::Texture { asset: [3; 32] }, Relation::UsedBy)),
        ..Query::default()
    };
    assert!(ix.query(&q).is_empty(), "ninguem usa a `unrelated`");
}

/// ⚠️ **Nada se relaciona consigo próprio.** Sem esta lei a âncora aparecia na própria resposta, e
/// o artista lia-a como um utilizador dela mesma.
#[test]
fn the_anchor_is_never_in_its_own_answer() {
    let ix = library_with_one_prefab();
    for dir in [Relation::Uses, Relation::UsedBy] {
        let anchor = AssetRef::Component { stable_id: 10 };
        let q = Query {
            related: Some((anchor, dir)),
            ..Query::default()
        };
        assert!(
            !ix.query(&q).iter().any(|e| e.key == anchor),
            "{dir:?}: a ancora entrou na propria resposta"
        );
    }
}

/// ⛔⛔ **A ÂNCORA QUE JÁ NÃO EXISTE nunca devolve a biblioteca inteira.**
///
/// É a direcção de falha que decide: a âncora pode sair da biblioteca entre o clique no menu e o
/// quadro seguinte, e um filtro que se desligasse sozinho devolveria **tudo** por baixo de uma
/// faixa a dizer *«o que usa X»* — a resposta errada com a etiqueta certa.
///
/// ⚠️⚠️ **A 1.ª versão deste gate percorria os DOIS sentidos a exigir vazio, e só um deles podia
/// sangrar.** O `UsedBy` é fechado por construção (ninguém nomeia uma âncora inexistente), então
/// aquele laço dava a impressão de medir duas leis e media uma. E a régua estava **errada** para o
/// outro caso, que é o interessante — ver o gate irmão abaixo.
///
/// **Mutação que deve sangrar:** trocar o `is_some_and` do `relates` por `map_or(true, …)`.
#[test]
fn a_missing_anchor_never_opens_the_uses_filter() {
    let ix = library_with_one_prefab();
    let ghost = AssetRef::Component { stable_id: 999 };
    let q = Query {
        related: Some((ghost, Relation::Uses)),
        ..Query::default()
    };
    assert!(
        ix.query(&q).is_empty(),
        "uma ancora ausente devolveu {} entradas em `Uses`",
        ix.query(&q).len()
    );
}

/// ⭐⭐ **E um endereço que saiu da biblioteca mas que alguém ainda NOMEIA devolve quem o nomeia.**
///
/// ⚠️ **Não é uma fuga da lei de cima — é a resposta honesta**, e a única que ajuda quem vai
/// reparar o buraco: *estas receitas ainda apontam para uma coisa que já não está cá*. Um vazio
/// aqui esconderia exactamente a informação que se foi buscar.
#[test]
fn a_dangling_address_still_answers_who_points_at_it() {
    let mut ix = AssetIndex::new();
    let gone = AssetRef::Texture { asset: [42; 32] };
    let mut house = comp(10, "house");
    house.deps = vec![gone];
    ix.push(house);
    ix.push(tex(3, "unrelated"));
    assert!(
        ix.get(&gone).is_none(),
        "a fixtura tem de ter o endereco FORA"
    );

    let q = Query {
        related: Some((gone, Relation::UsedBy)),
        ..Query::default()
    };
    assert_eq!(names(&ix.query(&q)), vec!["house"]);
}

/// ⚠️ **A relação COMPÕE com os outros filtros em vez de os substituir** — um modo que desligasse
/// a busca deixaria a caixa de texto a mentir no ecrã.
#[test]
fn the_relation_composes_with_the_search_instead_of_replacing_it() {
    let ix = library_with_one_prefab();
    let q = Query {
        text: "leaf".to_string(),
        related: Some((AssetRef::Component { stable_id: 10 }, Relation::Uses)),
        ..Query::default()
    };
    assert_eq!(
        names(&ix.query(&q)),
        vec!["leaf"],
        "a busca tem de continuar a estreitar dentro da relacao"
    );
}
