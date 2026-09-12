//! **A FAIXA de uma row de param** — um `#[path]` filho de `motion_bridge_params.rs`,
//! cortado pelo cap de 600 LOC da shell (HR-18). `super` é o pai, então o
//! `params_channel` dele está no escopo.
//!
//! As duas funções respondem à MESMA pergunta por dois lados: uma alarga a faixa para o
//! widget nunca mentir sobre um valor que já existe, a outra alarga para o que um CANAL
//! significa. Juntas porque uma faixa que sai de dois lugares diverge.

use super::params_channel;

/// A behaviour's magnitude param needs a **channel-aware widget range**, not just
/// a channel-aware value: a `ParamUiHint`'s range is static, and the behaviours'
/// were authored for position (`±10` world units). On the Rotation channel the
/// same param means DEGREES, where `±10` is a barely-visible tilt — and, worse, a
/// range that cannot even represent the `±90` preset: the slider would saturate,
/// display `-10`, and overwrite the doc with `-10` on the first touch.
///
/// Widen `[min, max]` so it **contains** `value` — the invariant every row must
/// satisfy before it reaches the panel.
///
/// A `ParamUiHint`'s range is a suggestion, not a constraint: `Graph::set_param`
/// never clamps, so a preset, an undo, or a loaded document can hold a value
/// outside it. A row whose range does not contain its value is a *lying widget* —
/// `normalized_track` clamps it to the track end, the panel PAINTS the clamped
/// number instead of the real one, and the first touch emits that clamped number
/// straight back into the doc, silently destroying the authored value. (That is
/// exactly the bug the Enio caught with Stagger on the Rotation channel.)
///
/// Containing the value costs a degraded slider span in the pathological case and
/// self-heals the moment the value returns inside the hint's range — a cheap
/// price for a widget that can never lie or destroy.
pub(super) fn contain(min: f32, max: f32, value: f32) -> (f32, f32) {
    (min.min(value), max.max(value))
}

/// Devolve `(min, max, step)` a usar no lugar da do hint, ou `None` para a manter.
///
/// ⚠️ **Isto era uma TABELA DE NOMES DE NÓ, e ela apodreceu com número.** Ela
/// listava três nós; a varredura de 2026-08-14 mediu **seis** que precisavam dela
/// — `motion.drive`, `motion.step` e `motion.noise` shipavam sem, cada um a
/// esperar o próprio report do artista (o do `drive` chegou: *"Scale não aceita
/// mais que 4 em sua caixa de texto e 4 não é quase nada para rot"*, o mesmo
/// defeito que o Stagger já tinha reportado).
///
/// Agora a lista é uma **DECLARAÇÃO do nó** (`register_param_channel_range`), ao
/// lado dos hints dele, e a divisão de saber é a que já existia: o nó diz *quais
/// dos meus params são magnitudes na unidade do canal*, e este arquivo diz *quais
/// canais são angulares* — a pergunta que o [`super::params_channel::channel_unit`]
/// já respondia. Um nó que nasça amanhã mostra a ausência **no próprio arquivo**.
pub(super) fn channel_range_override(
    registry: &ph2d_node_registry::NodeRegistry,
    type_name: &str,
    param: &str,
    channel: i32,
) -> Option<(f32, f32, f32)> {
    if params_channel::channel_unit(channel) != ph2d_node_registry::ParamUnit::Angle {
        return None;
    }
    let id = ph2d_nodegraph::node::NodeTypeId::of(type_name);
    registry
        .param_channel_range(id, param)
        .map(|r| (r.min, r.max, r.step))
}
