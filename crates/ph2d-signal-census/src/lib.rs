//! **O censo dos nomes de sinal autorados** (GEF02 onda 1): cada campo de dados que emite ou ouve um
//! sinal por nome, e os scripts Luau, somados numa [`SignalCensus`]. A tabela única [`CAMPOS`] é a
//! lista; o censo itera-a — campo novo de sinal = uma linha aqui.

pub use ph2d_signal_ends::{Ends, Hint, Listener, Quem, Role, SignalCensus, SignalListeners};

pub mod fora;
mod luau;

pub use fora::{FORA, Fora, Origem};

use ph2d_ecs::SpriteAnimMarkers;
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
use ph2d_ecs::{Entity, World};
use ph2d_physics_ecs::{
    AnimRules, AnimWhen, Health, NavAgent, NavLink, RaySignals, SignalOnHit, SignalOnLeave,
};

/// Uma linha da tabela: o componente, o campo, o papel, e como ler os nomes dele no mundo.
pub struct Campo {
    pub component: &'static str,
    pub field: &'static str,
    pub role: Role,
    pub read: Leitor,
}

/// Como uma linha de [`CAMPOS`] lê os nomes do mundo.
pub type Leitor = fn(&World, &mut dyn FnMut(Entity, &str));

macro_rules! campo {
    ($comp:ty, $field:literal, $role:ident, |$c:ident, $put:ident| $body:expr) => {
        Campo {
            component: stringify!($comp),
            field: $field,
            role: Role::$role,
            read: |w, put| {
                if let Some(mut q) = w.try_query::<(Entity, &$comp)>() {
                    for (e, $c) in q.iter(w) {
                        let $put = &mut |n: &str| put(e, n);
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

/// ⭐ **O censo do mundo**: cada linha de [`CAMPOS`], cada script Luau ligado a um objecto, e cada
/// linha de [`FORA`] sobre os documentos que a composição passa.
#[must_use]
pub fn census(world: &World, fora: &Fora<'_>) -> SignalCensus {
    let mut out = SignalCensus::default();
    for c in CAMPOS {
        (c.read)(world, &mut |_, n| out.add(n, c.role));
    }
    for o in FORA {
        (o.read)(fora, &mut |n| out.add(n, o.role));
    }
    luau::scripts(world, &mut out);
    out
}

/// A impressão digital do que o censo lê (nomes por linha das duas tabelas e os scripts ligados),
/// sem alocar: FNV-1a de 64 bits, determinística.
#[must_use]
pub fn fingerprint(world: &World, fora: &Fora<'_>) -> u64 {
    let mut h = Fnv(0xcbf2_9ce4_8422_2325);
    for (i, c) in CAMPOS.iter().enumerate() {
        h.u64(i as u64);
        (c.read)(world, &mut |_, n| {
            h.bytes(n.as_bytes());
            h.u64(u64::MAX);
        });
    }
    for (i, o) in FORA.iter().enumerate() {
        h.u64((CAMPOS.len() + i) as u64);
        (o.read)(fora, &mut |n| {
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
        let (blocos, cauda) = b.as_chunks::<8>();
        for w in blocos {
            self.mix(u64::from_le_bytes(*w));
        }
        let mut resto = [0u8; 8];
        resto[..cauda.len()].copy_from_slice(cauda);
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
pub fn when_changed(world: &World, fora: &Fora<'_>, publish: impl FnOnce(SignalCensus)) {
    let f = fingerprint(world, fora);
    if ULTIMA.with(|u| u.replace(Some(f))) != Some(f) {
        publish(census(world, fora));
    }
}

/// ⭐ **Quem ouve `name`** (doc Timeline 17 item 8, a lista «Heard by» do cartão do marcador): as
/// linhas `Listen` de [`CAMPOS`] e de [`FORA`] e os scripts que fazem `ph2d.connect(obj, "name", …)`,
/// uma por (ENTIDADE, o que ouve) — dois objectos com o mesmo nome são duas linhas —, na ordem das
/// tabelas. Nome vazio = ninguém. A linha de script nomeia o objecto que CORRE o script, não o alvo
/// do `connect(obj, …)`.
#[must_use]
pub fn listeners(world: &World, fora: &Fora<'_>, name: &str) -> SignalListeners {
    let mut list: Vec<Listener> = Vec::new();
    if name.is_empty() {
        return SignalListeners::default();
    }
    let mut vistos: Vec<(Option<ph2d_ecs::Entity>, Quem)> = Vec::new();
    let mut put = |e: Option<ph2d_ecs::Entity>, what: Quem| {
        if !vistos.contains(&(e, what)) {
            vistos.push((e, what));
            let object = e.and_then(|e| ph2d_ecs::signal_actions::name_of(world, e));
            list.push(Listener { object, what });
        }
    };
    for c in CAMPOS.iter().filter(|c| c.role == Role::Listen) {
        (c.read)(world, &mut |e, n| {
            if n == name {
                put(Some(e), Quem::Component(c.component));
            }
        });
    }
    for o in FORA.iter().filter(|o| o.role == Role::Listen) {
        (o.read)(fora, &mut |n| {
            if n == name {
                put(None, Quem::Doc(o.doc));
            }
        });
    }
    let script_may = luau::listeners(world, name, &mut |e| put(Some(e), Quem::Script));
    SignalListeners { list, script_may }
}
