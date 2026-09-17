//! **O instantâneo da CUTSCENE e o dreno das edições dela** (TOP-20 #19, W3).
//!
//! Irmão do [`crate::hud_inspector`]: a shell publica o que o painel mostra, e o painel devolve
//! edições que voltam por aqui.
//!
//! # ⛔⛔ Esta crate NÃO vê a timeline, e a lei fica pura à custa disso
//!
//! `ph2d-app-components` não depende de `ph2d-timeline` (medido no plano 16 §1), e a mesma lei que
//! o [`ph2d_ecs::SequencePlayer`] escreve no cabeçalho dele vale aqui: **as cutscenes CHEGAM** como
//! uma lista de `(nome, duração)`, e quem a colhe do documento é a shell — a única que vê as duas
//! coisas. ⇒ este módulo testa-se sem uma timeline.
//!
//! # ⚠️ O relógio é o timer `0`
//!
//! A escolha está declarada no [`ph2d_ecs::sequence::em_corrida`], com as três razões. Aqui ela é
//! **lida**, nunca redecidida: uma segunda resposta a *«qual relógio é o da cutscene?»* poria o
//! painel a descrever um relógio e o motor a andar noutro.

use ph2d_ecs::{Entity, SequencePlayer, SimWorld, timer::TimerRuntime, timer::Timers};
use ph2d_editor_core::sequence_edits::{InspectorSequenceInfo, SequenceFieldEdit as E};

/// **Uma cutscene do documento, como o painel a mostra** — o nome e quanto ela dura.
///
/// ⚠️ **Um PAR e não duas listas paralelas.** Duas fatias indexadas em conjunto são o *vector
/// paralelo* que esta casa proíbe por escrito noutro sítio: elas divergem no dia em que alguém
/// filtrar uma e esquecer a outra, e a divergência é muda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cutscene<'a> {
    /// O nome do container, como o documento o guarda.
    pub nome: &'a str,
    /// Quanto ele dura, em segundos (a porta `container_length_seconds` da timeline).
    pub duracao: f64,
}

/// **Os dois factos do APP que decidem se a cutscene anda** — e que não vivem no objecto.
///
/// ⚠️ **Um par e não dois `bool` soltos:** dois argumentos do mesmo tipo lado a lado trocam de
/// sítio em silêncio, e o sintoma seria o painel a dar a razão errada para o facto certo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Relogios {
    /// O relógio da CENA está a tocar? Sem ele nenhum `Timer` conta.
    pub clock_playing: bool,
    /// A vista da timeline deixa correr? A porta é do lado da shell — ver o campo do instantâneo.
    pub vista_deixa_correr: bool,
}

/// **O que o painel mostra da cutscene deste objecto**, ou `None` se ele não tiver o componente.
#[must_use]
pub fn build_info(
    sim: &SimWorld,
    bits: u64,
    cutscenes: &[Cutscene<'_>],
    relogios: Relogios,
    selected_count: usize,
) -> Option<InspectorSequenceInfo> {
    let e = Entity::from_bits(bits);
    let seq = sim.world().get::<SequencePlayer>(e)?;

    let nomes: Vec<String> = cutscenes.iter().map(|c| c.nome.to_owned()).collect();
    // ⚠️ A resolução é a do MOTOR (`SequencePlayer::resolve`), nunca uma comparação escrita aqui:
    // ela apara os dois lados, e um segundo `==` cru mostraria «escolhida» ao lado de uma cutscene
    // que não corre.
    let escolhido = seq.resolve(cutscenes.iter().map(|c| c.nome));
    let duracao_da_cutscene = escolhido
        .and_then(|i| cutscenes.get(i))
        .map_or(0.0, |c| c.duracao);

    // O relógio `0` — o da sequência.
    let relogio = sim.world().get::<Timers>(e).and_then(|t| t.0.first());
    let estado = sim.world().get::<TimerRuntime>(e).and_then(|r| r.0.first());

    Some(InspectorSequenceInfo {
        entity_bits: bits,
        container: seq.container.clone(),
        nomes,
        escolhido,
        duracao_da_cutscene,
        tem_relogio: relogio.is_some(),
        a_correr: estado.is_some_and(|s| s.running),
        #[expect(
            clippy::cast_precision_loss,
            reason = "microssegundos em f64: um traço de cutscene cabe muito abaixo de 2^53"
        )]
        duracao_do_relogio: relogio.map_or(0.0, |t| t.duration_us as f64 / 1_000_000.0),
        #[expect(
            clippy::cast_precision_loss,
            reason = "idem — e é a MESMA conversão que o `em_corrida` faz, em f64 de propósito"
        )]
        t: estado.map_or(0.0, |s| s.elapsed_us as f64 / 1_000_000.0),
        clock_playing: relogios.clock_playing,
        vista_deixa_correr: relogios.vista_deixa_correr,
        selected_count,
    })
}

/// Uma edição. `true` = o documento mudou.
fn apply(sim: &mut SimWorld, bits: u64, edit: &E) -> bool {
    let e = Entity::from_bits(bits);
    let E::Container(nome) = edit;
    let Some(mut seq) = sim.world_mut().get_mut::<SequencePlayer>(e) else {
        return false;
    };
    // ⚠️ **Escrever o mesmo nome não é uma mudança** — o `get_mut` do bevy marca o componente como
    // alterado por ser PEDIDO, e um clique na cutscene que já estava escolhida entraria no undo.
    if seq.container == *nome {
        return false;
    }
    seq.container = nome.clone();
    true
}

/// Todas as edições de um quadro. `true` = alguma mudou.
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, E)]) -> bool {
    let mut mudou = false;
    for (bits, edit) in edits {
        mudou |= apply(sim, *bits, edit);
    }
    mudou
}

#[cfg(test)]
#[path = "sequence_inspector_tests.rs"]
mod tests;
