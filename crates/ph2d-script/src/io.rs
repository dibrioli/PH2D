//! Buffered write queue — the only way a Luau script WRITES to ECS state (reads go through the
//! scene lane, `lane.rs`) — plus the devices the scripts read.
//!
//! Why buffered: scripts run inside `Lua::exec`, which holds the
//! single mlua VM lock. Letting Luau callbacks reach into bevy_ecs
//! mid-tick is racy and crosses the sim/script boundary. Instead:
//! - During the tick, `ph2d.get` reads the world as it was at the start
//!   of the tick (writes are not applied yet), `ph2d.set` pushes an
//!   `EntityWrite` into `WriteQueue`.
//! - After the tick, the host drains the queue and applies the writes
//!   to ECS in one pass.
//!
//! This is the same pattern Defold and Roblox use for script ↔ engine
//! marshalling (HR-8 keeps scripts pure-data at the FFI boundary).
//!
//! ### Backpressure / DoS resistance
//!
//! [`WriteQueue::push`] enforces [`WriteQueue::DEFAULT_CAP`] (250k
//! pending writes). A malicious or runaway script doing
//! `while true do ph2d.set(...) end` triggers [`QueueFull`] instead
//! of OOMing the host. The `ph2d.set` Lua binding surfaces this as
//! a Luau-level error so the script fails fast and the host stays
//! responsive.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use ph2d_input::{Key, KeyboardState, MouseButton, MouseState};

#[derive(Debug, Clone, PartialEq)]
pub struct EntityWrite {
    /// ⚠️ **`u64` = os bits da entidade** (TOP-20 #16): um `u32` era só o ÍNDICE, e depois de
    /// um despawn o mesmo índice nomeia outra entidade com outra geração. Os bits valem **só
    /// dentro do quadro** — nunca se gravam (a lei do `StableId`).
    pub entity: u64,
    pub field: String,
    pub value: WriteValue,
}

/// O que um `ph2d.set` escreve: um número da pose, ou o valor de uma propriedade nomeada
/// (`"health.max"`, doc CodeEditor 08) — os dois pela MESMA fila, para pousarem no fim do tique (HR-8)
/// e caírem juntos quando o objecto falha.
#[derive(Debug, Clone, PartialEq)]
pub enum WriteValue {
    Pose(f64),
    Property(serde_json::Value),
    /// `ph2d.damage`/`ph2d.heal` — pela MESMA fila, para pousar na ordem em que o script os escreveu
    /// (um `health.now = 10` seguido de `damage(5)` acaba em 5).
    Health(crate::lane::HealthRequest),
}

/// Returned by [`WriteQueue::push`] when the cap is reached. Carried
/// across the FFI boundary as a Luau runtime error in `ph2d.set`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueFull {
    pub cap: usize,
}

impl std::fmt::Display for QueueFull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WriteQueue full ({} pending writes); script must yield or batch fewer ph2d.set calls per tick",
            self.cap
        )
    }
}

impl std::error::Error for QueueFull {}

/// Write-side: Luau `ph2d.set` enqueues here. Cloning is cheap (Arc),
/// so the host stashes one clone in `Lua::set_app_data` and keeps
/// another for `drain` after the tick.
#[derive(Clone)]
pub struct WriteQueue {
    inner: Arc<Mutex<Vec<EntityWrite>>>,
    cap: usize,
}

impl WriteQueue {
    /// Default ceiling on pending writes. 250k = 2500 entities × 100
    /// fields, comfortably above realistic per-tick budgets and
    /// 5-10× below the OOM threshold on a typical desktop.
    pub const DEFAULT_CAP: usize = 250_000;

    pub fn new() -> Self {
        Self::with_cap(Self::DEFAULT_CAP)
    }

    pub fn with_cap(cap: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
            cap,
        }
    }

    /// Enqueue a write. Returns [`QueueFull`] if the cap is reached.
    pub fn push(&self, w: EntityWrite) -> Result<(), QueueFull> {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if g.len() >= self.cap {
            return Err(QueueFull { cap: self.cap });
        }
        g.push(w);
        Ok(())
    }

    /// Take all pending writes; the queue is empty after this call.
    pub fn drain(&self) -> Vec<EntityWrite> {
        std::mem::take(&mut *self.inner.lock().unwrap_or_else(|p| p.into_inner()))
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap_or_else(|p| p.into_inner()).len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_empty()
    }

    pub fn cap(&self) -> usize {
        self.cap
    }
}

impl Default for WriteQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// A FONTE ÚNICA dos nomes da pose: a lista, a união do `ph2d.d.luau` e a frase dos erros.
macro_rules! pose_fields {
    ($first:literal $(, $rest:literal)*) => {
        /// **Os campos de pose que um script lê e escreve**, na ordem de `scene::pose_of`.
        /// Ângulos (`rotation`, `skew_x`, `skew_y`) em RADIANOS, como o `Transform` os guarda.
        pub const POSE_FIELDS: [&str; [$first $(, $rest)*].len()] = [$first $(, $rest)*];
        /// O tipo `PoseField` do `ph2d.d.luau`: `"x" | "y" | …`.
        pub const POSE_FIELD_UNION: &str = concat!("\"", $first, "\"" $(, " | \"", $rest, "\"")*);
        /// Os nomes para a frase de um erro: `x, y, …`.
        pub const POSE_FIELD_LIST: &str = concat!($first $(, ", ", $rest)*);
    };
}
pose_fields!("x", "y", "rotation", "scale_x", "scale_y", "skew_x", "skew_y");

/// O erro de um nome que não é da pose nem de uma propriedade (`op` = `ph2d.get`/`ph2d.set`).
#[must_use]
pub fn unknown_field(op: &str, field: &str) -> String {
    format!("{op}: unknown field '{field}' ({POSE_FIELD_LIST})")
}

/// O índice de `field` em [`POSE_FIELDS`] — a porta única do nome ao campo.
#[must_use]
pub fn pose_field(field: &str) -> Option<usize> {
    POSE_FIELDS.iter().position(|f| *f == field)
}

/// Per-frame input snapshot consulted by `ph2d.input(key)` from Luau.
/// Keys are flat strings (e.g. `"gamepad.held.south"`,
/// `"gamepad.axis.left_stick_x"`) — input isn't entity-scoped.
///
/// The host (shells/desktop) populates this each frame from
/// `ph2d_input::InputState`. M8 ships gamepad + Pencil-stub keys;
/// future devices (e.g. mouse wheel, IMU) just add new keys.
#[derive(Clone, Default)]
pub struct InputSnapshot {
    inner: Arc<Mutex<BTreeMap<String, f64>>>,
}

impl InputSnapshot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&self, key: &str, value: f64) {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(key.to_owned(), value);
    }

    pub fn get(&self, key: &str) -> Option<f64> {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(key)
            .copied()
    }

    /// Drop every key. Call before re-populating per-frame so stale
    /// "pressed" edges from the previous frame don't leak.
    pub fn clear(&self) {
        self.inner.lock().unwrap_or_else(|p| p.into_inner()).clear();
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap_or_else(|p| p.into_inner()).len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_empty()
    }
}

/// Que pergunta se faz a um botão.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    /// Em baixo agora.
    Down,
    /// Desceu desde o tique anterior.
    Pressed,
    /// Subiu desde o tique anterior.
    Released,
}

#[derive(Default)]
struct DevicesInner {
    keys_down: BTreeSet<Key>,
    keys_pressed: BTreeSet<Key>,
    keys_released: BTreeSet<Key>,
    mouse_down: BTreeSet<MouseButton>,
    mouse_pressed: BTreeSet<MouseButton>,
    mouse_released: BTreeSet<MouseButton>,
    mouse_world: Option<[f64; 2]>,
}

/// ⭐ **O teclado e o rato que os scripts leem** (`ph2d.key_*`, `ph2d.mouse_*`).
///
/// ⚠️ **A borda vive no TIQUE, não no quadro** (`docs/CodeEditor/05_api_que_funciona.md` §3.5): as
/// bordas de cada quadro ACUMULAM-SE até o primeiro tique as ver ([`Devices::edges_seen`]) — dois
/// tiques num quadro veem um só «pressed», e um quadro sem tique não perde a borda.
#[derive(Clone, Default)]
pub struct Devices(Arc<Mutex<DevicesInner>>);

impl Devices {
    fn lock(&self) -> std::sync::MutexGuard<'_, DevicesInner> {
        self.0.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Um quadro dos dispositivos: o que está em baixo agora, e as bordas deste quadro somadas às
    /// que nenhum tique viu ainda. `mouse_world` = o ponto do mundo sob o rato (`None` sem vista).
    pub fn feed(
        &self,
        keyboard: &KeyboardState,
        mouse: &MouseState,
        mouse_world: Option<[f64; 2]>,
    ) {
        let mut g = self.lock();
        g.keys_down = keyboard.iter_held().collect();
        g.keys_pressed.extend(keyboard.iter_pressed());
        g.keys_released.extend(keyboard.iter_released());
        g.mouse_down = MouseButton::ALL
            .into_iter()
            .filter(|b| mouse.held(*b))
            .collect();
        for b in MouseButton::ALL {
            if mouse.pressed(b) {
                g.mouse_pressed.insert(b);
            }
            if mouse.released(b) {
                g.mouse_released.insert(b);
            }
        }
        g.mouse_world = mouse_world;
    }

    /// Um tique viu as bordas: esquece-as.
    pub fn edges_seen(&self) {
        let mut g = self.lock();
        g.keys_pressed.clear();
        g.keys_released.clear();
        g.mouse_pressed.clear();
        g.mouse_released.clear();
    }

    /// Sem corrida, nenhuma borda espera pelo Play (senão uma tecla de há minutos dispararia no
    /// primeiro tique).
    pub fn forget_edges(&self) {
        self.edges_seen();
    }

    /// Alguma das `keys` responde a `edge`?
    #[must_use]
    pub fn key(&self, keys: &[Key], edge: Edge) -> bool {
        let g = self.lock();
        let set = match edge {
            Edge::Down => &g.keys_down,
            Edge::Pressed => &g.keys_pressed,
            Edge::Released => &g.keys_released,
        };
        keys.iter().any(|k| set.contains(k))
    }

    /// O botão responde a `edge`?
    #[must_use]
    pub fn button(&self, b: MouseButton, edge: Edge) -> bool {
        let g = self.lock();
        match edge {
            Edge::Down => g.mouse_down.contains(&b),
            Edge::Pressed => g.mouse_pressed.contains(&b),
            Edge::Released => g.mouse_released.contains(&b),
        }
    }

    /// O ponto do mundo sob o rato.
    #[must_use]
    pub fn mouse_world(&self) -> Option<[f64; 2]> {
        self.lock().mouse_world
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ew(e: u64, f: &str, v: f64) -> EntityWrite {
        EntityWrite {
            entity: e,
            field: f.into(),
            value: WriteValue::Pose(v),
        }
    }

    #[test]
    fn write_queue_drain_resets() {
        let q = WriteQueue::new();
        q.push(ew(1, "x", 1.0)).unwrap();
        q.push(ew(2, "y", 2.0)).unwrap();
        let drained = q.drain();
        assert_eq!(drained.len(), 2);
        assert!(q.is_empty());
    }

    #[test]
    fn clones_share_inner_storage() {
        let q1 = WriteQueue::new();
        let q2 = q1.clone();
        q1.push(ew(0, "z", 9.0)).unwrap();
        assert_eq!(q2.len(), 1);
    }

    #[test]
    fn input_snapshot_round_trips() {
        let s = InputSnapshot::new();
        s.set("gamepad.held.south", 1.0);
        s.set("gamepad.axis.left_stick_x", -0.42);
        assert_eq!(s.get("gamepad.held.south"), Some(1.0));
        assert!((s.get("gamepad.axis.left_stick_x").unwrap() + 0.42).abs() < 1e-6);
        assert_eq!(s.get("gamepad.held.north"), None);
        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn push_returns_queue_full_at_cap() {
        let q = WriteQueue::with_cap(3);
        assert!(q.push(ew(1, "x", 1.0)).is_ok());
        assert!(q.push(ew(2, "x", 2.0)).is_ok());
        assert!(q.push(ew(3, "x", 3.0)).is_ok());
        let err = q.push(ew(4, "x", 4.0)).unwrap_err();
        assert_eq!(err, QueueFull { cap: 3 });
        // Drain re-opens capacity.
        assert_eq!(q.drain().len(), 3);
        assert!(q.push(ew(5, "x", 5.0)).is_ok());
    }
}
