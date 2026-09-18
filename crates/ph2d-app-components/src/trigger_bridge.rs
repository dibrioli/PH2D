//! ⭐⭐⭐ **A ponte do GATILHO: do teclado resolvido às amostras que a lei consome** (suplente #24).
//!
//! # ⚠️ Porque ela vive AQUI e não na shell
//!
//! Ela é **pura** — `(mapa, estado) → uma amostra por acção, pelo nome` — e não toca em nada que a
//! shell segure: nem o `gfx`, nem o barramento, nem o relógio. ⇒ o que fica na shell é só
//! **composição**: a cerca do relógio e a publicação no outbox. *O que sai são os CORPOS; o que
//! decide a ordem do quadro fica* (a lei que a W2 escreveu ao partir a `shells/desktop`).
//!
//! ⛔ E foi a catraca `the_shell_only_shrinks` que a mandou sair — `+77` linhas contra o tecto.
//! **A cura de uma catraca é MOVER para a crate da família, nunca subir o número**; aqui as duas
//! respostas coincidem, que é o sinal de que o corte é por responsabilidade e não por aritmética.
//!
//! # ⚠️⚠️ A lei que esta função dá de GRAÇA
//!
//! Ela varre o **MAPA** e não os gatilhos. Um nome que o mapa não conhece simplesmente **não está
//! aqui**, e o `unwrap_or_default` de quem a lê devolve silêncio. *Perguntar nome a nome ao mundo
//! poria a mesma decisão em dois sítios*, e o defeito mudo que isso abre é um `Release` a disparar
//! em **TODO quadro** sobre uma acção que ninguém ligou — porque `!pressed` é trivialmente verdade.
//!
//! ⚠️ **`BTreeMap` e não `HashMap`** — a espinha do determinismo desta casa (lint estrutural).

use std::collections::BTreeMap;

/// **As amostras de TODA acção do mapa, pelo nome.**
#[must_use]
pub fn amostras_das_accoes(
    map: &ph2d_input::InputMap,
    estado: &ph2d_input::ActionState,
) -> BTreeMap<String, ph2d_ecs::ActionSample> {
    let input = ph2d_input::Input::new(map, estado);
    map.actions()
        .iter()
        .map(|a| {
            (
                a.name.clone(),
                ph2d_ecs::ActionSample {
                    pressed: input.pressed(&a.name),
                    just_pressed: input.just_pressed(&a.name),
                    just_released: input.just_released(&a.name),
                },
            )
        })
        .collect()
}

/// ⭐⭐⭐ **O que o MAPA sabe de uma acção, pelo NOME** — a coluna que o painel não pode calcular.
///
/// ⚠️⚠️ **TRÊS estados, e o do meio é o que faltava:** uma acção **declarada e sem ligação
/// nenhuma** resolve para [`ph2d_input::Sample::default`] — o `ActionState::tick` percorre o MAPA e
/// não os dispositivos, e o doc dele escreve a lei por extenso (*«declarada e por atribuir não é
/// inexistente»*) ⇒ as três leituras dão `false`, e o gatilho fica **tão calado como com um nome
/// errado**. *Duas causas, o mesmo silêncio, e o painel dizia que estava tudo bem numa delas.*
///
/// ⛔ **E as CURAS são diferentes** (criar a acção · ligar-lhe uma tecla), que é o que obriga a
/// distinguir: um aviso só mandaria metade dos artistas ao sítio errado.
///
/// ⚠️ **Ela vive AQUI e não na shell** pela razão da irmã acima: é pura, e o que fica lá é a
/// composição. Um `match` inline na shell não teria gate — *um gate que chama a função em vez de
/// percorrer a rota afirma que a peça existe, nunca que quem a usa a usa*.
#[must_use]
pub fn no_mapa(
    map: &ph2d_input::InputMap,
    nome: &str,
) -> ph2d_editor_core::action_trigger_edits::NoMapa {
    use ph2d_editor_core::action_trigger_edits::NoMapa;
    map.id(nome)
        .and_then(|id| map.get(id))
        .map_or(NoMapa::Desconhecida, |a| {
            if a.bindings.is_empty() {
                NoMapa::SemTecla
            } else {
                NoMapa::Ligada
            }
        })
}

#[cfg(test)]
#[path = "trigger_bridge_tests.rs"]
mod tests;
