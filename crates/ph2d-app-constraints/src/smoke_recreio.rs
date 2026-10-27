//! Cena 10 — **Recreio** (`docs/BlenderConstraints/06_W5_resto.md` §3.2, os *behaviours*): o ÍMAN vai
//! e volta (curva da timeline, em laço); a LIMALHA é atraída por ele (Atrair: passa, volta e assenta);
//! a MOEDA treme, e treme mais quando o íman passa perto (Tremer com peso por distância ao íman); o
//! BALÃO flutua; a LUA orbita o PLANETA e vira com a volta. Abre com a moeda escolhida.

use std::sync::atomic::{AtomicBool, Ordering};

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_anim::RationalTime;
use ph2d_constraints::Kind;
use ph2d_constraints::comportamento::{Comportamento, Perfil, Peso};
use ph2d_core::Playhead;
use ph2d_ecs::{Name, SimWorld, stable_id_of};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_render::{DISC_TILE_KEY, WHITE_TILE_KEY};
use ph2d_timeline::{
    AnimValue, Easing, EasingFamily, EasingMode, Interp, PropKind, TimelineIntent, TimelineState,
};

use crate::smoke::{peca, pilha};

/// O x do íman: `(quadro, x)` — atravessa a cena e volta.
pub const IMA: [(i64, f32); 3] = [(0, -4.5), (60, 4.5), (120, -4.5)];
/// A linha por onde o íman passa.
pub const IMA_Y: f32 = -2.0;
/// Onde a moeda está: o íman passa por baixo dela a meio da ida.
pub const MOEDA: [f32; 2] = [-1.0, -0.6];
/// O raio do peso da moeda: longe do íman ela fica quieta.
pub const PESO_RAIO: f64 = 2.0;

static FEITA: AtomicBool = AtomicBool::new(false);

/// ⭐ Monta a cena 10. Devolve a moeda.
pub(crate) fn recreio(world: &mut World) -> Entity {
    let ima = peca(
        world,
        "Íman",
        WHITE_TILE_KEY,
        [IMA[0].1, IMA_Y],
        0.0,
        [0.6, 0.6],
        [0.9, 0.2, 0.2, 1.0],
    );
    let limalha = peca(
        world,
        "Limalha",
        DISC_TILE_KEY,
        [0.0, IMA_Y + 1.0],
        0.0,
        [0.3, 0.3],
        [0.4, 0.9, 0.5, 1.0],
    );
    let moeda = peca(
        world,
        "Moeda",
        DISC_TILE_KEY,
        MOEDA,
        0.0,
        [0.6, 0.6],
        [1.0, 0.8, 0.2, 1.0],
    );
    let balao = peca(
        world,
        "Balão",
        DISC_TILE_KEY,
        [-3.5, 1.5],
        0.0,
        [0.8, 1.0],
        [0.3, 0.75, 0.95, 1.0],
    );
    let planeta = peca(
        world,
        "Planeta",
        DISC_TILE_KEY,
        [2.5, 1.5],
        0.0,
        [1.2, 1.2],
        [0.25, 0.5, 0.9, 1.0],
    );
    let lua = peca(
        world,
        "Lua",
        WHITE_TILE_KEY,
        [4.0, 1.5],
        0.0,
        [0.35, 0.2],
        [0.85, 0.85, 0.8, 1.0],
    );
    // A limalha cola-se ao íman: desenha-se por cima dele, senão some debaixo do quadrado.
    world
        .entity_mut(limalha)
        .insert(ph2d_ecs::ZIndexOverride(1));
    ph2d_ecs::assign_missing_stable_ids(world);
    let peso = Peso {
        objecto: stable_id_of(world, ima).map_or(0, |s| s.0),
        raio: PESO_RAIO,
        curva: 2,
        inverte: false,
    };
    let treme = Comportamento {
        peso: Some(peso),
        ..Perfil::Tremer.numeros()
    };
    let orbita = Comportamento {
        leva_rotacao: true,
        ..Perfil::Orbitar.numeros()
    };
    pilha(world, moeda, &[(Kind::Comportamento(treme), None, 1.0)]);
    let flutua = Kind::Comportamento(Perfil::Flutuar.numeros());
    pilha(world, balao, &[(flutua, None, 1.0)]);
    pilha(
        world,
        lua,
        &[(Kind::Comportamento(orbita), Some(planeta), 1.0)],
    );
    let atrai = Kind::Comportamento(Perfil::Atrair.numeros());
    pilha(world, limalha, &[(atrai, Some(ima), 1.0)]);
    moeda
}

/// As chaves do íman ([`IMA`]), a acelerar e a travar.
#[must_use]
pub fn chaves_do_ima(ima: u64, fps: f64) -> Vec<TimelineIntent> {
    #[expect(clippy::cast_possible_truncation, reason = "fps inteiros")]
    let fps = fps.round() as u32;
    IMA.iter()
        .map(|&(q, x)| TimelineIntent::AddKey {
            entity: ima,
            prop: PropKind::TranslationX,
            t: RationalTime::from_frame(q, fps),
            value: AnimValue::Float(x),
            interp: Interp::Eased(Easing::new(EasingFamily::Cubic, EasingMode::InOut)),
        })
        .collect()
}

/// Uma vez, com `PH2D_CONSTRAINT_SMOKE=10`: grava a curva do íman, mostra a timeline e o cartão, e
/// toca em laço.
pub fn step(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    if crate::smoke::nivel_armado() != Some(10) || FEITA.swap(true, Ordering::Relaxed) {
        return None;
    }
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    let ima = q.iter(w).find(|(_, n)| n.0 == "Íman").map(|(e, _)| e)?;
    for i in chaves_do_ima(ima.to_bits(), tl.doc.fps_display) {
        ph2d_timeline::apply_intent(tl, ph, i);
    }
    hero.set_panel_visible("timeline", true);
    hero.set_panel_visible("inspector", true);
    hero.store
        .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_CONSTRAINTS_SECTION, false);
    #[expect(clippy::cast_precision_loss, reason = "quadros da cena")]
    let fim = IMA[IMA.len() - 1].0 as f64 / tl.doc.fps_display;
    for keys in [true, false] {
        tl.doc.set_active_loop_for(keys, Some((0.0, fim)));
    }
    ph.set_loop(0.0, fim);
    ph.seek_frame(0, tl.doc.fps_display);
    ph.play();
    Some("[constraint-smoke] =10 a curva do íman gravada; a tocar".to_owned())
}
