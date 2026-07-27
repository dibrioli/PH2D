//! **A cena do CARRO** (`PH2D_PHYSICS_SMOKE=57`, W-Wheel).
//!
//! Uma roda é o primeiro joint do kit que deixa **dois** graus de liberdade
//! livres, e a única forma de mostrar isso é ao lado do que se tem sem ela: dois
//! veículos IDÊNTICOS atravessam a MESMA pista de lombadas, e a única diferença
//! é como as rodas estão presas.
//!
//! - **VERDE**: rodas por `Wheel` — cubo que gira **e** cavalga uma suspensão.
//! - **LARANJA** (o controle): rodas por `Pin` — o eixo rígido, que é tudo o que
//!   este kit sabia fazer antes desta wave. Ele gira igual; o que ele não tem é
//!   para onde ceder.
//!
//! Os dois carregam o MESMO motor, então a comparação é sobre a suspensão e
//! nada mais. Os números da mensagem saem da sonda `probe_smoke_57`, rodada
//! sobre ESTAS peças antes de a mensagem ser escrita.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const GROUND: [f32; 4] = [0.42, 0.42, 0.46, 1.0];
const BUMP: [f32; 4] = [0.62, 0.55, 0.42, 1.0];
const SPRUNG: [f32; 4] = [0.45, 0.9, 0.45, 1.0];
const RIGID: [f32; 4] = [0.95, 0.62, 0.25, 1.0];
const TYRE: [f32; 4] = [0.22, 0.22, 0.26, 1.0];

/// **O curso que a suspensão do VERDE de fato percorre** ao cruzar a pista, em
/// metros — a distância que o cubo sobe e desce DENTRO do chassi.
///
/// ⚠️ Os `MEASURED_*` saem da sonda `probe_smoke_57` e existem para a mensagem
/// afirmar NÚMEROS em vez de adjetivos — com um gate
/// (`the_scene_message_states_the_numbers_the_sim_produces`) provando que eles
/// continuam sendo o que a simulação produz.
///
/// ⚠️ **E a escolha DESTE número em vez do "balanço" custou duas medições.** A
/// primeira versão da cena afirmava *"o chassi suspenso passa nivelado"* e a
/// sonda mediu o contrário (23,4° contra 14,5° do eixo rígido) — corretamente,
/// porque duas suspensões independentes deixam uma comprimir enquanto a outra
/// estende, que é o **mergulho** de um carro de verdade. A segunda tentativa
/// (o SOLAVANCO do chassi) varreu cinco alturas de lombada e a melhor separação
/// foi **1,35×** — real, e pequena demais para se ver sem cronômetro. O que
/// separa por três ordens de grandeza é o próprio curso, que também é
/// literalmente a feature: **0,150 m contra 0,000**.
pub(crate) const MEASURED_TRAVEL_M: f32 = 0.150;
/// Idem para o carro LARANJA — o controle. Um Pin não tem para onde ceder, e o
/// que sobra é o erro do solver.
pub(crate) const MEASURED_RIGID_TRAVEL_M: f32 = 0.000;
/// Quanto o VERDE anda em 4 s de pista. Roda que fica no chão tem tração.
pub(crate) const MEASURED_SPRUNG_DIST_M: f32 = 6.85;
/// Idem para o LARANJA. ⚠️ A diferença é **2%** — real e medida, mas nomeada
/// como o que é: não dá para ver, e vendê-la como o resultado da cena seria
/// pedir ao artista que acreditasse num número que a tela não mostra.
pub(crate) const MEASURED_RIGID_DIST_M: f32 = 6.71;

/// Altura do chão da pista.
const GROUND_Y: f32 = 0.0;
/// Raio da roda.
const WHEEL_R: f32 = 0.3;
/// Altura de marcha AUTORADA — o quanto o centro do chassi fica acima do cubo.
const RIDE: f32 = 0.5;
/// Meia-largura do chassi; as rodas ficam nas quinas.
const HALF_X: f32 = 0.9;
/// Onde os dois carros começam, em x.
const START_X: f32 = -9.0;
/// Rotação do joint = a direção da suspensão. Para CIMA.
const UP: f32 = std::f32::consts::FRAC_PI_2;

#[allow(clippy::too_many_arguments)]
fn cuboid(
    world: &mut World,
    name: &str,
    x: f32,
    y: f32,
    hx: f32,
    hy: f32,
    kind: BodyKind,
    tint: [f32; 4],
) {
    world.spawn((
        Transform::from_translation(Vec2::new(x, y)),
        Sprite::atlas(WHITE_TILE_KEY, [hx * 2.0, hy * 2.0], tint),
        Name::new(name),
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: hx,
                half_y: hy,
            },
            density: 1.0,
            friction: 1.0,
            ..Collider::default()
        },
    ));
}

fn wheel_body(world: &mut World, name: &str, x: f32, y: f32) {
    world.spawn((
        Transform::from_translation(Vec2::new(x, y)),
        Sprite::atlas(WHITE_TILE_KEY, [WHEEL_R * 2.0, WHEEL_R * 2.0], TYRE),
        Name::new(name),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: WHEEL_R },
            density: 1.0,
            friction: 1.0,
            ..Collider::default()
        },
    ));
}

/// Um veículo: chassi + duas rodas presas pelo `kind` pedido.
///
/// ⚠️ **A ÚNICA diferença entre os dois carros é esse `kind`** — mesma massa,
/// mesmas rodas, mesmo motor, mesma pista. Uma comparação em que duas coisas
/// mudam não mede nenhuma das duas.
fn car(world: &mut World, label: &str, y_offset: f32, kind: JointKind, tint: [f32; 4]) {
    let hub_y = GROUND_Y + WHEEL_R + y_offset;
    cuboid(
        world,
        &format!("{label} Chassis"),
        START_X,
        hub_y + RIDE,
        HALF_X,
        0.2,
        BodyKind::Dynamic,
        tint,
    );
    for (side, dx) in [("F", HALF_X * 0.8), ("R", -HALF_X * 0.8)] {
        let hub_x = START_X + dx;
        wheel_body(world, &format!("{label} {side}"), hub_x, hub_y);
        world.spawn((
            Transform {
                translation: Vec2::new(hub_x, hub_y),
                // A rotação do joint É a direção da suspensão (o eixo do
                // Slider/Wheel mora no `Transform` da entidade-joint). Um Pin a
                // ignora, o que é justamente o ponto do controle.
                rotation: UP,
                ..Transform::IDENTITY
            },
            Name::new(format!("{label} {side} Joint")),
            PhysicsJoint {
                body_a: stable_name_id(&format!("{label} Chassis")),
                body_b: stable_name_id(&format!("{label} {side}")),
                kind,
                // O curso ARMADO: o batente de compressão e o de extensão. Um
                // Pin não tem faixa em metros, e `of_kind` já semeia a dele em
                // radianos — por isso o `limits_enabled` só vale para a roda.
                limits_enabled: kind == JointKind::Wheel,
                // TRAÇÃO: o motor é o do GIRO nos dois tipos (o Pin tem
                // dobradiça, a roda tem o eixo angular), então os dois andam.
                motor_enabled: true,
                motor_speed: -6.0,
                motor_max_force: 40.0,
                ..PhysicsJoint::of_kind(kind)
            },
        ));
    }
}

/// Meia-altura das lombadas, metros.
///
/// ⚠️ **MEDIDO, e o primeiro valor (0,12) não discriminava nada.** Uma suspensão
/// só absorve o degrau que **cabe no curso** dela: com afundamento estático de
/// ~0,06 m e curso `±WHEEL_TRAVEL` (0,15), sobram ~0,09 m de compressão — uma
/// lombada de 0,24 m de altura estoura o batente e o chassi leva o resto do
/// degrau exatamente como o de eixo rígido (medido: solavanco 0,0151 contra
/// 0,0150 — parid­ade). Ver a tabela em `probe_smoke_57_bump_sweep`.
pub(crate) const BUMP_HALF_Y: f32 = 0.05;

/// A pista + os dois carros. `pub(crate)` porque a sonda monta as MESMAS peças
/// que o artista abre — uma sonda com cena própria mede outra coisa.
pub(crate) fn spawn_props(world: &mut World) {
    spawn_props_with(world, BUMP_HALF_Y);
}

/// [`spawn_props`] com a altura da lombada aberta, **e só para a varredura que
/// escolheu [`BUMP_HALF_Y`]** — o produto entra sempre pela porta acima.
pub(crate) fn spawn_props_with(world: &mut World, bump_half_y: f32) {
    // A pista, e uma para cada carro: pistas separadas em altura para os dois
    // rodarem lado a lado sem se tocarem (dois carros na MESMA pista viram uma
    // corrida, e uma corrida mede quem largou melhor).
    for (label, y) in [("Sprung", 0.0_f32), ("Rigid", 4.0)] {
        cuboid(
            world,
            &format!("{label} Ground"),
            0.0,
            GROUND_Y - 0.25 + y,
            14.0,
            0.25,
            BodyKind::Static,
            GROUND,
        );
        // As lombadas: baixas o bastante para um carro passar, altas o bastante
        // para um eixo rígido sentir. Espaçadas irregularmente de propósito —
        // espaçamento regular casaria com a frequência da suspensão e o
        // resultado mediria a ressonância, não o amortecimento.
        for (i, bx) in [-4.0_f32, -1.2, 1.6, 5.0].into_iter().enumerate() {
            cuboid(
                world,
                &format!("{label} Bump {i}"),
                bx,
                GROUND_Y + bump_half_y + y,
                0.35,
                bump_half_y,
                BodyKind::Static,
                BUMP,
            );
        }
    }
    car(world, "Sprung", 0.0, JointKind::Wheel, SPRUNG);
    car(world, "Rigid", 4.0, JointKind::Pin, RIGID);
}

impl crate::App {
    /// **Cena 57 (W-Wheel).** Dois carros idênticos, uma pista de lombadas, e a
    /// única diferença é como as rodas estão presas.
    pub(crate) fn physics_smoke_wheel(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_props(gfx.sim.world_mut());
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 57] A RODA -- o cubo que gira E cavalga uma suspensao.\n  \
               Aperte B para ver os joints, e depois PLAY (o toggle Physics ja esta armado).\n\n  \
               1. OLHE AS RODAS, nao os carros. Mesma massa, mesmas rodas, mesmo motor,\n     \
                  mesma pista -- a UNICA diferenca e o tipo de joint:\n     \
                  - VERDE (embaixo): rodas por 'Wheel'. Cada cubo SOBE E DESCE dentro do\n       \
                    carro ao passar nas lombadas -- e isso que uma suspensao e.\n     \
                  - LARANJA (em cima): rodas por 'Pin'. Elas giram igual e NAO se movem\n       \
                    em relacao ao carro: eixo rigido, que era tudo o que este kit tinha.\n     \
                  (medido, curso percorrido pelo cubo dentro do chassi ao longo da pista:\n      \
                  verde {travel:.3} m, laranja {rigid_travel:.3} m)\n  \
               2. E o carro suspenso MERGULHA nas lombadas, em vez de subir inteiro. Isso\n     \
                  e um carro de verdade, nao um defeito: com duas suspensoes independentes\n     \
                  uma comprime enquanto a outra estende. (Ele tambem anda 2% mais longe --\n     \
                  {sprung_d:.2} m contra {rigid_d:.2} -- porque a roda fica no chao; e real,\n     \
                  medido, e pequeno demais para se ver. Nao e o ponto da cena.)\n  \
               3. Selecione uma roda verde ('Sprung F Joint') na Hierarquia. A secao do\n     \
                  joint mostra as TRES coisas que so uma roda tem juntas:\n     \
                  - 'Travel' + Min/Max em METROS: os batentes da suspensao. Ponha Max\n       \
                    em 0.02 e de Play -- o curso some e o verde vira o laranja.\n     \
                  - 'Stiffness' e 'Damping': a mola. Ponha Stiffness em 60 e de Play --\n       \
                    o carro senta no batente. Volte para 400.\n     \
                  - 'Motor' em GRAUS por segundo: a tracao. Note a unidade -- o limite\n       \
                    esta em METROS e o motor em GRAUS no MESMO joint, porque sao dois\n       \
                    graus de liberdade diferentes.\n  \
               4. O DESENHO diz o tipo: um anel (o cubo, que gira) MAIS um zigue-zague\n     \
                  ao longo do eixo (a suspensao) -- nem so o anel de um pino, nem so o\n     \
                  zigue-zague de uma mola.\n  \
               5. E o gesto de CRIAR: selecione um chassi e uma roda, va na secao\n     \
                  Physics Body, escolha 'Wheel' no seletor 'Join As' e clique em Join.",
            travel = MEASURED_TRAVEL_M,
            rigid_travel = MEASURED_RIGID_TRAVEL_M,
            sprung_d = MEASURED_SPRUNG_DIST_M,
            rigid_d = MEASURED_RIGID_DIST_M,
        );
    }
}

#[cfg(test)]
#[path = "physics_smoke_wheel_tests.rs"]
mod tests;
