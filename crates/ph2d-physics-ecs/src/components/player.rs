//! **O componente do PLAYER DE PLATAFORMA** (W2) — a config autorada.
//!
//! Módulo irmão dos outros de `components/` pelo motivo de sempre (LOC e
//! isolamento), e a fronteira dele é a regra mais importante desta wave:
//!
//! # ⚠️ CONFIG, nunca estado vivo
//!
//! Um controlador de plataforma carrega estado por-tick — `is_grounded`, o
//! coyote timer, o jump buffer. **Nada disso mora aqui.** O `canonicalize` do
//! undo ordena as entidades pelos BYTES dos componentes, então um campo que
//! muda por tick faria **cada frame virar um passo de undo** (o ADR-0131 escreve
//! essa lei, e o `PhysicsJoint` já a honra).
//!
//! O estado vivo mora na PONTE, ao lado do `grab` — e a partir da W7 ele deixa
//! de ser guardado e passa a ser **derivado da fita de entrada**, que é o que o
//! torna reproduzível num scrub.

use bevy_ecs::component::Component;
use ph2d_platformer::{
    CrouchConfig, DashConfig, GlideConfig, JumpConfig, LedgeConfig, PlayerConfig, ReactionConfig,
    RideConfig, SwimConfig, WalkConfig, WallConfig,
};
use serde::{Deserialize, Serialize};

/// **Este corpo é um player de plataforma.**
///
/// Presença = o comportamento existe; os campos são os ganhos da perna
/// ([`RideConfig`], a lei pura). Ausente é o mundo de antes desta wave, byte a
/// byte — nenhum corpo sem o componente muda de trajetória.
///
/// ⚠️ **Só faz sentido num corpo `Dynamic`**, e por FÍSICA, não por gosto: a
/// mola é um impulso, e um impulso não move um corpo estático nem um kinematic
/// (massa infinita — o fato que a W-BakeJoint mediu e que faz a MÃO recusar os
/// dois). A row do Inspector será oferecida para Dynamic apenas, e a ponte
/// recusa em silêncio o resto.
///
/// ⚠️ **Componente NOVO ⇒ blob-key própria ⇒ `PROJECT_SCHEMA` NÃO bumpa** (o
/// precedente do `PhysicsJoint`/W3): um arquivo antigo simplesmente não o tem, e
/// o load o deixa ausente. O que bumpa é apendar campo a um componente que já
/// existe, porque o postcard é posicional.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlatformPlayer {
    /// A que altura o personagem paira, medida do CENTRO do corpo para baixo.
    pub float_height: f32,
    /// Quanto acima da altura de repouso a mola ainda age (ver [`RideConfig`]).
    pub cling_distance: f32,
    /// Rigidez da perna, em aceleração-por-metro.
    pub spring_strength: f32,
    /// Amortecimento, em fração da velocidade relativa removida por tick.
    ///
    /// ⚠️ Tem TETO medido ([`RideConfig::MAX_DAMPING`]) — acima dele o boost
    /// inverte a velocidade em vez de matá-la, e o personagem pipoca.
    pub spring_damping: f32,

    /// Velocidade de cruzeiro, m/s — **relativa ao chão** (W3).
    pub speed: f32,
    /// Aceleração no chão, m/s².
    pub acceleration: f32,
    /// Aceleração no ar — o controle aéreo. `0` conserva o arco do salto.
    pub air_acceleration: f32,
    /// A inclinação máxima em que o personagem fica de pé, em GRAUS.
    ///
    /// ⚠️ Graus na fronteira, cosseno no motor — a mesma política do ângulo de
    /// joint (graus na UI, radianos no componente): o artista autora o número
    /// que ele entende, e a conversão acontece **uma vez**, na porta única
    /// [`WalkConfig::max_slope_cos`].
    pub max_slope_deg: f32,

    /// A altura de um pulo COMPLETO, metros — **com gravidade neutra** (W4).
    pub jump_height: f32,

    /// **Quantos pulos DEPOIS de sair do chão** (`W-MultiJump`) — `0` desliga, e
    /// desligado é byte-idêntico ao mundo anterior. Recarrega no chão.
    pub air_jumps: u32,
    /// **A altura de um pulo do AR**, metros — a mesma régua do
    /// [`Self::jump_height`], e um fato AUTORADO separado dele.
    pub air_jump_height: f32,
    /// Multiplicador de gravidade na saída, acima de [`Self::takeoff_speed`].
    pub takeoff_gravity: f32,
    /// A velocidade acima da qual a gravidade de saída age, m/s.
    pub takeoff_speed: f32,
    /// Multiplicador perto do ápice — ⚠️ **abaixo de 1 ALONGA** (a decisão do
    /// módulo: o *forgiveness* do Celeste, não o *snappiness* do tnua).
    pub peak_gravity: f32,
    /// A janela do ápice, m/s.
    pub peak_speed: f32,
    /// Multiplicador na queda — acima de 1 desce mais rápido do que sobe.
    pub fall_gravity: f32,
    /// Multiplicador enquanto sobe com o botão SOLTO — a altura variável.
    pub cut_gravity: f32,

    /// **COYOTE TIME** (W8) — segundos de perdão depois de sair do chão.
    pub coyote_time: f32,
    /// **JUMP BUFFER** (W8) — segundos que um aperto cedo demais sobrevive.
    pub jump_buffer: f32,
    /// **CORNER CORRECTION** (W10) — metros de deslocamento lateral que a
    /// assistência pode dar para o personagem passar por baixo de uma beirada.
    ///
    /// ⚠️ Uma DISTÂNCIA, não um tempo: os dois irmãos acima perdoam erros de
    /// *quando*, este perdoa um erro de *onde*. `0` desliga — e desliga também o
    /// sensor, então não custa um raio sequer.
    /// **Quantas amostras o perfil da quina varre** — ímpar, teto
    /// [`ph2d_platformer::MAX_CORNER_SAMPLES`].
    /// **Quantos raios a perna casta** — ímpar. Um raio só afundava 46% do
    /// `float_height` numa fenda de 10 cm (`W-Probes2`).
    pub foot_samples: u16,

    /// **Onde os raios de fora da perna se sentam**, fração da meia-largura.
    /// `0` junta tudo no centro e reproduz o raio único.
    pub foot_spread: f32,

    pub corner_samples: u16,

    /// **Quantos tiques de antecedência o perfil da quina olha.**
    pub corner_lookahead: f32,

    /// **Quantos raios o flanco casta** — ímpar, teto
    /// [`ph2d_platformer::MAX_WALL_SAMPLES`].
    pub wall_samples: u16,

    /// **Onde as amostras de fora do flanco se sentam**, fração da meia-altura.
    pub wall_spread: f32,

    pub corner_reach: f32,
    /// **LIFT MOMENTUM** (W10) — segundos em que o controle aéreo continua
    /// medindo no referencial do chão que se deixou.
    ///
    /// ⚠️ Sem ele, sair de uma plataforma móvel faz o controle aéreo FREAR a
    /// velocidade que ela deu. Em chão estático é inerte (a memória é `[0, 0]`).
    pub lift_momentum: f32,

    /// **Quanto do PESO volta para o chão** (W6). `1` transmite inteiro.
    ///
    /// ⚠️ Em `0` o personagem é um FANTASMA: ele paira sobre uma jangada sem a
    /// afundar e uma balança não o pesa. O default é `1` porque a física é essa;
    /// o número existe para desligar, não para ligar.
    pub reaction_support: f32,
    /// **Quanto da CAMINHADA volta para o chão** (W6). Nasce em `0`.
    ///
    /// ⚠️ O default oposto ao irmão é de PRODUTO, não de física: com ele em `1`
    /// a plataforma escorrega para trás como um tapete quando o personagem anda
    /// em cima dela — atrito honesto e péssimo de jogar.
    pub reaction_movement: f32,
    /// **Quanto de um bloqueio LATERAL volta** (W-KinPush). Nasce em `1`.
    ///
    /// ⚠️ **Só o modo CINEMÁTICO o lê**, e não é uma limitação: sob Spring o
    /// personagem é um corpo dinâmico e o solver já empurra o que ele esbarra —
    /// medido, o dinâmico leva um caixote **16,55 m em 3 s** e o cinemático
    /// **0,0000**. Este número existe para dar ao segundo modo o que o primeiro
    /// tem de graça, e a §14 só o oferece a quem o lê.
    pub reaction_push: f32,

    /// **WALL SLIDE** (W13) — a velocidade com que se desce uma parede agarrada,
    /// m/s. `0` desliga.
    ///
    /// ⚠️ **A velocidade, não um teto:** com o atrito default o personagem
    /// pressionado contra uma parede **não cai**, e um teto nunca dispararia —
    /// a medição está em [`ph2d_platformer::wall_slide`].
    ///
    /// ⚠️ **Nasce DESLIGADO**, ao contrário de quase tudo aqui: parede é uma
    /// CAPACIDADE do personagem, não uma correção de física. Um platformer sem
    /// paredes é um gênero inteiro, e ligá-la por default mudaria o
    /// comportamento de todo player já autorado.
    pub wall_slide_speed: f32,
    /// **WALL JUMP** (W13) — a altura de um pulo de parede, metros. `0` desliga.
    pub wall_jump_height: f32,
    /// **O empurrão para LONGE da parede**, m/s, no instante do pulo.
    ///
    /// ⚠️ Uma velocidade e não uma altura: o que ela decide é *quão longe da
    /// parede ele chega*, e a métrica que o artista julga é a distância entre
    /// duas paredes que dá para subir em ziguezague.
    pub wall_jump_push: f32,
    /// **Quanto ALÉM da própria largura o sensor lateral procura**, metros.
    ///
    /// ⚠️ Não é zero de propósito — ver [`ph2d_platformer::WallConfig::reach`]:
    /// um alcance de exactamente meia-largura faria o agarrar-se piscar.
    pub wall_reach: f32,
    /// **Por quantos segundos o controle aéreo fica calado depois de um pulo de
    /// parede**, s. `0` desliga.
    ///
    /// ⚠️ Sem ele o pulo de parede entrega **76% da altura autorada e 0,44 m de
    /// afastamento**, medido — o jogador ainda segura a direção da parede e o
    /// controle aéreo puxa-o de volta para ela. Ver
    /// [`ph2d_platformer::WallConfig::jump_lockout`].
    pub wall_jump_lockout: f32,
    /// **WALL GRAB** (W23) — por quantos SEGUNDOS ele segura a parede de vez,
    /// com o botão de agarrar apertado. `0` desliga (a capacidade não existe).
    ///
    /// ⚠️ **Nasce DESLIGADO**, como as outras metades da parede. E o zero não é
    /// um caso especial: segurar por zero segundos **é** não ter agarrar-se.
    ///
    /// ⚠️ A reserva volta ao cheio no CHÃO, de uma vez — ver
    /// [`ph2d_platformer::grab_step`], que também traz o porquê de ser uma
    /// RESERVA e não um interruptor.
    pub wall_grab_stamina: f32,

    /// **DASH** (W14) — a velocidade do arranque, m/s. `0` desliga.
    ///
    /// ⚠️ **Nasce DESLIGADO**, como as paredes e pela mesma razão: um arranque é
    /// uma CAPACIDADE do personagem, não uma correção de física, e ligá-lo por
    /// default mudaria o comportamento de todo player já autorado.
    pub dash_speed: f32,
    /// **Quanto tempo o arranque dura**, segundos.
    ///
    /// ⚠️ É a companheira da velocidade e é o par que decide a **DISTÂNCIA** —
    /// que é o número que o artista de facto julga (*"atravessa aquele
    /// buraco?"*). Ver a tabela do `measure_dash`.
    pub dash_time: f32,
    /// **Quanto tempo depois do FIM até poder arrancar de novo**, segundos.
    ///
    /// ⚠️ Ela **não** é o que impede voar — quem faz isso é a carga, reposta
    /// pelo pé no chão ([`ph2d_platformer::DashState::charged`]). Esta existe
    /// para não se encadearem arranques no chão mais depressa do que a animação
    /// consegue mostrar.
    pub dash_cooldown: f32,

    /// **AGACHAR** (W15) — a altura de flutuação enquanto agachado, m. `0`
    /// desliga.
    ///
    /// ⚠️ **O agachar é uma perna mais CURTA, não um corpo menor** — o collider
    /// nunca é reescrito, e é por isso que esta wave não pergunta quantas formas
    /// o corpo tem (a premissa que a W-Compound derrubou). Ver o topo de
    /// [`ph2d_platformer::crouch`].
    ///
    /// ⚠️ **Nasce DESLIGADO**, como o arranque e as paredes, e um valor que não
    /// seja MENOR que a [`Self::float_height`] também conta como desligado: um
    /// "agachar" que sobe não é um agachar.
    pub crouch_height: f32,
    /// **A velocidade de cruzeiro agachado**, m/s.
    ///
    /// ⚠️ **Zero aqui é LEGÍTIMO** e significa *agachar sem andar* — quem quer a
    /// capacidade desligada põe zero na [`Self::crouch_height`]. São dois
    /// números com dois significados, de propósito.
    pub crouch_speed: f32,

    /// **NADAR** (W-Swim) — a velocidade de nado, m/s. `0` desliga.
    ///
    /// ⚠️ **Nasce DESLIGADO**, como o arranque, o agachar e as paredes, e pela
    /// mesma razão: nadar é uma CAPACIDADE do personagem, não uma correção de
    /// física. Ligá-la por default mudaria o comportamento de todo player já
    /// autorado que tenha uma poça na cena.
    pub swim_speed: f32,
    /// **Quão depressa se chega à velocidade de nado**, m/s².
    ///
    /// ⚠️ **É AUTORIDADE contra a água, não só resposta:** o empuxo empurra
    /// enquanto este servo corrige, então um número pequeno deixa o corpo boiar
    /// para a tona sozinho quando ninguém dirige, e um número grande deixa o
    /// nadador tredar água parado onde quiser.
    pub swim_acceleration: f32,
    /// **Quantos PESOS o fluido tem de carregar para ele começar a nadar.**
    ///
    /// ⚠️ **Não é uma altura, e não podia ser:** a mesma altura significa coisas
    /// diferentes em cada poça. O default `1.0` é uma frase de física — *a água
    /// sozinha me sustenta* —, que é por construção a linha em que ele boiaria
    /// parado, seja qual for a densidade dela ou a dele. A tabela que converte
    /// este número em imersão está em [`ph2d_platformer::swim`].
    ///
    /// ⚠️ **Só a ENTRADA usa este número.** Sair é uma trava (chão, ou estar
    /// completamente fora da água), porque um limiar só faria o nadador oscilar
    /// em torno dele exatamente onde o jogador tenta emergir.
    pub swim_enter: f32,

    /// **O ALCANCE do braço numa beirada**, metros — o **X** (`W-Ledge`). `0.0`
    /// **desliga** — ver [`LedgeConfig::grab`].
    pub ledge_grab: f32,
    /// **A ALTURA da janela**, metros — o **Y** (`W-LedgeSensor`); ver
    /// [`LedgeConfig::reach_y`].
    pub ledge_reach_y: f32,
    /// **A EXTENSÃO do sensor**, metros — o *scale* (`W-LedgeSensor`). `0.0` é o
    /// raio único de antes da wave, **ao bit**; ver [`LedgeConfig::span`].
    pub ledge_span: f32,
    /// **O DESLOCAMENTO vertical do sensor**, metros (`W-LedgeSensor`). `0` e' a
    /// janela centrada no topo do corpo, o mundo de antes ao bit; ver
    /// [`LedgeConfig::offset_y`].
    pub ledge_offset_y: f32,
    /// **A velocidade com que ele se acomoda e sobe**, m/s — ver
    /// [`LedgeConfig::speed`].
    pub ledge_speed: f32,

    /// **O TETO da descida enquanto se plana**, m/s (`W-Glide`). `0.0`
    /// **desliga** — ver [`GlideConfig::fall_speed`].
    ///
    /// ⚠️ **Um teto, e não a velocidade do planeio:** ele só age sobre quem já
    /// cai mais depressa, então segurar o botão a subir ou no ápice não faz
    /// nada. A medição que descartou as outras duas formas está no topo do
    /// [`ph2d_platformer::glide`].
    pub glide_fall_speed: f32,

    /// **Quanto do orçamento de aceleração ele gasta a FREAR** (`W-Brake`) — a
    /// fração usada no chão com o eixo SOLTO; ver [`WalkConfig::brake_scale`].
    ///
    /// ⚠️ **Mora no FIM do struct e não ao lado da `acceleration`**, que é onde
    /// ele se lê: o postcard é POSICIONAL, e apendar é a política dos cinco
    /// degraus anteriores deste componente. A vizinhança semântica está na UI,
    /// onde a row nasce dentro do card ANDAR.
    ///
    /// ⚠️ **`1` é o mundo de antes desta wave, AO BIT** — e é por isso que o
    /// degrau de schema é o único preço dela: nenhum projeto reabre diferente.
    pub brake_scale: f32,
}

impl PlatformPlayer {
    /// A config da lei que este componente descreve.
    ///
    /// Existe para que a ponte **não** remonte o `PlayerConfig` campo a campo:
    /// duas cópias da mesma tradução divergem no dia em que um campo novo entra
    /// só numa delas.
    #[must_use]
    pub fn config(&self) -> PlayerConfig {
        PlayerConfig {
            ride: RideConfig {
                float_height: self.float_height,
                cling_distance: self.cling_distance,
                spring_strength: self.spring_strength,
                spring_damping: self.spring_damping,
                samples: self.foot_samples as usize,
                spread: self.foot_spread,
            },
            walk: WalkConfig {
                speed: self.speed,
                acceleration: self.acceleration,
                air_acceleration: self.air_acceleration,
                brake_scale: self.brake_scale,
                max_slope_deg: self.max_slope_deg,
            },
            jump: JumpConfig {
                jump_height: self.jump_height,
                air_jumps: self.air_jumps,
                air_jump_height: self.air_jump_height,
                takeoff_gravity: self.takeoff_gravity,
                takeoff_speed: self.takeoff_speed,
                peak_gravity: self.peak_gravity,
                peak_speed: self.peak_speed,
                fall_gravity: self.fall_gravity,
                cut_gravity: self.cut_gravity,
                coyote_time: self.coyote_time,
                jump_buffer: self.jump_buffer,
                corner_reach: self.corner_reach,
                corner_samples: self.corner_samples as usize,
                corner_lookahead: self.corner_lookahead,
                lift_momentum: self.lift_momentum,
            },
            wall: WallConfig {
                slide_speed: self.wall_slide_speed,
                jump_height: self.wall_jump_height,
                jump_push: self.wall_jump_push,
                reach: self.wall_reach,
                samples: self.wall_samples as usize,
                spread: self.wall_spread,
                jump_lockout: self.wall_jump_lockout,
                grab_stamina: self.wall_grab_stamina,
            },
            dash: DashConfig {
                speed: self.dash_speed,
                time: self.dash_time,
                cooldown: self.dash_cooldown,
            },
            crouch: CrouchConfig {
                height: self.crouch_height,
                speed: self.crouch_speed,
            },
            swim: SwimConfig {
                speed: self.swim_speed,
                acceleration: self.swim_acceleration,
                enter: self.swim_enter,
            },
            ledge: LedgeConfig {
                grab: self.ledge_grab,
                reach_y: self.ledge_reach_y,
                span: self.ledge_span,
                offset_y: self.ledge_offset_y,
                speed: self.ledge_speed,
            },
            glide: GlideConfig {
                fall_speed: self.glide_fall_speed,
            },
            react: ReactionConfig {
                support: self.reaction_support,
                movement: self.reaction_movement,
                push: self.reaction_push,
            },
        }
    }
}

impl Default for PlatformPlayer {
    /// ⚠️ **Ponto de partida, não default de produto** — os números que shipam
    /// saem da varredura da wave, com a tabela ao lado (CLAUDE.md §0).
    fn default() -> Self {
        let c = PlayerConfig::STARTING_POINT;
        Self {
            float_height: c.ride.float_height,
            cling_distance: c.ride.cling_distance,
            spring_strength: c.ride.spring_strength,
            spring_damping: c.ride.spring_damping,
            foot_samples: u16::try_from(c.ride.samples).unwrap_or(u16::MAX),
            foot_spread: c.ride.spread,
            speed: c.walk.speed,
            acceleration: c.walk.acceleration,
            air_acceleration: c.walk.air_acceleration,
            brake_scale: c.walk.brake_scale,
            max_slope_deg: c.walk.max_slope_deg,
            jump_height: c.jump.jump_height,
            takeoff_gravity: c.jump.takeoff_gravity,
            takeoff_speed: c.jump.takeoff_speed,
            peak_gravity: c.jump.peak_gravity,
            air_jumps: c.jump.air_jumps,
            air_jump_height: c.jump.air_jump_height,
            peak_speed: c.jump.peak_speed,
            fall_gravity: c.jump.fall_gravity,
            cut_gravity: c.jump.cut_gravity,
            coyote_time: c.jump.coyote_time,
            jump_buffer: c.jump.jump_buffer,
            corner_reach: c.jump.corner_reach,
            corner_samples: u16::try_from(c.jump.corner_samples).unwrap_or(u16::MAX),
            corner_lookahead: c.jump.corner_lookahead,
            lift_momentum: c.jump.lift_momentum,
            reaction_support: c.react.support,
            reaction_movement: c.react.movement,
            reaction_push: c.react.push,
            wall_slide_speed: c.wall.slide_speed,
            wall_jump_height: c.wall.jump_height,
            wall_jump_push: c.wall.jump_push,
            wall_reach: c.wall.reach,
            wall_samples: u16::try_from(c.wall.samples).unwrap_or(u16::MAX),
            wall_spread: c.wall.spread,
            wall_jump_lockout: c.wall.jump_lockout,
            wall_grab_stamina: c.wall.grab_stamina,
            dash_speed: c.dash.speed,
            dash_time: c.dash.time,
            dash_cooldown: c.dash.cooldown,
            crouch_height: c.crouch.height,
            crouch_speed: c.crouch.speed,
            swim_speed: c.swim.speed,
            swim_acceleration: c.swim.acceleration,
            swim_enter: c.swim.enter,
            ledge_grab: c.ledge.grab,
            ledge_reach_y: c.ledge.reach_y,
            ledge_span: c.ledge.span,
            ledge_offset_y: c.ledge.offset_y,
            ledge_speed: c.ledge.speed,
            glide_fall_speed: c.glide.fall_speed,
        }
    }
}

/// **Como este player é MOVIDO** (W-KinMove, K2 do plano 07) — o segundo modo.
///
/// # ⚠️ Componente VALUADO, e ausente = [`PlayerMode::Dynamic`]
///
/// Componente novo cunha `stable_type_id` próprio ⇒ **`PROJECT_SCHEMA` NÃO
/// bumpa** (o idioma do `GravityScale`/`MassOverride`/`Dominance`, com *detach
/// no neutro*), e a razão é a que este módulo escreveu sete vezes: *um bump
/// RECUSA todo projeto já salvo*.
///
/// ⚠️ **Um MARCADOR seria a representação errada**, e é a ordem do Enio de
/// 2026-08-07 que o prova: ela nomeia um **terceiro** modo (*"um cinemático puro
/// sangue"*), e a presença de um marcador só sabe dizer duas coisas. Um bit hoje
/// seria um segundo componente amanhã.
///
/// # ⚠️ O que ele decide, e o que NÃO decide
///
/// Ele decide **a lei** ([`ph2d_platformer::Support`]) e **quem é dono da
/// pose**. Ele **não** decide o tipo do corpo no rapier: quem o decide é o
/// `RigidBody.kind`, sempre — e essa separação é o que faz o **bake** continuar
/// a funcionar. Assar um player escreve `RigidBody.kind = Kinematic` para
/// entregar a pose; se este componente sobrescrevesse o tipo do corpo, o bake
/// seria desfeito no frame seguinte, em silêncio.
///
/// **A pergunta do §8 do plano fica respondida:** *um player cinemático assado e
/// um player cinemático dirigido são o mesmo `BodyKind` com donos diferentes da
/// pose*, e o discriminador é **este componente**, nunca o `BodyKind`. Assado
/// sem ele, a cena é dona da pose; com ele, o player é.
///
/// ⚠️ O gesto da §14 escreve **os dois** (este e o `RigidBody.kind`) por UMA
/// porta, porque pedir ao artista que ponha o corpo em Kinematic noutra seção
/// seria a falha de duas-portas que este módulo já pagou.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerMode {
    /// A **cápsula flutuante**: impulsos, mola, e o solver dono da pose.
    #[default]
    Dynamic,
    /// O **controlador**: a pose é escrita, o mundo só diz quanto coube.
    Kinematic,
    /// O **puro sangue**: o mesmo controlador, com o mundo físico virando
    /// CENÁRIO — ele é parado por tudo e não move nada (W-KinPure).
    Pure,
}

impl PlayerMode {
    /// **Este modo escreve a própria pose?** — a pergunta que separa os dois
    /// caminhos da ponte.
    ///
    /// ⚠️ [`PlayerMode::Pure`] responde SIM: ele é o mesmo controlador. O que o
    /// distingue não é como ele se move, é o que ele DEVE ao mundo — a
    /// [`PlayerMode::transmits`].
    #[must_use]
    pub fn drives_itself(self) -> bool {
        matches!(self, PlayerMode::Kinematic | PlayerMode::Pure)
    }

    /// **O que este personagem faz ao mundo volta para o mundo?**
    ///
    /// A 3ª lei tem duas metades — o PESO que o chão sente (K6) e o EMPURRÃO
    /// lateral (`W-KinPush`) — e as duas saem do mesmo [`ReactionConfig`]. Este
    /// é o interruptor delas.
    ///
    /// # ⚠️ Por que o MODO decide, e não os knobs
    ///
    /// Zerar os três escalares à mão dá o mesmo resultado numérico — medido,
    /// `0,0000` na jangada e `0,0000` no caixote. A diferença é de INTENÇÃO: um
    /// modo é uma declaração que sobrevive a alguém mexer num slider, e é o que
    /// permite ao painel **não oferecer** controles que não fazem nada. Se as
    /// duas portas coexistissem, a de baixo seria a que ninguém lembra de
    /// consultar.
    ///
    /// ⚠️ E é por isso que o `Pure` **não zera os knobs autorados**: eles ficam
    /// guardados, e voltar para [`PlayerMode::Kinematic`] devolve o que o
    /// artista tinha escrito.
    #[must_use]
    pub fn transmits(self) -> bool {
        !matches!(self, PlayerMode::Pure)
    }

    /// **Alguém LÊ o `react.push` deste personagem?** (report do Enio,
    /// 2026-08-09: *"Push on Bodies parece não funcionar em Dynamic"*.)
    ///
    /// Ele não funcionava, e não é defeito — é a wave: o empurrão lateral
    /// (W-KinPush) existe porque um corpo **cinemático** tem massa infinita
    /// para o solver e desliza contra um caixote sem lhe transmitir nada,
    /// enquanto o **dinâmico já empurra pelo solver** (medido: 16,55 m contra
    /// 0,0000). ⇒ `push_from_hits` só corre no laço dos `KinMove`, e sob
    /// `Pure` a `ReactionConfig` inteira é zerada.
    ///
    /// ⚠️ **O que ERA defeito é o painel oferecer o slider assim mesmo**, com
    /// um tooltip a confessar *"KINEMATIC only"*. Uma dica não é o remédio
    /// deste app: *"deve ser retirado do painel para dynamic, assim como
    /// qualquer input morto"*. Esta é a porta que o painel pergunta.
    ///
    /// ⚠️ **A composição é deliberada** (`drives_itself && transmits`, e não um
    /// `matches!(Kinematic)`): as duas metades são os dois mecanismos REAIS que
    /// o silenciam, então um quarto modo herda a resposta certa em vez de a
    /// esquecer.
    #[must_use]
    pub fn pushes_bodies(self) -> bool {
        self.drives_itself() && self.transmits()
    }

    /// O `u8` com que este modo atravessa a fronteira da UI, e volta.
    ///
    /// Uma porta, as duas direções — o precedente literal do
    /// [`super::BodyKind::tag`], cujo doc explica o preço de duas: *"o momento
    /// em que existe um terceiro, é um chip que o artista clica e que
    /// silenciosamente seleciona outro"*. E o terceiro CHEGOU (`W-KinPure`).
    #[must_use]
    pub fn tag(self) -> u8 {
        match self {
            PlayerMode::Dynamic => 0,
            PlayerMode::Kinematic => 1,
            PlayerMode::Pure => 2,
        }
    }

    /// A volta: `None` para um tag que este build não conhece — que o chamador
    /// trata ignorando o clique, nunca dobrando num modo qualquer.
    #[must_use]
    pub fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(PlayerMode::Dynamic),
            1 => Some(PlayerMode::Kinematic),
            2 => Some(PlayerMode::Pure),
            _ => None,
        }
    }
}
