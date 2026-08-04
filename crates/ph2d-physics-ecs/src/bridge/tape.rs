//! **A FITA DE ENTRADA** (W7) — o dedo do jogador vira função do TICK.
//!
//! Porta IRMÃ da [`SceneAtTick`], com a mesma forma e no mesmo lugar do laço de
//! ticks devidos, e ela existe pela mesma razão que aquela existiu: até aqui, o
//! mundo era função de `(tick, repouso autorado, curvas)` — e o player quebrava
//! isso, porque a entrada dele chegava uma vez por **FRAME** e era guardada.
//!
//! # ⚠️ O que estava errado, medido
//!
//! O laço de replay do `rewind` dirige as poses da cena e **nunca chamou
//! `drive_players`**. Um scrub para trás replayava as plataformas e deixava o
//! personagem sem perna e sem caminhada: ele caía pelos ticks replayados, e
//! parava onde a gravidade o deixasse. A trajetória de um scrub e a de um play
//! discordavam sobre o mesmo tick.
//!
//! Com a fita, `drive_players` entra nos DOIS laços e o controlador volta a ser
//! reproduzível: o scrub replaya, e o ring de checkpoints continua servindo.
//!
//! # ⚠️ E o ESTADO DE PULO viaja com o checkpoint
//!
//! O `JumpState` é estado **cross-frame da ponte** — `airborne`, o corte, a
//! borda do botão. Um seed do ring devolve o mundo do tique T e deixaria esse
//! estado com o valor de AGORA: um personagem no meio de um pulo em T seria
//! tratado como estando no chão, a perna dispararia no ar, e a resposta para um
//! tick dependeria de o cache ter o âncora ou não — que é exatamente o modo de
//! falha que o ring existe para não ter.
//!
//! É o mesmo argumento que pôs o `pulley_payout` no checkpoint do rapier
//! (*"config não é capturada, mas a INTEGRAL de uma taxa ao longo do run é
//! estado simulado tanto quanto uma velocidade"*), um nível acima. Ele não cabe
//! naquele checkpoint — é chaveado por `Entity`, que é do ECS e não do solver —,
//! então a ponte guarda o dela em paralelo, **nos mesmos tiques âncora**.
//!
//! # ⚠️ Runtime-only, e nomeado
//!
//! A fita **não é serializada** — a classe do `TimelineFlags::performing`.
//! Persistí-la (um replay que sobrevive a fechar o app) é wave posterior, e está
//! escrita aqui para não virar promessa esquecida.

use std::collections::BTreeMap;

use bevy_ecs::entity::Entity;
use ph2d_ecs::SimWorld;
use ph2d_platformer::{JumpState, PlayerInput};

use crate::components::PlatformPlayer;

use super::PhysicsBridge;

/// **O que o dedo do jogador estava fazendo naquele tick.**
///
/// `None` significa *"não tenho nada a dizer sobre esse tick"*, e a ponte cai
/// na entrada SEGURADA — a irmã exata do `false` do [`SceneAtTick::put`].
///
/// ⚠️ **Uma entrada por TICK, não por player**, e é o modelo que o produto já
/// tem: o teclado é um dedo só e o `hand_input_to_players` da shell já a
/// distribui a todos. Uma fita por-entidade seria a resposta certa para dois
/// jogadores no mesmo teclado, e nada aqui a impede — ela nasce quando houver
/// um segundo dedo.
///
/// [`SceneAtTick::put`]: super::SceneAtTick::put
pub trait PlayerInputAtTick {
    /// A entrada daquele tick, ou `None` para *"use a segurada"*.
    fn input(&mut self, tick: u64) -> Option<PlayerInput>;
}

/// Nenhuma fita — a entrada é a que o chamador segurou (o comportamento de
/// antes desta wave, byte a byte).
///
/// A irmã do `FrozenScene`, e a resposta para todo gate headless e para o C9.
pub struct HeldInput;

impl PlayerInputAtTick for HeldInput {
    fn input(&mut self, _tick: u64) -> Option<PlayerInput> {
        None
    }
}

/// **Uma fita gravada**, do tick em que a gravação começou em diante.
///
/// ⚠️ **Um `Vec` indexado pelo tick, não um mapa**: a fita é densa por
/// construção (todo tick simulado tem uma entrada) e um `BTreeMap` custaria uma
/// busca por tick para responder o que um índice responde. O `first` existe
/// porque uma gravação que começa no tique 1000 não quer mil buracos.
#[derive(Clone, Debug, Default)]
pub struct InputTape {
    first: u64,
    frames: Vec<PlayerInput>,
}

impl InputTape {
    /// Uma fita vazia.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Grava o que o dedo fez NESTE tick.
    ///
    /// ⚠️ Regravar um tick já gravado **sobrescreve**, e é o certo: o artista
    /// que scrubba para trás e toca de novo está autorando por cima, e a
    /// alternativa (ignorar) faria a fita descrever uma corrida que ninguém deu.
    /// Um tick à FRENTE do fim preenche o vão com a última entrada — o dedo não
    /// mudou de posição enquanto ninguém olhava.
    pub fn record(&mut self, tick: u64, input: PlayerInput) {
        if self.frames.is_empty() {
            self.first = tick;
        }
        if tick < self.first {
            return;
        }
        let i = (tick - self.first) as usize;
        if i < self.frames.len() {
            self.frames[i] = input;
            return;
        }
        let fill = self.frames.last().copied().unwrap_or_default();
        while self.frames.len() < i {
            self.frames.push(fill);
        }
        self.frames.push(input);
    }

    /// Quantos ticks a fita cobre.
    #[must_use]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// A fita não tem nada.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Esquece tudo.
    pub fn clear(&mut self) {
        self.first = 0;
        self.frames.clear();
    }
}

impl PlayerInputAtTick for InputTape {
    fn input(&mut self, tick: u64) -> Option<PlayerInput> {
        if tick < self.first {
            return None;
        }
        self.frames.get((tick - self.first) as usize).copied()
    }
}

impl PhysicsBridge {
    /// Guarda o estado de pulo de todos os players no tique âncora.
    ///
    /// Chamado ao lado do `ring.record`, e sob a MESMA condição: um checkpoint
    /// e o estado que o acompanha descrevem o mesmo instante, e gravá-los em
    /// momentos diferentes é como o seed devolveria um mundo com a memória de
    /// outro tick.
    pub(super) fn record_jump_states(&mut self, tick: u64) {
        self.jump_ring.insert(tick, self.player_jump.clone());
        // A janela do ring é limitada; a nossa segue a dele pela borda de baixo
        // para não crescer sem teto num run longo.
        while self.jump_ring.len() > JUMP_RING_CAP {
            let Some(&oldest) = self.jump_ring.keys().next() else {
                break;
            };
            self.jump_ring.remove(&oldest);
        }
    }

    /// Devolve o estado de pulo do tique âncora, se ele foi gravado.
    ///
    /// ⚠️ `None` (o âncora não está na janela) deixa o estado como está, e é o
    /// certo: quem chama nesse caso é um `rebuild_from_rest`, que já o limpou.
    pub(super) fn seed_jump_states(&mut self, tick: u64) {
        if let Some(states) = self.jump_ring.get(&tick) {
            self.player_jump = states.clone();
        }
    }

    /// Esquece os estados guardados — irmão do `ring.clear()`.
    pub(super) fn clear_jump_ring(&mut self) {
        self.jump_ring.clear();
    }
}

/// Quantos tiques âncora de estado de pulo guardar.
///
/// ⚠️ Um teto em CONTAGEM e não em bytes, ao contrário do ring do rapier — e a
/// diferença é o tamanho da coisa guardada: um checkpoint do solver pesa ~1 kB
/// POR CORPO (daí o teto em bytes do ADR-0117), e um `JumpState` são **três
/// bools**. 256 âncoras de 100 players são 76 kB, ou seja abaixo do
/// arredondamento do orçamento do ring que ele acompanha.
const JUMP_RING_CAP: usize = 256;

/// O tipo da tabela — um mapa por tique âncora, cada um com um estado por player.
pub(super) type JumpRing = BTreeMap<u64, BTreeMap<Entity, JumpState>>;

impl PhysicsBridge {
    /// Pergunta à fita o que o dedo fez NESTE tick e instala a resposta.
    ///
    /// ⚠️ **Sem resposta, nada é tocado** — a entrada segurada continua valendo,
    /// que é o comportamento de antes desta wave e o de todo chamador sem fita.
    /// Escrever um `PlayerInput::default()` aqui faria uma cena sem fita PARAR
    /// de andar, que é o oposto de uma adição neutra.
    ///
    /// ⚠️ **Quem são os players é perguntado ao MUNDO, não à tabela de entrada**,
    /// e o gate `scrubbing_back_and_forward_reproduces_the_run` nasceu VERMELHO
    /// exatamente por eu ter feito o contrário: uma cena dirigida SÓ por fita
    /// nunca chamou `set_player_input`, então a tabela está vazia, a fita não
    /// tinha a quem entregar, e o personagem **assentava e não andava** — com a
    /// fita gravada, correta, e completamente inerte.
    pub(super) fn take_taped_input(
        &mut self,
        sim: &SimWorld,
        tape: &mut dyn PlayerInputAtTick,
        tick: u64,
    ) {
        let Some(input) = tape.input(tick) else {
            return;
        };
        let world = sim.world();
        // A MESMA pergunta que o `drive_players` faz, na MESMA ordem
        // determinística do `BTreeMap` de corpos.
        let players: Vec<Entity> = self
            .bodies
            .keys()
            .copied()
            .filter(|&e| world.get::<PlatformPlayer>(e).is_some())
            .collect();
        for e in players {
            self.player_input.insert(e, input);
        }
    }
}
