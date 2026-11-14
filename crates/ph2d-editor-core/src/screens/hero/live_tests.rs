//! N10 F6d: o `tick_hover` pelos ids mudados publica o MESMO `hover_live` que a varredura antiga.

use super::*;
use crate::interaction::state::InteractiveState;
use crate::motion::Role;
use crate::widget::ButtonState;
use ph2d_a11y::NodeId;

/// O caminho antigo, verbatim (o oráculo): todo widget, todo quadro.
fn tick_hover_varredura(hero: &mut HeroScreen, dt: f64) {
    let mut pending: Vec<(ph2d_a11y::NodeId, f32)> = hero.store.hover_targets().collect();
    let thumbs: Vec<(ph2d_a11y::NodeId, f32)> = hero.store.scrollbar_hover_targets().collect();
    for slot in &mut pending {
        slot.1 = hero
            .motion
            .animate(slot.0, slot.1, crate::motion::Role::Number);
    }
    for (id, target) in thumbs {
        if hero.motion.get(id).is_none() {
            hero.motion.animate(id, 0.0, crate::motion::Role::Fade);
        }
        let live = hero.motion.animate(id, target, crate::motion::Role::Fade);
        pending.push((id, live));
    }
    hero.motion.advance(dt);
    hero.store.note_reduced_motion(hero.motion.reduced_motion());
    for (id, seeded) in pending {
        let live = hero.motion.get(id).unwrap_or(seeded);
        hero.store.set_hover_live(id, live);
    }
}

fn mexe(hero: &mut HeroScreen, quadro: u64, semente: &mut u64) {
    let ids = 1000..1040u64;
    if quadro == 0 {
        for i in ids.clone() {
            let st = if i % 5 == 0 {
                InteractiveState::Plain
            } else {
                InteractiveState::Button {
                    state: ButtonState::Normal,
                }
            };
            hero.store.register(NodeId(i), st);
        }
    }
    for i in ids.clone() {
        *semente = semente
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        if (*semente >> 60) == 0
            && let Some(InteractiveState::Button { state }) = hero.store.get_mut(NodeId(i))
        {
            *state = if *state == ButtonState::Normal {
                ButtonState::Hovered
            } else {
                ButtonState::Normal
            };
        }
    }
    match quadro {
        7 => hero.store.unregister(NodeId(1003)),
        9 => hero.store.register(
            NodeId(1003),
            InteractiveState::Button {
                state: ButtonState::Pressed,
            },
        ),
        // Uma track de outro papel num widget: a varredura devolvia-o ao instantâneo.
        12 => {
            hero.motion.animate(NodeId(1007), 0.0, Role::Fade);
            hero.motion.animate(NodeId(1007), 0.4, Role::Fade);
        }
        15 => {
            let e = hero.store.abre_ensaio();
            let _ = hero.store.fecha_ensaio(e);
        }
        _ => {}
    }
}

#[test]
fn o_hover_vivo_pelos_mudados_e_o_da_varredura() {
    let (mut novo, mut velho) = (HeroScreen::new(NodeId(1)), HeroScreen::new(NodeId(1)));
    let (mut sa, mut sb) = (7_u64, 7_u64);
    let mut mudou = 0;
    for quadro in 0..40 {
        mexe(&mut novo, quadro, &mut sa);
        mexe(&mut velho, quadro, &mut sb);
        tick_hover(&mut novo, 1.0 / 60.0);
        tick_hover_varredura(&mut velho, 1.0 / 60.0);
        let (a, b) = (
            novo.store.hover_live_retrato(),
            velho.store.hover_live_retrato(),
        );
        assert_eq!(a, b, "quadro {quadro}: hover_live");
        assert_eq!(
            novo.motion.track_ids().collect::<Vec<_>>(),
            velho.motion.track_ids().collect::<Vec<_>>(),
            "quadro {quadro}: tracks"
        );
        mudou += a.values().filter(|t| **t == 1.0).count();
    }
    assert!(mudou > 0, "o controlo tem widgets acesos");
}
