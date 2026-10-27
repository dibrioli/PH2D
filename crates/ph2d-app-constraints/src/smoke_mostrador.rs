//! Cena 9 — **Mostrador** (`docs/BlenderConstraints/06_W5_resto.md` §6, a primeira peça): a BÓIA sobe e
//! desce (curva da timeline, em laço); o PONTEIRO do mostrador segue a altura dela com mola (Follow Value
//! → rotação: passa, volta e assenta), e a LÂMPADA acende com ela sem mola (Follow Value → a opacidade,
//! um campo que não é pose). Abre com o ponteiro escolhido.

use std::sync::atomic::{AtomicBool, Ordering};

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_anim::RationalTime;
use ph2d_constraints::{Canal, Kind, Mola, Prop, Valor};
use ph2d_core::Playhead;
use ph2d_ecs::canal::familia::OBJETO;
use ph2d_ecs::canal_objeto::OPACIDADE;
use ph2d_ecs::{ChildOf, GlobalTransform, Name, SimWorld, Transform};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_render::{DISC_TILE_KEY, WHITE_TILE_KEY};
use ph2d_timeline::{
    AnimValue, Easing, EasingFamily, EasingMode, Interp, PropKind, TimelineIntent, TimelineState,
};

use crate::smoke::{peca, pilha};

/// A altura da bóia: `(quadro, y)` — sobe depressa, espera, desce depressa, espera.
pub const BOIA: [(i64, f32); 5] = [(0, -1.5), (6, 1.5), (30, 1.5), (36, -1.5), (60, -1.5)];
/// O ponteiro vai de 225° (a bóia em baixo) a −45° (em cima), pelo alto do mostrador.
pub const PONTEIRO_GRAUS: [f64; 2] = [225.0, -45.0];
const COMPRIMENTO: f32 = 1.4;

static FEITA: AtomicBool = AtomicBool::new(false);

/// ⭐ Monta a cena 9. Devolve o ponteiro.
pub(crate) fn mostrador(world: &mut World) -> Entity {
    let boia = peca(
        world,
        "Bóia",
        DISC_TILE_KEY,
        [-3.0, BOIA[0].1],
        0.0,
        [0.7, 0.7],
        [0.2, 0.6, 0.95, 1.0],
    );
    let _ = peca(
        world,
        "Mostrador",
        DISC_TILE_KEY,
        [0.5, 0.0],
        0.0,
        [3.4, 3.4],
        [0.25, 0.25, 0.28, 1.0],
    );
    let mut t = Transform::IDENTITY;
    t.translation.x = 0.5;
    let ponteiro = world
        .spawn((Name::new("Ponteiro"), t, GlobalTransform::from_transform(t)))
        .id();
    let agulha = peca(
        world,
        "Agulha",
        WHITE_TILE_KEY,
        [COMPRIMENTO / 2.0, 0.0],
        0.0,
        [COMPRIMENTO, 0.12],
        [0.95, 0.35, 0.2, 1.0],
    );
    world.entity_mut(agulha).insert(ChildOf(ponteiro));
    let lampada = peca(
        world,
        "Lâmpada",
        DISC_TILE_KEY,
        [3.5, 0.0],
        0.0,
        [1.0, 1.0],
        [1.0, 0.85, 0.3, 1.0],
    );
    let [baixo, cima] = [BOIA[0].1, BOIA[1].1].map(f64::from);
    let ponteiro_segue = Valor {
        fonte: Prop::Pose(Canal::Y),
        de_faixa: [baixo, cima],
        destino: Prop::Pose(Canal::Rot),
        para_faixa: PONTEIRO_GRAUS.map(f64::to_radians),
        mola: Some(Mola {
            duracao: 0.6,
            ressalto: 0.6,
        }),
        ..Valor::default()
    };
    let lampada_segue = Valor {
        fonte: Prop::Pose(Canal::Y),
        de_faixa: [baixo, cima],
        destino: Prop::Campo([OBJETO, OPACIDADE, 0, 0]),
        para_faixa: [0.15, 1.0],
        mola: None,
        ..Valor::default()
    };
    pilha(
        world,
        ponteiro,
        &[(Kind::Valor(ponteiro_segue), Some(boia), 1.0)],
    );
    pilha(
        world,
        lampada,
        &[(Kind::Valor(lampada_segue), Some(boia), 1.0)],
    );
    ponteiro
}

/// As chaves da bóia ([`BOIA`]), a acelerar e a travar.
#[must_use]
pub fn chaves_da_boia(boia: u64, fps: f64) -> Vec<TimelineIntent> {
    #[expect(clippy::cast_possible_truncation, reason = "fps inteiros")]
    let fps = fps.round() as u32;
    BOIA.iter()
        .map(|&(q, y)| TimelineIntent::AddKey {
            entity: boia,
            prop: PropKind::TranslationY,
            t: RationalTime::from_frame(q, fps),
            value: AnimValue::Float(y),
            interp: Interp::Eased(Easing::new(EasingFamily::Cubic, EasingMode::InOut)),
        })
        .collect()
}

/// Uma vez, com `PH2D_CONSTRAINT_SMOKE=9`: grava a curva da bóia, mostra a timeline e o cartão, e toca
/// em laço.
pub fn step(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    if crate::smoke::nivel_armado() != Some(9) || FEITA.swap(true, Ordering::Relaxed) {
        return None;
    }
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    let boia = q.iter(w).find(|(_, n)| n.0 == "Bóia").map(|(e, _)| e)?;
    for i in chaves_da_boia(boia.to_bits(), tl.doc.fps_display) {
        ph2d_timeline::apply_intent(tl, ph, i);
    }
    hero.set_panel_visible("timeline", true);
    hero.set_panel_visible("inspector", true);
    hero.store
        .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_CONSTRAINTS_SECTION, false);
    #[expect(clippy::cast_precision_loss, reason = "quadros da cena")]
    let fim = BOIA[BOIA.len() - 1].0 as f64 / tl.doc.fps_display;
    for keys in [true, false] {
        tl.doc.set_active_loop_for(keys, Some((0.0, fim)));
    }
    ph.set_loop(0.0, fim);
    ph.seek_frame(0, tl.doc.fps_display);
    ph.play();
    Some("[constraint-smoke] =9 a curva da bóia gravada; a tocar".to_owned())
}
