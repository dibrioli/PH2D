//! **A lane do PLAYER DE PLATAFORMA** (W7) — irmão próprio pelo cap de 700 LOC,
//! e cortado por assunto pela mesma razão dos outros três: uma zona é um corpo
//! com um flag, um rig é uma montagem, um joint é um vínculo — e um player é a
//! única lane cujo estado depende de um **fluxo de entrada por tique**.
//!
//! # ⚠️ Por que ele TEM de estar aqui
//!
//! O `physics_ecs_c9` existe para provar que o **NOSSO** código é bit-idêntico
//! nos três OSes — a ordem de iteração, a fronteira metros↔rapier, o readback.
//! O controlador de player acrescenta caminhos que nenhuma outra lane exercita:
//! um **ray cast** contra o BVH (a perna), uma **escrita direta de velocidade**
//! (o amortecedor e a caminhada), um **impulso resistido pela massa** (a
//! decolagem) e a **reação da 3ª lei** de volta ao chão. Fora do hash, nada
//! provaria que eles atravessam a fronteira igual em toda plataforma.
//!
//! # ⚠️ A fita é ROTEIRIZADA, nunca gravada
//!
//! Uma fita saída de um teclado descreve uma corrida que ninguém consegue
//! repetir — e um harness de determinismo cuja ENTRADA não é reproduzível não
//! mede nada. Aqui ela é função pura do tique.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InputTape, LockRotation, PlatformPlayer, PlayerInput,
    RigidBody,
};

/// O chão do player, longe das outras lanes (o piso principal do harness vai só
/// até x = 50) — uma lane que empurra outra mediria as duas.
const GROUND_X: f32 = 90.0;

/// O chão da lane do EMPURRÃO, longe das outras pela mesma razão do `GROUND_X`:
/// uma lane que esbarra noutra mede as duas.
const PUSH_X: f32 = 140.0;

/// **A corrida roteirizada.** Anda para a direita, pula no meio, e para no fim —
/// os três regimes da lei (caminhada, decolagem+arco, freio) num run de 120
/// tiques.
///
/// ⚠️ **Função pura do tique**: nenhum relógio de parede, nenhum RNG, nenhuma
/// entrada de dispositivo. É o que a torna parte do harness em vez de ruído
/// dentro dele.
pub fn tape(ticks: u64) -> InputTape {
    let mut t = InputTape::new();
    for k in 1..=ticks {
        t.record(
            k,
            PlayerInput {
                drive: if k < 90 { 1.0 } else { 0.0 },
                jump: (40..48).contains(&k),
                // ⚠️ **A fita não segura o baixo, e é deliberado** (W12): a
                // descida através de uma plataforma jump-through é uma
                // capacidade nova, e pô-la aqui moveria o hash sem que este
                // harness ganhasse nada — ele mede DETERMINISMO, não cobertura.
                // O gate da descida é comportamental e mora ao lado.
                down: false,
                // ⚠️ **Nem o arranque** (W14), pela razão exacta da linha acima
                // — e aqui ela vira uma PROVA: o `physics_ecs_c9` sai
                // byte-idêntico ao da wave anterior, que é o que torna
                // verificável a promessa de que a capacidade é opt-in.
                dash: false,
                grab: false,
            },
        );
    }
    t
}

/// O player e o chão dele.
pub fn spawn(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        Name::new("C9 Player Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 10.0,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(GROUND_X, 0.0)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Player"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.4,
                radius: 0.3,
            },
            ..Collider::default()
        },
        // ⚠️ Um personagem que TOMBA não é um personagem — a mesma razão pela
        // qual todo platformer 2D trava o DOF angular.
        LockRotation,
        PlatformPlayer::default(),
        Transform::from_translation(Vec2::new(GROUND_X - 8.0, 1.5)),
    ));

    // ── A LADEIRA RECUSADA (W9) ──────────────────────────────────────────────
    //
    // ⚠️ **Um segundo player, e ele existe para um RAMO.** A lei do `no_uphill`
    // só corre quando o sensor devolve `Footing::Steep` — uma superfície ao
    // alcance da perna e íngreme demais —, e nenhuma lane do harness tinha essa
    // forma: o hash cobria a caminhada, a decolagem e a reação, e o ramo novo
    // atravessava a fronteira sem ninguém olhar em três OSes.
    //
    // A rampa é de **60°** contra o limite de partida (45), e a fita é a MESMA
    // (`drive = 1` nos primeiros 90 tiques): ele empurra ladeira acima o run
    // inteiro, que é exatamente a entrada em que o defeito vivia.
    sim.world_mut().spawn((
        Name::new("C9 Player Slope"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 6.0,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform {
            rotation: 60.0_f32.to_radians(),
            ..Transform::from_translation(Vec2::new(GROUND_X + 24.0, 0.0))
        },
    ));
    sim.world_mut().spawn((
        Name::new("C9 Player On Slope"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.4,
                radius: 0.3,
            },
            ..Collider::default()
        },
        LockRotation,
        // ⚠️ A altura de flutuação tem o PISO GEOMÉTRICO desta cápsula com folga
        // (`half + radius / cos(45°)` ≈ 0,82): sem ela o personagem nasceria
        // tangente e a rampa o faria penetrar, medindo outra coisa.
        PlatformPlayer {
            float_height: 1.2,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(GROUND_X + 24.0, 2.0)),
    ));

    // ── A QUINA (W10) ────────────────────────────────────────────────────────
    //
    // ⚠️ **Um terceiro player, e ele existe por um RAMO que muda a POSE.** A
    // correção de quina é a primeira coisa desta lei que escreve *translação* em
    // vez de força ou velocidade (`PhysicsWorld::nudge_body`), e sem uma beirada
    // no harness ela nunca dispara: numa cena sem teto o perfil sai todo livre,
    // a lei devolve `None`, e os 67 raios por tique de subida atravessam a
    // fronteira sem que ninguém compare os três sistemas operacionais.
    //
    // ⚠️ **`speed = 0` é o que torna a geometria previsível.** A fita é
    // COMPARTILHADA (`drive = 1` nos 90 primeiros tiques), então este player
    // andaria para longe da beirada antes de pular; com a velocidade de cruzeiro
    // em zero ele fica onde nasceu, e a sobreposição de 8 cm que a cena descreve
    // é a que a lei de fato vê.
    const CORNER_X: f32 = GROUND_X + 48.0;
    sim.world_mut().spawn((
        Name::new("C9 Corner Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 6.0,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(CORNER_X, 0.0)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Corner Ledge"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 3.0,
                half_y: 0.5,
            },
            // A borda ESQUERDA 8 cm dentro da cabeça (meia-largura 0,3): um
            // encosto raso, dentro do `corner_reach` de partida (0,12).
            ..Collider::default()
        },
        // ⚠️ A face de baixo em **2,9**, e a altura foi VARRIDA em vez de
        // calculada: a fita segura o pulo por 8 tiques e o `cut_gravity` corta o
        // resto, então o arco não é o da altura autorada. Com a beirada em 3,3 o
        // personagem NÃO a alcançava e a lane ficava inerte — o hash saía
        // idêntico com a assistência ligada e desligada, que é a forma de uma
        // lane de determinismo não cobrir o ramo que ela diz cobrir.
        Transform::from_translation(Vec2::new(CORNER_X + 0.22 + 3.0, 2.9 + 0.5)),
    ));
    // ── O EMPURRÃO CINEMÁTICO (W-KinPush) ───────────────────────────────────
    //
    // ⚠️ **Uma lane, porque o canal acrescenta aritmética que nenhuma outra
    // atravessa:** o `move_shape` do controlador, a projeção do bloqueio na
    // normal do contato e um `apply_impulse_at_point` derivado dela. As lanes de
    // player acima são todas DINÂMICAS, então o modo Snap inteiro — e com ele o
    // empurrão — passava a fronteira sem ninguém olhar em três OSes.
    //
    // ⚠️ **O caixote é PESADO** (densidade 16) pela razão medida na wave: um
    // leve é limitado pela VELOCIDADE do personagem e a trajetória dele deixa de
    // depender do tamanho do impulso — uma lane assim seria quase insensível ao
    // número que ela existe para vigiar.
    sim.world_mut().spawn((
        Name::new("C9 Push Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 10.0,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(PUSH_X, 0.0)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Push Crate"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.3,
                half_y: 0.3,
            },
            density: 16.0,
            ..Collider::default()
        },
        // ⚠️ **SEM `LockRotation`, e a ausência é o gate** (§8.2). A W-KinPush
        // travava a rotação aqui pelo mesmo motivo que a cena `=102`: sem isso
        // o caixote TOMBAVA e a lane passava a medir tombo em vez de empurrão.
        // Era a decisão certa para aquela wave e a errada para o harness — *um
        // defeito contornado por uma fixture não é um defeito nomeado*, e o
        // rodopio de 74,29 rad viveu meses atrás desta linha.
        //
        // Com o empurrão a entrar no CENTRO de massa o giro é residual, então a
        // rotação livre é um número estável que o hash pode carregar — e um
        // ponto de volta na porta lateral o move na hora.
        Transform::from_translation(Vec2::new(PUSH_X + 1.5, 0.4)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Push Player"),
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.3,
                radius: 0.2,
            },
            ..Collider::default()
        },
        LockRotation,
        ph2d_physics_ecs::PlayerMode::Kinematic,
        PlatformPlayer {
            float_height: 0.9,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(PUSH_X, 0.6)),
    ));

    // ── A PLATAFORMA MÓVEL (W-KinCarry) ─────────────────────────────────────
    //
    // ⚠️ **Uma lane, e a razão é uma AUSÊNCIA que ninguém tinha nomeado:**
    // nenhuma lane deste harness tinha plataforma móvel, então
    // `PhysicsWorld::point_velocity` **nunca saía de zero** em três OSes — e
    // ela é a entrada de um número do rapier que o NOSSO código projeta
    // (`ph2d_platformer::ground_carry`) e transforma numa POSE lida de volta
    // para o `Transform`. É exactamente o que este harness existe para vigiar.
    //
    // ⚠️ **A laje é DINÂMICA e flutua**, e isso é deliberado: uma plataforma
    // cinemática teria a velocidade derivada de uma pose que a CENA escreve,
    // e o laço deste harness não escreve na cena (a fita é a única entrada
    // por-tique, e é função pura do tique). Com `GravityScale(0)` a laje
    // guarda a velocidade lateral que nasceu com ela — nada em `x` a freia —
    // e a reação do personagem sobre ela cruza a fronteira junto.
    const CARRY_X: f32 = 200.0;
    sim.world_mut().spawn((
        Name::new("C9 Carry Platform"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 12.0,
                half_y: 0.25,
            },
            ..Collider::default()
        },
        LockRotation,
        ph2d_physics_ecs::GravityScale(0.0),
        ph2d_physics_ecs::InitialVelocity {
            linvel: [2.0, 0.0],
            angvel: 0.0,
        },
        Transform::from_translation(Vec2::new(CARRY_X, 0.0)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Carry Player"),
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.3,
                radius: 0.2,
            },
            ..Collider::default()
        },
        LockRotation,
        ph2d_physics_ecs::PlayerMode::Kinematic,
        PlatformPlayer {
            float_height: 0.9,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(CARRY_X - 6.0, 0.6)),
    ));

    // ⚠️ **O PURO SANGUE (W-KinPure) NÃO tem lane aqui, e é a mesma razão que a
    // fita usa para recusar o arranque:** ele não acrescenta aritmética nenhuma
    // — ele é a AUSÊNCIA de dois termos que as lanes acima já atravessam. Uma
    // lane dele moveria o hash sem cobrir um único `f32` novo em três OSes, e
    // este harness mede DETERMINISMO, não cobertura.
    //
    // E a ausência dela é o que torna verificável a promessa de que a wave é
    // byte-neutra para todo o resto: o hash saiu IDÊNTICO ao da W-KinPush.
    sim.world_mut().spawn((
        Name::new("C9 Corner Player"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.4,
                radius: 0.3,
            },
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height: 1.2,
            speed: 0.0,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(CORNER_X, 1.3)),
    ));

    // ── A RAMPA A DESCER (§8.1) ─────────────────────────────────────────────
    //
    // ⚠️ **Esta lane existe porque o hash NÃO se moveu quando a lei mudou.** O
    // piso da absorção (`surface_descent`) só é diferente de zero numa
    // superfície INCLINADA, e nenhuma lane do c9 tinha uma sob um player
    // cinemático — o harness de determinismo era **cego à lei nova**, e um
    // harness cego não prova nada sobre ela.
    //
    // ⚠️ A rampa é ESTÁTICA e o personagem CAMINHA (a fita dá `drive`): o piso
    // é função da velocidade tangencial, então um personagem parado o deixaria
    // em zero e a lane voltaria a não medir nada.
    const RAMP_X: f32 = 240.0;
    let slope = -20.0_f32.to_radians();
    sim.world_mut().spawn((
        Name::new("C9 Ramp"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 14.0,
                half_y: 0.25,
            },
            ..Collider::default()
        },
        Transform {
            rotation: slope,
            ..Transform::from_translation(Vec2::new(RAMP_X, 0.0))
        },
    ));
    sim.world_mut().spawn((
        Name::new("C9 Ramp Player"),
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.3,
                radius: 0.2,
            },
            ..Collider::default()
        },
        LockRotation,
        ph2d_physics_ecs::PlayerMode::Kinematic,
        PlatformPlayer {
            float_height: 0.9,
            ..PlatformPlayer::default()
        },
        // Acima da face da rampa naquele x, com folga para assentar.
        Transform::from_translation(Vec2::new(RAMP_X - 6.0, 6.0 * slope.abs().tan() + 1.1)),
    ));

    // ── A RAMPA QUE A LEI RECUSA (§8.3) ─────────────────────────────────────
    //
    // ⚠️ **Pelo MESMO motivo da lane acima, e o hash provou-o outra vez:** a
    // absorção passou a pedir as duas respostas (`grounded` E `footing`), e o
    // hash **não se moveu** — porque a rampa de −20° é caminhável e ali as duas
    // são verdadeiras. A lane que exercita a mudança é uma rampa mais INGREME
    // que o `max_slope` autorado, onde o `footing` recusa e o `grounded` não.
    //
    // ⚠️ **A rampa sobe para a DIREITA, e o sinal é load-bearing.** A fita é
    // uma só para todos os players (`drive = +1` nos primeiros 90 tiques), e a
    // primeira versão desta lane pôs a rampa a DESCER para a direita: o
    // personagem escorregava ladeira abaixo empurrado pela própria fita, nas
    // DUAS leis, e a lane media o mesmo nos dois lados — cobertura aparente
    // sobre um fixture que não continha o fenômeno. Subindo, o empurrão trabalha
    // CONTRA o escorregão, e só a lei nova o deixa descer.
    const STEEP_X: f32 = 300.0;
    let steep = 60.0_f32.to_radians();
    sim.world_mut().spawn((
        Name::new("C9 Steep"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 14.0,
                half_y: 0.25,
            },
            ..Collider::default()
        },
        Transform {
            rotation: steep,
            ..Transform::from_translation(Vec2::new(STEEP_X, 0.0))
        },
    ));
    sim.world_mut().spawn((
        Name::new("C9 Steep Player"),
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.3,
                radius: 0.2,
            },
            ..Collider::default()
        },
        LockRotation,
        ph2d_physics_ecs::PlayerMode::Kinematic,
        PlatformPlayer {
            float_height: 0.9,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(STEEP_X, 0.25 / steep.cos() + 1.0)),
    ));
}
