//! **A cena do SERVO e do GUINCHO** (`PH2D_PHYSICS_SMOKE=48`, W-J6).
//!
//! Até esta wave um motor existia só no Pin e só sabia dizer *"gire a esta
//! taxa"*. Agora ele diz também *"vá até aqui e SEGURE"* (o servo), e existe
//! também no **Slider** (o trilho motorizado) e na **Rope** (o guincho).
//!
//! As duas metades são separadas de propósito: `has_motor` responde QUAIS tipos
//! são dirigidos, `MotorMode` responde QUAL é a instrução. E a unidade segue o
//! grau de liberdade livre — graus numa dobradiça, metros num trilho e num
//! guincho —, que é uma pergunta DIFERENTE da unidade dos limites: uma Rope não
//! tem alcance nenhum e tem motor linear.
//!
//! Os números abaixo saíram de uma sonda headless sobre esta mesma armação,
//! rodada ANTES desta mensagem ser escrita.

use crate::physics_smoke::spawn_floor;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, MotorMode, PhysicsJoint, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

impl crate::App {
    /// **Cena 48 (W-J6).** Um servo, um trilho motorizado, um guincho — e um par
    /// solto para armar um motor à mão. PAUSADA.
    pub(crate) fn physics_smoke_joint_motor(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        {
            let mut body = |name: &str,
                            kind: BodyKind,
                            shape: ColliderShape,
                            size: [f32; 2],
                            rgba: [f32; 4],
                            at: [f32; 2]| {
                world.spawn((
                    Transform::from_translation(Vec2::new(at[0], at[1])),
                    Sprite::atlas(WHITE_TILE_KEY, size, rgba),
                    Name::new(name.to_string()),
                    RigidBody { kind },
                    Collider {
                        shape,
                        ..Collider::default()
                    },
                ));
            };
            let grey = [0.75, 0.75, 0.8, 1.0];
            let hot = [0.95, 0.6, 0.2, 1.0];
            let cool = [0.4, 0.8, 0.95, 1.0];
            let peg = ColliderShape::Ball { radius: 0.08 };
            let plank = ColliderShape::Cuboid {
                half_x: 0.6,
                half_y: 0.1,
            };

            // ── O BRAÇO servo, e ao lado dele o MESMO braço sem motor.
            body(
                "Servo Hub",
                BodyKind::Static,
                peg,
                [0.16, 0.16],
                grey,
                [-5.5, 7.0],
            );
            body(
                "Servo Arm",
                BodyKind::Dynamic,
                plank,
                [1.2, 0.2],
                hot,
                [-4.9, 7.0],
            );
            body(
                "Limp Hub",
                BodyKind::Static,
                peg,
                [0.16, 0.16],
                grey,
                [-2.8, 7.0],
            );
            body(
                "Limp Arm",
                BodyKind::Dynamic,
                plank,
                [1.2, 0.2],
                grey,
                [-2.2, 7.0],
            );

            // ── O ELEVADOR: trilho vertical + cabine dirigida por TAXA.
            body(
                "Shaft",
                BodyKind::Static,
                ColliderShape::Cuboid {
                    half_x: 0.08,
                    half_y: 1.4,
                },
                [0.16, 2.8],
                grey,
                [0.5, 6.5],
            );
            body(
                "Cabin",
                BodyKind::Dynamic,
                ColliderShape::Cuboid {
                    half_x: 0.35,
                    half_y: 0.35,
                },
                [0.7, 0.7],
                hot,
                [0.5, 5.5],
            );

            // ── O GUINCHO: gancho + carga numa corda de 2 m.
            body(
                "Crane",
                BodyKind::Static,
                peg,
                [0.16, 0.16],
                grey,
                [3.8, 8.0],
            );
            body(
                "Load",
                BodyKind::Dynamic,
                ColliderShape::Ball { radius: 0.3 },
                [0.6, 0.6],
                cool,
                [3.8, 6.0],
            );

            // ── O par para ARMAR à mão (nenhuma joint entre eles ainda).
            body(
                "Bare Hub",
                BodyKind::Static,
                peg,
                [0.16, 0.16],
                grey,
                [6.6, 7.0],
            );
            body(
                "Bare Arm",
                BodyKind::Dynamic,
                plank,
                [1.2, 0.2],
                cool,
                [6.6, 6.4],
            );
        }

        // As joints. Cada uma responde uma pergunta diferente sobre o motor.
        // ⚠️ `rot` não é decoração: num Slider a rotação da entidade-joint É o
        // eixo do trilho (W-J5), então a mesma porta que coloca a âncora coloca a
        // direção. Os outros tipos a ignoram.
        let mut joint = |name: &str, a: &str, b: &str, at: [f32; 2], rot: f32, j: PhysicsJoint| {
            let mut t = Transform::from_translation(Vec2::new(at[0], at[1]));
            t.rotation = rot;
            world.spawn((
                Name::new(name.to_string()),
                PhysicsJoint {
                    body_a: stable_name_id(a),
                    body_b: stable_name_id(b),
                    ..j
                },
                t,
            ));
        };
        // O servo: MIRA um lugar (+50°) e o segura contra o peso do braço.
        joint(
            "Servo Pin",
            "Servo Hub",
            "Servo Arm",
            [-5.5, 7.0],
            0.0,
            PhysicsJoint {
                kind: JointKind::Pin,
                motor_enabled: true,
                motor_mode: MotorMode::Position,
                motor_target: 50.0_f32.to_radians(),
                ..PhysicsJoint::default()
            },
        );
        // O controle: a MESMA dobradiça, sem motor. Sem ela, "o braço está a 50
        // graus" seria satisfeito por uma armação onde a gravidade não chega.
        joint(
            "Limp Pin",
            "Limp Hub",
            "Limp Arm",
            [-2.8, 7.0],
            0.0,
            PhysicsJoint {
                kind: JointKind::Pin,
                ..PhysicsJoint::default()
            },
        );
        // O elevador: TAXA, num trilho vertical (a rotação do joint É o eixo).
        joint(
            "Shaft Rail",
            "Shaft",
            "Cabin",
            [0.5, 5.5],
            std::f32::consts::FRAC_PI_2,
            PhysicsJoint {
                kind: JointKind::Slider,
                limits_enabled: true,
                limit_min: -0.1,
                limit_max: 2.0,
                motor_enabled: true,
                motor_mode: MotorMode::Velocity,
                motor_speed: PhysicsJoint::DEFAULT_LINEAR_MOTOR_SPEED,
                ..PhysicsJoint::default()
            },
        );
        // O guincho: uma corda de 2 m recolhida a 0,5 m — e ela SEGURA a carga
        // lá em cima, que é a diferença entre um servo e um empurrão.
        joint(
            "Winch",
            "Crane",
            "Load",
            [3.8, 8.0],
            0.0,
            PhysicsJoint {
                kind: JointKind::Rope,
                max_length: 2.0,
                motor_enabled: true,
                motor_mode: MotorMode::Position,
                motor_target: 0.5,
                ..PhysicsJoint::default()
            },
        );

        eprintln!(
            "[physics-smoke 48] O SERVO E O GUINCHO -- o motor ganha um MODO, e\n\
             passa a existir no Slider e na Rope (W-J6). PAUSADA.\n  \
               1. Play. Medido (headless, sobre esta mesma armacao):\n       \
                  - BRACO LARANJA (esquerda): sobe do repouso e PARA a **49,80 graus**\n         \
                    -- pediram 50, e os 0,20 que faltam sao a flexao do servo sob o\n         \
                    peso do braco. Ele SEGURA: tick 120 e tick 300 leem o mesmo\n         \
                    angulo. Um servo mira um LUGAR.\n       \
                  - BRACO CINZA (ao lado): o CONTROLE. Mesma dobradica, sem motor --\n         \
                    cai e fica balancando (nao ha atrito para para-lo). Se os dois\n         \
                    ficarem iguais, o motor nao chegou ao solver.\n       \
                  - CABINE (laranja, meio): SOBE a 0,49 m/s pelo trilho vertical e\n         \
                    ESTACIONA no fim do curso, em y = 7,500 (2,0 m acima da ancora).\n         \
                    E o motor LINEAR -- o Slider nao tinha nenhum ate esta wave.\n       \
                  - CARGA (ciano, direita): a corda de 2 m e RECOLHIDA e a carga fica\n         \
                    pendurada em y = 7,499, meio metro abaixo do guincho. Ela nao\n         \
                    desce: o servo segura o peso.\n  \
               2. Rebobine e selecione 'Servo Pin' na Hierarquia. A secao Physics\n     \
                  Joint mostra o card **Motor** com uma linha nova: **Mode**\n     \
                  (Velocity | Position). Em Position a row diz **Target (graus)**;\n     \
                  clique **Velocity** e ela vira **Speed (graus/s)** -- cada modo\n     \
                  mostra a SUA instrucao, nunca as duas.\n  \
               3. Selecione 'Winch'. O MESMO card, e as rows dizem **(m)** e **(m/s)**:\n     \
                  a unidade segue o grau de liberdade livre. Repare que a Rope NAO\n     \
                  tem rows de limite -- por isso a unidade do MOTOR e uma pergunta\n     \
                  separada da unidade dos LIMITES.\n  \
               4. Mude o Target do 'Winch' para 1.5 e de Play: a carga desce e para\n     \
                  a 1,5 m do gancho. O guincho SEGURA -- nao e um empurrao. Ponha\n     \
                  o Mode em **Velocity** e a Speed nasce NEGATIVA (-0,5 m/s): num\n     \
                  guincho o sinal e a direcao, e a corda ja esta toda solta, entao\n     \
                  a unica direcao com para onde ir e RECOLHER. Digite +0,5 e nada\n     \
                  se move -- o limite da propria corda ja proibe soltar mais.\n  \
               5. ARME UM A MAO (a estrela): selecione 'Bare Hub' e 'Bare Arm', aperte\n     \
                  **Join Selected Bodies** (Join As = Pin). A joint nova ja nasce\n     \
                  selecionada: ligue **Motor**, escolha **Position**, digite 90 no\n     \
                  Target e de Play -- o braco sobe ate a vertical e fica.\n  \
               6. Troque o Kind dessa joint nova para **Rope**: as rows do motor\n     \
                  passam a dizer (m)/(m/s) e os numeros sao RE-SEMEADOS. Sem isso os\n     \
                  90 graus (1,57 rad) virariam 1,57 METROS de guincho -- um numero\n     \
                  que ninguem digitou.\n  \
               7. **O TRILHO SE MIRA PELAS ALCAS** (W-J6c): PAUSE primeiro -- alca de\n     \
                  ponto so existe em REPOUSO (tocando, o overlay desenha a geometria\n     \
                  do SOLVER, e ela autora a AUTORADA). O contorno ja esta ligado (B o\n     \
                  alterna). Entao selecione\n     \
                  'Shaft Rail' e repare nas DUAS alcas sobre a reta, nos fins de\n     \
                  curso. Elas sao livres em x e y: arraste uma para o lado e o\n     \
                  trilho INTEIRO gira para apontar nela, com a outra ponta viajando\n     \
                  junto. A reta pela ancora e a ponta arrastada E o eixo, e a\n     \
                  distancia ate ela e aquele fim de curso -- duas alcas dizem tudo\n     \
                  que um trilho e, sem digitar angulo nenhum.\n  \
               8. Um Spring e um Weld NAO oferecem o card Motor, e nao e gosto: o\n     \
                  rapier modela mola COMO motor no mesmo eixo, entao um segundo ali\n     \
                  comeria a rigidez que voce autorou."
        );
    }
}
