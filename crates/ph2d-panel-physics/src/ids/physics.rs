//! Physics world panel widget NodeIds (PHYSICS_*).
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/physics.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_physics_ecs::MAX_LAYERS;
use ph2d_tool_registry::{hash_node_id, hash_node_id_bytes};

/// Physics panel close (X) button.
pub const PHYSICS_CLOSE: NodeId = hash_node_id("physics.close");

// ── Section headers ─────────────────────────────────────────────────────────
// Collapsible, and they have to be: `paint_section_header` ALWAYS paints the
// chevron, so a header without a live id would draw a "click me to fold"
// affordance that does nothing.
/// World section (gravity).
pub const PHYSICS_SEC_WORLD: NodeId = hash_node_id("physics.sec.world");

/// Solver section (sub-steps, iterations, contact frequency).
pub const PHYSICS_SEC_SOLVER: NodeId = hash_node_id("physics.sec.solver");

/// Air-drag section (the size-aware model).
pub const PHYSICS_SEC_AIR: NodeId = hash_node_id("physics.sec.air");

/// Damping section (the uniform model).
pub const PHYSICS_SEC_DAMPING: NodeId = hash_node_id("physics.sec.damping");

/// Sleep section.
pub const PHYSICS_SEC_SLEEP: NodeId = hash_node_id("physics.sec.sleep");

/// Collision-layer matrix section.
pub const PHYSICS_SEC_LAYERS: NodeId = hash_node_id("physics.sec.layers");

/// Debug section (collider overlay + readouts + reset).
pub const PHYSICS_SEC_DEBUG: NodeId = hash_node_id("physics.sec.debug");

// ── World ───────────────────────────────────────────────────────────────────
/// Gravity X (m/s²).
pub const PHYSICS_GRAVITY_X: NodeId = hash_node_id("physics.gravity_x");

/// Chip linked to [`PHYSICS_GRAVITY_X`].
pub const PHYSICS_GRAVITY_X_NUM: NodeId = hash_node_id("physics.gravity_x_num");

/// Gravity Y (m/s²). Y-up, so Earth is negative.
pub const PHYSICS_GRAVITY_Y: NodeId = hash_node_id("physics.gravity_y");

/// Chip linked to [`PHYSICS_GRAVITY_Y`].
pub const PHYSICS_GRAVITY_Y_NUM: NodeId = hash_node_id("physics.gravity_y_num");

// ── Solver ──────────────────────────────────────────────────────────────────
/// Integration sub-steps per tick.
pub const PHYSICS_SUBSTEPS: NodeId = hash_node_id("physics.substeps");

/// Chip linked to [`PHYSICS_SUBSTEPS`].
pub const PHYSICS_SUBSTEPS_NUM: NodeId = hash_node_id("physics.substeps_num");

/// Solver iterations per step.
pub const PHYSICS_ITERATIONS: NodeId = hash_node_id("physics.iterations");

/// Chip linked to [`PHYSICS_ITERATIONS`].
pub const PHYSICS_ITERATIONS_NUM: NodeId = hash_node_id("physics.iterations_num");

/// Contact spring frequency (Hz).
pub const PHYSICS_CONTACT_HZ: NodeId = hash_node_id("physics.contact_hz");

/// Chip linked to [`PHYSICS_CONTACT_HZ`].
pub const PHYSICS_CONTACT_HZ_NUM: NodeId = hash_node_id("physics.contact_hz_num");

// ── Air drag ────────────────────────────────────────────────────────────────
/// Air density — the size-aware drag coefficient.
pub const PHYSICS_AIR_DRAG: NodeId = hash_node_id("physics.air_drag");

/// Chip linked to [`PHYSICS_AIR_DRAG`].
pub const PHYSICS_AIR_DRAG_NUM: NodeId = hash_node_id("physics.air_drag_num");

// ── Damping ─────────────────────────────────────────────────────────────────
/// World linear drag.
pub const PHYSICS_LINEAR_DAMPING: NodeId = hash_node_id("physics.linear_damping");

/// Chip linked to [`PHYSICS_LINEAR_DAMPING`].
pub const PHYSICS_LINEAR_DAMPING_NUM: NodeId = hash_node_id("physics.linear_damping_num");

/// World angular drag.
pub const PHYSICS_ANGULAR_DAMPING: NodeId = hash_node_id("physics.angular_damping");

/// Chip linked to [`PHYSICS_ANGULAR_DAMPING`].
pub const PHYSICS_ANGULAR_DAMPING_NUM: NodeId = hash_node_id("physics.angular_damping_num");

// ── Sleep ───────────────────────────────────────────────────────────────────
/// Speed below which a body may sleep.
pub const PHYSICS_SLEEP_SPEED: NodeId = hash_node_id("physics.sleep_speed");

/// Chip linked to [`PHYSICS_SLEEP_SPEED`].
pub const PHYSICS_SLEEP_SPEED_NUM: NodeId = hash_node_id("physics.sleep_speed_num");

/// Spin below which a body may sleep.
pub const PHYSICS_SLEEP_SPIN: NodeId = hash_node_id("physics.sleep_spin");

/// Seconds under both thresholds before a body sleeps.
pub const PHYSICS_SLEEP_DELAY: NodeId = hash_node_id("physics.sleep_delay");

/// Chip linked to [`PHYSICS_SLEEP_DELAY`].
pub const PHYSICS_SLEEP_DELAY_NUM: NodeId = hash_node_id("physics.sleep_delay_num");

// ── Collision-layer matrix ──────────────────────────────────────────────────
/// Quantas células: o triângulo de `MAX_LAYERS` com a diagonal.
pub const PHYSICS_LAYER_CELLS: usize = MAX_LAYERS * (MAX_LAYERS + 1) / 2;

/// `prefixo` + `n` em decimal, no fim de `buf` a partir de `at`. Devolve o novo fim.
const fn escreve_numero(buf: &mut [u8; 32], at: usize, n: usize) -> usize {
    let mut at = at;
    if n >= 10 {
        buf[at] = b'0' + (n / 10) as u8;
        at += 1;
    }
    buf[at] = b'0' + (n % 10) as u8;
    at + 1
}

const fn com_prefixo(prefixo: &[u8]) -> ([u8; 32], usize) {
    let mut buf = [0u8; 32];
    let mut k = 0;
    while k < prefixo.len() {
        buf[k] = prefixo[k];
        k += 1;
    }
    (buf, k)
}

/// O id de `physics.layer_<a>_<b>`: o mesmo do literal, para as 36 células de antes não mudarem.
const fn id_da_celula(a: usize, b: usize) -> NodeId {
    let (mut buf, at) = com_prefixo(b"physics.layer_");
    let at = escreve_numero(&mut buf, at, a);
    buf[at] = b'_';
    let at = escreve_numero(&mut buf, at + 1, b);
    hash_node_id_bytes(buf.split_at(at).0)
}

const fn tabela_das_celulas() -> [NodeId; PHYSICS_LAYER_CELLS] {
    let mut t = [NodeId(0); PHYSICS_LAYER_CELLS];
    let mut a = 0;
    while a < MAX_LAYERS {
        let mut b = 0;
        while b <= a {
            t[physics_layer_cell_index(a, b)] = id_da_celula(a, b);
            b += 1;
        }
        a += 1;
    }
    t
}

const fn tabela_dos_nomes() -> [NodeId; MAX_LAYERS] {
    let mut t = [NodeId(0); MAX_LAYERS];
    let mut n = 0;
    while n < MAX_LAYERS {
        let (mut buf, at) = com_prefixo(b"physics_layer_name_");
        let at = escreve_numero(&mut buf, at, n);
        t[n] = hash_node_id_bytes(buf.split_at(at).0);
        n += 1;
    }
    t
}

/// The cells of the triangular layer matrix, in row-major lower-triangle order — cell `(i, j)` with
/// `j <= i` sits at index `i(i+1)/2 + j`, so the first 36 are the 8-layer matrix of before: the
/// channels recorded on them keep their pair.
///
/// ⚠️ `node_id_collisions` checks literal consts and `architecture_panel_wiring_parity` cannot see
/// a LOOP: the unicity test below and the seam test that CLICKS cells cover this widget.
///
/// Only the lower triangle exists because the matrix is symmetric by construction: `(i, j)` and
/// `(j, i)` are one fact, so painting both halves would be two controls for one checkbox.
pub static PHYSICS_LAYER_CELL: [NodeId; PHYSICS_LAYER_CELLS] = tabela_das_celulas();

/// O campo do nome de cada camada, ao lado da matriz.
pub static PHYSICS_LAYER_NAME: [NodeId; MAX_LAYERS] = tabela_dos_nomes();

const fn tabela_de(prefixo: &[u8]) -> [NodeId; MAX_LAYERS] {
    let mut t = [NodeId(0); MAX_LAYERS];
    let mut n = 0;
    while n < MAX_LAYERS {
        let (mut buf, at) = com_prefixo(prefixo);
        let at = escreve_numero(&mut buf, at, n);
        t[n] = hash_node_id_bytes(buf.split_at(at).0);
        n += 1;
    }
    t
}

/// O rótulo à esquerda de cada linha da grade da aba (duplo clique renomeia).
pub static PHYSICS_LAYER_ROW_LABEL: [NodeId; MAX_LAYERS] = tabela_de(b"physics.matrix.row_");
/// O rótulo de cima de cada coluna da grade da aba (duplo clique renomeia).
pub static PHYSICS_LAYER_COL_LABEL: [NodeId; MAX_LAYERS] = tabela_de(b"physics.matrix.col_");
/// Cada linha da lista «Layers» da aba: clique abre a camada, duplo clique renomeia.
pub static PHYSICS_LAYER_LIST: [NodeId; MAX_LAYERS] = tabela_de(b"physics.layers.row_");

/// A porta do painel: abre a aba «Collision Layers».
pub const PHYSICS_MATRIX_OPEN: NodeId = hash_node_id("physics.matrix.open");
/// «+ Add» da lista da aba: mostra a primeira camada escondida e abre-a.
pub const PHYSICS_LAYER_ADD: NodeId = hash_node_id("physics.layers.add");

/// Index of cell `(a, b)` in [`PHYSICS_LAYER_CELL`], order-independent.
pub const fn physics_layer_cell_index(a: usize, b: usize) -> usize {
    let (hi, lo) = if a >= b { (a, b) } else { (b, a) };
    hi * (hi + 1) / 2 + lo
}

/// O grupo de arrasto das células (`interaction::toggle_paint`): arrastar pinta só a matriz.
pub const PHYSICS_LAYER_PAINT_GROUP: &str = "physics.layers";

// ── Debug + commands ────────────────────────────────────────────────────────
/// "Show Colliders" toggle.
///
/// ⚠️ It **reads the shell's `App.show_colliders`** — the same flag the `B` key
/// toggles — rather than carrying its own. Two doors to one question diverge,
/// and here the divergence would be visible: the key and the checkbox
/// disagreeing about whether the outlines are on.
pub const PHYSICS_SHOW_COLLIDERS: NodeId = hash_node_id("physics.show_colliders");

/// **Descartar a CORRIDA GRAVADA** — a mesma que a §14 do Inspector descarta
/// (W25).
///
/// ⚠️ **Uma segunda VISTA, nunca uma segunda porta** — o precedente do
/// [`PHYSICS_SHOW_COLLIDERS`] logo acima e o dos chips de ferramenta do rail
/// contra a lista do Impasto: os dois botões leem o MESMO número publicado e
/// caem na MESMA função da shell, então não há estado a divergir.
///
/// ⚠️ **E ele existe porque a §14 é por-ENTIDADE enquanto a fita é do
/// DOCUMENTO:** o `build_player_info` devolve `None` para tudo o que não é um
/// corpo Dynamic, então apagar o player deixava a corrida gravada **presa** —
/// ela continua no arquivo, continua a ser o que o Bake replaya, e não havia
/// gesto nenhum que a alcançasse.
pub const PHYSICS_CLEAR_RUN: NodeId = hash_node_id("physics.clear_run");

/// **Devolver a corrida DESCARTADA** — a vista de mundo do `RestoreRun` da §14.
///
/// Os dois nunca são oferecidos ao mesmo tempo: o guardado só é alcançável com a
/// fita viva vazia, que é a mesma regra derivada que a §14 honra.
pub const PHYSICS_RESTORE_RUN: NodeId = hash_node_id("physics.restore_run");

/// Restore every world setting to the engine defaults.
pub const PHYSICS_RESET_DEFAULTS: NodeId = hash_node_id("physics.reset_defaults");

/// Interaction section header.
pub const PHYSICS_SEC_INTERACT: NodeId = hash_node_id("physics.sec.interact");

/// The tool radio group (Hand / Blast / Pull).
pub const PHYSICS_INTERACT_TOOL: NodeId = hash_node_id("physics.interact.tool");

/// The three tool options, in `InteractionTool::ALL` order.
///
/// ⚠️ A fixed array for the reason [`PHYSICS_LAYER_CELL`] documents: the
/// segmented helper registers its options in a LOOP, which
/// `architecture_panel_wiring_parity` cannot see, so the seam test that clicks
/// each chip is the only thing covering them.
pub const PHYSICS_INTERACT_TOOL_OPT: [NodeId; 3] = [
    hash_node_id("physics.interact.tool.hand"),
    hash_node_id("physics.interact.tool.explode"),
    hash_node_id("physics.interact.tool.attract"),
];

/// Joints section header.
pub const PHYSICS_SEC_JOINT: NodeId = hash_node_id("physics.sec.joint");

/// The joint-tool radio group (Body / Rig / Links / IK / FK).
pub const PHYSICS_JOINT_TOOL: NodeId = hash_node_id("physics.joint.tool");

/// The five options, in `JointTool::ALL` order. Fixed array for the reason
/// [`PHYSICS_INTERACT_TOOL_OPT`] documents.
pub const PHYSICS_JOINT_TOOL_OPT: [NodeId; 5] = [
    hash_node_id("physics.joint.tool.body"),
    hash_node_id("physics.joint.tool.rig"),
    hash_node_id("physics.joint.tool.links"),
    hash_node_id("physics.joint.tool.ik"),
    hash_node_id("physics.joint.tool.fk"),
];

/// The hold-mode radio group (Spring / Rigid / Rope) — Hand only.
pub const PHYSICS_HOLD_MODE: NodeId = hash_node_id("physics.hold.mode");

/// The three hold options, in `HoldMode::ALL` order.
pub const PHYSICS_HOLD_MODE_OPT: [NodeId; 3] = [
    hash_node_id("physics.hold.mode.spring"),
    hash_node_id("physics.hold.mode.rigid"),
    hash_node_id("physics.hold.mode.rope"),
];

/// Hand spring stiffness (acceleration per metre).
pub const PHYSICS_HOLD_STIFFNESS: NodeId = hash_node_id("physics.hold_stiffness");

/// Chip linked to [`PHYSICS_HOLD_STIFFNESS`].
pub const PHYSICS_HOLD_STIFFNESS_NUM: NodeId = hash_node_id("physics.hold_stiffness_num");

/// Hand damping RATIO (1 = critical, at any stiffness).
pub const PHYSICS_HOLD_DAMPING: NodeId = hash_node_id("physics.hold_damping");

/// Chip linked to [`PHYSICS_HOLD_DAMPING`].
pub const PHYSICS_HOLD_DAMPING_NUM: NodeId = hash_node_id("physics.hold_damping_num");

/// Rope slack (metres) — the measured trail distance.
pub const PHYSICS_HOLD_SLACK: NodeId = hash_node_id("physics.hold_slack");

/// Chip linked to [`PHYSICS_HOLD_SLACK`].
pub const PHYSICS_HOLD_SLACK_NUM: NodeId = hash_node_id("physics.hold_slack_num");

/// Blast radius (metres).
pub const PHYSICS_BLAST_RADIUS: NodeId = hash_node_id("physics.blast_radius");

/// Chip linked to [`PHYSICS_BLAST_RADIUS`].
pub const PHYSICS_BLAST_RADIUS_NUM: NodeId = hash_node_id("physics.blast_radius_num");

/// Blast impulse at the centre (N·s).
pub const PHYSICS_BLAST_FORCE: NodeId = hash_node_id("physics.blast_force");

/// Chip linked to [`PHYSICS_BLAST_FORCE`].
pub const PHYSICS_BLAST_FORCE_NUM: NodeId = hash_node_id("physics.blast_force_num");

/// Pull radius (metres).
pub const PHYSICS_PULL_RADIUS: NodeId = hash_node_id("physics.pull_radius");

/// Chip linked to [`PHYSICS_PULL_RADIUS`].
pub const PHYSICS_PULL_RADIUS_NUM: NodeId = hash_node_id("physics.pull_radius_num");

/// Pull force at the centre (N) — negative REPELS.
pub const PHYSICS_PULL_FORCE: NodeId = hash_node_id("physics.pull_force");

/// Chip linked to [`PHYSICS_PULL_FORCE`].
pub const PHYSICS_PULL_FORCE_NUM: NodeId = hash_node_id("physics.pull_force_num");

/// IK damping — the Levenberg factor of the posing solver (W-IK).
pub const PHYSICS_IK_DAMPING: NodeId = hash_node_id("physics.ik_damping");

/// Chip linked to [`PHYSICS_IK_DAMPING`].
pub const PHYSICS_IK_DAMPING_NUM: NodeId = hash_node_id("physics.ik_damping_num");

/// The "does the tip keep its angle?" radio — Pose only.
pub const PHYSICS_IK_ANGLE: NodeId = hash_node_id("physics.ik.angle");

/// Its two options, in `[Free, Match]` order.
///
/// ⚠️ Two options and not a checkbox because this panel says everything with
/// segmented rows, and a lone checkbox among six of them reads as a different
/// KIND of setting than it is.
pub const PHYSICS_IK_ANGLE_OPT: [NodeId; 2] = [
    hash_node_id("physics.ik.angle.free"),
    hash_node_id("physics.ik.angle.match"),
];

#[cfg(test)]
mod tabela_tests {
    use super::*;

    /// Os literais de antes: uma chave gravada na célula continua a apontar para o mesmo par.
    #[test]
    fn a_tabela_gerada_da_o_mesmo_id_do_literal_e_o_mesmo_indice_das_36() {
        assert_eq!(PHYSICS_LAYER_CELLS, 528);
        assert_eq!(PHYSICS_LAYER_CELL[0], hash_node_id("physics.layer_0_0"));
        assert_eq!(PHYSICS_LAYER_CELL[35], hash_node_id("physics.layer_7_7"));
        assert_eq!(
            PHYSICS_LAYER_CELL[physics_layer_cell_index(25, 20)],
            hash_node_id("physics.layer_25_20")
        );
        assert_eq!(PHYSICS_LAYER_NAME[7], hash_node_id("physics_layer_name_7"));
        assert_eq!(
            PHYSICS_LAYER_NAME[31],
            hash_node_id("physics_layer_name_31")
        );
        // As 36 de antes, lidas do triângulo escrito à mão: fila a fila, sem saltos.
        let mut k = 0;
        for a in 0..8 {
            for b in 0..=a {
                assert_eq!(physics_layer_cell_index(a, b), k);
                assert_eq!(physics_layer_cell_index(b, a), k);
                k += 1;
            }
        }
        assert_eq!(physics_layer_cell_index(31, 31), 527);
    }

    #[test]
    fn nenhum_id_gerado_colide() {
        let mut v: Vec<u64> = PHYSICS_LAYER_CELL
            .iter()
            .chain(&PHYSICS_LAYER_NAME)
            .chain(&PHYSICS_LAYER_ROW_LABEL)
            .chain(&PHYSICS_LAYER_COL_LABEL)
            .chain(&PHYSICS_LAYER_LIST)
            .chain(&[PHYSICS_MATRIX_OPEN, PHYSICS_LAYER_ADD])
            .map(|n| n.0)
            .collect();
        let n = v.len();
        v.sort_unstable();
        v.dedup();
        assert_eq!(v.len(), n);
    }
}
