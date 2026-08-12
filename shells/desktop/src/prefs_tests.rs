//! Gates do ficheiro de preferências.
//!
//! ⚠️ **O que estes gates NÃO cobrem, e porquê:** o IO real (`save`/`load`) escreve em
//! `$HOME/.ph2d/`, e um teste que o exercite mexe na casa de quem corre a suíte. O irmão
//! [`crate::palette_persist`] tem exactamente a mesma limitação e a mesma decisão. O que fica
//! coberto é o que de facto pode divergir: a **lei do formato** (ida e volta) e a **tolerância**
//! que substitui o número de versão. Quem cobre a fiação é o arch-gate do `forwarding.rs`.

use super::{Prefs, parse, serialize};
use ph2d_editor::motion::UiCharacter;

/// As QUATRO combinações, e é por isso que são quatro e não duas: o ficheiro tem de conseguir
/// dizer **Expressivo + reduced**, a combinação que um selector de três posições tornaria
/// inexprimível (a mesma lei que o `the_taste_and_the_guarantee_are_two_axes` pina no motor).
///
/// ⚠️ Este gate é também o que torna o `wire()`/`from_wire()` uma porta ÚNICA: se um lado
/// escrevesse `"Expressive"` e o outro lesse `"expressive"`, a ida-e-volta partia aqui.
#[test]
fn every_combination_of_the_two_axes_survives_a_round_trip() {
    for character in [UiCharacter::Discrete, UiCharacter::Expressive] {
        for reduced_motion in [false, true] {
            let p = Prefs {
                character,
                reduced_motion,
            };
            assert_eq!(
                parse(&serialize(&p)),
                p,
                "{character:?} + reduced={reduced_motion} tem de voltar igual"
            );
        }
    }
}

/// Ausente, vazio, lixo, valor inválido — as quatro degradam para o MESMO sítio: o default.
/// Uma preferência que se recusa a arrancar é pior que uma preferência perdida.
#[test]
fn a_broken_file_is_the_default_never_an_error() {
    for text in [
        "",
        "\n\n   \n",
        "isto não é um ficheiro de preferências",
        "motion_character=\nreduced_motion=",
        "motion_character=Expressive\n", // maiúscula: NÃO é o nome de fio
        "motion_character=orbital\nreduced_motion=talvez\n",
    ] {
        assert_eq!(
            parse(text),
            Prefs::default(),
            "entrada malformada tem de dar o default: {text:?}"
        );
    }
}

/// **A propriedade que substitui o número de schema.** Um build ANTIGO tem de ler o ficheiro que um
/// build NOVO escreveu: as chaves que conhece sobrevivem, as que não conhece são saltadas.
///
/// ⚠️ Num formato posicional (postcard, o `ProjectFile`) isto seria impossível e a versão seria
/// obrigatória. Aqui a versão custaria o oposto do que promete — o build antigo **recusaria** o
/// ficheiro inteiro em vez de ler a metade que entende.
#[test]
fn a_key_from_a_newer_build_is_skipped_and_the_rest_survives() {
    let from_the_future = "# PH2D prefs\n\
         motion_character=expressive\n\
         ui_sound_volume=0.4\n\
         tether_slack=1.2\n\
         reduced_motion=1\n";
    assert_eq!(
        parse(from_the_future),
        Prefs {
            character: UiCharacter::Expressive,
            reduced_motion: true,
        },
        "as duas chaves conhecidas sobrevivem intactas ao lado de duas que este build nunca viu"
    );
}

/// O default do ficheiro é o default do PRODUTO: um `prefs.txt` que ninguém escreveu tem de abrir
/// o app exactamente como ele abre hoje (Discreto, sem reduced) — a mesma neutralidade que tornou
/// a wave F0 segura de landar sozinha.
#[test]
fn the_absent_file_opens_the_app_the_way_it_opens_today() {
    let p = Prefs::default();
    assert_eq!(p.character, UiCharacter::Discrete);
    assert!(!p.reduced_motion);
}

/// **A lei da primeira observação**, executável em vez de comentada.
///
/// ⚠️ A mutação óbvia (`previous != Some(now)`) parece equivalente e não é: com ela, a primeira
/// observação de uma sessão que CARREGOU preferências não-default vê "mudou" e reescreve o ficheiro
/// sem ninguém ter tocado em nada. É a linha que este gate defende.
#[test]
fn the_first_observation_seeds_the_mirror_it_does_not_write() {
    use super::should_save;
    let default = Prefs::default();
    let loaded = Prefs {
        character: UiCharacter::Expressive,
        reduced_motion: true,
    };

    assert!(
        !should_save(None, default),
        "primeira observação, ficheiro ausente: nada a gravar"
    );
    assert!(
        !should_save(None, loaded),
        "primeira observação de preferências VINDAS DO DISCO: o ficheiro já as tem"
    );
    assert!(should_save(Some(default), loaded), "uma mudança real grava");
    assert!(
        !should_save(Some(loaded), loaded),
        "e um evento de ponteiro que não mudou nada não toca no disco"
    );
}
