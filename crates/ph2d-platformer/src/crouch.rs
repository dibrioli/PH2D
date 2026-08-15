//! **O AGACHAR** (W15) — e ele é uma **perna mais curta**, não um corpo menor.
//!
//! # ⚠️ A previsão deste módulo estava escrita, e a construção corrigiu-a
//!
//! O [`06_plano_player_plataforma.md`](../../../docs/Physics/06_plano_player_plataforma.md)
//! §4 mantinha o agachar de fora com um preço nomeado: *"o que ele exige é
//! **encolher o collider**, que é a primeira coisa desta linha a reescrever a
//! forma de um corpo por tique … e a resposta para um corpo composto (encolher
//! qual? por quanto?) é decisão de desenho, não mecânica"*.
//!
//! **Ele não exige nada disso.** O personagem é uma **cápsula flutuante**: a
//! silhueta dele acima do chão vale `float_height + meia-altura do collider`, e
//! baixar a `float_height` baixa a silhueta inteira **pelo mesmo delta**, com o
//! collider intocado. É a D1 do plano a apagar mais um caso especial — a mesma
//! frase que já apagara degrau, rampa e plataforma móvel —, e é por isso que a
//! premissa que a W-Compound derrubou (*"um corpo tem exactamente um collider"*)
//! **nunca é tocada aqui**: esta wave não pergunta quantas formas o corpo tem.
//!
//! É também o que o `bevy-tnua` faz (o `TnuaBuiltinCrouch` desce a altura de
//! flutuação), e pela mesma razão: quem paira não tem forma a encolher.
//!
//! # ⚠️ A faixa de agarre CRESCE pelo que a perna encurtou, e isso não é ajuste
//!
//! A [`RideConfig::cling_distance`] é o quanto ACIMA do repouso a mola ainda
//! age. Se ela não crescer, o personagem que agacha sai do alcance da própria
//! mola no instante do gesto — ele está a `float_height` e a faixa nova acabaria
//! em `crouch + cling` — e **cai em queda livre** até ela o reencontrar. ⚠️ E o
//! que se perde não é sobretudo essa queda (ela mede `drop − cling`, e some se a
//! faixa já for larga): é o **DESACORDO** — a ponte casta e julga com a config
//! autorada, então ela veria chão onde a lei já não vê.
//!
//! Crescendo-a pelo `drop`, a soma `float_height + cling_distance` — que é
//! **exactamente** o que [`crate::within_reach`] compara — fica **INVARIANTE**.
//! E essa invariância paga uma dívida que nem chegou a existir: o alcance do
//! sensor de chão e o veredito *"isto é chão?"* não mudam com o agachar, então a
//! ponte pode continuar a castar e a julgar com a config **autorada**, sem
//! ordenar coisa nenhuma. É a representação a apagar o caso especial, e está
//! gateado nos dois lados
//! ([`tests::the_crouch_does_not_move_what_the_sensor_reaches`]).
//!
//! # ⚠️ O estado NÃO é uma função pura do botão
//!
//! Levantar-se pode ser **RECUSADO**: sob um teto baixo não há para onde subir.
//! É por isso que existe um [`CrouchState`] — soltar o botão debaixo de uma
//! marquise deixa o personagem agachado, e é assim que todo platformer o faz. A
//! alternativa (levantar sempre) empurraria o corpo para dentro da pedra e
//! deixaria o solver a resolver uma penetração que o artista nunca pediu.

use crate::{PlayerConfig, RideConfig, WalkConfig};

/// **Como o personagem agacha.**
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CrouchConfig {
    /// A altura de flutuação enquanto agachado, em metros.
    ///
    /// ⚠️ **ZERO desliga a capacidade** — e um valor que não seja MENOR que a
    /// [`RideConfig::float_height`] também: um "agachar" que sobe não é um
    /// agachar, e deixá-lo passar encolheria a faixa de agarre em vez de a
    /// crescer (ver o aviso do módulo).
    pub height: f32,
    /// A velocidade de cruzeiro agachado, m/s.
    ///
    /// ⚠️ **Zero aqui é LEGÍTIMO**, e é a diferença entre os dois números: quem
    /// põe `0` está a autorar um agachar em que não se anda (o *duck* parado),
    /// que é uma escolha. Quem quer a capacidade desligada põe zero na
    /// [`Self::height`].
    pub speed: f32,
    /// **Ele pode andar para fora de um patamar AGACHADO?** (`W-Brink` — o
    /// `bCanWalkOffLedgesWhenCrouching` do Unreal.)
    ///
    /// ⚠️ **Existe porque a resposta de pé e a resposta agachado são
    /// genuinamente diferentes**, e o caso canónico é o único que o Unreal cita:
    /// aproximar-se de uma beirada **a espreitar**, sem cair dela, enquanto de pé
    /// se anda normalmente. Com um knob só, esse pedido é inexprimível.
    ///
    /// ⚠️ **`true` é o mundo que já shipava** — a nossa política, e o oposto do
    /// default do Unreal: lá ele nasce `false`, o que aqui faria todo personagem
    /// já autorado ganhar uma trava ao agachar-se por ter sido aberto num build
    /// mais recente.
    ///
    /// ⚠️ **Ele só é lido enquanto AGACHADO e com o agachar ARMADO** — a porta é
    /// a [`walk_for`], que já existe exactamente para *"o dia em que um terceiro
    /// termo tiver de encolher"*.
    pub walk_off_ledges: bool,
}

impl CrouchConfig {
    /// Um perfil de partida — ⚠️ **DESLIGADO**, a mesma política do arranque e
    /// das paredes: um player já autorado não ganha um gesto novo por ter sido
    /// aberto num build mais recente.
    pub const STARTING_POINT: Self = Self {
        height: 0.0,
        speed: 2.0,
        // ⚠️ **`true` = o mundo que já shipava** (ver o doc do campo), e não o
        // `false` do Unreal.
        walk_off_ledges: true,
    };

    /// **A capacidade está autorada?**
    ///
    /// ⚠️ Toma a perna porque a pergunta é comparativa: agachar é ficar mais
    /// baixo, e "mais baixo" só existe contra a altura de pé.
    #[must_use]
    pub fn armed(&self, ride: &RideConfig) -> bool {
        self.height.is_finite() && self.height > 0.0 && self.height < ride.float_height
    }

    /// **Quanto o corpo SOBE ao levantar-se** — e portanto a folga que o teto
    /// tem de ter para o gesto ser reversível.
    ///
    /// Zero quando a capacidade não está armada.
    #[must_use]
    pub fn rise(&self, ride: &RideConfig) -> f32 {
        if self.armed(ride) {
            ride.float_height - self.height
        } else {
            0.0
        }
    }
}

/// **O que há sobre a cabeça** — o sensor que decide se ele pode levantar-se.
///
/// # ⚠️ Era uma GRADE DE RAIOS, e passou a ser uma VARREDURA DO CORPO
///
/// Até a `W-ShapeCast` isto carregava `[bool; 3]` — o teto visto de três alturas
/// —, e a ponte reduzia com um *any*. A lei só alguma vez perguntou
/// [`Headroom::is_blocked`], então a grade nunca foi informação que alguém
/// usasse: era o **preço de o wrapper só saber lançar linhas**.
///
/// Com `PhysicsWorld::sweep_body` a pergunta é feita uma vez, com a forma real
/// do corpo, e **duas limitações escritas morrem juntas**:
///
/// - *"um teto mais estreito que meia largura, entre duas amostras, não é
///   visto"* — medido, um pilar de 8 cm punha a cabeça **1,267** contra pedra em
///   **1,25**;
/// - *"a caixa envolvente é conservadora … um teto que toque só a quina da caixa
///   recusa o levantar"* — a varredura usa a **cápsula**, então a quina deixa de
///   ser um obstáculo que não existe.
///
/// ⚠️ **E a afirmação sobre a direcção do erro era falsa de um dos dois lados:**
/// o doc antigo dizia que *"o erro possível é ficar agachado onde caberia, nunca
/// levantar-se para dentro da pedra"*. Verdade sobre a caixa, falsa sobre o vão
/// entre dois raios — e a segunda metade é a perigosa. Fica registada aqui em
/// vez de apagada, porque é o motivo de esta wave existir (CLAUDE.md §0).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Headroom {
    /// Há teto ao alcance de `rise` — a resposta da varredura, já reduzida
    /// porque **não há nada a reduzir**: uma varredura devolve o primeiro
    /// contacto do corpo INTEIRO.
    pub blocked: bool,
}

impl Headroom {
    /// Céu limpo — o neutro, e o que uma cena sem teto produz.
    pub const CLEAR: Self = Self { blocked: false };

    /// **Há alguma coisa no caminho?**
    #[must_use]
    pub fn is_blocked(&self) -> bool {
        self.blocked
    }
}

/// **Vale a pena perguntar ao teto?**
///
/// ⚠️ Porta única, e o motivo é o mesmo dos outros dois sensores desta lei
/// ([`crate::corner_probe_wanted`], [`crate::wall_probe_wanted`]): a ponte a
/// pergunta para decidir se casta, a lei a pergunta para decidir se lê. A
/// resposta é `false` em quase todo tique — só quem **está agachado** e **soltou
/// o botão** tem alguma coisa a perguntar.
#[must_use]
pub fn headroom_probe_wanted(
    cfg: &CrouchConfig,
    ride: &RideConfig,
    state: CrouchState,
    held: bool,
) -> bool {
    cfg.armed(ride) && state.crouched && !held
}

/// **O que o agachar carrega entre tiques** — um bit, e ele existe porque
/// levantar-se pode ser recusado (ver o aviso do módulo).
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct CrouchState {
    /// Ele está agachado.
    pub crouched: bool,
}

/// **A lei do agachar neste tique.**
///
/// - `grounded`: o pé está no chão — ⚠️ a MESMA resposta que o pulo e o arranque
///   consomem ([`crate::JumpState::on_ground`]), nunca uma segunda.
/// - `held`: o botão de BAIXO está pressionado (o mesmo da W12 — o botão já
///   existia com o significado certo, e é por isso que esta wave não acrescenta
///   nenhum).
/// - `headroom`: o que o sensor viu, ou `None` quando ninguém perguntou.
///
/// # ⚠️ Por que o agachar exige o CHÃO
///
/// A perna só existe no chão (no ar a mola cala-se), então um agachar aéreo
/// mudaria uma coisa só — a velocidade de cruzeiro do controlo aéreo —, que não
/// é o que a palavra significa. E o botão já tem, no ar, um significado que a
/// W12 lhe deu (`down + jump` = descer através da plataforma); dar-lhe um
/// segundo ali seria pôr dois donos no mesmo dedo.
#[must_use]
pub fn crouch_step(
    cfg: &CrouchConfig,
    ride: &RideConfig,
    state: CrouchState,
    grounded: bool,
    held: bool,
    headroom: Option<&Headroom>,
) -> CrouchState {
    if !cfg.armed(ride) {
        return CrouchState::default();
    }
    let want = held && grounded;
    // ⚠️ **Sem sensor, levanta-se.** A ausência de resposta é a ausência de
    // teto: quem não perguntou é porque não estava agachado, ou porque ainda
    // segura o botão — nos dois casos a recusa não teria o que recusar.
    let stuck = state.crouched && !want && headroom.is_some_and(Headroom::is_blocked);
    CrouchState {
        crouched: want || stuck,
    }
}

/// **A perna deste tique** — encurtada, e com a faixa de agarre crescida pelo
/// que ela encurtou (ver o aviso do módulo).
#[must_use]
pub fn ride_for(cfg: &CrouchConfig, ride: &RideConfig, crouched: bool) -> RideConfig {
    if !crouched || !cfg.armed(ride) {
        return *ride;
    }
    let drop = ride.float_height - cfg.height;
    RideConfig {
        float_height: cfg.height,
        cling_distance: ride.cling_distance + drop,
        ..*ride
    }
}

/// **A caminhada deste tique** — a velocidade de cruzeiro e a trava de beirada.
///
/// ⚠️ A aceleração fica: ela é o quão depressa o personagem RESPONDE, e agachar
/// não o torna menos responsivo — torna-o mais lento. São duas grandezas, e
/// mexer nas duas com um knob só seria escolher por quem autora.
///
/// ⚠️ **O segundo termo (`W-Brink`) entrou aqui, e o doc abaixo já o previa:**
/// a [`effective_crouched`] existe para que *"o dia em que um terceiro termo
/// tiver de encolher, ele encolhe aqui e não em três sítios"*. A trava agachado
/// é o segundo, e não custou um sítio novo.
///
/// ⚠️ **Ele só APERTA, nunca solta:** `walk_off_ledges` agachado é lido com um
/// `&&`, então autorar `true` ali não devolve a liberdade a quem já a perdeu de
/// pé. Um `=` faria o agachar **ligar** a caminhada para fora do patamar num
/// personagem cujo autor a proibiu, que é a trava a fazer o contrário do nome.
#[must_use]
pub fn walk_for(
    cfg: &CrouchConfig,
    ride: &RideConfig,
    walk: &WalkConfig,
    crouched: bool,
) -> WalkConfig {
    if !crouched || !cfg.armed(ride) {
        return *walk;
    }
    WalkConfig {
        speed: cfg.speed.max(0.0),
        walk_off_ledges: walk.walk_off_ledges && cfg.walk_off_ledges,
        ..*walk
    }
}

/// **A config EFECTIVA deste tique** — a porta única do *"o que agachar muda"*.
///
/// ⚠️ Ela existe para haver **uma** resposta: a mola, a caminhada e todo consumidor
/// futuro leem a mesma `PlayerConfig`, em vez de cada um lembrar-se de perguntar
/// ao agachar. O dia em que um terceiro termo tiver de encolher, ele encolhe
/// aqui e não em três sítios.
#[must_use]
pub fn effective_crouched(cfg: &PlayerConfig, crouched: bool) -> PlayerConfig {
    if !crouched {
        return *cfg;
    }
    PlayerConfig {
        ride: ride_for(&cfg.crouch, &cfg.ride, true),
        walk: walk_for(&cfg.crouch, &cfg.ride, &cfg.walk, true),
        ..*cfg
    }
}

#[cfg(test)]
#[path = "crouch_tests.rs"]
mod tests;
