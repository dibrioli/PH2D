//! **Os ids das DUAS secções do ABANÃO** (suplente #25) — a da câmera e a de quem explode.
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC**, como o [`super::inspector_path_follow`].
//!
//! # ⚠️ Eles estão no mesmo ficheiro e NUNCA na mesma tela
//!
//! A `CameraShake` mora na câmera e o `ShakeEmitter` em quem explode — dois objectos diferentes,
//! logo o Inspector nunca as pinta juntas. Ficam juntos aqui porque são **um assunto**: separá-los
//! poria a lei do abanão em dois sítios que ninguém liga.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── A CÂMERA: *como* ela treme ───────────────────────────────────────────────────────────────

/// Metros de deslocamento no pico.
pub const INSP_SHAKE_AMPLITUDE: NodeId = hash_node_id("insp_shake_amplitude");
/// Hz — quantas vezes por segundo a vista muda de direcção.
pub const INSP_SHAKE_FREQUENCIA: NodeId = hash_node_id("insp_shake_frequencia");
/// Trauma por segundo ⇒ `1 / decaimento` é a duração de um abanão cheio.
pub const INSP_SHAKE_DECAIMENTO: NodeId = hash_node_id("insp_shake_decaimento");
/// A semente do ruído.
pub const INSP_SHAKE_SEMENTE: NodeId = hash_node_id("insp_shake_semente");

/// **Os chips do EXPOENTE.**
///
/// ⚠️⚠️ **Aqui o índice NÃO é a tag de um enum — ele é `expoente − 1`**, porque a faixa da lei
/// começa em `ph2d_shake::EXPOENTE_MIN` e não em zero. ⛔ Ler o índice como o expoente entregaria
/// um `0`, que é o valor que a lei recusa **por apagar o trauma**. Há gate de ida-e-volta.
pub const INSP_SHAKE_EXPOENTE: [NodeId; 3] = [
    hash_node_id("insp_shake_expoente_0"),
    hash_node_id("insp_shake_expoente_1"),
    hash_node_id("insp_shake_expoente_2"),
];

// ── QUEM EXPLODE: *ao ouvir o quê* ───────────────────────────────────────────────────────────

/// `+ Add Source`.
pub const INSP_EMITTER_ADD: NodeId = hash_node_id("insp_emitter_add");
/// `x Remove Source` — apaga a que está aberta.
pub const INSP_EMITTER_REMOVE: NodeId = hash_node_id("insp_emitter_remove");
/// O nome do sinal que a fonte ouve. **Vazio = calada.**
pub const INSP_EMITTER_ON: NodeId = hash_node_id("insp_emitter_on");
/// Quanto trauma ela levanta à queima-roupa.
pub const INSP_EMITTER_FORCA: NodeId = hash_node_id("insp_emitter_forca");
/// O raio interno, em metros — até aqui o impulso chega inteiro.
pub const INSP_EMITTER_DENTRO: NodeId = hash_node_id("insp_emitter_dentro");
/// O raio externo, em metros — a partir daqui não chega nada.
pub const INSP_EMITTER_FORA: NodeId = hash_node_id("insp_emitter_fora");

/// **Os chips da CERCA de quem falou** — a posição É a tag do `ph2d_ecs::SignalFrom`.
///
/// ⚠️ **Reordenar isto muda a semântica de toda cena gravada**, sem uma linha de erro: aquele enum
/// é `append-only` e a posição dele viaja no ficheiro. Há gate na shell a prender os dois.
pub const INSP_EMITTER_DE: [NodeId; 2] = [
    hash_node_id("insp_emitter_de_0"),
    hash_node_id("insp_emitter_de_1"),
];

/// **As linhas da lista** — uma por fonte, até ao cap de `ph2d_ecs::SHAKE_EMITTERS_MAX`.
///
/// ⚠️ O comprimento deste array **é** o cap do modelo, e há gate na shell a prendê-los: *um modelo
/// que aceita o que o painel não mostra produz estado inalcançável por gesto nenhum.*
pub const INSP_EMITTER_ROW: [NodeId; 16] = [
    hash_node_id("insp_emitter_row_00"),
    hash_node_id("insp_emitter_row_01"),
    hash_node_id("insp_emitter_row_02"),
    hash_node_id("insp_emitter_row_03"),
    hash_node_id("insp_emitter_row_04"),
    hash_node_id("insp_emitter_row_05"),
    hash_node_id("insp_emitter_row_06"),
    hash_node_id("insp_emitter_row_07"),
    hash_node_id("insp_emitter_row_08"),
    hash_node_id("insp_emitter_row_09"),
    hash_node_id("insp_emitter_row_10"),
    hash_node_id("insp_emitter_row_11"),
    hash_node_id("insp_emitter_row_12"),
    hash_node_id("insp_emitter_row_13"),
    hash_node_id("insp_emitter_row_14"),
    hash_node_id("insp_emitter_row_15"),
];
