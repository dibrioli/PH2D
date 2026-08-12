//! **A PORTA QUE ARMA O VERBO** — que ela arma os QUATRO campos, e que nenhum
//! deles apaga uma escolha do artista.
//!
//! ⚠️ **Este arquivo existe porque a suíte ficou VERDE quando a porta nasceu.**
//! O seam anterior cobria força e accumulate; a curva e o raio entraram e
//! **nada** teria acusado se eles não fossem armados — que é exatamente como o
//! D1 do `docs/3D/20_divergencias_tools.md` atravessou o app inteiro sem ninguém
//! ver: o seletor de verbo era a porta, e ela estava incompleta.

use ph2d_panel_sculpt3d::state::{BASE_RADIUS_PX, Sculpt3dUi, arm_verb_defaults};
use ph2d_sculpt3d::Verb;

/// **Um pincel de fábrica trocado de verbo veste a referência daquele verbo** —
/// os quatro campos, num gate só, porque eles são UMA decisão.
///
/// Os números são os do `Crease.js` do SculptGL: força `0,75`, raio `25` (a
/// METADE do resto — um vinco é fino por definição), accumulate desarmado, e a
/// quártica que toda tool de geometria dele compartilha.
#[test]
fn switching_to_a_verb_wears_the_reference_of_that_verb() {
    let mut ui = Sculpt3dUi::default();
    assert_eq!(ui.brush.verb, Verb::Draw, "a fixture parte do de fábrica");

    arm_verb_defaults(&mut ui, Verb::Crease);

    assert_eq!(ui.brush.verb, Verb::Crease);
    assert!(
        (ui.brush.strength - 0.75).abs() < 1e-6,
        "força: Crease.js:10 diz 0,75, veio {}",
        ui.brush.strength
    );
    assert!(
        (ui.radius_px - BASE_RADIUS_PX * 0.5).abs() < 1e-6,
        "raio: Crease.js:9 diz 25 contra a base 50 — METADE; veio {}",
        ui.radius_px
    );
    assert!(!ui.brush.accumulate, "Crease.js não declara accumulate");
    assert_eq!(
        ui.brush.falloff,
        Verb::Crease.default_falloff(),
        "a CURVA é a do verbo, e era ela que este seam não armava"
    );
}

/// ⚠️ **A CURVA só é observável no verbo que a referência NÃO TEM** — e este
/// gate existe porque a mutação *"apague o arming do falloff"* **SOBREVIVEU** ao
/// gate acima.
///
/// O motivo é aritmético e vale a pena estar escrito: **todas as tools de
/// geometria do SculptGL compartilham a mesma quártica**, então trocar entre
/// duas delas arma um valor idêntico ao que já estava, e *armar* e *não armar*
/// dão o mesmo pixel. O `Sharpen` é o único cujo perfil é `None` (o original não
/// o tem) ⇒ ele cai no nosso `Smooth`, e a transição fica visível nos dois
/// sentidos.
///
/// ⚠️ **E é isto que a W1 torna load-bearing em todo verbo:** no dia em que o
/// perfil `B` declarar a curva editável do Blender, cada troca de ferramenta
/// passa a mover a curva — e é este arming que a move.
#[test]
fn the_curve_is_armed_and_the_verb_without_a_reference_is_what_proves_it() {
    let mut ui = Sculpt3dUi::default();
    let quartic = Verb::Draw.default_falloff();
    let ours = Verb::Sharpen.default_falloff();
    assert_ne!(
        quartic, ours,
        "o fixture só distingue se os dois verbos declararem curvas DIFERENTES"
    );

    arm_verb_defaults(&mut ui, Verb::Sharpen);
    assert_eq!(ui.brush.falloff, ours, "o Sharpen não tem perfil `S`");

    arm_verb_defaults(&mut ui, Verb::Draw);
    assert_eq!(
        ui.brush.falloff, quartic,
        "e a volta re-arma a da referência"
    );
}

/// **O raio do Move é o TRIPLO**, e é o outro lado do mesmo achado: um puxão é
/// um gesto de REGIÃO, e um Move com raio de pincel de detalhe é o que faz um
/// artista concluir que a ferramenta não funciona.
#[test]
fn the_grab_is_born_three_times_wider() {
    let mut ui = Sculpt3dUi::default();
    arm_verb_defaults(&mut ui, Verb::Move);
    assert!(
        (ui.radius_px - BASE_RADIUS_PX * 3.0).abs() < 1e-6,
        "Move.js:10 diz 150 contra a base 50; veio {}",
        ui.radius_px
    );
}

/// ⚠️ **NENHUM verbo apaga uma escolha deliberada** — a lei que existe desde o
/// `arm_inflate_defaults` do Painter, agora afirmada sobre os QUATRO campos.
///
/// A fixture mexe em tudo com valores que **não são o default de verbo nenhum**,
/// e depois troca de ferramenta: os quatro têm de sobreviver.
#[test]
fn nothing_the_artist_touched_is_overwritten() {
    let mut ui = Sculpt3dUi::default();
    ui.brush.strength = 0.37;
    ui.radius_px = 123.0;
    ui.brush.falloff = ph2d_sculpt3d::Falloff::Root;
    // O accumulate é booleano, então *"mexido"* é ele estar no OPOSTO do que o
    // verbo que sai declara — não há terceiro valor a escolher.
    ui.brush.accumulate = !Verb::Draw.default_accumulate();

    arm_verb_defaults(&mut ui, Verb::Smooth);

    assert!((ui.brush.strength - 0.37).abs() < 1e-6, "força autorada");
    assert!((ui.radius_px - 123.0).abs() < 1e-6, "raio autorado");
    assert_eq!(
        ui.brush.falloff,
        ph2d_sculpt3d::Falloff::Root,
        "curva autorada"
    );
    assert_eq!(
        ui.brush.accumulate,
        !Verb::Draw.default_accumulate(),
        "accumulate autorado"
    );
    assert_eq!(
        ui.brush.verb,
        Verb::Smooth,
        "e o verbo TROCA de qualquer forma"
    );
}

/// **A ordem é load-bearing:** os quatro comparam contra o verbo que SAI, então
/// trocar `verb` antes deles faria toda comparação ser contra o verbo que
/// ENTRA — e um pincel de fábrica pareceria mexido, ficando preso para sempre.
///
/// ⚠️ Duas trocas seguidas é o que torna isto observável: com uma só, sair e
/// entrar coincidem em qualquer campo que os dois verbos declarem igual.
#[test]
fn two_switches_in_a_row_still_wear_the_reference() {
    let mut ui = Sculpt3dUi::default();
    arm_verb_defaults(&mut ui, Verb::Inflate);
    assert!(
        (ui.brush.strength - 0.3).abs() < 1e-6,
        "Inflate.js:10 diz 0,3 — o mais fraco do catálogo"
    );
    arm_verb_defaults(&mut ui, Verb::Mask);
    assert!(
        (ui.brush.strength - 1.0).abs() < 1e-6,
        "Masking.js:15 diz força CHEIA; veio {} (a comparação foi contra o \
         verbo que SAI?)",
        ui.brush.strength
    );
}
