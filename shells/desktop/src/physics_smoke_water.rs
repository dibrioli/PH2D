//! **O PERSONAGEM E A ÁGUA** (`PH2D_PHYSICS_SMOKE=100`).
//!
//! Uma cena, TRÊS waves — e elas partilham a cena porque se tocam:
//!
//! - **`W-Water`** — ele não pode ficar **de pé sobre a poça**. Medido antes da
//!   cura: assentava em `y = 0,9023`, a `float_height` exacta acima da
//!   superfície, e imune à correnteza.
//! - **`W-ClingPull`** — a jangada não pode **subir ao encontro** de quem cai
//!   nela. Medido: `+96,90 mm` antes de descer.
//! - **`W-Landing`** — o pouso não pode **escorregar**. Medido: `0,500 s` do
//!   toque ao repouso, com o perfil de um decaimento e não de uma parada.
//!
//! ⚠️ **Separá-las em três cenas faria o artista julgar cada uma sem ver que a
//! terceira PIORA a segunda:** a cura do pouso é uma perna mais rígida, e uma
//! perna mais rígida amplifica tanto o empurrão quanto a puxada.
//!
//! Os números da mensagem saem da sonda `probe_smoke_100`.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World};
use ph2d_physics_ecs::{
    AreaBuoyancy, AreaDrag, BodyKind, Collider, ColliderShape, LockRotation, PlatformPlayer,
    RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// A poça, o cais e a jangada.
const POOL_HALF: [f32; 2] = [14.0, 3.0];
const POOL_CENTRE: [f32; 2] = [7.0, -3.0];
// ⚠️ **Esta cena NÃO tem correnteza, e a ausência é uma decisão medida.** Ela
// encenaria a metade que a `W-Water` **não** entrega (o freio do ar ainda luta
// contra a corrente: o personagem viaja menos de 1% do que uma cápsula viaja) —
// e, pior, varreria a jangada e o controle para fora da poça enquanto o artista
// olha (medido na 1ª versão: −810 e −1564 em vinte segundos). O item aberto é
// nomeado na mensagem, que é onde um item aberto pertence.
const DOCK_HALF: [f32; 2] = [4.0, 0.5];
const RAFT_HALF: [f32; 2] = [1.8, 0.25];
/// ⚠️ **A jangada é DENSA, e não é decoração.** Com densidade 1 ela pesa
/// `0,96 kg` contra os `0,37 kg` do personagem, e ele a **atropela**: medido na
/// 1ª versão, ela viajava 17 m ao ser atingida e saía da poça, onde não há
/// arrasto que a pare. Uma jangada em que se pisa é uma PLATAFORMA.
const RAFT_DENSITY: f32 = 2.0;
const FLUID_DENSITY: f32 = 4.0;

/// A cápsula do personagem e a altura a que a perna o segura.
const CAP_HALF_H: f32 = 0.3;
const CAP_RADIUS: f32 = 0.2;
const FLOAT: f32 = 0.9;

/// **MEDIDO** (`probe_smoke_100`), e cada um é o número de uma wave.
///
/// - `0` — onde o personagem assenta na água **hoje**. Uma cápsula idêntica sem
///   `PlatformPlayer` (o CONTROLE) assenta em `WATERLINE`.
/// - `1` — quanto a jangada sobe ao encontro dele no primeiro toque.
/// - `2` — segundos do toque ao repouso, num pouso no cais.
pub(crate) const MEASURED: [f32; 3] = [0.342, 0.0, 0.133];
/// A linha d'água de uma cápsula idêntica **sem** o player — o oráculo da cena.
pub(crate) const WATERLINE: f32 = 0.227;

const DOCK_RGBA: [f32; 4] = [0.55, 0.50, 0.44, 1.0];
const RAFT_RGBA: [f32; 4] = [0.72, 0.74, 0.80, 1.0];
const WATER_RGBA: [f32; 4] = [0.20, 0.34, 0.46, 1.0];
const PLAYER_RGBA: [f32; 4] = [0.90, 0.72, 0.32, 1.0];
const CONTROL_RGBA: [f32; 4] = [0.45, 0.78, 0.62, 1.0];

const CAMERA_CENTRE: [f32; 2] = [4.0, 0.2];
const CAMERA_HEIGHT: f32 = 10.0;

fn cuboid(half_x: f32, half_y: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid { half_x, half_y },
        density: 1.0,
        ..Collider::default()
    }
}

pub(crate) fn build_water_scene(world: &mut World) {
    // O CAIS — sólido, topo em y = 0, de onde o artista anda e pula.
    world.spawn((
        Name::new("Dock"),
        RigidBody {
            kind: BodyKind::Static,
        },
        cuboid(DOCK_HALF[0], DOCK_HALF[1]),
        Sprite::atlas(
            WHITE_TILE_KEY,
            [DOCK_HALF[0] * 2.0, DOCK_HALF[1] * 2.0],
            DOCK_RGBA,
        ),
        Transform::from_translation(Vec2::new(-DOCK_HALF[0], -DOCK_HALF[1])),
    ));

    // A POÇA — um SENSOR, e é por isso que a cena existe.
    world.spawn((
        Name::new("Pool"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            ..cuboid(POOL_HALF[0], POOL_HALF[1])
        },
        AreaBuoyancy(FLUID_DENSITY),
        AreaDrag(0.6),
        Sprite::atlas(
            WHITE_TILE_KEY,
            [POOL_HALF[0] * 2.0, POOL_HALF[1] * 2.0],
            WATER_RGBA,
        ),
        Transform::from_translation(Vec2::new(POOL_CENTRE[0], POOL_CENTRE[1])),
    ));

    // A JANGADA — perto do cais, ao alcance de um pulo.
    world.spawn((
        Name::new("Raft"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            density: RAFT_DENSITY,
            ..cuboid(RAFT_HALF[0], RAFT_HALF[1])
        },
        LockRotation,
        Sprite::atlas(
            WHITE_TILE_KEY,
            [RAFT_HALF[0] * 2.0, RAFT_HALF[1] * 2.0],
            RAFT_RGBA,
        ),
        Transform::from_translation(Vec2::new(3.0, 0.6)),
    ));

    // ⚠️ **O CONTROLE, e ele faz parte da cena e não do teste:** uma cápsula
    // IDÊNTICA sem `PlatformPlayer`, boiando ao lado. Sem ela o artista não tem
    // com que comparar a linha d'água do personagem, e *"parece certo"* não é um
    // veredito.
    world.spawn((
        Name::new("Control Capsule"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: CAP_HALF_H,
                radius: CAP_RADIUS,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        Sprite::atlas(
            WHITE_TILE_KEY,
            [CAP_RADIUS * 2.0, (CAP_HALF_H + CAP_RADIUS) * 2.0],
            CONTROL_RGBA,
        ),
        Transform::from_translation(Vec2::new(16.0, 1.0)),
    ));

    // O PERSONAGEM, em pé no cais.
    world.spawn((
        Name::new("Player"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: CAP_HALF_H,
                radius: CAP_RADIUS,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height: FLOAT,
            ..PlatformPlayer::default()
        },
        Sprite::atlas(
            WHITE_TILE_KEY,
            [CAP_RADIUS * 2.0, (CAP_HALF_H + CAP_RADIUS) * 2.0],
            PLAYER_RGBA,
        ),
        Transform::from_translation(Vec2::new(-5.0, FLOAT)),
    ));
}

#[cfg(test)]
#[path = "physics_smoke_water_tests.rs"]
mod tests;

impl crate::App {
    pub(crate) fn physics_smoke_water(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_water_scene(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 100] O PERSONAGEM E A AGUA -- tres reports numa cena so'.\n  \
               Um cais solido a esquerda, uma poca a direita, uma jangada a boiar, e uma\n  \
               CAPSULA VERDE que e' o CONTROLE: mesma forma e mesma densidade do\n  \
               personagem, sem o componente de player. Ela boia em {line:.3}.\n\n  \
               1. A AGUA NAO E' CHAO (W-Water). Ande para a DIREITA (D) e caia na poca.\n     \
                  Ele tem de MOLHAR O PE'. O QUE ESTAVA QUEBRADO: ele ficava DE PE' sobre\n     \
                  a superficie, em 0,903 -- a altura de flutuacao exacta acima da agua --\n     \
                  e a correnteza nao o movia um milimetro. O sensor de chao tratava um\n     \
                  SENSOR como pedra, e o mesmo raio serve o teto e a parede: ele tambem\n     \
                  batia a cabeca num volume de gatilho (0,17 m de desvio, medido).\n\n  \
               (!) ABERTO, E E' O PROXIMO ITEM: submerso ele ainda NAO e' estavel. Os\n      \
                   multiplicadores de gravidade do pulo (peak 0,5 / fall 2,0) sao uma\n      \
                   BOMBA DE ENERGIA quando o empuxo e' quem o segura -- medido, largado\n      \
                   dentro da poca ele oscila -1,05 / +4,71 / +12,08 / -20,31 e sai da\n      \
                   cena; com os multiplicadores neutros ele converge para {line:.3}, a\n      \
                   linha do controle. Entrando de CIMA ele assenta em {y0:.3} -- 11 cm\n      \
                   acima do controle, pelo mesmo termo. Nao mergulhe fundo neste smoke.\n\n  \
               2. A JANGADA NAO SOBE AO SEU ENCONTRO (W-ClingPull). Reset, e pule (Espaco)\n     \
                  do cais para a jangada. No primeiro toque ela tem de ir para BAIXO.\n     \
                  O QUE ESTAVA QUEBRADO: ela SUBIA 96,90 mm ao encontro dele antes de\n     \
                  descer, porque a metade esticada da perna PUXA -- e a 3a lei transmitia\n     \
                  a puxada. Hoje: {rise:.2} mm.\n\n  \
               3. O POUSO NAO ESCORREGA (W-Landing). Pule no cais e olhe a aterragem: ela\n     \
                  tem de PARAR, nao derrapar. Do toque ao repouso: {land:.3} s.\n     \
                  O QUE ESTAVA QUEBRADO: 0,500 s, e o perfil era um decaimento -- ele se\n     \
                  aproximava do chao para sempre sem chegar.\n\n  \
               (!) A rigidez da perna passou de 400 para 2000, e o teto MEDIDO e' 3600\n      \
                   (= 1/dt^2): acima dele a mola passa do alvo e o personagem afunda e\n      \
                   range. Experimente no Inspector -- 1200 da' 0,217 s, 3600 da' 0,033 s\n      \
                   -- e repare que a DERIVA de rampa fica em zero em toda a faixa.\n\n  \
               (!) Toque B para o contorno. A poca e' magenta (sensor) e o personagem\n      \
                   ciano (dinamico); a capsula verde nao tem perna nenhuma.\n",
            line = WATERLINE,
            y0 = MEASURED[0],
            rise = MEASURED[1],
            land = MEASURED[2],
        );
    }
}
