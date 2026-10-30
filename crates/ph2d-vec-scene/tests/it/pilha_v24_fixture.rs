//! ⭐⭐ **Uma cena gravada no degrau 24 (id por vértice e regras de CAD, pilha ainda sem ids) abre
//! igual, com os ids da pilha por atribuir** (F40, o braço `V24` do `from_bytes`).
//!
//! Os bytes de `fixtures/vec_scene_v24_pilha_cad.bin` foram gravados pelo código v24 (commit
//! `5228501c26`, o main antes do degrau 25) — ⛔ nunca os regenere. A cena: «caixa» retângulo com duas
//! tintas (preenchimento deslocado `(0,25, −0,5)` a `0,6`; contorno `0,75` desligado com `dilate 0,3`) e
//! dois efeitos (Trim `0,1..0,8` desvio `0,2`; ZigZag `12`/`7` desligado); «dentes», 4 quinas abertas;
//! uma elipse com uma tinta; duas regras (Horizontal entre os vértices 1 e 2 dos dentes; Raio `8` da
//! elipse, de referência).

use ph2d_item_id::ItemId;
use ph2d_vec_scene::cad::{Lei, PontoCad, RegraId, SegCad};
use ph2d_vec_scene::effect::PathEffect;
use ph2d_vec_scene::{PaintKind, VEC_SCENE_SCHEMA_VERSION, VecScene, VertId};

const V24: &[u8] = include_bytes!("fixtures/vec_scene_v24_pilha_cad.bin");

#[test]
fn a_vec_scene_v24_le_com_vertices_regras_e_pilha_por_atribuir() {
    assert_eq!(V24[0], 24, "a fixtura é v24");
    let cena = VecScene::from_bytes(V24).expect("um v24 abre");
    let [caixa, dentes, elipse] = cena.paths() else {
        panic!("três formas, lidas {}", cena.paths().len());
    };

    let [t0, t1] = caixa.paints.as_slice() else {
        panic!("duas tintas");
    };
    assert!(matches!(t0.kind, PaintKind::Fill(_)));
    assert_eq!(
        (t0.offset, t0.opacity.get(), t0.enabled),
        ([0.25, -0.5], 0.6, true)
    );
    let PaintKind::Stroke(s) = &t1.kind else {
        panic!("a 2.ª é um contorno");
    };
    assert_eq!((s.width, t1.enabled, t1.dilate), (0.75, false, 0.3));
    let [trim, zig] = caixa.effects.as_slice() else {
        panic!("dois efeitos");
    };
    let PathEffect::Trim(t) = &trim.effect else {
        panic!("Trim primeiro");
    };
    assert_eq!(
        (t.start, t.end, t.offset, trim.enabled),
        (0.1, 0.8, 0.2, true)
    );
    let PathEffect::ZigZag(z) = &zig.effect else {
        panic!("ZigZag depois");
    };
    assert_eq!((z.amplitude, z.ridges, zig.enabled), (12.0, 7.0, false));
    assert_eq!(elipse.paints.len(), 1);

    // O que o 24 trouxe fica: os ids dos vértices e as regras com os seus ids.
    let ids: Vec<VertId> = dentes.verts.iter().map(|v| v.id).collect();
    assert_eq!(ids, [1, 2, 3, 4].map(VertId));
    let [h, r] = cena.regras() else {
        panic!("duas regras, lidas {}", cena.regras().len());
    };
    assert_eq!(
        h.lei,
        Lei::Horizontal(SegCad(
            PontoCad::Vert(dentes.id, VertId(1)),
            PontoCad::Vert(dentes.id, VertId(2)),
        ))
    );
    assert_eq!((h.id, h.referencia), (RegraId(1), false));
    assert_eq!(r.lei, Lei::Raio(elipse.id, 8.0));
    assert_eq!((r.id, r.referencia), (RegraId(2), true));

    // ⭐ Os ids da pilha ficam POR ATRIBUIR.
    let pilha: Vec<ItemId> = cena
        .paths()
        .iter()
        .flat_map(|p| {
            p.paints
                .iter()
                .map(|e| e.id)
                .chain(p.effects.iter().map(|e| e.id))
        })
        .collect();
    assert_eq!(pilha.len(), 5);
    assert!(pilha.iter().all(|i| i.is_none()), "{pilha:?}");

    let nova = cena.to_bytes().expect("grava");
    assert_eq!(nova[0], u8::try_from(VEC_SCENE_SCHEMA_VERSION).unwrap());
    assert_eq!(
        VecScene::from_bytes(&nova).expect("o degrau novo abre"),
        cena
    );
}
