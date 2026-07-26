//! **A cena do ROMPIMENTO** (`PH2D_PHYSICS_SMOKE=49`, W-J7).
//!
//! Um joint segurava dois corpos *aconteça o que acontecer*, e essa é exatamente
//! a coisa que o artista uma hora não quer: a corda que arrebenta, a dobradiça
//! arrancada, a corrente que parte sob um peso grande demais. rapier não modela
//! rompimento, mas publica o número que o decide — a reação que a restrição está
//! aplicando — e agora ele tem uma unidade e um teto.
//!
//! ⚠️ **É um teto de CARGA, não de impacto**, e a cena é montada para dizer isso:
//! o pico de uma pancada resolve DENTRO de um sub-passo do solver e não é
//! observável de fora (medido, `ph2d-physics/tests/measure_joint_break.rs`). O
//! que se vê aqui é o que a feature de fato faz — *"isto está segurando mais do
//! que aguenta"*.
//!
//! Os números abaixo saíram de uma sonda headless sobre esta mesma armação,
//! rodada ANTES desta mensagem ser escrita.

use crate::physics_smoke::spawn_floor;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, MassOverride, PhysicsJoint, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// O que toda corda desta cena aguenta, newtons. Escolhido pela MEDIÇÃO: uma
/// carga pendurada lê o próprio peso exatamente (1 kg = 9,81 N), então 60 N fica
/// entre os 49 N da carga média e os 98 N da pesada.
const RATING_N: f32 = 60.0;

/// O que cada elo da CORRENTE aguenta, newtons. Maior porque a corrente carrega
/// muito mais: o elo de cima segura os três elos de 4 kg MAIS a bigorna, e o
/// número saiu da mesma sonda — 127,5 N estáticos, 144,2 N no assentamento,
/// contra 88,3 N no elo seguinte.
const CHAIN_RATING_N: f32 = 120.0;

impl crate::App {
    /// **Cena 49 (W-J7).** Três cargas na mesma corda, uma corrente que parte no
    /// elo de cima, uma dobradiça arrancada por TORQUE, e um par solto para
    /// autorar à mão. PAUSADA.
    pub(crate) fn physics_smoke_joint_break(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        let grey = [0.75, 0.75, 0.8, 1.0];
        let hot = [0.95, 0.6, 0.2, 1.0];
        let cool = [0.4, 0.8, 0.95, 1.0];
        let peg = ColliderShape::Ball { radius: 0.08 };

        {
            let mut hook = |name: &str, at: [f32; 2]| {
                world.spawn((
                    Transform::from_translation(Vec2::new(at[0], at[1])),
                    Sprite::atlas(WHITE_TILE_KEY, [0.16, 0.16], grey),
                    Name::new(name.to_string()),
                    RigidBody {
                        kind: BodyKind::Static,
                    },
                    Collider {
                        shape: peg,
                        ..Collider::default()
                    },
                ));
            };
            for (n, x) in [("Hook 1kg", -6.0), ("Hook 5kg", -4.5), ("Hook 10kg", -3.0)] {
                hook(n, [x, 8.0]);
            }
            hook("Chain Hook", [0.5, 8.5]);
            hook("Door Hinge", [4.5, 7.5]);
            hook("Bare Hook", [7.5, 8.0]);
        }
        {
            let mut load = |name: &str, at: [f32; 2], mass: f32, size: f32, rgba: [f32; 4]| {
                world.spawn((
                    Transform::from_translation(Vec2::new(at[0], at[1])),
                    Sprite::atlas(WHITE_TILE_KEY, [size, size], rgba),
                    Name::new(name.to_string()),
                    RigidBody {
                        kind: BodyKind::Dynamic,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: size * 0.5,
                            half_y: size * 0.5,
                        },
                        ..Collider::default()
                    },
                    MassOverride(mass),
                ));
            };
            // ── A: a MESMA corda, três cargas. 9,81 N · 49,05 N · 98,1 N.
            load("Light 1kg", [-6.0, 7.0], 1.0, 0.4, cool);
            load("Medium 5kg", [-4.5, 7.0], 5.0, 0.6, cool);
            load("Heavy 10kg", [-3.0, 7.0], 10.0, 0.8, hot);
            // ── B: a corrente. O elo de CIMA carrega tudo que está pendurado
            // nele, então é ele que parte — e é isso que se vê.
            // ⚠️ Elos PESADOS e bigorna leve, e a escolha é MEDIDA, não estética:
            // com elos leves o transiente de assentamento (~1,3× a carga estática)
            // parte três dos quatro no primeiro tick, e a cena passa a ensinar
            // "uma corrente rateada baixo demais falha em todo lugar" em vez de
            // "o elo de cima carrega o resto". Para que só o de cima passe do
            // teto, o elo tem de pesar mais que ~0,75 da bigorna.
            for (i, y) in [7.5_f32, 6.5, 5.5].iter().enumerate() {
                load(&format!("Link {}", i + 1), [0.5, *y], 4.0, 0.5, grey);
            }
            load("Anvil 4kg", [0.5, 4.5], 1.0, 0.35, hot);
            // ── D: o par para ARMAR à mão (nenhuma joint entre eles ainda).
            load("Bare Load", [7.5, 7.0], 4.0, 0.5, cool);
        }
        // ── C: a porta pesada. Uma dobradiça travada segura `m·g·r`, e é TORQUE
        // — o único teto que só um Pin consegue reportar. Fora do bloco acima
        // porque ela não é um quadrado: a alavanca é o que faz o torque.
        world.spawn((
            Transform::from_translation(Vec2::new(5.5, 7.5)),
            Sprite::atlas(WHITE_TILE_KEY, [2.0, 0.2], hot),
            Name::new("Heavy Door".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 1.0,
                    half_y: 0.1,
                },
                ..Collider::default()
            },
            MassOverride(6.0),
        ));

        let mut joint = |name: &str, a: &str, b: &str, at: [f32; 2], j: PhysicsJoint| {
            world.spawn((
                Name::new(name.to_string()),
                PhysicsJoint {
                    body_a: stable_name_id(a),
                    body_b: stable_name_id(b),
                    ..j
                },
                Transform::from_translation(Vec2::new(at[0], at[1])),
            ));
        };
        let rope = |max_length: f32| PhysicsJoint {
            kind: JointKind::Rope,
            max_length,
            break_enabled: true,
            break_force: RATING_N,
            ..PhysicsJoint::default()
        };
        for (n, a, b, x) in [
            ("Rope 1kg", "Hook 1kg", "Light 1kg", -6.0),
            ("Rope 5kg", "Hook 5kg", "Medium 5kg", -4.5),
            ("Rope 10kg", "Hook 10kg", "Heavy 10kg", -3.0),
        ] {
            joint(n, a, b, [x, 8.0], rope(1.0));
        }
        // A corrente: gancho → elo 1 → elo 2 → elo 3 → bigorna. Todos rateados
        // igual; o de cima carrega 3 elos + a bigorna e é o que parte.
        // A corrente tem teto PRÓPRIO — cada joint é rateado por si, e uma
        // corrente de elos de 4 kg carrega muito mais que uma carga solta.
        for (n, a, b, y) in [
            ("Chain 1", "Chain Hook", "Link 1", 8.5),
            ("Chain 2", "Link 1", "Link 2", 7.5),
            ("Chain 3", "Link 2", "Link 3", 6.5),
            ("Chain 4", "Link 3", "Anvil 4kg", 5.5),
        ] {
            joint(
                n,
                a,
                b,
                [0.5, y],
                PhysicsJoint {
                    break_force: CHAIN_RATING_N,
                    ..rope(1.0)
                },
            );
        }
        // A dobradiça travada: `m·g·r` = 6 kg × 9,81 × 1 m = 58,9 N·m, contra um
        // teto de 20. O teto de FORÇA fica em ∞ de propósito, para que só o
        // torque possa ter sido o que rompeu.
        joint(
            "Door Hinge Joint",
            "Door Hinge",
            "Heavy Door",
            [4.5, 7.5],
            PhysicsJoint {
                kind: JointKind::Pin,
                limits_enabled: true,
                limit_min: -0.01,
                limit_max: 0.01,
                break_enabled: true,
                // ⚠️ Fora de alcance, e não `∞`: o componente guarda UM checkbox
                // para os dois tetos (é assim que o card é desenhado), então
                // *"não rompe por força, rompe por torque"* se escreve pondo o
                // teto de força alto o bastante para nada o cruzar — a porta pesa
                // 58,9 N. `f32::INFINITY` não serve: o `clamped()` o troca pelo
                // default, porque um não-finito vindo de um arquivo é lixo.
                break_force: 1000.0,
                break_torque: 20.0,
                ..PhysicsJoint::default()
            },
        );

        eprintln!(
            "[physics-smoke 49] O ROMPIMENTO -- um joint pode ser autorado para\n\
             PARTIR sob carga (W-J7). PAUSADA.\n  \
               1. Aperte B (contornos) e de Play. Medido (headless, sobre esta\n     \
                  mesma armacao) -- as tres cordas da esquerda sao IDENTICAS, todas\n     \
                  rateadas em 60 N, e so a carga muda:\n       \
                  - 1 kg (9,8 N): SEGURA. Fica pendurada em y = 7,00.\n       \
                  - 5 kg (49,1 N): SEGURA. Idem.\n       \
                  - 10 kg (98,1 N): **PARTE** -- a corda fica VERMELHA com um\n         \
                    estouro de seis pontas no ponto onde arrebentou, um toast diz\n         \
                    'Rope 10kg broke at ... N', e a caixa cai no chao.\n  \
               2. A CORRENTE (meio): quatro elos iguais segurando uma bigorna de\n     \
                  1 kg, cada elo rateado em 120 N. O elo de CIMA carrega os tres\n     \
                  elos MAIS a bigorna -- medido, **144,2 N** -- e e o UNICO que\n     \
                  passa; o elo seguinte carrega 88,3 N e segura. Entao a corrente\n     \
                  parte NO TOPO e cai INTEIRA, ainda presa entre si. E o\n     \
                  fato fisico que a feature torna visivel: numa corrente todos os\n     \
                  elos sao iguais e nem todos carregam o mesmo.\n  \
               3. A PORTA (direita): uma dobradica TRAVADA segurando uma porta de\n     \
                  6 kg com um metro de braco -- 58,9 N.m contra um teto de 20. O\n     \
                  teto de FORCA dela esta em 1000 N (fora de alcance -- a porta pesa\n     \
                  58,9 N), entao so o TORQUE pode ter sido o que rompeu.\n  \
               4. Rebobine (o rompimento e RUNTIME, nao autoria): tudo volta\n     \
                  inteiro. Selecione 'Rope 10kg' na Hierarquia -- a secao Physics\n     \
                  Joint mostra o card **Breakable** com **Break Force (N)** em 60.\n     \
                  Digite 200 e de Play: agora ela segura.\n  \
               5. Selecione 'Door Hinge Joint'. O MESMO card mostra uma row a mais,\n     \
                  **Break Torque (N.m)** -- e ela existe SO no Pin. Clique pelos\n     \
                  cinco tipos no seletor Kind: Spring/Rope/Weld/Slider oferecem\n     \
                  Break Force e nao oferecem Break Torque. Nao e gosto: o rapier\n     \
                  reporta a reacao de um eixo angular LIMITADO ou MOTORIZADO e nada\n     \
                  de um eixo TRAVADO (medido -- um Weld em balanco segura 4,905 N.m\n     \
                  e le 0,0000), entao um teto de torque num Weld nunca poderia\n     \
                  disparar.\n  \
               6. ARME UM A MAO: selecione 'Bare Hook' e 'Bare Load', ponha\n     \
                  **Join As = Rope**, aperte **Join Selected Bodies**. A joint nova\n     \
                  ja nasce selecionada: ligue **Breakable**, digite 20 no Break\n     \
                  Force e de Play -- 4 kg sao 39,2 N, entao ela parte na hora.\n  \
               7. ⚠️ E um teto de CARGA, nao de impacto. O pico de uma pancada\n     \
                  resolve DENTRO de um sub-passo do solver e nao e observavel de\n     \
                  fora (medido: uma corda que para 1 kg vindo a 6,26 m/s reporta os\n     \
                  mesmos 9,8 N que ela reporta parada). Uma corda nao arrebenta\n     \
                  'no tranco' aqui -- ela arrebenta quando o que esta pendurado\n     \
                  nela pesa mais do que ela aguenta."
        );
    }
}
