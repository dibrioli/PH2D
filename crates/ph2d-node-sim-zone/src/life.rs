//! **O CICLO DE VIDA da zona** — o *Emitter State* do Niagara (doc 89, folha 13, célula 60).
//!
//! A célula pede `start`/`delay`/`duration`/`loop` e responde **NÃO**, com a razão certa:
//! `ctx.started()` é *"eu emiti algo no tique passado?"*, **não um relógio**. Medido
//! (`measure_zone_life_cycle`): uma zona semeada com 5 peças e gravidade dentro já está a cair
//! no tique 0 (`y = −1,97` em `t = 1`), desce monotonamente até `−49,5` e **nunca volta** — nada
//! a montante a adia, nada a devolve ao `init`.
//!
//! ## A lei é uma função PURA do relógio, e é isso que a torna segura
//!
//! ⚠️ **A fase não guarda nada.** Ela é `f(playhead)`, então um *scrub* para trás dá exactamente
//! a mesma fase que a reprodução dava — a mesma disciplina que faz o id do `sim.spawn` ser
//! `floor(rate·t)` e não um contador (cerca 6 da folha: *"um contador faria os ids dependerem da
//! HISTÓRIA do cook; um scrub renumeraria o mundo"*).
//!
//! ⚠️ **E o `start` NÃO é construído sobre «o meu estado está vazio»** — a cerca 3 da folha
//! nomeia as duas respostas erradas já pagas por essa via. Ele é construído sobre o relógio, que
//! é a grandeza que a célula diz faltar.

/// **O piso da duração**, em segundos — ver o ramo que o usa em [`Life::phase`].
pub(crate) const MIN_DURATION: f64 = 0.01;

/// **A folga de ULP da aresta do ciclo** — ver [`Life::emit`].
///
/// ⚠️ **Ela NÃO é um epsilon de gosto: é o erro de arredondamento de UMA subtração**, e o
/// número está medido. Com `start = 1` e `dt = 1/60`, o segundo tique da janela dá
/// `(1,0 + dt) − 1,0 = 0,016666666666666607`, que é **6e-17 abaixo** de `dt =
/// 0,016666666666666666` — logo `since < dt` era **verdadeiro** ali e a zona re-semeava a sim
/// no segundo tique de cada janela, para sempre. `1e-9` é largo perante um ULP de `f64` (~1e-16
/// nesta escala) e apertado perante qualquer passo de relógio real.
const EDGE_SLACK: f64 = 1e-9;

/// Em que ponto do ciclo a zona está.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Antes do `start` — a sim ainda não nasceu.
    Dormant,
    /// A correr.
    Running,
    /// Entre dois ciclos (o `loop_delay`).
    Resting,
    /// A duração acabou e não há repetição.
    Ended,
}

/// O que a zona deve emitir NESTE tique.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Emit {
    /// Nada existe — a sim não está a correr.
    Nothing,
    /// O primeiro tique de um ciclo: relê o `init`.
    Seed,
    /// Um tique qualquer: continua do `state`.
    Carry,
}

/// **O que a zona faz depois de começar** — o `Loop Behavior` do Niagara.
///
/// ⚠️ **Um MODO e não uma sentinela**, e a casa já pagou por saber a diferença: o `mode` do
/// `motion.path` regista o raciocínio inteiro (*"uma sentinela deixaria um intervalo do param a
/// pintar os dois controles com só um a mandar — um knob que mente"*). Aqui a sentinela seria
/// `duration = 0 ⇒ para sempre`, e ela traria um **knob morto** de borda: com a duração infinita
/// um interruptor de repetição não muda um quadro, que é exactamente o que a caça do doc 90
/// existiu para apagar.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Uma vez começada, corre sempre — **a zona que shipava**.
    Forever,
    /// Corre `duration` e acaba.
    Once,
    /// Corre `duration`, descansa `rest`, recomeça do `init`.
    Loop,
}

impl Mode {
    fn of(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::Once,
            2 => Self::Loop,
            _ => Self::Forever,
        }
    }
}

/// Os números do ciclo, já resolvidos.
///
/// ⚠️ **Em `f64`, como o `playhead`.** Um ciclo curto num relógio longo é onde um `f32` perde a
/// conta (`t = 1000 s` com ciclo de `0,1 s` já não tem dígitos para a parte fraccionária), e o
/// resto (`%`) amplifica exactamente esse erro.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Life {
    pub start: f64,
    pub duration: f64,
    pub rest: f64,
    pub mode: Mode,
}

impl Life {
    /// Lê os quatro params do nó.
    pub fn of(param: &dyn Fn(&str) -> f32) -> Self {
        Self {
            start: f64::from(param("start")),
            duration: f64::from(param("duration")),
            rest: f64::from(param("loop_delay")),
            mode: Mode::of(param("mode")),
        }
    }

    /// **A zona de sempre**, palavra por palavra: sem atraso e sem fim.
    ///
    /// ⚠️ É esta pergunta que decide se a maquinaria do ciclo corre — e não um `start == 0.0`
    /// solto: em `Once` a zona TEM fim mesmo começando em zero.
    pub fn is_default(self) -> bool {
        self.start <= 0.0 && self.mode == Mode::Forever
    }

    /// Em que fase o relógio `t` cai.
    pub fn phase(self, t: f64) -> Phase {
        let local = t - self.start;
        if local < 0.0 {
            return Phase::Dormant;
        }
        if self.mode == Mode::Forever {
            return Phase::Running;
        }
        // ⚠️ **Uma duração não-positiva é COAGIDA na porta, não interpretada.** Em `Once` ou
        // `Loop` ela viria de um slider ou de um fio, e um `0` ali faria `Ended` no primeiro
        // tique (a sim nunca visível) ou uma divisão por um ciclo nulo. O piso é o menor passo
        // que o painel oferece, e abaixo dele a pergunta degenera — a lei do `MIN_SPACING` do
        // `motion.path`.
        let duration = self.duration.max(MIN_DURATION);
        if self.mode == Mode::Once {
            return if local < duration {
                Phase::Running
            } else {
                Phase::Ended
            };
        }
        // `cycle > 0` porque `duration >= MIN_DURATION` e `rest >= 0`.
        let u = local % (duration + self.rest.max(0.0));
        if u < duration {
            Phase::Running
        } else {
            Phase::Resting
        }
    }

    /// **Há quanto tempo o ciclo CORRENTE começou.** `0` no primeiro tique de uma janela.
    ///
    /// ⚠️ Só faz sentido quando a fase é [`Phase::Running`] — os outros ramos não têm janela.
    fn since_window_began(self, t: f64) -> f64 {
        let local = t - self.start;
        if self.mode == Mode::Loop {
            local % (self.duration.max(MIN_DURATION) + self.rest.max(0.0))
        } else {
            local
        }
    }

    /// **O que emitir**, dado o relógio, o passo dele e se a zona já emitiu alguma vez.
    ///
    /// ⚠️⚠️ **A aresta do ciclo pergunta «há quanto tempo esta janela começou?», e NÃO
    /// «em que fase eu estava em `t − dt`?»** — a 1.ª versão fazia a segunda, e ela tem um
    /// defeito de fronteira que só um gate no instante exacto apanha: `t` e `dt` chegam do
    /// relógio **independentemente**, então `t − dt` não reconstrói o tique anterior. Com
    /// `start = 1` e `dt = 1/60` ele dá `0,999999999999999`, que cai ANTES do `start` ⇒ a zona
    /// lia «acabei de entrar» e **re-semeava a sim no segundo tique de cada janela**, para
    /// sempre.
    ///
    /// ⚠️ **E a 2.ª versão tinha o MESMO defeito uma casa adiante**, o que é o que o torna
    /// interessante: trocar a subtração de instantes por `since < dt` não a curou, porque
    /// `since` é ele próprio uma subtração e vem `6e-17` **abaixo** de `dt` no segundo tique.
    /// *Curar o mecanismo não cura a aritmética.* Daí a folga nomeada — ver [`EDGE_SLACK`].
    ///
    /// ⚠️ **O `started` NÃO é redundante, e o caso que o exige é o `dt = 0`**: no primeiro
    /// tique de um cook o passo é zero (não há tique anterior), então `since < dt` é falso
    /// mesmo no instante exacto do começo. Sem o `started` a sim nunca semearia. É a mesma
    /// pergunta que a zona sempre fez, agora com uma segunda porta ao lado dela.
    pub fn emit(self, t: f64, dt: f64, started: bool) -> Emit {
        match self.phase(t) {
            Phase::Dormant | Phase::Resting | Phase::Ended => Emit::Nothing,
            Phase::Running if !started || self.since_window_began(t) < dt * (1.0 - EDGE_SLACK) => {
                Emit::Seed
            }
            Phase::Running => Emit::Carry,
        }
    }
}
