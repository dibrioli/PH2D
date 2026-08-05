#![forbid(unsafe_code)]
//! **A LEI de um player de plataforma** — pura, sem rapier, sem ECS, sem shell.
//!
//! Dado *(config, o que o sensor viu, a velocidade do corpo, a gravidade, a
//! entrada, dt)*, esta crate responde **o que fazer com o corpo neste tick**.
//! Quem traduz isso em chamadas de solver é a ponte
//! (`ph2d-physics-ecs::bridge::player`); quem desenha e autora é a shell. Ver
//! [`docs/Physics/06_plano_player_plataforma.md`].
//!
//! # Por que uma CÁPSULA FLUTUANTE
//!
//! O personagem **não encosta no chão**: ele paira a [`RideConfig::float_height`]
//! sobre o que o sensor achou, e a **perna é uma mola**. A imprecisão de um corpo
//! dinâmico vem da negociação de contato — que o artista não controla —, e um
//! corpo que paira não tem contato de pé para negociar. Degrau, rampa e
//! plataforma móvel deixam de ser casos especiais: são o mesmo número
//! (`distance`) entrando na mesma mola.
//!
//! É o desenho do `bevy-tnua` e da cápsula flutuante do *Very Very Valet*, e a
//! pesquisa que o escolheu (com a tabela das cinco famílias) é o doc 05.
//!
//! # ⚠️ Duas grandezas, e a distinção NÃO é cosmética
//!
//! [`Motor`] carrega **aceleração** e **boost**, e a escolha entre elas é
//! por-termo, com critério:
//!
//! - **`accel`** é o regime CONTÍNUO (a mola, a caminhada). Vira força na ponte
//!   (`força = accel × massa`), então o solver a resolve **junto com** os
//!   contatos e os joints — é isso que mantém o personagem pendurável numa corda
//!   e empurrável por um caixote.
//! - **`boost`** é escrita DIRETA de velocidade, para o que precisa ser
//!   **exato**: amortecer a mola, parar no lugar, herdar a velocidade de uma
//!   plataforma. O `bevy-tnua` chegou aos dois pelo mesmo caminho e deixou o
//!   motivo em dois issues (#34 para a força, #39 para o boost).
//!
//! ⚠️ **E é o boost que torna o amortecimento independente de `dt`:** ele remove
//! `rel_v · damping` da velocidade de uma vez, então `damping = 1.0` amortece
//! por completo em UM tick. Acima de `1.0` ele começa a inverter a velocidade
//! em vez de matá-la, e em `2.0` a inversão é total — o limite de estabilidade
//! está medido em `ride::tests::the_damping_ceiling_is_where_the_boost_inverts`,
//! e é por isso que [`RideConfig::spring_damping`] é validado contra ele.
//!
//! # A composição
//!
//! Cada lei é uma função pura e independente ([`ride_spring`], [`walk`]), e
//! [`player_motor`] é a **porta única** que as soma e — mais importante —
//! responde **UMA vez** a pergunta *"isto aqui é chão?"* ([`footing`]). Duas
//! respostas para essa pergunta seriam a mola segurando o personagem numa parede
//! que a caminhada considera intransponível.

pub mod corner;
pub mod jump;
pub mod react;
pub mod ride;
pub mod slope;
pub mod walk;

pub use corner::{
    CORNER_LOOKAHEAD, CORNER_SAMPLES, CORNER_SEARCH_STEPS, CeilingProbe, corner_escape,
    corner_nudge, corner_offsets, corner_probe_wanted,
};
pub use jump::{JumpConfig, JumpState, JumpStep, carried_frame, jump_step};
pub use react::{Reaction, ReactionConfig};
pub use ride::{RideConfig, damping_axis, ride_spring, within_reach};
pub use slope::{Footing, footing, footing_verdict, is_grounded, no_uphill};
pub use walk::{WalkConfig, walk};

/// Um vetor 2D em MUNDO (metros), na convenção do módulo (Y para cima).
pub type Vec2 = [f32; 2];

/// A perpendicular no sentido horário: com `up = [0, 1]` devolve `[1, 0]`.
///
/// É a porta única do *"para que lado é a direita?"* — do `up` sai o eixo
/// horizontal, e da normal do chão sai a tangente da rampa, com a MESMA função:
/// numa rampa que sobe para a direita a normal tomba para a esquerda, e a
/// tangente sai apontando para cima e para a direita, que é para onde se anda.
#[must_use]
pub fn perp_cw(v: Vec2) -> Vec2 {
    [v[1], -v[0]]
}

/// **O que o sensor de chão viu.** `None` no chamador significa *"nada ao
/// alcance"*, e a lei lê isso como estar no ar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GroundSample {
    /// Distância do ponto de origem do raio até a superfície.
    pub distance: f32,
    /// A normal da superfície.
    ///
    /// ⚠️ Pode vir **degenerada** (`[0, 0]`) quando o raio nasce DENTRO da
    /// geometria — é o contrato do `cast_ray` do wrapper, que reporta a
    /// penetração em vez de a esconder. A [`footing`] a trata como chão plano:
    /// não sabemos a orientação, e a suposição menos daninha é a que deixa a
    /// mola empurrar o personagem para fora.
    pub normal: Vec2,
    /// **A velocidade do CHÃO no ponto de contato.**
    ///
    /// ⚠️ É ela que faz a plataforma móvel cair de graça: tudo nesta lei é
    /// medido *relativo ao chão*, então andar sobre um vagão é andar, e o vagão
    /// acelerando não derruba ninguém. Um chão estático manda `[0, 0]`.
    pub ground_velocity: Vec2,
    /// **Este chão é uma plataforma jump-through?** (W12)
    ///
    /// ⚠️ **É o SENSOR quem responde, e é por isso que o campo mora aqui:** a
    /// lei precisa saber *que tipo de chão* achou para decidir o que o botão de
    /// pulo significa neste tique (pular, ou DESCER através dele), e a única
    /// coisa que sabe se um collider é one-way é quem o consultou. Derivá-lo
    /// noutro lugar seria uma segunda resposta para um fato que a amostra já
    /// carrega.
    ///
    /// Chão comum manda `false`, e é isso que mantém a wave inteira inerte em
    /// toda cena que nunca autorou uma plataforma jump-through.
    pub one_way: bool,
}

/// A config inteira de um player — as metades que a [`footing`] precisa
/// consultar juntas.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PlayerConfig {
    /// A perna.
    pub ride: RideConfig,
    /// A caminhada.
    pub walk: WalkConfig,
    /// O pulo.
    pub jump: JumpConfig,
    /// O que volta para o chão (a 3ª lei).
    pub react: ReactionConfig,
}

impl PlayerConfig {
    /// O ponto de partida — ⚠️ **não são defaults de produto**.
    pub const STARTING_POINT: Self = Self {
        ride: RideConfig::STARTING_POINT,
        walk: WalkConfig::STARTING_POINT,
        jump: JumpConfig::STARTING_POINT,
        react: ReactionConfig::STARTING_POINT,
    };
}

/// **O que a porta única decidiu neste tick** — as três respostas.
///
/// ⚠️ Uma struct e não uma tupla porque a lista **cresce**: a W7 traz a fita e a
/// W8 os contadores de tolerância, e cada um deles seria mais um elemento sem
/// nome num `(_, _, _, _)` que todo chamador desempacota na ordem certa por
/// disciplina.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PlayerStep {
    /// O que fazer com o corpo do personagem.
    pub motor: Motor,
    /// O estado de pulo a guardar para o próximo tick.
    pub state: JumpState,
    /// **O que devolver ao chão** — `None` quando não há chão em que empurrar.
    pub reaction: Option<Reaction>,
    /// **Um DESLOCAMENTO em metros** (W10) — a correção de quina, e a única
    /// saída desta lei que não é força nem velocidade.
    ///
    /// ⚠️ A ponte o aplica escrevendo a translação do corpo, **sem tocar a
    /// velocidade**. O porquê de não ser um `boost` está no topo de
    /// [`crate::corner`]: um impulso lateral daria o mesmo deslocamento neste
    /// tique e deixaria o personagem derivando de lado depois, porque ninguém o
    /// remove — a assistência viraria um empurrão.
    pub nudge: Vec2,
    /// **A parte do [`Self::motor`] que CANCELA a gravidade** (W11), e que por
    /// isso tem de ser integrada exatamente como ela.
    ///
    /// # ⚠️ Por que a lei declara isto em vez de a ponte o deduzir
    ///
    /// A perna carrega o personagem cancelando o peso (`− gravity` no `accel` do
    /// [`crate::ride_spring`]) — mas *se* ela está a fazê-lo depende de o pulo
    /// ter armado a mola e de a `footing` ter aceite a superfície, e nenhuma
    /// dessas duas perguntas atravessa a fronteira. Uma ponte que subtraísse
    /// `− gravity` do motor sempre que houvesse uma amostra de chão estaria a
    /// adivinhar, e adivinharia errado no exato tique da decolagem.
    ///
    /// # ⚠️ O que a ponte tem de fazer com ele, e o defeito que isso corrige
    ///
    /// O resto do `accel` é aplicado como **um impulso no topo do tique** (é o
    /// ordenamento semi-implícito, e é o que mantém a mola estável); este canal
    /// é aplicado **por SUB-PASSO**, que é como o `rapier` integra a gravidade
    /// que ele cancela.
    ///
    /// Aplicá-lo agrupado deixa a **velocidade** certa (o impulso total é o
    /// mesmo) e o **DESLOCAMENTO** errado: sobra uma fatia de tique de
    /// velocidade para cima, que numa rampa tem componente TANGENTE — e o freio
    /// da caminhada, que é um controlador de velocidade amostrado na fronteira
    /// do tique, não tem o que ver. Era essa a subida involuntária do
    /// [`BUGS_physics.md`](../../../docs/Physics/BUGS_physics.md) §7, e a
    /// medição está no `measure_substep.rs`.
    ///
    /// ⚠️ **Ele NÃO sai da reação da 3ª lei.** A força que o pé faz no chão é um
    /// fato físico (`m·(up·k − g)`) e não muda com o modo como o impulso é
    /// distribuído dentro do tique — o [`crate::react`] segue lendo o `Motor`
    /// inteiro, e é por isso que esta wave não move um único gate daquele
    /// módulo.
    ///
    /// Zero no ar, e zero é a resposta certa: quem não está a ser segurado não
    /// tem peso a cancelar.
    pub gravity_hold: Vec2,
    /// **COMEÇA a atravessar a plataforma jump-through de baixo dos pés** (W12)
    /// — verdadeiro no tique do gesto, e só nele.
    ///
    /// # ⚠️ A lei diz COMEÇAR; quem diz QUANDO ACABA é a ponte
    ///
    /// É a mesma divisão do sensor de quina (a lei pergunta *"vale a pena
    /// castar?"*, a ponte casta): decidir que o gesto aconteceu é uma pergunta
    /// sobre a ENTRADA e sobre o tipo do chão, e as duas estão aqui; decidir
    /// que o corpo já **passou** é uma pergunta sobre duas caixas envolventes,
    /// e a lei pura não tem nenhuma.
    ///
    /// Colapsar as duas num contador de segundos aqui dentro seria escolher um
    /// número onde existe uma resposta exata — e o número erraria exatamente
    /// onde a plataforma fosse grossa ou a queda lenta, re-solidificando com o
    /// personagem dentro dela.
    pub drop_through: bool,
}

/// **A entrada do jogador neste tick.**
///
/// ⚠️ Não é config e não é componente: é o que o dedo do jogador estava fazendo.
/// Hoje a ponte a guarda como estado transiente (set-and-hold); a partir da W7
/// ela vem de uma **fita por tick**, o que torna o player uma função de
/// `(tick, fita)` e devolve o scrub bit-exato que o resto do módulo tem.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PlayerInput {
    /// O eixo de caminhada em `[-1, 1]`. Positivo é a direita.
    pub drive: f32,
    /// O botão de pulo está PRESSIONADO agora.
    ///
    /// ⚠️ O estado, não a borda. A borda é derivada pela lei
    /// ([`JumpState::was_held`]), e tem de ser: quem a derivasse do lado de fora
    /// precisaria de uma segunda memória do mesmo fato, e as duas divergiriam no
    /// primeiro tick em que um dispatch devesse mais de um passo.
    pub jump: bool,
    /// **O botão de BAIXO está pressionado agora** (W12).
    ///
    /// ⚠️ Ele não anda para lugar nenhum sozinho — hoje serve a uma pergunta
    /// só: *o que o botão de pulo significa em cima de uma plataforma
    /// jump-through?* Segurado, o pulo vira **descida**
    /// ([`PlayerStep::drop_through`]).
    ///
    /// ⚠️ **É `down + jump`, e não `down` sozinho, de propósito:** um jogador
    /// que segura baixo enquanto anda não pode cair da plataforma sem ter
    /// pedido, e o dia em que existir um AGACHAR o botão já estará lá com o
    /// significado certo. É o idioma de Celeste, Hollow Knight, Ori e Dead
    /// Cells.
    pub down: bool,
}

/// **O que fazer com o corpo neste tick.** Ver a distinção accel/boost no topo.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Motor {
    /// Aceleração desejada (m/s²) — vira FORÇA na ponte.
    pub accel: Vec2,
    /// Mudança instantânea de velocidade (m/s) — escrita direta.
    pub boost: Vec2,
}

impl Motor {
    /// Soma dois termos do motor.
    ///
    /// As leis são independentes e **aditivas** por desenho: é isso que permite
    /// gatear a mola sem a caminhada e vice-versa, e o que fará o pulo (W4)
    /// entrar sem reabrir nenhuma das duas.
    #[must_use]
    pub fn plus(self, other: Self) -> Self {
        Self {
            accel: [
                self.accel[0] + other.accel[0],
                self.accel[1] + other.accel[1],
            ],
            boost: [
                self.boost[0] + other.boost[0],
                self.boost[1] + other.boost[1],
            ],
        }
    }
}

/// **A velocidade de SUBIDA relativa ao chão** — o número em que quase toda esta
/// lei ramifica (o pouso, as fases da gravidade, a porta do sensor de teto).
///
/// ⚠️ **Porta única, e a W10 é quem a exigiu:** a ponte precisa da MESMA grandeza
/// para decidir se casta os raios da quina ([`corner_probe_wanted`]), e a
/// tentação era ela ler `velocidade · up` direto — igual **enquanto** o probe só
/// existir no ar, onde a velocidade do chão é zero. Uma premissa verdadeira por
/// acidente de escopo é exatamente a que envelhece: bastaria um dia oferecer a
/// assistência de pé numa plataforma que sobe.
#[must_use]
pub fn relative_rise(footing: Option<&GroundSample>, body_velocity: Vec2, up: Vec2) -> f32 {
    let g = footing.map_or([0.0, 0.0], |s| s.ground_velocity);
    (body_velocity[0] - g[0]) * up[0] + (body_velocity[1] - g[1]) * up[1]
}

/// **A PORTA ÚNICA** — o motor inteiro de um player neste tick.
///
/// Ela existe por uma razão que não é conveniência: [`footing`] é chamada
/// **uma** vez e o resultado é entregue às duas leis. Se cada uma perguntasse
/// por conta, uma rampa de 46° com `max_slope = 45` teria a mola a segurar o
/// personagem e a caminhada a considerá-lo no ar — o estado impossível que
/// nenhum gate de lei isolada consegue ver.
///
/// ⚠️ **Nove argumentos, e o `allow` é deliberado** — o precedente é o
/// `body_desc` desta mesma linha.
///
/// ⚠️ **A nota antiga prometia empacotar o quadro físico "quando a lista crescer
/// de novo", e a W10 a corrigiu em vez de a cumprir:** o argumento que entrou
/// (`ceiling`) **não é** parte do quadro físico — é um **segundo SENSOR**, irmão
/// do `sample`, e empacotá-lo com `gravity`/`dt` juntaria coisas que mudam por
/// motivos diferentes. O pacote certo, no dia em que valer a pena, é *os
/// sentidos* (`ground` + `ceiling`), não *o quadro*; hoje seriam duas linhas de
/// cerimônia para dois campos.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn player_motor(
    cfg: &PlayerConfig,
    sample: Option<&GroundSample>,
    ceiling: Option<&CeilingProbe>,
    input: PlayerInput,
    state: JumpState,
    body_velocity: Vec2,
    gravity: Vec2,
    up: Vec2,
    dt: f32,
) -> PlayerStep {
    let verdict = footing_verdict(cfg, sample, up);
    let footing = verdict.ground();

    // ⚠️ **O pulo decide PRIMEIRO, e não é ordem arbitrária:** é ele que
    // responde *"a perna pode agir?"*. No instante da decolagem o raio ainda vê
    // o chão, então uma mola viva puxaria de volta o que o boost acabou de dar
    // (o aviso do `jump`).
    let rel_up = relative_rise(footing, body_velocity, up);
    let jump = jump_step(
        &cfg.jump, state, footing, rel_up, input.jump, input.down, gravity, up, dt,
    );

    // A perna e a caminhada veem o MESMO chão, e é o que o pulo lhes deixou ver:
    // duas respostas para *"estou no chão?"* seriam um personagem que anda no
    // chão enquanto voa.
    let standing = if jump.spring_armed { footing } else { None };
    let spring = ride_spring(&cfg.ride, standing, body_velocity, gravity, up);
    // ⚠️ **Só o termo de CAMINHADA passa pelo `no_uphill`, e a escolha é o
    // desenho.** A mola já está calada numa superfície recusada (é a `standing`
    // acima) e o PULO é um gesto deliberado do artista — capá-lo faria o
    // personagem perder o salto por encostar numa ladeira, que é outra feature e
    // não esta correção. O que a lei recusa é *escalar sem querer*.
    // ⚠️ **O referencial do ar é a memória do chão que se deixou** (W10), e ela
    // é lida do estado que ESTE tique produziu — a mesma ordem do coyote, que
    // enche e é consultado no mesmo tique.
    let carried = jump::carried_frame(&cfg.jump, &jump.state);
    let step = no_uphill(
        walk(
            &cfg.walk,
            standing,
            body_velocity,
            up,
            input.drive,
            carried,
            dt,
        ),
        verdict.steep(),
        up,
    );

    // ── A 3ª LEI ─────────────────────────────────────────────────────────────
    // Só há em quem empurrar se houver chão. ⚠️ E o que volta é o CONTATO: a
    // mola (o peso) e o empurrão da decolagem, nunca a gravidade de fase — ver
    // o `react`.
    let reaction = footing.map(|_| {
        // ⚠️ Os canais entram SEPARADOS: a mola é força contínua, a decolagem é
        // um impulso de um tick só, e a diferença decide se o `boost` volta —
        // ver o aviso do `react`.
        let impulse = if jump.takeoff {
            jump.motor
        } else {
            Motor::default()
        };
        react::reaction(&cfg.react, spring, impulse, step)
    });

    // ── A QUINA (W10) ────────────────────────────────────────────────────────
    // ⚠️ A porta é a MESMA que a ponte consulta para decidir se casta os raios
    // (`corner_probe_wanted`), então a assistência não pode existir num lado e
    // não no outro. Sem sensor não há correção — e um sensor entregue fora da
    // hora ainda assim não age.
    let nudge = if corner::corner_probe_wanted(&cfg.jump, footing.is_some(), rel_up) {
        corner::corner_nudge(ceiling, &cfg.jump, up)
    } else {
        [0.0, 0.0]
    };

    PlayerStep {
        motor: spring.plus(step).plus(jump.motor),
        state: jump.state,
        reaction,
        nudge,
        // ⚠️ A pergunta é *"a MOLA agiu?"*, e ela é a `standing` — não a
        // `footing`: no tique da decolagem o raio ainda vê o chão e a mola já
        // está calada. Ver [`PlayerStep::gravity_hold`].
        gravity_hold: if standing.is_some() {
            [-gravity[0], -gravity[1]]
        } else {
            [0.0, 0.0]
        },
        drop_through: jump.drop_through,
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
