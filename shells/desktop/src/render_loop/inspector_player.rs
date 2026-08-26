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
    // ⚠️ **A vista chega PRONTA, e a shell não a re-deriva** (`W-PlayerOut`, A3):
    // quem a computa é a lei, dentro do laço de tiques da ponte, e uma segunda
    // resposta aqui descreveria um personagem que a simulação não simulou.
    live: Option<ph2d_physics_ecs::PlayerView>,
    // **O que a LEI lê deste corpo** — a resposta do `bridge::pose_owner`,
    // resolvida no chamador (que é quem tem a ponte). Ela chega PRONTA pelo
    // mesmo motivo da vista: as quatro perguntas que a §14 faz são os quatro
    // `if` do `drive_players`, e respondê-las aqui a partir do `PlayerMode` é a
    // segunda cópia que já mentia sobre um player assado.
    law: ph2d_physics_ecs::PlayerLiveness,
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
        // ⚠️ **As três saem do MESMO veredito, e a conjunção com `law_runs` é o
        // que as torna honestas:** um player que a CENA dirige (um bake) não lê
        // nenhum dos três, e a resposta do modo sozinha dizia que sim. O painel
        // recebe bools, nunca o mapeamento (W-KinPure).
        reaction_is_live: law.transmits && law.law_runs,
        // ⚠️ E o empurrão lateral é mais estreito que o card: só o laço dos
        // `KinMove` o corre (um corpo dinâmico já empurra pelo solver).
        push_is_live: law.pushes,
        // ⚠️ **A perna elástica é o terceiro** (a auditoria de 15/08): sob Snap
        // a lei sobrescreve o `float_height` pela geometria do corpo e os dois
        // números da mola não têm consumidor.
        spring_is_live: law.spring,
        float_height: p.float_height,
        min_float_height: min.unwrap_or(0.0),
        min_float_known: min.is_some(),
        cling_distance: p.cling_distance,
        spring_strength: p.spring_strength,
        spring_damping: p.spring_damping,
        speed: p.speed,
        acceleration: p.acceleration,
        air_acceleration: p.air_acceleration,
        brake_scale: p.brake_scale,
        max_slope_deg: p.max_slope_deg,
        jump_height: p.jump_height,
        #[expect(
            clippy::cast_precision_loss,
            reason = "a contagem de pulos do ar cabe num f32 exato muito além de \
                      qualquer valor autorável"
        )]
        air_jumps: p.air_jumps as f32,
        air_jump_height: p.air_jump_height,
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
        ledge_grab: p.ledge_grab,
        ledge_reach_y: p.ledge_reach_y,
        ledge_span: p.ledge_span,
        ledge_offset_y: p.ledge_offset_y,
        glide_fall_speed: p.glide_fall_speed,
        max_fall_speed: p.max_fall_speed,
        platform_lift: p.platform_lift.tag(),
        // ⚠️ **O chip diz `0` = pode andar para fora**, e o campo guarda a
        // CAPACIDADE — a inversão mora aqui, na fronteira, e em nenhum outro
        // lugar (o `u8` é o índice do chip, o `bool` é a lei).
        walk_off_ledges: u8::from(!p.walk_off_ledges),
        crouch_walk_off_ledges: u8::from(!p.crouch_walk_off_ledges),
        // ⚠️ O veredito é COMPARATIVO e mora na lei, não numa segunda regra
        // aqui: agachar é ficar mais baixo, e *mais baixo* só existe contra a
        // altura de pé.
        crouch_armed: p.config().crouch.armed(&p.config().ride),
        ledge_speed: p.ledge_speed,
        reaction_support: p.reaction_support,
        reaction_movement: p.reaction_movement,
        reaction_push: p.reaction_push,
        recorded_run_seconds,
        discarded_run_seconds,
        live: live.map(|v| ph2d_editor::screens::hero::PlayerLive {
            // ⚠️ A tradução variante→número é a do `FootingKind::tag`, e só ela.
            footing_tag: v.footing.tag(),
            facing: v.facing,
            velocity: v.velocity,
        }),
        // ⚠️ **Presença = ligado** — o marcador não tem valor a ler.
        emits_signals: sim
            .world()
            .get::<ph2d_physics_ecs::PlayerSignals>(entity)
            .is_some(),
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
/// ⭐ **A §14 aparece se, e só se, o componente estiver lá** (ADR-0166 / F3).
///
/// ⚠️ **Isto MUDOU na F3, e a mudança é a fase inteira em miniatura.** Até aqui a resposta era
/// *"`true` para todo corpo Dynamic, COM OU SEM o componente"* — a *face vazia*, cujo botão «+ Add
/// Platform Player» era a **única** rota para a feature. Apagá-la antes de a porta nova existir
/// tornaria o player inalcançável; por isso a ordem da F3 é **porta primeiro, poda depois**, com o
/// censo (`component_reach_tests`) verde entre as duas.
///
/// ⚠️ **E o `BodyKind` DEIXOU de mandar, o que é uma reversão deliberada.** A regra antiga também
/// dizia *"e o corpo não pode ser `Static`"*, porque a mola do player é um impulso e um impulso não
/// move massa infinita — mas isso era o critério de **OFERECER O BOTÃO**, e o botão mudou-se para a
/// paleta. Mantê-lo aqui produziria o pior estado dos dois mundos: o artista anexa o componente
/// pelo `+` e **nada aparece**. *Um componente presente e invisível lê-se como defeito*, que é
/// exatamente a doença que a F3 existe para curar. A física continua verdadeira; ela é assunto da
/// §11, onde o tipo do corpo se muda.
fn player_section_applies(_kind: BodyKind, has_player: bool) -> bool {
    has_player
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

/// **Uma FRAÇÃO da 3ª lei** — o que volta ao chão, em `[0, 1]`.
///
/// ⚠️ **Porta única para os três canais** (peso · tapete · empurrão): eles são a
/// mesma grandeza medida em lugares diferentes, e três `clamp` copiados são
/// como o quarto nasce sem um dos lados.
///
/// ⚠️ **Não-finito vira ZERO, e não o teto:** um `NaN` que chegasse ao
/// componente envenenaria o impulso e, por ele, a pose que o `readback` escreve
/// — e um `NaN.clamp(0,1)` devolve `NaN` em Rust. Zero é a resposta segura (o
/// personagem volta a ser um fantasma), nunca a máxima.
fn clamp_fraction(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// **Uma CONTAGEM autorada por um campo numérico** — arredonda, recusa não-finito
/// e nunca é negativa.
///
/// ⚠️ **Sem teto, e é o §0:** ao contrário do [`clamp_samples`], aqui não há
/// recurso que se esgote — um contador de pulos do ar custa uma comparação por
/// tique. A faixa do slider é conforto de arrasto; a caixa de texto aceita o que
/// o artista escrever.
fn round_count(v: f32) -> u32 {
    if !v.is_finite() {
        return 0;
    }
    let n = v.round().clamp(0.0, f32::from(u16::MAX));
    // Seguro: `n` está clampado a `[0, 65535]`.
    n as u32
}

/// ⭐ **O que ANEXAR um `PlatformPlayer` faz além de inserir o ponto neutro** (ADR-0166 / F3, e é a
/// emenda que a F0 mediu: *nem toda porta por-seção é redundante com o `+` — as que **SEMEIAM do
/// valor vivo** fazem algo que a paleta genérica não pode fazer*).
///
/// ⚠️ **Ele nasce PAIRANDO, não tangente.** O ponto de partida do modelo é `0,5`, e a cápsula
/// canónica precisa de mais que isso só para sair do chão — um personagem novo que não flutua é a
/// primeira impressão errada, num app cuja tese é que ele paira. O `insert_default` do registo é
/// type-erased e **não pode** saber isto: a altura certa depende do `Collider` da entidade.
///
/// ⚠️ **Idempotente e conservador:** ele só SOBE (`max`), então correr duas vezes não move nada e
/// um `float_height` autorado alto nunca é rebaixado.
pub(crate) fn seed_attached_player(sim: &mut SimWorld, entity_bits: u64) {
    let entity = Entity::from_bits(entity_bits);
    let shape = sim.world().get::<Collider>(entity).map(|c| c.shape);
    let Some(mut p) = sim.world().get::<PlatformPlayer>(entity).copied() else {
        return;
    };
    let Some(fit) = fitted_float(shape, p.max_slope_deg) else {
        return;
    };
    if fit <= p.float_height {
        return;
    }
    p.float_height = fit;
    sim.world_mut().entity_mut(entity).insert(p);
}

/// **O gesto INTEIRO de anexar um player** — o ponto neutro do tipo, e depois o seed.
///
/// ⚠️ **`PlatformPlayer::default()`, nunca campo a campo.** A 1.ª versão montava o componente a
/// partir do `PlayerConfig::STARTING_POINT` — uma SEGUNDA porta para a tradução que o `Default` já
/// faz —, e ela apodreceu na 1.ª wave que acrescentou campos.
///
/// ⚠️ **Isto era o `PlayerFieldEdit::Add`, e ele MORREU na F3:** o botão «Make Platform Player»
/// vivia dentro da §14, que hoje só se pinta **com** o componente lá — a porta ficaria fechada
/// sobre a própria chave. Quem anexa é o `+` do cabeçalho.
///
/// ⚠️ **Ele atravessa a PORTA DE PRODUÇÃO** (`component_attach::attach_by_name`), e não um
/// `insert` à mão: um atalho de teste que constrói o componente por outro caminho é a segunda porta
/// que diverge — e o que os 27 gates da §14 têm de medir é o gesto que o artista faz. O que ele
/// poupa é só o registo, que de outro modo cada um dos 27 montaria.
#[cfg(test)]
pub(crate) fn attach_player(sim: &mut SimWorld, entity_bits: u64) {
    let reg = crate::init::build_component_registry();
    crate::component_attach::attach_by_name(
        sim,
        &reg,
        entity_bits,
        "ph2d::physics::PlatformPlayer",
    )
    .unwrap_or_else(|m| panic!("a porta de anexar recusou o player: {m}"));
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
        // ⚠️ **Desfaz o que o `Add` e o `Mode` FIZERAM, e não só o componente.**
        //
        // O gesto do modo escreve DUAS metades (o `PlayerMode` e o
        // `RigidBody.kind`), e tirar uma deixava um corpo `Kinematic` sem
        // player: o estado que a §14 **não oferece**, porque ele é dirigido pela
        // cena. O artista removia o comportamento e ficava PRESO, sem controle
        // nenhum na tela para o trazer de volta — o mesmo beco sem saída que a
        // W-KinMove consertou para o gesto do modo, reaberto pela porta do lado.
        //
        // ⚠️ E a consequência silenciosa era pior que o beco: o corpo **deixava
        // de cair**, e um `PlayerMode` órfão viajava no arquivo a descrever um
        // personagem que já não existe.
        PlayerFieldEdit::Remove => {
            let mut e = sim.world_mut().entity_mut(entity);
            e.remove::<PlatformPlayer>();
            e.remove::<PlayerMode>();
            // O opt-in de sinais é uma row DESTA seção: sem ela, ninguém o
            // desliga (a lei do controle sem porta).
            e.remove::<ph2d_physics_ecs::PlayerSignals>();
            if let Some(mut rb) = e.get_mut::<RigidBody>() {
                rb.kind = BodyKind::Dynamic;
            }
            return;
        }
        // ⚠️ **Anexa ou REMOVE — nunca escreve um campo** (A3): o marcador é o
        // booleano, e um componente que ficasse presente a dizer *"desligado"*
        // seria um no-op guardado no arquivo (o idioma do detach dos overrides).
        PlayerFieldEdit::EmitSignals(on) => {
            let mut e = sim.world_mut().entity_mut(entity);
            if on {
                e.insert(ph2d_physics_ecs::PlayerSignals);
            } else {
                e.remove::<ph2d_physics_ecs::PlayerSignals>();
            }
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
        PlayerFieldEdit::Remove
        | PlayerFieldEdit::ClearRun
        | PlayerFieldEdit::RestoreRun
        | PlayerFieldEdit::EmitSignals(_) => {}
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
        // ⚠️ O piso aqui é a FRONTEIRA, e o que garante a lei é o piso do
        // consumidor (`walk::brake_scale`) — ver o gate do freio negativo.
        PlayerFieldEdit::BrakeScale(v) => p.brake_scale = v.max(0.0),
        PlayerFieldEdit::AirAcceleration(v) => p.air_acceleration = v.max(0.0),
        PlayerFieldEdit::MaxSlopeDeg(v) => p.max_slope_deg = v.clamp(0.0, 90.0),
        PlayerFieldEdit::JumpHeight(v) => p.jump_height = v.max(0.0),
        // ⚠️ **Arredonda, não trunca:** a row é um campo numérico e o artista
        // que arrasta pousa em `0,999…`; truncar daria zero — a capacidade
        // desligada com o slider visivelmente em 1.
        PlayerFieldEdit::AirJumps(v) => p.air_jumps = round_count(v),
        PlayerFieldEdit::AirJumpHeight(v) => p.air_jump_height = v.max(0.0),
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
        PlayerFieldEdit::LedgeGrab(v) => p.ledge_grab = v.max(0.0),
        PlayerFieldEdit::LedgeReachY(v) => p.ledge_reach_y = v.max(0.0),
        PlayerFieldEdit::LedgeSpan(v) => p.ledge_span = v.max(0.0),
        // ⚠️ **Sem `max(0.0)`, e é o único da família assim:** um offset é uma
        // DIREÇÃO, e deslizar o sensor para BAIXO é um pedido legítimo (uma
        // arte cuja mão fica abaixo do topo do collider).
        PlayerFieldEdit::LedgeOffsetY(v) => p.ledge_offset_y = v,
        PlayerFieldEdit::GlideFallSpeed(v) => p.glide_fall_speed = v.max(0.0),
        PlayerFieldEdit::MaxFallSpeed(v) => p.max_fall_speed = v.max(0.0),
        // ⚠️ Um tag que nenhuma variante reivindica é IGNORADO, e não dobrado
        // num plausível — a disciplina do `from_tag` (`ph2d_physics_ecs`).
        PlayerFieldEdit::PlatformLift(tag) => {
            if let Some(l) = ph2d_physics_ecs::PlatformLift::from_tag(tag) {
                p.platform_lift = l;
            }
        }
        PlayerFieldEdit::WalkOffLedges(v) => p.walk_off_ledges = v,
        PlayerFieldEdit::CrouchWalkOffLedges(v) => p.crouch_walk_off_ledges = v,
        PlayerFieldEdit::LedgeSpeed(v) => p.ledge_speed = v.max(0.0),
        // ⚠️ **Os três são FRAÇÕES, e o teto é de RECURSO** — o registro das
        // faixas já o diz (*"acima de 1 o personagem devolveria mais do que
        // recebeu"*), e o slider parava em 1; era a **escrita** que só impunha o
        // piso, então digitar `5` na caixa guardava um chão a levar cinco vezes
        // o peso do personagem.
        //
        // ⚠️ Isto é o oposto do slider dual: ali a caixa passa da faixa de
        // propósito (o arrasto é conforto, o disfuncional começa depois). Aqui
        // não há duas grandezas — há **uma**, e ela é a física.
        PlayerFieldEdit::ReactionSupport(v) => p.reaction_support = clamp_fraction(v),
        PlayerFieldEdit::ReactionMovement(v) => p.reaction_movement = clamp_fraction(v),
        PlayerFieldEdit::ReactionPush(v) => p.reaction_push = clamp_fraction(v),
    }
}
