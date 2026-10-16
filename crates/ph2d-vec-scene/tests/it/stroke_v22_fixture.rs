//! ⭐⭐ **Uma cena gravada ANTES do tracejado em pares abre igual** (F16c, ordem do dono: *«ficheiros
//! antigos têm de continuar a abrir iguais»*).
//!
//! Os bytes de `fixtures/vec_scene_v22_traco.bin` foram gravados pelo código v22 (commit `d138b96401`,
//! antes da mudança do `StrokeSpec`) — ⛔ **nunca os regenere**: um gate que relê o que o código de
//! hoje escreve só prova que o código concorda consigo mesmo. A cena: uma linha tracejada `(3, 1,5)`
//! com Triangle/Circle, `marker_scale 1,7` e `marker_round 0,25`; um retângulo tracejado `(2, 1)` com
//! uma camada de contorno `(1, 0,5)` e `marker_scale 2,5`; um retângulo liso.

use ph2d_vec_scene::{
    DashPattern, LineCap, Marker, VEC_SCENE_SCHEMA_VERSION, VecScene, paint_stack::PaintKind,
};

const V22: &[u8] = include_bytes!("fixtures/vec_scene_v22_traco.bin");

#[test]
fn a_v22_scene_opens_the_same() {
    let cena = VecScene::from_bytes(V22).expect("um v22 abre");
    let [seta, caixa, lisa] = cena.paths() else {
        panic!("três formas, lidas {}", cena.paths().len());
    };

    let s = seta.stroke.as_ref().expect("a seta tem traço");
    assert_eq!(
        s.dash,
        Some(DashPattern::single(3.0, 1.5)),
        "o par antigo é o 1.º par"
    );
    assert_eq!(
        (s.marker_scale_start, s.marker_scale_end),
        (1.7, 1.7),
        "o tamanho único vai às duas"
    );
    assert_eq!(
        (s.marker_start, s.marker_end),
        (Marker::Triangle, Marker::Circle)
    );
    assert_eq!(
        (s.marker_round, s.cap, s.width),
        (0.25, LineCap::Round, 0.5)
    );
    // ⭐ O que se DESENHA: o mesmo par em comprimento que o v22 dava (`[d·w, g·w]`).
    assert_eq!(s.dash_lengths().expect("tracejada"), [1.5, 0.75]);

    let s = caixa.stroke.as_ref().expect("a caixa tem traço");
    assert_eq!(s.dash, Some(DashPattern::single(2.0, 1.0)));
    let [camada] = caixa.paints.as_slice() else {
        panic!("uma camada na pilha");
    };
    let PaintKind::Stroke(c) = &camada.kind else {
        panic!("a camada é um contorno");
    };
    assert_eq!(c.dash, Some(DashPattern::single(1.0, 0.5)));
    assert_eq!((c.marker_scale_start, c.marker_scale_end), (2.5, 2.5));

    let s = lisa.stroke.as_ref().expect("a lisa tem traço");
    assert_eq!(
        (s.dash, s.marker_scale_start, s.marker_scale_end),
        (None, 1.0, 1.0)
    );

    // E volta a gravar-se na forma NOVA, sem perder nada.
    let nova = cena.to_bytes().expect("grava");
    assert_eq!(nova[0], u8::try_from(VEC_SCENE_SCHEMA_VERSION).unwrap());
    assert_eq!(VecScene::from_bytes(&nova).expect("um v23 abre"), cena);
}

/// ⛔ CONTROLO: os mesmos bytes lidos SEM o âmbito v22 não dão a mesma cena — é o âmbito que a
/// prova de cima mede, e não um acaso do formato.
#[test]
fn without_the_v22_scope_the_same_bytes_do_not_read_the_same() {
    let certo = VecScene::from_bytes(V22).expect("um v22 abre");
    let torto = postcard::from_bytes::<(u32, VecScene)>(V22).map(|(_, c)| c);
    assert!(
        torto.as_ref() != Ok(&certo),
        "o âmbito não mudou nada — o gate de cima é cego"
    );
}
