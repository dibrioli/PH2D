//! **A FERRAMENTA DE INTERAÇÃO** (W-Hand) — o que o ponteiro faz a uma cena que
//! está rodando.
//!
//! Três ferramentas: a **MÃO** (segura um corpo — W-Grab), a **EXPLOSÃO** (um
//! estouro radial) e a **ATRAÇÃO** (um campo sustentado que puxa ou repele). O
//! motor das três mora no wrapper (`ph2d_physics::world::grab` e `::blast`, com as
//! medições); o que mora aqui é o **estado AUTORADO** delas e a tradução dos
//! números do artista para as leis que o solver toma.
//!
//! # Isto NÃO é persistido, e é decisão
//!
//! `PhysicsSettings` viaja no arquivo de projeto (`PROJECT_SCHEMA`) porque
//! gravidade e sub-passos descrevem o MUNDO. Uma ferramenta de interação descreve
//! **o ponteiro**, não a cena: abrir o projeto de outra pessoa não deve armar o
//! raio de explosão dela, e o número certo não é uma propriedade da obra. Então
//! isto é irmã do `App.show_colliders` — estado de runtime do editor — e o
//! corolário é que **nenhum schema bumpa** (um bump recusa todo projeto já salvo).
//!
//! Se um dia duas pessoas discordarem do mesmo default, ele vira preferência de
//! APP. Não vira campo de documento.
//!
//! # Uma porta por pergunta
//!
//! [`InteractionSettings::hold_spec`] é a ÚNICA coisa que sabe traduzir *rigidez
//! e razão de amortecimento* na `HoldSpec` que o solver toma, e
//! [`InteractionSettings::attract_at`] a única que monta o campo. Quem quer
//! segurar/puxar pergunta a elas — uma segunda cópia da conversão
//! `d = ratio·2·√k` é como o painel e o solver passariam a discordar sobre o que
//! o slider significa.

use ph2d_physics::{Attract, HoldSpec, PhysicsWorld};

/// **O tamanho do mundo em que se trabalha**, em metros — a régua de todo
/// alcance desta família (raio de explosão, raio de atração, folga da corda).
///
/// Não é gosto: `ProjectSettings::DEFAULT_PIXELS_PER_METER` é **100**, então uma
/// janela de 1920 px mostra **19,2 m** de mundo. Um alcance maior que isso é um
/// slider cuja metade de cima não distingue nada — todo corpo visível já está
/// dentro dele, e o falloff sobre a cena inteira é quase plano.
///
/// ⚠️ Uma régua, três consumidores. Três `MAX_*` independentes seriam três
/// respostas para *"quão grande é o mundo?"*, livres para divergir na primeira vez
/// que alguém mexesse em uma.
pub const WORLD_REACH_M: f32 = 20.0;

/// Piso da rigidez da mão. Medido: a 10 o corpo fica **1,582 m** atrás de um
/// cursor a 4 m/s — o extremo "elástico" que ainda é um seguidor.
pub const MIN_HOLD_STIFFNESS: f32 = 10.0;

/// **Teto da rigidez da mão, e o recurso é a PAREDE.**
///
/// Medido (`world::tests::sweep_the_hands_stiffness_against_a_wall`, cursor 5 m
/// dentro de uma parede estática): a 6400 a caixa para **62 mm** dentro dela, a
/// 12800 **122 mm** (meia caixa enterrada) e a 25600 ela simplesmente
/// **atravessa**. 6400 é o último valor em que uma parede ainda lê como parede;
/// quem quer ignorar geometria de propósito tem [`HoldMode::Rigid`], que a
/// atravessa por construção e diz isso no nome.
pub const MAX_HOLD_STIFFNESS: f32 = 6400.0;

/// Teto da razão de amortecimento. `1` é o crítico; medido
/// (`sweep_the_hands_damping_ratio`, k=400): `0,5` compra atraso (0,170 m em vez
/// de 0,369) ao preço de **58 mm de sobressinal**, `0` passa do cursor em 0,358 m
/// e nunca assenta, e `2` custa 0,715 m de atraso sem comprar nada. Dois é onde a
/// pista deixa de oferecer um trade.
pub const MAX_HOLD_DAMPING_RATIO: f32 = 2.0;

/// **Qual das três ferramentas o ponteiro está segurando.**
///
/// A diferença que decide a fiação inteira: a MÃO precisa de um CORPO sob o
/// cursor, as outras duas precisam só de um PONTO. Por isso
/// [`InteractionTool::needs_a_body`] é a porta única — o gesto da mão continua
/// pendurado no pick de canvas (para a seleção seguir acontecendo), e as outras
/// duas são interceptadas ANTES dele, onde não haver nada sob o cursor não
/// importa.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum InteractionTool {
    /// Segurar um corpo (W-Grab). O default: é o que um ponteiro faz.
    #[default]
    Hand,
    /// Um impulso radial, uma vez, onde o artista clicar.
    Explode,
    /// Um campo sustentado enquanto o botão estiver apertado.
    Attract,
}

impl InteractionTool {
    /// **A pergunta que separa as duas famílias.** Ver os docs do enum.
    #[must_use]
    pub fn needs_a_body(self) -> bool {
        matches!(self, Self::Hand)
    }

    /// Wire string ↔ variante. O mapeamento mora **numa** função por par, porque
    /// escrito duas vezes ele apodrece exatamente como o `BodyKind::tag` do W4
    /// documenta (o consumidor dobra todo tag desconhecido no primeiro variant e
    /// o chip passa a selecionar outra coisa).
    #[must_use]
    pub fn tag(self) -> &'static str {
        match self {
            Self::Hand => "hand",
            Self::Explode => "explode",
            Self::Attract => "attract",
        }
    }

    /// Inverso de [`Self::tag`]. `None` para um tag que não existe — o chamador
    /// decide, em vez de receber `Hand` em silêncio.
    #[must_use]
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "hand" => Some(Self::Hand),
            "explode" => Some(Self::Explode),
            "attract" => Some(Self::Attract),
            _ => None,
        }
    }

    /// A ordem em que o painel pinta os chips — a mesma lista, um índice.
    pub const ALL: [Self; 3] = [Self::Hand, Self::Explode, Self::Attract];
}

/// **Como a mão segura.** A face autorável da [`HoldSpec`] do wrapper: o enum lá
/// carrega os NÚMEROS que o solver toma, este carrega a ESCOLHA que o artista
/// fez, e [`InteractionSettings::hold_spec`] é a ponte entre os dois.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum HoldMode {
    /// Mola macia — o default, e o único modo que respeita geometria.
    #[default]
    Spring,
    /// Rígido: posição E atitude, sem atraso. Atravessa parede (medido).
    Rigid,
    /// Corda: o corpo não passa de `slack` do cursor e gira livre dentro disso.
    Rope,
}

impl HoldMode {
    #[must_use]
    pub fn tag(self) -> &'static str {
        match self {
            Self::Spring => "spring",
            Self::Rigid => "rigid",
            Self::Rope => "rope",
        }
    }

    #[must_use]
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "spring" => Some(Self::Spring),
            "rigid" => Some(Self::Rigid),
            "rope" => Some(Self::Rope),
            _ => None,
        }
    }

    pub const ALL: [Self; 3] = [Self::Spring, Self::Rigid, Self::Rope];

    /// Este modo usa os ganhos da mola? O painel pergunta para OFERECER as duas
    /// rows, e `hold_spec` para HONRÁ-las — a mesma função, duas perguntas, senão
    /// um knob pintado seria um knob que o solver ignora.
    #[must_use]
    pub fn uses_spring_gains(self) -> bool {
        matches!(self, Self::Spring)
    }

    /// Este modo usa a folga? (Só a corda.)
    #[must_use]
    pub fn uses_slack(self) -> bool {
        matches!(self, Self::Rope)
    }
}

/// O estado autorado da ferramenta de interação. Runtime-only (ver o topo do
/// módulo).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InteractionSettings {
    pub tool: InteractionTool,
    pub hold: HoldMode,
    /// Aceleração-por-metro da mola da mão (não é N/m: o modelo é
    /// `AccelerationBased`, escalado pela massa).
    pub stiffness: f32,
    /// `1` = amortecimento crítico, em QUALQUER rigidez. É por isso que é uma
    /// razão e não um segundo número solto: quem mexe na rigidez não precisa
    /// recalcular `2·√k` de cabeça ([[feedback_ergonomics_verdict_is_a_design_bug]]).
    pub damping_ratio: f32,
    /// Folga da corda, em metros. Medido: o corpo fica **exatamente** `slack`
    /// atrás do cursor.
    pub slack: f32,
    /// Alcance do estouro, metros.
    pub blast_radius: f32,
    /// Impulso do estouro NO CENTRO, `N·s`.
    pub blast_impulse: f32,
    /// Alcance do campo de atração, metros.
    pub attract_radius: f32,
    /// Força do campo NO CENTRO, `N`. **Negativa REPELE** — e isso não duplica a
    /// explosão: um empurrão contínuo segura um corpo no ar, um estalo o arremessa.
    pub attract_force: f32,
}

impl Default for InteractionSettings {
    fn default() -> Self {
        Self {
            tool: InteractionTool::Hand,
            hold: HoldMode::Spring,
            // Os dois defaults da mão vêm do WRAPPER, não transcritos: são as
            // constantes medidas do W-Grab, e o desenho aprovado no smoke é o
            // delas. Um literal aqui seria a segunda cópia livre para derivar.
            stiffness: PhysicsWorld::GRAB_STIFFNESS,
            damping_ratio: 1.0,
            slack: 0.5,
            // Estouro: raio 3 m e impulso 10 N·s dão **8,3 m/s** ao corpo de 1 kg
            // mais próximo e 1,7 m/s ao de 2,5 m — um empurrão forte que fica na
            // tela (medido, `sweep_the_blast_and_the_pull`).
            blast_radius: 3.0,
            blast_impulse: 10.0,
            // Atração: 50 N num raio de 3 m é o menor valor medido que **vence o
            // peso** de um corpo de 1 kg a 2 m do foco (ali o falloff vale 1/3, e
            // 50/3 = 16,7 N contra 9,81 N de peso). A 20 N ele cai.
            attract_radius: 3.0,
            attract_force: 50.0,
        }
    }
}

impl InteractionSettings {
    /// **Todo número numa faixa que o solver aceita.** A mesma guarda que a porta
    /// de carga dos joints tem, e pela mesma razão medida: um `NaN` em `stiffness`
    /// leva a pose para `(NaN, NaN)`, o readback a escreve no `Transform` **e no
    /// hash determinista**, e a subárvore inteira é envenenada.
    #[must_use]
    pub fn clamped(&self) -> Self {
        let f = |v: f32, lo: f32, hi: f32| if v.is_finite() { v.clamp(lo, hi) } else { lo };
        Self {
            tool: self.tool,
            hold: self.hold,
            stiffness: f(self.stiffness, MIN_HOLD_STIFFNESS, MAX_HOLD_STIFFNESS),
            damping_ratio: f(self.damping_ratio, 0.0, MAX_HOLD_DAMPING_RATIO),
            slack: f(self.slack, 0.0, WORLD_REACH_M),
            blast_radius: f(self.blast_radius, 0.0, WORLD_REACH_M),
            blast_impulse: f(self.blast_impulse, 0.0, MAX_BLAST_IMPULSE),
            attract_radius: f(self.attract_radius, 0.0, WORLD_REACH_M),
            attract_force: f(self.attract_force, -MAX_ATTRACT_FORCE, MAX_ATTRACT_FORCE),
        }
    }

    /// **A porta única que traduz os números do artista na lei do solver.**
    ///
    /// `d = ratio · 2·√k` é o amortecimento crítico escalado pela razão, e mora
    /// aqui e em lugar nenhum mais.
    #[must_use]
    pub fn hold_spec(&self) -> HoldSpec {
        let c = self.clamped();
        match c.hold {
            HoldMode::Spring => HoldSpec::Spring {
                stiffness: c.stiffness,
                damping: c.damping_ratio * 2.0 * c.stiffness.sqrt(),
            },
            HoldMode::Rigid => HoldSpec::Rigid,
            HoldMode::Rope => HoldSpec::Rope {
                max_length: c.slack,
            },
        }
    }

    /// O campo de atração centrado em `center`. A resistência é a constante
    /// MEDIDA do wrapper ([`PhysicsWorld::ATTRACT_DAMPING`]) — não é knob: sem
    /// ela o campo é um oscilador, com ela é uma ferramenta.
    #[must_use]
    pub fn attract_at(&self, center: [f32; 2]) -> Attract {
        let c = self.clamped();
        Attract {
            center,
            radius: c.attract_radius,
            force: c.attract_force,
            damping: PhysicsWorld::ATTRACT_DAMPING,
        }
    }

    /// O alcance que a ferramenta ATUAL desenha — o anel de mira do overlay.
    /// `None` para a mão, que não tem alcance (ela pega o que está sob o cursor).
    #[must_use]
    pub fn aim_radius(&self) -> Option<f32> {
        let c = self.clamped();
        match c.tool {
            InteractionTool::Hand => None,
            InteractionTool::Explode => Some(c.blast_radius),
            InteractionTool::Attract => Some(c.attract_radius),
        }
    }
}

/// **Teto do impulso do estouro, e o recurso é o DETECTOR DE COLISÃO.**
///
/// Medido (`sweep_the_blast_and_the_pull`, corpo de 1 kg): 100 N·s dão **83 m/s**
/// ao corpo mais próximo, e a varredura do W-CCD mediu que a detecção discreta
/// começa a **tunelar** entre 100 e 600 m/s. Acima daqui um estouro mais forte não
/// espalha a cena com mais violência — ele **apaga objetos através de paredes**.
pub const MAX_BLAST_IMPULSE: f32 = 100.0;

/// Teto da força de atração. Medido: 200 N sobre 1 kg no meio do alcance são
/// ~100 m/s² (10 g), e o corpo cruza um alcance de 3 m em ~0,25 s — além disso o
/// campo deixa de puxar e passa a teleportar.
pub const MAX_ATTRACT_FORCE: f32 = 200.0;
