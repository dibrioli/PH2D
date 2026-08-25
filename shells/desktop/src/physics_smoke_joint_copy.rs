//! **A cena do COPIAR/COLAR propriedades** (`PH2D_PHYSICS_SMOKE=66`,
//! W-JointCopy).
//!
//! ⚠️ **A cena é desenhada para tornar o PASTE visível de longe.** Quatro
//! portões idênticos, pendurados no cenário pela mesma altura, com o mesmo corpo
//! e o mesmo tipo de joint. Um deles — o da esquerda — foi afinado: a dobradiça
//! dele tem **batentes**. Os outros três não têm nada, e por isso caem até
//! pendurar.
//!
//! O gesto que a cena pede é o do Unreal PhAT: afine UM, copie, selecione os
//! outros, cole. O que se vê é os três portões pararem de cair.
//!
//! Os números da mensagem saem da sonda `probe_smoke_66`, rodada sobre ESTAS
//! constantes — e a primeira rodada dela corrigiu dois palpites meus.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, JointWorldAnchor, PhysicsJoint, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// O afinado — o que já tem batentes.
const TUNED: [f32; 4] = [0.40, 0.85, 0.55, 1.0];
/// Os três por afinar.
const PLAIN: [f32; 4] = [0.95, 0.55, 0.35, 1.0];

/// Meia-largura do portão. A âncora fica na ponta ESQUERDA dele, então o corpo
/// gira em torno de um pivô que se vê.
const HALF_W: f32 = 0.75;
const HALF_H: f32 = 0.09;
const ANCHOR_Y: f32 = 6.0;
/// Os centros dos quatro portões — o afinado primeiro.
const XS: [f32; 4] = [-4.5, -1.5, 1.5, 4.5];
const NAMES: [&str; 4] = ["Gate A", "Gate B", "Gate C", "Gate D"];
const PINS: [&str; 4] = ["Pin A", "Pin B", "Pin C", "Pin D"];

/// A meia-faixa dos batentes do portão afinado, **graus** — o número que a §12
/// mostra e que o paste carrega.
pub(crate) const TUNED_LIMIT_DEG: f32 = 25.0;

const CAMERA_CENTRE: [f32; 2] = [0.0, 5.2];
const CAMERA_HEIGHT: f32 = 9.0;

/// **MEDIDO** pela sonda `probe_smoke_66`: a MAIOR excursão de cada portão em
/// 2 s, partindo da horizontal.
///
/// ⚠️ A maior excursão, e não o ângulo do último tick — um portão sem batente é
/// um pêndulo, e o instante `t = 2 s` é um ponto arbitrário do ciclo dele. A
/// primeira versão da sonda mediu ali e reportou **38,1°** para os livres: um
/// número que não descreve nada, e que teria virado a constante desta mensagem
/// (o doc da sonda guarda a lição inteira).
///
/// O afinado para no próprio batente.
pub(crate) const MEASURED_TUNED_DEG: f32 = 25.1;
/// O de um portão SEM batentes: ele passa da vertical e quase dá a volta.
pub(crate) const MEASURED_PLAIN_DEG: f32 = 179.7;
/// E o de um portão que RECEBEU a colagem — o número que fecha a demonstração,
/// e ele é o do afinado ao décimo.
pub(crate) const MEASURED_PASTED_DEG: f32 = 25.1;

/// Um portão e o pino que o segura ao cenário.
///
/// `tune` decide se a dobradiça nasce com batentes. Tudo o mais é idêntico entre
/// os quatro **de propósito**: se algo além do joint diferisse, a cena estaria
/// demonstrando outra coisa.
fn gate(world: &mut World, i: usize, tune: bool) {
    let x = XS[i];
    let tint = if tune { TUNED } else { PLAIN };
    world.spawn((
        Name::new(NAMES[i]),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: HALF_W,
                half_y: HALF_H,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [HALF_W * 2.0, HALF_H * 2.0], tint),
        Transform::from_translation(Vec2::new(x, ANCHOR_Y)),
    ));
    world.spawn((
        Name::new(PINS[i]),
        PhysicsJoint {
            body_a: stable_name_id(NAMES[i]),
            // O cenário — o pino de mundo da wave anterior, e é o que deixa a
            // cena montar quatro rigs sem quatro ganchos inventados.
            body_b: 0,
            kind: JointKind::Pin,
            limits_enabled: tune,
            limit_min: -TUNED_LIMIT_DEG.to_radians(),
            limit_max: TUNED_LIMIT_DEG.to_radians(),
            ..PhysicsJoint::default()
        },
        JointWorldAnchor,
        // A âncora na ponta ESQUERDA do portão: o pivô fica onde se vê.
        Transform::from_translation(Vec2::new(x - HALF_W, ANCHOR_Y)),
    ));
    ph2d_physics_ecs::resolve_body_names(world);
}

pub(crate) fn build_joint_copy(world: &mut World) {
    for i in 0..4 {
        gate(world, i, i == 0);
    }
}

#[cfg(test)]
#[path = "physics_smoke_joint_copy_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 66 (W-JointCopy).** Quatro portões iguais; um afinado, três por
    /// afinar — e um clique que os iguala.
    pub(crate) fn physics_smoke_joint_copy(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_joint_copy(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 66] COPIAR E COLAR AS PROPRIEDADES DE UM JOINT -- afine\n  \
               UM e carimbe o rig inteiro.\n  \
               A cena nasce PARADA e o contorno JA ESTA LIGADO -- B o ALTERNA.\n\n  \
               Quatro portoes IGUAIS, pendurados no cenario pela mesma altura, com o\n  \
               mesmo corpo e o mesmo tipo de joint (Pin). So a dobradica difere:\n  \
               1. VERDE (esquerda) -- 'Pin A' esta AFINADO: batentes de +/-{limit:.0} graus.\n  \
               2. LARANJA (os tres) -- dobradicas cruas, sem batente nenhum.\n\n  \
               DE PLAY antes de copiar, para ver o problema: o verde mal se mexe\n  \
               ({tuned:.0} graus, que e' o proprio batente) e os tres laranjas DESABAM e\n  \
               giram ate quase dar a volta ({plain:.0} graus). Volte a regua ao zero.\n\n  \
               O GESTO (o 'Copy properties to...' do Unreal PhAT):\n  \
               1. selecione 'Pin A' na Hierarquia. Na secao Joint, no fim, ha\n     \
                  'Copy Properties'. Clique.\n  \
               2. selecione 'Pin B', 'Pin C' e 'Pin D' (Ctrl+clique nos tres).\n  \
               3. o botao agora diz **'Paste to 3 Joints'** -- a contagem esta NO\n     \
                  rotulo, porque um clique que muda tres objetos tem de dizer isso\n     \
                  antes. Clique.\n  \
               4. Play: os quatro batem no mesmo batente ({pasted:.0} graus) -- os tres\n     \
                  que giravam 180 param junto com o verde.\n\n  \
               O QUE **NAO** VIAJA, e vale conferir -- e' o desenho inteiro:\n  \
               - as DUAS PONTAS. Cada pino continua segurando o SEU portao; colar as\n    \
                 pontas seria duplicar o joint, nao copiar as propriedades dele.\n  \
               - a ANCORA. Arraste o dot ambar de 'Pin D' para o meio do portao ANTES\n    \
                 de colar: depois da colagem ele continua ali. O offset e' medido no\n    \
                 corpo, e o corpo do vizinho pode ter outro tamanho.\n  \
               - o interruptor **Active**. Desligue 'Pin C' (Active = Off) e cole por\n    \
                 cima: ele continua desligado. Active e' o 'experimente o rig sem\n    \
                 este aqui' -- uma investigacao sobre UM joint, e a colagem age sobre\n    \
                 muitos.\n\n  \
               E O TIPO **VIAJA**, de proposito: metade destes numeros nao tem unidade\n  \
               propria (o curso e' RADIANO num Pin e METRO num Slider). Troque 'Pin A'\n  \
               para Spring, afine rigidez/amortecimento, copie e cole nos outros: os\n  \
               tres viram Spring com os MESMOS numeros. Sem o tipo junto, colar um\n  \
               '0,785' de um Pin num Slider viraria 0,785 METRO de curso.\n\n  \
               (!) UM Ctrl+Z desfaz a colagem inteira -- os tres joints num passo so.\n",
            limit = TUNED_LIMIT_DEG,
            tuned = MEASURED_TUNED_DEG,
            plain = MEASURED_PLAIN_DEG,
            pasted = MEASURED_PASTED_DEG,
        );
    }
}
