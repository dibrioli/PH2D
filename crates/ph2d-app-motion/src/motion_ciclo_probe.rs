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
            ph2d_editor_core::ProjectSettings::default(),
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
            ph2d_editor_core::ProjectSettings::default(),
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
    // ⛔⛔ **Um param DOBRADO no widget de outro não é um param sem rótulo.** O `value.attribute`
    // declara `mode` e nunca lhe dá hint — de propósito: o picker `Read` (`ParamWidget::Channels`)
    // escreve os dois, e uma row própria para o `mode` seria a segunda superfície a decidir a mesma
    // coisa. A 1.ª redacção desta sonda acusava-o de *«(sem hint)»* e mandava consertar o que já
    // estava certo. ⇒ a isenção é **DERIVADA** do widget que o dobra, nunca uma lista de nomes.
    let dobrados = dobrados_por_outro_widget(&m, grupo);
    eprintln!("\n  param partilhado          | rótulo(s) | quem");
    eprintln!("  --------------------------|-----------|------");
    for (param, rotulos) in &tabela {
        if dobrados.contains(*param) {
            continue;
        }
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

// ---------------------------------------------------------------------------------------------
// 5-bis. O VOCABULÁRIO PELO OUTRO EIXO — o que o ARTISTA lê.
// ---------------------------------------------------------------------------------------------

/// ⭐⭐⭐ **O MESMO VOCABULÁRIO, agrupado pelo RÓTULO e pelas PALAVRAS DO ENUM.**
///
/// ⛔⛔ **A [`vocabulario`] agrupa pela CHAVE, e por isso é cega a metade do problema.** Medido no
/// grupo do ciclo 6: três nós perguntam *«como é que eu interpolo?»* e a sonda por chave só vê dois
/// deles — o `value.map_range` escreve `interpolation` onde o `value.pattern` e o `value.table`
/// escrevem `interp`, logo eles **nunca se encontram** numa tabela indexada por chave. *Uma chave é
/// o que o código escreve; um rótulo é o que o artista lê, e é o rótulo que tem de ser um só.*
///
/// Ela imprime **três** listas, porque são três defeitos diferentes:
///
/// 1. **um RÓTULO, várias chaves** — o artista lê a mesma palavra e o grafo guarda coisas
///    diferentes (pode estar certo: dois nós podem ter *Mode* sem ser o mesmo modo);
/// 2. **uma CHAVE, vários rótulos** — a lista que a [`vocabulario`] já dava, repetida aqui para o
///    leitor não ter de correr duas sondas;
/// 3. ⚠️ **as PALAVRAS de um enum** — o caso mais fino, e o que nenhuma das duas via: dois nós com
///    o mesmo rótulo podem oferecer `Step` num e `Stepped` no outro para a MESMA coisa.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib -- --ignored --nocapture the_vocabulary_the_artist_reads
/// ```
pub fn vocabulario_do_artista(grupo: &[&str]) {
    use std::collections::{BTreeMap, BTreeSet};
    let m = MotionState::new();
    // rótulo → (chave → quem) · chave → (rótulo → quem) · palavra de enum → quem a oferece
    let mut por_rotulo: BTreeMap<String, BTreeMap<&str, Vec<String>>> = BTreeMap::new();
    let mut por_chave: BTreeMap<&str, BTreeMap<String, Vec<String>>> = BTreeMap::new();
    let mut palavras: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for nome in grupo {
        let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
        for h in m.registry.param_ui(tid).unwrap_or(&[]) {
            por_rotulo
                .entry(h.label.to_string())
                .or_default()
                .entry(h.param)
                .or_default()
                .push((*nome).to_string());
            por_chave
                .entry(h.param)
                .or_default()
                .entry(h.label.to_string())
                .or_default()
                .push((*nome).to_string());
            if let ph2d_node_registry::ParamWidget::Enum { labels } = h.widget {
                for w in labels {
                    palavras
                        .entry((*w).to_string())
                        .or_default()
                        .insert(format!("{nome}::{}", h.label));
                }
            }
        }
    }

    eprintln!("\n  (1) UM RÓTULO, VÁRIAS CHAVES -- o artista lê a mesma palavra");
    eprintln!("  rótulo               | chave            | quem");
    eprintln!("  ---------------------|------------------|------");
    for (rotulo, chaves) in &por_rotulo {
        if chaves.len() < 2 {
            continue;
        }
        for (chave, quem) in chaves {
            eprintln!("⚠️  {rotulo:<19} | {chave:<16} | {}", quem.join(", "));
        }
    }

    eprintln!(
        "\n  (2) UMA CHAVE, VÁRIOS RÓTULOS -- o grafo guarda o mesmo e a tela diz outra coisa"
    );
    eprintln!("  chave                | rótulo           | quem");
    eprintln!("  ---------------------|------------------|------");
    for (chave, rotulos) in &por_chave {
        if rotulos.len() < 2 {
            continue;
        }
        for (rotulo, quem) in rotulos {
            eprintln!("⚠️  {chave:<19} | {rotulo:<16} | {}", quem.join(", "));
        }
    }

    // ⚠️ As palavras que se leem como VARIANTES uma da outra (mesmo prefixo de 4 letras, texto
    // diferente) -- `Step`/`Stepped`, `Smooth`/`Smoother`. ⛔ Não é um dicionário: é um sinal para
    // um humano olhar, e por isso a sonda imprime e NÃO reprova.
    eprintln!("\n  (3) PALAVRAS DE ENUM que se leem como variantes uma da outra");
    eprintln!("  palavra              | outra            | quem as oferece");
    eprintln!("  ---------------------|------------------|------");
    let lista: Vec<&String> = palavras.keys().collect();
    for (i, a) in lista.iter().enumerate() {
        for b in lista.iter().skip(i + 1) {
            let (p, q) = (a.to_lowercase(), b.to_lowercase());
            if p.len() >= 4 && q.len() >= 4 && p != q && (q.starts_with(&p) || p.starts_with(&q)) {
                eprintln!(
                    "⚠️  {a:<19} | {b:<16} | {} / {}",
                    palavras[*a].iter().cloned().collect::<Vec<_>>().join(", "),
                    palavras[*b].iter().cloned().collect::<Vec<_>>().join(", ")
                );
            }
        }
    }
    eprintln!();
}

/// **Os params que o widget de OUTRO param já escreve** — o `mode_param` de um
/// [`ParamWidget::Channels`] e os quatro canais de um [`ParamWidget::Color`].
///
/// ⚠️ Derivado do registry, nunca uma lista: o dia em que um widget novo dobrar um vizinho, ele
/// entra aqui por declarar-se, e não por alguém se lembrar.
fn dobrados_por_outro_widget(
    m: &MotionState,
    grupo: &[&str],
) -> std::collections::BTreeSet<String> {
    use ph2d_node_registry::ParamWidget;
    let mut fora = std::collections::BTreeSet::new();
    for nome in grupo {
        let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
        for h in m.registry.param_ui(tid).unwrap_or(&[]) {
            match h.widget {
                ParamWidget::Channels { mode_param, .. } => {
                    fora.insert(mode_param.to_string());
                }
                ParamWidget::Color { channels } => {
                    for c in channels {
                        fora.insert((*c).to_string());
                    }
                }
                _ => {}
            }
        }
    }
    fora
}
