//! Cena 12 — **Partículas e mola** (`docs/BlenderConstraints/07_W6_pontas.md`, pontas 4 e 5): à esquerda a
//! FONTE, um objecto Motion (formas numa simulação com campos: gravidade e um redemoinho) que lança bolinhas; a
//! BOLA laranja tem um Copy Particle preso à partícula [`PARTICULA`] — no Play ela anda montada nela e, quando a
//! partícula morre, fica na pose dela. À direita a ANTENA (a da cena 5, parada): com a timeline PARADA, arraste o
//! CORPO e a PONTA balança e assenta (a mola ao vivo).
//!
//! O grafo da Fonte é do documento do Motion: quem o monta é o smoke do Motion
//! (`ph2d_app_motion::motion_objeto::smoke_particula`), quando a entidade [`FONTE`] nasce.

use std::sync::atomic::{AtomicBool, Ordering};

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_constraints::{Kind, PontoDeSimulacao};
use ph2d_core::Playhead;
use ph2d_ecs::{GlobalTransform, MotionGenerator, Name, SimWorld, Transform};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_render::DISC_TILE_KEY;
use ph2d_timeline::TimelineState;

/// O nome do objecto Motion — o smoke do Motion procura-o por ele.
pub const FONTE: &str = "Fonte";
/// Onde a Fonte, a Bola (a pose dela) e o Corpo da antena ficam.
pub const FONTE_EM: [f32; 2] = [-3.0, -1.6];
pub const BOLA_EM: [f32; 2] = [0.0, 2.5];
pub const ANTENA_EM: [f32; 2] = [3.5, -1.2];
/// A partícula a que a Bola se prende (o ordinal de emissão).
pub const PARTICULA: u32 = 1;
/// O laço do Play, em segundos: a partícula vive menos que isto e a Bola volta à pose antes do recomeço.
pub const LACO_S: f64 = 4.0;

/// Cena 12. Abre com o Corpo da antena escolhido.
pub(crate) fn particulas_e_mola(world: &mut World) -> Entity {
    let mut t = Transform::IDENTITY;
    t.translation.x = FONTE_EM[0];
    t.translation.y = FONTE_EM[1];
    let fonte = world
        .spawn((
            Name::new(FONTE),
            t,
            GlobalTransform::from_transform(t),
            MotionGenerator,
        ))
        .id();
    let bola = crate::smoke::peca(
        world,
        "Bola",
        DISC_TILE_KEY,
        BOLA_EM,
        0.0,
        [0.8, 0.8],
        [1.0, 0.5, 0.1, 1.0],
    );
    crate::smoke::pilha(
        world,
        bola,
        &[(
            Kind::PontoDeSimulacao(PontoDeSimulacao { id: PARTICULA }),
            Some(fonte),
            1.0,
        )],
    );
    crate::smoke::antena_em(world, ANTENA_EM).0
}

static FEITA: AtomicBool = AtomicBool::new(false);

/// Uma vez, com `PH2D_CONSTRAINT_SMOKE=12`: o laço de [`LACO_S`] e a timeline à vista, PARADA (a mola ao vivo
/// é com o tempo parado; o Play mostra a Bola na partícula).
pub fn step(
    _sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    if crate::smoke::nivel_armado() != Some(12) || FEITA.swap(true, Ordering::Relaxed) {
        return None;
    }
    for keys in [true, false] {
        tl.doc.set_active_loop_for(keys, Some((0.0, LACO_S)));
    }
    ph.set_loop(0.0, LACO_S);
    ph.seek_frame(0, tl.doc.fps_display);
    // A foto do meio do Play (`fotografa_cena.sh` não clica): `PH2D_CONSTRAINT_SMOKE_TOCA=1` abre a tocar.
    if std::env::var("PH2D_CONSTRAINT_SMOKE_TOCA").is_ok_and(|v| v == "1") {
        ph.play();
    } else {
        ph.pause();
    }
    hero.set_panel_visible("timeline", true);
    Some(
        "[constraint-smoke] =12 parado: arraste o Corpo da antena e a ponta balança; dê Play e a Bola anda \
         numa partícula da Fonte até ela morrer"
            .to_owned(),
    )
}
