//! **A cena 5 conduzida** (`PH2D_CONSTRAINT_SMOKE=5`): grava a curva do corpo e toca em laço. A sonda que mede
//! tocar × arrastar o tempo é o teste `a_cena_5_toca_e_arrasta_igual` (mesmas chaves, [`chaves_do_corpo`]).

use std::sync::atomic::{AtomicBool, Ordering};

use ph2d_anim::RationalTime;
use ph2d_core::Playhead;
use ph2d_ecs::{Name, SimWorld};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_timeline::{
    AnimValue, Easing, EasingFamily, EasingMode, Interp, PropKind, TimelineIntent, TimelineState,
};

use crate::smoke::CORPO;

static FEITA: AtomicBool = AtomicBool::new(false);

/// As chaves da curva do corpo ([`CORPO`]), a acelerar e a travar (cúbica entrada-saída): um corpo
/// real não muda de sentido de uma vez.
#[must_use]
pub fn chaves_do_corpo(corpo: u64, fps: f64) -> Vec<TimelineIntent> {
    #[expect(clippy::cast_possible_truncation, reason = "fps inteiros")]
    let fps = fps.round() as u32;
    CORPO
        .iter()
        .flat_map(|&(q, x, y)| {
            [(PropKind::TranslationX, x), (PropKind::TranslationY, y)].map(|(prop, v)| {
                TimelineIntent::AddKey {
                    entity: corpo,
                    prop,
                    t: RationalTime::from_frame(q, fps),
                    value: AnimValue::Float(v),
                    interp: Interp::Eased(Easing::new(EasingFamily::Cubic, EasingMode::InOut)),
                }
            })
        })
        .collect()
}

/// Uma vez, com `PH2D_CONSTRAINT_SMOKE=5`: grava a curva, mostra a timeline e o cartão, e toca.
pub fn step(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    if crate::smoke::nivel_armado() != Some(5) || FEITA.swap(true, Ordering::Relaxed) {
        return None;
    }
    let w = sim.world_mut();
    let mut q = w.query::<(bevy_ecs::entity::Entity, &Name)>();
    let corpo = q.iter(w).find(|(_, n)| n.0 == "Corpo").map(|(e, _)| e)?;
    for i in chaves_do_corpo(corpo.to_bits(), tl.doc.fps_display) {
        ph2d_timeline::apply_intent(tl, ph, i);
    }
    hero.set_panel_visible("timeline", true);
    hero.set_panel_visible("inspector", true);
    hero.store
        .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_CONSTRAINTS_SECTION, false);
    // Em laço até 1 s depois de o corpo parar: no fim a mola já assentou e o recomeço não salta.
    #[expect(clippy::cast_precision_loss, reason = "quadros da cena")]
    let fim = (CORPO[CORPO.len() - 1].0 + 24) as f64 / tl.doc.fps_display;
    // O laço é do CLIP, por vista (Keys e Arrange); o transporte espelha-o.
    for keys in [true, false] {
        tl.doc.set_active_loop_for(keys, Some((0.0, fim)));
    }
    ph.set_loop(0.0, fim);
    ph.seek_frame(0, tl.doc.fps_display);
    ph.play();
    Some("[constraint-smoke] =5 a curva do corpo gravada; a tocar".to_owned())
}
