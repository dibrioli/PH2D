//! ⭐⭐ **Uma cena gravada no degrau 25 abre igual, sem tinta de base escondida** (F44b, o braço `V25`
//! do `from_bytes`; `VecPath::fill_hidden`/`stroke_hidden` no FIM desde a v26).
//!
//! Os bytes de `fixtures/vec_scene_v25_tinta.bin` foram gravados pelo código v25 (commit
//! `d48b76b8fd`, numa worktree temporária, `VecScene::to_bytes`) — ⛔ nunca os regenere. A cena:
//! «caixa» retângulo `(0,0)..(40,30)` com fill `(10,20,30)`, contorno `(200,100,50)` de `2,5` e uma
//! tinta Fill `(0,128,255)` deslocada `(1,5, −2)` com id `7`; uma elipse `(100,50)` `20×10` sem fill,
//! contorno `(1,2,3)` de `4`.

use ph2d_item_id::ItemId;
use ph2d_vec_scene::{
    Paint, PaintKind, Rgba8, StrokeSpec, VEC_SCENE_SCHEMA_VERSION, VecScene, rectangle,
};

const V25: &[u8] = include_bytes!("fixtures/vec_scene_v25_tinta.bin");

#[test]
fn a_vec_scene_v25_le_sem_tinta_escondida() {
    assert_eq!(V25[0], 25, "a fixtura é v25");
    let cena = VecScene::from_bytes(V25).expect("um v25 abre");
    let [caixa, elipse] = cena.paths() else {
        panic!("duas formas, lidas {}", cena.paths().len());
    };
    assert_eq!(caixa.fill, Some(Paint::solid(Rgba8::new(10, 20, 30, 255))));
    let s = caixa.stroke.as_ref().expect("a caixa tem contorno");
    assert_eq!((s.width, s.color()), (2.5, Rgba8::new(200, 100, 50, 255)));
    let [t] = caixa.paints.as_slice() else {
        panic!("uma tinta");
    };
    assert_eq!(
        t.kind,
        PaintKind::Fill(Paint::solid(Rgba8::new(0, 128, 255, 255)))
    );
    assert_eq!((t.offset, t.id), ([1.5, -2.0], ItemId(7)));
    assert!(elipse.fill.is_none());
    assert_eq!(elipse.stroke.as_ref().map(|s| s.width), Some(4.0));
    // ⭐ O que a v26 trouxe nasce vazio: nada escondido.
    for p in cena.paths() {
        assert_eq!(
            (&p.fill_hidden, &p.stroke_hidden),
            (&None, &None),
            "{}",
            p.id
        );
    }

    let nova = cena.to_bytes().expect("grava");
    assert_eq!(nova[0], u8::try_from(VEC_SCENE_SCHEMA_VERSION).unwrap());
    assert_eq!(
        VecScene::from_bytes(&nova).expect("o degrau novo abre"),
        cena
    );
}

/// ⛔ CONTROLO: os MESMOS bytes com o cabeçalho de hoje lêem-se pela forma viva e não abrem igual.
#[test]
fn os_bytes_v25_sob_o_cabecalho_v26_nao_abrem_igual() {
    let certo = VecScene::from_bytes(V25).expect("um v25 abre");
    let mut hoje = V25.to_vec();
    hoje[0] = u8::try_from(VEC_SCENE_SCHEMA_VERSION).unwrap();
    assert!(
        VecScene::from_bytes(&hoje).ok().is_none_or(|c| c != certo),
        "lido como v26, o v25 deu a mesma cena — o gate de cima é cego"
    );
}

/// A tinta de base escondida pelo olho sobrevive ao ficheiro (v26).
#[test]
fn a_tinta_escondida_faz_a_volta_pelo_ficheiro() {
    let mut cena = VecScene::new();
    let mut p = rectangle([0.0, 0.0], [10.0, 10.0]);
    p.fill = None;
    p.fill_hidden = Some(Paint::solid(Rgba8::new(9, 8, 7, 255)));
    p.stroke = None;
    p.stroke_hidden = Some(StrokeSpec::new(Rgba8::new(4, 5, 6, 255), 3.0));
    cena.push_path(p);
    let b = cena.to_bytes().expect("grava");
    assert_eq!(u32::from(b[0]), VEC_SCENE_SCHEMA_VERSION);
    let lida = VecScene::from_bytes(&b).expect("abre");
    assert_eq!(lida, cena);
    let [q] = lida.paths() else {
        panic!("uma forma")
    };
    assert_eq!(q.fill_hidden, Some(Paint::solid(Rgba8::new(9, 8, 7, 255))));
    assert_eq!(q.stroke_hidden.as_ref().map(|s| s.width), Some(3.0));
}
