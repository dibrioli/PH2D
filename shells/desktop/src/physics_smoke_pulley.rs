//! **A cena do ELEVADOR** (`PH2D_PHYSICS_SMOKE=58`, W-Pulley).
//!
//! Uma polia é o primeiro vínculo do kit que **não é um joint do rapier** — ele
//! não tem polia e não oferece gancho de restrição, então ela é imposta de fora,
//! por um passe de impulso por sub-passo. O que se vê na tela, porém, tem de ser
//! simplesmente *uma corda que passa por duas roldanas*.
//!
//! Dois rigs, lado a lado, e a única diferença entre eles é a **razão**:
//!
//! - **VERDE, razão 1** — o elevador com contrapeso. O que um lado desce, o
//!   outro sobe, e um contrapeso mais leve não segura a carga.
//! - **ÂMBAR, razão 0,25** — a TALHA. Mesmo par de massas, e agora o lado leve
//!   ergue o pesado: é a vantagem mecânica, o motivo de a razão existir.
//!
//! ⚠️ **A primeira versão desta cena usava razão 2 e afirmava o contrário — a
//! sonda a desmentiu.** Com `l1 + r·l2 = L0` o lado B anda `1/r` do que A anda e
//! precisa pesar `r` vezes mais para equilibrar, então a vantagem de B é `1/r`:
//! ela vem de `r` **menor** que 1. Com `r = 2` a carga de 3 kg não só ganhava
//! como caía o DOBRO. É a terceira vez nesta jornada que uma cena afirma o que a
//! medição nega, e é por isso que a sonda roda antes da mensagem.
//!
//! Os números da mensagem saem da sonda `probe_smoke_58`, rodada sobre ESTAS
//! peças antes de a mensagem ser escrita.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const GROUND: [f32; 4] = [0.42, 0.42, 0.46, 1.0];
const SIMPLE: [f32; 4] = [0.45, 0.9, 0.45, 1.0];
const TACKLE: [f32; 4] = [0.95, 0.75, 0.30, 1.0];
const WEIGHT: [f32; 4] = [0.30, 0.32, 0.40, 1.0];

/// Onde os dois corpos de cada rig nascem.
const START_Y: f32 = 3.0;
/// A distância entre a carga e o contrapeso de um rig — é ela que decide a
/// altura das roldanas (o semeio põe cada uma meia distância acima do corpo).
const SPAN: f32 = 3.0;

/// **Quanto a carga do rig VERDE desce em 3 s**, metros. Razão 1: a carga é 3×
/// o contrapeso, então ela ganha e desce.
pub(crate) const MEASURED_SIMPLE_LOAD_DROP: f32 = 1.50;
/// **Quanto o contrapeso VERDE sobe no mesmo tempo.** A corda é inextensível, e
/// é isso que estes dois números dizem juntos.
pub(crate) const MEASURED_SIMPLE_CW_RISE: f32 = 1.50;
/// **Quanto a carga do rig ÂMBAR desce em 3 s** — com razão 0,25 a mesma carga
/// perde para o mesmo contrapeso, então ela SOBE e este número é negativo.
pub(crate) const MEASURED_TACKLE_LOAD_DROP: f32 = -1.13;
/// **Quanto o contrapeso da TALHA desce** para erguer aquilo. Ele anda `1/r` = 4
/// vezes o que a carga anda, e é essa troca — distância por força — que É uma
/// talha.
pub(crate) const MEASURED_TACKLE_CW_DROP: f32 = 4.51;

/// A massa da carga e a do contrapeso de cada rig, em kg. O par é o MESMO nos
/// dois — o que muda é só a razão, que é o ponto da cena.
const LOAD_KG: f32 = 3.0;
const COUNTERWEIGHT_KG: f32 = 1.0;
const R: f32 = 0.25;

fn ball(world: &mut World, name: &str, x: f32, mass: f32, rgba: [f32; 4], half: f32) {
    world.spawn((
        Name::new(name),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: half },
            density: mass / (std::f32::consts::PI * half * half),
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [half * 2.0, half * 2.0], rgba),
        Transform::from_translation(Vec2::new(x, START_Y)),
    ));
}

/// Um rig: carga + contrapeso + a corda que os une.
fn rig(world: &mut World, tag: &str, centre: f32, ratio: f32, rgba: [f32; 4]) {
    let load = format!("{tag} Load");
    let cw = format!("{tag} Counterweight");
    ball(world, &load, centre - SPAN / 2.0, LOAD_KG, rgba, R);
    ball(
        world,
        &cw,
        centre + SPAN / 2.0,
        COUNTERWEIGHT_KG,
        WEIGHT,
        R * 0.7,
    );
    world.spawn((
        Name::new(format!("{tag} Rope")),
        PhysicsJoint {
            body_a: stable_name_id(&load),
            body_b: stable_name_id(&cw),
            kind: JointKind::Pulley,
            ratio,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(centre - SPAN / 2.0, START_Y)),
    ));
}

/// Monta a cena inteira.
pub(crate) fn build(world: &mut World) {
    world.spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 14.0,
                half_y: 0.3,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [28.0, 0.6], GROUND),
        Transform::from_translation(Vec2::new(0.0, -2.0)),
    ));
    rig(world, "Simple", -4.5, 1.0, SIMPLE);
    rig(world, "Tackle", 4.5, 0.25, TACKLE);
}

impl crate::App {
    /// **Cena 58 (W-Pulley).** Dois elevadores, mesmo par de massas, e a única
    /// diferença é a razão da talha.
    pub(crate) fn physics_smoke_pulley(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build(gfx.sim.world_mut());
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 58] A POLIA -- uma corda por duas roldanas.\n  \
               Aperte B para ver os vinculos, e depois PLAY (o toggle Physics ja esta armado).\n\n  \
               1. OLHE A CORDA, nao os pesos. Ela NAO vai de um corpo ao outro: ela sobe\n     \
                  ate uma roldana, atravessa por cima, e desce ate a outra ponta. Os dois\n     \
                  aneis GROSSOS sao as roldanas.\n  \
               2. VERDE (esquerda), razao 1 -- o elevador com contrapeso. A carga de 3 kg\n     \
                  ganha do contrapeso de 1 kg e desce; o que um lado desce o outro sobe,\n     \
                  porque a corda nao estica. (medido em 3 s: desceu {simple_drop:.2} m e o\n     \
                  contrapeso subiu {simple_rise:.2} m -- o MESMO numero.)\n  \
               3. AMBAR (direita), razao 0.25 -- a TALHA, e MESMO PAR DE MASSAS. Agora o\n     \
                  contrapeso de 1 kg ERGUE a carga de 3 kg, e o preco esta a vista: ele\n     \
                  DESCE {tackle_cw:.2} m para levantar a carga {tackle_rise:.2} m -- quatro vezes\n     \
                  mais caminho, em troca de tres vezes o proprio peso. Isso e uma talha.\n  \
               4. Arraste uma ROLDANA. Ela e um ponto pregado no cenario -- nao pertence a\n     \
                  corpo nenhum -- e e o unico jeito de autora-la: nao ha campo de Inspector\n     \
                  para ela, porque o 'Position' do joint e a ANCORA. Mova uma para o lado e\n     \
                  de Play: a corda passa por onde voce a pos.\n  \
               5. Selecione uma corda ('Simple Rope') na Hierarquia. A secao do joint tem\n     \
                  'Ratio' e 'Rope Length (m)'. Ponha o Ratio do VERDE em 0.25 e de Play --\n     \
                  ele vira a talha. Ponha em 2 e a carga cai o DOBRO: a vantagem e do lado\n     \
                  B e vale 1/razao, entao razao MAIOR que 1 handicapa aquele lado. E note o que NAO esta la: nao ha 'Breakable'. Uma polia nao\n     \
                  vive no solver do rapier, entao nada mede a reacao dela, e uma caixa que\n     \
                  nunca pode disparar seria um controle so no nome.\n  \
               6. E o gesto de CRIAR, pelas DUAS rotas -- a segunda era a quebrada:\n     \
                  (a) selecione dois corpos, escolha 'Pulley' no seletor 'Join As' da\n     \
                      secao Physics Body e clique em Join;\n     \
                  (b) OU aperte sobre um corpo no canvas, arraste e solte sobre o outro.\n     \
                  Nas duas, as roldanas nascem ACIMA de cada corpo e a corda ja nasce\n     \
                  esticada. Pela rota (b) elas nasciam as duas na ORIGEM do mundo, com a\n     \
                  corda saindo de cada corpo ate la -- se voltar a ver isso, o rig nao\n     \
                  foi semeado.",
            simple_drop = MEASURED_SIMPLE_LOAD_DROP,
            simple_rise = MEASURED_SIMPLE_CW_RISE,
            tackle_rise = -MEASURED_TACKLE_LOAD_DROP,
            tackle_cw = MEASURED_TACKLE_CW_DROP,
        );
    }
}

#[cfg(test)]
#[path = "physics_smoke_pulley_tests.rs"]
mod tests;
