//! ⭐⭐ **Uma cena gravada ANTES do id das entradas da pilha abre igual, com os ids por atribuir**
//! (F31 §B.1).
//!
//! Os bytes de `fixtures/vec_scene_v23_pilha.bin` foram gravados pelo código v23 (commit
//! `727fc9a529`, antes do degrau) — ⛔ nunca os regenere. A cena: um retângulo com duas tintas
//! (preenchimento deslocado `(0,25, −0,5)` a `0,6`; contorno `0,75` desligado com `dilate 0,3`) e
//! dois efeitos (Trim `0,1..0,8` desvio `0,2`; ZigZag `12`/`7` desligado); uma elipse com uma tinta;
//! um retângulo liso.

use ph2d_item_id::ItemId;
use ph2d_vec_scene::effect::PathEffect;
use ph2d_vec_scene::{PaintKind, VEC_SCENE_SCHEMA_VERSION, VecScene};

const V23: &[u8] = include_bytes!("fixtures/vec_scene_v23_pilha.bin");

#[test]
fn a_vec_scene_v23_le_com_ids_por_atribuir() {
    assert_eq!(V23[0], 23, "a fixtura é v23");
    let cena = VecScene::from_bytes(V23).expect("um v23 abre");
    let [caixa, elipse, lisa] = cena.paths() else {
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
    assert!(lisa.paints.is_empty() && lisa.effects.is_empty());

    // ⭐ Os ids ficam POR ATRIBUIR (o passe do `ph2d-app-vec` dá-os no 1.º quadro).
    let ids: Vec<ItemId> = cena
        .paths()
        .iter()
        .flat_map(|p| {
            p.paints
                .iter()
                .map(|e| e.id)
                .chain(p.effects.iter().map(|e| e.id))
        })
        .collect();
    assert_eq!(ids.len(), 5);
    assert!(ids.iter().all(|i| i.is_none()), "{ids:?}");

    // E volta a gravar-se na forma NOVA, com os ids.
    let mut cena = cena;
    for (n, p) in cena.paths_mut().iter_mut().enumerate() {
        for (k, e) in p.paints.iter_mut().enumerate() {
            e.id = ItemId(u32::try_from(10 * n + k + 1).unwrap());
        }
    }
    let nova = cena.to_bytes().expect("grava");
    assert_eq!(nova[0], u8::try_from(VEC_SCENE_SCHEMA_VERSION).unwrap());
    assert_eq!(VecScene::from_bytes(&nova).expect("um v24 abre"), cena);
}

/// ⛔ CONTROLO: os mesmos bytes SEM o âmbito v23 não dão a mesma cena.
#[test]
fn sem_o_ambito_v23_os_mesmos_bytes_nao_leem_igual() {
    let certo = VecScene::from_bytes(V23).expect("um v23 abre");
    let torto = postcard::from_bytes::<(u32, VecScene)>(V23).map(|(_, c)| c);
    assert!(
        torto.as_ref() != Ok(&certo),
        "o âmbito não mudou nada — o gate é cego"
    );
}
