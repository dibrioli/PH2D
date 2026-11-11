//! Cena 13 — **Interruptor e lâmpada** (F41, o Follow Value sobre uma CAIXA depois do `TimelineDoc` v25:
//! `0` = desmarcada, `1` = marcada). O SWITCH tem a caixa «Enable 9-slice» animada (desmarcada no 0,
//! marcada no 30, desmarcada no 60, em degrau) e sobe enquanto ela está marcada. A LAMP segue as duas:
//! a opacidade («Opacity») lê a caixa do Switch (Follow Value caixa → Opacity, 0,2 … 1) e a caixa dela lê a
//! altura do Switch (Follow Value altura → caixa). Abre com a Lamp escolhida.

use std::sync::atomic::{AtomicBool, Ordering};

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_anim::RationalTime;
use ph2d_constraints::{Canal, Kind, Prop, Valor};
use ph2d_core::Playhead;
use ph2d_ecs::slice_nine::SliceNine;
use ph2d_ecs::{FieldKey, Name, SimWorld};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_timeline::{
    AnimValue, Easing, EasingFamily, EasingMode, Interp, PropKind, TimelineIntent, TimelineState,
};

use crate::smoke::{bloco, disco, pilha};

pub const SWITCH: &str = "Switch";
pub const LAMP: &str = "Lamp";
/// A caixa do Switch: `(quadro, 0 = desmarcada | 1 = marcada)`, em degrau.
pub const CAIXA: [(i64, f32); 3] = [(0, 0.0), (30, 1.0), (60, 0.0)];
/// A altura do Switch: em baixo, sobe até ao 30, fica, desce até ao 60.
pub const ALTURA: [(i64, f32); 4] = [(0, -1.0), (24, -1.0), (30, 1.0), (54, 1.0)];
/// A opacidade da Lamp com a caixa do Switch desmarcada e marcada.
pub const LUZ: [f64; 2] = [0.2, 1.0];

/// A chave do canal «Enable 9-slice» (a caixa `View::Toggle(Sliced, Simple)` do cartão 9-Slice).
#[must_use]
pub fn caixa() -> FieldKey {
    ph2d_ecs::canal_componente::chaves_do_rotulo("component.field.slice_nine.8")[0]
}

fn prop(k: FieldKey) -> Prop {
    Prop::Campo([k.familia, k.campo, k.i, k.j])
}

static FEITA: AtomicBool = AtomicBool::new(false);

/// ⭐ Monta a cena 13. Devolve a Lamp.
pub(crate) fn interruptor(world: &mut World) -> Entity {
    let switch = bloco(
        world,
        SWITCH,
        [-2.0, ALTURA[0].1],
        [0.9, 0.9],
        [0.2, 0.6, 0.95],
    );
    let lamp = disco(world, LAMP, [2.0, 0.0], [1.6, 1.6], [1.0, 0.85, 0.3]);
    world.entity_mut(switch).insert(SliceNine::INERT);
    world.entity_mut(lamp).insert(SliceNine::INERT);
    let luz = Valor {
        fonte: prop(caixa()),
        de_faixa: [0.0, 1.0],
        destino: Prop::Opacidade,
        para_faixa: LUZ,
        mola: None,
        ..Valor::default()
    };
    let marca = Valor {
        fonte: Prop::Pose(Canal::Y),
        de_faixa: [ALTURA[0].1, ALTURA[2].1].map(f64::from),
        destino: prop(caixa()),
        para_faixa: [0.0, 1.0],
        mola: None,
        ..Valor::default()
    };
    pilha(
        world,
        lamp,
        &[
            (Kind::Valor(luz), Some(switch), 1.0),
            (Kind::Valor(marca), Some(switch), 1.0),
        ],
    );
    lamp
}

/// As chaves do Switch: a caixa em degrau ([`CAIXA`]) e a altura ([`ALTURA`]).
#[must_use]
pub fn chaves_do_switch(switch: u64, fps: f64) -> Vec<TimelineIntent> {
    #[expect(clippy::cast_possible_truncation, reason = "fps inteiros")]
    let fps = fps.round() as u32;
    let chave = |prop, (q, v): (i64, f32), interp| TimelineIntent::AddKey {
        entity: switch,
        prop,
        t: RationalTime::from_frame(q, fps),
        value: AnimValue::Float(v),
        interp,
    };
    let suave = Interp::Eased(Easing::new(EasingFamily::Cubic, EasingMode::InOut));
    let mut ks: Vec<_> = CAIXA
        .iter()
        .map(|&k| chave(PropKind::Field(caixa()), k, Interp::Hold))
        .collect();
    ks.extend(
        ALTURA
            .iter()
            .chain(std::iter::once(&(CAIXA[2].0, ALTURA[0].1)))
            .map(|&k| chave(PropKind::TranslationY, k, suave)),
    );
    ks
}

/// Uma vez, com `PH2D_CONSTRAINT_SMOKE=13`: grava as chaves do Switch, mostra a timeline e o cartão, e
/// toca em laço.
pub fn step(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    if crate::smoke::nivel_armado() != Some(13) || FEITA.swap(true, Ordering::Relaxed) {
        return None;
    }
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    let switch = q.iter(w).find(|(_, n)| n.0 == SWITCH).map(|(e, _)| e)?;
    for i in chaves_do_switch(switch.to_bits(), tl.doc.fps_display) {
        ph2d_timeline::apply_intent(tl, ph, i);
    }
    hero.set_panel_visible("timeline", true);
    hero.set_panel_visible("inspector", true);
    // A caixa da Lamp à vista, por cima dos dois cartões.
    for s in [
        ph2d_editor_core::ids::INSP_LIVE_SLICE_SECTION,
        ph2d_editor_core::ids::INSP_LIVE_CONSTRAINTS_SECTION,
    ] {
        hero.store.set_collapsed(s, false);
    }
    #[expect(clippy::cast_precision_loss, reason = "quadros da cena")]
    let fim = CAIXA[CAIXA.len() - 1].0 as f64 / tl.doc.fps_display;
    for keys in [true, false] {
        tl.doc.set_active_loop_for(keys, Some((0.0, fim)));
    }
    ph.set_loop(0.0, fim);
    ph.seek_frame(0, tl.doc.fps_display);
    ph.play();
    Some("[constraint-smoke] =13 as chaves do Switch gravadas; a tocar".to_owned())
}
