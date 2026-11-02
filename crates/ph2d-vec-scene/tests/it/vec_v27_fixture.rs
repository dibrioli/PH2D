//! ⭐⭐ **Uma cena gravada no degrau 27 abre com a MESMA aparência** (F47, o braço `V27` do
//! `from_bytes`; `StrokeSpec::above_effects` no FIM desde a v28).
//!
//! Os bytes de `fixtures/vec_scene_v27_traco.bin` foram gravados pelo código v27 (HEAD `d95d9a1bea`,
//! `VecScene::to_bytes`, sha256 `50d3b34f…`) — ⛔ nunca os regenere. A cena: «caixa» retângulo
//! `(0,0)..(40,30)` com fill `(10,20,30)`, contorno de base `(200,100,50)` de `2,5` alinhado por FORA,
//! uma camada Stroke `(0,128,255)` de `6` com id `7`, e o fill de base movido para cima do traço.

use ph2d_item_id::ItemId;
use ph2d_vec_scene::{
    Paint, PaintKind, Rgba8, StackItem, StrokeAlign, VEC_SCENE_SCHEMA_VERSION, VecScene,
};

const V27: &[u8] = include_bytes!("fixtures/vec_scene_v27_traco.bin");

#[test]
fn a_vec_scene_v27_abre_com_os_tracos_sob_os_filtros() {
    assert_eq!(V27[0], 27, "a fixtura é v27");
    let cena = VecScene::from_bytes(V27).expect("um v27 abre");
    let [caixa] = cena.paths() else {
        panic!("uma forma, lidas {}", cena.paths().len());
    };
    assert_eq!(caixa.fill, Some(Paint::solid(Rgba8::new(10, 20, 30, 255))));
    let base = caixa.stroke.as_ref().expect("o traço de base");
    assert_eq!(
        (base.color(), base.width, base.align, base.above_effects),
        (
            Rgba8::new(200, 100, 50, 255),
            2.5,
            StrokeAlign::Outer,
            false
        )
    );
    let [t] = caixa.paints.as_slice() else {
        panic!("uma camada");
    };
    let PaintKind::Stroke(s) = &t.kind else {
        panic!("a camada é um traço");
    };
    assert_eq!((s.width, s.above_effects, t.id), (6.0, false, ItemId(7)));
    assert_eq!(
        caixa.draw_order().collect::<Vec<_>>(),
        [
            StackItem::BaseStroke,
            StackItem::BaseFill,
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
fn os_bytes_v27_sob_o_cabecalho_v28_nao_abrem_igual() {
    let certo = VecScene::from_bytes(V27).expect("um v27 abre");
    let mut hoje = V27.to_vec();
    hoje[0] = u8::try_from(VEC_SCENE_SCHEMA_VERSION).unwrap();
    assert!(
        VecScene::from_bytes(&hoje).ok().is_none_or(|c| c != certo),
        "lido como v28, o v27 deu a mesma cena — o gate de cima é cego"
    );
}

/// O «por cima dos Filtros» sobrevive ao ficheiro (v28), na base e numa camada.
#[test]
fn o_traco_por_cima_dos_filtros_faz_a_volta_pelo_ficheiro() {
    let mut cena = VecScene::from_bytes(V27).expect("um v27 abre");
    let caixa = &mut cena.paths_mut()[0];
    caixa.stroke.as_mut().unwrap().above_effects = true;
    if let PaintKind::Stroke(s) = &mut caixa.paints[0].kind {
        s.above_effects = true;
    }
    let lida = VecScene::from_bytes(&cena.to_bytes().expect("grava")).expect("abre");
    assert_eq!(lida, cena);
    assert!(lida.paths()[0].stroke.as_ref().unwrap().above_effects);
}
