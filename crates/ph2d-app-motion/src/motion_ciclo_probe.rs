//! ⭐⭐⭐ **O INSTRUMENTO DE UM CICLO** — o retrato, os params, o cartão, o despertar e o preço,
//! para **qualquer** grupo de nós (protocolo do [doc 103](../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md)).
//!
//! ⛔ **Nasceu ao abrir o ciclo 4, e a razão é a lei da casa:** este código foi escrito uma vez
//! para o ciclo 3 e ia ser copiado para o 4 — *uma lei escrita em dois sítios ainda não é uma
//! lei, só uma PORTA é*. Copiá-lo deixaria duas réguas a envelhecer em sentidos diferentes, e
//! sete ciclos ainda por abrir.
//!
//! ⚠️ **A coluna do dispositivo lê-se do `register_gpu_kernel`, NUNCA do `lowerings`** — o ciclo
//! 2 pagou essa: o `lowerings` é o que o nó declara saber baixar **sozinho**, e imprimir essa
//! coluna fabrica uma tabela de dívida com oito kernels que já existem.
//!
//! ⚠️ **Cada ciclo mantém os NOMES dos seus testes** (os docs citam-nos) e chama estas portas —
//! o que muda entre ciclos é a lista de nós, nunca a régua.

use crate::motion_state::MotionState;

// ---------------------------------------------------------------------------------------------
// 0. O GRUPO — derivado do registry quando ele é uma FAMÍLIA.
// ---------------------------------------------------------------------------------------------

/// ⭐⭐ **Os nós registados cujo nome começa por `prefixo`, em ordem.**
///
/// ⚠️ **Um ciclo cujo grupo é uma FAMÍLIA (`force.*`, `value.*`, `rig.*`) não a escreve à mão:**
/// uma lista escrita aqui envelhece em silêncio no dia em que um nó da família nasce, e o ciclo
/// fecha com ele por auditar. *A fonte é o registry.*
pub fn familia(prefixo: &str) -> Vec<&'static str> {
    let m = MotionState::new();
    let mut v: Vec<&'static str> = m
        .registry
        .manifests()
        .filter(|man| man.name.starts_with(prefixo))
        .map(|man| man.name)
        .collect();
    v.sort_unstable();
    v
}

// ---------------------------------------------------------------------------------------------
// 1. O RETRATO — o que cada nó do grupo declara.
// ---------------------------------------------------------------------------------------------

/// Uma linha por nó: params · quantos chegam ao cartão · dispositivo · portas · efeito.
pub fn retrato(grupo: &[&str]) {
    let base = MotionState::new();
    eprintln!("\n  nó                        | params | no cartão | device | portas | efeito");
    eprintln!("  --------------------------|--------|-----------|--------|--------|--------");
    for nome in grupo {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node((*nome).to_string());
        let tid = m.doc.graph.node(id).expect("no'").type_id();
        let man = {
            use ph2d_nodegraph::cook::OpResolver;
            base.registry.resolve(tid).map(|op| op.manifest())
        };
        let Some(man) = man else {
            eprintln!("  {nome:<26} | (nao registado)");
            continue;
        };
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        crate::motion_bridge::params::card::stamp_card_params(
            &m,
            ph2d_editor::ProjectSettings::default(),
            &mut snap,
        );
        let no_cartao = snap
            .nodes
            .iter()
            .find(|v| v.id == id.0)
            .map_or(0, |v| v.params.len());
        let device = {
            use ph2d_nodegraph::gpu::KernelResolver;
            m.registry.gpu_kernel(tid).is_some()
        };
        eprintln!(
            "  {nome:<26} | {:>6} | {no_cartao:>9} | {:^6} | {}->{:<4} | {:?}",
            man.params.len(),
            if device { "sim" } else { "NAO" },
            man.inputs.len(),
            man.outputs.len(),
            man.effect,
        );
    }
    eprintln!();
}

// ---------------------------------------------------------------------------------------------
// 2. OS PARAMS, um a um — o que a auditoria compara contra as referências.
// ---------------------------------------------------------------------------------------------

/// ⚠️ Sem esta lista, *«falta X»* é um palpite.
pub fn params_de(grupo: &[&str]) {
    let m = MotionState::new();
    for nome in grupo {
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
                "    {:<20} default {:>9.3}  {:<16}  {}",
                spec.name,
                spec.default,
                h.map_or("—".to_string(), |h| format!("{}..{}", h.min, h.max)),
                w
            );
        }
        // ⚠️ **Os params de TEXTO não estão no manifesto** — eles vivem no canal aditivo
        // (`Graph::set_text_param`) e aparecem só como um `ParamUiHint` sem `ParamSpec` ao
        // lado. Derivá-los da diferença é a única forma de não escrever uma segunda lista.
        for h in hints {
            if man.params.iter().any(|p| p.name == h.param) {
                continue;
            }
            eprintln!(
                "    {:<20} (TEXTO)  {:<16}  {:?}",
                h.param, h.label, h.widget
            );
        }
    }
    eprintln!();
}

// ---------------------------------------------------------------------------------------------
// 3. O CARTÃO — as rows que ele DE FACTO pinta, com o rótulo da tela.
// ---------------------------------------------------------------------------------------------

/// ⚠️ **Um passo de smoke que manda clicar numa linha AFIRMA que ela está na lista**, e a casa
/// já pagou por escrever um passo impossível. Esta porta imprime o que o `stamp_card_params`
/// produz, que é literalmente o que o pintor desenha.
///
/// ⚠️ **As SECÇÕES fazem parte do que o cartão MOSTRA, e a 1.ª redacção não as lia:** o plano do
/// ciclo 3 acusou o `motion.bezier_warp` de pintar `In X · In Y · …` quatro vezes sem dizer de
/// que aresta — uma acusação construída sobre a lista de rótulos, que é metade da resposta.
/// *Uma sonda que lê metade da superfície fabrica dívida.*
/// O NOME que o cartão de cada nó pinta — o que o tutorial tem de escrever.
///
/// ⚠️ **Ele não é o `type_name`**: o `field.remap` pinta-se **`Remap`**, e um passo de smoke que
/// diga *«o cartão `Field Remap`»* manda o dono procurar uma coisa que não existe.
pub fn nomes(grupo: &[&str]) {
    let m = MotionState::new();
    for nome in grupo {
        let mut d = MotionState::new();
        let id = d.doc.graph.add_node((*nome).to_string());
        let snap = ph2d_panel_motion_graph::snapshot_from(&d.doc.graph, &d.registry);
        let t = snap
            .nodes
            .iter()
            .find(|v| v.id == id.0)
            .map_or("(sem cartao)", |v| v.display_name.as_str());
        eprintln!("  {nome:<26} -> cartão «{t}»");
    }
    let _ = m;
    eprintln!();
}

pub fn cartao(grupo: &[&str]) {
    for nome in grupo {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node((*nome).to_string());
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        crate::motion_bridge::params::card::stamp_card_params(
            &m,
            ph2d_editor::ProjectSettings::default(),
            &mut snap,
        );
        let view = snap.nodes.iter().find(|v| v.id == id.0);
        let rows: Vec<String> = view
            .map(|v| v.params.iter().map(|c| c.hint.label.to_string()).collect())
            .unwrap_or_default();
        let secs: Vec<String> = view
            .map(|v| {
                v.sections
                    .iter()
                    .map(|s| {
                        format!(
                            "{}@{}{}",
                            s.title,
                            s.at,
                            if s.open { "" } else { " (fechada)" }
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        eprintln!("  {nome:<26} | {}", rows.join(" · "));
        if !secs.is_empty() {
            eprintln!("  {:<26} > secções: {}", "", secs.join(" · "));
        }
    }
    eprintln!();
}

// ---------------------------------------------------------------------------------------------
// 3-bis. O VOCABULÁRIO — dois nós que guardam a MESMA pergunta chamam-lhe o mesmo nome?
// ---------------------------------------------------------------------------------------------

/// ⭐⭐⭐ **O achado §2.3 do ciclo 3, virado régua** — *«seis vocabulários para onde é o centro»*.
///
/// Para cada nome de param que **dois ou mais** nós do grupo declaram, imprime os rótulos
/// distintos que eles pintam. Um nome partilhado com rótulos diferentes é o defeito: com o
/// painel lateral fora, o cartão é a **única** superfície onde estes nomes aparecem, e o artista
/// que aprendeu um tem de reconhecer o outro.
///
/// ⚠️ **A população é DERIVADA do manifesto**, nunca de uma lista de nós escrita à mão — foi
/// assim que o censo do canto do ciclo 3 (`every_node_that_offsets_a_corner_calls_it_the_same_thing`)
/// se manteve honesto quando um terceiro nó apareceu.
///
/// ⚠️ **Divergir pode ser CERTO** (um rótulo mais específico desambigua), e é por isso que esta
/// porta **imprime** em vez de reprovar: quem lê decide, e escreve a decisão ao lado.
pub fn vocabulario(grupo: &[&str]) {
    use std::collections::BTreeMap;
    let m = MotionState::new();
    // nome do param → (rótulo → quem o pinta)
    let mut tabela: BTreeMap<&str, BTreeMap<String, Vec<String>>> = BTreeMap::new();
    for nome in grupo {
        let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
        let man = {
            use ph2d_nodegraph::cook::OpResolver;
            let Some(op) = m.registry.resolve(tid) else {
                continue;
            };
            op.manifest()
        };
        let hints = m.registry.param_ui(tid).unwrap_or(&[]);
        for spec in man.params {
            let rotulo = hints
                .iter()
                .find(|h| h.param == spec.name)
                .map_or("(sem hint)", |h| h.label);
            tabela
                .entry(spec.name)
                .or_default()
                .entry(rotulo.to_string())
                .or_default()
                .push((*nome).to_string());
        }
    }
    eprintln!("\n  param partilhado          | rótulo(s) | quem");
    eprintln!("  --------------------------|-----------|------");
    for (param, rotulos) in &tabela {
        let quantos: usize = rotulos.values().map(Vec::len).sum();
        if quantos < 2 {
            continue; // um nó só não tem com quem divergir
        }
        let marca = if rotulos.len() > 1 { "⚠️ " } else { "   " };
        for (rotulo, quem) in rotulos {
            eprintln!("{marca} {param:<24} | {rotulo:<9} | {}", quem.join(", "));
        }
    }
    eprintln!();
}

// ---------------------------------------------------------------------------------------------
// 4. O PREÇO — mudou-se para o irmão [`crate::motion_ciclo_preco`] pelo tecto de LOC.
// ---------------------------------------------------------------------------------------------

pub use crate::motion_ciclo_preco::{
    cook_com, porque_nao_medir, quem_o_despertar_nao_acorda, tabela,
};
