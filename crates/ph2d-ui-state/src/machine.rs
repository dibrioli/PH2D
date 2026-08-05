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
use crate::transition::Transition;
use ph2d_anim::Easing;

/// Uma transição em voo.
struct Flight {
    tr: Transition,
    to: usize,
    elapsed: f64,
    duration: f64,
    easing: Easing,
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
        let Some(i) = self
            .index_of(role)
            .or_else(|| self.index_of(StateRole::Default))
        else {
            return;
        };
        self.go_to(i, duration, easing);
    }

    #[must_use]
    pub fn state_count(&self) -> usize {
        self.states.len()
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
    /// Alvo inválido, ou o alvo em que já se está e parado: **no-op**.
    pub fn go_to(&mut self, target: usize, duration: f64, easing: Easing) {
        if target >= self.states.len() {
            return;
        }
        if self.flight.is_none() && target == self.current {
            return;
        }
        let tr = Transition::new(&self.live, &self.states[target].objects);
        // Duração não-positiva (ou nada a mover) é uma troca INSTANTÂNEA — e ela também passa pela
        // chegada exata abaixo, em vez de por um caminho próprio que pudesse divergir dela.
        self.flight = Some(Flight {
            tr,
            to: target,
            elapsed: 0.0,
            duration: duration.max(0.0),
            easing,
        });
        if duration <= 0.0 {
            self.arrive(target);
        }
    }

    /// Anda `dt` segundos. Sem transição no ar é um no-op barato.
    pub fn advance(&mut self, dt: f64) {
        let Some(f) = self.flight.as_mut() else {
            return;
        };
        f.elapsed += dt.max(0.0);
        if f.elapsed >= f.duration {
            let to = f.to;
            self.arrive(to);
            return;
        }
        let u = f.elapsed / f.duration;
        // ⚠️ O EASING deforma o `t`, **nunca o dt**. Deformar o relógio faria a duração autorada
        // deixar de ser a duração real, e duas transições com o mesmo número acabariam em
        // instantes diferentes.
        let t = f.easing.eval(u);
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
