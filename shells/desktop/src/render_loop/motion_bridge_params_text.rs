//! A metade do snapshot de params que **NÃO é f32** — os text params.
//!
//! O `NodeManifest` é f32-only por contrato congelado (ADR-0039), então uma fórmula, uma
//! curva, um gradiente, uma paleta e uma LISTA DE PASSOS viajam pelo canal aditivo de
//! texto (doc 32) e **nunca aparecem no laço do manifesto**. Cortado do pai pelo teto de
//! LOC da shell, e cortado por ASSUNTO: *de onde o valor vem* — do `Graph`, por chave de
//! texto — em vez de do `ParamSpec`.

use super::params_stream;
use crate::motion_state::MotionState;
use ph2d_node_registry::{ParamUiHint, ParamWidget};
use ph2d_nodegraph::graph::NodeId;
use ph2d_panel_motion_params::{ChannelsRow, CurveRow, GradientRow, ParamRow, SourceRow, TextRow};

/// Empurra uma row por text param VISÍVEL, na ordem dos hints.
///
/// `consumed` recebe os params f32 que uma row de texto DOBRA (o `mode` de um picker de
/// canal), para o laço do manifesto não os desenhar uma segunda vez — e é por isso que o
/// `value_of` entra: aquele `mode` é `ParamSpec`, não texto.
pub(super) fn push_text_rows(
    motion: &MotionState,
    nid: NodeId,
    hints: Option<&'static [ParamUiHint]>,
    shown: &dyn Fn(&str) -> bool,
    value_of: &dyn Fn(&str) -> f32,
    rows: &mut Vec<ParamRow>,
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
        // Uma faixa de PASSOS (a lista de números como barras arrastáveis) — a gêmea
        // numérica da paleta, no mesmo canal de texto. A FAIXA vem do hint: o strip a lê
        // para desenhar a altura e a escreve de volta no arrasto, e é a mesma que os
        // sliders escalares do nó declaram.
        if h.widget == ParamWidget::Steps {
            let value = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            rows.push(ParamRow::Steps(ph2d_panel_motion_params::StepsRow {
                name: h.param,
                label: h.label.to_string(),
                value,
                min: h.min,
                max: h.max,
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
