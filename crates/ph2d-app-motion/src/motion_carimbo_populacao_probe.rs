//! ⭐⭐⭐ **A POPULAÇÃO DO CARIMBO NAS CENAS DO PRODUTO** — irmão do
//! [`crate::motion_carimbo_probe`] pelo tecto de LOC, e o corte é por RESPONSABILIDADE, não por
//! tamanho: ali mede-se o que uma cadeia **SINTÉTICA** declara (a rota, os verbos estruturais),
//! aqui **quantas cenas do produto caem em cada regime**. O precedente é o par `motion_ciclo_*`
//! desta mesma crate — *o que um nó DECLARA fica num sítio, o que o CORPUS traz noutro*.
//!
//! ⚠️ **Nenhuma destas é um relógio.** São contagens sobre os documentos que shipam, logo são
//! imunes à carga da máquina — e é por isso que elas podem viver longe do `loadavg`. O relógio do
//! carimbo mudou-se para o terceiro irmão, [`crate::motion_carimbo_relogio_probe`], que imprime a
//! carga ao lado de cada número (`CLAUDE.md` §5.0).
//!
//! ⭐ **Cada sonda daqui imprime a POPULAÇÃO que produziu** — a lei que o
//! [`crate::motion_custo_do_quadro_probe`] pagou: *contar a lista errada lê-se exactamente como uma
//! cena vazia*.
//!
//! À mão:
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib -- --ignored --nocapture audit_the_stamp
//! ```

use ph2d_nodegraph::graph::NodeId;

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

/// ⭐⭐⭐ **QUANTAS LINHAS VECTORIAIS AS CENAS DO PRODUTO DE FACTO DESENHAM** — a medição que decide
/// se a W2 é um entregável ou a TERCEIRA bancada seguida.
///
/// ⚠️ **A escada do §4.3 é uma FUNÇÃO, não um veredito.** Ela diz quanto custam `N` cópias; o que
/// ela **não** diz é qual é o `N` das cenas que existem. O report do dono é `1000 × 1000`, mas um
/// cartão do produto pode ser de dezenas — e a `0,056 µs` por cópia, dezenas custam **nada**.
///
/// ⛔ *Duas waves seguidas foram reordenadas por eu ter lido a população do NÓ e não a da CENA
/// (§5.2 e §5.6). Esta pergunta é a mesma, um eixo acima: a população do DESENHO.*
///
/// Cozinha cada cena com carimbo pela porta do produto (o `pump`, três tiques) e conta as linhas
/// **vectoriais** — que são exactamente as que pagam o encode do §4.3.
#[test]
#[ignore = "sonda de auditoria (ciclo 10), nao gate"]
fn audit_the_stamp_live_vector_population() {
    let uv = [0.0, 0.0, 1.0, 1.0];
    let tam = [1.0, 1.0];
    eprintln!("\n  ═══ LINHAS VECTORIAIS POR CENA DO PRODUTO (ciclo 10, W2) ═══\n");
    eprintln!("     cena |   vector |     quad | geometrias | encode previsto");
    eprintln!("  --------|----------|----------|------------|----------------");
    let (mut pior, mut cena_pior, mut acima_de_10k) = (0usize, 0u32, 0usize);
    for level in 1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL {
        let mut m = crate::motion_state::MotionState::new();
        let _ = crate::motion_demo_legend::monta(&level.to_string(), &mut m.doc, &m.registry);
        if !m
            .doc
            .graph
            .nodes()
            .iter()
            .any(|n| n.type_name == "motion.duplicator")
        {
            continue;
        }
        let saidas: Vec<_> = m
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "motion.output")
            .map(|n| n.id)
            .collect();
        if saidas.is_empty() {
            eprintln!("  {level:>7} |  (a cena nao tem `motion.output`)");
            continue;
        }
        for t in 0..3u64 {
            let ph = f64::from(u32::try_from(t).unwrap_or(0)) / 60.0;
            crate::motion_shape_gen::publish(&mut m, ph);
            m.pump.mark_dirty();
            let _ = m
                .pump
                .pump(&m.doc.graph, &m.registry, &saidas, t, ph, uv, tam);
        }
        let n = m.pump.vector_instances.len();
        let distintas: std::collections::BTreeSet<u32> = m
            .pump
            .vector_instances
            .iter()
            .map(|vi| vi.geometry_id)
            .collect();
        // O µs/cópia do §4.3, no regime plano.
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de cena")]
        let previsto = n as f64 * 0.088e-3;
        if n > pior {
            pior = n;
            cena_pior = level;
        }
        acima_de_10k += usize::from(n >= 10_000);
        eprintln!(
            "  {level:>7} | {n:>8} | {:>8} | {:>10} | {previsto:>10.3} ms",
            m.pump.instances.len(),
            distintas.len()
        );
    }
    eprintln!("\n  pior cena: `={cena_pior}` com {pior} linhas vectoriais");
    eprintln!("  cenas com >= 10 000 linhas vectoriais: {acima_de_10k}");
    eprintln!("\n  ⚠️ `encode previsto` = linhas x 0,088 us (o regime PLANO do §4.3).\n");
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
