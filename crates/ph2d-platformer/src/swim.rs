//! **NADAR** (W-Swim) — a água deixa de ser um lugar onde se cai e passa a ser
//! um lugar onde se ANDA, nos dois eixos.
//!
//! # ⚠️ É um REGIME, e por isso a frase é uma só
//!
//! Como o [`crate::dash`], nadar não é um termo que se soma ao que já havia:
//! **enquanto ele dura, o que segurava o personagem em terra cala** — a perna,
//! a caminhada e a parede. E o motivo não é gosto: as três se apoiam num CHÃO,
//! e a lei só arma o regime quando não há nenhum ao alcance dos pés. Deixá-las
//! vivas seria a mola a disputar o eixo vertical com a braçada e as duas a
//! escreverem o mesmo número, que é a mesma razão pela qual o arranque as
//! silencia.
//!
//! ⚠️ **A modelagem do ARCO já estava calada antes desta wave** — a
//! [`crate::JumpState::waterborne`] arma com qualquer fluido, e nadar implica
//! fluido. Esta wave não a toca, e é isso que mantém a cura do `857 m` intacta.
//!
//! # ⚠️ O botão de pulo vira BRAÇADA, e a razão é a ENTRADA
//!
//! O [`crate::PlayerInput`] tem **um** eixo (o `drive`, horizontal) e quatro
//! botões — o `dash` deixou isto escrito como limitação em vez de a esconder.
//! Um eixo vertical seria a resposta óbvia e **tem preço medido**: a fita do
//! jogador (o degrau v67 do `PROJECT_SCHEMA`) guarda um quadro como
//! `(f32, u8)`, então um segundo eixo muda a forma do arquivo e **recusa toda
//! corrida já salva**; um BOTÃO é um bit livre no `u8` que já lá está.
//!
//! ⇒ **`jump` é subir, `down` é descer** — o mapeamento do Mario, do Rayman e
//! de quase todo platformer com água, e o que custa zero ao formato.
//!
//! ⚠️ **Consequência NOMEADA:** enquanto se nada o botão de pulo não pula. Sair
//! da água com ele apertado enche o *buffer* do pulo (o botão volta a ser
//! botão), então quem alcança chão dentro do [`crate::JumpConfig::jump_buffer`]
//! salta — o *hop out* que os mesmos jogos têm, e o preço honesto de um botão
//! com dois significados.
//!
//! # ⚠️ O limiar de ENTRADA é medido, e o que ele mede é PESO
//!
//! A grandeza que a lei recebe é a [`Buoyed`] — *quantos pesos deste corpo o
//! fluido carrega* —, e o default é **`1.0`**: o personagem nada quando a água
//! sozinha o sustenta. Isso é uma frase de FÍSICA e não um palpite de altura:
//! `buoyed == 1` é, por construção, a linha em que ele boiaria parado, seja
//! qual for a densidade do fluido ou a dele.
//!
//! Medido na cápsula e na poça das fixtures (`measure_the_swim_threshold`,
//! `ph2d-physics`), superfície em `y = 0`:
//!
//! ```text
//!   buoyed >= 0.25  <=>   9,6% submerso
//!   buoyed >= 0.50  <=>  15,8%
//!   buoyed >= 1.00  <=>  27,2%   <- o default: a linha de flutuação
//!   buoyed >= 1.50  <=>  38,7%
//!   submerso 100%    =   3,99    (= densidade do fluido / a do corpo)
//! ```
//!
//! ⚠️ **A conversão do limiar em altura DEPENDE das duas densidades**, e é isso
//! que torna `1.0` o número certo para escrever num default: ele diz a mesma
//! coisa em toda poça (*"mais fundo do que eu boiaria"*), enquanto uma altura
//! diria coisas diferentes em cada uma. Num fluido em que o corpo mal flutua a
//! linha fica quase toda submersa — e isso é o mundo, não um defeito da lei.
//!
//! # ⚠️ A saída é uma TRAVA, não o mesmo limiar ao contrário
//!
//! Com um limiar só, o nadador oscilaria em torno dele: ele sobe, cruza a linha
//! para cima, o regime cai, ele afunda, o regime volta — no exato ponto em que o
//! jogador está a tentar sair. A histerese é a lei:
//!
//! - o CHÃO desarma (a mesma ordem da [`crate::JumpState::waterborne`]: quem
//!   tem em que se apoiar anda, e é isso que faz sair numa praia funcionar);
//! - ficar **completamente fora do fluido** desarma;
//! - entre um e outro, quem já nada continua a nadar.
//!
//! ⇒ armar pede `buoyed >= enter`; continuar pede apenas `buoyed > 0`.
//!
//! # ⚠️ E o mesmo número é a LINHA em que ele fica quando solta os controlos
//!
//! A primeira versão desta lei mirava **velocidade vertical zero** sem entrada,
//! com um doc-comment a dizer *"o que sobra é o que a água faz com ele"*. **A
//! medição desmentiu a própria frase** (`measure_the_float_line`): o servo
//! CANCELA o empuxo a cada tique, então o nadador parado não boia — ele congela
//! onde estava e sobe ao ritmo de um tique de empuxo por tique. Vinte segundos
//! parado, poça `1,25×`, autoridade `44`:
//!
//! ```text
//!   nado 0 (o mundo de ontem)   ->  80,1% submerso   (a linha de flutuação)
//!   nado 4, alvo velocidade 0   -> 100,0% submerso   (afundado, e lá fica)
//! ```
//!
//! ⇒ **ligar o nado AFUNDAVA quem boiava.** O alvo de repouso passa a ser a
//! LINHA: sem entrada, o nadador procura `buoyed == enter`.
//!
//! ⚠️ **E o número já existia — não há knob novo, de propósito.** Um `ride`
//! próprio teria de ser co-afinado com o `enter` (o valor certo de um seria
//! função do outro, que é [[feedback_ergonomics_verdict_is_a_design_bug]]), e o
//! caso degenerado dele é feio: mirar uma linha que o fluido não alcança faz o
//! nadador remar para baixo **para sempre**. Com `ride ≡ enter` esse caso é
//! **inexprimível**: o regime só arma quando `buoyed >= enter` acontece, logo a
//! linha que ele procura é sempre alcançável, por construção.
//!
//! ⚠️ **O ganho é 1 e não é escolhido:** o erro é medido em PESOS e o alvo em
//! frações da [`SwimConfig::speed`], então *um peso de erro = uma braçada
//! cheia*. Perto da linha o erro tende a zero e a braçada some com ele — o que
//! também AMORTECE o bobeio que a água sozinha tem (medido: a amplitude de
//! repouso a `4×` cai de `0,43 m` para `0,02 m`).

use crate::{Buoyed, GroundSample, Motor, Vec2};

/// Como o personagem NADA.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SwimConfig {
    /// A velocidade de nado, m/s — a mesma em qualquer direção.
    ///
    /// ⚠️ **`0` DESLIGA a capacidade inteira**, e é assim que ela nasce — a
    /// mesma decisão (e a mesma razão) das paredes, do arranque e do agachar:
    /// nadar é uma CAPACIDADE do personagem, não uma correção de física, e
    /// ligá-la por default mudaria o comportamento de todo player já autorado
    /// que tenha uma poça na cena.
    pub speed: f32,
    /// Quão depressa se chega a ela, m/s².
    ///
    /// ⚠️ **É AUTORIDADE, não só resposta:** a água empurra (empuxo) enquanto
    /// este servo corrige, então um número pequeno deixa o corpo boiar para a
    /// tona sozinho quando ninguém dirige, e um número grande deixa o nadador
    /// tredar água parado onde quiser. É o mesmo trade que a
    /// [`crate::WalkConfig::acceleration`] tem contra a gravidade numa rampa.
    ///
    /// ⚠️ **E há um piso MEDIDO para poder MERGULHAR:** num fluido `d` vezes
    /// mais denso que o corpo, o empuxo líquido sobre um corpo submerso vale
    /// `|g|·(d − 1)`, e abaixo disso o servo **não desce** — ele apenas sobe
    /// mais devagar. Água a sério (`d ≈ 1,05`) cobra `~0,5 m/s²` e o ponto de
    /// partida abaixo a cobre com folga; a poça `4×` das fixtures cobra
    /// **29,4** e o smoke `=105` usa `44`. *Não é defeito, é a física da cena:
    /// uma rolha não mergulha por querer.* Gate:
    /// `diving_needs_more_authority_than_the_water_has`.
    pub acceleration: f32,
    /// **A LINHA** — quantos pesos o fluido carrega onde ele nada.
    ///
    /// Ela responde a duas perguntas com um número só, e as duas são a mesma
    /// frase (ver o topo do módulo):
    ///
    /// 1. **a porta**: ele começa a nadar quando o fluido carrega isto;
    /// 2. **o repouso**: sem entrada nenhuma, é aqui que ele fica.
    ///
    /// O default `1.0` é *"a água sozinha me sustenta"* — a definição de nadar,
    /// e por construção a linha de flutuação da física: um nadador parado
    /// assenta onde ele assentaria sem saber nadar (medido, `1/razão` submerso:
    /// **25% · 50% · 80%** nas poças `4×` · `2×` · `1,25×`).
    pub enter: f32,
}

impl SwimConfig {
    /// Um perfil de partida — ⚠️ **NÃO são defaults de produto** (a nota dos
    /// irmãos [`crate::DashConfig::STARTING_POINT`] e
    /// [`crate::WallConfig::STARTING_POINT`]).
    ///
    /// ⚠️ **Nasce DESLIGADO** (`speed = 0`). Os outros dois carregam números
    /// úteis para que ligar a capacidade seja **um** knob e não três — quem
    /// escreve `4` no primeiro recebe um nado que funciona.
    pub const STARTING_POINT: Self = Self {
        speed: 0.0,
        acceleration: 12.0,
        enter: 1.0,
    };

    /// **A capacidade está ligada?** — a porta única do *"vale a pena?"*.
    ///
    /// Com ela falsa nada nesta lei age, e o mundo é o de antes desta wave ao
    /// bit. A aceleração entra no predicado pela razão do `time` do arranque:
    /// um nado sem orçamento de correção é um regime que silencia a perna e a
    /// caminhada **para não fazer nada** — pior do que não existir.
    #[must_use]
    pub fn armed(&self) -> bool {
        self.speed.is_finite()
            && self.speed > 0.0
            && self.acceleration.is_finite()
            && self.acceleration > 0.0
    }
}

/// O estado VIVO do nado — o que o tick anterior deixou.
///
/// ⚠️ **Mora no [`crate::PlayerState`]**, como os irmãos, e nunca no
/// componente: um campo que muda por tique faria o `canonicalize` do undo ver
/// cada quadro como um passo (a lei do ADR-0131). E morar naquele tipo é o que
/// o põe no ring de tiques âncora de graça — um estado de player fora dele é um
/// scrub que devolve o mundo de um tique e a memória do controlador de outro.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SwimState {
    /// **Ele está a nadar?** — a trava do topo do módulo.
    pub active: bool,
}

/// **A TRAVA** — a porta única do *"isto aqui é nadar?"*.
///
/// ⚠️ Ela é uma função do estado de ENTRADA e do mundo deste tique, e devolve o
/// estado de SAÍDA: quem a chama não pode responder a pergunta por outro
/// caminho, que é o que impede o motor e o silenciamento de discordarem sobre o
/// mesmo tique (a lição do `clinging` da W13, feito uma vez para as duas
/// metades da parede).
///
/// A ordem das três cláusulas é a lei, e cada uma tem um caso que só ela cobre:
///
/// | cláusula | o caso que ela cobre |
/// |---|---|
/// | `armed()` | a capacidade desligada — o mundo de antes desta wave, ao bit |
/// | `footing.is_none()` | **vadear**: com o fundo ao alcance dos pés, anda-se |
/// | `buoyed > 0` | ter saído da água — a trava larga |
/// | `was \|\| buoyed >= enter` | a histerese: raspar a superfície não arma |
#[must_use]
pub fn swim_step(
    cfg: &SwimConfig,
    was: SwimState,
    footing: Option<&GroundSample>,
    buoyed: Buoyed,
) -> SwimState {
    let wet = buoyed.0 > 0.0;
    let deep = buoyed.0 >= cfg.enter;
    SwimState {
        active: cfg.armed() && footing.is_none() && wet && (was.active || deep),
    }
}

/// **A BRAÇADA** — o servo de velocidade em dois eixos.
///
/// É o mesmo verbo da [`crate::walk`] (empurre em direção ao alvo; quando o que
/// falta cabe num tique, escreva o exato), com duas diferenças que são o
/// regime:
///
/// 1. **dois eixos**, porque o jogador pode dizer *para cima* — e o eixo
///    vertical é o `up`, não a tangente de um chão que não existe;
/// 2. **sem entrada, o eixo vertical não é zero — é a LINHA** (ver
///    [`swim_rise`]): um nadador que solta os controlos não congela onde estava,
///    ele procura a altura em que a água o sustenta.
///
/// ⚠️ **`accel`, nunca `boost`, fora da última fração** — a distinção do topo da
/// crate: nadar é um regime CONTÍNUO, então ele vira força na ponte e o solver o
/// resolve junto com os contatos. É isso que mantém o nadador empurrável por um
/// caixote e pendurável numa corda dentro d'água.
///
/// ⚠️ **A autoridade é por EIXO, não pelo módulo do vetor**: um servo que
/// dividisse um orçamento único entre os dois eixos faria a braçada na diagonal
/// ser mais fraca em cada eixo do que a mesma braçada na horizontal — o jogador
/// sentiria a diagonal a "pesar", sem nada na tela a explicar.
///
/// - `drive`: o eixo horizontal do jogador, em `[-1, 1]`.
/// - `rise`: o eixo VERTICAL derivado dos botões — ver [`vertical_drive`].
/// - `velocity`: a velocidade do corpo, em MUNDO.
#[must_use]
pub fn swim_motor(
    cfg: &SwimConfig,
    drive: f32,
    rise: f32,
    velocity: Vec2,
    up: Vec2,
    dt: f32,
) -> Motor {
    if !cfg.armed() || dt <= 0.0 {
        return Motor::default();
    }
    // O eixo horizontal é o perpendicular ao `up` — a mesma porta que a
    // caminhada usa no ar, e não um `[1, 0]` escrito à mão (que seria a segunda
    // resposta para *"para que lado é a frente?"* no dia em que a gravidade
    // apontar para outro lado).
    let side = crate::perp_cw(up);
    let target = [
        side[0] * drive.clamp(-1.0, 1.0) * cfg.speed + up[0] * rise * cfg.speed,
        side[1] * drive.clamp(-1.0, 1.0) * cfg.speed + up[1] * rise * cfg.speed,
    ];
    let window = cfg.acceleration * dt;
    let mut motor = Motor::default();
    for i in 0..2 {
        let delta = target[i] - velocity[i];
        if delta.abs() <= window {
            motor.boost[i] = delta;
        } else {
            motor.accel[i] = cfg.acceleration * delta.signum();
        }
    }
    motor
}

/// **O que o DEDO diz no eixo vertical** — a porta única dos dois botões.
///
/// ⚠️ Ela existe porque a pergunta tem **dois** consumidores hoje (a
/// [`swim_rise`] e o gate que prova que subir e descer são simétricos) e porque
/// o mapeamento é uma DECISÃO (ver o topo do módulo), não um detalhe do laço:
/// escrevê-la nos dois sítios é como o dia em que um deles ganhar um caso
/// especial nasce com o outro a discordar.
///
/// Os dois apertados dão **zero** — pedir para subir e descer ao mesmo tempo é
/// pedir para ficar onde está, e é o que o eixo horizontal já faz com um
/// direcional de duas teclas.
#[must_use]
pub fn vertical_drive(up_held: bool, down_held: bool) -> f32 {
    f32::from(up_held) - f32::from(down_held)
}

/// **O EIXO VERTICAL** — o que o dedo diz, e o que a lei faz quando ele não diz
/// nada.
///
/// ⚠️ **É a porta única do eixo**, e é por isso que ela engole a
/// [`vertical_drive`] em vez de ficar ao lado dela: com as duas alcançáveis pelo
/// chamador, o dia em que o repouso ganhar uma cláusula nasce com um sítio a
/// perguntar a antiga (a cicatriz que o `smears()` do Painter carrega).
///
/// - **dedo apertado** ⇒ o dedo manda, e a linha cala. É isto que permite sair
///   da água (subir acima da linha) e ir ao fundo (descer abaixo dela);
/// - **nada apertado** ⇒ ele procura a [`SwimConfig::enter`], em braçadas
///   proporcionais ao erro — *um peso de erro, uma braçada cheia*.
///
/// ⚠️ **O sinal:** `buoyed` cresce com a profundidade (mais submerso, mais
/// empuxo), então estar ABAIXO da linha dá erro positivo, que é subir.
#[must_use]
pub fn swim_rise(cfg: &SwimConfig, up_held: bool, down_held: bool, buoyed: Buoyed) -> f32 {
    let held = vertical_drive(up_held, down_held);
    if held != 0.0 {
        return held;
    }
    (buoyed.0 - cfg.enter).clamp(-1.0, 1.0)
}

#[cfg(test)]
#[path = "swim_tests.rs"]
mod tests;
