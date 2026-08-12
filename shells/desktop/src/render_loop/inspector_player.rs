//! **§14 Platform Player** — o snapshot e a aplicação (W5).
//!
//! Irmão do `inspector_joint_wheel.rs`: o info que o painel lê, e a escrita que
//! o clique dele provoca. Módulo próprio pela mesma razão que a seção é própria
//! — a §11 responde *"que corpo é este?"* e esta responde *"que comportamento
//! ele tem?"*.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor::{InspectorPlayerInfo, PlayerFieldEdit};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PlatformPlayer, PlayerMode, RideConfig, RigidBody,
    WalkConfig,
};

/// A altura de flutuação MÍNIMA que a forma deste corpo exige, se ela tiver uma.
///
/// ⚠️ **A cápsula e a bola têm; a caixa NÃO**, e a diferença é geometria: a
/// extensão de uma cápsula ao longo de uma normal é `radius + half_height·cos θ`
/// (o raio é isotrópico), e a de uma caixa é `half_x·sin θ + half_y·cos θ` — uma
/// fórmula diferente, com um piso diferente. Devolver a fórmula da cápsula para
/// uma caixa seria um número errado apresentado como certo, e por isso o info
/// carrega `min_float_known` em vez de um zero silencioso.
fn min_float_for(shape: ColliderShape, max_slope_deg: f32) -> Option<f32> {
    let cos = WalkConfig {
        max_slope_deg,
        ..WalkConfig::STARTING_POINT
    }
    .max_slope_cos();
    match shape {
        ColliderShape::Capsule {
            half_height,
            radius,
        } => Some(RideConfig::min_float_height(half_height, radius, cos)),
        ColliderShape::Ball { radius } => Some(RideConfig::min_float_height(0.0, radius, cos)),
        ColliderShape::Cuboid { .. } => None,
    }
}

/// **O snapshot da §14.**
///
/// ⚠️ **`Some` para todo corpo DYNAMIC, com ou sem o componente** — a face vazia
/// é o botão que faz o comportamento existir (a lição da §11 do W2a). E `None`
/// para o resto por FÍSICA: a mola é um impulso, e um impulso não move massa
/// infinita, então oferecer o botão num Static seria oferecer um gesto que a
/// ponte recusa em silêncio.
///
/// ⚠️ **`recorded_run_seconds` entra por ARGUMENTO, e é o único campo que não sai
/// do componente** (W17): a fita é um fato do DOCUMENTO, e o construtor do info
/// só recebe o mundo. Derivá-la aqui de alguma outra coisa seria inventá-la.
pub(crate) fn build_player_info(
    sim: &SimWorld,
    entity_bits: u64,
    recorded_run_seconds: f32,
    discarded_run_seconds: f32,
) -> Option<InspectorPlayerInfo> {
    let entity = Entity::from_bits(entity_bits);
    let body = sim.world().get::<RigidBody>(entity)?;
    let shape = sim.world().get::<Collider>(entity).map(|c| c.shape);
    let player = sim.world().get::<PlatformPlayer>(entity).copied();
    let has_player = player.is_some();
    if !player_section_applies(body.kind, has_player) {
        return None;
    }
    let p = player.unwrap_or_default();
    let min = shape.and_then(|s| min_float_for(s, p.max_slope_deg));
    let mode = sim
        .world()
        .get::<PlayerMode>(entity)
        .copied()
        .unwrap_or_default();
    Some(InspectorPlayerInfo {
        entity_bits,
        has_player,
        mode_tag: mode.tag(),
        // ⚠️ A pergunta é feita à PORTA do modo, aqui na shell — o painel recebe
        // a resposta, nunca o mapeamento (W-KinPure).
        reaction_is_live: mode.transmits(),
        // ⚠️ E o empurrão lateral é mais estreito que o card: só o cinemático o
        // lê (um corpo dinâmico já empurra pelo solver). A porta é do MODO.
        push_is_live: mode.pushes_bodies(),
        float_height: p.float_height,
        min_float_height: min.unwrap_or(0.0),
        min_float_known: min.is_some(),
        cling_distance: p.cling_distance,
        spring_strength: p.spring_strength,
        spring_damping: p.spring_damping,
        speed: p.speed,
        acceleration: p.acceleration,
        air_acceleration: p.air_acceleration,
        max_slope_deg: p.max_slope_deg,
        jump_height: p.jump_height,
        takeoff_gravity: p.takeoff_gravity,
        takeoff_speed: p.takeoff_speed,
        peak_gravity: p.peak_gravity,
        peak_speed: p.peak_speed,
        fall_gravity: p.fall_gravity,
        cut_gravity: p.cut_gravity,
        coyote_time: p.coyote_time,
        jump_buffer: p.jump_buffer,
        corner_reach: p.corner_reach,
        corner_samples: f32::from(p.corner_samples),
        corner_lookahead: p.corner_lookahead,
        lift_momentum: p.lift_momentum,
        wall_slide_speed: p.wall_slide_speed,
        wall_jump_height: p.wall_jump_height,
        wall_jump_push: p.wall_jump_push,
        wall_jump_lockout: p.wall_jump_lockout,
        wall_reach: p.wall_reach,
        foot_samples: f32::from(p.foot_samples),
        foot_spread: p.foot_spread,
        wall_samples: f32::from(p.wall_samples),
        wall_spread: p.wall_spread,
        wall_grab_stamina: p.wall_grab_stamina,
        dash_speed: p.dash_speed,
        dash_time: p.dash_time,
        dash_cooldown: p.dash_cooldown,
        crouch_height: p.crouch_height,
        crouch_speed: p.crouch_speed,
        swim_speed: p.swim_speed,
        swim_acceleration: p.swim_acceleration,
        swim_enter: p.swim_enter,
        reaction_support: p.reaction_support,
        reaction_movement: p.reaction_movement,
        reaction_push: p.reaction_push,
        recorded_run_seconds,
        discarded_run_seconds,
    })
}

/// **A §14 vale para este corpo?** — a porta ÚNICA, e ela tem DOIS consumidores.
///
/// O `build_player_info` a pergunta para OFERECER a seção; o `apply_player_edit`
/// a pergunta para HONRAR um clique. ⚠️ **Duas cópias divergiram e o preço foi
/// medido:** a versão anterior era `kind == Dynamic`, escrita duas vezes, e sob
/// o modo cinemático da `W-KinMove` isso significa que clicar `Kinematic` fazia
/// a seção DESAPARECER — e, pior, o clique de volta era recusado pela outra
/// cópia. O artista escolhia o modo e ficava preso nele, sem controle nenhum na
/// tela a dizer por quê.
///
/// ⚠️ **`Dynamic`, ou um player que JÁ é cinemático.** Um corpo cinemático SEM o
/// componente continua fora, e por física: ele é dirigido pela CENA (um bake,
/// uma curva), e criar um player ali daria um personagem que a ponte não dirige
/// (o `pose_owner` responderia `Scene`).
fn player_section_applies(kind: BodyKind, has_player: bool) -> bool {
    kind == BodyKind::Dynamic || (kind == BodyKind::Kinematic && has_player)
}

/// Uma `float_height` que de fato FLUTUA sobre esta forma.
///
/// O mínimo mais uma folga de 20% — um valor exatamente no piso deixa a cápsula
/// tangente, que é o defeito, e não a cura.
fn fitted_float(shape: Option<ColliderShape>, max_slope_deg: f32) -> Option<f32> {
    let min = shape.and_then(|s| min_float_for(s, max_slope_deg))?;
    min.is_finite().then_some(min * 1.2)
}

/// **O número que uma row de CONTAGEM guarda** — recusa lixo e nada mais.
///
/// ⚠️ **O clamp para ÍMPAR mora na LEI** (`ph2d_platformer::odd_samples`), e não
/// aqui: a ponte e o desenho perguntam à mesma porta, e um segundo
/// arredondamento no caminho de autoria faria o número GUARDADO discordar do
/// número CASTADO — o artista leria 4 e receberia 5, sem nada na tela a dizê-lo.
fn clamp_samples(v: f32, max: usize) -> u16 {
    let max = u16::try_from(max).unwrap_or(u16::MAX);
    if !v.is_finite() {
        return 1;
    }
    let n = v.round().clamp(1.0, f32::from(max));
    // Seguro: `n` está clampado a `[1, max]` e `max` cabe num `u16`.
    n as u16
}

/// **Aplica uma edição da §14.** Sem fan-out — a seção descreve UM personagem.
pub(crate) fn apply_player_edit(sim: &mut SimWorld, entity_bits: u64, edit: PlayerFieldEdit) {
    let entity = Entity::from_bits(entity_bits);
    let kind = sim.world().get::<RigidBody>(entity).map(|b| b.kind);
    let has_player = sim.world().get::<PlatformPlayer>(entity).is_some();
    // A MESMA porta que decide se a seção é pintada — o pintor não oferece e o
    // barramento não honra a partir de uma resposta só.
    if !kind.is_some_and(|k| player_section_applies(k, has_player)) {
        return;
    }
    let shape = sim.world().get::<Collider>(entity).map(|c| c.shape);

    match edit {
        PlayerFieldEdit::Add => {
            // ⚠️ **`default()`, nunca campo a campo.** A 1ª versão montava o
            // componente aqui a partir do `PlayerConfig::STARTING_POINT` — uma
            // SEGUNDA porta para a tradução que o `Default` já faz —, e ela
            // apodreceu na primeira wave que acrescentou campos (a W4, o pulo):
            // o compilador pegou, mas se o `PlatformPlayer` tivesse `..default()`
            // no meio o componente novo teria nascido com sete zeros.
            let mut p = PlatformPlayer::default();
            // ⚠️ **Ele nasce PAIRANDO, não tangente.** O ponto de partida do
            // modelo é `0,5`, e a cápsula canônica precisa de mais que isso só
            // para sair do chão — um personagem novo que não flutua é a primeira
            // impressão errada, e o app sabe a resposta.
            if let Some(fit) = fitted_float(shape, p.max_slope_deg) {
                p.float_height = p.float_height.max(fit);
            }
            sim.world_mut().entity_mut(entity).insert(p);
            return;
        }
        // ⚠️ **UM gesto, DOIS campos** — ver `ids::INSP_PLAYER_MODE`. O
        // `PlayerMode` decide a LEI e quem escreve a pose; o `RigidBody.kind`
        // decide o que o corpo É no rapier. Pedir os dois ao artista, em duas
        // seções, é a falha de duas-portas; e derivar o kind do componente
        // desfaria o BAKE (que põe `Kinematic` num player que a CENA dirige).
        PlayerFieldEdit::Mode(tag) => {
            let Some(mode) = PlayerMode::from_tag(tag) else {
                // Tag que este build não conhece: o clique é IGNORADO, nunca
                // dobrado num modo qualquer (a lição do `BodyKind::tag`).
                return;
            };
            let mut e = sim.world_mut().entity_mut(entity);
            if mode == PlayerMode::default() {
                // **Detach no neutro** — o idioma do `GravityScale`: um arquivo
                // não carrega um no-op.
                e.remove::<PlayerMode>();
            } else {
                e.insert(mode);
            }
            if let Some(mut rb) = e.get_mut::<RigidBody>() {
                rb.kind = if mode.drives_itself() {
                    BodyKind::Kinematic
                } else {
                    BodyKind::Dynamic
                };
            }
            return;
        }
        PlayerFieldEdit::Remove => {
            sim.world_mut()
                .entity_mut(entity)
                .remove::<PlatformPlayer>();
            return;
        }
        _ => {}
    }

    let Some(mut p) = sim.world_mut().get_mut::<PlatformPlayer>(entity) else {
        return;
    };
    match edit {
        // Tratados acima; repetidos aqui só para o `match` ser exaustivo sem um
        // `_` que engoliria um variant novo em silêncio.
        //
        // ⚠️ **O `ClearRun` NÃO é tratado acima — ele nunca chega aqui** (W17): a
        // fita não é um componente, então o laço de ações o intercepta antes do
        // fan-out por entidade, como faz com o `Join` da §11. Ele está nomeado
        // neste braço em vez de num `_` justamente para o dia em que alguém mover
        // a interceptação: o braço fica INERTE e visível, e não engole o verbo.
        PlayerFieldEdit::Add
        | PlayerFieldEdit::Remove
        | PlayerFieldEdit::ClearRun
        | PlayerFieldEdit::RestoreRun => {}
        PlayerFieldEdit::Mode(_) => {}
        PlayerFieldEdit::FitFloatHeight => {
            if let Some(fit) = fitted_float(shape, p.max_slope_deg) {
                p.float_height = fit;
            }
        }
        // ⚠️ **A MESMA função, e é o desenho inteiro** (W18): o piso é da FORMA e
        // da rampa, não de qual perna está em uso — as duas alturas de flutuação
        // são a mesma grandeza, e uma segunda fórmula para a de baixo divergiria
        // no dia em que a caixa ganhasse a dela.
        //
        // ⚠️ E ele **não pode passar da altura de pé**: um agachar mais alto que
        // estar em pé não é um agachar. Só morde numa cápsula cujo piso já esteja
        // acima da perna autorada — e nesse caso o número honesto é a própria
        // perna, porque o corpo não desce mesmo.
        PlayerFieldEdit::FitCrouchHeight => {
            if let Some(fit) = fitted_float(shape, p.max_slope_deg) {
                p.crouch_height = fit.min(p.float_height);
            }
        }
        PlayerFieldEdit::FloatHeight(v) => p.float_height = v.max(0.0),
        PlayerFieldEdit::ClingDistance(v) => p.cling_distance = v.max(0.0),
        // ⚠️ O teto é MEDIDO e é um fato da discretização — acima de `1/dt²` a
        // mola passa do alvo em vez de chegar nele, e o personagem afunda e
        // range. Mesmo tratamento do amortecimento, logo abaixo.
        PlayerFieldEdit::SpringStrength(v) => {
            p.spring_strength = v.clamp(0.0, RideConfig::MAX_SPRING_STRENGTH);
        }
        // ⚠️ O clamp honra o TETO MEDIDO da lei: acima dele o boost inverte a
        // velocidade em vez de matá-la. A porta da lei também o clampa — duas
        // camadas, e a de fora existe para o número AUTORADO nunca guardar algo
        // que o motor não vai honrar (um campo que mente sobre si mesmo).
        PlayerFieldEdit::SpringDamping(v) => {
            p.spring_damping = v.clamp(0.0, RideConfig::MAX_DAMPING);
        }
        PlayerFieldEdit::Speed(v) => p.speed = v.max(0.0),
        PlayerFieldEdit::Acceleration(v) => p.acceleration = v.max(0.0),
        PlayerFieldEdit::AirAcceleration(v) => p.air_acceleration = v.max(0.0),
        PlayerFieldEdit::MaxSlopeDeg(v) => p.max_slope_deg = v.clamp(0.0, 90.0),
        PlayerFieldEdit::JumpHeight(v) => p.jump_height = v.max(0.0),
        // ⚠️ Os multiplicadores só têm PISO. Um teto aqui seria um número que
        // eu escolheria sem medir nada — o topo é onde o desenho deixa de ser
        // um pulo, e isso é decisão de LOOK; o range do slider já é a resposta
        // de UX (CLAUDE.md §0: um limite legítimo diz de que RECURSO ele é).
        PlayerFieldEdit::TakeoffGravity(v) => p.takeoff_gravity = v.max(0.0),
        PlayerFieldEdit::TakeoffSpeed(v) => p.takeoff_speed = v.max(0.0),
        PlayerFieldEdit::PeakGravity(v) => p.peak_gravity = v.max(0.0),
        PlayerFieldEdit::PeakSpeed(v) => p.peak_speed = v.max(0.0),
        PlayerFieldEdit::FallGravity(v) => p.fall_gravity = v.max(0.0),
        PlayerFieldEdit::CutGravity(v) => p.cut_gravity = v.max(0.0),
        PlayerFieldEdit::CoyoteTime(v) => p.coyote_time = v.max(0.0),
        PlayerFieldEdit::JumpBuffer(v) => p.jump_buffer = v.max(0.0),
        PlayerFieldEdit::CornerReach(v) => p.corner_reach = v.max(0.0),
        // ⚠️ **O clamp para ÍMPAR mora na LEI** (`ph2d_platformer::odd_samples`),
        // não aqui: a ponte e o desenho perguntam à mesma porta, e um segundo
        // arredondamento no caminho de autoria faria o número guardado discordar
        // do número castado. O que este braço faz é só recusar lixo.
        PlayerFieldEdit::CornerSamples(v) => {
            p.corner_samples = clamp_samples(v, ph2d_physics_ecs::MAX_CORNER_SAMPLES);
        }
        PlayerFieldEdit::CornerLookahead(v) => p.corner_lookahead = v.max(0.0),
        // ⚠️ A perna cai no MESMO clamp de contagem do flanco e da quina — o
        // arredondamento para ímpar mora na LEI (`odd_samples`), nunca aqui: um
        // segundo arredondamento faria o número guardado discordar do castado.
        PlayerFieldEdit::FootSamples(v) => {
            p.foot_samples = clamp_samples(v, ph2d_physics_ecs::MAX_WALL_SAMPLES);
        }
        PlayerFieldEdit::FootSpread(v) => p.foot_spread = v.clamp(0.0, 1.0),
        PlayerFieldEdit::WallSamples(v) => {
            p.wall_samples = clamp_samples(v, ph2d_physics_ecs::MAX_WALL_SAMPLES);
        }
        PlayerFieldEdit::WallSpread(v) => p.wall_spread = v.clamp(0.0, 1.0),
        PlayerFieldEdit::LiftMomentum(v) => p.lift_momentum = v.max(0.0),
        // AS PAREDES (W13) — todos não-negativos: cada um é uma distância, uma
        // velocidade ou um tempo, e nenhum tem sentido negativo.
        PlayerFieldEdit::WallSlideSpeed(v) => p.wall_slide_speed = v.max(0.0),
        PlayerFieldEdit::WallJumpHeight(v) => p.wall_jump_height = v.max(0.0),
        PlayerFieldEdit::WallJumpPush(v) => p.wall_jump_push = v.max(0.0),
        PlayerFieldEdit::WallJumpLockout(v) => p.wall_jump_lockout = v.max(0.0),
        PlayerFieldEdit::WallReach(v) => p.wall_reach = v.max(0.0),
        PlayerFieldEdit::WallGrabStamina(v) => p.wall_grab_stamina = v.max(0.0),
        PlayerFieldEdit::DashSpeed(v) => p.dash_speed = v.max(0.0),
        PlayerFieldEdit::DashTime(v) => p.dash_time = v.max(0.0),
        PlayerFieldEdit::DashCooldown(v) => p.dash_cooldown = v.max(0.0),
        PlayerFieldEdit::CrouchHeight(v) => p.crouch_height = v.max(0.0),
        PlayerFieldEdit::CrouchSpeed(v) => p.crouch_speed = v.max(0.0),
        PlayerFieldEdit::SwimSpeed(v) => p.swim_speed = v.max(0.0),
        PlayerFieldEdit::SwimAcceleration(v) => p.swim_acceleration = v.max(0.0),
        // ⚠️ **Sem teto aqui, e é o §0:** o limiar conta PESOS, e o número em que
        // ele deixa de ser alcançável depende das duas densidades da CENA — não
        // há recurso a capar. Quem escolhe a faixa confortável é o slider da §14
        // (0..4, o valor medido na poça das fixtures); a caixa aceita o resto.
        PlayerFieldEdit::SwimEnter(v) => p.swim_enter = v.max(0.0),
        PlayerFieldEdit::ReactionSupport(v) => p.reaction_support = v.max(0.0),
        PlayerFieldEdit::ReactionMovement(v) => p.reaction_movement = v.max(0.0),
        PlayerFieldEdit::ReactionPush(v) => p.reaction_push = v.max(0.0),
    }
}
