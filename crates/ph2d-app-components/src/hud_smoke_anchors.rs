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

/// **Um dos dois cantos de baixo da cena do HUD** — e ele responde, num sítio só, às DUAS
/// perguntas que o report de 2026-09-20 apanhou a discordar.
///
/// # ⛔⛔ O defeito: a REGRA dizia um canto e a POSIÇÃO autorada dizia o outro
///
/// A pontuação prendia-se à aresta **direita** e estava desenhada em `x = −7`; a contagem
/// prendia-se à **esquerda** e estava em `x = +8`. Com [`ph2d_hud::Fit::Keep`] a caixa efectiva
/// **é** a de referência, o delta sai `0,0` por subtracção de iguais e nada se move — logo *os
/// outros modos liam-se correctos*. Com [`ph2d_hud::Fit::Expand`] a caixa cresce, cada peça anda
/// para a borda a que está presa, e a certa altura elas **atravessam-se**.
///
/// Medido pelas portas do produto ([`crate::hud_bridge::anchor_frame_of`] +
/// [`VecAnchors::delta_local`]), com a autoria de então (ref `32 × 18`, meia-altura da vista `4,5`):
///
/// | aspecto da vista | pontuação | contagem | |
/// |---|---|---|---|
/// | `1,78` (`16:9`) | `−3,50` | `+4,00` | cada uma no lado ERRADO |
/// | `2,22` | `−1,50` | `+2,00` | a aproximarem-se |
/// | `2,67` | `+0,50` | `0,00` | **cruzam-se** |
/// | `4,00` | `+6,50` | `−6,00` | já do outro lado uma da outra |
///
/// # ⭐⭐ A cura não é mover dois literais: é o SINAL passar a sair da REGRA
///
/// [`Self::local`] deriva o lado de [`Self::fraccao`] ⇒ **uma peça autorada do lado oposto à
/// âncora dela deixa de ser exprimível**. *Duas respostas à mesma pergunta divergem no dia em que
/// alguém mexe numa delas* — e aqui divergiram no dia em que o `Expand` deu voz à segunda.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Canto {
    /// A aresta de baixo à ESQUERDA — onde vive a contagem da ronda.
    Esquerda,
    /// A aresta de baixo à DIREITA — onde vive a pontuação.
    Direita,
}

/// Quanto DENTRO da borda cada peça nasce, em unidades da caixa de referência.
///
/// # ⚠️ O recurso é o SUB-RECTÂNGULO entre os painéis, e não a caixa de referência
///
/// A caixa tem `±16` e os rótulos cabem à vontade (medido: `Pontos: 1230` tem meia-largura
/// `3,391`; `30.0 s` tem `1,640`). O que corta é o **editor**: a vista da câmera é a da JANELA e o
/// mundo é desenhado entre os painéis — o item que o handoff do #20 deixa ABERTO.
///
/// Medido na foto de 2026-09-20 (janela `1930×1012`, painéis nas larguras de fábrica, `escala
/// 0,556`): a borda direita da área de canvas cai em `x ≈ 5,84` de mundo, e com `DENTRO = 7` a
/// ponta do placar ia a `6,29` ⇒ **`Pontos: 16` desaparecia por baixo do Inspector**, e pior a cada
/// ponto que o relógio soma. Com `5` ela fica em `5,31`, e a contagem — que é estreita e vive do
/// outro lado, onde a Hierarquia é mais magra — sobra de longe.
///
/// ⛔ **É uma calibração, e ela diz-se:** as larguras dos painéis vivem em `~/.ph2d/layout.txt`,
/// FORA do repositório, logo nenhum número aqui é seguro para toda arrumação. A cura geral é a do
/// item aberto (a vista do HUD ser a BANDA e não a janela); esta é a que torna a cena legível.
const DENTRO: f32 = 5.0;

/// A altura das duas peças. ⚠️ **Em BAIXO, e é uma medição:** os avisos de sinal empilham-se no
/// TOPO do canvas e tapavam o placar metade do tempo (ver o cabeçalho da cena).
const BAIXO: f32 = -5.0;

impl Canto {
    /// A fracção da caixa que as duas pontas do filho seguem, em `x` — **`0` é a aresta mínima**
    /// (esquerda) e `1` a máxima, que é o idioma do [`VecAnchors`].
    #[must_use]
    pub const fn fraccao(self) -> f64 {
        match self {
            Self::Esquerda => 0.0,
            Self::Direita => 1.0,
        }
    }

    /// **Onde a peça deste canto é autorada**, na caixa de referência.
    ///
    /// ⭐ O SINAL sai da [`Self::fraccao`] e nunca de um literal — ver o doc do tipo. O que é
    /// autoria é só a distância à borda ([`DENTRO`]), que é a mesma dos dois lados.
    #[must_use]
    pub fn local(self) -> [f32; 2] {
        let sinal = if self.fraccao() > 0.5 { 1.0 } else { -1.0 };
        [sinal * DENTRO, BAIXO]
    }

    /// A regra de ancoragem deste canto — um **PINO** (`min == max`: o filho anda inteiro).
    ///
    /// ⚠️ `min != max` seria um esticão, e os dois saem da mesma lei — é o que o gate
    /// `um_filho_esticavel_cresce_com_a_banda` afirma.
    #[must_use]
    pub fn regra(self) -> VecAnchors {
        VecAnchors {
            min: [self.fraccao(), 0.0],
            max: [self.fraccao(), 0.0],
            base: BASE,
        }
    }
}

/// **As duas peças de baixo prendem-se aos cantos OPOSTOS** — o item que o handoff do #20 deixou
/// aberto (*«as âncoras não estão ligadas ao canvas»*).
///
/// Numa janela de `16:9` a banda do letterbox é zero, a caixa efectiva **É** a de referência e as
/// duas ficam onde estão — **ao bit**. Alargue a janela e elas seguem as bordas REAIS, em vez de
/// ficarem a uma banda delas.
///
/// ⭐ A contagem vai ao canto de baixo-**ESQUERDA** e a pontuação ao de baixo-**DIREITA**, que é o
/// arranjo que a própria doc do passe das âncoras nomeia como o caso de uso do HUD: *«a vida no
/// canto de cima, a pontuação colada no canto oposto»* — e é o [`Canto`] que impede a posição
/// autorada de dizer outra coisa.
pub fn prende_os_cantos(world: &mut World, contagem: Entity, pontos: Entity) {
    world.entity_mut(contagem).insert(Canto::Esquerda.regra());
    world.entity_mut(pontos).insert(Canto::Direita.regra());
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

#[cfg(test)]
#[path = "hud_smoke_anchors_tests.rs"]
mod tests;
