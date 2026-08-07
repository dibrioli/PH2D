//! **§14 Platform Player** — o snapshot e a aplicação (W5).
//!
//! Irmão do `inspector_joint_wheel.rs`: o info que o painel lê, e a escrita que
//! o clique dele provoca. Módulo próprio pela mesma razão que a seção é própria
//! — a §11 responde *"que corpo é este?"* e esta responde *"que comportamento
//! ele tem?"*.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor::{InspectorPlayerInfo, PlayerFieldEdit};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PlatformPlayer, RideConfig, RigidBody, WalkConfig,
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
) -> Option<InspectorPlayerInfo> {
    let entity = Entity::from_bits(entity_bits);
    let body = sim.world().get::<RigidBody>(entity)?;
    if body.kind != BodyKind::Dynamic {
        return None;
    }
    let shape = sim.world().get::<Collider>(entity).map(|c| c.shape);
    let player = sim.world().get::<PlatformPlayer>(entity).copied();
    let has_player = player.is_some();
    let p = player.unwrap_or_default();
    let min = shape.and_then(|s| min_float_for(s, p.max_slope_deg));
    Some(InspectorPlayerInfo {
        entity_bits,
        has_player,
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
        lift_momentum: p.lift_momentum,
        wall_slide_speed: p.wall_slide_speed,
        wall_jump_height: p.wall_jump_height,
        wall_jump_push: p.wall_jump_push,
        wall_jump_lockout: p.wall_jump_lockout,
        wall_reach: p.wall_reach,
        wall_grab_stamina: p.wall_grab_stamina,
        dash_speed: p.dash_speed,
        dash_time: p.dash_time,
        dash_cooldown: p.dash_cooldown,
        crouch_height: p.crouch_height,
        crouch_speed: p.crouch_speed,
        reaction_support: p.reaction_support,
        reaction_movement: p.reaction_movement,
        recorded_run_seconds,
    })
}

/// Uma `float_height` que de fato FLUTUA sobre esta forma.
///
/// O mínimo mais uma folga de 20% — um valor exatamente no piso deixa a cápsula
/// tangente, que é o defeito, e não a cura.
fn fitted_float(shape: Option<ColliderShape>, max_slope_deg: f32) -> Option<f32> {
    let min = shape.and_then(|s| min_float_for(s, max_slope_deg))?;
    min.is_finite().then_some(min * 1.2)
}

/// **Aplica uma edição da §14.** Sem fan-out — a seção descreve UM personagem.
pub(crate) fn apply_player_edit(sim: &mut SimWorld, entity_bits: u64, edit: PlayerFieldEdit) {
    let entity = Entity::from_bits(entity_bits);
    if sim.world().get::<RigidBody>(entity).map(|b| b.kind) != Some(BodyKind::Dynamic) {
        // A ponte recusaria em silêncio; recusar AQUI é o que torna a regra
        // uma só (o pintor não oferece, e o barramento não honra).
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
        PlayerFieldEdit::Add | PlayerFieldEdit::Remove | PlayerFieldEdit::ClearRun => {}
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
        PlayerFieldEdit::SpringStrength(v) => p.spring_strength = v.max(0.0),
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
        PlayerFieldEdit::ReactionSupport(v) => p.reaction_support = v.max(0.0),
        PlayerFieldEdit::ReactionMovement(v) => p.reaction_movement = v.max(0.0),
    }
}
