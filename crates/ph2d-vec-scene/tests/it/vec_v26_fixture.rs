//! ⭐⭐ **Uma cena gravada no degrau 26 abre com a MESMA aparência** (F45, o braço `V26` do
//! `from_bytes`; `VecPath::base_order` no FIM desde a v27).
//!
//! Os bytes de `fixtures/vec_scene_v26_ordem.bin` foram gravados pelo código v26 (commit
//! `a020be7be2`, numa worktree temporária, `VecScene::to_bytes`) — ⛔ nunca os regenere. A cena:
//! «caixa» retângulo `(0,0)..(40,30)` com fill `(10,20,30)`, contorno `(200,100,50)` de `2,5` e uma
//! camada Stroke `(0,128,255)` de `6` com id `7`; uma elipse `(100,50)` `20×10` sem fill, com o
//! contorno `(1,2,3)` de `4` ESCONDIDO pelo olho.

use ph2d_item_id::ItemId;
use ph2d_vec_scene::{
    BaseOrder, Paint, Place, Rgba8, StackItem, VEC_SCENE_SCHEMA_VERSION, VecScene, rectangle,
};

const V26: &[u8] = include_bytes!("fixtures/vec_scene_v26_ordem.bin");

#[test]
fn a_vec_scene_v26_abre_na_ordem_de_antes() {
    assert_eq!(V26[0], 26, "a fixtura é v26");
    let cena = VecScene::from_bytes(V26).expect("um v26 abre");
    let [caixa, elipse] = cena.paths() else {
        panic!("duas formas, lidas {}", cena.paths().len());
    };
    assert_eq!(caixa.fill, Some(Paint::solid(Rgba8::new(10, 20, 30, 255))));
    let [t] = caixa.paints.as_slice() else {
        panic!("uma camada");
    };
    assert_eq!((t.width(), t.id), (Some(6.0), ItemId(7)));
    assert_eq!(elipse.stroke_hidden.as_ref().map(|s| s.width), Some(4.0));
    // ⭐ A mesma aparência: base fill, base stroke, depois a camada.
    for p in cena.paths() {
        assert_eq!(p.base_order, BaseOrder::default(), "{}", p.id);
    }
    assert_eq!(
        caixa.draw_order().collect::<Vec<_>>(),
        [
            StackItem::BaseFill,
            StackItem::BaseStroke,
            StackItem::Entry(0)
        ]
    );

    let nova = cena.to_bytes().expect("grava");
    assert_eq!(u32::from(nova[0]), VEC_SCENE_SCHEMA_VERSION);
    assert_eq!(
        VecScene::from_bytes(&nova).expect("o degrau novo abre"),
        cena
    );
}

/// ⛔ CONTROLO: os MESMOS bytes com o cabeçalho de hoje lêem-se pela forma viva e não abrem igual.
#[test]
fn os_bytes_v26_sob_o_cabecalho_v27_nao_abrem_igual() {
    let certo = VecScene::from_bytes(V26).expect("um v26 abre");
    let mut hoje = V26.to_vec();
    hoje[0] = u8::try_from(VEC_SCENE_SCHEMA_VERSION).unwrap();
    assert!(
        VecScene::from_bytes(&hoje).ok().is_none_or(|c| c != certo),
        "lido como v27, o v26 deu a mesma cena — o gate de cima é cego"
    );
}

/// A ordem que o artista pôs sobrevive ao ficheiro (v27).
#[test]
fn a_ordem_da_pilha_faz_a_volta_pelo_ficheiro() {
    let mut cena = VecScene::new();
    let mut p = rectangle([0.0, 0.0], [10.0, 10.0]);
    p.fill = Some(Paint::solid(Rgba8::new(9, 8, 7, 255)));
    assert!(p.move_paint(StackItem::BaseFill, Place::Above(StackItem::BaseStroke)));
    cena.push_path(p);
    let lida = VecScene::from_bytes(&cena.to_bytes().expect("grava")).expect("abre");
    assert_eq!(lida, cena);
    assert_eq!(
        lida.paths()[0].draw_order().collect::<Vec<_>>(),
        [StackItem::BaseStroke, StackItem::BaseFill]
    );
}
