//! **A MÁQUINA DE ESTADOS** — quem está no ar, para onde vai, e onde a cena está agora.
//!
//! ⚠️ **Ela não tem relógio.** [`Machine::advance`] recebe o `dt` de quem a chama, e o `dt` vem do
//! `Playhead`. É a lição W4.T7 do Motion, onde o `MotionTransport` **morreu**: dois relógios
//! divergem, e o modo de falha é a UI a andar noutra velocidade que a cena.
//!
//! ⚠️ **E ela não sabe o que é um mouse.** *O que aconteceu* (entrou, saiu, apertou) é do shell;
//! *que pose a cena tem* é dela. Uma tabela de gatilhos aqui teria de inventar um modelo de
//! entrada, e o gatilho certo já existe do outro lado da fronteira — [`Machine::go_to`] é a porta.

use crate::pose::{ObjectPose, UiState};
use crate::role::StateRole;
use crate::spring::{Spring, SpringState};
use crate::transition::Transition;
use ph2d_anim::Easing;

/// **O que faz o `t` andar.** Ou o relógio deformado por uma curva, ou uma mola.
///
/// ⚠️ São exclusivos porque respondem a MESMA pergunta — *quanto do caminho já andou?* — e um
/// hospedeiro que carregasse os dois teria de decidir qual manda a cada quadro. É a razão de o
/// painel TROCAR as linhas em vez de as somar.
enum Drive {
    /// `elapsed / duration`, deformado pela curva. O caminho que já shipava.
    Curve { duration: f64, easing: Easing },
    /// Rigidez e amortecimento, integrados. **Sem duração**: ela acaba quando assenta.
    Spring { spring: Spring, state: SpringState },
}

/// Uma transição em voo.
struct Flight {
    tr: Transition,
    to: usize,
    elapsed: f64,
    drive: Drive,
    /// **Para onde o caminho aponta, e quanto ele mede** — o eixo unitário e o comprimento.
    ///
    /// ⚠️ **A velocidade da mola é um VETOR, e é por isso que o eixo não pode faltar.** Reescalar
    /// só pelo comprimento faz a reversão arrancar *para o lado errado*: o objeto ia para a
    /// frente, o caminho novo aponta para trás, e um escalar positivo herdado empurra-o para trás
    /// mais depressa — o oposto de momento. Projetar o vetor velho no eixo novo dá o sinal certo
    /// de graça, e num caminho perpendicular dá zero, que é a resposta honesta.
    axis: [f64; 2],
    span: f64,
}

/// Os estados de um objeto de UI, mais quem está no ar.
pub struct Machine {
    states: Vec<UiState>,
    current: usize,
    /// **A pose VISÍVEL agora** — não a autorada. É esta distinção que faz uma transição
    /// interrompida no meio continuar de onde está em vez de saltar.
    live: Vec<ObjectPose>,
    flight: Option<Flight>,
}

impl Machine {
    /// Uma máquina sobre `states`, parada no primeiro.
    ///
    /// `None` para uma lista vazia: uma máquina sem estado nenhum não tem pose para mostrar, e
    /// devolver uma vazia obrigaria todo chamador a tratar um caso que não pode acontecer.
    #[must_use]
    pub fn new(states: Vec<UiState>) -> Option<Self> {
        let live = states.first()?.objects.clone();
        Some(Self {
            states,
            current: 0,
            live,
            flight: None,
        })
    }

    /// O índice do estado em que ela **está** — ou de onde ela SAIU, se há uma transição no ar.
    #[must_use]
    pub fn current(&self) -> usize {
        self.current
    }

    /// O índice do estado para onde ela vai, se está indo a algum lugar.
    #[must_use]
    pub fn target(&self) -> Option<usize> {
        self.flight.as_ref().map(|f| f.to)
    }

    #[must_use]
    pub fn is_animating(&self) -> bool {
        self.flight.is_some()
    }

    /// O papel de um estado.
    #[must_use]
    pub fn role(&self, i: usize) -> Option<StateRole> {
        self.states.get(i).map(|s| s.role)
    }

    /// **O índice do papel `role`**, se este hospedeiro o autora.
    ///
    /// ⚠️ É a porta única do gatilho: quem sabe que o rato entrou é a shell, e quem sabe que
    /// estado isso É são os papéis — mas a MÁQUINA continua a andar por índice, e é essa
    /// separação que a mantém sem opinião sobre o que um mouse é.
    #[must_use]
    pub fn index_of(&self, role: StateRole) -> Option<usize> {
        self.states.iter().position(|s| s.role == role)
    }

    /// **Vai para o papel `role`, ou para o [`StateRole::Default`] se ele não existe.**
    ///
    /// ⚠️ O recuo para o Default é o que torna a lista de papéis **opcional**: um botão que só
    /// autora Hover continua a responder ao aperto — voltando ao repouso — em vez de ficar preso
    /// no hover porque ninguém gravou o Pressed. Sem ele, autorar um papel a mais seria um
    /// requisito escondido de autorar todos.
    pub fn go_to_role(&mut self, role: StateRole, duration: f64, easing: Easing) {
        let Some(i) = self.resolve(role) else { return };
        self.go_to(i, duration, easing);
    }

    /// O irmão de [`Self::go_to_role`] pela MOLA — mesmo recuo para o `Default`, outro motor.
    pub fn go_to_role_spring(&mut self, role: StateRole, spring: Spring) {
        let Some(i) = self.resolve(role) else { return };
        self.go_to_spring(i, spring);
    }

    /// **O papel pedido, ou o `Default`** — a regra do recuo, escrita UMA vez.
    ///
    /// ⚠️ Ela é uma função e não duas cópias porque o recuo é a razão de a lista de papéis ser
    /// opcional: se a mola e a curva discordassem sobre para onde recuar, um hospedeiro com mola
    /// ficaria preso onde o outro não fica.
    fn resolve(&self, role: StateRole) -> Option<usize> {
        self.index_of(role)
            .or_else(|| self.index_of(StateRole::Default))
    }

    #[must_use]
    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    /// **Troca a lista de estados, preservando a pose VIVA.**
    ///
    /// ⚠️ Ela existe porque o artista RE-GRAVA: a tabela do documento muda debaixo de uma máquina
    /// que já está no ar, e reconstruí-la do zero perderia onde a cena está — uma transição
    /// interrompida SALTARIA para a ponta, que é o defeito que o `go_to` foi desenhado para não
    /// ter.
    ///
    /// ⚠️ **Um voo em curso é ABORTADO**, e é a resposta certa: ele foi planeado contra poses que
    /// já não são as autoradas, então continuar seria animar para um destino que o documento não
    /// tem mais. A cena fica onde está e o próximo pedido parte dali.
    ///
    /// ⚠️ **Mas re-alinhar à MESMA tabela não é uma mudança**, e a igualdade é o que separa as
    /// duas coisas. A ponte chama isto a cada pedido, então um aborto incondicional destruía a
    /// transição em curso a cada clique em Show — antes sequer de examinar se alguma coisa tinha
    /// mudado. *Abortar é a resposta a uma tabela nova; a uma tabela igual é trabalho destruído
    /// por nada.*
    ///
    /// Lista vazia: a máquina fica como estava (uma máquina sem estados não tem pose para mostrar
    /// — quem a remove é o chamador, que sabe se o hospedeiro ainda existe).
    pub fn retarget(&mut self, states: Vec<UiState>) {
        if states.is_empty() || states == self.states {
            return;
        }
        self.states = states;
        self.current = self.current.min(self.states.len() - 1);
        self.flight = None;
    }

    /// **A pose que a cena tem AGORA.** É o que o shell escreve de volta no mundo.
    #[must_use]
    pub fn pose(&self) -> &[ObjectPose] {
        &self.live
    }

    /// **Vai para `target`.** O caminho começa na pose **VIVA**, nunca na autorada.
    ///
    /// ⚠️ É essa escolha que faz a interrupção parecer produto: sair do hover no meio da animação
    /// de entrada **volta de onde está**. Partir do estado autorado faria a cena SALTAR para a
    /// ponta antes de começar a voltar — o defeito que qualquer um vê e ninguém sabe nomear.
    ///
    /// ⚠️ **Duas chamadas antes de um `advance` não EMPILHAM:** a segunda substitui a primeira, e
    /// como nada avançou entre elas a pose viva é a mesma — o resultado é uma transição só, para o
    /// último alvo pedido. Uma fila faria a UI perseguir gestos que o artista já abandonou.
    ///
    /// ⚠️ **O atalho pergunta pela POSE, nunca pelo RÓTULO.** Pedir o estado que a cena já mostra
    /// continua a não animar — mas *"a cena mostra este estado"* é um fato sobre a pose VIVA, e
    /// `target == current` era só um proxy dele. O proxy **expira no instante em que um voo é
    /// abortado**: `current` continua a nomear o estado de onde se saiu enquanto a pose viva está
    /// a meio caminho do outro, e o Show daquele papel era **recusado** — a cena ficava parada e
    /// só voltava a andar quando o artista pedia o OUTRO papel (reportado, 2026-08-05). Pela pose,
    /// a pergunta não pode envelhecer.
    ///
    /// ⚠️ E o atalho **assume o papel** em vez de o descartar: sem isso, dois estados com a mesma
    /// pose deixariam o readout do painel a acender o nome de onde a máquina saiu.
    ///
    /// Alvo inválido: **no-op**.
    pub fn go_to(&mut self, target: usize, duration: f64, easing: Easing) {
        if target >= self.states.len() {
            return;
        }
        if self.flight.is_none() && self.live == self.states[target].objects {
            self.current = target;
            return;
        }
        let tr = Transition::new(&self.live, &self.states[target].objects);
        let (axis, span) = Self::path(&self.live, &self.states[target].objects);
        // Duração não-positiva (ou nada a mover) é uma troca INSTANTÂNEA — e ela também passa pela
        // chegada exata abaixo, em vez de por um caminho próprio que pudesse divergir dela.
        self.flight = Some(Flight {
            tr,
            to: target,
            elapsed: 0.0,
            drive: Drive::Curve {
                duration: duration.max(0.0),
                easing,
            },
            axis,
            span,
        });
        if duration <= 0.0 {
            self.arrive(target);
        }
    }

    /// **Vai para `target` por uma MOLA.** O irmão do [`Self::go_to`], e a única diferença é o
    /// que faz o `t` andar.
    ///
    /// ⚠️ **Uma reversão HERDA a velocidade.** É a única coisa que uma mola compra sobre uma
    /// curva (medido: `Cubic InOut` interrompido arranca a **0,00×** — a cena para e recomeça), e
    /// ela é reescalada para as unidades do caminho NOVO: `v` é fração-de-caminho por segundo, e
    /// os dois caminhos não medem o mesmo.
    ///
    /// ⚠️ **Sem voo em curso ela parte do repouso** — não há velocidade a carregar, e inventar uma
    /// faria um hover isolado arrancar como se algo o tivesse empurrado.
    pub fn go_to_spring(&mut self, target: usize, spring: Spring) {
        if target >= self.states.len() {
            return;
        }
        if self.flight.is_none() && self.live == self.states[target].objects {
            self.current = target;
            return;
        }
        // A velocidade que a cena TEM agora, como VETOR de mundo.
        //
        // ⚠️ Interromper uma CURVA não carrega nada: a `Easing` não expõe derivada, e estimá-la
        // por diferença finita seria um segundo modelo de velocidade ao lado do que a mola
        // integra. O caso comum (mola a trocar de alvo) carrega; o misto não.
        let world_v = match self.flight.as_ref() {
            Some(f) => match &f.drive {
                Drive::Spring { state, .. } => {
                    let m = state.v * f.span;
                    [f.axis[0] * m, f.axis[1] * m]
                }
                Drive::Curve { .. } => [0.0, 0.0],
            },
            None => [0.0, 0.0],
        };
        let tr = Transition::new(&self.live, &self.states[target].objects);
        let (axis, span) = Self::path(&self.live, &self.states[target].objects);
        // ⚠️ **A PROJEÇÃO é o que dá o sinal.** Numa reversão o eixo novo aponta ao contrário, e o
        // produto interno sai NEGATIVO — o objeto continua a andar para onde ia e só depois volta,
        // que é o que momento significa. Reusar a magnitude daria o oposto exato.
        let v = if span > f64::EPSILON {
            (world_v[0] * axis[0] + world_v[1] * axis[1]) / span
        } else {
            0.0
        };
        self.flight = Some(Flight {
            tr,
            to: target,
            elapsed: 0.0,
            drive: Drive::Spring {
                spring,
                state: SpringState::resuming(v),
            },
            axis,
            span,
        });
    }

    /// **Para onde um caminho aponta, e quanto ele mede** — pelo objeto que mais anda.
    ///
    /// ⚠️ O objeto que mais anda é o que domina a percepção do movimento; uma média diluiria o
    /// percurso num conjunto grande, e é a percepção que a continuidade de velocidade serve.
    /// Comprimento `0` quando nada se move — e aí não há velocidade a converter.
    fn path(from: &[ObjectPose], to: &[ObjectPose]) -> ([f64; 2], f64) {
        let mut best = ([0.0, 0.0], 0.0);
        for a in from {
            let Some(b) = to.iter().find(|p| p.id == a.id) else {
                continue;
            };
            let d = [
                b.translation[0] - a.translation[0],
                b.translation[1] - a.translation[1],
            ];
            let len = d[0].hypot(d[1]);
            if len > best.1 {
                best = ([d[0] / len, d[1] / len], len);
            }
        }
        best
    }

    /// Anda `dt` segundos. Sem transição no ar é um no-op barato.
    pub fn advance(&mut self, dt: f64) {
        let Some(f) = self.flight.as_mut() else {
            return;
        };
        f.elapsed += dt.max(0.0);
        let t = match &mut f.drive {
            Drive::Curve { duration, easing } => {
                if f.elapsed >= *duration {
                    let to = f.to;
                    self.arrive(to);
                    return;
                }
                // ⚠️ O EASING deforma o `t`, **nunca o dt**. Deformar o relógio faria a duração
                // autorada deixar de ser a duração real, e duas transições com o mesmo número
                // acabariam em instantes diferentes.
                easing.eval(f.elapsed / *duration)
            }
            Drive::Spring { spring, state } => {
                // ⚠️ Uma mola **não termina sozinha** — ela converge assintoticamente. É o
                // critério de assentamento que faz a máquina chamar o `arrive`, e é o `arrive`
                // que põe a pose EXATA: sem ele a cena derivaria um resíduo a cada hover.
                if state.advance(dt.max(0.0), *spring) {
                    let to = f.to;
                    self.arrive(to);
                    return;
                }
                state.x
            }
        };
        let moved = f.tr.at(t);
        Self::overlay(&mut self.live, moved);
    }

    /// **A chegada é EXATA.** A pose vira o estado autorado, ao bit — nunca o resultado de um
    /// `t = 1` numérico.
    ///
    /// ⚠️ Sem isto a cena **deriva**: cada ida-e-volta deixa um resíduo de ponto flutuante, e
    /// depois de algumas dezenas de hovers o botão já não está onde o artista o desenhou. E é
    /// também isto que **remove** quem estava a sair — no fim, quem saiu não está lá.
    fn arrive(&mut self, target: usize) {
        self.current = target;
        self.live = self.states[target].objects.clone();
        self.flight = None;
    }

    /// Escreve as poses em movimento por cima das vivas, casando **por id**; quem entra é
    /// acrescentado.
    fn overlay(live: &mut Vec<ObjectPose>, moved: Vec<ObjectPose>) {
        for m in moved {
            match live.iter_mut().find(|p| p.id == m.id) {
                Some(slot) => *slot = m,
                None => live.push(m),
            }
        }
    }
}
