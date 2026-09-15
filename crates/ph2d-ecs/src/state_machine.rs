//! ⭐⭐⭐ **O `StateMachine`** — o item **#15** do TOP-20, e *«enquanto o script não tem UI, o único
//! cérebro autorável»*.
//!
//! Plano e medições: `docs/Components/12_plano_state_machine.md`.
//!
//! # O que ele fecha, medido antes da 1.ª linha (§5.0)
//!
//! A sonda [`mede_o_que_a_composicao_ja_da_ao_cerebro`] respondeu **NÃO** em três sítios
//! independentes: o mesmo sinal com duas linhas contraditórias dispara **as duas** (nada escolhe
//! uma — *e escolher uma é o que um estado é*); a `SignalAction` tem **5** campos e **zero** são
//! uma guarda; e dos **7** verbos com sink, **nenhum** emite um sinal.
//!
//! ⇒ **o que falta não são acções: é a MEMÓRIA de em que estado se está, e o sítio onde se escreve
//! «SE estou em X E acontecer Y, vá para Z».**
//!
//! # ⚠️ Onde ele vive, e porque NÃO é uma crate-folha
//!
//! O plano escrevia `ph2d-statemachine`. A implementação refutou-o: a família `Logic` inteira — o
//! [`crate::Timer`] (#2), o [`crate::Lifetime`] (#12) e o [`crate::SignalActions`] (#5) — tem a lei
//! **aqui**, ao lado do componente. As folhas puras desta casa (`ph2d-topdown`, `ph2d-projectile`)
//! existem porque são **matemática** partilhada com uma ponte de física; esta é uma tabela de
//! nomes. ⇒ uma crate nova seria o **primeiro** membro da família fora da casa dela, por zero ganho
//! medido.
//!
//! # As leis, e de onde cada uma veio
//!
//! - **A entrada é um SINAL** — o `ph2d-runtime` já tem seis produtores autorados. ⛔ **Sem
//!   expressões:** a autoria de expressões desta casa foi **retirada** por decisão (Timeline doc
//!   14), e reintroduzi-la por uma porta lateral seria reconstruir trabalho recusado.
//! - **A saída é um SINAL** — entrar e sair de um estado **anunciam-se**, e o `SignalActions` (#5)
//!   faz o resto. ⛔ Uma segunda lista de verbos seria uma segunda resposta a *«o que pode
//!   acontecer neste app»*.
//!   ⭐ É isto que **dissolve a recusa do #5 exactamente como ela prescreveu**: *«[um verbo que
//!   emite] seria a classe inteira dos laços … quando existir, ela entra com um ORÇAMENTO DE
//!   PROFUNDIDADE, não com um `if`»*.
//! - ⭐⭐⭐ **UM SINAL É CONSUMIDO pela transição que o usa**, e o orçamento de profundidade é
//!   **quantos sinais soaram neste tique** — um recurso real, e não um número escolhido.
//!
//!   ⚠️⚠️ **Esta é uma DIVERGÊNCIA DECLARADA do oráculo, e a implementação achou-a com um gate
//!   vermelho.** O que ele mede (Godot 4.7.2, MIT, corrido sem interface —
//!   `docs/Components/ferramentas/godot_statemachine_probe.gd`) é uma máquina cuja entrada é um
//!   **NÍVEL**: um `advance_condition` é um booleano que **fica** verdadeiro, logo re-satisfaz a
//!   transição seguinte, e é por isso que ele precisa de uma regra de paragem própria (medida: uma
//!   cadeia acíclica resolve-se **inteira num avanço** e um ciclo **não pendura**, andando um
//!   estado por avanço — ou seja, ele pára ao voltar a um estado já visitado).
//!
//!   A nossa entrada é um **EVENTO**: um sinal ACONTECE e é gasto por quem o ouve. ⇒ portar a
//!   regra dele à letra dava uma porta que, com um toque só, ia de `Aberta` a `Fechada` **e logo a
//!   `A abrir`** — medido por este gate, que nasceu vermelho sobre o meu próprio desenho. ⭐ Com o
//!   consumo, a lei observável do oráculo fica intacta (uma cadeia resolve-se num tique **quando os
//!   sinais lá estão**, um ciclo nunca pendura) e **nenhum `MAX_DEPTH` é preciso**: o laço termina
//!   porque a lista de sinais é finita.
//! - **A arbitragem é a ORDEM DA TABELA** — ganha a **primeira** transição satisfeita.
//!   ⚠️ **Divergência DECLARADA do oráculo:** ele tem um campo `priority` (medido: o número
//!   **menor** ganha) e desempata pela **última** declarada, porque um GRAFO não tem ordem visível.
//!   A nossa autoria é uma tabela ordenada que o artista vê e reordena, como a do `SignalActions`;
//!   um segundo eixo de precedência daria duas respostas à mesma pergunta.
//! - ⛔ **Uma transição para SI MESMO é recusada** — como no oráculo (medido: `has_transition`
//!   devolve `false`). Entrar onde já se está não é entrar.
//! - ⛔ **O que este avanço EMITE não realimenta este avanço.** Os sinais vão para o outbox e
//!   chegam no tique seguinte, como todo sinal desta casa. Realimentá-los aqui seria a classe dos
//!   laços **sem** o conjunto de visitados a protegê-la, porque o conjunto é sobre ESTADOS.
//!
//! [`mede_o_que_a_composicao_ja_da_ao_cerebro`]: ../../../crates/ph2d-ecs/tests/it/mede_o_que_a_composicao_ja_da_ao_cerebro.rs

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// Quantos estados uma máquina pode ter.
///
/// ⚠️ **O número sai do PAINEL**, como o [`crate::TIMERS_MAX`] e o [`crate::SIGNAL_ACTIONS_MAX`], e
/// pela mesma lei: *um modelo que aceita o que o painel não mostra produz estado inalcançável*.
pub const STATES_MAX: usize = 16;

/// Quantas transições uma máquina pode ter — o dobro dos estados, porque uma máquina útil tem mais
/// setas que caixas (a porta do smoke tem 3 estados e 4 setas).
pub const TRANSITIONS_MAX: usize = 32;

/// O maior nome de estado, em bytes — o mesmo tecto de um nome de timer, e pela mesma razão: ele
/// viaja no snapshot e é comparado por igualdade.
pub const STATE_NAME_MAX_BYTES: usize = 64;

/// **Um estado** — um nome, e o que ele anuncia ao entrar e ao sair.
///
/// ⚠️ **Entrar e sair são NOMES diferentes, não um campo de fase** — a lei dos toques da física e
/// das tags de animação. ⛔ Vazio = **calado**, o espelho exacto da lei do produtor de sinal.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineState {
    /// Como o artista lhe chama — *«Fechada»*, *«A abrir»*.
    pub name: String,
    /// O sinal publicado ao ENTRAR. Vazio = calado.
    pub on_enter: String,
    /// O sinal publicado ao SAIR. Vazio = calado.
    pub on_exit: String,
}

/// **Uma seta** — de que estado, ao ouvir que sinal, para que estado.
///
/// ⚠️ **Os estados são ÍNDICES e não nomes**, e é a mesma decisão que liga o [`crate::TimerRuntime`]
/// ao [`crate::Timers`]: um nome ligaria a seta ao texto que o artista pode reescrever a meio, e a
/// seta saltaria de estado. ⛔ O **sinal**, esse, é um nome — ele é o contrato com o resto do mundo.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition {
    /// O índice do estado de partida.
    pub from: u8,
    /// O nome do sinal que a dispara. **Vazio = nunca** (a lei do `SignalActions`).
    pub on: String,
    /// O índice do estado de chegada.
    pub to: u8,
}

/// **O cérebro de um objecto** — o componente registado, e **CONFIG**.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateMachine {
    /// Os estados, na ordem em que o artista os escreveu.
    pub states: Vec<MachineState>,
    /// As setas. ⚠️ **A ORDEM É A PRECEDÊNCIA** — ver o cabeçalho do módulo.
    pub transitions: Vec<Transition>,
    /// Em que estado a cena abre.
    pub initial: u8,
}

/// **Em que estado ele está AGORA** — ⛔ **NÃO registado, de propósito.**
///
/// ⭐⭐⭐ **A ausência de `Serialize`/`Deserialize` é LOAD-BEARING e a cerca é o TIPO**, exactamente
/// como no [`crate::TimerRuntime`]: o `register_default` exige `Serialize`, então registá-lo é
/// **erro de compilação** e não um gate por escrever. *Derivá-lo por conveniência faria cada
/// transição virar um passo de `Ctrl+Z`, em silêncio.*
///
/// ⚠️ E é por ele ser vivo que ele passa pela porta do [`crate::rewind_runtime`] — a wave W0 desta
/// mesma jornada, escrita porque *o que o undo não fotografa, o undo também não repõe*.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StateMachineRuntime {
    /// O índice do estado corrente.
    pub current: u8,
    /// A máquina já **entrou** alguma vez? ⚠️ Enquanto for `false`, o próximo avanço anuncia a
    /// entrada no estado inicial — *entrar no inicial é entrar*.
    pub started: bool,
}

/// **O que um cérebro ACABADO DE NASCER recebe** — a porta, com dois leitores (o
/// [`reconcile`] e o [`crate::rewind_runtime`]), pela mesma lei que o [`crate::timer::born`].
#[must_use]
pub fn born(m: &StateMachine) -> StateMachineRuntime {
    StateMachineRuntime {
        current: if (m.initial as usize) < m.states.len() {
            m.initial
        } else {
            0
        },
        started: false,
    }
}

/// **O que um avanço produziu.**
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Advanced {
    /// Os sinais a publicar, na ordem em que aconteceram (saídas e entradas intercaladas).
    pub emitted: Vec<String>,
    /// Quantas setas foram atravessadas — `0` é o caso comum e o que diz *«nada mudou»*.
    pub steps: u32,
}

/// **Põe o vivo em dia com a config** — o irmão do [`crate::timer::reconcile`].
///
/// ⚠️ Apagar estados pode deixar o `current` a apontar para fora; um índice inválido não é um caso
/// especial a lembrar em cada leitura, é um estado que **não pode existir**. ⇒ ele recua ao inicial.
/// Devolve `true` se mexeu.
pub fn reconcile(m: &StateMachine, rt: &mut StateMachineRuntime) -> bool {
    if (rt.current as usize) < m.states.len() {
        return false;
    }
    *rt = born(m);
    true
}

/// **Um avanço.** Ver o cabeçalho do módulo — em especial o orçamento, que é o conjunto de
/// visitados e não um número.
///
/// `fired` são os sinais que soaram **neste tique**; eles valem para a travessia inteira, que é o
/// que permite a uma cadeia resolver-se num tique só (medido no oráculo).
pub fn advance(m: &StateMachine, rt: &mut StateMachineRuntime, fired: &[&str]) -> Advanced {
    let mut out = Advanced::default();
    if m.states.is_empty() {
        return out;
    }
    reconcile(m, rt);

    // ⭐ **Entrar no inicial é ENTRAR** — e só uma vez.
    if !rt.started {
        rt.started = true;
        anuncia(&mut out.emitted, &m.states[rt.current as usize].on_enter);
    }
    if fired.is_empty() {
        return out;
    }

    // ⭐⭐⭐ O ORÇAMENTO: cada sinal que soou é GASTO por quem o ouve. Ver o cabeçalho — o laço
    // termina porque esta lista é finita, e não por um número escrito à mão.
    // ⚠️ Um `Vec<bool>` e não um conjunto de nomes: o MESMO sinal pode soar duas vezes num tique
    // (dois corpos tocaram a mesma armadilha), e aí ele move a máquina duas vezes — que é o que
    // «aconteceu duas vezes» quer dizer.
    let mut gasto = vec![false; fired.len()];
    loop {
        // **A primeira satisfeita ganha** — a ordem da tabela é a precedência (divergência
        // declarada do oráculo, ver o cabeçalho).
        let achado = m.transitions.iter().find_map(|t| {
            if t.from != rt.current
                || t.on.is_empty()
                || t.to == t.from
                || (t.to as usize) >= m.states.len()
            {
                return None;
            }
            // O primeiro sinal AINDA POR GASTAR com este nome.
            let (i, _) = fired
                .iter()
                .enumerate()
                .find(|(i, s)| !gasto[*i] && **s == t.on.as_str())?;
            Some((t.to, i))
        });
        let Some((destino, i)) = achado else {
            break;
        };
        gasto[i] = true;
        anuncia(&mut out.emitted, &m.states[rt.current as usize].on_exit);
        rt.current = destino;
        anuncia(&mut out.emitted, &m.states[rt.current as usize].on_enter);
        out.steps += 1;
    }
    out
}

/// Um nome vazio é **calado** — a lei do produtor, escrita uma vez.
fn anuncia(para: &mut Vec<String>, nome: &str) {
    if !nome.is_empty() {
        para.push(nome.to_string());
    }
}

#[cfg(test)]
#[path = "state_machine_tests.rs"]
mod tests;
