//! **O ARRANQUE** (W14) — uma linha reta, e nada mais acontece enquanto ela dura.
//!
//! # ⚠️ A velocidade é DEFINIDA, nunca somada
//!
//! É a lição que a W13 pagou com uma medição: um arranque somado a uma queda de
//! 8 m/s é um arranque diferente do mesmo arranque somado a uma caminhada, e o
//! artista que digita `18 m/s` está a descrever a **coisa que ele vê**, não uma
//! parcela dela. O boost leva a velocidade AO alvo — o mesmo verbo do
//! [`crate::wall_slide`] e do salto parametrizado por altura.
//!
//! # ⚠️ Enquanto ele dura, mais NADA age — e isso é uma regra, não três
//!
//! A perna cala, a caminhada cala, a gravidade é cancelada. Um arranque com
//! gravidade sagaria; com controle aéreo o jogador conseguiria curvá-lo; com a
//! perna viva a mola disputaria o eixo vertical com o boost e os dois
//! escreveriam o mesmo número. **Silenciar os três é uma frase só** — *durante
//! o arranque o personagem é uma velocidade*, e é o que faz o desenho ser um
//! traço reto em vez de um arco que depende de onde ele começou.
//!
//! ⚠️ Isso vale também no CHÃO: a perna cala e a gravidade é cancelada, então
//! ele atravessa à altura de flutuação em que estava e a perna volta a pegá-lo
//! no tique seguinte ao fim. É o mesmo desenho que o pulo já usa
//! ([`crate::jump`]), pela mesma razão.
//!
//! # ⚠️ UM arranque por tempo-de-voo, e a recarga é o CHÃO
//!
//! Um relógio de recuperação sozinho deixaria voar: bastaria esperar o tempo e
//! arrancar de novo, para sempre, sem tocar o chão. A carga é reposta pelo pé no
//! chão — o mesmo argumento que impede o coyote de virar pulo duplo (`JumpState`
//! §coyote). O relógio existe para outra coisa: impedir que se encadeiem
//! arranques no chão mais rápido do que a animação consegue mostrar.
//!
//! # ⚠️ Horizontal, e a razão é a ENTRADA — não a lei
//!
//! [`crate::PlayerInput`] tem **um eixo**, então um arranque diagonal não é
//! exprimível: não há como o jogador dizer *"para cima e para a frente"*. A lei
//! aqui já toma uma direção, então o dia em que a entrada ganhar um eixo
//! vertical, é a entrada que muda — e não este módulo. Nomeado em vez de
//! escondido, para ninguém procurar a limitação aqui dentro.

use crate::{Motor, Vec2, perp_cw};

/// Como o personagem ARRANCA.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DashConfig {
    /// A velocidade do arranque, m/s.
    ///
    /// ⚠️ **`0` DESLIGA a capacidade inteira**, e é assim que ela nasce — a
    /// mesma decisão (e a mesma razão) do escorregamento de parede: um arranque
    /// é uma CAPACIDADE do personagem, não uma correção de física, e ligá-lo por
    /// default mudaria o comportamento de todo player já autorado.
    pub speed: f32,
    /// Quanto tempo ele dura, segundos.
    ///
    /// ⚠️ Junto com a velocidade, é ele que decide a **DISTÂNCIA** — que é o
    /// número que o artista de facto julga (*"este arranque atravessa aquele
    /// buraco"*). Ver a tabela do `measure_dash`.
    pub time: f32,
    /// Quanto tempo depois do FIM até poder arrancar de novo, segundos.
    ///
    /// ⚠️ **Do FIM, não do começo**, e a diferença importa: medido do começo,
    /// baixar o `time` encurtaria a recuperação junto — dois números que o
    /// artista mexe por razões diferentes acabariam presos um ao outro.
    pub cooldown: f32,
}

impl DashConfig {
    /// Um perfil de partida — ⚠️ **NÃO são defaults de produto** (a nota dos
    /// irmãos [`crate::WallConfig::STARTING_POINT`] e
    /// [`crate::JumpConfig::STARTING_POINT`]).
    ///
    /// ⚠️ **Nasce DESLIGADO** (`speed = 0`): ver [`DashConfig::speed`]. Os outros
    /// dois carregam números úteis para que ligar a capacidade seja **um** knob e
    /// não três — quem escreve `18` no primeiro recebe um arranque que funciona.
    pub const STARTING_POINT: Self = Self {
        speed: 0.0,
        time: 0.15,
        cooldown: 0.2,
    };

    /// **A capacidade está ligada?** — a porta única do *"vale a pena?"*.
    ///
    /// Com ela falsa nada nesta lei age, e o mundo é o de antes desta wave ao
    /// bit. `time <= 0` conta como desligado porque um arranque de duração zero
    /// é um botão que não faz nada: melhor recusar do que consumir a carga.
    #[must_use]
    pub fn armed(&self) -> bool {
        self.speed.is_finite() && self.speed > 0.0 && self.time.is_finite() && self.time > 0.0
    }
}

/// O estado VIVO de um arranque — o que o tick anterior deixou.
///
/// ⚠️ **Mora na PONTE, dentro do [`crate::PlayerState`]**, e nunca no
/// componente: um campo que muda por tick faria o `canonicalize` do undo ver
/// cada frame como um passo (a lei do ADR-0131 que o `JumpState` já honra). E
/// morar no MESMO tipo que o estado de pulo é o que o põe no ring da fita de
/// graça — um estado de player fora daquele ring seria um scrub que devolve o
/// mundo de um tique e a memória do controlador de outro.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DashState {
    /// Segundos restantes do arranque em curso. `0` = não está a arrancar.
    pub left: f32,
    /// A direção do arranque EM CURSO, `+1` ou `−1`.
    ///
    /// ⚠️ Congelada no instante do arranque, e é o que o torna um traço reto:
    /// lê-la do `drive` vivo deixaria o jogador virar no meio do gesto, e o que
    /// ele desenharia seria uma curva com a assistência inteira ligada — ou
    /// seja, exactamente o que este módulo silencia.
    pub dir: f32,
    /// Segundos até poder arrancar de novo.
    pub cool: f32,
    /// **Há um arranque em mãos?** — reposto pelo pé no chão, gasto ao arrancar.
    ///
    /// ⚠️ É ele que impede voar; o [`DashConfig::cooldown`] não faria isso
    /// sozinho (ver o aviso do módulo).
    pub charged: bool,
    /// **Para que lado o personagem olha** — a última direção não-nula.
    ///
    /// ⚠️ **Mora aqui porque o arranque é o único consumidor HOJE**, e é a
    /// resposta honesta a um botão apertado com o eixo neutro: sem ele, arrancar
    /// parado seria uma recusa em silêncio, que é a forma de um botão parecer
    /// quebrado. Quando um segundo consumidor aparecer (uma animação, um tiro),
    /// ele sobe para o [`crate::PlayerState`] — e a mudança é mecânica.
    pub facing: f32,
    /// O botão estava segurado no tick anterior — a BORDA sai daqui.
    ///
    /// A mesma razão do [`crate::JumpState::was_held`]: sem ela, segurar a tecla
    /// re-arrancaria assim que a recuperação acabasse, para sempre.
    pub was_held: bool,
}

impl Default for DashState {
    /// ⚠️ **Não é `derive`**, e as duas diferenças são deliberadas: um player
    /// nasce **olhando para a direita** (`facing = 1`, e `0` não é uma direção)
    /// e **com o arranque em mãos** (`charged`) — a carga é reposta pelo chão, e
    /// quem nasce no ar não devia começar em dívida.
    fn default() -> Self {
        Self {
            left: 0.0,
            dir: 1.0,
            cool: 0.0,
            charged: true,
            facing: 1.0,
            was_held: false,
        }
    }
}

/// O que a lei do arranque decidiu neste tick.
pub struct DashStep {
    /// O estado a guardar para o próximo tick.
    pub state: DashState,
    /// **Está a arrancar NESTE tique** — é este bool que cala a perna, a
    /// caminhada e a gravidade.
    pub active: bool,
}

/// **ARRANCAR.** Ver a lei no topo do módulo.
///
/// - `grounded`: o pé está no chão — é ele que RECARREGA.
/// - `drive`: o eixo de caminhada, de onde sai a direção que o personagem olha.
/// - `held`: o botão de arranque está pressionado AGORA (o estado, não a borda —
///   ela é derivada aqui, como no pulo).
/// - `cancelled`: alguma coisa mais forte aconteceu neste tique e o arranque
///   acaba já. Hoje é **um pulo**, de qualquer tipo — ver o chamador.
#[must_use]
pub fn dash_step(
    cfg: &DashConfig,
    state: DashState,
    grounded: bool,
    drive: f32,
    held: bool,
    cancelled: bool,
    dt: f32,
) -> DashStep {
    let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
    let pressed = held && !state.was_held;
    let mut next = DashState {
        was_held: held,
        ..state
    };

    // ── PARA ONDE ELE OLHA ───────────────────────────────────────────────────
    // ⚠️ Um eixo neutro **não** apaga a direção: parar de andar não é virar-se
    // para lugar nenhum, e é isso que dá resposta a um arranque com o eixo solto.
    if drive > 0.0 {
        next.facing = 1.0;
    } else if drive < 0.0 {
        next.facing = -1.0;
    }

    // ── A RECARGA ────────────────────────────────────────────────────────────
    // O pé no chão devolve o arranque — não um relógio (ver o aviso do módulo).
    if grounded {
        next.charged = true;
    }
    // A recuperação escorre em segundos de RELÓGIO, nunca em contagem de tiques:
    // a mesma lei dos dois relógios do perdão.
    next.cool = (state.cool - dt).max(0.0);

    // ── O ARRANQUE EM CURSO ──────────────────────────────────────────────────
    if state.left > 0.0 {
        // ⚠️ **O cancelamento é imediato e NÃO é activo neste tique**: quem
        // cancela é um pulo, e o boost dele já está no motor — deixar o arranque
        // activo faria os dois escreverem a mesma velocidade, com o último a
        // ganhar. Um pulo que sai de um arranque tem de ser um pulo.
        if cancelled {
            next.left = 0.0;
            next.cool = cfg.cooldown.max(0.0);
            return DashStep {
                state: next,
                active: false,
            };
        }
        next.left = (state.left - dt).max(0.0);
        if next.left <= 0.0 {
            next.cool = cfg.cooldown.max(0.0);
        }
        return DashStep {
            state: next,
            active: true,
        };
    }

    // ── COMEÇAR ──────────────────────────────────────────────────────────────
    // ⚠️ O tique do arranque **consome o próprio `dt`**, como todo relógio deste
    // módulo. Sem isso um arranque de `time` segundos dura um tique a mais, e a
    // distância percorrida passa a depender da taxa da sim — que é exactamente a
    // dependência que os relógios em segundos existem para não ter.
    if cfg.armed() && pressed && !cancelled && next.charged && next.cool <= 0.0 {
        next.left = (cfg.time - dt).max(0.0);
        next.charged = false;
        next.dir = next.facing;
        return DashStep {
            state: next,
            active: true,
        };
    }

    DashStep {
        state: next,
        active: false,
    }
}

/// **O motor de um tique de arranque** — a velocidade que ele DEFINE.
///
/// ⚠️ O alvo é medido no referencial do chão (`carried`), como tudo nesta lei:
/// arrancar de cima de um vagão a 5 m/s leva os 5 m/s junto, em vez de os apagar.
/// Em chão estático — e depois de a janela da memória fechar — `carried` é
/// `[0, 0]` e a expressão reduz ao arranque cru.
///
/// ⚠️ **A vertical vai a zero relativo**, e é metade do que faz um arranque ser
/// um arranque: sem isso, arrancar durante uma queda seria uma diagonal, e o
/// mesmo botão daria desenhos diferentes conforme a altura de onde se saltou.
///
/// ⚠️ **A gravidade é cancelada aqui, no `accel`** — e o chamador declara o mesmo
/// vetor em [`crate::PlayerStep::gravity_hold`], porque é ele que a ponte integra
/// **por sub-passo** (a lei da W11). Cancelá-la só no boost deixaria a velocidade
/// certa e o deslocamento errado, que é o defeito que aquela wave mediu.
#[must_use]
pub fn dash_burst(
    cfg: &DashConfig,
    dir: f32,
    carried: Vec2,
    body_velocity: Vec2,
    up: Vec2,
    gravity: Vec2,
) -> Motor {
    let right = perp_cw(up);
    let speed = if cfg.speed.is_finite() {
        cfg.speed.max(0.0)
    } else {
        0.0
    };
    let target = [
        carried[0] + right[0] * dir * speed,
        carried[1] + right[1] * dir * speed,
    ];
    Motor {
        accel: [-gravity[0], -gravity[1]],
        boost: [target[0] - body_velocity[0], target[1] - body_velocity[1]],
    }
}

/// Os gates desta lei — irmão por `#[path]` pelo cap de 700 LOC, o molde do
/// `jump_tests`.
#[cfg(test)]
#[path = "dash_tests.rs"]
mod tests;
