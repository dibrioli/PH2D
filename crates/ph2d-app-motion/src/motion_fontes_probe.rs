//! ⭐⭐ **A AUDITORIA DO GRUPO DAS FONTES** (ciclo 8, passo 2 — doc 103 §5, doc 113).
//!
//! *«De onde vêm as coisas»*: os sete ciclos anteriores fizeram coisas às peças — pô-las em grelha,
//! movê-las, dobrá-las, escolher quem é afectado, entregá-las a uma lei, dar-lhes um cérebro e uma
//! cara. Este grupo responde à pergunta que vem ANTES de todas: **de onde vem a peça** — uma forma
//! desenhada, um objecto da cena, um texto, uma tabela, uma gramática que cresce, um emissor.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib -- --ignored --nocapture the_source_palette_census
//! ```

use ph2d_node_registry::NodeUiCategory;

/// **O CENSO DA PALETA** — cada tipo oferecido, a categoria dele e o nome do cartão.
///
/// ⚠️ É o instrumento do passo 1: o grupo sai do que o artista VÊ (a categoria e o sub-cluster), e
/// a linha do doc 103 §5 é uma verificação, não a fonte.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_source_palette_census() {
    let m = crate::motion_state::MotionState::new();
    let mut linhas: Vec<(String, &'static str, String)> = m
        .registry
        .manifests()
        .filter(|man| !m.registry.is_fixture(man.id))
        .filter_map(|man| {
            m.registry.ui_manifest(man.id).map(|u| {
                (
                    format!("{:?}", u.category),
                    man.name,
                    u.display_name.to_string(),
                )
            })
        })
        .collect();
    linhas.sort();
    let mut atual = String::new();
    for (cat, nome, cartao) in &linhas {
        if *cat != atual {
            eprintln!("\n  == {cat}");
            atual = cat.clone();
        }
        eprintln!("  {nome:<32} | {cartao}");
    }
    let fontes = linhas
        .iter()
        .filter(|(c, _, _)| *c == format!("{:?}", NodeUiCategory::Source))
        .count();
    eprintln!("\n  categoria Source: {fontes} tipos\n");
}

/// O único nó do grupo que não é `source.*`: o emissor de partículas. ⚠️ Ele é `Source` na paleta
/// e o ciclo 5 (a simulação) não o contou — o doc 103 §5 põe-no aqui, e é aqui que ele se lê: um
/// emissor é **de onde vêm** as partículas.
const EMISSOR: &str = "motion.emitter";

/// O grupo — a categoria `Source` da paleta, no sub-grupo das fontes de DADOS (`source.*`) mais o
/// emissor.
///
/// ⚠️ **Os outros catorze tipos da categoria têm dono noutro ciclo** (os arranjos no 1, o
/// `sim.spawn` no 5, o `value.pattern` no 6, os corpos moles e o esqueleto no 9) — e o prefixo é o
/// que o artista vê como sub-grupo: os cartões `Shape · Object · Text · Table · L-System` são as
/// fontes que partem de uma coisa que ELE fez (um desenho, um objecto, um texto, um ficheiro, uma
/// gramática).
pub fn grupo() -> Vec<&'static str> {
    let m = crate::motion_state::MotionState::new();
    let mut v: Vec<&'static str> = m
        .registry
        .manifests()
        .filter(|man| !m.registry.is_fixture(man.id))
        .filter(|man| {
            m.registry
                .ui_manifest(man.id)
                .is_some_and(|u| u.category == NodeUiCategory::Source)
        })
        .map(|man| man.name)
        .filter(|n| n.starts_with("source.") || *n == EMISSOR)
        .collect();
    v.sort_unstable();
    v
}

/// ⚠️ **O GRUPO ESTÁ VIVO** — piso de população e as duas metades (as `source.*` e o emissor).
#[test]
fn the_source_group_is_derived_and_not_empty() {
    let g = grupo();
    assert!(g.len() >= 6, "piso de populacao (6 em 2026-09-16): {g:?}");
    assert!(
        g.contains(&EMISSOR),
        "o emissor saiu da categoria Source: {g:?}"
    );
    assert!(
        g.iter().filter(|n| n.starts_with("source.")).count() >= 5,
        "as fontes de dados sairam da categoria Source: {g:?}"
    );
}

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn audit_the_source_group() {
    crate::motion_ciclo_probe::retrato(&grupo());
}

/// **OS PARAMS DE CADA NÓ DO GRUPO** — o que a auditoria compara contra as referências.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_each_source_node_offers() {
    crate::motion_ciclo_probe::params_de(&grupo());
}

/// **O QUE O CARTÃO PINTA**, com o rótulo da tela.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_the_source_card_shows() {
    crate::motion_ciclo_probe::cartao(&grupo());
}

/// **O VOCABULÁRIO pelo eixo do artista** — rótulo, e as palavras do enum.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_source_vocabulary_the_artist_reads() {
    crate::motion_ciclo_probe::vocabulario(&grupo());
    crate::motion_ciclo_probe::vocabulario_do_artista(&grupo());
}

/// ⭐⭐⭐ **SONDA — ONDE CORRE UMA CADEIA QUE COMEÇA NUMA FONTE?** (lei 1 do protocolo).
///
/// ⚠️ **Uma fonte é o PRIMEIRO nó de um grafo** (o espelho do ciclo 7, onde a aparência era o
/// último): se ela não tem kernel, TODA a cadeia a jusante cozinha na CPU para a alimentar — a
/// mesma costura, na outra ponta. ⇒ cada nó à cabeça de `X → scale → output`, e pergunta-se ao
/// planeador onde é a costura e quanto sobe por ela.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_does_a_source_chain_stay -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao, nao um gate"]
fn probe_does_a_source_chain_stay_on_the_device() {
    use ph2d_nodegraph::graph::Edge;
    let mede = |no: &str| {
        let mut m = crate::motion_state::MotionState::new();
        let x = m.doc.graph.add_node(no.to_string());
        let s = m.doc.graph.add_node("motion.scale".to_string());
        let o = m.doc.graph.add_node("motion.output".to_string());
        let mut ligou = true;
        for (de, para) in [(x, s), (s, o)] {
            ligou &= m
                .doc
                .graph
                .connect(Edge {
                    from: (de, 0),
                    to: (para, 0),
                    delayed: false,
                })
                .is_ok();
        }
        if !ligou {
            eprintln!("  {no:<22} | a saida 0 nao liga a um `Scale` (outro tipo de porta)");
            return;
        }
        let dirigidos = crate::motion_bridge::gpu::valores_dirigidos(&mut m, 0.0);
        let plano =
            ph2d_gpu_cook::plan_driven(&m.doc.graph, &m.registry, &m.registry, o, &dirigidos);
        let costuras: Vec<String> = plano
            .boundaries
            .iter()
            .map(|(n, p)| {
                format!(
                    "{}:{p}",
                    m.doc.graph.node(*n).map_or("?", |i| i.type_name.as_str())
                )
            })
            .collect();
        let elementos: Vec<String> = plano
            .boundaries
            .iter()
            .map(|(n, p)| {
                m.pump
                    .cook
                    .cook(&m.doc.graph, &m.registry, *n, 0.0)
                    .ok()
                    .and_then(|out| out.get(*p).map(|v| v.as_stream().count()))
                    .map_or_else(|| "?".to_string(), |k| k.to_string())
            })
            .collect();
        eprintln!(
            "  {no:<22} | {:<11} | {:>6} | {:<24} | {}",
            if plano.is_fully_gpu() {
                "dispositivo"
            } else {
                "⛔ CPU"
            },
            plano.stages.len(),
            if costuras.is_empty() {
                "—".to_string()
            } else {
                costuras.join(" · ")
            },
            if elementos.is_empty() {
                "—".to_string()
            } else {
                elementos.join(" · ")
            }
        );
    };
    eprintln!(
        "\n  X -> scale -> output   | onde corre  | stages | costura                  | elementos que SOBEM"
    );
    mede("motion.grid");
    for no in grupo() {
        mede(no);
    }
    eprintln!();
}
