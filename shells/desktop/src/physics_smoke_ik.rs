//! **A cena da POSE por cinemática inversa** (`PH2D_PHYSICS_SMOKE=54`, W-IK).
//!
//! A cena 53 é sobre o que o ponteiro faz numa cena RODANDO. Esta é o contrário:
//! ela abre **PAUSADA**, porque posar é autorar — o resultado é `Transform`, e
//! com o relógio andando o readback o sobrescreveria no mesmo frame.
//!
//! Três estações, cada uma para uma pergunta que a wave teve de responder:
//!
//! - **O BRAÇO** (esquerda): três elos pendurados num ombro ESTÁTICO. É a
//!   pergunta central — arrastar a mão dobra o cotovelo. A raiz é o ombro
//!   porque a cena diz que ele é estático, não porque alguém a escolheu.
//! - **A PERNA** (centro): dois elos com o joelho **LIMITADO**. É a pergunta que
//!   o rapier respondia errado: o `inverse_kinematics` dele ignora limites, e sem
//!   a projeção o joelho dobra para trás — uma pose que o Play desfaz no 1º tick.
//! - **A COBRA** (direita): quatro elos e **nenhuma âncora**. A raiz é a cauda (o
//!   elo mais distante da ponta) e ela ganha 3 graus de liberdade, então a IK
//!   pode transladar o conjunto — que é o que faz sentido num rig solto.
//!
//! Os números da mensagem saem da sonda `probe_smoke_54`, rodada sobre ESTAS
//! peças antes de a mensagem ser escrita — a regra que esta linha adotou depois
//! de duas cenas afirmarem coisas que a medição desmentiu.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const BONE: [f32; 4] = [0.85, 0.82, 0.72, 1.0];
const LIMB: [f32; 4] = [0.55, 0.9, 0.35, 1.0];
const KNEE: [f32; 4] = [0.95, 0.6, 0.2, 1.0];
const SNAKE: [f32; 4] = [0.4, 0.8, 0.95, 1.0];

/// Um elo: caixa de `len × 0,2` centrada em `(x, y)`, com nome (um corpo sem
/// nome é um corpo que um joint não aponta).
fn link(world: &mut World, name: &str, x: f32, y: f32, len: f32, kind: BodyKind, tint: [f32; 4]) {
    world.spawn((
        Transform::from_translation(Vec2::new(x, y)),
        Sprite::atlas(WHITE_TILE_KEY, [len, 0.2], tint),
        Name::new(name),
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: len * 0.5,
                half_y: 0.1,
            },
            ..Collider::default()
        },
    ));
}

/// Um Pin entre dois corpos, ancorado no ponto de MUNDO `at`. Com `limits`, um
/// joelho que só dobra para um lado.
fn pin(world: &mut World, name: &str, a: &str, b: &str, at: [f32; 2], limits: Option<[f32; 2]>) {
    let mut j = PhysicsJoint {
        body_a: stable_name_id(a),
        body_b: stable_name_id(b),
        kind: JointKind::Pin,
        ..PhysicsJoint::of_kind(JointKind::Pin)
    };
    if let Some([min, max]) = limits {
        j.limits_enabled = true;
        j.limit_min = min;
        j.limit_max = max;
    }
    world.spawn((
        Transform::from_translation(Vec2::new(at[0], at[1])),
        Name::new(name),
        j,
    ));
}

/// As três estações. A MESMA construção que a sonda headless mede.
///
/// ⚠️ **Sem chão**, e é decisão: a cena abre PAUSADA e nada cai, então um chão
/// só acrescentaria um corpo que o artista pode pegar por engano. A cobra
/// flutuando é a demonstração da raiz livre, não um descuido.
pub(crate) fn spawn_props(world: &mut World) {
    // ── O BRAÇO: ombro ESTÁTICO + três elos de 1 m, deitados em +X.
    link(world, "Shoulder", -7.0, 2.5, 0.4, BodyKind::Static, BONE);
    for (i, name) in ["UpperArm", "Forearm", "Hand"].iter().enumerate() {
        link(
            world,
            name,
            -6.5 + i as f32,
            2.5,
            1.0,
            BodyKind::Dynamic,
            LIMB,
        );
    }
    pin(
        world,
        "J.Shoulder",
        "Shoulder",
        "UpperArm",
        [-7.0, 2.5],
        None,
    );
    pin(world, "J.Elbow", "UpperArm", "Forearm", [-6.0, 2.5], None);
    pin(world, "J.Wrist", "Forearm", "Hand", [-5.0, 2.5], None);

    // ── A PERNA: quadril estático + coxa + canela, com o JOELHO limitado a
    // dobrar só para um lado (0..2 rad ≈ 0..115°).
    link(world, "Hip", -1.0, 2.5, 0.4, BodyKind::Static, BONE);
    link(world, "Thigh", -0.5, 2.5, 1.0, BodyKind::Dynamic, LIMB);
    link(world, "Shin", 0.5, 2.5, 1.0, BodyKind::Dynamic, KNEE);
    pin(world, "J.Hip", "Hip", "Thigh", [-1.0, 2.5], None);
    pin(
        world,
        "J.Knee",
        "Thigh",
        "Shin",
        [0.0, 2.5],
        Some([0.0, 2.0]),
    );

    // ── A COBRA: quatro elos, NENHUM estático.
    for i in 0..4 {
        link(
            world,
            &format!("Snake{i}"),
            4.5 + i as f32,
            2.5,
            1.0,
            BodyKind::Dynamic,
            SNAKE,
        );
    }
    for i in 0..3 {
        pin(
            world,
            &format!("J.Snake{i}"),
            &format!("Snake{i}"),
            &format!("Snake{}", i + 1),
            [5.0 + i as f32, 2.5],
            None,
        );
    }
}

impl crate::App {
    /// **Cena 54 (W-IK).** Três cadeias, PAUSADA, com o painel de física aberto.
    pub(crate) fn physics_smoke_ik(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_props(gfx.sim.world_mut());
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }
        // A ferramenta NÃO é armada em código: o passo 1 é escolher 'Pose' no
        // painel, e um smoke que arma o estado por baixo pula exatamente a
        // costura que ele deveria provar (a cicatriz que o `impasto_smoke`
        // prega, e que os smokes do Painter tiveram de corrigir).

        eprintln!(
            "[physics-smoke 54] A cena esta PAUSADA e o painel PHYSICS esta aberto (tecla W).\n  \
               Posar e AUTORAR: o resultado e a pose do documento, nao simulacao. Por isso\n  \
               esta cena nao toca -- com o relogio andando o solver sobrescreveria tudo.\n\n  \
               1. Abra a secao INTERACTION e escolha a ferramenta 'Pose'. A dica embaixo\n     \
                  muda para 'Paused + drag a jointed body' -- as outras tres pedem Play.\n  \
               2. Aperte B (mostra os colliders e os joints).\n  \
               3. O BRACO (esquerda, verde): arraste a MAO (o elo da ponta) para cima e\n     \
                  para os lados. O COTOVELO e o OMBRO dobram atras dela -- e isso que a\n     \
                  cinematica inversa e. O ombro claro e ESTATICO e NAO se move: ele e a\n     \
                  raiz, e a cena e quem diz isso (nenhuma escolha foi pedida).\n     \
                  (medido: alvo (-5,5 / 4,5) -> mao a 0,002 m dele, cotovelo em\n      \
                   (-6,13 / 3,72), ombro parado em (-7,00 / 2,50))\n  \
               4. Arraste a mao para MUITO longe (para fora da tela). A cadeia ESTICA e\n     \
                  APONTA para o cursor -- ela nao enrola sobre si mesma nem trava numa\n     \
                  direcao qualquer.\n     \
                  (medido, alvo (30 / 20): alcance 2,50 de 2,50 e a cadeia aponta a 25,3\n      \
                   graus, que e exatamente a direcao do alvo)\n  \
               5. A PERNA (centro): a canela laranja tem o JOELHO limitado a [0, 2] rad.\n     \
                  Arraste-a em volta do quadril, para todos os lados: a junta NUNCA passa\n     \
                  da faixa. O solver do rapier ignora limites; sem a projecao desta wave\n     \
                  o joelho ia a -1,82 rad (dobrado ao contrario) e o Play desfazia a pose\n     \
                  no primeiro tick.\n  \
               6. A COBRA (direita, azul): quatro elos e NENHUMA ancora. Arraste a cabeca:\n     \
                  a cadeia inteira acompanha e TRANSLADA -- a raiz e livre, porque nao ha\n     \
                  nada estatico a que ela se prenda.\n     \
                  (medido: puxando a cabeca 3,5 m para cima, a CAUDA anda 1,91 m)\n  \
               7. Smoothing (o unico knob): 0,05 responde na hora, 1,00 e macio e nao\n     \
                  chega ao alvo. E o unico numero do solver que muda algo -- o outro foi\n     \
                  medido INERTE e por isso nao tem slider.\n  \
               8. Tip Angle 'Match': a ponta mantem a ATITUDE que tinha enquanto voce a\n     \
                  arrasta. Em 'Free' ela gira com a cadeia.\n  \
               9. Ctrl+Z: UM passo desfaz o arrasto INTEIRO, com todos os elos juntos.\n  \
              10. Marque 'Physics' no transporte e de Play: a cadeia cai a partir da pose\n     \
                  que voce autorou. Posar nao simula -- ele prepara o que a simulacao usa."
        );
    }
}

#[cfg(test)]
#[path = "physics_smoke_ik_tests.rs"]
mod tests;
