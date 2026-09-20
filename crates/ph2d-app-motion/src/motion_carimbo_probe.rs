//! ⭐⭐⭐ **O CARIMBO NO DISPOSITIVO** — a auditoria do ciclo 10 (passo 2; doc 103 §5.1, doc 116).
//!
//! O pedido é do dono e chegou duas vezes em reports: *«no primeiro grafo tentei colocar 1000×1000
//! no grid e pesou muito. Retirando Shape e Duplicator fica um pouco melhor»* (2026-09-06) e
//! *«usando shape (exemplo: star) fps cai para 27»* (2026-09-14).
//!
//! ⚠️ **A fila separa isto em DUAS perguntas** (doc 103 §5.1), e estas sondas medem uma cada:
//!
//! 1. **A CONTAGEM** — uma cadeia com carimbo continuar no dispositivo. O `motion.duplicator` muda
//!    a contagem (`formas × pontos`), que é estrutural, e declara `lowerings: &[Cpu]` ⇒ ele é uma
//!    FRONTEIRA, e um nó de CPU no meio de uma cadeia não custa o que ele custa: custa o
//!    dispositivo inteiro (doc 98, `50,9×`).
//! 2. **A FORMA DESENHÁVEL** — mesmo com a rota curada, `N` cópias de UMA forma vectorial são `N`
//!    caminhos encodados a cada quadro. É a *Fase 3* que o ADR-0154 deixou escrita — *«bake
//!    fallback para contagens extremas, **se** for medido»* — e estas sondas são a medição que ela
//!    pede.
//!
//! ⚠️ **Este ficheiro guarda a metade SINTÉTICA** — o que uma cadeia montada aqui declara. Os dois
//! irmãos foram cortados por RESPONSABILIDADE quando o tecto de LOC mordeu:
//! [`crate::motion_carimbo_populacao_probe`] (quantas CENAS DO PRODUTO caem em cada regime) e
//! [`crate::motion_carimbo_relogio_probe`] (os relógios, com o `loadavg` ao lado).
//!
//! ⚠️ **Nada aqui é um relógio** — uma rota e uma contagem são decisões, logo são imunes à carga.
//! À mão:
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib --release -- --ignored --nocapture audit_the_stamp
//! ```

use ph2d_nodegraph::graph::{Edge, NodeId};

/// O índice de um `kind` da `source.shape`, LIDO do registo pelo rótulo.
///
/// ⚠️ **Não é um literal:** um índice de enum é uma posição numa lista que outra pessoa pode
/// reordenar — a mesma lei que o [`crate::motion_custo_do_quadro_probe`] já paga.
fn indice_da_forma(reg: &ph2d_node_registry::NodeRegistry, rotulo: &str) -> Option<f32> {
    use ph2d_node_registry::ParamWidget;
    let tid = ph2d_nodegraph::node::NodeTypeId::of("source.shape");
    let hint = reg.param_ui(tid)?.iter().find(|h| h.param == "kind")?;
    let ParamWidget::Enum { labels } = hint.widget else {
        return None;
    };
    let i = labels.iter().position(|l| *l == rotulo)?;
    #[expect(
        clippy::cast_precision_loss,
        reason = "um indice de enum, sempre pequeno"
    )]
    Some(i as f32)
}

/// ⚠️ Falha ALTO: uma aresta recusada em silêncio faz a sonda medir um grafo DESLIGADO — o
/// controlo que o [`crate::motion_custo_do_quadro_probe`] pagou primeiro.
fn liga(m: &mut crate::motion_state::MotionState, de: NodeId, dp: u16, para: NodeId, pp: u16) {
    m.doc
        .graph
        .connect(Edge {
            from: (de, dp),
            to: (para, pp),
            delayed: false,
        })
        .unwrap_or_else(|e| panic!("aresta {dp}->{pp} recusada: {e:?}"));
}

/// As cadeias que a auditoria mede, montadas por nome — cada uma com o `sink` devolvido.
///
/// ⚠️ **A grade é `320 × 320`** (`102 400`), que é a população do report da estrela — nem o milhão
/// (onde tudo dói) nem um punhado (onde nada dói).
fn monta(qual: &str) -> (crate::motion_state::MotionState, NodeId) {
    let mut m = crate::motion_state::MotionState::new();
    let estrela = indice_da_forma(&m.registry, "Star");
    let grade = m.doc.graph.add_node("motion.grid");
    m.doc.graph.set_param(grade, "rows", 320.0);
    m.doc.graph.set_param(grade, "cols", 320.0);
    let saida = m.doc.graph.add_node("motion.output");
    let forma_com = |m: &mut crate::motion_state::MotionState| {
        let f = m.doc.graph.add_node("source.shape");
        // ⚠️ O `kind` vem do REGISTO — sem ele a forma emite a primitiva de omissão.
        if let Some(k) = estrela {
            m.doc.graph.set_param(f, "kind", k);
        }
        f
    };
    match qual {
        // O CONTROLO: a grade sozinha é reivindicada pelo dispositivo.
        "grade" => liga(&mut m, grade, 0, saida, 0),
        // O CONTROLO do mapa por-elemento: um deformador não muda a contagem e não recusa.
        "grade + escala" => {
            let s = m.doc.graph.add_node("motion.scale");
            liga(&mut m, grade, 0, s, 0);
            liga(&mut m, s, 0, saida, 0);
        }
        // A cadeia do report: uma forma carimbada em cada ponto.
        "grade + carimbo" => {
            let f = forma_com(&mut m);
            let dup = m.doc.graph.add_node("motion.duplicator");
            liga(&mut m, f, 0, dup, 0);
            liga(&mut m, grade, 0, dup, 1);
            liga(&mut m, dup, 0, saida, 0);
        }
        // O IRMÃO do carimbo: ele também muda a contagem e também é CPU-only.
        "grade + clone" => {
            let c = m.doc.graph.add_node("motion.clone");
            liga(&mut m, grade, 0, c, 0);
            liga(&mut m, c, 0, saida, 0);
        }
        // A forma SEM carimbo — uma peça só, para o controlo do desenho.
        "so' a forma" => {
            let f = forma_com(&mut m);
            liga(&mut m, f, 0, saida, 0);
        }
        outro => panic!("cadeia desconhecida: {outro}"),
    }
    (m, saida)
}

const CADEIAS: [&str; 5] = [
    "grade",
    "grade + escala",
    "grade + carimbo",
    "grade + clone",
    "so' a forma",
];

/// ⭐⭐⭐ **ONDE A CADEIA PARA, E PORQUÊ** — a metade 1, e ela é IMUNE À CARGA.
///
/// A coluna que decide o ciclo não é um relógio: é o `🔴`. Uma fronteira de CPU no meio da cadeia
/// desliga o dispositivo para tudo o que vem antes dela.
#[test]
#[ignore = "sonda de medição — corra à mão, pelo filtro `audit_the_stamp`"]
fn audit_the_stamp_route() {
    eprintln!("\n  ═══ A ROTA DE UMA CADEIA COM CARIMBO (ciclo 10, metade 1) ═══\n");
    eprintln!("  cadeia           | rota      | etapas | fronteira (CPU)      | recusa da ponte");
    eprintln!("  -----------------|-----------|--------|----------------------|----------------");
    for qual in CADEIAS {
        let (mut m, saida) = monta(qual);
        crate::motion_shape_gen::publish(&mut m, 0.0);
        let dirigidos = crate::motion_bridge::gpu::valores_dirigidos(&mut m, 0.0);
        let plano =
            ph2d_gpu_cook::plan_driven(&m.doc.graph, &m.registry, &m.registry, saida, &dirigidos);
        let fronteira: Vec<String> = plano
            .boundaries
            .iter()
            .map(|&(n, _)| {
                m.doc
                    .graph
                    .node(n)
                    .map_or("?".into(), |i| i.type_name.clone())
            })
            .collect();
        // ⚠️ **A SEGUNDA cerca, e ela é de OUTRA camada**: mesmo com o plano inteiro no
        // dispositivo, a ponte recusa um grafo que traga um vector VIVO — ele é desenhado pelo
        // passe vectorial, que o cozimento residente não sabe alimentar (ADR-0154).
        let vivo = m.doc.graph.nodes().iter().any(|n| {
            m.registry
                .is_live_vector_source(ph2d_nodegraph::node::NodeTypeId::of(n.type_name.as_str()))
        });
        eprintln!(
            "  {qual:<16} | {:<9} | {:>6} | {:<20} | {}",
            if plano.is_fully_gpu() {
                "PLACA ok"
            } else {
                "CPU RED"
            },
            plano.dispatching_stages(&m.registry),
            if fronteira.is_empty() {
                "-".into()
            } else {
                fronteira.join(", ")
            },
            if vivo { "vector VIVO RED" } else { "-" },
        );
    }
    eprintln!(
        "\n  ⚠️ as duas colunas da direita são cercas DIFERENTES: a 1.ª é o COZIMENTO, a 2.ª é o DESENHO.\n"
    );
}

/// ⭐⭐⭐ **POR QUE CAMINHO AS PEÇAS DESCEM** — a metade 2, e também imune à carga.
///
/// ⚠️ **As DUAS listas**: uma linha com `geometry_id` desce para `vector_instances` (um CAMINHO
/// vectorial cada) e uma de ladrilho para `instances` (um quad com textura, que o passe de sprites
/// instancia). Contar só a primeira lia ZERO sobre uma cena de formas — a armadilha que o
/// [`crate::motion_custo_do_quadro_probe`] já pagou.
///
/// ⭐ **E a coluna que nomeia a cura é a das GEOMETRIAS DISTINTAS:** `N` cópias da MESMA estrela
/// são `N` encodes de uma geometria só.
#[test]
#[ignore = "sonda de medição — corra à mão, pelo filtro `audit_the_stamp`"]
fn audit_the_stamp_draw() {
    eprintln!("\n  ═══ POR QUE CAMINHO AS PEÇAS DESCEM (ciclo 10, metade 2) ═══\n");
    eprintln!("  cadeia           |      quad |    vector |  geometrias");
    eprintln!("  -----------------|-----------|-----------|------------");
    for qual in ["grade", "grade + carimbo", "so' a forma"] {
        let (mut m, saida) = monta(qual);
        let uv = [0.0, 0.0, 1.0, 1.0];
        let tam = [1.0, 1.0];
        for t in 0..3u64 {
            let ph = f64::from(u32::try_from(t).unwrap_or(0)) / 60.0;
            crate::motion_shape_gen::publish(&mut m, ph);
            m.pump.mark_dirty();
            assert!(
                m.pump
                    .pump(&m.doc.graph, &m.registry, &[saida], t, ph, uv, tam),
                "o quadro tem de cozinhar ({qual})"
            );
        }
        let distintas: std::collections::BTreeSet<u32> = m
            .pump
            .vector_instances
            .iter()
            .map(|vi| vi.geometry_id)
            .collect();
        eprintln!(
            "  {qual:<16} | {:>9} | {:>9} | {:>11}",
            m.pump.instances.len(),
            m.pump.vector_instances.len(),
            distintas.len()
        );
    }
    eprintln!(
        "\n  ⚠️ `geometrias` = quantos `geometry_id` DISTINTOS há entre as linhas vectoriais.\n"
    );
}

/// ⭐⭐ **QUEM JÁ TEM VERBO ESTRUTURAL** — a população em que o carimbo teria de entrar.
///
/// ⛔ **Derivada do REGISTO, nunca escrita à mão:** os cinco verbos do
/// [`StreamOp`](ph2d_nodegraph::gpu::StreamOp) são o canal por onde um nó que muda a contagem
/// declara a lei dele ao sequenciador (ADR-0136). O carimbo não está aqui, e é essa a ausência que
/// o ciclo 10 mede.
#[test]
#[ignore = "sonda de medição — corra à mão, pelo filtro `audit_the_stamp`"]
fn audit_the_stream_op_family() {
    use ph2d_nodegraph::gpu::{KernelResolver, StreamOp};
    // ⛔ Exaustivo de propósito: um verbo NOVO tem de parar aqui em erro de compilação.
    fn verbo(op: &StreamOp) -> &'static str {
        match op {
            StreamOp::Compact { .. } => "Compact",
            StreamOp::SourceRows { .. } => "SourceRows",
            StreamOp::Concat { .. } => "Concat",
            StreamOp::Project { .. } => "Project",
            StreamOp::Carry { .. } => "Carry",
        }
    }
    let m = crate::motion_state::MotionState::new();
    let mut com_verbo: Vec<(&str, &str)> = Vec::new();
    let (mut com_kernel, mut sem_nada, mut total) = (0usize, 0usize, 0usize);
    for man in m.registry.manifests() {
        if m.registry.is_fixture(man.id) {
            continue;
        }
        total += 1;
        if let Some(op) = m.registry.stream_op(man.id) {
            com_verbo.push((man.name, verbo(op)));
        } else if m.registry.gpu_kernel(man.id).is_some() {
            com_kernel += 1;
        } else {
            sem_nada += 1;
        }
    }
    com_verbo.sort_unstable();
    eprintln!("\n  ═══ OS VERBOS ESTRUTURAIS QUE JÁ EXISTEM ═══\n");
    for (no, v) in &com_verbo {
        eprintln!("  {no:<26} {v}");
    }
    eprintln!(
        "\n  {} com verbo estrutural · {com_kernel} com kernel por-elemento · {sem_nada} sem rota \
         nenhuma · {total} nós\n",
        com_verbo.len()
    );
}

/// ⭐⭐⭐ **OS `len` DE UMA CADEIA SÃO UNIFORMES?** — a medição que decide a cura do report
/// *«porque a corda afina no final?»* (dono, 2026-09-20).
///
/// ⛔⛔ **As duas ordens dele colidem exactamente aqui.** *«A peça ajusta-se ao tamanho do osso»*
/// diz que a peça segue o `len`; *«a corda não deve afinar»* diz que a ESPESSURA não deve.
/// Numa cadeia de `len` UNIFORMES as duas dizem o mesmo e não há escolha a fazer; numa de `len`
/// variados elas pedem coisas opostas.
///
/// ⇒ esta sonda conta, em cada `rig.bones` de cada cena, a dispersão dos `len` que ele entrega.
/// *Uma decisão de produto tomada sem saber se o caso que a força existe é um palpite.*
#[test]
#[ignore = "sonda de auditoria (report da corda), nao gate"]
fn audit_the_bone_lengths_are_uniform() {
    eprintln!("\n=== dispersao dos `len` em cada cadeia de ossos do produto ===\n");
    eprintln!("  cena | cartao |    n |      min |      max |  max/min | mediana");
    eprintln!("  -----|--------|------|----------|----------|----------|--------");
    let mut variadas = 0usize;
    for level in 1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL {
        let mut m = crate::motion_state::MotionState::new();
        let (sinks, _) =
            crate::motion_demo_legend::monta(&level.to_string(), &mut m.doc, &m.registry);
        let ossos: Vec<NodeId> = m
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "rig.bones")
            .map(|n| n.id)
            .collect();
        if ossos.is_empty() {
            continue;
        }
        crate::motion_shape_gen::publish(&mut m, 0.0);
        // ⚠️ **Em REGIME, não no tique zero:** uma corda nasce recta e no tique `0` os `len` dela
        // são todos o repouso — *a fixtura do instante zero não contém o fenómeno*.
        let mut t = 0.0f64;
        for _ in 0..40 {
            for s in &sinks {
                let _ = m.pump.cook.cook(&m.doc.graph, &m.registry, *s, t);
            }
            let _ = m.pump.cook.advance_tick(&m.doc.graph, &m.registry, t);
            t += 1.0 / 60.0;
        }
        for (k, id) in ossos.iter().enumerate() {
            let Ok(o) = m.pump.cook.cook(&m.doc.graph, &m.registry, *id, t) else {
                continue;
            };
            let Some(ph2d_nodegraph::attr::Column::Scalar(len)) = o[0].as_stream().get("len")
            else {
                continue;
            };
            if len.is_empty() {
                continue;
            }
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            for v in len {
                lo = lo.min(*v);
                hi = hi.max(*v);
            }
            let mut ord = len.clone();
            ord.sort_by(f32::total_cmp);
            let med = ord[ord.len() / 2];
            let razao = hi / lo.max(f32::MIN_POSITIVE);
            if razao > 1.01 {
                variadas += 1;
            }
            eprintln!(
                "  {level:>4} | {k:>6} | {:>4} | {lo:>8.5} | {hi:>8.5} | {razao:>7.3}x | {med:>7.5}",
                len.len()
            );
        }
    }
    eprintln!("\n  cadeias com `len` NAO uniforme (max/min > 1,01): {variadas}\n");
}
