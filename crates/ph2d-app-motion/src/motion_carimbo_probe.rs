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
//! ⚠️ **Três das quatro são IMUNES À CARGA** (uma rota e uma população são decisões, não
//! relógios); só a escada do encode é um relógio, e ela imprime o `loadavg` ao lado. À mão:
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib --release -- --ignored --nocapture audit_the_stamp
//! ```

use ph2d_nodegraph::graph::{Edge, NodeId};

/// O `loadavg` de 1 minuto — ao lado de todo relógio (`CLAUDE.md` §5.0).
fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .unwrap_or("?")
        .to_string()
}

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

/// ⭐⭐ **O QUE CUSTA ENCODAR `N` CÓPIAS DA MESMA FORMA** — a escada que o ADR-0154 Fase 3 pede.
///
/// ⚠️ **Isto É um relógio** (as três sondas acima não são), logo a carga vai impressa ao lado e um
/// número tirado acima de `load ~5` não vale nada (`CLAUDE.md` §5.0).
///
/// ⭐ O lote passa pela **porta do produto** ([`ph2d_vec_render::draw_shared_instances`]), que já
/// tessela cada geometria DISTINTA uma vez — logo o que esta escada mede é o que SOBRA depois
/// dessa economia: o encode de `N` preenchimentos no Vello.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_the_stamp_encode_cost() {
    use ph2d_vector::{Affine, VectorScene};
    let caminho = ph2d_vec_scene::cook(
        ph2d_vec_scene::ShapeKind::Star,
        [-0.5, -0.5],
        [0.5, 0.5],
        &[5.0, 0.5],
    );
    eprintln!(
        "\n  ═══ O ENCODE DE `N` CÓPIAS DA MESMA ESTRELA (load {}) ═══\n",
        carga()
    );
    eprintln!("     cópias |       encode |   % quadro |  por cópia");
    eprintln!("  ----------|--------------|------------|-----------");
    for n in [1_000usize, 10_000, 102_400, 1_000_000] {
        let mut melhor = f64::INFINITY;
        for _ in 0..3 {
            let mut cena = VectorScene::new();
            let t = std::time::Instant::now();
            ph2d_vec_render::draw_shared_instances(
                (0..n).map(|i| {
                    let x = f64::from(u32::try_from(i % 320).unwrap_or(0)) * 0.01;
                    (1u32, Affine::translate((x, 0.0)), [1.0, 1.0, 1.0, 1.0])
                }),
                |_| Some(&caminho),
                &mut cena,
            );
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
        }
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de cena")]
        let por = melhor * 1e3 / n as f64;
        eprintln!(
            "  {n:>9} | {melhor:>9.2} ms | {:>9.0}% | {por:>7.3} µs",
            melhor / 16.67 * 100.0
        );
    }
    eprintln!("\n  load no fim: {}\n", carga());
}

/// Os quatro nomes de param que decidem em que REGIME um `motion.clone` corre.
///
/// ⚠️ **Eles são conferidos contra o manifesto antes de qualquer leitura** ([`regime_do_clone`]):
/// um nome que o nó renomeou lê-se como *«toda gente está no default»*, que é a resposta mais
/// tranquilizadora que uma régua pode dar e a única que não se nota. *Um censo que classifica por
/// string tem de provar que as strings existem.*
const KNOBS_DO_CLONE: [&str; 4] = ["mode", "time_offset", "scale_taper", "rot_taper"];

/// O regime de UM `motion.clone`: `(radial, com leque, com taper)`.
fn regime_do_clone(
    g: &ph2d_nodegraph::graph::Graph,
    reg: &ph2d_node_registry::NodeRegistry,
    id: NodeId,
) -> (bool, bool, bool) {
    let tid = ph2d_nodegraph::node::NodeTypeId::of("motion.clone");
    let man = reg
        .manifests()
        .find(|m| m.id == tid)
        .expect("o `motion.clone` esta' registado");
    for nome in KNOBS_DO_CLONE {
        assert!(
            man.params.iter().any(|p| p.name == nome),
            "o param `{nome}` sumiu do manifesto do `motion.clone` — este censo classificaria \
             tudo como default"
        );
    }
    let p = |nome: &str| -> f32 {
        g.node_param_overrides(id)
            .and_then(|o| o.get(nome).copied())
            .unwrap_or_else(|| {
                man.params
                    .iter()
                    .find(|s| s.name == nome)
                    .map_or(0.0, |s| s.default)
            })
    };
    (
        p("mode") >= 0.5,
        p("time_offset") != 0.0,
        p("scale_taper") != 1.0 || p("rot_taper") != 0.0,
    )
}

/// ⭐⭐⭐ **A POPULAÇÃO DA W1(a)** — em que REGIME o `motion.clone` de facto corre nas cenas do
/// produto (doc 116 §5.1, e a lei do `CLAUDE.md` §0.0: *medir antes de construir*).
///
/// A W1(a) é *«o caso de UMA porta é exprimível com os verbos que já existem»* — e ela tem
/// **fronteiras declaradas** que a auditoria ainda não contou: o leque de relógios é COZIMENTO e
/// não kernel, o modo `Radial` carrega trig por cópia, e o taper **cunha** uma coluna ausente (o
/// `size`/`rot` que a grelha não traz), que é a única coisa da lei que uma binding de escrita não
/// sabe fazer condicionalmente.
///
/// ⇒ *«quantas das cenas que usam o nó caem no regime que a wave alcança?»* é a pergunta que
/// decide se ela vale a pena, e nenhuma régua deste repo a respondia.
#[test]
#[ignore = "sonda de auditoria (ciclo 10), nao gate"]
fn audit_the_stamp_clone_population() {
    let (mut total, mut so_linear, mut com_leque, mut com_taper, mut radiais) = (0, 0, 0, 0, 0);
    let mut cenas: Vec<u32> = Vec::new();
    for level in 1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL {
        let mut m = crate::motion_state::MotionState::new();
        let _ = crate::motion_demo_legend::monta(&level.to_string(), &mut m.doc, &m.registry);
        let clones: Vec<NodeId> = m
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "motion.clone")
            .map(|n| n.id)
            .collect();
        if clones.is_empty() {
            continue;
        }
        cenas.push(level);
        for id in clones {
            total += 1;
            let (radial, leque, taper) = regime_do_clone(&m.doc.graph, &m.registry, id);
            radiais += usize::from(radial);
            com_leque += usize::from(leque);
            com_taper += usize::from(taper);
            so_linear += usize::from(!radial && !leque && !taper);
        }
    }
    eprintln!("\n=== o `motion.clone` nas cenas do produto (ciclo 10, W1a) ===\n");
    eprintln!("  cenas que o usam ...... {} {cenas:?}", cenas.len());
    eprintln!("  cartoes no total ...... {total}");
    eprintln!("  LINEAR, sem leque, sem taper (o que a W1a alcanca) ... {so_linear}");
    eprintln!("  em modo Radial ........ {radiais}");
    eprintln!("  com leque de relogios . {com_leque}");
    eprintln!("  com taper ............. {com_taper}\n");
}

/// ⭐⭐⭐ **A POPULAÇÃO DA W1(b) E DA W2** — quantas cenas do produto levam o `motion.duplicator`
/// (e com que `pick`), e quantas levam uma `source.shape`.
///
/// ⚠️ **É a irmã da [`audit_the_stamp_clone_population`], e existe porque aquela mediu `3` cenas
/// em ~126.** O `motion.clone` é o caso **puro** da contagem — prova-se sem uma forma no caminho —,
/// e o que o dono fotografou nos dois reports é o OUTRO: `grid → shape → duplicator`. *Uma wave
/// escolhida pela pureza da prova e não pela população é uma wave que ninguém sente.*
///
/// ⚠️ O `pick` é a cerca §6.4 do doc 116: só o `Off` é o produto cartesiano — os outros dois
/// emitem exactamente `np` linhas, e uma lei de contagem que o ignore erra em dois dos três.
#[test]
#[ignore = "sonda de auditoria (ciclo 10), nao gate"]
fn audit_the_stamp_duplicator_population() {
    let tid = ph2d_nodegraph::node::NodeTypeId::of("motion.duplicator");
    let (mut dups, mut pick_off, mut com_escala) = (0, 0, 0);
    let (mut cenas_dup, mut cenas_forma): (Vec<u32>, Vec<u32>) = (Vec::new(), Vec::new());
    for level in 1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL {
        let mut m = crate::motion_state::MotionState::new();
        let _ = crate::motion_demo_legend::monta(&level.to_string(), &mut m.doc, &m.registry);
        let man = m
            .registry
            .manifests()
            .find(|man| man.id == tid)
            .expect("o `motion.duplicator` esta' registado");
        for nome in ["pick", "point_scale"] {
            assert!(
                man.params.iter().any(|p| p.name == nome),
                "o param `{nome}` sumiu do manifesto do `motion.duplicator`"
            );
        }
        let mut tem_dup = false;
        let mut tem_forma = false;
        for n in m.doc.graph.nodes() {
            if n.type_name == "source.shape" {
                tem_forma = true;
            }
            if n.type_name != "motion.duplicator" {
                continue;
            }
            tem_dup = true;
            dups += 1;
            let p = |nome: &str| -> f32 {
                m.doc
                    .graph
                    .node_param_overrides(n.id)
                    .and_then(|o| o.get(nome).copied())
                    .unwrap_or_else(|| {
                        man.params
                            .iter()
                            .find(|s| s.name == nome)
                            .map_or(0.0, |s| s.default)
                    })
            };
            pick_off += usize::from(p("pick") < 0.5);
            com_escala += usize::from(p("point_scale") > 0.0);
        }
        if tem_dup {
            cenas_dup.push(level);
        }
        if tem_forma {
            cenas_forma.push(level);
        }
    }
    eprintln!("\n=== o CARIMBO nas cenas do produto (ciclo 10, W1b e W2) ===\n");
    eprintln!("  cenas com `motion.duplicator` ... {}", cenas_dup.len());
    eprintln!("  cenas com `source.shape` ....... {}", cenas_forma.len());
    eprintln!("  cartoes de duplicador .......... {dups}");
    eprintln!("  com `pick = Off` (o produto cartesiano) ... {pick_off}");
    eprintln!("  com `point_scale > 0` .......... {com_escala}\n");
    eprintln!("  cenas do duplicador: {cenas_dup:?}\n");
}

/// ⭐⭐⭐ **DE QUE LADO VEM A FORMA** — o ⏳ que a §5.2 do doc 116 deixou por medir, e o que decide
/// se a W1(b) é um ENTREGÁVEL ou mais uma bancada.
///
/// ⚠️ **A sonda irmã conta os dois nós por CENA; esta conta o PAR.** Para cada cartão de
/// `motion.duplicator` ela sobe a montante da porta `0` (a forma) e da porta `1` (os pontos) e
/// pergunta o que lá encontra — porque as três cercas da §2 vivem em camadas diferentes:
///
/// - uma **`source.shape`** a montante é um **vector VIVO**, e a ponte recusa o grafo inteiro
///   (ADR-0154) **mesmo com o carimbo a ter kernel** ⇒ ali a W1(b) não muda uma linha;
/// - um **`source.object`** é uma textura, e aí a cerca é a da §2.1: dar um verbo estrutural ao
///   carimbo faz o `suffix_changes_count` responder `true` e a ponte passa a **RECUSAR o que hoje
///   aceita** — uma regressão noutra cena, não uma melhoria nesta;
/// - **nenhum dos dois** é o caso em que a W1(b) entrega sozinha.
///
/// ⛔ *Sem esta contagem a wave escolhe-se pela pureza da prova outra vez, que é o erro que a
/// §5.2 já apanhou uma vez.*
#[test]
#[ignore = "sonda de auditoria (ciclo 10), nao gate"]
fn audit_the_stamp_shape_side() {
    /// Os nomes de tipo a montante de `(no, porta)`, subindo por todas as portas.
    fn montante(
        g: &ph2d_nodegraph::graph::Graph,
        no: ph2d_nodegraph::graph::NodeId,
        porta: usize,
        vistos: &mut Vec<String>,
    ) {
        let Some((src, _, _)) = g.input_edge(no, porta) else {
            return;
        };
        let Some(inst) = g.node(src) else { return };
        if vistos.contains(&inst.type_name) && vistos.len() > 64 {
            return; // cerca de ciclo: um grafo do produto é acíclico, isto é defesa
        }
        vistos.push(inst.type_name.clone());
        for p in 0..8 {
            montante(g, src, p, vistos);
        }
    }

    let (mut so_forma, mut so_objecto, mut ambos, mut nenhum, mut total) = (0, 0, 0, 0, 0);
    let (mut com_transfer, mut pontos_com_forma) = (0, 0);
    let mut cenas_objecto: Vec<u32> = Vec::new();
    for level in 1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL {
        let mut m = crate::motion_state::MotionState::new();
        let _ = crate::motion_demo_legend::monta(&level.to_string(), &mut m.doc, &m.registry);
        let man = m
            .registry
            .manifests()
            .find(|man| man.id == ph2d_nodegraph::node::NodeTypeId::of("motion.duplicator"))
            .expect("o `motion.duplicator` esta' registado");
        let ids: Vec<_> = m
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "motion.duplicator")
            .map(|n| n.id)
            .collect();
        for id in ids {
            total += 1;
            let mut lado_forma = Vec::new();
            montante(&m.doc.graph, id, 0, &mut lado_forma);
            let mut lado_pontos = Vec::new();
            montante(&m.doc.graph, id, 1, &mut lado_pontos);
            let f = |v: &[String], t: &str| v.iter().any(|n| n == t);
            let (tem_forma, tem_obj) = (
                f(&lado_forma, "source.shape"),
                f(&lado_forma, "source.object"),
            );
            match (tem_forma, tem_obj) {
                (true, true) => ambos += 1,
                (true, false) => so_forma += 1,
                (false, true) => {
                    so_objecto += 1;
                    if !cenas_objecto.contains(&level) {
                        cenas_objecto.push(level);
                    }
                }
                (false, false) => nenhum += 1,
            }
            pontos_com_forma += usize::from(f(&lado_pontos, "source.shape"));
            let t = m
                .doc
                .graph
                .node_param_overrides(id)
                .and_then(|o| o.get("transfer").copied())
                .unwrap_or_else(|| {
                    man.params
                        .iter()
                        .find(|s| s.name == "transfer")
                        .map_or(0.0, |s| s.default)
                });
            com_transfer += usize::from(t.round() as i32 != 0);
        }
    }
    eprintln!("\n=== DE QUE LADO VEM A FORMA (ciclo 10, W1b) ===\n");
    eprintln!("  cartoes de `motion.duplicator` .............. {total}");
    eprintln!("  porta 0 com `source.shape` (vector VIVO) .... {so_forma}");
    eprintln!("  porta 0 com `source.object` (textura) ....... {so_objecto}");
    eprintln!("  porta 0 com os DOIS ......................... {ambos}");
    eprintln!("  porta 0 com NENHUM dos dois ................. {nenhum}");
    eprintln!("  (controlo) porta 1 com `source.shape` ....... {pontos_com_forma}");
    eprintln!("  com `transfer != Shape Wins` ................ {com_transfer}\n");
    eprintln!("  cenas com objecto no lado da forma: {cenas_objecto:?}\n");
}

/// ⭐⭐⭐ **A ENTRADA DE UM MULTIPLICADOR TRAZ `Index` E `Count`?** — a medição que decide o
/// DESENHO da W1 (doc 116 §5.1).
///
/// ⛔⛔ **A lei da CPU do `motion.clone` RENUMERA**: `Index += cópia · n` e `Count = total`. Num
/// kernel `SourceRows` as colunas que o corpo não escreve chegam por um **gather** do template —
/// uma CÓPIA —, logo renumerar obriga o kernel a escrevê-las. É por isso que o `motion.kaleidoscope`
/// **recua** (`applicable`) quando a renumeração dele está ligada, com a razão escrita no kernel.
///
/// ⚠️ **E escrever uma coluna que a entrada não traz CUNHA-A** — uma coluna a mais viaja, é
/// serializada e muda o que um nó a jusante vê (a lei que o próprio `clone_stream` documenta) ⇒ a
/// pergunta *«ela está lá?»* decide entre três desenhos: escrever sempre · recuar sempre · ou
/// `ReadWriteExisting`, que escreve só quando a coluna existe.
///
/// *A resposta não é adivinhável do manifesto: `Index`/`Count` são escritas por quem gera, e nem
/// toda cadeia passa por um gerador que as escreve.*
#[test]
#[ignore = "sonda de auditoria (ciclo 10), nao gate"]
fn audit_the_stamp_index_and_count_reach_the_multipliers() {
    eprintln!("\n=== a entrada de cada multiplicador traz `Index`/`Count`? ===\n");
    eprintln!("  cena | no'               | porta |    n | Index | Count");
    eprintln!("  -----|-------------------|-------|------|-------|------");
    let (mut com, mut sem) = (0usize, 0usize);
    for level in 1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL {
        let mut m = crate::motion_state::MotionState::new();
        let _ = crate::motion_demo_legend::monta(&level.to_string(), &mut m.doc, &m.registry);
        let alvos: Vec<(NodeId, String)> = m
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "motion.clone" || n.type_name == "motion.duplicator")
            .map(|n| (n.id, n.type_name.clone()))
            .collect();
        if alvos.is_empty() {
            continue;
        }
        crate::motion_shape_gen::publish(&mut m, 0.0);
        for (id, tipo) in alvos {
            // ⚠️ **A entrada é a de quem ALIMENTA a porta**, e não a saída do nó: é sobre ela que
            // o gather de um `SourceRows` copia.
            let fontes: Vec<(u16, NodeId)> = m
                .doc
                .graph
                .edges()
                .iter()
                .filter(|e| e.to.0 == id)
                .map(|e| (e.to.1, e.from.0))
                .collect();
            for (porta, de) in fontes {
                let Ok(o) = m.pump.cook.cook(&m.doc.graph, &m.registry, de, 0.0) else {
                    continue;
                };
                let s = o[0].as_stream();
                let (i, c) = (s.get("Index").is_some(), s.get("Count").is_some());
                if i && c {
                    com += 1;
                } else {
                    sem += 1;
                }
                eprintln!(
                    "  {level:>4} | {tipo:<17} | {porta:>5} | {:>4} | {:>5} | {:>5}",
                    s.count(),
                    if i { "sim" } else { "NAO" },
                    if c { "sim" } else { "NAO" },
                );
            }
        }
    }
    eprintln!("\n  portas COM as duas: {com} · portas sem pelo menos uma: {sem}\n");
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
