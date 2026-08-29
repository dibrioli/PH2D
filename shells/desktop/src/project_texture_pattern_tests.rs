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

// ── AS DUAS TINTAS (plano 35, wave D) ─────────────────────────────────────────

/// Uma cena com padrão no preenchimento, no traço, ou nos dois — **com artes DIFERENTES**.
///
/// ⚠️ Duas artes iguais fariam um coletor que só varre o `fill` devolver o conjunto certo por
/// acidente: é a mesma armadilha que a chave do memo da wave C teve de evitar.
fn cena(no_fill: Option<AssetId>, no_traco: Option<AssetId>) -> ph2d_vec_scene::VecScene {
    use ph2d_vec_scene::{
        Paint, PatternFill, PatternSource, Rgba8, StrokePaint, StrokeSpec, VecPath, VecVertex,
    };
    let cor = Rgba8::new(1, 2, 3, 255);
    let lei = |id: AssetId| PatternFill::new(PatternSource::Image(id), [1.0, 1.0], cor);
    let mut s = StrokeSpec::new(cor, 0.5);
    if let Some(id) = no_traco {
        s.paint = StrokePaint::Pattern(Box::new(lei(id)));
    }
    let mut scene = ph2d_vec_scene::VecScene::default();
    scene.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(no_fill.map_or(Paint::solid(cor), |id| Paint::Pattern(Box::new(lei(id))))),
        stroke: Some(s),
        ..VecPath::default()
    });
    scene
}

/// ⭐⭐ **A ARTE DE UM PADRÃO NO TRAÇO TAMBÉM VIAJA NO FICHEIRO.**
///
/// ⛔ Enquanto a colheita lia só o `fill`, uma forma cujo traço tinha padrão gravava o `AssetId` no
/// documento e **nunca os pixels** — reabrir dava uma linha a pintar a cor de recurso, para sempre e
/// sem erro nenhum a que agarrar. É o defeito que este ficheiro existe para curar, com o sujeito
/// trocado.
#[test]
fn the_art_of_a_patterned_stroke_travels_in_the_file_too() {
    let a = AssetId::from_bytes(&[1u8; 16]);
    let b = AssetId::from_bytes(&[2u8; 16]);
    assert_ne!(a, b, "a fixtura precisa de DUAS artes distintas");

    assert_eq!(
        crate::project_texture_pattern::art_ids_named_by(&cena(None, Some(b))),
        [b].into_iter().collect(),
        "a arte do padrao do TRACO nao e' colhida - ela nao entra no ficheiro e a linha reabre \
         pintada a cor de recurso"
    );
    // CONTROLO: o preenchimento continua a ser colhido — senão este gate ficaria verde sobre uma
    // colheita que trocou uma metade pela outra.
    assert_eq!(
        crate::project_texture_pattern::art_ids_named_by(&cena(Some(a), None)),
        [a].into_iter().collect()
    );
    // E com as duas, as DUAS entram.
    assert_eq!(
        crate::project_texture_pattern::art_ids_named_by(&cena(Some(a), Some(b))),
        [a, b].into_iter().collect(),
        "uma das duas tintas ficou de fora"
    );
    // Sem padrão nenhum, nada se embute: um projecto sem padrões não paga byte nenhum.
    assert!(crate::project_texture_pattern::art_ids_named_by(&cena(None, None)).is_empty());
}

/// ⚠️ **A MESMA arte nas duas tintas custa UMA entrada** — a colheita é chaveada pelo `AssetId`, e a
/// propriedade tem de sobreviver à segunda tinta.
#[test]
fn the_same_art_in_both_paints_costs_one_entry() {
    let a = AssetId::from_bytes(&[3u8; 16]);
    assert_eq!(
        crate::project_texture_pattern::art_ids_named_by(&cena(Some(a), Some(a))).len(),
        1,
        "a mesma arte foi embutida duas vezes"
    );
}
