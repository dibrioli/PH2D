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

mod contract;
pub use contract::{PlayerConfig, PlayerState, PlayerStep};

pub mod corner;
pub mod crouch;
pub mod dash;
pub mod jump;
pub mod kinematic;
pub mod react;
pub mod ride;
pub mod slope;
pub mod walk;
pub mod wall;

pub use corner::{
    CORNER_LOOKAHEAD, CORNER_SAMPLES, CORNER_SEARCH_STEPS, CeilingProbe, corner_escape,
    corner_nudge, corner_offsets, corner_probe_wanted,
};
pub use crouch::{
    CrouchConfig, CrouchState, HEADROOM_SAMPLES, Headroom, crouch_step, effective_crouched,
    headroom_offsets, headroom_probe_wanted, ride_for, walk_for,
};
pub use dash::{DashConfig, DashState, DashStep, dash_burst, dash_step};
pub use jump::{JumpConfig, JumpState, JumpStep, carried_frame, jump_step};
pub use kinematic::{KinematicState, kinematic_advance, kinematic_settle, supported_velocity};
pub use react::{Reaction, ReactionConfig, push_transfer};
pub use ride::{
    RideConfig, Support, damping_axis, ride_hold, ride_spring, ride_support_on_ground, within_reach,
};
pub use slope::{Footing, footing, footing_verdict, is_grounded, no_uphill};
pub use walk::{WalkConfig, walk};
pub use wall::{
    GrabState, WALL_SAMPLES, WallConfig, WallHit, WallLaunch, WallProbe, WallSample, cling,
    grab_step, wall_launch, wall_offsets, wall_probe_wanted, wall_slide,
};

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

/// **QUANTO DO PESO o fluido está a carregar**, em `[0, 1]` — o sentido que diz
/// à lei que ela **não está num arco balístico**.
///
/// # ⚠️ Por que a lei precisa disto, e por que não é "quanto estou submerso"
///
/// A modelagem do arco de um pulo — leve no ápice, pesada na queda — descreve um
/// corpo em **voo livre**, onde a gravidade é a única força e o arco é o produto
/// dela. Quando é o **empuxo** quem o segura, os mesmos multiplicadores viram
/// **amplificação paramétrica**: pesado ao descer injeta mais energia do que
/// leve ao subir devolve, ciclo após ciclo (medido no produto: largado numa poça
/// ele oscila `−1,05 / +4,71 / +12,08 / −20,31` e sai da cena).
///
/// ⚠️ **A grandeza é a razão empuxo÷peso, e a diferença foi MEDIDA:** à tona, a
/// cápsula de controle desta linha submerge **26%** da área — uma lei que
/// desvanecesse por *"quanto está molhado"* deixaria **74% da bomba ligada
/// exactamente onde o personagem passa a vida**. A razão vale `1` ali por
/// construção, porque *boiar em repouso* **é** o empuxo igualar o peso.
///
/// # ⚠️ A LEI é uma trava, e ela mora no [`crate::JumpState`]
///
/// A lei não desvanece com a fração — ela **cala** enquanto o fluido tiver o
/// corpo, e só re-arma num contato com o CHÃO. O porquê (a energia é ganha no
/// AR, entre dois mergulhos, onde não há fluido nenhum a medir) está escrito em
/// [`crate::JumpState::waterborne`], com os números da ablação.
///
/// ⚠️ **O valor CONTÍNUO fica mesmo assim**, e não é enfeite: ele é o que a
/// sonda imprime para verificar a teoria (à tona a razão vale `1,0000` porque
/// *boiar em repouso* **é** o empuxo igualar o peso, e foi essa leitura que
/// provou que o personagem passou a assentar na linha do controle). Um `bool`
/// vindo da ponte teria escondido isso.
///
/// ⚠️ **E a MAGNITUDE não é load-bearing na lei — só o sinal é.** Está medido:
/// uma mutação que troca o denominador (a razão deixa de ser peso e vira outra
/// coisa) sangra **dois gates da CONSULTA e nenhum do produto**, porque a trava
/// pergunta `> 0`. Escrito aqui para ninguém a ler como um número que a lei
/// pesa; quem o pesa é quem lê a sonda, e a wave que trouxer natação.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Buoyed(pub f32);

impl Buoyed {
    /// Ar seco — o neutro, e o que uma cena sem poça produz.
    pub const DRY: Self = Self(0.0);

    /// **O fluido carrega ALGUMA parte deste peso?** — a única pergunta que a
    /// lei faz a este sentido.
    ///
    /// ⚠️ **O predicado mora AQUI, e não na ponte**, porque é a lei que depende
    /// dele: uma ponte que publicasse um `bool` teria decidido o limiar longe do
    /// único código que sabe o que ele significa.
    ///
    /// ⚠️ **`NaN` conta como SECO**, e é a escolha segura: uma zona degenerada
    /// não pode calar a modelagem que o artista autorou (`NaN > 0.0` é falso, e
    /// esta linha existe para dizer que isso é intencional, não descuido).
    #[must_use]
    pub fn carries_weight(self) -> bool {
        self.0 > 0.0
    }
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
    /// **O botão de ARRANQUE está pressionado agora** (W14).
    ///
    /// ⚠️ O estado, não a borda — a lei a deriva sozinha
    /// ([`DashState::was_held`]), pela razão exata do pulo: uma segunda memória
    /// do mesmo fato do lado de fora divergiria no primeiro dispatch que deve
    /// mais de um tique.
    pub dash: bool,
    /// **O botão de AGARRAR está pressionado agora** (W23).
    ///
    /// ⚠️ O estado, como os outros três — e aqui nem sequer há borda a derivar:
    /// agarrar-se é um regime que dura enquanto o dedo dura.
    pub grab: bool,
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

/// **O QUE O CHÃO AINDA DEVE** — a parte da velocidade do chão que a lei da
/// caminhada **não** põe na velocidade do personagem (K7).
///
/// # ⚠️ Porque isto existe: a plataforma era contada DUAS vezes
///
/// A [`walk`] mede tudo *relativo ao chão* (`rel = v − ground_v`) e empurra
/// `rel_along` até o alvo, então parado sobre um vagão ela leva `body_velocity`
/// **até a velocidade do vagão** — a tração é o modelo de transporte deste
/// motor. O integrador cinemático então somava `ground_velocity` outra vez, e a
/// medição é inequívoca (`tests/measure_kinematic_carry.rs`, vagão a 2 m/s por
/// 4,00 m):
///
/// | eixo | modo | tração | levado |
/// |---|---|---|---|
/// | horizontal | dinâmico | cheia | 3,95 m (**0,99×**) |
/// | horizontal | dinâmico | **zero** | 0,00 m (**0,00×**) |
/// | horizontal | CINEMÁTICO | cheia | 7,92 m (**1,98×**) |
/// | horizontal | CINEMÁTICO | **zero** | 3,97 m (0,99×) |
/// | vertical | qualquer | qualquer | ~4,00 m (1,00×) |
///
/// ⚠️ **A linha `dinâmico / zero` é a que fecha a atribuição:** desligada a
/// tração pela porta do ARTISTA (`PlatformPlayer::acceleration`), o modo
/// dinâmico não é levado **de todo** — logo quem o carrega é a caminhada, e só
/// ela. As duas somam 1,98×; o eixo vertical nunca teve o problema porque o
/// eixo da caminhada é a TANGENTE e ela não toca a normal.
///
/// # A lei
///
/// O que falta é `g` menos a projeção dele no **MESMO eixo** que a [`walk`]
/// usa. Perguntar pelo mesmo `perp_cw(normal)` — em vez de re-derivar de `up` —
/// é o que impede as duas metades de discordarem sobre *o que a tração cobre*.
///
/// ⚠️ **A representação apaga o caso degenerado:** com a normal `[0, 0]` (raio
/// nascido dentro da geometria) o eixo é `[0, 0]`, a caminhada não cobre nada, e
/// esta função devolve `g` INTEIRO — que é a resposta certa, sem um `if`.
///
/// ⚠️ **Sem tração o chão não leva de lado, e isso é físico:** um elevador
/// empurra pelo CONTATO (componente normal, sempre paga) e uma esteira leva por
/// ATRITO (componente tangente, que a caminhada modela). O modo dinâmico já se
/// comportava assim — esta lei é o que faz o cinemático concordar com ele.
#[must_use]
pub fn ground_carry(footing: Option<&GroundSample>) -> Vec2 {
    let Some(s) = footing else {
        return [0.0, 0.0];
    };
    let g = s.ground_velocity;
    // O eixo da caminhada, verbatim (`walk.rs`): a normal é unitária por
    // contrato do sensor, e é essa premissa que a projeção herda.
    let axis = perp_cw(s.normal);
    let along = g[0] * axis[0] + g[1] * axis[1];
    [g[0] - axis[0] * along, g[1] - axis[1] * along]
}

/// **A PORTA ÚNICA** — o motor inteiro de um player neste tick.
///
/// Ela existe por uma razão que não é conveniência: [`footing`] é chamada
/// **uma** vez e o resultado é entregue às duas leis. Se cada uma perguntasse
/// por conta, uma rampa de 46° com `max_slope = 45` teria a mola a segurar o
/// personagem e a caminhada a considerá-lo no ar — o estado impossível que
/// nenhum gate de lei isolada consegue ver.
///
/// ⚠️ **O `allow` de argumentos é deliberado** — o precedente é o `body_desc`
/// desta mesma linha.
///
/// ⚠️ **E a nota abaixo, escrita na W10, tem agora um QUINTO sentido a nomear**
/// ([`Buoyed`]): *o pacote certo, no dia em que valer a pena, é os SENTIDOS*, e
/// eles são cinco. Isto é uma OBSERVAÇÃO de custo, não uma promessa de wave — a
/// nota que a antecedeu prometeu empacotar *"quando a lista crescer de novo"* e
/// a W10 teve de a corrigir em vez de a cumprir. O dia barato é aquele em que
/// alguma wave já reescreva estes sítios de chamada por outro motivo.
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
    wall: Option<&WallProbe>,
    headroom: Option<&Headroom>,
    input: PlayerInput,
    state: PlayerState,
    body_velocity: Vec2,
    gravity: Vec2,
    up: Vec2,
    dt: f32,
    buoyed: Buoyed,
    support: Support,
) -> PlayerStep {
    let verdict = footing_verdict(cfg, sample, up);
    let footing = verdict.ground();

    // ⚠️ **O pulo decide PRIMEIRO, e não é ordem arbitrária:** é ele que
    // responde *"a perna pode agir?"*. No instante da decolagem o raio ainda vê
    // o chão, então uma mola viva puxaria de volta o que o boost acabou de dar
    // (o aviso do `jump`).
    let rel_up = relative_rise(footing, body_velocity, up);

    // ── A PAREDE (W13) ───────────────────────────────────────────────────────
    // ⚠️ **A pergunta *"estou agarrado?"* é feita UMA vez**, e as duas metades
    // da wave são vistas dela — o freio da queda e o que a parede oferece ao
    // pulo. Duas perguntas dariam um tique em que o personagem escorrega numa
    // superfície de que ele não consegue pular, e a diferença seria invisível
    // até alguém mexer num dos dois testes.
    let clinging = wall::cling(cfg, wall, input.drive, rel_up, up);
    // ⚠️ **A reserva anda AQUI, uma vez, ao lado da pergunta que ela qualifica.**
    // Agarrar-se cavalga o `clinging` — ele já respondeu *é parede, ele empurra,
    // ele desce* —, então o grip só acrescenta *o botão está apertado* e *ainda
    // há reserva*. Uma segunda pergunta sobre a parede daria um tique em que ele
    // se agarra a uma superfície de que não consegue pular.
    let (grab, gripping) = wall::grab_step(
        &cfg.wall,
        state.grab,
        clinging.is_some(),
        input.grab,
        footing.is_some(),
        dt,
    );
    let jump = jump_step(
        &cfg.jump,
        state.jump,
        footing,
        rel_up,
        input.jump,
        input.down,
        wall::wall_launch(&cfg.wall, clinging.as_ref()),
        gravity,
        up,
        dt,
        buoyed,
    );

    // ── O ARRANQUE (W14) ─────────────────────────────────────────────────────
    // ⚠️ **A pergunta *"o pé está no chão?"* é a MESMA que o pulo faz**, pela
    // MESMA porta ([`JumpState::on_ground`]) e sobre o estado de ENTRADA — que é
    // o que o `jump_step` também consulta. Duas cópias do predicado dariam um
    // tique em que o pulo recarrega o coyote e o arranque não recarrega a carga.
    let grounded = state.jump.on_ground(footing);
    // ⚠️ **Um pulo de QUALQUER tipo cancela o arranque**, e a pergunta é a
    // TRANSIÇÃO para o ar — não o `jump.takeoff`, que é só a decolagem do chão e
    // deixaria de fora precisamente o pulo de parede, o gesto que mais se
    // encadeia com um arranque.
    let jumped = !state.jump.airborne && jump.state.airborne;
    let dash = dash::dash_step(
        &cfg.dash,
        state.dash,
        grounded,
        input.drive,
        input.dash,
        jumped,
        dt,
    );
    let dashing = dash.active;

    // ── O AGACHAR (W15) ──────────────────────────────────────────────────────
    // ⚠️ **O `grounded` é o MESMO que o arranque consumiu**, pela mesma porta —
    // e o botão é o MESMO `down` da W12: esta wave não acrescenta entrada
    // nenhuma, e o gesto é de graça porque o dedo já estava no lugar certo.
    let crouch = crouch::crouch_step(
        &cfg.crouch,
        &cfg.ride,
        state.crouch,
        grounded,
        input.down,
        headroom,
    );
    // ⚠️ **A config EFECTIVA, e a partir daqui é ELA que manda.** A perna encurta
    // e a caminhada abranda; tudo o resto é a config autorada, ao bit.
    //
    // ⚠️ **E o que o sensor alcança NÃO se move** — a faixa de agarre cresce pelo
    // que a perna encurtou, então `float_height + cling_distance` fica
    // invariante. É isso que deixa a `footing_verdict` acima (calculada com a
    // config AUTORADA, antes de o agachar ser conhecido) responder o mesmo que
    // responderia agora, e é isso que deixa a ponte castar sem saber que esta
    // wave existe. O invariante está gateado nos dois lados — ver o topo de
    // [`crate::crouch`].
    let cfg = crouch::effective_crouched(cfg, crouch.crouched);

    // A perna e a caminhada veem o MESMO chão, e é o que o pulo lhes deixou ver:
    // duas respostas para *"estou no chão?"* seriam um personagem que anda no
    // chão enquanto voa.
    //
    // ⚠️ **E o arranque cala a perna junto** (W14): enquanto ele dura, o
    // personagem é uma velocidade — a mola disputaria o eixo vertical com o
    // boost e os dois escreveriam o mesmo número. Ver o topo de [`crate::dash`].
    let standing = if jump.spring_armed && !dashing {
        footing
    } else {
        None
    };
    // ⚠️ **A pergunta *"o que me segura?"* e' feita UMA vez** (K3): sob Snap a
    // mola cala e quem mantem a altura e' a translacao escrita no corpo.
    let spring = ride::ride_hold(support, &cfg.ride, standing, body_velocity, gravity, up);
    // ⚠️ **E este bool e' o unico consumidor da resposta fora dela**: ele decide
    // se a `gravity_hold` pode reclamar um cancelamento. Sob Snap nao ha' termo
    // de `- gravity` no `accel`, entao reclama-lo seria a ponte a subtrair uma
    // coisa que nao esta' la'.
    let spring_holds = support.is_spring() && standing.is_some();
    // ⚠️ **Só o termo de CAMINHADA passa pelo `no_uphill`, e a escolha é o
    // desenho.** A mola já está calada numa superfície recusada (é a `standing`
    // acima) e o PULO é um gesto deliberado do artista — capá-lo faria o
    // personagem perder o salto por encostar numa ladeira, que é outra feature e
    // não esta correção. O que a lei recusa é *escalar sem querer*.
    // ⚠️ **O referencial do ar é a memória do chão que se deixou** (W10), e ela
    // é lida do estado que ESTE tique produziu — a mesma ordem do coyote, que
    // enche e é consultado no mesmo tique.
    let carried = jump::carried_frame(&cfg.jump, &jump.state);
    // ⚠️ **O SILÊNCIO do controle aéreo depois de um pulo de parede** (W13) — e
    // ele é lido do estado que ESTE tique produziu, a mesma ordem do coyote:
    // o silêncio tem de valer já no tique da decolagem, senão o primeiro tique
    // de controle aéreo desfaz o empurrão que o pulo acabou de dar.
    //
    // ⚠️ **CALAR, e não zerar o `drive`:** com o alvo em zero a caminhada
    // FREARIA o empurrão — o mesmo defeito com outra roupa.
    //
    // ⚠️ **E o arranque cala a caminhada pela mesma razão que o silêncio do pulo
    // de parede a cala** (W14): com o controle aéreo vivo o jogador curvaria o
    // traço, e o desenho deixaria de ser função só do botão.
    let step = if jump.state.wall_lock > 0.0 || dashing {
        Motor::default()
    } else {
        no_uphill(
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
        )
    };

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
        // ⚠️ **O que o CHÃO sente NÃO é o motor da perna** (W-ClingPull): a
        // metade esticada da mola puxa o personagem para baixo, e transmitir a
        // reação dela puxa o chão para CIMA — medido, uma jangada subia 96,9 mm
        // ao encontro de quem caía nela. A porta que separa as duas metades mora
        // no `ride`, ao lado da lei que as produz.
        let felt = ride::ride_support_on_ground(support, &cfg.ride, standing, gravity, up);
        react::reaction(&cfg.react, felt, impulse, step)
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

    // ⚠️ **O freio da parede só vale com a perna CALADA**, e não é higiene: com
    // o pé no chão a mola já governa o eixo vertical, e um segundo termo a
    // escrever velocidade ali seria dois donos do mesmo número. A `standing` é a
    // mesma resposta que a mola consumiu.
    let slide = if standing.is_none() && !dashing {
        wall::wall_slide(&cfg.wall, clinging.is_some(), gripping, rel_up, up)
    } else {
        Motor::default()
    };

    // ⚠️ **O arranque SUBSTITUI o termo do pulo, em vez de somar a ele:** o que
    // o `jump.motor` carrega fora da decolagem é a **gravidade de FASE**
    // (`(escala − 1)·g`), e somá-la a um tique que cancela a gravidade daria
    // `−g + (escala−1)·g` — um arranque que sobe ou cai conforme a fase em que o
    // personagem estava quando o botão foi apertado. E no tique em que um pulo
    // sai, `dashing` já é falso (o arranque foi cancelado), então o boost da
    // decolagem nunca é perdido.
    let (jump_motor, burst) = if dashing {
        (
            Motor::default(),
            dash::dash_burst(
                &cfg.dash,
                dash.state.dir,
                carried,
                body_velocity,
                up,
                gravity,
            ),
        )
    } else {
        (jump.motor, Motor::default())
    };

    PlayerStep {
        motor: spring.plus(step).plus(jump_motor).plus(slide).plus(burst),
        state: PlayerState {
            jump: jump.state,
            dash: dash.state,
            crouch,
            grab,
            // ⚠️ **A LEI DE INTENÇÃO não toca a velocidade cinemática**, e é a
            // divisão inteira: ela diz o que o personagem QUER; quem integra é
            // o `kinematic_advance`, chamado pela ponte com o `was.kin` na mão.
            // Passar `was` por aqui só para o devolver seria uma segunda porta
            // para o mesmo número.
            kin: state.kin,
        },
        reaction,
        nudge,
        // ⚠️ A pergunta é *"a MOLA agiu?"*, e ela é a `standing` — não a
        // `footing`: no tique da decolagem o raio ainda vê o chão e a mola já
        // está calada. Ver [`PlayerStep::gravity_hold`].
        //
        // ⚠️ **E o ARRANQUE é o segundo dono deste canal** (W14): ele também
        // cancela a gravidade, e o cancelamento tem de ser integrado como ela,
        // por sub-passo. Os dois nunca coincidem — a `standing` é `None` sempre
        // que `dashing`, pela linha que cala a perna acima —, então isto continua
        // a ser **um** `−g`, e não dois.
        gravity_hold: if spring_holds || dashing {
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
