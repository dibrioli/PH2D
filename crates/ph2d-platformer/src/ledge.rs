//! **A BEIRADA** (`W-Ledge`) — pendurar-se num patamar, e subir por cima dele.
//!
//! # ⚠️ As três decisões de desenho, e o plano exigia-as ANTES do código
//!
//! O plano 08 §4.5 nomeia a pergunta que decide a wave: *o mantle move o corpo
//! para um lugar que a física não escolheu; no modo cinemático isso é natural,
//! no dinâmico é um teleporte, e `set_body_pose` zera a velocidade*. As três
//! respostas abaixo saíram de medição (`tests/measure_ledge.rs`), não de gosto.
//!
//! **D1 — nem o pendurar nem o mantle escrevem POSE.** Os dois são
//! **velocidade**, exatamente como o arranque: um `boost` que substitui o que
//! havia, mais o cancelamento da gravidade pelo canal que já existe
//! ([`crate::PlayerStep::gravity_hold`]). Com isso a bifurcação do plano
//! **dissolve-se**: não há teleporte a discutir, o solver continua a resolver
//! contatos, e a lei é **a mesma nos dois modos** — o `kinematic_advance`
//! integra o motor e a ponte dinâmica aplica o impulso.
//!
//! **D2 — o gatilho é uma JANELA e o que a torna suficiente é a TRAVA.** Medido
//! em queda livre, uma janela de 0,1 m dura **0 a 1 tique** a partir de meio
//! metro de queda (e mesmo 0,8 m dura 4 tiques a 12 m/s): apanhar um lábio *no
//! instante certo* é uma moeda ao ar. Agarrado à parede a mesma janela dura **8
//! tiques** — mas exigir a parede tornaria a beirada **inalcançável** em quem
//! não a autorou, porque empurrar contra uma parede sem `wall_slide_speed`
//! **prende pelo atrito** (medido: o personagem não desce). A saída é a do
//! *jump buffer*: a condição é breve, o **estado** dura. Uma vez agarrado, o
//! personagem é levado à pose canónica e fica lá.
//!
//! **D3 — o mantle é um L, e a ORDEM é a correção.** Sobe primeiro até acima do
//! lábio, atravessa depois. A diagonal cortaria a quina do patamar, e o que o
//! solver faria com isso é empurrá-lo de volta para fora.
//!
//! # ⚠️ Por que o SENSOR é um raio para BAIXO, à frente
//!
//! A §4.3 do plano deixou nomeado que achar a beirada é uma pergunta de
//! **perfil**, não de varredura — e o perfil mais barato que a responde é um
//! raio único: nascendo `grab` acima da cabeça, à frente do corpo, ele mede
//! **exatamente** a que altura está o patamar. E a rejeição do caso *"a parede
//! continua acima da minha cabeça"* sai de graça do mesmo raio: a origem cai
//! **dentro** da geometria e o cast devolve `distance == 0` (o contrato de
//! penetração que o [`ph2d_physics::PhysicsWorld::cast_ray`] publica). Medido —
//! com o corpo 0,4 m abaixo do lábio e uma janela de 0,2 m o raio reportava um
//! lábio em `y = 3,3`, onde **não há superfície nenhuma**.
//!
//! ⚠️ **O `x` do raio É o alvo do mantle** — uma segunda escolha daria um sensor
//! que mede uma beirada e um mantle que aterra noutra.

use crate::{Motor, Vec2};

/// **Quantos raios um sensor com EXTENSÃO gasta** (`W-LedgeSensor`).
///
/// ⚠️ **Cinco, e o número é do Unreal** — o mantle dele varre *"um total de 5
/// traços (este número pode ser mudado para mais ou menos precisão)"*, e o
/// irmão deste módulo (`WALL_SAMPLES`) usa 3 pela mesma razão de custo.
///
/// ⚠️ **E o que N controla é a PRECISÃO do pouso, não se o lábio é achado:** um
/// patamar plano é encontrado por qualquer amostra que caia depois da beirada,
/// e o que mais amostras compram é o `x` do pouso medido mais fino — a
/// resolução é `span / (N − 1)`.
///
/// ⚠️ Com [`LedgeConfig::span`] em zero o leque tem **uma** amostra e o custo é
/// exactamente o de antes desta wave: um raycast.
pub const LEDGE_SPAN_SAMPLES: usize = 5;

/// **A beirada, como o artista a autora.**
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LedgeConfig {
    /// **O ALCANCE do braço**, metros — a janela acima da cabeça em que um lábio
    /// é apanhado, e a distância à frente em que ele é procurado. `0.0`
    /// **desliga** a capacidade.
    ///
    /// ⚠️ **ELE É SÓ O X, e a frase que estava aqui foi REFUTADA pela
    /// referência** (`W-LedgeSensor`, 2026-08-12). Ela dizia *"um número para os
    /// dois eixos, e não é economia: até onde ele alcança é uma grandeza só"* —
    /// e o que a varredura dos motores que shipam devolve é o oposto, com o
    /// motivo do eixo Y nomeado: o **GDevelop** expõe `Grab tolerance` (o X,
    /// *quão perto ele tem de estar*) **e** `Grab offset` (o Y, e o doc diz para
    /// quê: *"to match character animation"*); o **Corgi Engine** expõe *origem e
    /// comprimento* do raycast no inspector. O Y é independente **pela ARTE** —
    /// a mão agarra onde a mão é DESENHADA, não na quina do collider —, e o
    /// nosso pendurar é derivado (`topo do corpo = lábio`), que é exactamente o
    /// caso que aquele offset existe para consertar.
    ///
    /// ⚠️ **E é ele que torna o alvo do mantle PROVADO em vez de suposto:** o
    /// raio nasce a `meia-largura + grab` à frente, então o `x` em que ele bate
    /// **é** um ponto do patamar; o corpo atravessa até pôr a borda de dentro
    /// exactamente ali. Não há aqui nenhuma suposição sobre *onde está a face da
    /// parede* — que é o que obrigaria a beirada a depender do sensor lateral.
    ///
    /// ⚠️ O zero é o idioma de todos os irmãos deste módulo (`coyote_time`,
    /// `corner_reach`, `wall_slide_speed`, `air_jumps`): a ausência da
    /// assistência, não um extremo dela. Sem um `bool` ao lado a discordar.
    ///
    /// ⚠️ **Ela mede só para CIMA**, e não é uma metade esquecida: um lábio
    /// ABAIXO da cabeça é um degrau, e um degrau já tem dono — a perna sobe-o
    /// (`cling_distance`) ou a assistência de quina desvia dele. Uma janela que
    /// olhasse para baixo faria o personagem pendurar-se em vez de subir um
    /// degrau que ele alcança a pé.
    pub grab: f32,

    /// **A ALTURA da janela**, metros — quão acima do topo do corpo o sensor
    /// nasce, e portanto a que altura um lábio ainda é apanhado.
    ///
    /// ⚠️ **É ele que possui a HISTERESE**, e por isso a janela é simétrica:
    /// `[topo − reach_y, topo + reach_y]`. **Agarrar** exige um lábio acima da
    /// cabeça (`lip_rise > 0`, que é o que *alcançar para cima* significa) e
    /// **continuar agarrado** aceita a faixa inteira — o servo mira
    /// `lip_rise = 0`, que ficaria na **borda** do alcance se o sensor só
    /// olhasse para cima, e um sensor que pisca na pose que a lei procura é um
    /// pendurar que **treme**.
    ///
    /// ⚠️ **A recomendação inicial desta wave estava ERRADA e a construção a
    /// corrigiu:** eu escrevi que a banda seria derivada do [`Self::span`]. O
    /// `span` é uma extensão **HORIZONTAL** e não tem como dar uma banda
    /// vertical; quem a dá é este número, e é assim que os três controlos
    /// continuam três em vez de quatro.
    ///
    /// ⚠️ **Não nasce em zero**, pela razão do `jump_push` da parede e da
    /// [`Self::speed`] aqui ao lado: é o que a capacidade vale quando alguém a
    /// liga, e uma janela de altura zero entregaria um `Ledge Grab` que nunca
    /// apanha nada. **0,60 é MEDIDO** — a cena 111 mediu o arco COLADO À PAREDE
    /// pondo o topo do corpo a **2,145** contra um lábio em **2,400**, logo a
    /// janela tem de passar de **0,255** para o corpo sequer a atravessar; 0,60
    /// é o valor com que o smoke foi aprovado.
    pub reach_y: f32,

    /// **A EXTENSÃO do sensor**, metros — `0` é um raio único (o mundo de antes
    /// desta wave, **ao bit**), e acima disso ele vira um **SEGMENTO** centrado
    /// no [`Self::grab`], amostrado por [`crate::LEDGE_SPAN_SAMPLES`] raios.
    ///
    /// ⚠️ **Há um defeito real por trás deste controlo, e não é conforto:** um
    /// raio acha o lábio num **único `x`**. Funciona nos blocos alinhados das
    /// cenas de smoke e erra uma quina chanfrada — e, pior, não distingue *"o
    /// lábio está dois centímetros adiante"* de *"não há lábio nenhum"*. É a
    /// mesma pergunta que o **Unreal** responde com **cinco** traços (*"este
    /// número pode ser mudado para mais ou menos precisão"*), que o **Corgi**
    /// expõe como o `length` do raycast, e que o padrão de *hotspots* do Sonic
    /// resolve com um **segmento** de extensão autorada.
    ///
    /// ⚠️ **Quem VENCE é o acerto mais PERTO do corpo** — aproximando-se de um
    /// patamar as amostras de dentro caem no vazio e as de fora batem no topo,
    /// então o acerto mais próximo **é a beirada**. E é ele que define o
    /// [`LedgeProbe::across`]: com `span > 0` o alvo do mantle deixa de ser uma
    /// função do `grab` e passa a ser o `x` que a amostra vencedora **provou**
    /// ser patamar.
    ///
    /// ⚠️ **E o zero não é só o default — é o que mantém esta wave byte-idêntica
    /// ao mundo aprovado:** com `span = 0` a leque tem uma amostra só, na
    /// posição exacta do raio de antes.
    pub span: f32,

    /// **A velocidade com que ele se acomoda no pendurar e sobe no mantle**,
    /// m/s.
    ///
    /// ⚠️ **Um número para os dois**, e é deliberado: os dois são *o mesmo
    /// gesto de braço* visto em dois momentos, e dois knobs seriam dois números
    /// que o artista teria de manter de acordo para o movimento não mudar de
    /// ritmo a meio.
    pub speed: f32,
}

impl LedgeConfig {
    /// O ponto de partida — ⚠️ **DESLIGADO**, como toda capacidade deste módulo.
    ///
    /// ⚠️ **A `speed` não nasce em zero**, pela razão que o `jump_push` da
    /// parede já documenta: ela é o que a capacidade vale quando alguém a liga,
    /// e nascer em zero faria o primeiro clique em `Ledge Grab` entregar um
    /// personagem que se agarra e **nunca sobe**.
    pub const STARTING_POINT: Self = Self {
        grab: 0.0,
        reach_y: 0.6,
        span: 0.0,
        speed: 3.0,
    };

    /// **A capacidade está armada?** — a porta única, e o sensor não é castado
    /// sem ela.
    #[must_use]
    pub fn armed(&self) -> bool {
        self.grab > 0.0
    }
}

/// **O que o raio para baixo viu** — já reduzido: se este tipo existe, há lábio.
///
/// # ⚠️ DUAS soleiras, e elas saem do `reach_y`
///
/// O sensor nasce `reach_y` acima da cabeça e desce `2·reach_y`, então ele
/// enxerga lábios em `[topo − reach_y, topo + reach_y]` e o [`Self::lip_rise`]
/// pode ser **negativo**. Isso não é folga de sobra: **agarrar** exige um lábio
/// acima da cabeça (`lip_rise > 0`, que é o que *alcançar para cima* significa),
/// e **continuar agarrado** aceita a faixa inteira — o servo mira
/// `lip_rise = 0`, que ficaria exactamente na **borda** do alcance se o sensor
/// só olhasse para cima, e um sensor que pisca na pose que a lei procura é um
/// pendurar que treme. É o idioma da trava do nado (`swim_enter`): *entrar e
/// sair pedem limiares diferentes*, e aqui os dois são derivados do mesmo
/// [`LedgeConfig::reach_y`].
///
/// ⚠️ **Antes da `W-LedgeSensor` os dois saíam do `grab`**, que era também o
/// alcance para a FRENTE — mexer no braço mexia na histerese sem que nada o
/// dissesse. Hoje cada eixo tem o seu número.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LedgeProbe {
    /// **Quanto o lábio está ACIMA do topo do corpo**, metros — em
    /// `[−grab, grab]`, e **negativo quer dizer abaixo da cabeça**.
    pub lip_rise: f32,
    /// Para que lado ele está: `-1` à esquerda, `+1` à direita.
    ///
    /// ⚠️ É o SINAL do `drive` que o achou, pela razão exata do
    /// [`crate::WallSample::side`]: o sensor olha para onde o jogador empurra, e
    /// o lado é um fato sobre o **gesto**.
    pub side: f32,
    /// **Quanto o centro do corpo tem de andar de lado** para ficar em cima do
    /// patamar, metros (sem sinal — a direção é o [`Self::side`]).
    pub across: f32,
    /// **Quanto o centro tem de SUBIR** para ficar à altura de quem está de pé
    /// no patamar, metros.
    ///
    /// # ⚠️ Por que ele vem do sensor em vez de a lei o derivar
    ///
    /// É **uma** medição — a altura do lábio — projetada de duas maneiras: o
    /// [`Self::lip_rise`] é o que o SERVO do pendurar mira (o topo do corpo no
    /// lábio) e este é o que o MANTLE mira (o centro onde a perna o seguraria).
    /// Derivá-lo aqui exigiria que a lei soubesse a meia-altura do corpo e a
    /// `float_height`, que é geometria de quem casta — é o padrão do
    /// `ProbeRay::skin`: *um fato, dois consumidores com necessidades opostas*.
    pub rise: f32,
}

/// **O que o personagem carrega entre tiques sobre a beirada.**
///
/// ⚠️ Mora no [`crate::PlayerState`] e não num mapa à parte da ponte, pela razão
/// que aquele tipo documenta: é o `PlayerState` que a fita guarda no ring de
/// tiques âncora, e um estado que vivesse noutro lugar teria de ser
/// acrescentado àquele ring **à mão**.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct LedgeState {
    /// **Agarrado** — a TRAVA que faz de uma janela de dois tiques um gesto.
    pub hanging: bool,
    /// **O botão de pulo estava apertado no tique anterior** — a BORDA sai daqui.
    ///
    /// # ⚠️ Sem ela o pendurar é INVISÍVEL, e está medido
    ///
    /// Chega-se a uma beirada a **pular contra ela**, ou seja com o botão já
    /// preso. Um gatilho de NÍVEL faria o mesmo aperto que levou o personagem lá
    /// mandá-lo subir no tique seguinte — medido na cena 111, ele acabava **de
    /// pé no patamar** (topo 3,79) num roteiro que só pedia para se pendurar, e o
    /// regime que esta wave existe para dar nunca aparecia na tela.
    ///
    /// ⚠️ É a MESMA razão do [`crate::JumpState::was_held`] e do
    /// [`crate::DashState::was_held`], e o dono da borda tem de ser **esta** lei:
    /// ela consome o botão antes do pulo, então uma borda derivada do lado de
    /// fora leria um botão que já foi mascarado.
    pub was_jump: bool,
    /// **O que falta do mantle:** `[atravessar (com sinal), subir]`, metros.
    ///
    /// ⚠️ **Uma sobra, e não um alvo em coordenadas de mundo:** o que o
    /// personagem tem de fazer é o mesmo esteja o patamar onde estiver, e um
    /// ponto absoluto guardado no estado teria de ser reinterpretado por
    /// qualquer coisa que mova o mundo debaixo dele.
    ///
    /// ⚠️ **A travessia leva o SINAL** — sem ele haveria um terceiro campo só
    /// para dizer para que lado, e ele poderia discordar deste.
    pub climb: [f32; 2],
}

impl LedgeState {
    /// **A beirada está a agir?** — a porta única que silencia as outras leis.
    #[must_use]
    pub fn busy(&self) -> bool {
        self.hanging || self.climbing()
    }

    /// **Está a subir?**
    #[must_use]
    pub fn climbing(&self) -> bool {
        self.climb[1] > 0.0 || self.climb[0] != 0.0
    }
}

/// **O que a beirada decidiu neste tique.**
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LedgeStep {
    /// O motor — ⚠️ um `boost`, e ele **substitui** o que havia.
    pub motor: Motor,
    /// O estado a guardar.
    pub state: LedgeState,
    /// **A beirada manda neste tique?** — quem lê silencia perna, caminhada,
    /// parede e pulo.
    pub active: bool,
}

/// **Vale a pena castar o raio da beirada?** — a PORTA ÚNICA da pergunta.
///
/// ⚠️ O molde é o [`crate::wall_probe_wanted`], e a razão é a mesma: a ponte
/// pergunta isto para decidir se gasta um raio e a lei pergunta o mesmo para
/// decidir se pode agir. Duas cópias dariam uma assistência que existe de um
/// lado da fronteira e não do outro.
///
/// - **No CHÃO não há beirada que interesse** — quem está de pé sobe um degrau
///   a andar.
/// - **Sem `drive` não há lado para onde olhar**, e agarrar-se é um gesto: é o
///   mesmo que a parede exige, e é o que separa *apanhar uma beirada* de *raspar
///   nela a caminho de outro lugar*.
/// - **A SUBIDA não pergunta nada** — a sobra já está no estado, e o mundo lá
///   fora não a muda.
#[must_use]
pub fn ledge_probe_wanted(
    cfg: &LedgeConfig,
    grounded: bool,
    drive: f32,
    state: LedgeState,
) -> bool {
    cfg.armed() && !state.climbing() && !grounded && drive != 0.0
}

/// **A BEIRADA, num tique.**
///
/// # ⚠️ A ordem das três perguntas é a lei
///
/// 1. **a subida continua** — ela é um gesto comprometido, e nada a interrompe
///    (nem perder o sensor, nem chegar ao chão): interrompê-la a meio deixaria o
///    personagem no ar, à altura de um patamar em que ele não está;
/// 2. **o gesto de subir** — apertar o pulo pendurado é o pedido, e ele é
///    consumido AQUI (quem está pendurado não dá um pulo de parede);
/// 3. **agarrar / continuar agarrado** — a trava, e o servo que leva o topo do
///    corpo ao lábio.
///
/// ⚠️ **Soltar-se é NÃO fazer nada** — parar de empurrar, ou apertar baixo. Não
/// há um botão de largar, e não pode haver: o dedo que solta a direção é o gesto
/// que toda a gente faz, e um botão a mais seria uma segunda porta para ele.
#[must_use]
// ⚠️ **O `allow` é o precedente do `player_motor` desta mesma crate** — os dez
// argumentos são a config, o estado, o sensor, as TRÊS metades do dedo e o
// quadro físico, e empacotá-los juntaria coisas que mudam por motivos
// diferentes.
#[allow(clippy::too_many_arguments)]
pub fn ledge_step(
    cfg: &LedgeConfig,
    state: LedgeState,
    probe: Option<&LedgeProbe>,
    drive: f32,
    jump: bool,
    down: bool,
    grounded: bool,
    body_velocity: Vec2,
    up: Vec2,
    dt: f32,
) -> LedgeStep {
    // ⚠️ **O botão é registado SEMPRE, mesmo no ramo inerte**, e é isso que faz
    // da borda uma borda: quem chega à beirada a pular já tem o dedo em baixo, e
    // um `was_jump` que só começasse a existir depois de agarrar leria esse
    // mesmo aperto como um pedido de subir.
    let idle = LedgeStep {
        motor: Motor::default(),
        state: LedgeState {
            was_jump: jump,
            ..LedgeState::default()
        },
        active: false,
    };
    if !cfg.armed() {
        return idle;
    }
    let speed = cfg.speed.max(0.0);

    // ── 1. A SUBIDA continua ─────────────────────────────────────────────────
    if state.climbing() {
        let step = speed * dt.max(0.0);
        let mut next = state;
        next.was_jump = jump;
        next.hanging = false;
        // ⚠️ **SUBIR primeiro, ATRAVESSAR depois** — a diagonal corta a quina.
        let dir = if next.climb[1] > 0.0 {
            next.climb[1] = (next.climb[1] - step).max(0.0);
            [up[0], up[1]]
        } else {
            let left = next.climb[0];
            let take = step.min(left.abs()) * left.signum();
            next.climb[0] = left - take;
            // A perpendicular do `up`, no sentido em que ainda falta andar.
            [up[1] * left.signum(), -up[0] * left.signum()]
        };
        // ⚠️ **Chegar ao fim ZERA a sobra**, e o `f32` não é convidado a
        // decidir: `max(0.0)` acima e este teste são a mesma resposta.
        if next.climb[1] <= 0.0 && next.climb[0] == 0.0 {
            next.climb = [0.0, 0.0];
        }
        return LedgeStep {
            motor: hold_at(body_velocity, [dir[0] * speed, dir[1] * speed]),
            state: next,
            active: true,
        };
    }

    // ── 2. e 3. AGARRAR ──────────────────────────────────────────────────────
    // ⚠️ **O chão dissolve a trava** — quem pousou não está pendurado, e sem
    // esta linha um personagem que aterrasse a empurrar ficaria agarrado a um
    // lábio que já não está acima dele.
    if grounded || down {
        return idle;
    }
    let Some(p) = probe else {
        return idle;
    };
    // O jogador tem de estar a empurrar CONTRA ele — o mesmo que a parede exige.
    if drive * p.side <= 0.0 {
        return idle;
    }
    // ⚠️ **AGARRAR pede o lábio acima da cabeça; CONTINUAR agarrado não** — as
    // duas soleiras do doc de [`LedgeProbe`]. Sem a segunda metade o servo
    // levaria o topo do corpo ao lábio e o próprio sucesso desligaria a lei.
    if !state.hanging && p.lip_rise <= 0.0 {
        return idle;
    }

    // ⚠️ **A BORDA, e não o nível** — ver [`LedgeState::was_jump`].
    if jump && !state.was_jump {
        // ⚠️ **A sobra é medida AGORA**, do que o sensor vê neste tique, e não
        // do que ele via quando o personagem se agarrou: o servo do pendurar
        // move o corpo, então a distância mudou.
        return LedgeStep {
            motor: hold_at(body_velocity, [0.0, 0.0]),
            state: LedgeState {
                hanging: false,
                was_jump: jump,
                climb: [p.across * p.side, p.rise],
            },
            active: true,
        };
    }

    // O SERVO: leva o topo do corpo ao lábio, sem passar da `speed`.
    let want = (p.lip_rise / dt.max(f32::EPSILON)).clamp(-speed, speed);
    LedgeStep {
        motor: hold_at(body_velocity, [up[0] * want, up[1] * want]),
        state: LedgeState {
            hanging: true,
            was_jump: jump,
            climb: [0.0, 0.0],
        },
        active: true,
    }
}

/// **O motor que SUBSTITUI a velocidade** — o `boost` que leva a velocidade
/// atual exatamente ao alvo.
///
/// ⚠️ **Substituir, e não somar**, é o que faz da beirada um gesto: quem está
/// pendurado não continua a cair um pouco, e quem sobe não sobe mais depressa
/// por ter chegado depressa. É o desenho do [`crate::dash_burst`], e a razão é a
/// mesma — durante o gesto o personagem **é** uma velocidade.
fn hold_at(body_velocity: Vec2, target: Vec2) -> Motor {
    Motor {
        accel: [0.0, 0.0],
        boost: [target[0] - body_velocity[0], target[1] - body_velocity[1]],
    }
}

#[cfg(test)]
#[path = "ledge_tests.rs"]
mod tests;
