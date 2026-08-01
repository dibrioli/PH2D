//! **A cena do CORPO COMPOSTO** (`PH2D_PHYSICS_SMOKE=69`, W-Compound).
//!
//! Duas mesas iguais caem sobre o chão. Na da esquerda o tampo é o corpo e as
//! pernas são só desenho; na da direita as pernas são **peças** — filhos que
//! carregam `Collider` e não `RigidBody`.
//!
//! ⚠️ **A cena nasce com a esquerda JÁ ERRADA de propósito**, porque o defeito
//! que esta wave fecha é silencioso: sem peça, as pernas não existem para o
//! solver e o tampo desce até encostar sozinho — as pernas atravessam o chão e
//! nada avisa.
//!
//! Os números da mensagem saem da sonda `probe_smoke_69`.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, Transform, World};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const TOP_HALF: [f32; 2] = [1.0, 0.15];
const LEG_HALF: [f32; 2] = [0.15, 0.9];
const DROP_Y: f32 = 6.0;

/// Onde cada mesa nasce.
///
/// ⚠️ **Dentro do vão do chão do smoke**, que vai de `x = −4` a `+4` (`spawn_floor`).
/// A primeira versão pôs as mesas em ±3,2 com tampo de 2,8 m: a da direita
/// passava da borda, tombava e caía para **−74,88 m**, e eu quase reportei isso
/// como defeito do produto. A da esquerda, essa sim, media certo — o CONTROLE
/// funcionando é o que separou os dois casos.
pub(crate) const LANES: [f32; 2] = [-2.2, 2.2];

const WOOD: [f32; 4] = [0.70, 0.52, 0.32, 1.0];
const LEG_BARE: [f32; 4] = [0.85, 0.40, 0.35, 1.0];
const LEG_PART: [f32; 4] = [0.40, 0.75, 0.50, 1.0];

const CAMERA_CENTRE: [f32; 2] = [0.0, 3.0];
const CAMERA_HEIGHT: f32 = 12.0;

/// **MEDIDO** pela sonda: a altura em que o TAMPO de cada mesa descansa, e a da
/// ponta de baixo de uma perna.
///
/// A da esquerda desce até o próprio tampo tocar o chão; a da direita para sobre
/// as pernas — a diferença é uma altura de perna inteira.
pub(crate) const MEASURED_TOP_Y: [f32; 2] = [-0.65, 1.15];
/// A ponta de baixo da perna. Na esquerda ela fica **1,8 m abaixo do chão** (cujo
/// topo está em −0,80): o desenho atravessa inteiro, e é isso que ninguém avisa.
/// Na direita ela pousa exatamente no chão.
pub(crate) const MEASURED_LEG_TIP_Y: [f32; 2] = [-2.60, -0.80];

/// Uma mesa: tampo (corpo) + duas pernas (filhos). `parts` decide se as pernas
/// carregam `Collider` — a única diferença entre as duas faixas.
fn table(world: &mut World, i: usize, name: &str, parts: bool) -> Entity {
    let x = LANES[i];
    let top = world
        .spawn((
            Name::new(format!("{name} Top")),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: TOP_HALF[0],
                    half_y: TOP_HALF[1],
                },
                ..Collider::default()
            },
            Sprite::atlas(WHITE_TILE_KEY, [TOP_HALF[0] * 2.0, TOP_HALF[1] * 2.0], WOOD),
            Transform::from_translation(Vec2::new(x, DROP_Y)),
        ))
        .id();
    let tint = if parts { LEG_PART } else { LEG_BARE };
    for (k, dx) in [(-1.0f32), 1.0].into_iter().enumerate() {
        let mut leg = world.spawn((
            Name::new(format!("{name} Leg {}", k + 1)),
            Sprite::atlas(WHITE_TILE_KEY, [LEG_HALF[0] * 2.0, LEG_HALF[1] * 2.0], tint),
            // Local: pendurada sob o tampo, na ponta.
            Transform::from_translation(Vec2::new(
                dx * (TOP_HALF[0] - LEG_HALF[0]),
                -(TOP_HALF[1] + LEG_HALF[1]),
            )),
            ChildOf(top),
        ));
        if parts {
            // ⚠️ **`Collider` e NÃO `RigidBody`** — é a ausência do segundo que
            // faz desta perna uma FORMA do tampo em vez de um segundo corpo.
            leg.insert(Collider {
                shape: ColliderShape::Cuboid {
                    half_x: LEG_HALF[0],
                    half_y: LEG_HALF[1],
                },
                ..Collider::default()
            });
        }
    }
    top
}

pub(crate) fn build_compound(world: &mut World) {
    table(world, 0, "Bare", false);
    table(world, 1, "Solid", true);
}

#[cfg(test)]
#[path = "physics_smoke_compound_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 69 (W-Compound).** Duas mesas iguais; só uma tem pernas de verdade.
    pub(crate) fn physics_smoke_compound(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_compound(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 69] O CORPO COMPOSTO -- ate aqui um corpo tinha UMA forma,\n  \
               e a query da ponte dizia isso no tipo. Um artista que desenhava uma mesa\n  \
               recebia metade dela sem fisica, EM SILENCIO.\n\n  \
               As duas mesas sao IDENTICAS no desenho: um tampo e duas pernas, filhas\n  \
               dele na Hierarquia. So o COLLIDER das pernas difere.\n\n  \
               1. ESQUERDA (pernas VERMELHAS) -- as pernas sao so desenho. O tampo\n     \
                  desce ate encostar ELE no chao ({t0:.2} m) e as pernas ATRAVESSAM:\n     \
                  a ponta de baixo delas para em {l0:.2} m -- 1,8 m ABAIXO do chao,\n     \
                  cujo topo esta em -0,80.\n  \
               2. DIREITA (pernas VERDES) -- cada perna carrega um `Collider` e NAO um\n     \
                  `RigidBody`, entao ela e mais uma FORMA do tampo. A mesa para sobre\n     \
                  as pernas: tampo em {t1:.2} m, ponta em {l1:.2} m.\n\n  \
               (!) Toque B: o contorno agora desenha as PECAS tambem. Antes desta wave\n     \
               ele so desenhava colliders que fossem corpos, entao uma peca era\n     \
               invisivel -- e um collider invisivel e exatamente o que o contorno\n     \
               existe para nao deixar acontecer.\n\n  \
               AUTORE VOCE MESMO: selecione 'Bare Leg 1' na Hierarquia. A secao\n  \
               Physics Body oferece TRES portas, e a do meio e nova:\n  \
               - 'Add Physics Body' faz dela um corpo PROPRIO (e ai a mesa se\n    \
                 desmonta: duas massas que o solver pode separar);\n  \
               - 'Add Shape to Bare Top' faz dela uma PECA do tampo -- e a mesa da\n    \
                 esquerda passa a se comportar como a da direita;\n  \
               - 'Rig N Parts' monta o personagem inteiro (W-Rig).\n  \
               O rotulo NOMEIA o dono porque um collider e invisivel e a hierarquia\n  \
               pode ter um grupo no meio.\n\n  \
               (!) SCRUB: toque Play, deixe assentar e arraste a regua PARA TRAS ate o\n     \
               zero. As duas mesas tem de cair de novo IGUAL. Se a da direita afundar\n     \
               no replay, as pecas nao voltaram ao mundo reconstruido -- a licao do\n     \
               Weston.\n",
            t0 = MEASURED_TOP_Y[0],
            l0 = MEASURED_LEG_TIP_Y[0],
            t1 = MEASURED_TOP_Y[1],
            l1 = MEASURED_LEG_TIP_Y[1],
        );
    }
}
