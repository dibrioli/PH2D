//! **O censo dos nomes de sinal autorados** (GEF02 onda 1): cada campo de dados que emite ou ouve um
//! sinal por nome, e os scripts Luau, somados numa [`SignalCensus`]. A tabela única [`CAMPOS`] é a
//! lista; o censo itera-a — campo novo de sinal = uma linha aqui.

pub use ph2d_signal_ends::{Ends, Hint, Role, SignalCensus};

mod luau;

use ph2d_ecs::SpriteAnimMarkers;
use ph2d_ecs::World;
use ph2d_ecs::camera_shake::ShakeEmitter;
use ph2d_ecs::counter_watch::CounterWatch;
use ph2d_ecs::factory::Factory;
use ph2d_ecs::hud::UiButton;
use ph2d_ecs::lifetime::Lifetime;
use ph2d_ecs::particle_emitter::ParticleEmitter;
use ph2d_ecs::signal_actions::SignalActions;
use ph2d_ecs::signal_on_action::SignalOnAction;
use ph2d_ecs::sprite_anim::SpriteAnimations;
use ph2d_ecs::state_machine::StateMachine;
use ph2d_ecs::timer::Timers;
use ph2d_ecs::weapon::WeaponFire;
use ph2d_physics_ecs::{
    AnimRules, AnimWhen, Health, NavAgent, NavLink, RaySignals, SignalOnHit, SignalOnLeave,
};

/// Uma linha da tabela: o componente, o campo, o papel, e como ler os nomes dele no mundo.
pub struct Campo {
    pub component: &'static str,
    pub field: &'static str,
    pub role: Role,
    pub read: fn(&World, &mut dyn FnMut(&str)),
}

macro_rules! campo {
    ($comp:ty, $field:literal, $role:ident, |$c:ident, $put:ident| $body:expr) => {
        Campo {
            component: stringify!($comp),
            field: $field,
            role: Role::$role,
            read: |w, $put| {
                if let Some(mut q) = w.try_query::<&$comp>() {
                    for $c in q.iter(w) {
                        $body;
                    }
                }
            },
        }
    };
}

/// ⭐ **A tabela única** dos campos de nome de sinal: as 31 caixas do Inspector e os sem caixa (HUD,
/// partículas, regras de animação).
pub static CAMPOS: &[Campo] = &[
    // Ouvem.
    campo!(SignalActions, "on", Listen, |c, p| c
        .0
        .iter()
        .for_each(|a| p(&a.on))),
    campo!(WeaponFire, "on_signal", Listen, |c, p| p(&c.on_signal)),
    campo!(WeaponFire, "reload_on", Listen, |c, p| p(&c.reload_on)),
    campo!(Factory, "on_signal", Listen, |c, p| p(&c.on_signal)),
    campo!(StateMachine, "transitions.on", Listen, |c, p| c
        .transitions
        .iter()
        .for_each(|t| p(&t.on))),
    campo!(ShakeEmitter, "on", Listen, |c, p| c
        .0
        .iter()
        .for_each(|s| p(&s.on))),
    campo!(ParticleEmitter, "start_on", Listen, |c, p| p(&c.start_on)),
    campo!(ParticleEmitter, "stop_on", Listen, |c, p| p(&c.stop_on)),
    campo!(ParticleEmitter, "restart_on", Listen, |c, p| p(
        &c.restart_on
    )),
    campo!(AnimRules, "when.Signal", Listen, |c, p| c
        .0
        .iter()
        .for_each(|r| {
            if let AnimWhen::Signal(n) = &r.when {
                p(n);
            }
        })),
    // Emitem.
    campo!(SignalOnAction, "signal", Emit, |c, p| c
        .0
        .iter()
        .for_each(|r| p(&r.signal))),
    campo!(CounterWatch, "signal", Emit, |c, p| c
        .0
        .iter()
        .for_each(|r| p(&r.signal))),
    campo!(Timers, "signal", Emit, |c, p| c
        .0
        .iter()
        .for_each(|t| p(&t.signal))),
    campo!(WeaponFire, "on_fire", Emit, |c, p| p(&c.on_fire)),
    campo!(WeaponFire, "on_empty", Emit, |c, p| p(&c.on_empty)),
    campo!(WeaponFire, "on_reloaded", Emit, |c, p| p(&c.on_reloaded)),
    campo!(Factory, "on_spawned", Emit, |c, p| p(&c.on_spawned)),
    campo!(Factory, "on_exhausted", Emit, |c, p| p(&c.on_exhausted)),
    campo!(Lifetime, "on_death", Emit, |c, p| p(&c.on_death)),
    campo!(Health, "on_damage", Emit, |c, p| p(&c.on_damage)),
    campo!(Health, "on_heal", Emit, |c, p| p(&c.on_heal)),
    campo!(Health, "on_death", Emit, |c, p| p(&c.on_death)),
    campo!(NavAgent, "on_arrived", Emit, |c, p| p(&c.on_arrived)),
    campo!(NavAgent, "on_no_path", Emit, |c, p| p(&c.on_no_path)),
    campo!(NavAgent, "on_stuck", Emit, |c, p| p(&c.on_stuck)),
    campo!(NavLink, "on_crossed", Emit, |c, p| p(&c.on_crossed)),
    campo!(RaySignals, "on_enter", Emit, |c, p| p(&c.on_enter)),
    campo!(RaySignals, "on_exit", Emit, |c, p| p(&c.on_exit)),
    campo!(SignalOnHit, "0", Emit, |c, p| p(&c.0)),
    campo!(SignalOnLeave, "0", Emit, |c, p| p(&c.0)),
    campo!(StateMachine, "states.on_enter", Emit, |c, p| c
        .states
        .iter()
        .for_each(|s| p(&s.on_enter))),
    campo!(StateMachine, "states.on_exit", Emit, |c, p| c
        .states
        .iter()
        .for_each(|s| p(&s.on_exit))),
    campo!(SpriteAnimations, "signal_on_finish", Emit, |c, p| c
        .iter()
        .for_each(|t| p(&t.signal_on_finish))),
    campo!(SpriteAnimations, "signal_on_loop", Emit, |c, p| c
        .iter()
        .for_each(|t| p(&t.signal_on_loop))),
    campo!(SpriteAnimMarkers, "signal", Emit, |c, p| c
        .0
        .iter()
        .for_each(|m| p(&m.signal))),
    campo!(UiButton, "signal", Emit, |c, p| p(&c.signal)),
    campo!(ParticleEmitter, "finished_signal", Emit, |c, p| p(
        &c.finished_signal
    )),
];

/// ⭐ **O censo do mundo**: cada linha de [`CAMPOS`] e cada script Luau ligado a um objecto.
#[must_use]
pub fn census(world: &World) -> SignalCensus {
    let mut out = SignalCensus::default();
    for c in CAMPOS {
        (c.read)(world, &mut |n| out.add(n, c.role));
    }
    luau::scripts(world, &mut out);
    out
}

/// A impressão digital do que o censo lê (nomes por linha da tabela e os scripts ligados), sem
/// alocar: FNV-1a de 64 bits, determinística.
#[must_use]
pub fn fingerprint(world: &World) -> u64 {
    let mut h = Fnv(0xcbf2_9ce4_8422_2325);
    for (i, c) in CAMPOS.iter().enumerate() {
        h.u64(i as u64);
        (c.read)(world, &mut |n| {
            h.bytes(n.as_bytes());
            h.u64(u64::MAX);
        });
    }
    luau::fingerprint(world, &mut |x| h.u64(x));
    h.0
}

struct Fnv(u64);

impl Fnv {
    /// Oito bytes por passo (o byte a byte custava `35 µs` nos 200 objectos).
    fn bytes(&mut self, b: &[u8]) {
        let mut c = b.chunks_exact(8);
        for w in &mut c {
            self.mix(u64::from_le_bytes([
                w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7],
            ]));
        }
        let mut resto = [0u8; 8];
        resto[..c.remainder().len()].copy_from_slice(c.remainder());
        self.mix(u64::from_le_bytes(resto) ^ ((b.len() as u64) << 56));
    }
    fn mix(&mut self, x: u64) {
        self.0 = (self.0 ^ x).wrapping_mul(0x0100_0000_01b3).rotate_left(29);
    }
    fn u64(&mut self, x: u64) {
        self.mix(x);
    }
}

thread_local! {
    static ULTIMA: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

/// ⭐ **A porta do quadro**: entrega a `publish` o censo novo só quando a [`fingerprint`] mudou
/// (medido 09/11, `--profile smoke`, 200 objectos: censo `58 µs`, impressão digital bem menos —
/// ver `mede_o_custo_do_censo_por_quadro`).
pub fn when_changed(world: &World, publish: impl FnOnce(SignalCensus)) {
    let f = fingerprint(world);
    if ULTIMA.with(|u| u.replace(Some(f))) != Some(f) {
        publish(census(world));
    }
}
