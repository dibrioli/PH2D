//! **A cena da talha de WESTON** (`PH2D_PHYSICS_SMOKE=64`, W-Weston).
//!
//! Irmã de `physics_smoke_pulley{,_tackle,_break,_diff,_comp}.rs` pelo cap de 600
//! LOC, e o corte é o assunto: lá o tambor de dois raios é atravessado **uma** vez
//! (os dois contatos adjacentes, vantagem `R/r`); aqui o MESMO eixo é atravessado
//! **duas**, com a cadernal móvel abraçada entre os contatos — vantagem `2R/(R−r)`.
//!
//! ⚠️ **A cena é desenhada para mostrar POR QUE a máquina existe**, não que ela
//! funciona: os dois rigs têm as **mesmas duas circunferências**, a mesma carga e o
//! mesmo contrapeso, e a ÚNICA diferença é o chip `Drum | Weston`. Com dois anéis
//! gordos (0,500 e 0,375) o tambor compra 1,33× e a Weston compra **4×** — e é essa
//! desproporção, lida de duas figuras IDÊNTICAS, que é a wave.
//!
//! Os números da mensagem saem da sonda `probe_smoke_64`, rodada sobre ESTAS
//! constantes.

use super::physics_smoke_pulley::{GROUND, POST, R, ball};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, PulleyWheel, RigidBody, WestonAxle,
    WrapSide,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const WESTON: [f32; 4] = [0.45, 0.90, 0.55, 1.0];
const DRUM: [f32; 4] = [0.95, 0.45, 0.45, 1.0];
const COUNTER: [f32; 4] = [0.30, 0.32, 0.40, 1.0];

/// **A carga**, kg — os DOIS rigs a carregam.
///
/// Escolhida ENTRE as duas vantagens: acima dos 2,67 kg que o TAMBOR destas duas
/// circunferências segura, e abaixo dos 8 kg que a WESTON delas segura. É esse
/// intervalo que faz a mesma massa subir de um lado e cair do outro; fora dele os
/// dois rigs andariam para o mesmo lado e a wave sumiria da tela.
const LOAD_MASS: f32 = 5.0;
/// **O contrapeso**, kg — e os dois rigs têm o mesmo.
const COUNTER_MASS: f32 = 1.0;
/// O raio por onde a corda ENTRA no eixo.
const R_IN: f32 = 0.5;
/// **O segundo diâmetro — e ele é GORDO**, que é o ponto inteiro da máquina.
///
/// Como TAMBOR (os dois contatos adjacentes) ele compra `R/r` = 1,33×, e com a
/// cadernal a vantagem é 2,67 — quase nada além da talha sozinha. Como WESTON (os
/// contatos abraçando a cadernal) ele compra `R/(R−r)` = **4×**, e a vantagem é
/// **8**.
///
/// ⚠️ **Para o TAMBOR chegar aos mesmos 8 ele precisaria de `r = 0,125`** — um quarto
/// do irmão. E para chegar a 32, de `r = 0,031`: um tambor de espessura de fio de
/// cabelo, enquanto a Weston chega lá com `r = 0,469`. É a *diferença* de dois raios
/// gordos que é pequena, e é por isso que a máquina foi inventada.
const R_RET: f32 = 0.375;
/// O raio da cadernal móvel.
const SHEAVE_R: f32 = 0.3;
/// A meia-distância entre a ponta morta e o eixo.
const SPAN: f32 = 0.9;
/// Onde a carga nasce — baixa, pela mesma razão da cena 63: no rig do TAMBOR ela cai,
/// e perto do chão a queda termina antes de o contrapeso leve sair de quadro.
const LOAD_Y: f32 = 1.2;
/// Onde o contrapeso nasce.
///
/// ⚠️ **Alto, e a MEDIÇÃO escolheu o número.** No rig da Weston o lado do contrapeso
/// anda ~8× o que a carga anda, então ele DESPENCA enquanto a carga sobe um palmo: em
/// 2 s ele desce **6,85 m**. Nascendo a 6,0 (a altura da cena irmã) ele ENCOSTAVA NO
/// CHÃO aos ~1,9 s, e o pouso contaminava a razão medida (8,65 em vez de 8,50). Nascer
/// a 8 é o que lhe dá pista.
const COUNTER_Y: f32 = 8.0;
/// A altura dos eixos.
const AXLE_Y: f32 = 12.0;

/// **A talha de WESTON, ou o mesmo eixo lido como TAMBOR.**
///
/// ```text
///   morta (x−0.9, AXLE)           eixo (x+0.9, AXLE)   R = 0.5 · r = 0.375
///          \                          / \
///           \                        /   contrapeso (x+0.9)
///            \                      /
///             cadernal MÓVEL (x, LOAD_Y)  [montada na CARGA]
/// ```
///
/// A corda anda **do contrapeso** (ponta A) para a **ponta morta** (B). Com o
/// marcador, ela entra no eixo pelo raio grande, desce até a cadernal, abraça, e
/// **volta ao MESMO eixo** pelo pequeno — e o que sobra até a ponta morta é o ramo
/// SOLTO, que não carrega nada (é o lado frouxo da alça de mão de uma talha real).
///
/// `weston = false` é o **CONTROLE**: as MESMAS duas circunferências como tambor
/// adjacente, onde a corda troca de diâmetro no próprio nó.
fn weston_hoist(world: &mut World, tag: &str, x: f32, weston: bool, rgba: [f32; 4]) {
    let dead = format!("{tag} Post");
    let load = format!("{tag} Load");
    let counter = format!("{tag} Counterweight");
    let rope = format!("{tag} Rope");
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
        Transform::from_translation(Vec2::new(x - SPAN, AXLE_Y)),
    ));
    ball(world, &load, x, LOAD_MASS, rgba, R * 1.4);
    ball(world, &counter, x + SPAN, COUNTER_MASS, COUNTER, R);
    for (name, y) in [(&load, LOAD_Y), (&counter, COUNTER_Y)] {
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
            // ⚠️ **O CONTRAPESO é a ponta A**, e a ordem não é decorativa: o peso da
            // rota conta a partir de A, então é ela que decide se a carga está do lado
            // MULTIPLICADO da corda.
            body_a: stable_name_id(&counter),
            body_b: stable_name_id(&dead),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(x + SPAN, COUNTER_Y)),
    ));
    // **O EIXO COMPOSTO** (ordem 0) — as duas circunferências, iguais nos dois rigs.
    let axle = world.spawn((
        Name::new(format!("{rope} Axle")),
        PulleyWheel {
            rope: stable_name_id(&rope),
            order: 0,
            radius: R_IN,
            radius_out: R_RET,
            wrap: WrapSide::Auto,
            ..PulleyWheel::default()
        },
        Transform::from_translation(Vec2::new(x + SPAN, AXLE_Y)),
    ));
    if weston {
        // A ÚNICA diferença entre os dois rigs: a presença do marcador.
        let e = axle.id();
        world.entity_mut(e).insert(WestonAxle);
    }
    // **A CADERNAL MÓVEL** (ordem 1): montada na carga, e é ela que a Weston abraça.
    world.spawn((
        Name::new(format!("{rope} Sheave")),
        PulleyWheel {
            rope: stable_name_id(&rope),
            order: 1,
            radius: SHEAVE_R,
            wrap: WrapSide::Auto,
            body: stable_name_id(&load),
            ..PulleyWheel::default()
        },
        Transform::from_translation(Vec2::new(x, LOAD_Y)),
    ));
}

/// **Quanto a carga da WESTON sobe em 2 s**, metros. Medido por `probe_smoke_64`.
pub(crate) const MEASURED_WESTON_RISE: f32 = 0.81;
/// **Quanto a carga do TAMBOR cai no mesmo tempo**, metros — ela para aí porque
/// encontra o CHÃO, não porque a corda a pegou.
pub(crate) const MEASURED_DRUM_DROP: f32 = 0.75;
/// **Quanto o contrapeso da Weston DESCE**, metros — e é o número que se vê de longe.
///
/// ⚠️ **A razão medida é 8,50 e a cinemática é 8,00**, e a diferença é honesta: `8` é
/// a razão para um deslocamento INFINITESIMAL, e em 6,85 m de curso a geometria da
/// rota muda (as pernas deixam de ser quase verticais). É por isso que a mensagem diz
/// *cerca de 8×* em vez de cravar o número.
pub(crate) const MEASURED_WESTON_COUNTER_DROP: f32 = 6.85;

/// O enquadramento — os eixos estão a `y = 12` e a câmera padrão mostra até 5.
/// Ver o doc de `CAMERA_CENTRE` da cena 63: *uma cena de smoke enquadra o que ela
/// pede que se olhe*, e esta pede os anéis do eixo.
pub(crate) const CAMERA_CENTRE: [f32; 2] = [0.0, 6.0];
pub(crate) const CAMERA_HEIGHT: f32 = 15.0;

/// Enquadrar não é AFASTAR até tudo caber — compile-time, como na cena irmã.
const _: () = assert!(CAMERA_HEIGHT < 20.0);

/// Monta a cena 64.
pub(crate) fn build_weston(world: &mut World) {
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
        Transform::from_translation(Vec2::new(0.0, -0.3)),
    ));
    weston_hoist(world, "Weston", -3.0, true, WESTON);
    weston_hoist(world, "Drum", 3.0, false, DRUM);
}

#[cfg(test)]
#[path = "physics_smoke_pulley_weston_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 64 (W-Weston).** Duas máquinas com as MESMAS duas circunferências, a
    /// mesma carga e o mesmo contrapeso; a única diferença é o chip `Drum | Weston`.
    pub(crate) fn physics_smoke_weston(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_weston(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 64] A TALHA DE WESTON -- a vantagem sai de duas\n  \
               circunferencias que se consegue DESENHAR.\n  \
               A cena nasce PARADA e o contorno JA ESTA LIGADO -- B o ALTERNA.\n\n  \
               Os DOIS rigs tem as MESMAS duas circunferencias ({r_in:.3} e {r_ret:.4}), a\n  \
               MESMA carga ({load:.0} kg) e o MESMO contrapeso ({counter:.0} kg). A unica\n  \
               diferenca e o chip 'Differential' -- e eles andam para lados OPOSTOS.\n\n  \
               1. VERDE (esquerda) -- WESTON. A corda sai pelo aro GRANDE, desce, abraca\n     \
                  a cadernal movel e VOLTA ao mesmo eixo pelo PEQUENO. O eixo compra\n     \
                  R/(R-r) = {wgear:.0}x, a cadernal dobra: vantagem {wmech:.0}x. 1 kg segura ate\n     \
                  {whold:.0} kg, e os {load:.0} kg SOBEM (+{wrise:.2} m em 2 s). Olhe o CONTRAPESO:\n     \
                  ele desce {wdrop:.2} m no mesmo tempo -- CERCA de {wmech:.0}x o que a carga\n     \
                  sobe (medido 8,50: os 8 valem para um passo infinitesimal, e em 6,8 m\n     \
                  de curso a geometria da rota muda). E essa razao que se ve de longe.\n  \
               2. VERMELHO (direita) -- TAMBOR. As MESMAS duas circunferencias, mas a\n     \
                  corda troca de diametro NO PROPRIO NO: ela nem chega ao aro pequeno\n     \
                  por fora. O eixo compra R/r = {dgear:.2}x, vantagem {dmech:.2}x -- quase nada\n     \
                  alem da talha sozinha. Os {load:.0} kg CAEM ({ddrop:.2} m) ate o chao.\n\n  \
               (!) E POR ISSO QUE A MAQUINA EXISTE: para o TAMBOR chegar aos {wmech:.0}x da\n     \
               Weston ele precisaria de um aro de saida de {hair:.3} m -- um QUARTO do\n     \
               irmao. Para chegar a 32x, de 0,031: um tambor de espessura de fio de\n     \
               cabelo, enquanto a Weston chega la com 0,469. E a DIFERENCA de dois\n     \
               raios gordos que e pequena, e e ela que a Weston usa.\n  \
               (!) O contrapeso VERMELHO e arremessado para cima quando a carga pousa,\n     \
               e isso e a corda sendo corda: ela so PUXA.\n\n  \
               AUTORE VOCE MESMO (o chip): selecione 'Drum Rope Axle' na Hierarquia. Na\n  \
               secao Pulley Wheel a row 'Differential' mostra [Drum | Weston] e o\n  \
               readout 'Gear' mostra {dgear:.2} : 1. Clique 'Weston': o Gear salta para\n  \
               {wgear:.2} : 1 e, no Play, a carga da DIREITA para de cair e sobe.\n  \
               - a row so aparece porque ha um SEGUNDO diametro. Digite 0 em\n    \
                 'Out Radius (m)' e ela desaparece: sem o que retornar por, o chip\n    \
                 armaria o nada.\n  \
               - digite {r_in:.3} (o MESMO valor da entrada) em 'Out Radius' e ela\n    \
                 desaparece tambem: com os dois raios iguais a talha esta TRAVADA (a\n    \
                 carga nao anda por mais que se puxe), e isso nao e um orcamento que\n    \
                 uma corda -- que so puxa -- saiba segurar.\n\n  \
               E O DESENHO: no rig VERDE a corda TOCA os dois aneis do eixo (desce do\n  \
               grande, sobe para o pequeno) e o ramo que sobra ate o poste e o lado\n  \
               SOLTO -- ele nao carrega nada. No VERMELHO ela troca de diametro no no\n  \
               e sai direto para a cadernal. Os dois aneis sao os MESMOS nas duas\n  \
               figuras; o que muda e o CAMINHO.",
            load = LOAD_MASS,
            counter = COUNTER_MASS,
            r_in = R_IN,
            r_ret = R_RET,
            wgear = R_IN / (R_IN - R_RET),
            wmech = 2.0 * R_IN / (R_IN - R_RET),
            whold = COUNTER_MASS * 2.0 * R_IN / (R_IN - R_RET),
            dgear = R_IN / R_RET,
            dmech = 2.0 * R_IN / R_RET,
            // O `r_saida` que daria ao TAMBOR a MESMA engrenagem da Weston:
            // `R/r_saida = R/(R−r)` ⇒ `r_saida = R − r`.
            hair = R_IN - R_RET,
            wrise = MEASURED_WESTON_RISE,
            wdrop = MEASURED_WESTON_COUNTER_DROP,
            ddrop = MEASURED_DRUM_DROP,
        );
    }
}
