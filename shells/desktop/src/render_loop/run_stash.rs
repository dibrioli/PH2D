//! **A PORTA ÚNICA dos verbos de FITA** (W25) — descartar uma corrida gravada,
//! e devolvê-la.
//!
//! # Por que uma porta, e não duas linhas repetidas
//!
//! A corrida gravada tem **duas vistas**: a §14 do Inspector (por-entidade, ao
//! lado do personagem que a gravou) e o painel de **MUNDO** (onde os fatos do
//! documento moram). Duas vistas são o idioma que este repo já usa em três
//! sítios — o `Show Colliders` do painel de física contra a tecla `B`, a lista
//! de ferramentas do Impasto contra os chips do rail, o chip de meio de pintura
//! contra o rádio do rail — e a lei que as torna seguras é sempre a mesma:
//! **elas leem o mesmo número publicado e caem na mesma função.**
//!
//! Duas cópias do `mem::take` compilariam e, hoje, fariam a mesma coisa. É
//! precisamente essa forma que apodrece: no dia em que o descarte ganhar um caso
//! especial (um toast, um segundo guardado, um limite), quem o escrever vai
//! encontrar uma das cópias.
//!
//! # ⚠️ Por que a fita NÃO é `ProjectState`
//!
//! Ela viaja no arquivo (W17) mas fica **fora** do estado que o undo global
//! captura, de propósito: um Ctrl+Z do canvas não deve rebobinar uma gravação.
//! É essa decisão que torna o descarte irreversível pelo caminho normal — e é
//! por isso que ele **guarda** em vez de apagar.

use ph2d_physics_ecs::InputTape;

/// O que fazer com a corrida.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum RunVerb {
    /// Tirar a corrida do documento e guardá-la na sessão.
    Discard,
    /// Devolver ao documento a corrida guardada.
    Restore,
}

/// **Aplicar o verbo** — a troca, nos dois sentidos.
///
/// ⚠️ **`mem::take` e não `clone` + `clear`**, e a diferença é o ciclo de vida:
/// a troca deixa exatamente UMA das duas fitas com conteúdo, então *"há corrida
/// viva"* e *"há corrida guardada"* nunca são verdade ao mesmo tempo. É isso que
/// faz a escolha de qual botão pintar ser **derivada** em vez de mantida — não
/// existe um terceiro estado a manter de acordo, e nenhum caminho ressuscita uma
/// corrida velha.
///
/// ⚠️ **O guardado nunca viaja no arquivo:** uma corrida descartada foi
/// descartada, e um arquivo que a carregasse ressuscitaria o que o artista
/// apagou. Ele é o desfazer de um CLIQUE, não um segundo documento.
pub(crate) fn apply(verb: RunVerb, live: &mut InputTape, stash: &mut InputTape) {
    match verb {
        RunVerb::Discard => *stash = std::mem::take(live),
        RunVerb::Restore => *live = std::mem::take(stash),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_physics_ecs::{PlayerInput, PlayerInputAtTick};

    fn tape(n: u64) -> InputTape {
        let mut t = InputTape::new();
        for k in 1..=n {
            t.record(
                k,
                PlayerInput {
                    drive: k as f32,
                    ..Default::default()
                },
            );
        }
        t
    }

    /// **A troca é uma troca**, e o ida-e-volta devolve a corrida INTEIRA — não
    /// o comprimento dela.
    ///
    /// ⚠️ O oráculo é o conteúdo tique a tique: uma porta que devolvesse uma
    /// fita do mesmo tamanho descreveria outra corrida, e o gate ficaria verde.
    #[test]
    fn the_verb_moves_the_run_and_never_copies_it() {
        let mut live = tape(5);
        let mut stash = InputTape::new();

        apply(RunVerb::Discard, &mut live, &mut stash);
        assert!(live.is_empty(), "descartar tem de esvaziar a fita viva");
        assert_eq!(stash.len(), 5, "e a corrida tem de ficar guardada");

        apply(RunVerb::Restore, &mut live, &mut stash);
        assert!(stash.is_empty(), "devolver tem de esvaziar o guardado");
        for k in 1..=5u64 {
            assert_eq!(
                live.input(k).map(|i| i.drive),
                Some(k as f32),
                "o tique {k} nao voltou como estava"
            );
        }
    }

    /// **Nunca há duas corridas ao mesmo tempo** — a propriedade de onde a
    /// escolha do botão é derivada, afirmada sobre a porta em vez de sobre o
    /// painel.
    ///
    /// **Mutação que deve sangrar:** trocar o `mem::take` por um `clone`.
    #[test]
    fn only_one_of_the_two_tapes_can_be_full() {
        let mut live = tape(3);
        let mut stash = InputTape::new();
        for verb in [
            RunVerb::Discard,
            RunVerb::Restore,
            RunVerb::Discard,
            RunVerb::Discard,
        ] {
            apply(verb, &mut live, &mut stash);
            assert!(
                live.is_empty() || stash.is_empty(),
                "as duas fitas ficaram cheias: viva {} guardado {}",
                live.len(),
                stash.len()
            );
        }
    }

    /// **Descartar duas vezes seguidas não inventa uma corrida** — o guardado
    /// fica vazio, e o botão de devolver não é oferecido.
    ///
    /// É o caso degenerado que a segunda vista torna alcançável: no painel de
    /// mundo o artista pode clicar sem ter um personagem selecionado.
    #[test]
    fn discarding_an_empty_run_leaves_nothing_to_restore() {
        let mut live = InputTape::new();
        let mut stash = InputTape::new();
        apply(RunVerb::Discard, &mut live, &mut stash);
        assert!(live.is_empty() && stash.is_empty());
    }
}
