//! ⭐⭐ **Uma cena gravada ANTES do id por vértice abre igual, já numerada** (W4.2).
//!
//! Os bytes de `fixtures/vec_scene_v23_geometria.bin` foram gravados pelo código v23 (commit
//! `db09f4bd45`, antes do `VertId`) — ⛔ **nunca os regenere**. A cena: «dentes», 10 vértices abertos
//! (raio `2,5` no 4.º, um Smooth no 7.º, traço `1,25`); «caixa», 4 vértices fechados com um buraco de
//! 3 em `subpaths`, EvenOdd.

use ph2d_vec_scene::{FillRule, VEC_SCENE_SCHEMA_VERSION, VecScene, VertId, VertexKind};

const V23: &[u8] = include_bytes!("fixtures/vec_scene_v23_geometria.bin");

#[test]
fn a_v23_scene_opens_with_ids_in_order() {
    let cena = VecScene::from_bytes(V23).expect("um v23 abre");
    let [dentes, caixa] = cena.paths() else {
        panic!("duas formas, lidas {}", cena.paths().len());
    };

    let ids: Vec<u32> = dentes.verts.iter().map(|v| v.id.0).collect();
    assert_eq!(ids, (1..=10).collect::<Vec<_>>());
    assert_eq!(dentes.next_vert, 11);
    assert_eq!(dentes.verts[3].corner_radius, 2.5);
    assert_eq!(dentes.verts[6].kind, VertexKind::Smooth);
    assert_eq!(dentes.verts[6].in_handle, [57.0, -2.0]);
    assert_eq!(dentes.verts[9].anchor, [90.0, 7.5]);
    assert_eq!(dentes.stroke.as_ref().expect("traço").width, 1.25);

    let ids: Vec<u32> = caixa
        .verts
        .iter()
        .chain(&caixa.subpaths[0].verts)
        .map(|v| v.id.0)
        .collect();
    assert_eq!(
        ids,
        (1..=7).collect::<Vec<_>>(),
        "o buraco continua a numeração"
    );
    assert_eq!(caixa.subpaths[0].verts[2].anchor, [20.0, 25.0]);
    assert_eq!(caixa.fill_rule, FillRule::EvenOdd);
    assert!(caixa.vert_by_id(VertId(6)).is_some());

    // E volta a gravar-se na forma NOVA, sem perder nada.
    let nova = cena.to_bytes().expect("grava");
    assert_eq!(nova[0], u8::try_from(VEC_SCENE_SCHEMA_VERSION).unwrap());
    assert_eq!(VecScene::from_bytes(&nova).expect("um v24 abre"), cena);
}

/// ⛔ CONTROLO: os mesmos bytes lidos com a forma VIVA não dão a mesma cena.
#[test]
fn without_the_v23_scope_the_same_bytes_do_not_read_the_same() {
    let certo = VecScene::from_bytes(V23).expect("um v23 abre");
    let torto = postcard::from_bytes::<(u32, VecScene)>(V23).map(|(_, c)| c);
    assert!(
        torto.as_ref() != Ok(&certo),
        "o âmbito não mudou nada — o gate de cima é cego"
    );
}
