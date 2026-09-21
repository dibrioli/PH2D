//! **O registo dos widgets da secção LIVE MESH** — o catavento.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio. *Esta crate já
//! o pagou sete vezes.*
//!
//! # ⭐ As faixas, e de que recurso cada uma é
//!
//! | caixa | faixa | recurso |
//! |---|---|---|
//! | `Yaw` · `Pitch` | `−180°` … `+180°` | a VOLTA — fora dela a pose repete-se, e um número que só repete produz estado inalcançável |
//! | `Spin` | `−2` … `+2` voltas/s | o **OLHO**, e dizê-lo é a única forma honesta (§0.0): a lei não satura, quem satura é quem olha |

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::mesh3d_edits::{
    MESH3D_ANGLE_STEP as PASSO_ANG, MESH3D_MAX_SPIN as TECTO_SPIN, MESH3D_SPIN_STEP as PASSO_SPIN,
};
use ph2d_editor_core::widget::TextInputState;

/// Meia volta em graus — a fronteira das duas caixas de ângulo.
const MEIA_VOLTA: f64 = 180.0; // LITERAL-PX-OK: meia volta em GRAUS — geometria, não desenho

/// `(id, valor de partida, mínimo, máximo, passo)`.
///
/// ⚠️ **Os valores de partida são os do [`ph2d_ecs::Mesh3D::default()`]**, que são zeros — e aqui
/// isso é o certo: um catavento nasce **parado e na pose de repouso**, os dois estados neutros e
/// observáveis. ⛔ Nascer a girar faria o artista herdar um movimento que ele não escreveu, e há
/// gate nesse par.
const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 3] = [
    (
        ids::INSP_MESH3D_YAW,
        0.0,
        -MEIA_VOLTA,
        MEIA_VOLTA,
        PASSO_ANG,
    ),
    (
        ids::INSP_MESH3D_PITCH,
        0.0,
        -MEIA_VOLTA,
        MEIA_VOLTA,
        PASSO_ANG,
    ),
    (
        ids::INSP_MESH3D_SPIN,
        0.0,
        -TECTO_SPIN,
        TECTO_SPIN,
        PASSO_SPIN,
    ),
];

pub(crate) fn populate_mesh3d(store: &mut WidgetStore) {
    for (id, value, lo, hi, step) in NUMEROS {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, lo, hi, step);
    }
}
