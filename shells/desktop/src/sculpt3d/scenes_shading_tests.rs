//! **A LUZ QUE A CENA DO AMBIENTE ABRE** — os gates do [`super::scene_rig`].
//!
//! ⚠️ **Eles afirmam o MECANISMO, não uma fração amostrada.** A propriedade que o
//! artista precisa é *"metade da sombra olha para o céu"*, e seria tentador
//! integrá-la sobre o hemisfério visível e comparar contra `0,5`. Isso mede o
//! certo por um caminho que **não pode falhar bem**: uma quadratura devolve um
//! número perto de 0,5 por muitas razões, e a barra teria de ser folgada o
//! bastante para não flakar.
//!
//! A razão pela qual a metade é EXATA é algébrica: com `dir.y == 0` o
//! terminador — o plano `n · L = 0` — **contém o eixo vertical**, então ele parte
//! a sombra em duas partes espelhadas pelo horizonte. Um gate que afirma
//! `dir.y == 0` afirma a metade e mais nada.
//!
//! A sonda que MEDE a fração vive na `ph2d-light`
//! (`measure_how_much_shadow_the_sky_reaches`) e é `#[ignore]`: ela existe para
//! ESCOLHER o rig, e escolheu.

use super::{env_ambient_rig, scene_rig};

/// A lâmpada da cena não tem componente VERTICAL — e é só isso que parte a
/// sombra ao meio pelo horizonte.
///
/// ⚠️ **O controle é o rig DEFAULT**, e ele é o defeito do 1º smoke desta wave:
/// a `Light::KEY` fica a 230°/30° e resolve para `dir.y = −0,663`, ou seja ela
/// vem de CIMA — do mesmo lado em que este ambiente põe o céu. O hemisfério
/// aceso VIRA o hemisfério do céu, e o que sobra para o termo pintar é quase só
/// o chão (medido: 11,5% da sombra visível olha para o céu, fator médio 0,817).
#[test]
fn the_environment_scene_opens_under_a_lamp_with_no_vertical_component() {
    let rig = env_ambient_rig();
    let resolved = ph2d_light::resolve(&rig).expect("a cena abre com a lâmpada acesa");

    // ⚠️ **UMA lâmpada, e a asserção é load-bearing:** a metade exata vem de a
    // ÚNICA direção acesa ser horizontal. Uma segunda lâmpada com componente
    // vertical re-inclinaria o terminador e a propriedade morreria em silêncio.
    assert_eq!(
        resolved.lamps().len(),
        1,
        "uma segunda lâmpada re-inclina o terminador e a sombra deixa de ser \
         partida ao meio pelo horizonte"
    );

    let dir = resolved.lamps()[0].dir;
    // Em azimute 0 o rotor de 1° devolve `[1, 0]` ao bit, então este zero é
    // EXATO — não há tolerância a discutir. (Em 180° ele deixa 3,7e-7 de
    // resíduo, que é por isso que a cena usa 0.)
    assert_eq!(
        dir[1], 0.0,
        "a lâmpada da cena tem de ser horizontal em y (dir = {dir:?})"
    );

    // **CONTROLE:** o rig de todo dia NÃO tem esta propriedade.
    let default = ph2d_light::resolve(&ph2d_light::LightRig::default()).expect("acesa");
    assert!(
        default.lamps()[0].dir[1].abs() > 0.5,
        "o rig default tem de vir de CIMA — se ele já fosse horizontal, este gate \
         não estaria afirmando nada sobre a cena"
    );
}

/// A porta responde **só** à cena do ambiente.
///
/// As irmãs (`=18`/`=19`/`=20`) julgam a MALHA — a fresta, a espessura, a
/// curvatura —, e a malha lê igual sob qualquer luz; dar-lhes um rig próprio
/// mudaria a imagem que o Enio já aprovou por nenhuma razão.
#[test]
fn only_the_environment_scene_brings_its_own_light() {
    // Sem `PH2D_SCULPT3D_SMOKE=24` a porta se cala e o app abre no rig do
    // artista. ⚠️ Este teste roda sem env var setada, que é o caso do artista.
    assert!(
        scene_rig().is_none(),
        "fora da cena do ambiente a luz é a do documento, não a de uma cena"
    );
}
