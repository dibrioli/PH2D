//! ⭐⭐ **Uma cena gravada no degrau 28 abre igual e sem estilos de cor** (CartoesUI D28/D30, o braço
//! `V25..=V28` do `from_bytes`; `estilos`/`next_estilo` no FIM da `VecScene` desde a v29).
//!
//! Os bytes de `fixtures/vec_scene_v28_traco.bin` foram gravados pelo código v28 (`line/NovaUI`
//! `11b6952897`, `VecScene::to_bytes`, sha256 `919aa42d…`) — ⛔ nunca os regenere. A cena: retângulo
//! `(0,0)..(40,30)` com fill `(10,20,30)`, contorno de base `(200,100,50)` de `2,5` por cima dos
//! Filtros, e uma camada Stroke `(0,128,255)` de `6` com id `7`.

use ph2d_item_id::ItemId;
use ph2d_vec_scene::{Paint, PaintKind, Rgba8, VEC_SCENE_SCHEMA_VERSION, VecScene};

const V28: &[u8] = include_bytes!("fixtures/vec_scene_v28_traco.bin");

#[test]
fn a_vec_scene_v28_abre_sem_estilos_de_cor() {
    assert_eq!(V28[0], 28, "a fixtura é v28");
    let cena = VecScene::from_bytes(V28).expect("um v28 abre");
    assert!(cena.estilos().is_empty());
    let [caixa] = cena.paths() else {
        panic!("uma forma, lidas {}", cena.paths().len());
    };
    assert_eq!(caixa.fill, Some(Paint::solid(Rgba8::new(10, 20, 30, 255))));
    let base = caixa.stroke.as_ref().expect("o traço de base");
    assert_eq!(
        (base.color(), base.width, base.above_effects),
        (Rgba8::new(200, 100, 50, 255), 2.5, true)
    );
    let [t] = caixa.paints.as_slice() else {
        panic!("uma camada");
    };
    let PaintKind::Stroke(s) = &t.kind else {
        panic!("a camada é um traço");
    };
    assert_eq!((s.width, t.id), (6.0, ItemId(7)));

    let nova = cena.to_bytes().expect("grava");
    assert_eq!(u32::from(nova[0]), VEC_SCENE_SCHEMA_VERSION);
    assert_eq!(
        VecScene::from_bytes(&nova).expect("o degrau novo abre"),
        cena
    );
}

/// ⛔ CONTROLO: os MESMOS bytes com o cabeçalho de hoje lêem-se pela forma viva e não abrem.
#[test]
fn os_bytes_v28_sob_o_cabecalho_de_hoje_nao_abrem() {
    let mut hoje = V28.to_vec();
    hoje[0] = u8::try_from(VEC_SCENE_SCHEMA_VERSION).unwrap();
    assert!(
        VecScene::from_bytes(&hoje).is_err(),
        "lido como a forma viva, o v28 abriu — o gate de cima é cego"
    );
}

/// Os estilos e as ligações sobrevivem ao ficheiro.
#[test]
fn os_estilos_fazem_a_volta_pelo_ficheiro() {
    let mut cena = VecScene::from_bytes(V28).expect("um v28 abre");
    let path = cena.paths()[0].id;
    let id = cena.add_estilo("Grass", Rgba8::new(1, 2, 3, 255));
    assert!(cena.link_estilo((path, ItemId(7)), id));
    let lida = VecScene::from_bytes(&cena.to_bytes().expect("grava")).expect("abre");
    assert_eq!(lida, cena);
    assert_eq!(lida.usos_vivos(id), 1);
}
