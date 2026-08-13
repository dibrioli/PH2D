//! **O que a LEI fala** — a configuração autorada, o estado que atravessa um
//! tique, e as três respostas de um passo.
//!
//! ⚠️ **Corte por ASSUNTO, não por tamanho:** o `lib.rs` responde *"o que o
//! personagem faz"* (o `player_motor` e os helpers dele) e este arquivo responde
//! *"com que vocabulário se pergunta"*. Os três tipos crescem juntos e por
//! wave — cada capacidade nova acrescenta um campo em cada — e é isso que os
//! torna uma unidade.
//!
//! Re-exportados na raiz, então nenhum caminho de chamador muda.

use crate::{
    Motor, Reaction, Vec2, crouch::CrouchConfig, crouch::CrouchState, dash::DashConfig,
    dash::DashState, glide::GlideConfig, jump::JumpConfig, jump::JumpState, kinematic,
    ledge::LedgeConfig, ledge::LedgeState, react::ReactionConfig, ride::RideConfig,
    swim::SwimConfig, swim::SwimState, walk::WalkConfig, wall::GrabState, wall::WallConfig,
};

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
    /// As paredes (W13) — ⚠️ nasce DESLIGADA, ver [`WallConfig::STARTING_POINT`].
    pub wall: WallConfig,
    /// O arranque (W14) — ⚠️ nasce DESLIGADO, ver [`DashConfig::STARTING_POINT`].
    pub dash: DashConfig,
    /// O agachar (W15) — ⚠️ nasce DESLIGADO, ver [`CrouchConfig::STARTING_POINT`].
    pub crouch: CrouchConfig,
    /// O nado (W-Swim) — ⚠️ nasce DESLIGADO, ver [`SwimConfig::STARTING_POINT`].
    pub swim: SwimConfig,
    /// A beirada (W-Ledge) — ⚠️ nasce DESLIGADA, ver [`LedgeConfig::STARTING_POINT`].
    pub ledge: LedgeConfig,
    /// O planeio (W-Glide) — ⚠️ nasce DESLIGADO, ver [`GlideConfig::STARTING_POINT`].
    pub glide: GlideConfig,
}

impl PlayerConfig {
    /// O ponto de partida — ⚠️ **não são defaults de produto**.
    pub const STARTING_POINT: Self = Self {
        ride: RideConfig::STARTING_POINT,
        walk: WalkConfig::STARTING_POINT,
        jump: JumpConfig::STARTING_POINT,
        react: ReactionConfig::STARTING_POINT,
        wall: WallConfig::STARTING_POINT,
        dash: DashConfig::STARTING_POINT,
        crouch: CrouchConfig::STARTING_POINT,
        swim: SwimConfig::STARTING_POINT,
        ledge: LedgeConfig::STARTING_POINT,
        glide: GlideConfig::STARTING_POINT,
    };
}

/// **O estado que a LEI carrega entre tiques** — tudo o que o tick anterior
/// deixou, num tipo só.
///
/// # ⚠️ Por que UM tipo, e não um mapa por assunto na ponte
///
/// Este é o valor que a **fita** (W7) guarda no ring de tiques âncora, e é isso
/// que decide a forma: um estado de player que vivesse num segundo mapa da ponte
/// teria de ser acrescentado àquele ring **à mão**, e esquecê-lo é um scrub que
/// devolve o mundo de um tique e a memória do controlador de outro — sem erro,
/// sem aviso, e visível só como *"o arranque some quando arrasto a régua"*.
/// Estando aqui, um assunto novo entra no ring de graça.
///
/// ⚠️ **E o [`JumpState`] mantém o nome**: ele é o estado que [`jump_step`] toma
/// e devolve, e continua a ser só isso. Empurrar o arranque para dentro dele
/// daria a `jump_step` um campo que ela nunca toca — um nome que mente por
/// conveniência de armazenamento.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PlayerState {
    /// **PARA ONDE ELE OLHA** (`W-PlayerOut`) — `+1` direita, `−1` esquerda.
    ///
    /// ⚠️ **Ele morava dentro do [`DashState`] e mudou de casa**, pelo argumento
    /// que este mesmo doc já faz contra guardar o arranque dentro do
    /// [`JumpState`]: quem o ESCREVE é a caminhada, quem o LÊ é o arranque (para
    /// escolher a direção) e quem o PUBLICA é o readout — um campo do arranque
    /// era um nome que mentia por conveniência de armazenamento.
    ///
    /// ⚠️ **Um eixo neutro NÃO o apaga:** parar de andar não é virar-se para
    /// lugar nenhum, e é isso que dá resposta a um arranque com o eixo solto.
    pub facing: f32,
    /// O pulo, o corte, os dois relógios do perdão e a memória do chão.
    pub jump: JumpState,
    /// O arranque (W14) — o relógio, a carga, a direção e o lado que ele olha.
    pub dash: DashState,
    /// O agachar (W15) — um bit, e ele existe porque levantar-se pode ser
    /// RECUSADO (ver o topo de [`crate::crouch`]).
    pub crouch: CrouchState,
    /// O agarrar-se (W23) — quanto já se gastou da reserva de parede.
    pub grab: GrabState,
    /// O nado (W-Swim) — a TRAVA, e ela existe porque entrar e sair pedem
    /// limiares diferentes (ver o topo de [`crate::swim`]).
    pub swim: SwimState,
    /// A beirada (W-Ledge) — a trava do pendurar e o que falta da subida.
    ///
    /// ⚠️ A **trava** é o que faz de uma janela de dois tiques um gesto (o
    /// número está no topo de [`crate::ledge`]), e ela só serve para alguma
    /// coisa por estar aqui: um gatilho sem estado seria um agarre que depende
    /// de o sensor calhar de responder no tique certo.
    pub ledge: LedgeState,
    /// **A velocidade do modo CINEMÁTICO** (W-KinMove, K5) — o que o solver
    /// possuiria se o corpo fosse dinâmico.
    ///
    /// ⚠️ Mora AQUI e não num mapa da ponte pela razão que o doc deste tipo já
    /// enuncia acima: este é o valor que o ring de tiques âncora guarda, e um
    /// estado que vivesse noutro mapa teria de ser acrescentado ao ring **à
    /// mão** — esquecê-lo é um scrub que devolve o mundo de um tique e a
    /// memória do controlador de outro. **Inerte no modo dinâmico** (fica em
    /// zero: ninguém o escreve), então o campo é byte-neutro para quem não o usa.
    pub kin: kinematic::KinematicState,
}

impl Default for PlayerState {
    /// ⚠️ **O personagem nasce olhando para a DIREITA** (`facing = 1`), e `0`
    /// não é uma direção — era o default manual do [`DashState`], que veio junto
    /// com o campo.
    fn default() -> Self {
        Self {
            facing: 1.0,
            jump: JumpState::default(),
            dash: DashState::default(),
            crouch: CrouchState::default(),
            grab: GrabState::default(),
            swim: SwimState::default(),
            ledge: LedgeState::default(),
            kin: kinematic::KinematicState::default(),
        }
    }
}

/// **O que o personagem ESTÁ A FAZER** — o readout publicado (`W-PlayerOut`).
///
/// # ⚠️ Por que ele existe, e por que não é o [`PlayerState`]
///
/// O estado é o que a lei precisa para o PRÓXIMO tique (relógios, travas,
/// cargas); este é o que o RESTO DO APP precisa para desenhar, tocar e animar.
/// Publicar o estado cru obrigaria cada consumidor a re-derivar *"isto significa
/// que ele está agarrado?"* — e cada um o faria à sua maneira.
///
/// # ⚠️ FATOS, nunca um `enum` de regime único
///
/// Agachado **e** no chão são simultâneos; arrancar no ar também. Um enum teria
/// de escolher uma prioridade, e essa escolha é do JOGO — é literalmente para
/// isso que o `bevy_tnua` publica *animation helpers* em vez de um estado. Um
/// enum aqui seria uma segunda resposta ao lado dos bits, e a primeira a
/// divergir.
///
/// # ⚠️ Os dois vetores são FATOS; a diferença é interpretação
///
/// [`Self::velocity`] é a do corpo e [`Self::ground_velocity`] a da plataforma.
/// A **relativa** — que é a que casa com um ciclo de caminhada — é a subtração,
/// e publicá-la seria a mesma coisa dita duas vezes.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PlayerView {
    /// A postura: no ar · encostado numa rampa íngreme demais · no chão.
    pub footing: crate::FootingKind,
    /// Agarrado a uma parede, e de que LADO ela está (`+1` à direita).
    pub wall: Option<f32>,
    /// Segurando a parede de vez (o botão de agarrar, `W23`).
    pub gripping: bool,
    /// Agachado.
    pub crouching: bool,
    /// Nadando.
    pub swimming: bool,
    /// Pendurado numa beirada ou a subi-la.
    pub ledging: bool,
    /// Em arranque.
    pub dashing: bool,
    /// A planar (o teto de descida está a agir).
    pub gliding: bool,
    /// Para onde ele olha — ver [`PlayerState::facing`].
    pub facing: f32,
    /// A velocidade do CORPO, em mundo.
    pub velocity: Vec2,
    /// A velocidade do CHÃO no ponto de contato (`[0, 0]` sem chão).
    pub ground_velocity: Vec2,
    /// A normal do chão que a lei ACEITOU.
    pub ground_normal: Option<Vec2>,
    /// Quantos pulos do ar ainda restam.
    pub air_jumps_left: u32,
    /// O arranque está carregado.
    pub dash_charged: bool,
    /// Quantos segundos de reserva de parede sobram.
    pub grab_left: f32,
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
    /// O estado a guardar para o próximo tick — ver [`PlayerState`].
    pub state: PlayerState,
    /// **O que o personagem está a fazer**, para quem desenha e toca som — ver
    /// [`PlayerView`].
    ///
    /// ⚠️ **Ele sai da LEI e não é remontado pela ponte**, e a razão não é
    /// economia: metade destes campos são vereditos que só existem aqui dentro
    /// (*a perna agiu?*, *a parede foi aceite?*), e re-derivá-los lá fora seria
    /// uma segunda resposta que diverge no primeiro tique em que uma delas ganhe
    /// um caso. É também o que torna os EVENTOS baratos: a ponte compara duas
    /// vistas consecutivas em vez de reconstruir o mundo.
    pub view: PlayerView,
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
