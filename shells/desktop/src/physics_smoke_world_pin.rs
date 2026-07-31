//! **A cena do PINO DE MUNDO** (`PH2D_PHYSICS_SMOKE=65`, W-JointWorld).
//!
//! ⚠️ **A cena é desenhada para mostrar o que ela REMOVE**, não que ela funciona:
//! os dois rigs pendem da MESMA altura, com o MESMO corpo e o MESMO tipo de
//! joint. A diferença é o que existe na Hierarquia — o da esquerda precisa de um
//! **corpo estático inventado** só para servir de âncora, e o da direita não
//! precisa de nada.
//!
//! Os números da mensagem saem da sonda `probe_smoke_65`, rodada sobre ESTAS
//! constantes.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, JointWorldAnchor, PhysicsJoint, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const OLD: [f32; 4] = [0.95, 0.55, 0.35, 1.0];
const NEW: [f32; 4] = [0.40, 0.85, 0.55, 1.0];
const HOOK: [f32; 4] = [0.45, 0.47, 0.55, 1.0];

/// A altura da âncora — a MESMA nos dois rigs.
const ANCHOR_Y: f32 = 7.0;
/// O comprimento do braço: o corpo nasce um metro abaixo da âncora.
const ARM: f32 = 1.0;
const LEFT_X: f32 = -3.0;
const RIGHT_X: f32 = 3.0;

const CAMERA_CENTRE: [f32; 2] = [0.0, 5.0];
const CAMERA_HEIGHT: f32 = 12.0;

/// **MEDIDO** pela sonda `probe_smoke_65`: quanto cada pêndulo percorre em 2 s
/// partindo do repouso lateral.
///
/// ⚠️ Os DOIS rigs dão **exatamente** este número — 0,8383 contra 0,8383, quatro
/// decimais — e é essa igualdade, não o valor, que é a afirmação da cena: o pino
/// de mundo é um pivô de verdade, não um corpo congelado.
///
/// ⚠️ E o número é o da MEDIÇÃO, não o que eu tinha escrito: a primeira versão
/// desta constante era `1.0`, um palpite meu, e a sonda a corrigiu antes de a
/// mensagem chegar ao artista.
pub(crate) const MEASURED_SWING: f32 = 0.838;

fn bob(world: &mut World, name: &str, x: f32, tint: [f32; 4]) {
    world.spawn((
        Name::new(name),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.35 },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [0.7, 0.7], tint),
        // Deslocado do eixo vertical: um pêndulo largado de lado BALANÇA, e é o
        // balanço que mostra que a âncora é um pivô e não uma solda.
        Transform::from_translation(Vec2::new(x + ARM, ANCHOR_Y)),
    ));
}

pub(crate) fn build_world_pin(world: &mut World) {
    // ---- ESQUERDA: o jeito ANTIGO, e ele custa um objeto ----
    world.spawn((
        Name::new("Invented Hook"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.12 },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [0.24, 0.24], HOOK),
        Transform::from_translation(Vec2::new(LEFT_X, ANCHOR_Y)),
    ));
    bob(world, "Old Bob", LEFT_X, OLD);
    world.spawn((
        Name::new("Old Pin"),
        PhysicsJoint {
            body_a: stable_name_id("Old Bob"),
            body_b: stable_name_id("Invented Hook"),
            kind: JointKind::Pin,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(LEFT_X, ANCHOR_Y)),
    ));

    // ---- DIREITA: o pino de MUNDO — nenhum objeto a mais ----
    bob(world, "New Bob", RIGHT_X, NEW);
    world.spawn((
        Name::new("Wall Pin"),
        PhysicsJoint {
            body_a: stable_name_id("New Bob"),
            // ⚠️ ZERO, e é o mundo — não um nome que faltou. Quem diz isso é o
            // marcador na linha seguinte; sem ele este mesmo estado significa
            // *meio-autorado*, e o corpo cairia.
            body_b: 0,
            kind: JointKind::Pin,
            ..PhysicsJoint::default()
        },
        JointWorldAnchor,
        Transform::from_translation(Vec2::new(RIGHT_X, ANCHOR_Y)),
    ));
}

#[cfg(test)]
#[path = "physics_smoke_world_pin_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 65 (W-JointWorld).** Dois pêndulos idênticos; só o da esquerda
    /// precisa de um objeto inventado para existir.
    pub(crate) fn physics_smoke_world_pin(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_world_pin(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 65] O PINO DE MUNDO -- prender algo ao cenario deixa de\n  \
               custar um objeto inventado.\n  \
               A cena nasce PARADA e o contorno JA ESTA LIGADO -- B o ALTERNA.\n\n  \
               Os dois pendulos sao IGUAIS: mesmo corpo, mesma altura de ancora\n  \
               ({anchor:.0} m), mesmo tipo (Pin), mesmo braco ({arm:.0} m). A diferenca esta\n  \
               na HIERARQUIA, nao na fisica.\n\n  \
               1. LARANJA (esquerda) -- o jeito ANTIGO. Para pendurar UMA bola foi\n     \
                  preciso inventar 'Invented Hook': um corpo estatico que nao\n     \
                  representa nada, existe so para o joint ter uma segunda ponta, e\n     \
                  fica na Hierarquia para sempre podendo ser movido por acidente.\n  \
               2. VERDE (direita) -- o PINO DE MUNDO. Na Hierarquia ha 'New Bob' e\n     \
                  'Wall Pin', e mais nada. A ancora e o proprio joint.\n\n  \
               Os dois balancam IGUAL ({swing:.3} m de percurso em 2 s, os DOIS):\n  \
               e um pivo de verdade, nao um corpo congelado.\n\n  \
               AUTORE VOCE MESMO: selecione 'Wall Pin' na Hierarquia. Na secao Joint,\n  \
               a row 'Body B' diz **World** (nao '(missing)') e NAO tem conta-gotas --\n  \
               nao ha corpo a apontar. Logo abaixo, 'Anchor B' mostra [Object | World].\n  \
               - clique 'Object': o pino perde o mundo, 'Body B' volta a '(missing)' e\n    \
                 o conta-gotas REAPARECE. No Play a bola VERDE cai.\n  \
               - clique 'World' de novo: ela volta a pender.\n  \
               - com o pino selecionado, arraste o DOT AMBAR: a ancora anda e a bola\n    \
                 vai junto. Ctrl+Z desfaz em UM passo.\n  \
               - e faca o mesmo em 'Old Pin' (o laranja): clicar 'World' ali\n    \
                 ABANDONA o gancho inventado -- e ai da para apagar o objeto.\n\n  \
               (!) SCRUB: toque Play, deixe correr, edite qualquer numero do pino e\n     \
               arraste a regua PARA TRAS. A bola verde tem de continuar pendurada. Se\n     \
               ela cair, o replay correu sem a ancora -- e a licao do Weston.\n",
            anchor = ANCHOR_Y,
            arm = ARM,
            swing = MEASURED_SWING,
        );
    }
}
