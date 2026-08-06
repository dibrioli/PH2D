//! **Quantos widgets uma row tem, e como eles se CHAMAM** — os tetos e a derivação de id do
//! painel de params (irmão de `snapshot.rs`, que responde a outra pergunta: *o que uma row É*).
//!
//! O corte é por responsabilidade e nasceu de um teto de LOC: os tipos de row crescem quando o
//! vocabulário de params cresce, e os ids crescem quando um EDITOR novo chega (curva, gradiente,
//! paleta) — duas taxas diferentes no mesmo arquivo.
//!
//! ⚠️ Todo id sai de `fnv_id(&format!(…))`, nunca de um array — é por isso que um slot a mais
//! custa uma string e o teto de linhas é sobre o `populate`, não sobre o espaço de ids.

use ph2d_a11y::NodeId;

/// Max param rows the pooled slider/chip widgets support.
///
/// ⚠️ **É um teto de RECURSO, e o recurso é o `populate`:** ele registra **21 widgets por
/// slot** (slider · chip · number · reroll · text · checkbox · 2×8 botões de enum) de uma
/// vez, para todos os slots, então o número multiplica direto. Não é o espaço de ids — eles
/// saem de `fnv_id(&format!(…))`, e um slot a mais custa uma string a mais.
///
/// ⚠️ **E ele é MEDIDO, não escolhido** (§0). O valor anterior era **8**, com a justificativa
/// *"grid/transform/clone têm 3; o teto cobre os nós de fan-out"* — verdade quando foi escrita
/// e **falsa hoje**: o censo (`the_panel_shows_every_param_of_every_node`, na shell) mede o
/// pior nó do registry, e o `field.remap` sozinho declara **12** `ParamSpec`. Acima do teto o
/// `.take()` do `paint_rows` **não desenha nada e não registra nada** — o param existe no
/// modelo, o cook o lê, e o artista não tem como alcançá-lo: o modo de falha SILENCIOSO que
/// as quatro condições de UI existem para proibir.
///
/// O número é o pior caso medido mais folga de uma família; o gate é quem o mantém honesto —
/// o nó que passar deste teto deixa a suíte VERMELHA em vez de esconder um botão.
pub const MAX_PARAM_ROWS: usize = 16;

/// Max named options a single `Enum` row's segmented selector supports (covers
/// the behaviours' channel / wave / easing sets with headroom).
pub(crate) const MAX_ENUM_OPTIONS: usize = 8;

/// Option-id base for a [`ChannelsRow`]'s live-column chips (the Custom picker).
/// Well above `MAX_ENUM_OPTIONS` so a chip's `param_enum_id(slot, BASE + j)` never
/// collides with a curated segment's `param_enum_id(slot, opt)` (`opt < 9`).
pub(crate) const CHANNELS_EXTRA_BASE: usize = 32;

/// Max control points a single Curve row's editor supports (matches the field.remap
/// text param's practical ceiling; a handful of points shape any transfer). The
/// per-point `CurvePoint` widgets are pooled positionally like the enum options.
pub(crate) const MAX_CURVE_POINTS: usize = 8;

/// Max stops a single Gradient row's editor offers (doc 85). The model
/// (`ph2d_color::MAX_RAMP_STOPS`) allows 32, but the panel is narrow and the swatch
/// strip must stay legible — `+` refuses beyond this, the display ceiling. The
/// per-stop `CurvePoint` markers are registered per-paint like the Curve handles.
pub(crate) const MAX_GRADIENT_STOPS: usize = 8;

/// Stable widget id for the `slot`-th param row's slider (pooled, positional —
/// row `i` of whichever node is selected uses slot `i`).
pub(crate) fn param_slider_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/slider/{slot}"))
}

/// Stable widget id for the `slot`-th param row's numeric chip.
pub(crate) fn param_chip_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/chip/{slot}"))
}

/// Stable widget id for a colour-swatch row, keyed by its **anchor channel**
/// param name (unique within a node) — NOT positional like the slider/chip pool,
/// so the shell bridge computes the same id from the node's hints without
/// agreeing on row order. `pub` for the bridge's picker read-back / seeding.
pub fn param_swatch_id(anchor: &str) -> NodeId {
    fnv_id(&format!("motion_param/swatch/{anchor}"))
}

/// Stable widget id for the `slot`-th param row's checkbox (Toggle rows).
pub(crate) fn param_checkbox_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/check/{slot}"))
}

/// Stable widget id for option `opt` of the `slot`-th param row's segmented
/// selector (Enum rows).
pub(crate) fn param_enum_id(slot: usize, opt: usize) -> NodeId {
    fnv_id(&format!("motion_param/enum/{slot}/{opt}"))
}

/// Stable widget id for the `slot`-th param row's standalone numeric box — the
/// app-standard `NumberInput` (Angle + Seed rows; Scalar rows use the slider's
/// own chip instead).
pub(crate) fn param_number_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/number/{slot}"))
}

/// Stable widget id for the `slot`-th Seed row's re-roll button.
pub(crate) fn param_reroll_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/reroll/{slot}"))
}

/// Stable widget id for the `slot`-th Text row's `TextInput` field (formula editor).
pub(crate) fn param_text_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/text/{slot}"))
}

/// The `slot`-th Curve row's **editor parent** id — the `CurvePoint.parent` every
/// handle carries, so `apply_event` routes the drained drag to the right row (the
/// dispatch emits `ValueChanged(parent)` on a handle drag).
pub(crate) fn param_curve_editor_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}"))
}

/// The `slot`-th Curve row's `point`-th draggable control-point handle.
pub(crate) fn param_curve_point_id(slot: usize, point: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}/pt/{point}"))
}

/// The `slot`-th Curve row's **add-point** button.
pub(crate) fn param_curve_add_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}/add"))
}

/// The `slot`-th Curve row's **remove-point** button.
pub(crate) fn param_curve_remove_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}/remove"))
}

/// The `slot`-th Curve row's **interp** button — cycles the selected point's
/// segment interpolation (Linear → Smooth → Hold).
pub(crate) fn param_curve_interp_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}/interp"))
}

/// The `slot`-th Gradient row's **editor parent** id — the `CurvePoint.parent` every
/// position marker carries, so `apply_event` routes the drained drag to the right row.
pub(crate) fn param_grad_editor_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}"))
}

/// The `slot`-th Gradient row's `stop`-th draggable position marker.
pub(crate) fn param_grad_stop_id(slot: usize, stop: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/stop/{stop}"))
}

/// Stable widget id for a Gradient row's `stop`-th colour swatch, keyed by the text-param
/// **name** + index (NOT positional) — so the shell bridge computes the same id from the
/// node's hints to seed the swatch colour and read the OKLCH pick back into the string.
/// `pub` for the bridge, exactly like [`param_swatch_id`].
pub fn param_grad_swatch_id(name: &str, stop: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad_swatch/{name}/{stop}"))
}

/// Stable widget id for a Palette row's `i`-th colour swatch, keyed by the text-param
/// **name** + index (NOT positional), so the shell bridge computes the same id to seed the
/// swatch and read the OKLCH pick back. `pub` for the bridge, exactly like
/// [`param_grad_swatch_id`].
///
/// ⚠️ **Derived from a string, so there is no pool and no cap** — the 200th swatch has an
/// id as readily as the 2nd, which is what lets the row have no length limit.
pub fn param_pal_swatch_id(name: &str, i: usize) -> NodeId {
    fnv_id(&format!("motion_param/pal_swatch/{name}/{i}"))
}

/// The `slot`-th Palette row's **add-colour** button.
pub(crate) fn param_pal_add_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/pal/{slot}/add"))
}

/// The `slot`-th Palette row's **remove-colour** button (drops the LAST colour).
pub(crate) fn param_pal_remove_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/pal/{slot}/remove"))
}

/// The `slot`-th Gradient row's **add-stop** button.
pub(crate) fn param_grad_add_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/add"))
}

/// The `slot`-th Gradient row's **remove-stop** button.
pub(crate) fn param_grad_remove_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/remove"))
}

/// The `slot`-th Gradient row's **interp** button — cycles the ramp's global
/// interpolation (Linear → Ease → Constant → Cardinal → B-Spline).
pub(crate) fn param_grad_interp_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/interp"))
}

/// The `slot`-th Gradient row's `p`-th **preset seed** chip (Rainbow / Heat / Ice /
/// Grayscale) — clicking it LOADS that preset's stops into the editable ramp (doc 85).
pub(crate) fn param_grad_preset_id(slot: usize, p: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/preset/{p}"))
}

/// FNV-1a-64 of `key` (same scheme as the graph panel's dynamic hit ids).
fn fnv_id(key: &str) -> NodeId {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in key.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    NodeId(h)
}
