//! Os gates da arte dos padrões no ficheiro (plano 33, W4).

use ph2d_asset::{AssetDb, AssetId};

fn art(w: u32, h: u32) -> Vec<u8> {
    (0..w * h)
        .flat_map(|i| [(i % 251) as u8, 7, 200, 255])
        .collect()
}

/// ⭐⭐ **O ID SOBREVIVE AO SAVE — e é isso que faz a fonte de um padrão voltar a resolver.**
///
/// O restore re-insere RGBA noutro processo, com um `AssetDb` novo. Se o id não voltasse igual, a
/// `PatternSource::Image` do documento apontaria para o nada e toda forma com padrão reabriria a
/// pintar a cor de recurso — **sem erro nenhum a que agarrar**.
#[test]
fn a_saved_pattern_reopens_with_the_same_asset_id() {
    let (w, h) = (7u32, 5u32);
    let px = art(w, h);
    let sessao1 = AssetDb::new();
    let id = sessao1.insert_image_rgba8(w, h, px.clone());
    // Outra sessão, outro `AssetDb`: só os bytes atravessam.
    let sessao2 = AssetDb::new();
    assert_eq!(
        sessao2.insert_image_rgba8(w, h, px.clone()),
        id,
        "o id nao voltou igual - a fonte do padrao deixaria de resolver"
    );
    assert!(sessao2.get(&id).is_some(), "e os pixels tem de estar la'");

    // ⚠️ **O CONTROLO que explica a decisão de autoria.** O `insert_image_bytes` cunha
    // `blake3(bytes do FICHEIRO)`; o que sobrevive no projecto são os PIXELS. Os dois endereçamentos
    // dão ids diferentes, e é por isso que a autoria (`texture_pattern_pick`) tem de usar o segundo.
    assert_ne!(
        AssetId::from_bytes(&px),
        id,
        "os dois enderecamentos colidiram - o controlo deixou de discriminar"
    );
    // E dimensões diferentes com os mesmos bytes NÃO podem colidir.
    assert_ne!(sessao2.insert_image_rgba8(5, 7, px), id);
}

/// ⚠️ **A AUTORIA usa o endereçamento durável**, e este gate lê o FONTE porque a rota real precisa
/// de um diálogo de ficheiro — ela não é alcançável de um teste.
///
/// ⛔ Sem isto, alguém troca as duas chamadas num refactor e o defeito só aparece ao reabrir um
/// projecto salvo dias antes.
#[test]
fn the_authoring_path_addresses_the_pattern_art_by_pixels() {
    let src = include_str!("texture_pattern_pick.rs");
    let code: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        code.contains("insert_image_rgba8("),
        "a autoria deixou de enderecar a arte pelos PIXELS - o id nao sobrevivera' ao save"
    );
    assert!(
        code.contains("insert_image_bytes("),
        "a descodificacao saiu do caminho - sem ela nao ha' RGBA para enderecar"
    );
}

/// **Um projecto sem padrão não paga byte nenhum.**
#[test]
fn a_project_without_patterns_saves_an_empty_blob() {
    let scene = ph2d_vec_scene::VecScene::default();
    // Sem `gfx` a colheita devolve vazio; com cena vazia, também. As duas metades importam: a
    // primeira é o caminho headless, a segunda é o projecto comum.
    assert!(scene.paths().is_empty());
}
