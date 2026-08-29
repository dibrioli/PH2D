//! **AS ROWS QUE SAEM DE UM TEXT PARAM** — a metade do snapshot que não vem do manifesto.
//!
//! Separado do [`super`] pelo teto de LOC (HR-18, 600 para `shells/desktop`), no corte que a
//! pergunta desenha: lá fica *que rows este nó tem a partir dos `ParamSpec`*, aqui *as que
//! nascem do canal de TEXTO do `Graph`* — a fórmula de uma `motion.expression`, a curva de um
//! `field.remap`, a gramática de um `source.lsystem`, o caminho de um `audio.bands`.
//!
//! ⚠️ **Um `ParamSpec` é `f32`** (doc 32), então nada disto aparece no laço do manifesto: se
//! esta função não correr, aqueles nós ficam com o painel pela metade e nada acusa.

use super::*;
// ⚠️ Explícitos: os `use` do pai são privados, então o `super::*` não os traz — é o
// mesmo padrão do irmão `motion_bridge_params_sections.rs`.
use ph2d_node_registry::{ParamUiHint, ParamWidget};
use ph2d_panel_motion_params::{ChannelsRow, CurveRow, GradientRow, ParamRow, SourceRow, TextRow};

/// Acrescenta a `rows` uma row por text param VISÍVEL deste nó.
///
/// ⚠️ O `shown` é passado de fora, e não recalculado: ele é a porta única das três famílias de
/// gate de visibilidade ([`super::params_visible::Visibility`]), e uma segunda avaliação aqui
/// seria um param oferecido num sítio e escondido noutro — que foi o report do Enio sobre o
/// `motion.shape`.
pub(super) fn push_text_rows(
    rows: &mut Vec<ParamRow>,
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    hints: Option<&'static [ParamUiHint]>,
    shown: &dyn Fn(&str) -> bool,
    value_of: &dyn Fn(&str) -> f32,
    // ⚠️ **O `mode` de um picker de canais é CONSUMIDO aqui e não pode ganhar row própria** —
    // a lista atravessa a fronteira do corte porque a decisão nasce deste lado e é lida do
    // outro. Passá-la por valor faria o laço do manifesto voltar a pintar o knob dobrado.
    consumed: &mut Vec<&'static str>,
) {
    // Text params (a `motion.expression` formula, a `field.remap` Curve) are NOT
    // `ParamSpec`s (f32-only), so they never appear in the manifest loop below — surface
    // each as a row FIRST, reading the graph's text channel (docs/Motion Nodes/32-33).
    // `Curve` rides the SAME text channel as `Text` but paints an interactive curve editor
    // (A1-ui): draggable handles over the serialized curve, never the string.
    for h in hints.into_iter().flatten().filter(|h| shown(h.param)) {
        // A named-channel picker (plan §1.1): the artist-facing face of a stream-column
        // TEXT param. It reads the live column (a text param) + its `mode` (an f32
        // param), matches them to a channel, and folds `mode` in (consumed, so it gets
        // no row of its own). Custom = no match → the raw text field is offered.
        if let ParamWidget::Channels {
            mode_param,
            channels,
        } = h.widget
        {
            let attr = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            let mode = value_of(mode_param).round() as i32;
            let selected = channels
                .iter()
                .position(|c| c.column == attr && c.mode == mode)
                .unwrap_or(channels.len());
            // The live upstream columns the Custom picker offers (roadmap: dropdown
            // populated at runtime) — everything the curated channels already cover
            // is excluded, so the chips are the ADVANCED columns, not duplicates.
            let covered: std::collections::BTreeSet<&str> =
                channels.iter().map(|c| c.column).collect();
            let extra = params_stream::upstream_scalar_columns(motion, nid, &covered, &attr);
            rows.push(ParamRow::Channels(ChannelsRow {
                label: h.label.to_string(),
                text_param: h.param,
                mode_param,
                // Resolve to primitives so the panel needs no registry dependency.
                channels: channels
                    .iter()
                    .map(|c| (c.label, c.column, c.mode))
                    .collect(),
                selected,
                custom: attr,
                extra,
            }));
            consumed.push(mode_param);
            continue;
        }
        // A source picker (doc 65): a TEXT param that names a value the app published
        // into the external channel. The options are the LIVE published names — the same
        // `Cook::externals` the node reads from — so the artist picks a shape they drew
        // by name. `motion.pump.cook` holds the externals whether the graph cooked on the
        // CPU or the GPU (the shell republishes them every frame, ADR-0126-independent).
        if h.widget == ParamWidget::Source {
            let current = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            let options = params_stream::source_options(motion);
            rows.push(ParamRow::Source(SourceRow {
                label: h.label.to_string(),
                param: h.param,
                options,
                current,
            }));
            continue;
        }
        // A Gradient editor (doc 85) — a `ColorRamp` in a text param (`serialize_gradient`),
        // the colour sibling of the Curve. The panel draws the bar + draggable stops from the
        // string; a stop's COLOUR is read back through the OKLCH picker below.
        if h.widget == ParamWidget::Palette {
            let value = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            rows.push(ParamRow::Palette(ph2d_panel_motion_params::PaletteRow {
                name: h.param,
                label: h.label.to_string(),
                value,
            }));
            continue;
        }
        if h.widget == ParamWidget::Gradient {
            let value = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            rows.push(ParamRow::Gradient(GradientRow {
                name: h.param,
                label: h.label.to_string(),
                value,
            }));
            continue;
        }
        // Um CAMINHO de ficheiro: o campo editável, o botão que abre o diálogo, e a marca de
        // ausência. ⚠️ O `Path::exists` corre AQUI, uma vez por quadro por row — o painel não
        // pode tocar no disco enquanto pinta, e é a única informação desta row que o artista
        // não tem outra forma de ver (um caminho que aponta para nada lê-se exactamente como
        // um caminho bom, e o nó responde com silêncio).
        if let ParamWidget::File { .. } = h.widget {
            let value = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            let missing = !value.is_empty() && !std::path::Path::new(&value).exists();
            rows.push(ParamRow::File(ph2d_panel_motion_params::FileRow {
                name: h.param,
                label: h.label.to_string(),
                value,
                missing,
            }));
            continue;
        }
        if h.widget == ParamWidget::Text || h.widget == ParamWidget::Curve {
            let value = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            let name = h.param;
            let label = h.label.to_string();
            rows.push(if h.widget == ParamWidget::Curve {
                ParamRow::Curve(CurveRow { name, label, value })
            } else {
                ParamRow::Text(TextRow { name, label, value })
            });
        }
    }
}
