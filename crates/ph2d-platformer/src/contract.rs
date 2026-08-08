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
    dash::DashState, jump::JumpConfig, jump::JumpState, kinematic, react::ReactionConfig,
    ride::RideConfig, walk::WalkConfig, wall::GrabState, wall::WallConfig,
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
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PlayerState {
    /// O pulo, o corte, os dois relógios do perdão e a memória do chão.
    pub jump: JumpState,
    /// O arranque (W14) — o relógio, a carga, a direção e o lado que ele olha.
    pub dash: DashState,
    /// O agachar (W15) — um bit, e ele existe porque levantar-se pode ser
    /// RECUSADO (ver o topo de [`crate::crouch`]).
    pub crouch: CrouchState,
    /// O agarrar-se (W23) — quanto já se gastou da reserva de parede.
    pub grab: GrabState,
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
