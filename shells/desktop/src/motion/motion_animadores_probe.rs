//! ⭐⭐ **A AUDITORIA DO GRUPO DOS ANIMADORES** (ciclo 2, passo 2 — doc 105).
//!
//! O que cada nó do grupo **declara** hoje: params, onde corre, e o que o cartão dele mostra.
//! ⚠️ É a mesma régua do ciclo 1: a auditoria começa por medir o que existe, nunca por uma lista
//! do que eu acho que falta.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture audit_the_animator_group
//! ```

use crate::motion::motion_state::MotionState;

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

/// ⭐⭐⭐ **O QUE O CARTÃO NÃO MOSTRA, E POR QUÊ** (ciclo 2, W4 — doc 105).
///
/// A auditoria mediu que o `motion.noise` mostra **13 de 19** params. A pergunta é se os que
/// faltam estão escondidos por um GATE declarado (legítimo — um controlo que o cook não abre não
/// é pintado) ou se são **inalcançáveis**, que é a família do knob morto.
///
/// ⚠️ **A régua é a EXPLICAÇÃO, não a contagem:** para cada param ausente do cartão, ou existe um
/// `ParamGate`/`ParamGateText`/`ParamGateAbove` que o nomeia, ou ele é um achado.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture what_the_card_hides_and_why
/// ```
/// `(total de controlos, escondidos, as linhas, os SEM explicação)` sobre todo o catálogo.
fn census_of_hidden_params() -> (usize, usize, Vec<String>, Vec<String>) {
    let mut sem_explicacao: Vec<String> = Vec::new();
    let mut linhas: Vec<String> = Vec::new();
    let (mut total, mut escondidos) = (0usize, 0usize);
    let todos: Vec<String> = {
        let base = MotionState::new();
        base.registry
            .manifests()
            .map(|m| m.name.to_string())
            .collect()
    };
    for nome in todos.iter().map(String::as_str) {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.to_string());
        let tid = m.doc.graph.node(id).expect("no'").type_id();
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        crate::render_loop::motion_bridge::params::card::stamp_card_params(
            &m,
            ph2d_editor::ProjectSettings::default(),
            &mut snap,
        );
        let no_cartao: Vec<&str> = snap
            .nodes
            .iter()
            .find(|v| v.id == id.0)
            .map(|v| v.params.iter().map(|c| c.hint.param).collect())
            .unwrap_or_default();
        // ⚠️⚠️ **UMA SECÇÃO DOBRADA NÃO É UM CONTROLO ESCONDIDO** — e a 1.ª redacção desta
        // sonda acusou `motion.noise::rotation` e `::uniform` de *«sem explicação»* por não o
        // saber. Eles vivem na secção `Space`, que **nasce fechada** por decisão medida (o nó
        // desenhava 673 px num corpo de 664), e o cartão pinta o cabeçalho dela a dizer quantas
        // rows esconde: eles estão a **um clique**, que é o oposto de inalcançável.
        // *É a segunda tabela de dívida fabricada deste ciclo — a primeira foi a coluna do
        // dispositivo lida do `lowerings`.*
        let dobradas: Vec<&str> = snap
            .nodes
            .iter()
            .find(|v| v.id == id.0)
            .map(|v| {
                v.sections
                    .iter()
                    .filter(|s| !s.open)
                    .map(|s| s.title)
                    .collect()
            })
            .unwrap_or_default();
        let hints = m.registry.param_ui(tid).unwrap_or(&[]);
        let gates = m.registry.param_gates(tid).unwrap_or(&[]);
        let gates_txt = m.registry.param_gates_text(tid).unwrap_or(&[]);
        let gates_acima = m.registry.param_gates_above(tid).unwrap_or(&[]);
        // ⚠️ Uma COR ancora quatro params e mostra UMA row: os canais dela não são "escondidos",
        // são **consumidos** — a mesma lei que o bridge do cartão já escreve.
        let canais: Vec<&str> = hints
            .iter()
            .flat_map(|h| match h.widget {
                ph2d_node_registry::ParamWidget::Color { channels } => channels.to_vec(),
                _ => Vec::new(),
            })
            .collect();
        for h in hints {
            total += 1;
            if no_cartao.contains(&h.param) {
                continue;
            }
            escondidos += 1;
            let porque = if let Some(g) = m
                .registry
                .param_group(tid, h.param)
                .filter(|g| dobradas.contains(g))
            {
                Some(format!("na seccao `{g}`, que nasce FECHADA (um clique)"))
            } else if canais.contains(&h.param) {
                Some("canal de uma COR (consumido pela amostra)".to_string())
            } else if let Some(g) = gates.iter().find(|g| g.param == h.param) {
                Some(format!("gate: `{}` em {:?}", g.when, g.values))
            } else if let Some(g) = gates_txt.iter().find(|g| g.param == h.param) {
                Some(format!("gate de TEXTO: `{}`", g.when_text))
            } else {
                gates_acima
                    .iter()
                    .find(|g| g.param == h.param)
                    .map(|g| format!("gate de LIMIAR: `{}`", g.when))
            };
            match porque {
                Some(p) => linhas.push(format!("  {nome}::{:<16} {p}", h.param)),
                None => sem_explicacao.push(format!("{nome}::{}", h.param)),
            }
        }
    }
    (total, escondidos, linhas, sem_explicacao)
}

/// ⭐⭐⭐ **TODO CONTROLO QUE O CARTÃO NÃO MOSTRA TEM UMA EXPLICAÇÃO DECLARADA** — sobre o
/// catálogo INTEIRO, e sem lista de tolerância.
///
/// Medido em 2026-09-06: **810 controlos · 127 escondidos · 0 sem explicação.** Cada um dos 127
/// é (a) um `ParamGate`/`ParamGateText`/`ParamGateAbove` que o nomeia, (b) um canal de uma COR
/// consumido pela amostra, ou (c) uma row dentro de uma **secção que nasce fechada** — e essa
/// está a um clique, com o cabeçalho a dizer quantas esconde.
///
/// ⛔ **É a rede do knob INALCANÇÁVEL**, que é a espécie que nenhum gate de registo apanha: um
/// param declarado, com hint, que a superfície simplesmente não pinta e nada explica.
/// FALSIFICADO por esconder um param sem declarar porquê.
#[test]
fn every_param_the_card_hides_has_a_declared_reason() {
    let (total, escondidos, _, sem) = census_of_hidden_params();
    assert!(total > 600, "controle: a varredura viu {total} controlos");
    assert!(escondidos > 50, "controle: e {escondidos} escondidos");
    assert!(
        sem.is_empty(),
        "{} controlo(s) que o cartao esconde sem nada a explicar porque^ — e' a especie do knob \
         INALCANCAVEL:\n  {}",
        sem.len(),
        sem.join("\n  ")
    );
}

/// O CENSO — a mesma varredura, a imprimir a tabela.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture what_the_card_hides_and_why
/// ```
#[test]
#[ignore = "sonda de censo, nao um gate"]
fn what_the_card_hides_and_why() {
    let (total, escondidos, linhas, sem) = census_of_hidden_params();
    for l in &linhas {
        eprintln!("{l}");
    }
    eprintln!(
        "\n  {total} controlos · {escondidos} escondidos · {} SEM EXPLICACAO",
        sem.len()
    );
    for s in &sem {
        eprintln!("    ⛔ {s}");
    }
    eprintln!();
}
