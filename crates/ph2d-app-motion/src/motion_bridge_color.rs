//! **Colour authoring for the Motion params panel** — the sRGB↔linear boundary, in one
//! place (split from `motion_bridge_params.rs` for the shell's 600-LOC file cap). Declared
//! by `motion_bridge` as a `#[path]` sibling, so `super` is `render_loop::motion_bridge`.
//!
//! A colour crosses two spaces here, and that crossing is the whole reason this is a module:
//! the **wire is linear-straight** (what the nodes cook with) while the swatch and the OKLCH
//! picker speak **sRGB8** (what a human picks). Every conversion between the two lives below,
//! so there is exactly one place to be wrong — and one place the round-trip guard has to hold.

use crate::motion_state::MotionState;

/// The colour groups declared by a node type — the 4-channel RGBA param names
/// behind each [`ParamWidget::Color`](ph2d_node_registry::ParamWidget) hint.
pub(super) fn color_groups(
    registry: &ph2d_node_registry::NodeRegistry,
    type_id: ph2d_nodegraph::node::NodeTypeId,
) -> Vec<[&'static str; 4]> {
    use ph2d_node_registry::ParamWidget;
    registry
        .param_ui(type_id)
        .into_iter()
        .flatten()
        .filter_map(|h| match h.widget {
            ParamWidget::Color { channels } => Some(channels),
            _ => None,
        })
        .collect()
}

/// The current linear-straight values of a node's 4 colour channels (per-instance
/// override, else the manifest default). Shared by the read-back change-guard and
/// the snapshot builder so the swatch and the doc agree.
pub(super) fn channel_values(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    channels: [&'static str; 4],
) -> [f32; 4] {
    use ph2d_nodegraph::cook::OpResolver;
    let overrides = motion.doc.graph.node_param_overrides(nid);
    let manifest = motion
        .doc
        .graph
        .node(nid)
        .and_then(|i| motion.registry.resolve(i.type_id()))
        .map(|op| op.manifest());
    let value_of = |name: &str| -> f32 {
        if let Some(v) = overrides.and_then(|m| m.get(name)).copied() {
            return v;
        }
        manifest
            .and_then(|m| m.params.iter().find(|p| p.name == name))
            .map_or(0.0, |p| p.default)
    };
    [
        value_of(channels[0]),
        value_of(channels[1]),
        value_of(channels[2]),
        value_of(channels[3]),
    ]
}

/// Write a picked sRGB colour into a node's 4 linear-straight channel params
/// (RGB via the sRGB transfer function, alpha straight), re-cooking only when the
/// colour actually changed (the picker stays open across idle frames).
pub(super) fn apply_color_to_node(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    channels: [&'static str; 4],
    srgb: [u8; 4],
) {
    let cur = channel_values(motion, nid, channels);
    // Compare in the space the pick actually lives in — sRGB8, what the picker
    // reads and writes. Comparing the LINEAR values instead would fire on a doc
    // colour that is not an exact 8-bit round-trip (say `r = 0.5`): merely OPENING
    // the picker seeds it with the swatch's 8-bit display colour, the read-back
    // decodes to `0.50004…`, and the guard would see a change and quantize the
    // doc — an edit the artist never made, wrapped in an undo step.
    if srgb == linear_rgba_to_srgb8(cur) {
        return;
    }
    let new = srgb8_to_linear_rgba(srgb);
    for (name, v) in channels.into_iter().zip(new) {
        motion.doc.graph.set_param(nid, name, v);
    }
    motion.pump.mark_dirty();
}

/// Seed each colour swatch's `widget_color` from the snapshot's display colour
/// (the OKLCH picker reads it on open + the swatch paints it). Keyed by the
/// anchor channel — the same id the panel registers.
pub(super) fn seed_color_swatches(
    store: &mut ph2d_editor::interaction::WidgetStore,
    snap: &ph2d_panel_motion_params::ParamsSnapshot,
) {
    use ph2d_panel_motion_params::{ParamRow, param_swatch_id};
    for row in &snap.rows {
        if let ParamRow::Color(c) = row {
            store.set_widget_color(param_swatch_id(c.channels[0]), c.srgb);
        }
    }
}

/// Is a colour-swatch or gradient-stop picker open on this node right now? The undo-bracket
/// edge — a whole colour/stop pick coalesces into ONE step (like a slider drag). Read-only;
/// the caller opens the history bracket, THEN calls [`apply_picker_readback`] (writes must
/// land inside the bracket).
/// ⭐⭐⭐ **O ID DA AMOSTRA DE COR DE UM CARTÃO — e ele carrega o NÓ.**
///
/// ⛔⛔ **Por que não se reusa o do painel.** O [`ph2d_panel_motion_params::param_swatch_id`] é
/// função **só do nome do param âncora**, e o doc dele diz porquê: *«unique within a node»*. No
/// painel isso basta — há **um** nó selecionado de cada vez. No **cartão** a premissa cai: vinte
/// cartões estão visíveis ao mesmo tempo, e dois `motion.tint` na tela pediriam o MESMO id.
/// *Escolher a cor de um escreveria no outro, em silêncio.*
///
/// ⚠️ **São dois espaços de nomes, não duas respostas:** a amostra do painel e a do cartão são
/// widgets diferentes, em sítios diferentes. Quem responde *«que param este selector edita?»*
/// continua a ser **uma** função — [`picker_target_of`] —, e estes ids são a entrada dela.
///
/// ⚠️ Sem alocar: o `format!` deste id correria por cada row de cor de cada cartão, todo quadro.
pub(super) fn card_swatch_id(node: u32, anchor: &str) -> ph2d_editor::NodeId {
    // FNV-1a, a convenção de id desta casa — cada crate tem a sua cópia (o painel dos params, o
    // do grafo), e esta é a da shell.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut come = |bytes: &[u8]| {
        for b in bytes {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    come(b"motion-card/swatch/");
    come(&node.to_le_bytes());
    come(b"/");
    come(anchor.as_bytes());
    ph2d_editor::NodeId(h)
}

/// ⭐⭐ **QUE (NÓ, GRUPO DE CANAIS) O SELECTOR ABERTO ESTÁ A EDITAR** — a porta única.
///
/// Duas superfícies podem ter aberto o selector: a row do **painel** (id só com a âncora, e
/// então o nó é o SELECCIONADO) ou uma amostra de **cartão** (id com o nó lá dentro, e então o
/// nó sai do próprio id — o cartão clicado nem precisa de estar selecionado).
///
/// ⚠️ **A varredura pelos nós só corre com um selector ABERTO**, que é um gesto do artista e
/// não um quadro qualquer.
pub(super) fn picker_target_of(
    motion: &MotionState,
    sel: Option<ph2d_nodegraph::graph::NodeId>,
    groups: &[[&'static str; 4]],
    store: &ph2d_editor::interaction::WidgetStore,
) -> Option<(ph2d_nodegraph::graph::NodeId, [&'static str; 4])> {
    use ph2d_node_registry::ParamWidget;
    use ph2d_panel_motion_params::param_swatch_id;
    let alvo = store.picker_target()?;
    // (a) a row do painel — o nó é o seleccionado.
    if let Some(nid) = sel
        && let Some(ch) = groups.iter().find(|ch| param_swatch_id(ch[0]) == alvo)
    {
        return Some((nid, *ch));
    }
    // (b) uma amostra de cartão — o nó vem do id.
    for inst in motion.doc.graph.nodes() {
        let Some(hints) = motion.registry.param_ui(inst.type_id()) else {
            continue;
        };
        for h in hints {
            if let ParamWidget::Color { channels } = h.widget
                && card_swatch_id(inst.id.0, channels[0]) == alvo
            {
                return Some((inst.id, channels));
            }
        }
    }
    None
}

pub(super) fn picker_session(
    motion: &MotionState,
    sel: Option<ph2d_nodegraph::graph::NodeId>,
    groups: &[[&'static str; 4]],
    grad_params: &[&'static str],
    pal_params: &[&'static str],
    store: &ph2d_editor::interaction::WidgetStore,
) -> bool {
    let color = picker_target_of(motion, sel, groups, store).is_some()
        // ⚠️ **A janela do cartão abre sessão como qualquer outra amostra**: sem isto, arrastar
        // no selector com o gradiente aberto num cartão gravaria um passo de undo POR QUADRO.
        || card::card_editor_pick(motion, store).is_some();
    let grad = sel.is_some_and(|nid| {
        grad_params
            .iter()
            .any(|p| gradient_picker_stop(motion, nid, p, store).is_some())
    });
    let pal = sel.is_some_and(|nid| {
        pal_params
            .iter()
            .any(|p| palette_picker_index(motion, nid, p, store).is_some())
    });
    color || grad || pal
}

/// Feed the live OKLCH pick into the node it targets — a colour group's 4 channel params
/// ([`apply_color_to_node`]) or a gradient stop's colour in the ramp string
/// ([`apply_gradient_stop_pick`]). No-op when no picker is open. Must run INSIDE the undo
/// bracket [`picker_session`] opened.
/// A metade *«de QUEM é esta escolha»* quando quem abriu o selector foi um CARTÃO — irmã por
/// responsabilidade, cortada no tecto de LOC da shell.
#[path = "motion_bridge_color_card.rs"]
mod card;

/// Os gates da leitura de volta feita numa janela de CARTÃO — a metade que o censo de alcance
/// não vê.
#[cfg(test)]
#[path = "motion_bridge_color_card_tests.rs"]
mod card_tests;

/// Quantas paradas a rampa serializada tem AGORA — a contagem que o editor pintou.
pub(super) fn current_gradient_len(texto: &str) -> usize {
    ph2d_color::parse_gradient(texto).unwrap_or_default().len()
}

/// Quantas cores a paleta serializada tem AGORA — irmã de [`current_gradient_len`], e as duas
/// caem no MESMO default do editor quando o texto está vazio (senão a última amostra de uma
/// paleta por autorar ficaria sem dono).
pub(super) fn current_palette_len(texto: &str) -> usize {
    ph2d_color::parse_palette(texto)
        .filter(|p| !p.is_empty())
        .map_or(ph2d_color::DEFAULT_PALETTE_FALLBACK.len(), |p| p.len())
}

pub(super) fn apply_picker_readback(
    motion: &mut MotionState,
    sel: Option<ph2d_nodegraph::graph::NodeId>,
    groups: &[[&'static str; 4]],
    grad_params: &[&'static str],
    pal_params: &[&'static str],
    store: &ph2d_editor::interaction::WidgetStore,
) {
    let pick = || store.blender_picker(ph2d_editor::ids::INSP_BLENDER_PICKER);
    // ⭐ A cor vai ao nó que o ID nomeia — que pode NÃO ser o seleccionado, quando o artista
    // clicou a amostra num cartão. As duas outras famílias abaixo continuam presas ao
    // seleccionado, porque os ids delas ainda não carregam o nó (gradiente e paleta).
    if let Some((alvo, ch)) = picker_target_of(motion, sel, groups, store)
        && let Some((value, _, _, _)) = pick()
    {
        apply_color_to_node(motion, alvo, ch, value.rgba);
    }
    // ⭐⭐ **A escolha feita numa janela aberta SOBRE UM CARTÃO** — o nó sai do id, e por isso
    // ela não depende de o cartão estar seleccionado. ⚠️ Vem ANTES do caminho do painel de
    // propósito: os ids são disjuntos (prefixos diferentes), então nunca há duas respostas —
    // mas se um dia houvesse, a superfície que o artista TOCOU tem de ganhar.
    if let Some(escolha) = card::card_editor_pick(motion, store)
        && let Some((value, _, _, _)) = pick()
    {
        if escolha.gradient {
            apply_gradient_stop_pick(
                motion,
                escolha.node,
                escolha.param,
                escolha.index,
                value.rgba,
            );
        } else {
            apply_palette_pick(
                motion,
                escolha.node,
                escolha.param,
                escolha.index,
                value.rgba,
            );
        }
    }
    let Some(nid) = sel else { return };
    for p in grad_params {
        if let Some(stop) = gradient_picker_stop(motion, nid, p, store)
            && let Some((value, _, _, _)) = pick()
        {
            apply_gradient_stop_pick(motion, nid, p, stop, value.rgba);
        }
    }
    for p in pal_params {
        if let Some(i) = palette_picker_index(motion, nid, p, store)
            && let Some((value, _, _, _)) = pick()
        {
            apply_palette_pick(motion, nid, p, i, value.rgba);
        }
    }
}

/// The text params of a node type edited by a [`ParamWidget::Gradient`] (doc 85) — the
/// `ColorRamp` string keys whose per-stop swatches the picker read-back writes into.
pub(super) fn gradient_params(
    registry: &ph2d_node_registry::NodeRegistry,
    type_id: ph2d_nodegraph::node::NodeTypeId,
) -> Vec<&'static str> {
    use ph2d_node_registry::ParamWidget;
    registry
        .param_ui(type_id)
        .into_iter()
        .flatten()
        .filter_map(|h| (h.widget == ParamWidget::Gradient).then_some(h.param))
        .collect()
}

/// The text params edited by a [`ParamWidget::Palette`] — the palette string keys whose
/// per-colour swatches the picker read-back writes into. Sibling of [`gradient_params`],
/// asked separately because the two write DIFFERENT strings (a ramp has positions and an
/// interp; a palette is a list) and a shared list would need a second lookup to tell them
/// apart at the write.
pub(super) fn palette_params(
    registry: &ph2d_node_registry::NodeRegistry,
    type_id: ph2d_nodegraph::node::NodeTypeId,
) -> Vec<&'static str> {
    use ph2d_node_registry::ParamWidget;
    registry
        .param_ui(type_id)
        .into_iter()
        .flatten()
        .filter_map(|h| (h.widget == ParamWidget::Palette).then_some(h.param))
        .collect()
}

/// The palette a node paints with — the authored string, else the factory list. ⚠️ **The
/// SAME fallback the node and the panel use**: a read-back that started from a different
/// list would rewrite colours the artist never saw.
fn current_palette(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    param: &str,
) -> Vec<[f32; 4]> {
    motion
        .doc
        .graph
        .node_text_param_overrides(nid)
        .and_then(|m| m.get(param))
        .and_then(|v| ph2d_color::parse_palette(v))
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| ph2d_color::DEFAULT_PALETTE_FALLBACK.to_vec())
}

/// If the open picker targets a colour swatch of this node's `param` palette, its index.
/// The id is [`ph2d_panel_motion_params::param_pal_swatch_id`] — the SAME the panel
/// registers, so the two agree without sharing row order.
pub(super) fn palette_picker_index(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    param: &str,
    store: &ph2d_editor::interaction::WidgetStore,
) -> Option<usize> {
    let target = store.picker_target()?;
    let n = current_palette(motion, nid, param).len();
    (0..n).find(|&i| ph2d_panel_motion_params::param_pal_swatch_id(param, i) == target)
}

/// Write the live pick into the `i`-th colour and re-serialize the whole list — the same
/// channel `+`/`−` writes, so the palette has one representation.
pub(super) fn apply_palette_pick(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    param: &str,
    i: usize,
    srgb: [u8; 4],
) {
    let mut colors = current_palette(motion, nid, param);
    let Some(slot) = colors.get_mut(i) else {
        return;
    };
    // ⚠️ Compare in sRGB8 — the space the pick lives in — so merely OPENING the picker on
    // a colour that is not an exact 8-bit round-trip does not quantize the palette. The
    // same guard `apply_gradient_stop_pick` and `apply_color_to_node` document.
    if srgb == linear_rgba_to_srgb8(*slot) {
        return;
    }
    // ⚠️ **A alfa vem do PICK, e a premissa contrária custou um smoke.** Este bloco
    // preservava a alfa antiga afirmando que *"o picker OKLCH é opaco"* — ele **não é**:
    // tem a 4ª linha de canal (R+G+B+**A**) e um campo hex `#RRGGBBAA`. Com a premissa
    // errada o artista podia mover o slider A e o valor era descartado no caminho de
    // volta, que é a metade de *"a transparência das cores não está sendo respeitada"*
    // reportada em 2026-08-08. O guard acima já compara os QUATRO bytes, então abrir o
    // picker sobre uma cor translúcida e não mexer segue sendo um no-op.
    *slot = srgb8_to_linear_rgba(srgb);
    motion
        .doc
        .graph
        .set_text_param(nid, param, ph2d_color::serialize_palette(&colors));
}

/// If the OKLCH picker open right now targets a stop swatch of this node's `param` gradient,
/// the stop's index — else `None`. The stop count comes from parsing the current string, and
/// its swatch id is [`ph2d_panel_motion_params::param_grad_swatch_id`] — the SAME id the
/// panel registers, so the two agree without sharing row order.
pub(super) fn gradient_picker_stop(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    param: &str,
    store: &ph2d_editor::interaction::WidgetStore,
) -> Option<usize> {
    let target = store.picker_target()?;
    let ramp = current_gradient(motion, nid, param);
    (0..ramp.len()).find(|&i| ph2d_panel_motion_params::param_grad_swatch_id(param, i) == target)
}

/// The current `ColorRamp` of a node's gradient text param (override, else the default
/// black→white ramp) — shared by the read-back's stop lookup + its re-serialize so the
/// swatch and the string agree.
fn current_gradient(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    param: &str,
) -> ph2d_color::ColorRamp {
    motion
        .doc
        .graph
        .node_text_param_overrides(nid)
        .and_then(|m| m.get(param))
        .and_then(|s| ph2d_color::parse_gradient(s))
        .unwrap_or_default()
}

/// Write a picked sRGB colour into `stop` of a node's gradient text param (RGBA via the sRGB
/// transfer — a alfa do stop viaja no formato `g2` desde 2026-08-08), re-serializing the string
/// and re-cooking only when the colour actually changed (the picker stays open across idle
/// frames). The mirror of [`apply_color_to_node`] for a stop-in-a-string.
pub(super) fn apply_gradient_stop_pick(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    param: &'static str,
    stop: usize,
    srgb: [u8; 4],
) {
    let mut ramp = current_gradient(motion, nid, param);
    let Some(cur) = ramp.stops().get(stop).copied() else {
        return;
    };
    // Compare in sRGB8 — the space the pick lives in — so merely OPENING the picker on a
    // colour that is not an exact 8-bit round-trip does not quantize the doc (the exact
    // guard `apply_color_to_node` documents).
    if srgb == linear_rgba_to_srgb8(cur.color) {
        return;
    }
    // ⚠️ **A alfa do pick é honrada.** Ela era cravada em `1.0` sob a justificativa de que
    // *"stops não carregam alfa"* — o que era verdade do FORMATO (`g1` serializava três
    // canais), não do desejo do artista; o `g2` abriu o campo, e forçar opaco aqui seria
    // a segunda metade do mesmo defeito.
    ramp.set_color(stop, srgb8_to_linear_rgba(srgb));
    motion
        .doc
        .graph
        .set_text_param(nid, param, ph2d_color::serialize_gradient(&ramp));
    motion.pump.mark_dirty();
}

/// sRGB8 (straight) → linear-straight RGBA `[0,1]` (the Motion wire space): RGB
/// through the sRGB transfer function, alpha a plain `/255`.
fn srgb8_to_linear_rgba(srgb: [u8; 4]) -> [f32; 4] {
    use ph2d_color::srgb::srgb_to_linear_byte;
    [
        srgb_to_linear_byte(srgb[0]),
        srgb_to_linear_byte(srgb[1]),
        srgb_to_linear_byte(srgb[2]),
        f32::from(srgb[3]) / 255.0,
    ]
}

/// Linear-straight RGBA `[0,1]` → sRGB8 (straight) for the swatch display /
/// picker seed: RGB through the linear→sRGB transfer, alpha a plain `×255`.
pub(super) fn linear_rgba_to_srgb8(lin: [f32; 4]) -> [u8; 4] {
    use ph2d_color::srgb::linear_to_srgb_byte;
    [
        linear_to_srgb_byte(lin[0]),
        linear_to_srgb_byte(lin[1]),
        linear_to_srgb_byte(lin[2]),
        (lin[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor::interaction::WidgetStore;
    use ph2d_panel_motion_params::param_grad_swatch_id;

    /// **The gradient read-back writes a stop's OKLCH pick back into the string** (doc 85).
    /// The bridge sees the `motion.color_ramp` Custom gradient (`gradient_params`), locates
    /// which stop the open picker targets (`gradient_picker_stop`, keyed by the SAME
    /// `param_grad_swatch_id` the panel registers), and re-serializes the `ColorRamp` with the
    /// picked colour (sRGB→linear). RED-first: drop the `set_text_param` in
    /// `apply_gradient_stop_pick` and stop 0 stays black.
    #[test]
    fn a_gradient_stop_pick_re_serializes_the_string() {
        let mut motion = crate::motion_state::MotionState::new();
        motion.doc = ph2d_motion_doc::MotionDoc::new();
        let nid = motion.doc.graph.add_node("motion.color_ramp");
        motion
            .doc
            .graph
            .set_text_param(nid, "ramp", "g1 2 0:0,0,0 1:1,1,1".to_string());

        // The bridge recognizes the Gradient widget as a colour text param.
        let tid = motion.doc.graph.node(nid).unwrap().type_id();
        assert_eq!(gradient_params(&motion.registry, tid), vec!["ramp"]);

        // Open the picker on stop 0's swatch — the SAME id the panel would register.
        let mut store = WidgetStore::with_capacity(4);
        store.set_picker_target(Some(param_grad_swatch_id("ramp", 0)));
        assert_eq!(
            gradient_picker_stop(&motion, nid, "ramp", &store),
            Some(0),
            "the picker targets stop 0"
        );

        // Pick pure red → the string's stop 0 becomes red (sRGB 255 → linear 1.0).
        apply_gradient_stop_pick(&mut motion, nid, "ramp", 0, [255, 0, 0, 255]);
        let value = motion
            .doc
            .graph
            .node_text_param_overrides(nid)
            .and_then(|m| m.get("ramp"))
            .cloned()
            .expect("ramp text param written");
        let ramp = ph2d_color::parse_gradient(&value).expect("valid gradient");
        assert!(
            ramp.stops()[0].color[0] > 0.9 && ramp.stops()[0].color[1] < 0.05,
            "stop 0 is red now: {:?}",
            ramp.stops()[0].color
        );
        // Stop 1 (white) is untouched — the pick lands on ONE stop.
        assert!(ramp.stops()[1].color[0] > 0.9 && ramp.stops()[1].color[2] > 0.9);
    }
}
