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
//! # ⚠️ E o ESTADO DO CONTROLADOR viaja com o checkpoint
//!
//! O `PlayerState` é estado **cross-frame da ponte** — `airborne`, o corte, a
//! borda dos botões, e o arranque (W14). Um seed do ring devolve o mundo do tique T e deixaria esse
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
//! # A fita VIAJA no arquivo (W17)
//!
//! Até a W16 ela era runtime-only e esta nota prometia a wave. Ela chegou: a
//! fita tem uma [`TapeWire`] e o `ProjectFile` a carrega, então uma corrida
//! gravada sobrevive a fechar o app — e, com o bake da W16 a lendo, reabrir um
//! projeto e apertar Bake devolve a corrida de ontem.
//!
//! ⚠️ **A wave que persistiu teve de CORRIGIR o que era gravado primeiro.**
//! Medido pela porta do produto (`measure_player_tape`), em 120 frames a fita
//! gravava **120 tiques nas QUATRO células** — sem player na cena, e com a
//! simulação desarmada. Ela gravava o *relógio andando*, não uma corrida; a
//! condição que a torna uma corrida vive no chamador
//! (`render_loop::physics_bridge::dispatch`) e está escrita lá.

use std::collections::BTreeMap;

use bevy_ecs::entity::Entity;
use ph2d_ecs::SimWorld;
use ph2d_platformer::{PlayerInput, PlayerState};

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

/// **A forma de ARQUIVO de uma fita** (W17).
///
/// ⚠️ **Ela existe para o `PlayerInput` NÃO aprender serde.** A
/// `ph2d-platformer` é a crate da lei pura — sem rapier, sem ECS, sem formato de
/// arquivo —, e ensinar-lhe a serializar seria a primeira aresta na direção
/// errada. A tradução mora aqui, na crate-ponte, que já é o lugar onde o
/// `RigidBody` e o `Collider` viram bytes.
///
/// ⚠️ **Os botões viajam num BITMASK, e não como quatro `bool`s**, por uma razão
/// que esta linha já pagou duas vezes: o `PlayerInput` ganhou o `down` na W12 e o
/// `dash` na W14. Num `u8` o quinto botão é um BIT novo no mesmo byte — o layout
/// do arquivo não se move, um leitor velho ignora o bit e um leitor novo lê
/// `false` num arquivo velho. Com quatro `bool`s, o quinto seria um byte novo por
/// tique, ou seja um bump de schema por botão.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TapeWire {
    /// O tique da primeira entrada.
    pub first: u64,
    /// Um par por tique: o eixo de caminhada, e os botões no bitmask.
    pub frames: Vec<(f32, u8)>,
}

/// O bit do botão de PULO no bitmask do [`TapeWire`].
const BIT_JUMP: u8 = 1 << 0;
/// O bit do botão de BAIXO.
const BIT_DOWN: u8 = 1 << 1;
/// O bit do botão de ARRANQUE.
const BIT_DASH: u8 = 1 << 2;

impl InputTape {
    /// A fita na forma que vai para o arquivo.
    #[must_use]
    pub fn to_wire(&self) -> TapeWire {
        TapeWire {
            first: self.first,
            frames: self
                .frames
                .iter()
                .map(|i| {
                    let mut bits = 0u8;
                    if i.jump {
                        bits |= BIT_JUMP;
                    }
                    if i.down {
                        bits |= BIT_DOWN;
                    }
                    if i.dash {
                        bits |= BIT_DASH;
                    }
                    (i.drive, bits)
                })
                .collect(),
        }
    }

    /// A fita que um arquivo descreve.
    #[must_use]
    pub fn from_wire(w: &TapeWire) -> Self {
        Self {
            first: w.first,
            frames: w
                .frames
                .iter()
                .map(|&(drive, bits)| PlayerInput {
                    drive,
                    jump: bits & BIT_JUMP != 0,
                    down: bits & BIT_DOWN != 0,
                    dash: bits & BIT_DASH != 0,
                })
                .collect(),
        }
    }
}

impl PhysicsBridge {
    /// Guarda o estado de pulo de todos os players no tique âncora.
    ///
    /// Chamado ao lado do `ring.record`, e sob a MESMA condição: um checkpoint
    /// e o estado que o acompanha descrevem o mesmo instante, e gravá-los em
    /// momentos diferentes é como o seed devolveria um mundo com a memória de
    /// outro tick.
    pub(super) fn record_player_states(&mut self, tick: u64) {
        self.state_ring.insert(tick, self.player_state.clone());
        // A janela do ring é limitada; a nossa segue a dele pela borda de baixo
        // para não crescer sem teto num run longo.
        while self.state_ring.len() > STATE_RING_CAP {
            let Some(&oldest) = self.state_ring.keys().next() else {
                break;
            };
            self.state_ring.remove(&oldest);
        }
    }

    /// Devolve o estado de pulo do tique âncora, se ele foi gravado.
    ///
    /// ⚠️ `None` (o âncora não está na janela) deixa o estado como está, e é o
    /// certo: quem chama nesse caso é um `rebuild_from_rest`, que já o limpou.
    pub(super) fn seed_player_states(&mut self, tick: u64) {
        if let Some(states) = self.state_ring.get(&tick) {
            self.player_state = states.clone();
        }
    }

    /// Esquece os estados guardados — irmão do `ring.clear()`.
    pub(super) fn clear_state_ring(&mut self) {
        self.state_ring.clear();
    }
}

/// Quantos tiques âncora de estado de pulo guardar.
///
/// ⚠️ Um teto em CONTAGEM e não em bytes, ao contrário do ring do rapier — e a
/// diferença é o tamanho da coisa guardada: um checkpoint do solver pesa ~1 kB
/// POR CORPO (daí o teto em bytes do ADR-0117), e um `PlayerState` são umas
/// poucas dezenas de bytes. 256 âncoras de 100 players são 76 kB, ou seja abaixo do
/// arredondamento do orçamento do ring que ele acompanha.
const STATE_RING_CAP: usize = 256;

/// O tipo da tabela — um mapa por tique âncora, cada um com um estado por player.
pub(super) type PlayerStateRing = BTreeMap<u64, BTreeMap<Entity, PlayerState>>;

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
