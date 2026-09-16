//! ⭐⭐ **A AUDITORIA DO GRUPO DA APARÊNCIA** (ciclo 7, passo 2 — doc 103 §5).
//!
//! *«A cor e o rasto»*: os seis ciclos anteriores puseram coisas na tela, fizeram-nas andar,
//! dobraram-nas, escolheram quem é afectado, entregaram-nas a uma lei e deram-lhes um cérebro. Este
//! grupo decide **como elas se parecem** — a cor, o brilho, a sombra, o rasto que deixam.
//!
//! ⚠️ **O grupo é a CATEGORIA `Fx` da paleta, pedida ao registry** — que é exactamente o que o
//! artista vê (o cabeçalho magenta), e não um prefixo: a família mistura `motion.*` e `fx.*`, e uma
//! lista escrita à mão aqui envelhecia em silêncio no dia em que um nó `Fx` nascesse.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib -- --ignored --nocapture audit_the_fx_group
//! ```

use ph2d_node_registry::NodeUiCategory;

/// O grupo inteiro — todo tipo registado cuja categoria de paleta é `Fx`, sem as fixturas.
///
/// ⚠️ **A pergunta é a da PALETA** (`motion_bridge_library::build_catalog`): o que o artista vê
/// com o cabeçalho magenta. Uma fixtura não é oferecida, logo não é do grupo.
pub fn grupo() -> Vec<&'static str> {
    let m = crate::motion_state::MotionState::new();
    let mut v: Vec<&'static str> = m
        .registry
        .manifests()
        .filter(|man| !m.registry.is_fixture(man.id))
        .filter(|man| {
            m.registry
                .ui_manifest(man.id)
                .is_some_and(|u| u.category == NodeUiCategory::Fx)
        })
        .map(|man| man.name)
        .collect();
    v.sort_unstable();
    v
}

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn audit_the_fx_group() {
    crate::motion_ciclo_probe::retrato(&grupo());
}

/// **OS PARAMS DE CADA NÓ DO GRUPO** — o que a auditoria compara contra as referências.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_each_fx_node_offers() {
    crate::motion_ciclo_probe::params_de(&grupo());
}

/// **O QUE O CARTÃO PINTA**, com o rótulo da tela.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_the_fx_card_shows() {
    crate::motion_ciclo_probe::cartao(&grupo());
}

/// **OS NOMES dos cartões** — o que o tutorial terá de escrever.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_fx_card_names() {
    crate::motion_ciclo_probe::nomes(&grupo());
}

/// **O VOCABULÁRIO pelo eixo do artista** — rótulo, e as palavras do enum.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_fx_vocabulary_the_artist_reads() {
    crate::motion_ciclo_probe::vocabulario_do_artista(&grupo());
}

/// ⚠️ **O GRUPO ESTÁ VIVO, e cresce com a categoria.**
///
/// ⛔ Sem este piso, uma categoria renomeada ou um registo que deixasse de declarar a UI deixava as
/// sondas acima a auditar **zero** nós — e todas passariam, caladas. O piso é a contagem de
/// 2026-09-16 (`10`), e as duas metades da família têm de estar lá: um filtro que só apanhasse os
/// `fx.*` leria `3` e um que só apanhasse os `motion.*` leria `7`.
#[test]
fn the_fx_group_is_derived_and_not_empty() {
    let g = grupo();
    assert!(
        g.len() >= 10,
        "so' {} no(s) na categoria Fx -- a categoria ou o registo mudaram: {g:?}",
        g.len()
    );
    assert!(
        g.iter().any(|n| n.starts_with("fx.")) && g.iter().any(|n| n.starts_with("motion.")),
        "a categoria Fx mistura `fx.*` e `motion.*`, e o grupo tem de ter as duas: {g:?}"
    );
}

/// **Onde corre uma cadeia `grid 320² → scale → X → output`** — `true` se o planeador a põe
/// inteira no dispositivo. A mesma montagem que a sonda abaixo mede, sem o custo de cozinhar.
fn cadeia_no_dispositivo(no: &str) -> bool {
    use ph2d_nodegraph::graph::Edge;
    let mut m = crate::motion_state::MotionState::new();
    let g = m.doc.graph.add_node("motion.grid".to_string());
    m.doc.graph.set_param(g, "rows", 320.0);
    m.doc.graph.set_param(g, "cols", 320.0);
    let s = m.doc.graph.add_node("motion.scale".to_string());
    let x = m.doc.graph.add_node(no.to_string());
    let o = m.doc.graph.add_node("motion.output".to_string());
    for (de, para) in [(g, s), (s, x), (x, o)] {
        m.doc
            .graph
            .connect(Edge {
                from: (de, 0),
                to: (para, 0),
                delayed: false,
            })
            .expect("fio");
    }
    let dirigidos = crate::motion_bridge::gpu::valores_dirigidos(&mut m, 0.0);
    ph2d_gpu_cook::plan_driven(&m.doc.graph, &m.registry, &m.registry, o, &dirigidos).is_fully_gpu()
}

/// ⭐⭐⭐ **A CATRACA DA ROTA DO GRUPO** (ciclo 7, doc 112 §3) — um nó do grupo no caminho do
/// objecto não pode levar a cadeia para a CPU sem estar NOMEADO aqui.
///
/// ⚠️ **Um nó de aparência é o ÚLTIMO de um grafo**, e por isso a rota dele é a rota do grafo
/// inteiro: a sonda mediu **seis** dos dez a levar `grid 320² → scale` para a CPU e a subir o
/// stream inteiro (`102 400`–`204 800` elementos por quadro). O primeiro a sair desta lista foi o
/// `fx.glow`, que era um **passa-tudo** a derrubar a cadeia para lhe não mudar um byte.
///
/// ⚠️⚠️ **As DUAS metades:** um nó FORA da lista tem de ficar no dispositivo (a regressão que
/// voltaria em silêncio), e um nó DENTRO dela tem de continuar na CPU — senão a lista deixou de o
/// descrever, e *uma catraca sem censo de obsolescência vira licença* (`CLAUDE.md` §5.0). Quem pôr
/// um destes no dispositivo **apaga a linha dele** no mesmo commit.
#[test]
fn the_fx_group_route_only_improves() {
    /// Os que AINDA levam a cadeia para a CPU, cada um com a razão — doc 112 §3.
    ///
    /// ⭐ **Vazia desde a W1d** (2026-09-16): os dez ficam no dispositivo. A lista e as duas metades
    /// ficam — um nó novo da categoria que nasça sem kernel reprova aqui, com o nome.
    const NA_CPU: &[(&str, &str)] = &[];
    let g = grupo();
    assert!(g.len() >= 10, "piso de populacao: {g:?}");
    for (no, razao) in NA_CPU {
        assert!(
            g.contains(no),
            "`{no}` esta' na lista da CPU mas ja' nao e' do grupo Fx — apague a linha"
        );
        assert!(
            !cadeia_no_dispositivo(no),
            "`{no}` ja' fica no dispositivo — apague a linha dele da NA_CPU («{razao}»)"
        );
    }
    for no in &g {
        if NA_CPU.iter().any(|(n, _)| n == no) {
            continue;
        }
        assert!(
            cadeia_no_dispositivo(no),
            "`{no}` leva a cadeia `grid -> scale -> {no} -> output` para a CPU — doc 112 §3"
        );
    }
}

/// ⭐⭐⭐ **SONDA — UMA CADEIA COM UM NÓ DO GRUPO FICA NO DISPOSITIVO?** (ciclo 7, passo 2 · lei 1
/// do §2 do protocolo).
///
/// O retrato diz `NAO` a seis dos dez. ⚠️ **Um `NAO` na coluna do retrato não é um preço** (a lição
/// do ciclo 6, doc 110 §11.2): ele só custa se o nó estiver no caminho do OBJECTO, e o preço é
/// QUANTOS ELEMENTOS sobem na costura. ⇒ cada nó vai para o meio de uma cadeia que JÁ está no
/// dispositivo (`grid 320×320 → scale → X → output`), e pergunta-se ao planeador onde é a costura
/// e quanto sobe por ela.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_does_an_fx_chain_stay -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao, nao um gate"]
fn probe_does_an_fx_chain_stay_on_the_device() {
    use ph2d_nodegraph::graph::{Edge, NodeId};
    let liga =
        |m: &mut crate::motion_state::MotionState, de: (NodeId, u16), para: (NodeId, u16)| {
            m.doc
                .graph
                .connect(Edge {
                    from: de,
                    to: para,
                    delayed: false,
                })
                .expect("fio");
        };
    let mede = |rotulo: &str, meio: Option<&str>, pulso: bool| {
        let mut m = crate::motion_state::MotionState::new();
        let g = m.doc.graph.add_node("motion.grid".to_string());
        // ⚠️ **320 × 320**, a contagem das tabelas do doc 98 — a `3 × 3` do default esconde
        // exactamente a grandeza que esta sonda existe para medir.
        m.doc.graph.set_param(g, "rows", 320.0);
        m.doc.graph.set_param(g, "cols", 320.0);
        let s = m.doc.graph.add_node("motion.scale".to_string());
        let o = m.doc.graph.add_node("motion.output".to_string());
        liga(&mut m, (g, 0), (s, 0));
        let ultimo = match meio {
            None => s,
            Some(nome) => {
                let x = m.doc.graph.add_node(nome.to_string());
                liga(&mut m, (s, 0), (x, 0));
                if pulso {
                    let b = m.doc.graph.add_node("pulse.beat".to_string());
                    liga(&mut m, (s, 0), (b, 0));
                    liga(&mut m, (b, 0), (x, 1));
                }
                x
            }
        };
        liga(&mut m, (ultimo, 0), (o, 0));
        let dirigidos = crate::motion_bridge::gpu::valores_dirigidos(&mut m, 0.0);
        let plano =
            ph2d_gpu_cook::plan_driven(&m.doc.graph, &m.registry, &m.registry, o, &dirigidos);
        let costuras: Vec<String> = plano
            .boundaries
            .iter()
            .map(|(no, porta)| {
                format!(
                    "{}:{porta}",
                    m.doc.graph.node(*no).map_or("?", |i| i.type_name.as_str())
                )
            })
            .collect();
        let elementos: Vec<String> = plano
            .boundaries
            .iter()
            .map(|(no, porta)| {
                m.pump
                    .cook
                    .cook(&m.doc.graph, &m.registry, *no, 0.0)
                    .ok()
                    .and_then(|out| out.get(*porta).map(|v| v.as_stream().count()))
                    .map_or_else(|| "?".to_string(), |n| n.to_string())
            })
            .collect();
        eprintln!(
            "  {rotulo:<34} | {:<11} | {:>6} | {:<26} | {}",
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
                "— (nada sobe)".to_string()
            } else {
                elementos.join(" · ")
            }
        );
    };
    eprintln!(
        "\n  grid 320² -> scale -> X -> output   | onde corre  | stages | costura                    | elementos que SOBEM"
    );
    eprintln!(
        "  -----------------------------------|-------------|--------|----------------------------|--------------------"
    );
    mede("(sem X — o controlo)", None, false);
    for no in grupo() {
        mede(no, Some(no), false);
    }
    // O consumidor que o ciclo 6 deixou a apontar para aqui (doc 110 §8.6): o PULSO a chegar.
    mede("motion.strobe + pulse.beat", Some("motion.strobe"), true);
    eprintln!();
}
