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
    CrouchConfig, DashConfig, JumpConfig, PlayerConfig, ReactionConfig, RideConfig, WalkConfig,
    WallConfig,
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
            },
            walk: WalkConfig {
                speed: self.speed,
                acceleration: self.acceleration,
                air_acceleration: self.air_acceleration,
                max_slope_deg: self.max_slope_deg,
            },
            jump: JumpConfig {
                jump_height: self.jump_height,
                takeoff_gravity: self.takeoff_gravity,
                takeoff_speed: self.takeoff_speed,
                peak_gravity: self.peak_gravity,
                peak_speed: self.peak_speed,
                fall_gravity: self.fall_gravity,
                cut_gravity: self.cut_gravity,
                coyote_time: self.coyote_time,
                jump_buffer: self.jump_buffer,
                corner_reach: self.corner_reach,
                lift_momentum: self.lift_momentum,
            },
            wall: WallConfig {
                slide_speed: self.wall_slide_speed,
                jump_height: self.wall_jump_height,
                jump_push: self.wall_jump_push,
                reach: self.wall_reach,
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
            react: ReactionConfig {
                support: self.reaction_support,
                movement: self.reaction_movement,
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
            speed: c.walk.speed,
            acceleration: c.walk.acceleration,
            air_acceleration: c.walk.air_acceleration,
            max_slope_deg: c.walk.max_slope_deg,
            jump_height: c.jump.jump_height,
            takeoff_gravity: c.jump.takeoff_gravity,
            takeoff_speed: c.jump.takeoff_speed,
            peak_gravity: c.jump.peak_gravity,
            peak_speed: c.jump.peak_speed,
            fall_gravity: c.jump.fall_gravity,
            cut_gravity: c.jump.cut_gravity,
            coyote_time: c.jump.coyote_time,
            jump_buffer: c.jump.jump_buffer,
            corner_reach: c.jump.corner_reach,
            lift_momentum: c.jump.lift_momentum,
            reaction_support: c.react.support,
            reaction_movement: c.react.movement,
            wall_slide_speed: c.wall.slide_speed,
            wall_jump_height: c.wall.jump_height,
            wall_jump_push: c.wall.jump_push,
            wall_reach: c.wall.reach,
            wall_jump_lockout: c.wall.jump_lockout,
            wall_grab_stamina: c.wall.grab_stamina,
            dash_speed: c.dash.speed,
            dash_time: c.dash.time,
            dash_cooldown: c.dash.cooldown,
            crouch_height: c.crouch.height,
            crouch_speed: c.crouch.speed,
        }
    }
}
