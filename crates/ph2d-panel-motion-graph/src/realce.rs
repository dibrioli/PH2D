//! **O REALCE DE UMA LARGADA** — a decisão de *o que acende, e com que força*, numa função pura
//! com gate próprio. Ordem do dono (2026-09-19): *«nós e linhas podem ganhar um destaque de cor
//! […] de que estão sobrepostos prestes a trocar ou encaixar. Após a troca o conjunto linha e nó
//! piscam e se acentam.»*
//!
//! ⛔⛔ **Ela é uma FUNÇÃO e não um `if` dentro do pintor, e a razão é medida:** a primeira
//! redacção pôs a decisão nos dois pintores e gateou-a por um censo TEXTUAL (*«o ficheiro menciona
//! `largada_viva`»*) — e a prova de mutação **SOBREVIVEU** a desligar a pintura com um `if false
//! &&`, porque o nome continua lá. *Um censo de texto afirma que alguém escreveu a palavra, nunca
//! que o desenho acontece.* Com a decisão aqui, a mutação mata um gate de VALOR; o censo textual
//! fica ao lado, a afirmar a outra metade (que o pintor ainda a CHAMA).
//!
//! ⚠️ **As duas metades do gesto partilham o mesmo mecanismo de propósito** — a mesma cor, a mesma
//! largura —, e só a FORÇA muda: `1` enquanto a mão paira sobre o alvo, e a descer depois de
//! largar. *É isso que liga, para o olho, a promessa e o que aconteceu.*

use crate::interact::node_drop::Largada;
use crate::state::MotionGraphPanelState;

/// A força do realce de uma CARTA (`None` = nada a acender).
pub(crate) fn realce_do_cartao(state: &MotionGraphPanelState, id: u32) -> Option<f32> {
    if state.largada_viva == Some(Largada::Troca(id)) {
        return Some(1.0);
    }
    let p = state.piscada_viva.as_ref()?;
    p.nos.contains(&id).then(|| p.t.clamp(0.0, 1.0))
}

/// A força do realce de um FIO, nomeado pela ponta de chegada (`None` = nada a acender).
pub(crate) fn realce_do_fio(state: &MotionGraphPanelState, to: (u32, u16)) -> Option<f32> {
    if state.largada_viva == Some(Largada::Fio(to.0, to.1)) {
        return Some(1.0);
    }
    let p = state.piscada_viva.as_ref()?;
    p.fios.contains(&to).then(|| p.t.clamp(0.0, 1.0))
}

#[cfg(test)]
#[path = "realce_tests.rs"]
mod tests;
