//! O nome único das imagens da biblioteca ([`super`]).

use super::*;
use crate::asset_card_art::CardArt;
use crate::asset_index_build::build;
use ph2d_asset_index::AssetRef;

fn id(n: u8) -> AssetId {
    AssetId::from_digest([n; 32])
}

/// ⭐ Dois `door.png` em pastas diferentes: `door` e `door (2)`, pela ordem do caminho; um
/// `door (2).png` verdadeiro guarda o nome dele e empurra o repetido para `door (3)`.
#[test]
fn dois_ficheiros_com_o_mesmo_nome_ganham_um_sufixo() {
    let paths: BTreeMap<AssetId, PathBuf> = [
        (id(3), "b/door.png"),
        (id(1), "a/door.png"),
        (id(2), "c/door (2).png"),
        (id(4), "c/door.webp"),
    ]
    .into_iter()
    .map(|(i, p)| (i, PathBuf::from(p)))
    .collect();
    let mut ids: BTreeMap<AssetId, Option<String>> = paths
        .keys()
        .map(|i| (*i, Some("ignorado".into())))
        .collect();
    ids.insert(id(5), Some("Torso".into()));
    ids.insert(id(6), None);
    let n = unique(&paths, &ids);
    assert_eq!(n[&id(1)], "door", "a/ vem primeiro");
    assert_eq!(n[&id(3)], "door (3)", "b/: o (2) já é de outro ficheiro");
    assert_eq!(n[&id(4)], "door (4)", "a extensão não separa");
    assert_eq!(n[&id(2)], "door (2)");
    assert_eq!(n[&id(5)], "Torso", "sem ficheiro: o nome de quem a usa");
    assert_eq!(n[&id(6)], id(6).to_hex()[..12], "sem nada: o blake3");
    let mut todos: Vec<&String> = n.values().collect();
    todos.sort();
    todos.dedup();
    assert_eq!(todos.len(), n.len(), "únicos: {n:?}");
    // Determinista: a mesma resposta em outra ordem de entrada.
    let ids2: BTreeMap<AssetId, Option<String>> = ids.into_iter().rev().collect();
    assert_eq!(unique(&paths, &ids2), n);
}

#[test]
fn o_nome_base_nao_tem_pasta_nem_extensao() {
    assert_eq!(
        stem(Path::new("/x/y/door_open.png")).as_deref(),
        Some("door_open")
    );
    assert_eq!(stem(Path::new("door.tar.png")).as_deref(), Some("door.tar"));
}

/// ⭐ O painel e o script veem o MESMO nome: o universo inclui o que o artista trouxe sem uso
/// (`atlas_assets`) e tira o que ele mandou sair e ninguém usa.
#[test]
fn o_painel_e_o_script_dao_o_mesmo_nome() {
    let db = AssetDb::new();
    let a = db.insert_image_rgba8(1, 1, vec![255, 0, 0, 255]);
    let b = db.insert_image_rgba8(1, 1, vec![0, 255, 0, 255]);
    let trazida = db.insert_image_rgba8(1, 1, vec![0, 0, 255, 255]);
    let mut sim = SimWorld::new();
    for (sid, px) in [(1, a), (2, b)] {
        sim.world_mut()
            .spawn((Name::new("Door"), StableId(sid), SpritePixels(px)));
    }
    let atlas: BTreeMap<u32, AssetId> = [(9, trazida)].into_iter().collect();
    let mut lib = TextureLibrary::default();
    let index = build(
        &mut sim,
        &db,
        &atlas,
        &ph2d_asset_index::CatalogTree::new(),
        &mut CardArt::new(),
        &mut lib,
        100.0,
    );
    let names = library(&mut sim, &db, &atlas, &lib);
    // Sem caminho, o desempate é o blake3 (determinista).
    let mut par = [names[&a].as_str(), names[&b].as_str()];
    par.sort_unstable();
    assert_eq!(par, ["Door", "Door (2)"]);
    assert!(
        names.contains_key(&trazida),
        "trazida e sem uso: o script alcança-a"
    );
    for e in index.entries() {
        if let AssetRef::Texture { asset } = e.key {
            assert_eq!(e.name, names[&AssetId::from_digest(asset)], "o painel");
        }
    }
    lib.forget(trazida);
    assert!(
        !library(&mut sim, &db, &atlas, &lib).contains_key(&trazida),
        "mandada sair e sem uso"
    );
}

/// ⭐ **Editar não muda o nome da imagem** (dono 05/11: a «Platform» editada no Pixel não vira «Platform Low»): o nome é o
/// da origem, e ela sai da lista enquanto só a edição estiver em uso. Desfeita a edição, tudo volta.
#[test]
fn a_imagem_editada_herda_o_nome_da_de_onde_veio() {
    let id_p = |n| AssetId::from_digest([n; 32]);
    let (raiz, e1, e2) = (id_p(1), id_p(2), id_p(3));
    let origens: BTreeMap<_, _> = [(e1, raiz), (e2, e1)].into_iter().collect();
    let sem_caminhos = BTreeMap::new();
    let mut ids: BTreeMap<AssetId, Option<String>> = [
        (raiz, Some("Platform".to_owned())),
        (e2, Some("Platform Low".to_owned())),
    ]
    .into_iter()
    .collect();
    let live: BTreeSet<AssetId> = [e2].into_iter().collect();
    herda_o_nome(&mut ids, &live, &sem_caminhos, &origens);
    let n = unique(&sem_caminhos, &ids);
    assert_eq!(n[&e2], "Platform", "duas edições depois, o nome da raiz");
    assert!(!n.contains_key(&raiz), "a edição toma o lugar: {n:?}");
    // Com a origem ainda em uso são duas imagens: a regra dos repetidos separa-as.
    let mut ids: BTreeMap<AssetId, Option<String>> = [
        (raiz, Some("Platform".to_owned())),
        (e1, Some("Platform Low".to_owned())),
    ]
    .into_iter()
    .collect();
    let live: BTreeSet<AssetId> = [raiz, e1].into_iter().collect();
    herda_o_nome(&mut ids, &live, &sem_caminhos, &origens);
    assert_eq!(unique(&sem_caminhos, &ids).len(), 2);
    // Desfeito: a edição não está em uso e nada muda.
    let mut ids: BTreeMap<AssetId, Option<String>> =
        [(raiz, Some("Platform".to_owned()))].into_iter().collect();
    let live: BTreeSet<AssetId> = [raiz].into_iter().collect();
    herda_o_nome(&mut ids, &live, &sem_caminhos, &origens);
    assert_eq!(unique(&sem_caminhos, &ids)[&raiz], "Platform");
}

/// De ponta a ponta: pela `library`, com a imagem de fábrica de verdade e a troca de píxeis do cartão.
#[test]
fn a_platform_editada_nao_vira_platform_low() {
    let db = AssetDb::default();
    let raiz = db.insert_image_rgba8(1, 1, vec![9, 9, 9, 255]);
    let chave = ph2d_render::PLATFORM_TILE_KEY;
    let atlas: BTreeMap<u32, AssetId> = [(chave, raiz)].into_iter().collect();
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let sprite = || ph2d_render::Sprite::atlas(chave, [1.0, 1.0], [1.0; 4]);
    let a = w
        .spawn((sprite(), StableId(1), Name::new("Platform Low")))
        .id();
    w.spawn((sprite(), StableId(2), Name::new("Platform High")));
    let lib = TextureLibrary::default();
    assert_eq!(library(&mut sim, &db, &atlas, &lib)[&raiz], "Platform");
    ph2d_sprite_edit::imagem::troca_os_pixeis(
        sim.world_mut(),
        &mut |_, _, _| Ok(77),
        &db,
        &atlas,
        a,
        (1, 1, vec![1, 2, 3, 255]),
    )
    .expect("troca");
    let novo = sim.world().get::<SpritePixels>(a).expect("px").0;
    assert_ne!(novo, raiz);
    let n = library(&mut sim, &db, &atlas, &lib);
    assert_eq!(n[&novo], "Platform");
    assert!(!n.contains_key(&raiz), "{n:?}");
}
