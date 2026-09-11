//! **O canal de Morph é AUTORÁVEL** — os dois defeitos do smoke do Enio (2026-07-28):
//! *"nem autokey funcionou para morph e nem tem o track disponível no dropdown de tracks"*.
//!
//! O C4 (ADR-0152) fez o Morph ser LIDO de volta do mundo. Mas ler não é autorar, e o
//! artista não tinha gesto nenhum para criar a track: a cena do smoke a montou
//! programaticamente, então a wave anterior demonstrou o fix por um caminho que o
//! produto não oferece.
//!
//! Duas causas, de naturezas diferentes:
//!
//! 1. **A lista "+ Track" não tinha Morph.** O doc-comment de [`PropKind::Morph`] explica
//!    que ele fica fora do `PropKind::ALL` porque `ALL` é a pose do sprite, e conclui
//!    *"the artist keys it from the '+ Track' list"* — uma lista que não o oferecia. Tudo
//!    a jusante estava pronto (rótulo, chave i18n, altura do popup derivada do `len()`);
//!    só a linha da tabela nunca entrou. **A porta que o design nomeia não existia.**
//!
//! 2. **O auto-key não o amostrava.** `PoseSample` tinha a forma do `ALL` (6), então o
//!    laço do diff nunca via o `t`. ⚠️ E a cerca do `Morph` era ambígua sobre isso: estar
//!    fora do `ALL` explica por que ele não está no ARRAY DA POSE — não que ele não deva
//!    ser autokeyado. [`PropKind::Position`] já provava a diferença (também fora do `ALL`,
//!    autokeyado por um ramo de geometria 2D), e o shell já dizia, no
//!    `sample_prop_value`, que o `t` *"IS a scene value, so K captures it the same way it
//!    captures a pose"* — ou seja, o K manual capturava e o auto-key não. Dois gestos de
//!    autoria discordando sobre o mesmo canal.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_timeline::{PropKind, TimelineDoc, autokey_props};

/// **Com o auto-key armado, mexer o `t` do morph grava uma key.**
///
/// O `t` entra no `PoseSample` no índice de [`PropKind::AUTOKEYED`]; o diff é o MESMO
/// dos canais escalares da pose (nada de segundo caminho — duplicar a pergunta *"isto se
/// moveu?"* é a doença que o C4 acabou de curar um módulo adiante).
#[test]
fn arming_auto_key_and_moving_the_morph_t_writes_a_key() {
    const E: u64 = 7;
    let morph_i = PropKind::AUTOKEYED
        .iter()
        .position(|p| *p == PropKind::Morph)
        .expect("Morph tem de estar na lista que o auto-key varre");

    let mut doc = TimelineDoc::new();
    // Uma track de Morph JA existe (o artista a criou pelo "+ Track") e vale 0.20.
    doc.insert_key(
        E,
        PropKind::Morph,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(0.20),
        Interp::Linear,
    );

    // O artista arrasta o slider "Morph t" para 0.80 no playhead 1.0.
    let mut world = [None; 7];
    world[morph_i] = Some(0.80_f32);
    let baseline = [None; 7];

    let plan = autokey_props(&doc, E, 1.0, &world, &baseline, true, false);
    assert_eq!(
        plan.keys,
        vec![(PropKind::Morph, 0.80)],
        "a pose saiu da curva (0.20 -> 0.80) e o auto-key tem de gravar"
    );
}

/// E o contrapositivo, que é o que impede o gate acima de ser satisfeito por um
/// auto-key que grava sempre: **na curva, ninguém keya.**
///
/// É a garantia anti-realimentação que todo canal escalar tem — sem ela, arrastar a
/// régua com o auto-key armado mintaria uma key de morph por quadro.
#[test]
fn a_morph_sitting_on_its_curve_mints_no_key() {
    const E: u64 = 7;
    let morph_i = PropKind::AUTOKEYED
        .iter()
        .position(|p| *p == PropKind::Morph)
        .expect("Morph na lista do auto-key");

    let mut doc = TimelineDoc::new();
    doc.insert_key(
        E,
        PropKind::Morph,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(0.25),
        Interp::Linear,
    );
    doc.insert_key(
        E,
        PropKind::Morph,
        RationalTime::from_seconds(2.0),
        AnimValue::Float(0.75),
        Interp::Linear,
    );

    // ⚠️ 0.25 / 0.75 / 0.5 são EXATOS em `f32`, e a escolha é deliberada: o diff compara
    // por igualdade de bits, então uma fixture com 0.20 -> 0.60 keya no meio da rampa por
    // erro de arredondamento e acusaria o produto por um defeito da própria fixture. É a
    // armadilha que o doc do `autokey.rs` narra ("a test scrubs to 0.5, and 0.5 survives
    // the round-trip") vista do outro lado.
    let mut world = [None; 7];
    world[morph_i] = Some(0.5_f32);
    let plan = autokey_props(&doc, E, 1.0, &world, &[None; 7], true, false);
    assert!(
        plan.keys.is_empty(),
        "na curva o auto-key fica quieto; gravou {:?}",
        plan.keys
    );
}
