//! **A cena do JOINT DESCRITO POR EIXO** (`PH2D_PHYSICS_SMOKE=79`,
//! W-JointCustom).
//!
//! O kit tinha oito presets, e cada um é uma configuração FIXA de eixos: um Pin
//! trava os dois lineares e deixa o angular, um Slider trava `LinY` e o angular,
//! uma solda trava os três. O que não havia era dizer a configuração — e as
//! combinações que faltavam não são exóticas: *deslizar entre batentes E girar*
//! não é um Slider (ele proíbe o giro) nem um Pin (ele proíbe o deslize).
//!
//! ⚠️ **O Custom não substitui os presets, e a cena não pede que ele o faça.**
//! Um Pin diz *"isto é uma dobradiça"* — e é essa frase que o glifo desenha, que
//! o FK lê para posar, que a árvore de IK lê para montar. Um Custom com a mesma
//! máscara é a mesma restrição e uma afirmação mais fraca.
//!
//! Os números abaixo saíram da sonda `probe_smoke_79`, rodada ANTES desta
//! mensagem ser escrita.

use crate::physics_smoke::spawn_floor;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{
    AxisMode, AxisSpec, BodyKind, Collider, ColliderShape, CustomAxis, JointKind, MotorMode,
    PhysicsJoint, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const GREY: [f32; 4] = [0.75, 0.75, 0.8, 1.0];
const HOT: [f32; 4] = [0.95, 0.6, 0.2, 1.0];
const COOL: [f32; 4] = [0.4, 0.8, 0.95, 1.0];
const DEAD: [f32; 4] = [0.45, 0.46, 0.5, 1.0];

/// ⚠️ **A calha (a do meio) fica ALTA de propósito:** o piso desta cena mora em
/// `y = -1,0`, e um bloco que cai 1,5 m a partir de `y = 0,2` bateria no chão
/// antes do batente — a cena afirmaria o batente e estaria medindo o piso.
pub(crate) const LANE_Y: [f32; 3] = [5.0, 1.6, -2.4];
/// O curso dos batentes desta cena, metros.
pub(crate) const STROKE: f32 = 1.1;

fn body(
    world: &mut World,
    name: &str,
    kind: BodyKind,
    shape: ColliderShape,
    size: [f32; 2],
    rgba: [f32; 4],
    at: [f32; 2],
) {
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
}

fn joint(world: &mut World, name: &str, a: &str, b: &str, at: [f32; 2], j: PhysicsJoint) -> Entity {
    world
        .spawn((
            Name::new(name.to_string()),
            PhysicsJoint {
                body_a: stable_name_id(a),
                body_b: stable_name_id(b),
                ..j
            },
            Transform::from_translation(Vec2::new(at[0], at[1])),
        ))
        .id()
}

/// Um motor de velocidade no eixo `axis`.
fn driven(axis: CustomAxis, speed: f32) -> PhysicsJoint {
    let mut j = PhysicsJoint {
        kind: JointKind::Custom,
        motor_enabled: true,
        motor_mode: MotorMode::Velocity,
        motor_speed: speed,
        motor_max_force: 300.0,
        ..PhysicsJoint::default()
    };
    j.custom.motor_axis = axis;
    j
}

pub(crate) fn build_joint_custom_scene(world: &mut World) {
    spawn_floor(world);
    let peg = ColliderShape::Ball { radius: 0.08 };
    let block = ColliderShape::Cuboid {
        half_x: 0.28,
        half_y: 0.28,
    };
    let bar = ColliderShape::Cuboid {
        half_x: 0.6,
        half_y: 0.09,
    };

    // ── 1. O CARRINHO QUE GIRA: desliza entre batentes E gira livre.
    //    Nenhum preset diz isto — um Slider proíbe o giro.
    let y = LANE_Y[0];
    body(
        world,
        "RailPost",
        BodyKind::Static,
        peg,
        [0.16, 0.16],
        GREY,
        [-3.4, y],
    );
    // ⚠️ **Ancorada na PONTA ESQUERDA**, não no centro: com o eixo de rotação
    // livre, é a gravidade sobre o braço que a faz girar. Ancorada no centro ela
    // ficaria livre para girar e sem NADA que a girasse — a cena afirmaria uma
    // liberdade que ninguém exercita.
    body(
        world,
        "RailCart",
        BodyKind::Dynamic,
        bar,
        [1.2, 0.18],
        HOT,
        [-2.8, y],
    );
    let mut j = driven(CustomAxis::X, 1.4);
    *j.custom.axis_mut(CustomAxis::X) = AxisSpec {
        mode: AxisMode::Limited,
        min: -STROKE,
        max: STROKE,
    };
    j.custom.axis_mut(CustomAxis::Y).mode = AxisMode::Locked;
    j.custom.axis_mut(CustomAxis::Rotation).mode = AxisMode::Free;
    joint(world, "RailSpin", "RailPost", "RailCart", [-3.4, y], j);

    // O CONTROLE: um SLIDER com o mesmo curso e o mesmo motor. Ele desliza
    // igual e **nunca gira** — é ele que mostra o que o Custom acrescenta.
    body(
        world,
        "CtrlPost",
        BodyKind::Static,
        peg,
        [0.16, 0.16],
        GREY,
        [1.2, y],
    );
    body(
        world,
        "CtrlCart",
        BodyKind::Dynamic,
        bar,
        [1.2, 0.18],
        DEAD,
        [1.8, y],
    );
    joint(
        world,
        "CtrlSlider",
        "CtrlPost",
        "CtrlCart",
        [1.2, y],
        PhysicsJoint {
            kind: JointKind::Slider,
            limits_enabled: true,
            limit_min: -STROKE,
            limit_max: STROKE,
            motor_enabled: true,
            motor_mode: MotorMode::Velocity,
            motor_speed: 1.4,
            motor_max_force: 300.0,
            ..PhysicsJoint::default()
        },
    );

    // ── 2. A CALHA VERTICAL: X travado, Y limitado, giro livre. O bloco CAI até
    //    o batente de baixo e balança pendurado ali.
    let y = LANE_Y[1];
    body(
        world,
        "SlotPost",
        BodyKind::Static,
        peg,
        [0.16, 0.16],
        GREY,
        [-3.4, y],
    );
    body(
        world,
        "SlotBlock",
        BodyKind::Dynamic,
        block,
        [0.56, 0.56],
        COOL,
        [-3.4, y],
    );
    let mut j = PhysicsJoint {
        kind: JointKind::Custom,
        ..PhysicsJoint::default()
    };
    j.custom.axis_mut(CustomAxis::X).mode = AxisMode::Locked;
    *j.custom.axis_mut(CustomAxis::Y) = AxisSpec {
        mode: AxisMode::Limited,
        min: -1.5,
        max: 0.0,
    };
    j.custom.axis_mut(CustomAxis::Rotation).mode = AxisMode::Free;
    joint(world, "Slot", "SlotPost", "SlotBlock", [-3.4, y], j);

    // ── 3. O EIXO DO MOTOR É ESCOLHIDO: dois joints de configuração IDÊNTICA
    //    (tudo livre), o mesmo motor, e só o eixo difere. Um GIRA, o outro
    //    DESLIZA — e é isso que torna *"o motor dirige o primeiro eixo livre"*
    //    inexprimível.
    let y = LANE_Y[2];
    body(
        world,
        "SpinPost",
        BodyKind::Static,
        peg,
        [0.16, 0.16],
        GREY,
        [-3.4, y],
    );
    body(
        world,
        "SpinBar",
        BodyKind::Dynamic,
        bar,
        [1.2, 0.18],
        HOT,
        [-3.4, y],
    );
    joint(
        world,
        "MotorOnRotation",
        "SpinPost",
        "SpinBar",
        [-3.4, y],
        {
            // ⚠️ **Y TRAVADO nos dois, e não é decoração:** uma máscara VAZIA é
            // ausência de restrição — o rapier não prende nada, e as duas barras
            // caem em queda livre com o motor girando no ar. A cena mediu 122 m
            // de queda antes desta linha. Travar um eixo é o que faz um Custom
            // ser um joint.
            let mut j = driven(CustomAxis::Rotation, 3.0);
            j.custom.axis_mut(CustomAxis::Y).mode = AxisMode::Locked;
            // ⚠️ **O batente de X entra nos DOIS**, e é o que mantém a
            // configuração IDÊNTICA: sem ele a barra deslizante percorre 15 m em
            // 5 s e sai de quadro, e a bancada que existe para mostrar *só o
            // eixo difere* passaria a mostrar duas cenas diferentes.
            *j.custom.axis_mut(CustomAxis::X) = AxisSpec {
                mode: AxisMode::Limited,
                min: -2.0,
                max: 2.0,
            };
            j
        },
    );
    body(
        world,
        "SlidePost",
        BodyKind::Static,
        peg,
        [0.16, 0.16],
        GREY,
        [1.2, y],
    );
    body(
        world,
        "SlideBar",
        BodyKind::Dynamic,
        bar,
        [1.2, 0.18],
        COOL,
        [1.2, y],
    );
    joint(world, "MotorOnX", "SlidePost", "SlideBar", [1.2, y], {
        let mut j = driven(CustomAxis::X, 3.0);
        j.custom.axis_mut(CustomAxis::Y).mode = AxisMode::Locked;
        *j.custom.axis_mut(CustomAxis::X) = AxisSpec {
            mode: AxisMode::Limited,
            min: -2.0,
            max: 2.0,
        };
        j
    });
}

#[cfg(test)]
#[path = "physics_smoke_joint_custom_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 79 (W-JointCustom).** O joint que o artista descreve por eixo.
    pub(crate) fn physics_smoke_joint_custom(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_joint_custom_scene(gfx.sim.world_mut());
        gfx.camera.center = [-1.0, 0.0];
        gfx.camera.height_world = 14.0;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 79] O JOINT DESCRITO POR EIXO -- a configuracao e' AUTORADA.\n\n  \
               O kit tinha oito presets, e cada um e' uma configuracao FIXA: um Pin\n  \
               trava os dois eixos lineares e deixa o angular, um Slider trava Y e o\n  \
               angular, uma solda trava os tres. O Custom deixa voce dizer -- cada grau\n  \
               de liberdade e' **Free / Limited / Locked**, e o motor diz em QUAL deles\n  \
               age.\n\n  \
               Tres bancadas. **Aperte Espaco** e assista:\n     \
                  - EM CIMA, o CARRINHO QUE GIRA (laranja): X limitado a +-{stroke:.1} m,\n       \
                    Y travado, rotacao LIVRE. Ele vai e volta entre os batentes E gira.\n       \
                    Ao lado, em CINZA, o CONTROLE: um SLIDER com o mesmo curso e o\n       \
                    mesmo motor. Medido em 5 s: os dois percorrem 1,1 m, o Slider\n       \
                    gira **0,00 rad** e o Custom **13,72**. Essa diferenca e' a wave\n       \
                    inteira -- nenhum preset diz *deslize entre batentes E gire*.\n     \
                  - NO MEIO, a CALHA VERTICAL (azul): X travado, Y limitado a -1,5 m,\n       \
                    rotacao livre. O bloco CAI 1,50 m ate' o batente de baixo e fica\n       \
                    pendurado ali balancando.\n     \
                  - EMBAIXO, O EIXO DO MOTOR E' ESCOLHIDO: dois joints de configuracao\n       \
                    IDENTICA (tudo livre) e o MESMO motor. O da esquerda tem o motor no\n       \
                    eixo de ROTACAO e gira; o da direita tem no eixo X e desliza. Medido:\n       \
                    **15,00 rad de giro contra 0,00**, e **2,00 m de curso contra\n       \
                    0,00**. Configuracao identica, so' o eixo do motor difere.\n\n  \
               (!) AUTORE UM: selecione dois corpos, escolha **Custom** em 'Join As' na\n      \
                   secao Physics Body e aperte 'Join Selected Bodies'. A secao Physics\n      \
                   Joint mostra tres linhas -- X, Y, Rotation --, cada uma com\n      \
                   Free/Limited/Locked, e o par Min/Max aparece **so'** no modo Limited\n      \
                   (um batente num eixo travado e' um numero que o solver nao le).\n\n  \
               (!) E OLHE A UNIDADE. Com o motor armado aparece 'Motor Axis'. Ponha-o em\n      \
                   Rotation e as rows do motor dizem graus; ponha em X ou Y e elas dizem\n      \
                   metros. E' a mesma pergunta que num Pin e' respondida pelo TIPO e aqui\n      \
                   pelo EIXO -- rotular errado faria voce digitar 90 e a peca andar 1,57 m.\n\n  \
               (!) Aperte **B**: o glifo do Custom e' um HEXAGONO (nem o anel do pino,\n      \
                   nem o quadrado da solda), com um anel dentro quando o eixo de rotacao\n      \
                   esta' livre.\n",
            stroke = STROKE,
        );
    }
}
