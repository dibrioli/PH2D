//! ⭐⭐ **A AUDITORIA DO GRUPO DOS ANIMADORES** (ciclo 2, passo 2 — doc 105).
//!
//! O que cada nó do grupo **declara** hoje: params, onde corre, e o que o cartão dele mostra.
//! ⚠️ É a mesma régua do ciclo 1: a auditoria começa por medir o que existe, nunca por uma lista
//! do que eu acho que falta.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture audit_the_animator_group
//! ```

use crate::motion_state::MotionState;

/// Os oito do ciclo 2 (doc 105 §1).
pub(crate) const GRUPO: [&str; 8] = [
    "motion.oscillator",
    "value.lfo",
    "motion.wiggle",
    "motion.noise",
    "motion.stagger",
    "motion.orbit",
    "motion.spring",
    "motion.delay",
];

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn audit_the_animator_group() {
    let base = MotionState::new();
    eprintln!("\n  nó                   | params | no cartão | device | portas (in->out) | efeito");
    eprintln!("  ---------------------|--------|-----------|--------|-----------------|--------");
    for nome in GRUPO {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.to_string());
        let tid = m.doc.graph.node(id).expect("no'").type_id();
        let man = {
            use ph2d_nodegraph::cook::OpResolver;
            base.registry.resolve(tid).map(|op| op.manifest())
        };
        let Some(man) = man else {
            eprintln!("  {nome:<21} | (nao registado)");
            continue;
        };
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        crate::render_loop::motion_bridge::params::card::stamp_card_params(
            &m,
            ph2d_editor::ProjectSettings::default(),
            &mut snap,
        );
        let no_cartao = snap
            .nodes
            .iter()
            .find(|v| v.id == id.0)
            .map_or(0, |v| v.params.len());
        // ⚠️⚠️ **A RÉGUA CERTA NÃO É O `lowerings`.** A 1.ª versão desta sonda lia
        // `man.lowerings` e imprimiu **NÃO** para os oito — incluindo o `motion.oscillator`,
        // que tem kernel de GPU desde que existe. O `lowerings` é o que o NÓ declara saber
        // baixar sozinho; o caminho do dispositivo é **side-metadata no registry**
        // (`register_gpu_kernel`), exactamente como toda a outra lei desta casa.
        // *Uma coluna de auditoria lida da declaração errada é uma tabela de dívida fabricada.*
        let device = {
            use ph2d_nodegraph::gpu::KernelResolver;
            m.registry.gpu_kernel(tid).is_some()
        };
        eprintln!(
            "  {nome:<21} | {:>6} | {no_cartao:>9} | {:^6} | {}->{:<13} | {:?}",
            man.params.len(),
            if device { "sim" } else { "NAO" },
            man.inputs.len(),
            man.outputs.len(),
            man.effect,
        );
        let _ = &mut m;
    }
    eprintln!();
}

/// **OS PARAMS DE CADA NÓ DO GRUPO, um a um** — o que a auditoria compara contra as
/// referências. ⚠️ Sem esta lista, «falta X» é um palpite.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture what_each_animator_offers
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_each_animator_offers() {
    let m = MotionState::new();
    for nome in GRUPO {
        let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
        let man = {
            use ph2d_nodegraph::cook::OpResolver;
            let Some(op) = m.registry.resolve(tid) else {
                continue;
            };
            op.manifest()
        };
        eprintln!("\n  === {nome} ===");
        eprintln!(
            "  portas: [{}] -> [{}]",
            man.inputs
                .iter()
                .map(|p| p.name)
                .collect::<Vec<_>>()
                .join(", "),
            man.outputs
                .iter()
                .map(|p| p.name)
                .collect::<Vec<_>>()
                .join(", ")
        );
        let hints = m.registry.param_ui(tid).unwrap_or(&[]);
        for spec in man.params {
            let h = hints.iter().find(|h| h.param == spec.name);
            let w = h.map_or("(sem hint)".to_string(), |h| match h.widget {
                ph2d_node_registry::ParamWidget::Enum { labels } => {
                    format!("Enum[{}]", labels.join("|"))
                }
                outro => format!("{outro:?}"),
            });
            eprintln!(
                "    {:<16} default {:>8.3}  {}  {}",
                spec.name,
                spec.default,
                h.map_or("—".to_string(), |h| format!("{}..{}", h.min, h.max)),
                w
            );
        }
    }
    eprintln!();
}
