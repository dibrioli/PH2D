//! **A cena da TALHA** (`PH2D_PHYSICS_SMOKE=61`, W-Pulley W3).
//!
//! Irmã de `physics_smoke_pulley.rs` e `_break.rs` pelo cap de 600 LOC, e o
//! corte é o assunto: lá moram o elevador (58), o guincho (59) e a ruptura (60);
//! aqui, o que muda quando a roldana deixa de ser um ponto do cenário e passa a
//! ser um ponto de um **CORPO** — a *cadernal móvel*, e com ela a vantagem
//! mecânica que o `ratio` prometia e não entregava (§3 do plano).
//!
//! Os números da mensagem saem da sonda `probe_smoke_61`, rodada sobre ESTAS
//! constantes.

use super::physics_smoke_pulley::{BOOM_Y, GROUND, POST, R, ball};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, PulleyWheel, RigidBody, WrapSide,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const TACKLE: [f32; 4] = [0.45, 0.90, 0.55, 1.0];
const PLAIN: [f32; 4] = [0.95, 0.45, 0.45, 1.0];
const COUNTER: [f32; 4] = [0.30, 0.32, 0.40, 1.0];

/// **A carga**, kg. Os DOIS rigs a carregam.
const BLOCK_MASS: f32 = 2.0;
/// **O contrapeso**, kg — METADE da carga, e os dois rigs têm o mesmo.
///
/// É o número que faz a cena: com metade do peso, a talha SEGURA (dois ramos) e
/// a corda comum não (um ramo). Um contrapeso qualquer deixaria os dois caindo
/// ou os dois subindo, e a diferença — que é a wave inteira — sumiria.
const COUNTER_MASS: f32 = 1.0;
/// Onde o bloco nasce.
const BLOCK_Y: f32 = 3.2;
/// Onde o contrapeso nasce — abaixo da roldana fixa, pendurado.
///
/// ⚠️ **Baixo o bastante para ele nunca ALCANÇAR a roldana**, e a medição
/// escolheu o número: o bloco do controle cai 3,15 m até o chão e o contrapeso
/// sobe outro tanto, então a 5,0 ele terminava em 7,46 — **acima** da roldana de
/// 7,0, que é o degenerado nomeado no W1 (um corpo que passa da própria roldana
/// inverte o ramo). A 3,5 ele para em ~6,6 e a cena mostra o que ela promete.
const COUNTER_Y: f32 = 3.5;
/// A meia-distância entre a ponta morta e a roldana fixa.
const SPAN: f32 = 0.9;

/// **Um rig de talha**, ou o controle 1:1 dele.
///
/// ```text
///   morta (x−0.9, BOOM)        roldana FIXA (x+0.9, BOOM)   [cenário]
///          \                          / \
///           \                        /   contrapeso (x+0.9)
///            \                      /
///             roldana MÓVEL (x, BLOCK_Y)  [montada no BLOCO]
/// ```
///
/// ⚠️ **A cadernal FIXA é o que faz a talha funcionar.** Sem ela a mão e a ponta
/// morta ficam as duas ACIMA do bloco, descer o bloco alonga os dois ramos, e a
/// mão desce junto: os dois lados liberam energia e não há equilíbrio nenhum. Foi
/// a primeira fixture desta wave, e a medição a derrubou.
///
/// `mounted = false` é o CONTROLE: a roldana móvel some e a corda é amarrada
/// DIRETO no bloco. Mesma corda, mesmo contrapeso, mesma roldana fixa — a única
/// diferença é **quantos ramos seguram o bloco**.
fn tackle(world: &mut World, tag: &str, x: f32, mounted: bool, rgba: [f32; 4]) {
    let dead = format!("{tag} Post");
    let block = format!("{tag} Block");
    let counter = format!("{tag} Counterweight");
    let rope = format!("{tag} Rope");
    // A ponta morta: um poste estático. Massa infinita para a corda, que é o que
    // uma amarração no teto é.
    world.spawn((
        Name::new(dead.clone()),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.15,
                half_y: 0.15,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [0.3, 0.3], POST),
        Transform::from_translation(Vec2::new(x - SPAN, BOOM_Y)),
    ));
    ball(world, &block, x, BLOCK_MASS, rgba, R * 1.4);
    ball(world, &counter, x + SPAN, COUNTER_MASS, COUNTER, R);
    for (name, y) in [(&block, BLOCK_Y), (&counter, COUNTER_Y)] {
        let mut q = world.query::<(&Name, &mut Transform)>();
        for (n, mut t) in q.iter_mut(world) {
            if n.as_str() == *name {
                t.translation.y = y;
            }
        }
    }
    world.spawn((
        Name::new(rope.clone()),
        PhysicsJoint {
            // ⚠️ No controle a ponta morta É o bloco: a corda o segura por um
            // ramo. É essa troca — e só ela — que muda a vantagem de 2 para 1.
            body_a: stable_name_id(if mounted { &dead } else { &block }),
            body_b: stable_name_id(&counter),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(x - SPAN, BOOM_Y)),
    ));
    let mut order = 0_u16;
    if mounted {
        // **A CADERNAL MÓVEL.** `body` a monta no bloco; o `local` é semeado pela
        // ponte contra a pose de REPOUSO dele, uma vez — daí o `mounted: false`
        // aqui, que é o mesmo sentinela `anchored` do joint e não uma omissão.
        world.spawn((
            Name::new(format!("{rope} Wheel 1")),
            PulleyWheel {
                rope: stable_name_id(&rope),
                order,
                radius: 0.35,
                wrap: WrapSide::Auto,
                body: stable_name_id(&block),
                ..Default::default()
            },
            Transform::from_translation(Vec2::new(x, BLOCK_Y)),
        ));
        order += 1;
    }
    // A cadernal FIXA, no cenário.
    world.spawn((
        Name::new(format!("{rope} Wheel {}", order + 1)),
        PulleyWheel {
            rope: stable_name_id(&rope),
            order,
            radius: 0.35,
            wrap: WrapSide::Auto,
            ..Default::default()
        },
        Transform::from_translation(Vec2::new(x + SPAN, BOOM_Y)),
    ));
}

/// **Quanto o bloco da TALHA anda em 2 s**, metros — ele fica onde está: dois
/// ramos de corda o seguram, e cada um carrega metade do peso.
pub(crate) const MEASURED_TACKLE_DRIFT: f32 = 0.02;
/// **Quanto o bloco do controle 1:1 CAI em 2 s**, metros — um ramo só, e um
/// contrapeso de metade do peso não o segura. Ele para aí porque encontra o
/// CHÃO, não porque a corda o pegou.
///
/// ⚠️ Eu havia escrito **2,3** aqui antes de medir, e a sonda deu 3,15. O número
/// da mensagem é o que a sonda diz, sempre — a estimativa era minha, o número é
/// do produto.
pub(crate) const MEASURED_PLAIN_DROP: f32 = 3.15;
/// **Quantos pontos o ímã do eixo oferece nesta cena** (W6).
///
/// ⚠️ **CINCO, não nove:** o bloco é um DISCO, e um disco não tem quinas — o
/// conjunto é o centro mais os quatro cardinais do aro (`ShapeDesc::snap_points`).
/// Inventar quinas ofereceria ao artista um ponto que **não está no corpo**, e a
/// mensagem tem de dizer o número que a cena de fato dá, não o do caso da caixa.
pub(crate) const MEASURED_SNAP_TARGETS: usize = 5;

/// Monta a cena 61.
pub(crate) fn build_tackle(world: &mut World) {
    world.spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 16.0,
                half_y: 0.3,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [32.0, 0.6], GROUND),
        Transform::from_translation(Vec2::new(0.0, -0.6)),
    ));
    tackle(world, "Tackle", -4.0, true, TACKLE);
    tackle(world, "Plain", 4.0, false, PLAIN);
}

#[cfg(test)]
#[path = "physics_smoke_pulley_tackle_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 61 (W-Pulley W3).** Duas montagens com a MESMA carga e o MESMO
    /// contrapeso; a única diferença é se a roldana está montada no bloco.
    ///
    /// Os números saem da sonda `probe_smoke_61`, rodada sobre ESTA cena.
    pub(crate) fn physics_smoke_tackle(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_tackle(gfx.sim.world_mut());
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 61] A TALHA -- a roldana montada num corpo que se move.\n  \
               O contorno dos vinculos ja esta ligado; de PLAY (o toggle Physics ja esta armado).\n\n  \
               Os DOIS rigs tem a mesma carga ({block:.0} kg) e o mesmo contrapeso\n  \
               ({counter:.0} kg, METADE dela). A unica diferenca e onde a roldana esta.\n\n  \
               1. VERDE (esquerda) -- a roldana de baixo esta MONTADA no bloco: e a\n     \
                  cadernal MOVEL de uma talha, entao DOIS ramos da corda o seguram e\n     \
                  cada um carrega metade do peso. Ele fica onde esta ({tackle:.2} m em 2 s).\n  \
               2. VERMELHO (direita) -- a mesma corda, o mesmo contrapeso, mas a corda\n     \
                  e amarrada DIRETO no bloco: UM ramo. Ele CAI ({plain:.1} m) e bate no chao.\n\n  \
               ⚠️ Ninguem autorou um \"2\" em lugar nenhum -- a vantagem e a geometria do\n     \
               enlace, e e por isso que o `ratio` da v1 saiu: numa corda sobre roldanas\n     \
               LIVRES a tensao e uniforme e nao ha vantagem nenhuma a ganhar de diametro.\n\n  \
               AUTORE VOCE MESMO: selecione 'Plain Rope Wheel 1' (a roldana da direita)\n  \
               na Hierarquia. Na secao 'Pulley Wheel' a row 'Mounted On' diz '(scenery)'.\n  \
               Clique o CONTA-GOTAS ao lado dela e depois clique no bloco VERMELHO: a\n  \
               roldana passa a citar o nome dele. (Este rig nao vira uma talha so com\n  \
               isso -- a corda dele esta amarrada NO bloco, entao ele ja e o proprio no;\n  \
               o que a row prova e que a montagem e autoravel com dois cliques.)\n  \
               A LIXEIRA ao lado desmonta de volta para o cenario.\n\n  \
               E A CORDA TAMBEM SE RE-ESCOLHE (W1): a row 'Rope' ganhou um\n  \
               CONTA-GOTAS. Selecione uma roldana, clique nele e depois clique EM\n  \
               CIMA DA CORDA no canvas -- o alvo e o traco desenhado, nao um corpo.\n  \
               Prove que ela funciona onde importa: RENOMEIE uma corda na Hierarquia\n  \
               (a roldana dela fica orfa e a row passa a dizer '(no rope)'); o\n  \
               conta-gotas segue oferecido, e um clique na corda a religa. Antes\n  \
               desta wave a unica volta era apagar a roldana e refaze-la.\n\n  \
               E O IMA (W6): PAUSE primeiro (uma alca de ponto e rest-only -- durante o\n  \
               play o overlay desenha a geometria do SOLVER, e a alca autora a AUTORADA).\n  \
               Selecione 'Tackle Rope Wheel 1' (a roldana MONTADA, a de baixo no rig\n  \
               verde) e arraste o DOT central com CTRL apertado. Ele cola\n  \
               nos {snap} pontos do collider do bloco que a carrega -- o centro e os\n  \
               quatro cardinais do disco --, e a marca do encaixe acende no ponto.\n  \
               ⚠️ Sem CTRL o arrasto e livre, como sempre foi. E na roldana do CENARIO\n  \
               (a de cima) o ima NAO abre: ela nao pertence a corpo nenhum, entao nao ha\n  \
               a que colar -- e e essa recusa que ele nao pode ter perdido.",
            block = BLOCK_MASS,
            counter = COUNTER_MASS,
            tackle = MEASURED_TACKLE_DRIFT,
            plain = MEASURED_PLAIN_DROP,
            snap = MEASURED_SNAP_TARGETS,
        );
    }
}
