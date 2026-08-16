//! Os gates do ROTEADOR de cena do smoke da UI viva.
//!
//! ⚠️ Irmão de [`super`] e **FILHO por `#[path]`**, o precedente do `ui_spring_smoke_tests`: o
//! `use super::*` continua a alcançar [`super::smoke_level`], que é privado.
//!
//! ⚠️ **O que NÃO está aqui, e porquê:** as cenas montam estado dentro do `HeroScreen`, que exige
//! janela — `open_physics_for_smoke` e `collapsible_section_count` não são alcançáveis por um
//! teste de unidade, e o oráculo delas é o próprio roteiro (a contagem impressa, com o PARE).
//! Gatear o que sobra seria escrever um espelho do `eprintln!`.

use super::*;

fn level(raw: &str) -> u32 {
    smoke_level(std::ffi::OsStr::new(raw))
}

/// **Cada número alcança a sua própria cena.** O controlo positivo: sem ele, um roteador que
/// devolvesse `1` para tudo passaria no gate de baixo com folga.
#[test]
fn each_scene_number_reaches_its_own_scene() {
    assert_eq!(level("1"), 1);
    assert_eq!(level("2"), 2);
    assert_eq!(level("3"), 3);
    // Espaço à volta é o que um copy-paste de terminal deixa, e continua a nomear a cena.
    assert_eq!(level(" 3 "), 3);
}

/// **Uma cena que não existe cai na MAIS ANTIGA, nunca na mais nova.**
///
/// ⚠️ A lei já estava escrita para o valor que *não parseia* (`=sim`); esta metade é para o valor
/// que **parseia e nomeia uma cena futura**. Sem ela, um `=4` — o typo natural de quem espera a
/// cena seguinte — cairia no braço `_` do `match`, que é sempre a demo mais recente: o smoke
/// mostraria a coisa errada **e imprimiria o cabeçalho dela**, que é o que torna isto silencioso.
///
/// Mutação: apagar o `filter` do [`super::smoke_level`] faz `=4` devolver `4` ⇒ RED.
#[test]
fn a_scene_that_does_not_exist_falls_back_to_the_oldest() {
    assert_eq!(level("4"), 1, "uma cena futura nao pode virar a mais nova");
    assert_eq!(level("99"), 1);
    assert_eq!(level("0"), 1, "zero nao e' cena");
    assert_eq!(level("sim"), 1, "o caso que a lei ja' cobria");
    assert_eq!(level(""), 1);
    assert_eq!(level("-1"), 1);
}

/// **O teto do roteador é a cena mais nova que o `match` sabe pintar.**
///
/// ⚠️ Ele não é um número escolhido: se alguém acrescentar um braço ao `match` sem bumpar o
/// [`super::LAST_SCENE`], a cena nova fica **inalcançável em silêncio** — o `filter` recusa-a antes
/// de o `match` a ver, e o artista recebe a cena 1 com o cabeçalho da cena 1. Este gate não
/// consegue ler o `match`, então afirma o contrato mínimo: o teto é alcançável e o degrau seguinte
/// não é.
#[test]
fn the_ceiling_is_reachable_and_the_step_above_it_is_not() {
    assert_eq!(level(&LAST_SCENE.to_string()), LAST_SCENE);
    assert_eq!(level(&(LAST_SCENE + 1).to_string()), 1);
}
