//! Flip module chrome NodeIds (ADR-0114 W2 — docked `ph2d-panel-flip`).
//!
//! The `flip` tool's Brush / Color / Layers controls live in a right-docked
//! `Panel<State>` (the tool `FloatingPanel` is unpainted, mirror of the Vector
//! Style panel). Fixed chrome ids below (`FLIP_*`); the per-layer row widgets
//! use a runtime-hashed id family ([`flip_layer_widget_id`], mirror of the
//! Painter layers panel) since the layer count is only known at runtime.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/flip.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Frame-strip close (X) button.
pub const FLIP_STRIP_CLOSE: NodeId = hash_node_id("flip.strip.close");

/// **Scrub lane** (W7.3): the draggable ruler at the top of the strip that moves the
/// playhead WITHOUT touching the multiframe selection — the standard split of every
/// animation tool (the ruler scrubs, the cells select). A horizontal `Slider` drives
/// the drag; the panel maps its `0..1` value to a frame and the shell seeks there.
pub const FLIP_SCRUB: NodeId = hash_node_id("flip.strip.scrub");

// ── Transport ────────────────────────────────────────────────────────────────
/// Play / pause toggle.
pub const FLIP_PLAY: NodeId = hash_node_id("flip.strip.play");

/// Previous DRAWING (skips holds — the animator's flip, not a frame step).
pub const FLIP_PREV_DRAWING: NodeId = hash_node_id("flip.strip.prev");

/// Next DRAWING.
pub const FLIP_NEXT_DRAWING: NodeId = hash_node_id("flip.strip.next");

/// Frames-per-second chip (the object's own FPS).
pub const FLIP_FPS_NUM: NodeId = hash_node_id("flip.strip.fps_num");

// ── Ghost Frames ─────────────────────────────────────────────────────────────
/// Ghost Frames on/off (per object).
pub const FLIP_GHOST: NodeId = hash_node_id("flip.strip.ghost");

/// How many drawings BEFORE to ghost.
pub const FLIP_GHOST_BEFORE_NUM: NodeId = hash_node_id("flip.strip.ghost_before_num");

/// How many drawings AFTER to ghost.
pub const FLIP_GHOST_AFTER_NUM: NodeId = hash_node_id("flip.strip.ghost_after_num");

// ── Autokey (per-tool semantics — see `ph2d_flip::AutokeyPolicy`) ────────────
/// Autokey on/off: drawing past a key's hold creates a new key.
pub const FLIP_AUTOKEY: NodeId = hash_node_id("flip.strip.autokey");

/// Multiframe **Falloff** (W7): os quadros vizinhos recebem menos influência que o
/// active one. Brushes only — the bucket is a discrete op and always uses 1.0.
pub const FLIP_FALLOFF: NodeId = hash_node_id("flip.strip.falloff");

/// Additive: a new key born of DRAWING starts as a copy (not blank).
pub const FLIP_ADDITIVE: NodeId = hash_node_id("flip.strip.additive");

// ── Key ops ──────────────────────────────────────────────────────────────────
/// Add a blank key after the current one.
pub const FLIP_KEY_ADD: NodeId = hash_node_id("flip.strip.key_add");

/// Duplicate the current key (deep copy).
pub const FLIP_KEY_DUP: NodeId = hash_node_id("flip.strip.key_dup");

/// Duplicate the current key **as an instance**: the new key points at the SAME
/// drawing (`FlipDrawing::users += 1`), so editing one edits both — how a cycle
/// reuses art. Blender calls it a *linked duplicate*; the strip marks such cells
/// with a dot.
pub const FLIP_KEY_INSTANCE: NodeId = hash_node_id("flip.strip.key_instance");

/// **Quebra o vínculo** da chave atual com a arte compartilhada (o *make single user*):
/// ela passa a ter um desenho só dela. A saída de emergência da instância — sem ela,
/// instanciar seria irreversível.
pub const FLIP_KEY_UNLINK: NodeId = hash_node_id("flip.strip.key_unlink");

/// **Pin (light table, T3.9)** — marca a chave atual como REFERÊNCIA persistente: ela
/// aparece como fantasma **além** dos vizinhos, em qualquer modo e fora do alcance. É o
/// *bookmark* do TVPaint/OpenToonz — o quadro extremo que o animador quer no fundo da tela
/// enquanto desenha o meio. Clicar de novo desmarca.
pub const FLIP_KEY_PIN: NodeId = hash_node_id("flip.strip.key_pin");

/// Delete the current key.
pub const FLIP_KEY_DELETE: NodeId = hash_node_id("flip.strip.key_del");

/// Exposure (hold) of the selected key, in frames.
pub const FLIP_HOLD_NUM: NodeId = hash_node_id("flip.strip.hold_num");

/// Move the selected key one frame earlier / later.
pub const FLIP_KEY_LEFT: NodeId = hash_node_id("flip.strip.key_left");

pub const FLIP_KEY_RIGHT: NodeId = hash_node_id("flip.strip.key_right");

// ── Tween ────────────────────────────────────────────────────────────────────
/// How many inbetweens to generate.
pub const FLIP_TWEEN_NUM: NodeId = hash_node_id("flip.strip.tween_num");

/// Generate the inbetweens between the two selected keys (or the current key and
/// the next one).
pub const FLIP_TWEEN_ADD: NodeId = hash_node_id("flip.strip.tween_add");

/// The easing preset of the generated inbetweens (`Linear / Ease In / Out / In-Out`).
///
/// The MOTOR always supported it (`TweenOptions::easing`); only the toolbar did not
/// offer it — the "carry-over of UI, not of engine" that plan T3.7 declared.
pub const FLIP_TWEEN_EASE_DD: NodeId = hash_node_id("flip.strip.tween_ease_dd");

/// Fade the strokes that exist in only ONE of the two keys, instead of copying them
/// statically (`TweenOptions::fade_orphans`).
pub const FLIP_TWEEN_FADE: NodeId = hash_node_id("flip.strip.tween_fade");

/// Toggle the **pair-correction** overlay (the CACAni lesson: the matcher errs, the
/// artist corrects). While on, the canvas shows which stroke of A becomes which of B,
/// and a click re-pairs; the Add button then commits with the corrected plan.
pub const FLIP_TWEEN_PAIRS: NodeId = hash_node_id("flip.strip.tween_pairs");

/// Derive the id of easing option `preset` in the open easing dropdown popover.
#[must_use]
pub fn flip_tween_ease_option_id(preset: u8) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(&format!("flip.strip.easeopt.{preset}"))
}

// ── Cycle (post behavior of the active layer) ────────────────────────────────
/// The cycle dropdown chip (None / Hold / Loop / Ping-Pong).
pub const FLIP_CYCLE_DD: NodeId = hash_node_id("flip.strip.cycle_dd");

/// Derive the id of cycle option `mode` (`CycleMode as u8`) in the open cycle
/// dropdown popover.
#[must_use]
pub fn flip_cycle_option_id(mode: u8) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(&format!("flip.strip.cycleopt.{mode}"))
}

/// Derive the id of the strip cell at `index` (position in the active layer's
/// cell list, NOT the frame number — the index is bounded by what is painted).
#[must_use]
pub fn flip_cell_id(index: usize) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(&format!("flip.strip.cell.{index}"))
}

/// Derive the id of the **hold edge** of the strip cell at `index` — the grip on the
/// cell's right boundary that stretches its exposure.
///
/// A separate id from [`flip_cell_id`] because it is a separate target: the body of the
/// cell moves the key in time, its edge changes how long the key is held, and a single
/// widget cannot answer both. Same index space as the cell it belongs to.
#[must_use]
pub fn flip_hold_edge_id(index: usize) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(&format!("flip.strip.holdedge.{index}"))
}
