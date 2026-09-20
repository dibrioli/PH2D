//! ⭐⭐⭐ **As ÂNCORAS da cena do HUD** (TOP-20 #20) — a regra de *o que acontece quando a
//! janela muda de tamanho*.
//!
//! ⚠️⚠️ **Ela mora na CRATE DA FAMÍLIA e não na shell**, como as cenas irmãs desta linha: a
//! composição de uma cena é da família, e a shell só decide QUANDO a chamar. ⛔ Ela nasceu na shell
//! por inércia e a catraca `the_shell_only_shrinks` mandou-a para casa no mesmo dia.

use bevy_ecs::world::World;
use ph2d_ecs::{Entity, VecAnchors};

/// A caixa de referência da cena do HUD — `32 × 18`, o `16:9` de fábrica desta casa.
///
/// ⚠️ **A cena lê-a DAQUI**, e é isso que impede a régua da âncora e a caixa do canvas de
/// divergirem: duas constantes com o mesmo número são duas respostas à mesma pergunta.
pub const REF_W: f32 = 32.0;
/// Ver [`REF_W`].
pub const REF_H: f32 = 18.0;

/// A caixa de REFERÊNCIA do canvas, centrada na origem — a régua contra a qual as duas regras são
/// armadas.
///
/// ⚠️⚠️ **É a de referência e NÃO a efectiva**, e a diferença é a wave inteira: a efectiva é um
/// acidente da janela que estava aberta, e gravá-la faria o mesmo projecto comportar-se de maneira
/// diferente em cada máquina. O que muda por quadro é a efectiva; o que fica no documento é esta.
const BASE: [f64; 4] = [
    -REF_W as f64 / 2.0,
    -REF_H as f64 / 2.0,
    REF_W as f64 / 2.0,
    REF_H as f64 / 2.0,
];

/// **As duas peças de baixo prendem-se aos cantos OPOSTOS** — o item que o handoff do #20 deixou
/// aberto (*«as âncoras não estão ligadas ao canvas»*).
///
/// Numa janela de `16:9` a banda do letterbox é zero, a caixa efectiva **É** a de referência e as
/// duas ficam onde estão — **ao bit**. Alargue a janela e elas seguem as bordas REAIS, em vez de
/// ficarem a uma banda delas.
///
/// ⚠️ `min == max` é um **PINO** (o filho anda inteiro); `min != max` seria um esticão — e os dois
/// saem da mesma lei, que é o que o gate `um_filho_esticavel_cresce_com_a_banda` afirma.
///
/// ⭐ A contagem vai ao canto de baixo-**ESQUERDA** e a pontuação ao de baixo-**DIREITA**, que é o
/// arranjo que a própria doc do passe das âncoras nomeia como o caso de uso do HUD: *«a vida no
/// canto de cima, a pontuação colada no canto oposto»*.
pub fn prende_os_cantos(world: &mut World, contagem: Entity, pontos: Entity) {
    world.entity_mut(contagem).insert(VecAnchors {
        min: [0.0, 0.0],
        max: [0.0, 0.0],
        base: BASE,
    });
    world.entity_mut(pontos).insert(VecAnchors {
        min: [1.0, 0.0],
        max: [1.0, 0.0],
        base: BASE,
    });
}

/// **Uma linha da tabela do TOP-20 #5**: este sinal soma `quanto` ao contador chamado `placar`.
///
/// ⚠️ O nome do placar **entra por argumento** e não sai de uma const daqui: ele é um CONTRATO
/// entre quatro sítios da cena (o relógio, a tabela, o contador e o botão), e uma segunda cópia
/// dele nesta crate divergiria da primeira na primeira edição.
#[must_use]
pub fn accao(sinal: &str, placar: &str, quanto: &str) -> ph2d_ecs::SignalAction {
    ph2d_ecs::SignalAction {
        on: sinal.to_owned(),
        target: placar.to_owned(),
        verb: ph2d_ecs::SignalVerb::AddToCounter,
        arg: quanto.to_owned(),
        target_by: ph2d_ecs::SignalTarget::Named,
        from: ph2d_ecs::SignalFrom::Anyone,
    }
}

/// **Um relógio que publica `sinal` ao fechar**, e que nasce a correr.
#[must_use]
pub fn relogio(sinal: &str, us: u64, repeat: bool) -> ph2d_ecs::Timer {
    ph2d_ecs::Timer {
        name: sinal.to_owned(),
        duration_us: us,
        repeat,
        autostart: true,
        signal: sinal.to_owned(),
    }
}
