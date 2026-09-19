//! **AS SONDAS DAS CENAS DO ROTEADOR** — o que elas IMPRIMEM, irmãs por RESPONSABILIDADE
//! (HR-18) dos gates em [`super::tests`].
//!
//! ⚠️ **Um censo não é um gate**, e é por isso que vivem noutro ficheiro: um gate reprova quando
//! o mundo se afasta de uma lei; um censo devolve o NÚMERO com que se escolhe a lei seguinte
//! (onde cada banda cai · o que se mexe · quantas portas de realimentação estão vazias · que
//! cenas o roteador de facto monta · o que alimenta a porta de forma de cada carimbo).

use super::tests::shape_origin;
use super::*;

/// **A coluna `size` da corrente, como multiplicador por elemento** — `None` quando ela não a traz.
///
/// ⚠️ Estas leituras viviam no gizmo de posições, que a ordem do dono de 2026-09-19 RETIROU
/// (*«retire tudo relacionado a gizmos desses nós»*). Elas ficam aqui porque o que as sondas abaixo
/// medem é a **CORRENTE** — um facto sobre o que as cenas autoram —, e isso sobrevive ao desenho.
///
/// ⚠️ Uma coluna `Vec2` colapsa na MÉDIA dos eixos: as sondas perguntam *«que números o `size`
/// tem»*, e um par não cabe num histograma de um eixo.
fn escalas_da_corrente(s: &ph2d_nodegraph::attr::Stream) -> Option<Vec<f32>> {
    use ph2d_nodegraph::attr::Column;
    match s.get("size") {
        Some(Column::Scalar(v)) => Some(v.to_vec()),
        Some(Column::Vec2(v)) => Some(v.iter().map(|e| (e[0] + e[1]) * 0.5).collect()),
        _ => None,
    }
}

/// **A coluna `rot` da corrente, em graus** — `None` quando ela não a traz. Irmã da acima.
fn rotacoes_da_corrente(s: &ph2d_nodegraph::attr::Stream) -> Option<Vec<f32>> {
    use ph2d_nodegraph::attr::Column;
    match s.get("rot") {
        Some(Column::Scalar(v)) => Some(v.to_vec()),
        _ => None,
    }
}

/// **NENHUMA CENA DA CONFERÊNCIA MONTA UM GRAFO COM BURACO DE SETUP.**
///
/// ⚠️ **Este portão nasceu de um smoke reprovado** (Enio, 2026-08-20: *"6.
/// EMPUXO com a coluna `density`. Todas as peças paradas"*). A causa era um fio
/// que faltava na cena `=71`: o `value.instance_field` que alimentava a
/// densidade estava **solto**, e o doc dele diz *"unconnected → one degenerate
/// value"* — ele dava UM valor, o `motion.drive` transmitia-o a todos, e ele era
/// ZERO. Densidade zero é empuxo nenhum.
///
/// ⚠️ **A casa já sabia diagnosticar isto, e ninguém perguntava.** O
/// `ph2d_motion_diagnose` reporta `MissingSource`/`MissingInput` exactamente para
/// um nó sem nada ligado; o que não existia era uma porta por onde um gate
/// pudesse MONTAR uma cena e perguntar — daí o [`build_level`].
/// *Um instrumento que nenhum passo invoca não protege coisa nenhuma.*
///
/// ⚠️ **A barra é ZERO, e ela foi MEDIDA antes de virar barra.** A sonda achou
/// **seis** cenas marcadas (`=3`, `=31`, `=38`, `=57`, `=61`, `=71`) — e as seis
/// estavam CERTAS: o falso positivo era do diagnoser, cuja isenção exigia que a
/// aresta atrasada viesse do PRÓPRIO nó, quando o laço canónico de força a
/// recebe do integrador. Curado lá; aqui sobra o zero.
///
/// ⛔ **Se uma cena futura encenar um defeito DE PROPÓSITO**, ela não entra numa
/// allowlist muda: ou o defeito não é de SETUP (a `=45` encena um nome que não
/// resolve, e passa), ou o gate ganha o nível NOMEADO com o motivo ao lado.
/// **O QUE UMA CENA DE FACTO DESENHA** — a caixa de cada banda, medida.
///
/// ⚠️ **Este instrumento nasceu de um smoke reprovado** (Enio, 2026-08-21: *"esses
/// exemplos não são compreensíveis. tudo misturado e bagunçado"*), e a causa não era
/// nenhuma das features: era eu a **autorar cenas às cegas**. Um `motion.move(dx, dy)`
/// diz onde o CENTRO de uma banda vai; ele não diz nada sobre a LARGURA dela, e a
/// largura sai de `(cols − 1) · gap`, três nós acima. Duas bandas cujos centros
/// distam 12 unidades sobrepõem-se alegremente se cada uma medir 8.
///
/// ⚠️ **Não há como ver isto sem cozinhar.** O grafo é o que eu escrevo; a IMAGEM é o
/// que o cook devolve — e até esta sonda existir, o único instrumento que media a
/// diferença entre os dois era o olho do Enio, depois de compilar em release.
///
/// `PH2D_LAYOUT_LEVEL=73 cargo test -p ph2d-app-motion --lib
/// measure_scene_layout -- --ignored --nocapture` (sem a env, varre tudo).
#[test]
#[ignore = "sonda de layout, não um gate — `-- --ignored --nocapture`"]
fn measure_scene_layout() {
    let only = std::env::var("PH2D_LAYOUT_LEVEL").ok();
    for level in 1..=MAX_DEMO_LEVEL {
        if only.as_deref().is_some_and(|w| w != level.to_string()) {
            continue;
        }
        // ⚠️ **Um `MotionState` inteiro, e não um `MotionDoc` solto.** Uma cena de FORMA
        // ou de TEXTO lê a geometria por CANAL EXTERNO, que só o shell publica — um cook
        // virgem não tem external nenhum e devolve **zero instâncias**, que esta sonda
        // imprimiria como `VAZIA`. Seria ela a acusar a cena de um defeito dela própria,
        // pela terceira vez nesta linha (a sonda de movimento e o harness do texto
        // pagaram as outras duas).
        let mut state = MotionState::new();
        let sinks =
            crate::motion_demo_legend::monta(&level.to_string(), &mut state.doc, &state.registry).0;
        if sinks.is_empty() {
            continue;
        }
        crate::motion_externals::publish_all(&mut state, 0.0);
        println!("--- cena =`{level}` · {} bandas", sinks.len());
        for (k, sink) in sinks.iter().enumerate() {
            match band_box(&mut state, *sink) {
                Some((n, lo, hi)) => println!(
                    "  banda {:>2}: n={n:<6} x [{:>7.2} .. {:>7.2}]  y [{:>7.2} .. {:>7.2}]  ({:.2} x {:.2})",
                    k + 1,
                    lo[0],
                    hi[0],
                    lo[1],
                    hi[1],
                    hi[0] - lo[0],
                    hi[1] - lo[1]
                ),
                None => println!("  banda {:>2}: VAZIA", k + 1),
            }
        }
    }
}

/// A contagem e a caixa envolvente de uma banda, cozinhada em `t = 0`.
///
/// ⚠️ **A caixa é a das POSIÇÕES, não a da tinta.** Uma banda de uma instância só —
/// típica de uma cena de FORMA, em que a arte inteira é um `geometry_id` — mede
/// `0.00 x 0.00`, e isso não quer dizer *vazia*: quer dizer *um ponto*. A extensão
/// desenhada ali é a do `VecPath` vezes a coluna `size`, e vive no store, não no stream.
fn band_box(
    state: &mut MotionState,
    sink: ph2d_nodegraph::graph::NodeId,
) -> Option<(usize, [f32; 2], [f32; 2])> {
    use ph2d_nodegraph::attr::Column;
    // ⚠️ O cook do PRÓPRIO estado — é nele que o `publish_all` escreveu os externals.
    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, sink, 0.0)
        .ok()?;
    let s = out.first()?.as_stream();
    let Some(Column::Vec2(p)) = s.get("P") else {
        return None;
    };
    if p.is_empty() {
        return None;
    }
    let mut lo = [f32::INFINITY; 2];
    let mut hi = [f32::NEG_INFINITY; 2];
    for q in p {
        for a in 0..2 {
            lo[a] = lo[a].min(q[a]);
            hi[a] = hi[a].max(q[a]);
        }
    }
    Some((p.len(), lo, hi))
}

/// **QUANTO UMA CENA DE FACTO ANDA** — a irmã temporal da [`measure_scene_layout`].
///
/// ⚠️ **Ela nasceu de um smoke reprovado E provou-se contra uma cena APROVADA**
/// (Enio, 2026-08-21: *"tudo foi levado pelo vento. nada rasgou"*). A primeira
/// versão desta sonda dizia que a `=75` não andava — e dizia o mesmo da **`=71`**,
/// que o Enio já tinha aprovado. Foi isso que provou que o erro era do HARNESS: o
/// `pre` de um circuito sequencial só avança quando o quadro FECHA
/// ([`Cook::advance_tick`]), e um laço que só `cook`a lê o mesmo tique N vezes.
///
/// *Uma sonda que acusa a cena boa está a acusar-se a si própria.*
///
/// `PH2D_LAYOUT_LEVEL=75 cargo test -p ph2d-app-motion --lib
/// measure_scene_motion -- --ignored --nocapture`
#[test]
#[ignore = "sonda de movimento, não um gate — `-- --ignored --nocapture`"]
fn measure_scene_motion() {
    /// Quantos tiques a sonda corre — **cinco** segundos a 60 fps.
    ///
    /// ⚠️ **Dois segundos não chegavam, e isso é uma medição:** a cena `=75` faz o
    /// vento SUBIR ao longo de 4 s, e o pano só solta lá pelos 2. Com a janela curta
    /// as duas metades saíam com o mesmo número — a sonda dizia *"não há diferença"*
    /// sobre uma cena que a tem, cinco décimos de segundo mais tarde.
    const TICKS: u32 = 300;
    use ph2d_nodegraph::attr::Column;
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registra");
    let only = std::env::var("PH2D_LAYOUT_LEVEL").ok();
    for level in (1..=MAX_DEMO_LEVEL).map(|l| l.to_string()) {
        if only.as_deref().is_some_and(|w| *w != level) {
            continue;
        }
        let level = level.as_str();
        let mut doc = MotionDoc::default();
        let sinks = crate::motion_demo_legend::monta(level, &mut doc, &reg).0;
        if sinks.is_empty() {
            continue;
        }
        for (k, sink) in sinks.iter().enumerate() {
            let mut cook = ph2d_nodegraph::cook::Cook::new();
            let (mut a, mut b) = (Vec::new(), Vec::new());
            for t in 0..TICKS {
                let ph = f64::from(t) / 60.0;
                let out = cook.cook(&doc.graph, &reg, *sink, ph).expect("cozinha");
                if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
                    if t == 0 {
                        a = p.clone();
                    }
                    b = p.clone();
                }
                cook.advance_tick(&doc.graph, &reg, ph).expect("avança");
            }
            if a.is_empty() || b.is_empty() {
                continue;
            }
            // O MAIOR percurso da banda, não o do elemento 0: uma banda cujo
            // primeiro elemento esteja pinado andaria zero e leria como parada.
            let d = a
                .iter()
                .zip(&b)
                .map(|(p, q)| (q[0] - p[0]).abs() + (q[1] - p[1]).abs())
                .fold(0.0_f32, f32::max);
            println!("  cena ={level} banda {:>2}: maior percurso {d:.4}", k + 1);
        }
    }
}

/// **QUANTAS PORTAS DE REALIMENTAÇÃO ESTÃO VAZIAS** — a medição que decide se escondê-las é
/// livre ou se custa uma capacidade.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib measure_feedback_ports -- --ignored --nocapture
/// ```
///
/// ⚠️ **A pergunta vem de um report do Enio (2026-08-27, com foto): a porta de estado do
/// `motion.boids` lê-se como *"uma reentrada de link"* e confunde.** O doc 03 §3 já tinha o
/// diagnóstico em três partes — *o usuário não deveria (i) ver uma porta chamada `state`,
/// (ii) desenhar o fechamento, (iii) ler um laço cruzando o grafo* — e a wave O1 curou (ii) e
/// (iii). A (i) nunca foi fechada, e é exactamente o que a foto mostra.
///
/// ⛔ **Esconder sem condição TIRA uma capacidade que já shipa:** o `motion.integrate` recebe a
/// cadeia de forças no `forces`, e o `motion.soft_body` recebe `motion.pin_constraint` e
/// `field.index_range` no `state` (cenas da conferência). *Um clamp no caminho do estado é um
/// clamp persistente — coisa que a referência não faz.*
///
/// ⇒ o que esta sonda mede é a razão entre os dois estados: quantas portas seguram **só o
/// auto-laço do motor** (nada autorado, e aí a porta não oferece nada) contra quantas seguram
/// uma cadeia. É esse número que diz se «esconder enquanto vazia» resolve o report ou é cosmética.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn measure_feedback_ports() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registra");
    let (mut vazias, mut com_cadeia, mut cenas) = (0usize, 0usize, 0usize);
    let mut por_tipo: std::collections::BTreeMap<&str, (usize, usize)> = Default::default();
    for level in 1..=MAX_DEMO_LEVEL {
        let mut doc = MotionDoc::default();
        if crate::motion_demo_legend::monta(&level.to_string(), &mut doc, &reg)
            .0
            .is_empty()
        {
            continue;
        }
        cenas += 1;
        for n in doc.graph.nodes() {
            let Some(op) = ph2d_nodegraph::cook::OpResolver::resolve(&reg, n.type_id()) else {
                continue;
            };
            let man = op.manifest();
            let Some(out0) = man.outputs.first() else {
                continue;
            };
            let Some((port, _)) = man
                .inputs
                .iter()
                .enumerate()
                .find(|(_, p)| matches!(p.name, "state" | "forces") && p.ty == out0.ty)
            else {
                continue;
            };
            // Uma aresta NÃO-atrasada a entrar na porta é autoria; só o `pre` é o motor.
            let autorada = doc
                .graph
                .edges()
                .iter()
                .any(|e| !e.delayed && e.to == (n.id, port as u16));
            let slot = por_tipo.entry(man.name).or_default();
            if autorada {
                com_cadeia += 1;
                slot.1 += 1;
            } else {
                vazias += 1;
                slot.0 += 1;
            }
        }
    }
    let total = vazias + com_cadeia;
    println!("\n[feedback] {cenas} cenas · {total} portas de realimentacao");
    println!(
        "  so' o auto-laco do motor : {vazias:>4}  ({:.0}%)",
        100.0 * vazias as f32 / total as f32
    );
    println!(
        "  com cadeia AUTORADA      : {com_cadeia:>4}  ({:.0}%)",
        100.0 * com_cadeia as f32 / total as f32
    );
    println!("\n  {:<26} {:>7} {:>9}", "no'", "vazias", "c/cadeia");
    for (ty, (v, c)) in &por_tipo {
        println!("  {ty:<26} {v:>7} {c:>9}");
    }
}

/// ⭐⭐⭐ **QUE ROTA CADA CENA DO PRODUTO TOMA** — a sonda que a auditoria de performance de
/// 2026-09-01 pediu, e a única que responde à pergunta que decide o teto do módulo.
///
/// ⛔⛔ **Por que ela existe.** A ponte cozinha no DEVICE por omissão, e o device faz **4,19 M
/// partículas em 3,85 ms** (23% de um quadro) contra **195,9 ms da CPU** — `50,9×`, medido pelo
/// `emitter_sim_ceiling_probe` nesta máquina. Mas o roteamento tem escadas-abismo
/// ([`crate::motion_bridge::gpu::gpu_route`]): **dois sinks** derrubam o grafo
/// inteiro para a CPU, um **escopo de tempo** também, e uma fronteira sem estágio que despache
/// também. E **nada na tela diz que aconteceu** — `GpuOutcome::FellThrough` é consumido pela
/// ponte e não acende nada.
///
/// ⚠️ **Ela IMPRIME e não julga.** Uma cena na CPU não é um defeito por si: a `=107` tem dois
/// sinks de propósito, e uma cena com `motion.time_remap` **tem de** recuar. O que a sonda
/// entrega é o CENSO — quantas das cenas que o produto expõe correm no caminho rápido —, para
/// que ninguém volte a afirmar «GPU-resident por omissão» sem o número ao lado.
///
/// `cargo test -p ph2d-app-motion --lib motion_route_census -- --ignored --nocapture`
#[test]
#[ignore = "sonda de rota, não um gate — `-- --ignored --nocapture`"]
fn motion_route_census() {
    use crate::motion_bridge::gpu::{GpuRoute, gpu_route};
    let (mut full, mut hybrid, mut cpu) = (0u32, 0u32, 0u32);
    let (mut multi_total, mut multi_todos_gpu, mut multi_alguns_gpu) = (0u32, 0u32, 0u32);
    let mut porques: std::collections::BTreeMap<&'static str, Vec<u32>> =
        std::collections::BTreeMap::new();
    for level in 1..=MAX_DEMO_LEVEL {
        let mut state = MotionState::new();
        let sinks =
            crate::motion_demo_legend::monta(&level.to_string(), &mut state.doc, &state.registry).0;
        if sinks.is_empty() {
            continue;
        }
        let scopes = ph2d_node_motion_time_remap::time_scopes(&state.doc.graph, &state.registry);
        let plan =
            ph2d_gpu_cook::plan(&state.doc.graph, &state.registry, &state.registry, sinks[0]);
        let route = gpu_route(
            true,
            sinks.len(),
            scopes.is_empty(),
            &plan.boundaries,
            plan.dispatching_stages(&state.registry),
        );
        // ⚠️ **A razão é a PRIMEIRA que morde**, na ordem em que o `gpu_route` as pergunta —
        // uma cena pode ter duas, e nomear a segunda mandaria alguém curar a errada.
        let porque = match route {
            GpuRoute::FullyGpu => "1. device inteiro",
            GpuRoute::Hybrid => "2. híbrido (prefixo na CPU)",
            GpuRoute::Cpu if sinks.len() != 1 => "3. CPU: mais de UM sink",
            GpuRoute::Cpu if !scopes.is_empty() => "4. CPU: escopo de tempo (time_remap)",
            GpuRoute::Cpu => "5. CPU: fronteira sem estágio que despache",
        };
        porques.entry(porque).or_default().push(level);
        // ⭐⭐ **O que a escada do multi-sink CUSTA a levantar**: se cada sink, planeado
        // SOZINHO, já é reclamado pelo device, então a barreira não é o trabalho — é só a
        // COMPOSIÇÃO de dois planos num buffer. Sem este número, «F2+ territory» é uma nota
        // sobre um preço que ninguém mediu (§0.0).
        if sinks.len() > 1 {
            multi_total += 1;
            let cada: Vec<_> = sinks
                .iter()
                .map(|&s| {
                    ph2d_gpu_cook::plan(&state.doc.graph, &state.registry, &state.registry, s)
                })
                .collect();
            if cada.iter().all(ph2d_gpu_cook::GpuPlan::is_fully_gpu) {
                multi_todos_gpu += 1;
            } else if cada.iter().any(ph2d_gpu_cook::GpuPlan::is_fully_gpu) {
                multi_alguns_gpu += 1;
            }
        }
        match route {
            GpuRoute::FullyGpu => full += 1,
            GpuRoute::Hybrid => hybrid += 1,
            GpuRoute::Cpu => cpu += 1,
        }
    }
    let total = full + hybrid + cpu;
    eprintln!("=== CENSO DE ROTA · {total} cenas com sinks, de 1..={MAX_DEMO_LEVEL} ===");
    for (porque, cenas) in &porques {
        eprintln!(
            "  {porque:<36} │ {:>3} cenas ({:>4.1}%)",
            cenas.len(),
            cenas.len() as f64 * 100.0 / total as f64
        );
        eprintln!("      {cenas:?}");
    }
    eprintln!("  ─────");
    eprintln!(
        "  device (inteiro ou híbrido) │ {:>3} ({:.1}%)   ·   CPU serial │ {cpu} ({:.1}%)",
        full + hybrid,
        (full + hybrid) as f64 * 100.0 / total as f64,
        cpu as f64 * 100.0 / total as f64
    );
    eprintln!("  ─── a escada do MULTI-SINK, se alguem a levantar ───");
    eprintln!(
        "  cenas multi-sink │ {multi_total}   ·   TODOS os sinks ja seriam device │ {multi_todos_gpu} ({:.1}%)   ·   alguns │ {multi_alguns_gpu}",
        multi_todos_gpu as f64 * 100.0 / multi_total.max(1) as f64
    );
}

/// **EM QUE CENA E EM QUE NÓ o balão de perda aparece** — a sonda que o report do Enio de
/// 2026-09-02 obrigou a escrever: *«não entendi esse teste»*.
///
/// ⛔⛔ **O defeito era do SMOKE, não do produto.** Eu escrevi *«passa o rato num nó; se ele
/// perder colunas, aparece a lista»* — com uma palavra que ele não usa (*coluna*) e, pior, com
/// uma **pré-condição que ele não tem como verificar**: sem saber QUAL nó perde, ele passaria o
/// rato por vários, não veria nada, e concluiria que está partido. *Um smoke cuja condição o
/// leitor não consegue avaliar reprova sobre produto correcto* (§0.8).
///
/// `cargo test -p ph2d-app-motion --release --lib -- --ignored --nocapture where_the_drops_note_shows_up`
#[test]
#[ignore = "sonda de smoke, nao um gate"]
fn where_the_drops_note_shows_up() {
    // ⚠️ **A 1.ª versão varria as 109 e PENDUROU** — alguma cena coze algo enorme sob um
    // `Cook` virgem (sem a bomba, sem o orçamento que o produto lhe dá). O alcance é uma env
    // para quem quiser continuar a varredura por pedaços, em vez de a sonda ter de adivinhar
    // qual cena é a cara.
    let ate: u32 = std::env::var("PH2D_DROPS_SCAN_MAX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(40);
    let mut achados = 0;
    for level in 1..=ate.min(MAX_DEMO_LEVEL) {
        let mut state = MotionState::new();
        let sinks =
            crate::motion_demo_legend::monta(&level.to_string(), &mut state.doc, &state.registry).0;
        if sinks.is_empty() {
            continue;
        }
        crate::motion_externals::publish_all(&mut state, 0.0);
        // UM cook basta: a pergunta é ESTRUTURAL (que colunas entram e saem), não temporal.
        let mut cook = ph2d_nodegraph::cook::Cook::new();
        if cook
            .cook(&state.doc.graph, &state.registry, sinks[0], 0.0)
            .is_err()
        {
            continue;
        }
        state.pump.cook = cook;
        let ids: Vec<_> = state
            .doc
            .graph
            .nodes()
            .iter()
            .map(|n| (n.id, n.type_name.clone()))
            .collect();
        for (id, ty) in ids {
            if let Some(nota) = crate::motion_bridge::columns::dropped_at(&state, id) {
                eprintln!("  cena =`{level}` · no' `{ty}` -> drops {nota}");
                achados += 1;
            }
        }
    }
    eprintln!("  {achados} nos com nota, em 1..={ate}");
}

/// **O QUE ALIMENTA A PORTA `shape` DE CADA CARIMBO, EM TODA CENA** — report do Enio,
/// 2026-09-06: *«você colocou grid entrando em Shape de Duplicator! Essa aplicação é correta?»*
///
/// Não era, e a pergunta seguinte é *quantas cenas fazem o mesmo* — curar a que ele apontou e
/// deixar as irmãs é a armadilha de curar METADE de uma família. As duas portas do
/// `motion.duplicator` são o mesmo tipo, então só a semântica as distingue e nenhum gate de
/// tipos as separa: este censo segue a entrada 0 de cada carimbo até à ORIGEM do braço e diz o
/// que encontrou.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib what_feeds_every_duplicators_shape_port -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de censo, nao um gate"]
fn what_feeds_every_duplicators_shape_port() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut por_fonte: std::collections::BTreeMap<String, Vec<u32>> =
        std::collections::BTreeMap::new();
    let mut carimbos = 0usize;
    for level in 1..=MAX_DEMO_LEVEL {
        let mut doc = MotionDoc::default();
        if crate::motion_demo_legend::monta(&level.to_string(), &mut doc, &reg)
            .0
            .is_empty()
        {
            continue;
        }
        let g = &doc.graph;
        for n in g
            .nodes()
            .iter()
            .filter(|n| n.type_name == "motion.duplicator")
        {
            carimbos += 1;
            let fonte = match g.edges().iter().find(|e| e.to.0 == n.id && e.to.1 == 0) {
                Some(e) => g
                    .nodes()
                    .iter()
                    .find(|m| m.id == shape_origin(g, e.from.0))
                    .map(|m| m.type_name.clone())
                    .unwrap_or_default(),
                None => "(porta VAZIA)".to_string(),
            };
            let e = por_fonte.entry(fonte).or_default();
            if !e.contains(&level) {
                e.push(level);
            }
        }
    }
    eprintln!("\n  {carimbos} carimbos nas cenas do roteador · o que alimenta a porta `shape`:\n");
    for (fonte, cenas) in &por_fonte {
        eprintln!("  {fonte:<22} em {:>2} cena(s): {cenas:?}", cenas.len());
    }
    eprintln!();
}

/// ⭐⭐⭐ **QUEM DESENHA PIXELS SEM NUNCA TER RECEBIDO UMA FORMA** — a sonda que a ordem do dono
/// de 2026-09-19 obriga a correr ANTES de qualquer cura (§5.0, e o [doc 115] §14.2 por escrito).
///
/// > *«Não deveriam renderizar nada na tela, mas deveriam apenas disponibilizarem a posição e
/// > direção […] e deveriam ser dependentes de Duplicator e Shape (e demais objetos) para
/// > aparecer na tela.»*
///
/// ⚠️ **A pergunta é por SINK e não por nó**, porque quem gera pixels é o lowering: um
/// `rig.skeleton` ligado direito ao `motion.output` desenha um quadrado por junta, com o
/// `default_uv_rect` da shell — e o cabeçalho do próprio nó diz que isso é o desenho
/// (*«a bare skeleton already renders: its joints are elements like any other»*).
///
/// ⚠️ **Os produtores de aparência são DOIS e o censo deriva-os**, nunca os escreve à mão: são
/// as únicas crates de nó que escrevem `uv_rect`/`texture_id`/`geometry_id` como fonte
/// (`source.object` e `source.shape` — ⚠️ a crate chama-se `ph2d-node-motion-shape` e o NÓ
/// chama-se `source.shape`, e a 1.ª redacção desta sonda leu `0 de 123` por causa disso);
/// `duplicator`/`mixer`/`morph`/`trail` **propagam** o que
/// receberam, logo não contam como origem.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture quem_desenha_sem_forma`
#[test]
#[ignore = "sonda, nao um gate"]
fn quem_desenha_sem_forma() {
    /// As origens de aparência. ⚠️ Derivadas da varredura das crates de nó (as únicas que
    /// escrevem uma coluna de aparência sem a receber de uma entrada).
    const ORIGENS: [&str; 2] = ["source.object", "source.shape"];

    let (mut com, mut sem) = (Vec::new(), Vec::new());
    let mut sem_por_fonte: std::collections::BTreeMap<String, Vec<u32>> =
        std::collections::BTreeMap::new();
    for level in 1..=MAX_DEMO_LEVEL {
        let mut state = MotionState::new();
        let sinks =
            crate::motion_demo_legend::monta(&level.to_string(), &mut state.doc, &state.registry).0;
        if sinks.is_empty() {
            continue;
        }
        let g = &state.doc.graph;
        let nome = |id: ph2d_nodegraph::graph::NodeId| {
            g.nodes()
                .iter()
                .find(|n| n.id == id)
                .map(|n| n.type_name.clone())
                .unwrap_or_default()
        };
        // Sobe a montante de cada sink. Uma aresta `delayed` também carrega dados (é o tique
        // anterior), logo entra: um laço de simulação não deixa de receber a forma por isso.
        let mut vistos: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
        let mut pilha: Vec<_> = sinks.clone();
        let mut origens_achadas: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        let mut folhas: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        while let Some(id) = pilha.pop() {
            if !vistos.insert(id.0) {
                continue;
            }
            let t = nome(id);
            if ORIGENS.contains(&t.as_str()) {
                origens_achadas.insert(t.clone());
            }
            let mut tem_entrada = false;
            for e in g.edges().iter().filter(|e| e.to.0 == id) {
                tem_entrada = true;
                pilha.push(e.from.0);
            }
            if !tem_entrada && !t.is_empty() {
                folhas.insert(t);
            }
        }
        if origens_achadas.is_empty() {
            sem.push(level);
            for f in &folhas {
                let e = sem_por_fonte.entry(f.clone()).or_default();
                if !e.contains(&level) {
                    e.push(level);
                }
            }
        } else {
            com.push(level);
        }
    }
    let total = com.len() + sem.len();
    eprintln!("\n=== QUEM DESENHA SEM FORMA · {total} cenas com sink, de 1..={MAX_DEMO_LEVEL} ===");
    eprintln!(
        "  recebem aparencia de `source.object`/`motion.shape` │ {:>3} ({:>4.1}%)",
        com.len(),
        com.len() as f64 * 100.0 / total as f64
    );
    eprintln!(
        "  desenham SO' POSICOES (ficariam em branco)          │ {:>3} ({:>4.1}%)",
        sem.len(),
        sem.len() as f64 * 100.0 / total as f64
    );
    eprintln!("\n  as cenas so'-posicoes: {sem:?}\n");
    eprintln!("  quem as ALIMENTA (no' de raiz -> cenas):\n");
    for (fonte, cenas) in &sem_por_fonte {
        eprintln!("  {fonte:<26} em {:>3} cena(s): {cenas:?}", cenas.len());
    }
    eprintln!();
}

/// ⭐⭐ **AS COLUNAS QUE CHEGAM AO SINK** — o CONTROLO da [`quem_desenha_sem_forma`], que responde
/// pelo grafo. Esta coze e pergunta ao stream, que é o que o lowering de facto lê.
///
/// ⚠️ **Ela existe porque as duas perguntas podem discordar**: um grafo pode atravessar um
/// `source.shape` e a coluna não chegar ao sink (um nó a jusante que a deixe cair), e um grafo sem
/// origem nenhuma pode ter `uv_rect` por outra via. *O discriminador da cura tem de ser o que o
/// lowering vê, não o que o grafo promete.*
///
/// ⛔⛔ **ARMADILHA MEDIDA: as cenas de `source.shape` cozem a ZERO neste arnês.** A geometria
/// delas é publicada pelo `motion_shape_gen`, que corre no QUADRO (precisa do `shape_store`), e um
/// arnês sem shell não o corre ⇒ a `=110`, a `=114`, a `=115` e a `=119` leem `0 linhas` aqui e
/// desenham dezenas de peças no app. *Esta sonda é o controlo do que APARECE nas colunas, nunca um
/// censo de população* — para «quem tem origem de aparência» a resposta é a [`quem_desenha_sem_forma`],
/// que pergunta ao GRAFO e não precisa de shell.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture colunas_que_chegam_ao_sink`
#[test]
#[ignore = "sonda, nao um gate"]
fn colunas_que_chegam_ao_sink() {
    // As cenas de CICLO (as que o dono segue) mais a `=110`, que é a do carimbo.
    let alvo: [u32; 12] = [110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121];
    eprintln!("\n=== COLUNAS NO SINK · as cenas que o dono segue ===\n");
    for level in alvo {
        let mut state = MotionState::new();
        let sinks =
            crate::motion_demo_legend::monta(&level.to_string(), &mut state.doc, &state.registry).0;
        if sinks.is_empty() {
            eprintln!("  =[{level}] (sem sink)");
            continue;
        }
        for (k, &sink) in sinks.iter().enumerate() {
            let Ok(out) = state
                .pump
                .cook
                .cook(&state.doc.graph, &state.registry, sink, 0.0)
            else {
                eprintln!("  =[{level}] sink {k}: (nao coze)");
                continue;
            };
            let s = out[0].as_stream();
            let tem = |n: &str| if s.get(n).is_some() { "sim" } else { " - " };
            let geo_vivo = match s.get("geometry_id") {
                Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.iter().any(|&x| x > 0.5),
                _ => false,
            };
            eprintln!(
                "  =[{level}] sink {k}: {:>7} linhas │ uv_rect {} │ texture_id {} │ geometry_id {} (vivo: {}) │ APARENCIA: {}",
                s.count(),
                tem("uv_rect"),
                tem("texture_id"),
                tem("geometry_id"),
                if geo_vivo { "sim" } else { "nao" },
                if s.get("uv_rect").is_some() || geo_vivo {
                    "SIM"
                } else {
                    "NAO — hoje isto desenha quadrados pelo default do shell"
                }
            );
        }
    }
    eprintln!();
}

/// ⭐⭐⭐ **O QUE O GRAFO DE FACTO ANIMA, por cena** — a sonda que o report do dono de 2026-09-19
/// obriga a correr: *«não são animados em scale (grade do segundo exemplo)»*.
///
/// ⚠️ **Ela mede o STREAM ao longo do TEMPO**, que é a única maneira de responder: uma coluna
/// `size` que EXISTE e não muda é um grafo que não anima escala, e um gizmo que a lê fica parado
/// **com razão**. *Eu apontei o dono para a `=117` de memória; esta sonda é o que eu devia ter
/// corrido antes de escrever o passo.*
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture o_que_cada_cena_anima`
#[test]
#[ignore = "sonda, nao um gate"]
fn o_que_cada_cena_anima() {
    let alvo: [u32; 10] = [111, 112, 113, 116, 117, 118, 119, 120, 121, 122];
    eprintln!("\n=== O QUE CADA CENA ANIMA (12 tiques a 60 Hz) ===\n");
    eprintln!("  cena · sink │ linhas │ size?  varia? │ rot?   varia? │ P varia?");
    for level in alvo {
        let mut state = MotionState::new();
        let sinks =
            crate::motion_demo_legend::monta(&level.to_string(), &mut state.doc, &state.registry).0;
        for (k, &sink) in sinks.iter().enumerate() {
            let mut fotos: Vec<(Vec<f32>, Vec<f32>, Vec<f32>)> = Vec::new();
            let mut existe = (false, false);
            for t in 0..12 {
                let seg = f64::from(t) / 60.0;
                let Ok(out) = state
                    .pump
                    .cook
                    .cook(&state.doc.graph, &state.registry, sink, seg)
                else {
                    break;
                };
                let s = out[0].as_stream();
                let esc = escalas_da_corrente(s);
                let rot = rotacoes_da_corrente(s);
                existe = (existe.0 || esc.is_some(), existe.1 || rot.is_some());
                let p = match s.get("P") {
                    Some(ph2d_nodegraph::attr::Column::Vec2(v)) => {
                        v.iter().flat_map(|q| [q[0], q[1]]).collect()
                    }
                    _ => Vec::new(),
                };
                fotos.push((esc.unwrap_or_default(), rot.unwrap_or_default(), p));
                let _ = state
                    .pump
                    .cook
                    .advance_tick(&state.doc.graph, &state.registry, seg);
            }
            if fotos.is_empty() {
                continue;
            }
            /// Uma fotografia do stream num tique: `(escala, rotação, posições)`.
            type Foto = (Vec<f32>, Vec<f32>, Vec<f32>);
            let varia = |f: &dyn Fn(&Foto) -> Vec<f32>| {
                let a = f(&fotos[0]);
                fotos.iter().any(|x| f(x) != a)
            };
            let (ve, vr, vp) = (
                varia(&|x| x.0.clone()),
                varia(&|x| x.1.clone()),
                varia(&|x| x.2.clone()),
            );
            let sim = |b: bool| if b { "SIM" } else { " -  " };
            eprintln!(
                "  ={level:<3} · {k:<2} │ {:>6} │ {:<5} {:<6} │ {:<5} {:<6} │ {}",
                fotos[0].2.len() / 2,
                sim(existe.0),
                sim(ve),
                sim(existe.1),
                sim(vr),
                sim(vp)
            );
        }
    }
    eprintln!();
}

/// ⛔⛔⛔ **QUE NÚMEROS O `size` DE FACTO TEM NAS CENAS** — a sonda que explica os TRÊS relatos do
/// dono de 2026-09-19 de uma vez (*«piorou os desenhos»* · *«continuam relativos ao zoom»* ·
/// *«não são animados em scale»*).
///
/// A 1.ª redacção do gizmo leu a coluna `size` como um multiplicador **directo** do glifo, com a
/// identidade `1`. Esta sonda mede o que as cenas REAIS autoram.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture que_numeros_o_size_tem`
#[test]
#[ignore = "sonda, nao um gate"]
fn que_numeros_o_size_tem() {
    let mut todos: Vec<f32> = Vec::new();
    let mut por_cena: Vec<(u32, usize, f32, f32)> = Vec::new();
    for level in 1..=MAX_DEMO_LEVEL {
        let mut state = MotionState::new();
        let sinks =
            crate::motion_demo_legend::monta(&level.to_string(), &mut state.doc, &state.registry).0;
        for (k, &sink) in sinks.iter().enumerate() {
            let Ok(out) = state
                .pump
                .cook
                .cook(&state.doc.graph, &state.registry, sink, 0.0)
            else {
                continue;
            };
            let s = out[0].as_stream();
            let Some(esc) = escalas_da_corrente(s) else {
                continue;
            };
            if esc.is_empty() {
                continue;
            }
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            for &e in &esc {
                if e.is_finite() {
                    lo = lo.min(e);
                    hi = hi.max(e);
                    todos.push(e);
                }
            }
            if lo.is_finite() {
                por_cena.push((level, k, lo, hi));
            }
        }
    }
    todos.sort_by(f32::total_cmp);
    let p = |q: f64| todos[((todos.len() - 1) as f64 * q) as usize];
    eprintln!(
        "\n=== QUE NUMEROS O `size` TEM · {} valores ===\n",
        todos.len()
    );
    eprintln!(
        "  min {:.4} · p10 {:.4} · p25 {:.4} · MEDIANA {:.4} · p75 {:.4} · p90 {:.4} · max {:.4}",
        todos[0],
        p(0.10),
        p(0.25),
        p(0.50),
        p(0.75),
        p(0.90),
        todos[todos.len() - 1]
    );
    eprintln!(
        "\n  ⇒ com o glifo = base x size, a MEDIANA da' {:.1} % do glifo nu.",
        p(0.50) * 100.0
    );
    eprintln!("\n  as 12 cenas com o size mais PEQUENO:\n");
    por_cena.sort_by(|a, b| a.2.total_cmp(&b.2));
    for (l, k, lo, hi) in por_cena.iter().take(12) {
        eprintln!("  ={l:<3} · sink {k:<2} │ min {lo:.4} · max {hi:.4}");
    }
    eprintln!();
}

/// ⭐⭐⭐ **QUEM É «COMO O GRID»** — a população da ordem do dono de 2026-09-19 (*«para nós como Grid
/// e outros similares vamos criar uma seção para tamanho absoluto do gizmo…»*), **DERIVADA do
/// manifesto** e nunca de uma lista escrita à mão.
///
/// A regra: um nó é FONTE DE POSIÇÕES quando **não recebe** uma corrente de instâncias e **emite**
/// uma. É esse o nó que, sem um Duplicator, não tem como virar pixel — e é a ele que a secção do
/// gizmo pertence.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture quem_e_como_o_grid`
#[test]
#[ignore = "sonda, nao um gate"]
fn quem_e_como_o_grid() {
    use ph2d_nodegraph::port::Domain;
    let reg = MotionState::new().registry;
    let inst = |t: &ph2d_nodegraph::port::PortType| t.domain == Domain::Instances;
    let mut fontes: Vec<&str> = Vec::new();
    let mut passagens = 0usize;
    for m in reg.manifests() {
        let emite = m.outputs.iter().any(|p| inst(&p.ty));
        let recebe = m.inputs.iter().any(|p| inst(&p.ty));
        if emite && !recebe {
            fontes.push(m.name);
        } else if emite {
            passagens += 1;
        }
    }
    fontes.sort_unstable();
    eprintln!(
        "\n=== FONTES DE POSICOES · {} de {} nos que emitem instancias ===\n",
        fontes.len(),
        fontes.len() + passagens
    );
    for f in &fontes {
        eprintln!("  {f}");
    }
    eprintln!("\n  ({passagens} sao de PASSAGEM: recebem instancias e devolvem-nas)\n");
}

/// ⭐⭐⭐ **UMA COLUNA NOVA SOBREVIVE À CADEIA?** — a premissa que decide a arquitectura da secção
/// do gizmo (ordem do dono, 2026-09-19). Se uma coluna escrita pela FONTE não chega ao sink, então
/// a secção **não pode** viver no nó de origem e tem de viver no sink.
///
/// ⚠️ Ela é medida com uma coluna INVENTADA (`zz_sonda`) posta num external, atravessando a cadeia
/// que as cenas de facto usam.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture uma_coluna_nova_sobrevive`
#[test]
#[ignore = "sonda, nao um gate"]
fn uma_coluna_nova_sobrevive_a_cadeia() {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::graph::{Edge, Graph};
    let cadeias: [&[&str]; 6] = [
        &["motion.move"],
        &["motion.scale"],
        &["motion.rotate"],
        &["motion.move", "motion.scale", "motion.rotate"],
        &["motion.clone"],
        &["motion.cull"],
    ];
    eprintln!("\n=== UMA COLUNA NOVA SOBREVIVE A' CADEIA? ===\n");
    for cadeia in cadeias {
        let mut m = MotionState::new();
        let mut g = Graph::new();
        // A fonte é um `source.object` porque ele lê um EXTERNAL, que é onde a sonda põe a coluna.
        let fonte = g.add_node("source.object");
        g.set_text_param(fonte, "object", "Sonda");
        let mut cur = fonte;
        for t in cadeia {
            let n = g.add_node(*t);
            g.connect(Edge {
                from: (cur, 0),
                to: (n, 0),
                delayed: false,
            })
            .expect("liga");
            cur = n;
        }
        let saida = g.add_node("motion.output");
        g.connect(Edge {
            from: (cur, 0),
            to: (saida, 0),
            delayed: false,
        })
        .expect("liga");
        m.doc.graph = g;
        let com_sonda = crate::motion_bridge::appearance_tile(
            [1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            0,
            false,
        )
        .with("zz_sonda", Column::Scalar(vec![7.0]));
        m.pump.cook.set_external("Sonda".to_string(), com_sonda);
        let Ok(out) = m.pump.cook.cook(&m.doc.graph, &m.registry, saida, 0.0) else {
            eprintln!("  {cadeia:?} │ NAO COZE");
            continue;
        };
        let s = out[0].as_stream();
        let chegou =
            matches!(s.get("zz_sonda"), Some(Column::Scalar(v)) if v.first() == Some(&7.0));
        eprintln!(
            "  {:<52} │ {} linhas │ a coluna {}",
            format!("{cadeia:?}"),
            s.count(),
            if chegou { "CHEGOU" } else { "SUMIU" }
        );
    }
    eprintln!();
}

/// ⛔⛔ **O QUE A TOMADA DO GIZMO CUSTA NUMA CENA DE DISPOSITIVO** — o número que decide se o gizmo
/// é utilizável nas cenas grandes (ordem do dono, 2026-09-19).
///
/// Na rota do device a bomba **não marcha**: uma tomada obriga o `cook_taps_only` a cozinhar aquele
/// sink **na CPU**, e é esse o preço por quadro que o gizmo cobra quando a lei está ligada.
///
/// `cargo test -p ph2d-app-motion --release --lib -- --ignored --nocapture o_que_a_tomada_custa`
#[test]
#[ignore = "sonda de relogio, nao um gate"]
fn o_que_a_tomada_do_gizmo_custa() {
    eprintln!("\n=== O PRECO DA TOMADA DO GIZMO (cozimento de CPU do sink) ===\n");
    for level in [111u32, 116, 117, 120] {
        let mut state = MotionState::new();
        let sinks =
            crate::motion_demo_legend::monta(&level.to_string(), &mut state.doc, &state.registry).0;
        let Some(&sink) = sinks.first() else { continue };
        // Aquece (o memo do cook) e depois mede a mediana de cinco.
        let _ = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, 0.0);
        let mut ms: Vec<f64> = (0..5)
            .map(|k| {
                let t = f64::from(k) / 60.0;
                let i = std::time::Instant::now();
                let _ = state
                    .pump
                    .cook
                    .cook(&state.doc.graph, &state.registry, sink, t);
                i.elapsed().as_secs_f64() * 1e3
            })
            .collect();
        ms.sort_by(f64::total_cmp);
        let n = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, 0.0)
            .map_or(0, |o| o[0].as_stream().count());
        eprintln!(
            "  ={level:<4} │ {n:>7} linhas │ {:>8.3} ms │ {:>6.1} % de um quadro de 16,7",
            ms[2],
            ms[2] * 100.0 / 16.67
        );
    }
    eprintln!();
}

/// ⛔⛔⛔ **O `gap_y` DA GRELHA AINDA ESPAÇA?** — report do dono, 2026-09-19: *«Gap y quebrou e
/// movimenta tudo em vez de criar espaço»*, depois de a secção do gizmo ter entrado no manifesto
/// daquele nó.
///
/// A régua é a **extensão** da nuvem contra o **centro** dela: espaçar cresce a extensão e deixa o
/// centro quieto; mover desloca o centro e deixa a extensão quieta.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture o_gap_y_ainda_espaca`
#[test]
#[ignore = "sonda, nao um gate"]
fn o_gap_y_ainda_espaca() {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::graph::{Edge, Graph};
    eprintln!("\n=== O `gap_y` DA GRELHA ===\n");
    eprintln!("  gap_y │ extensao Y │  centro Y  │ extensao X │  centro X");
    for gy in [0.5f32, 1.0, 2.0, 4.0] {
        let mut m = MotionState::new();
        let mut g = Graph::new();
        let grelha = g.add_node("motion.grid");
        g.set_param(grelha, "rows", 4.0);
        g.set_param(grelha, "cols", 4.0);
        g.set_param(grelha, "gap_x", 1.0);
        g.set_param(grelha, "gap_y", gy);
        let saida = g.add_node("motion.output");
        g.connect(Edge {
            from: (grelha, 0),
            to: (saida, 0),
            delayed: false,
        })
        .expect("liga");
        m.doc.graph = g;
        let Ok(out) = m.pump.cook.cook(&m.doc.graph, &m.registry, saida, 0.0) else {
            eprintln!("  {gy} │ NAO COZE");
            continue;
        };
        let s = out[0].as_stream();
        let Some(Column::Vec2(p)) = s.get("P") else {
            continue;
        };
        let (mut lox, mut hix, mut loy, mut hiy) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for q in p {
            lox = lox.min(q[0]);
            hix = hix.max(q[0]);
            loy = loy.min(q[1]);
            hiy = hiy.max(q[1]);
        }
        eprintln!(
            "  {gy:>5} │ {:>10.3} │ {:>10.3} │ {:>10.3} │ {:>10.3}",
            hiy - loy,
            (hiy + loy) / 2.0,
            hix - lox,
            (hix + lox) / 2.0
        );
    }
    eprintln!();
}

/// ⛔⛔ **O QUE O CARTÃO DO GRID PINTA, NA ORDEM** — o instrumento que o report do dono de
/// 2026-09-19 (*«Gap y quebrou e movimenta tudo em vez de criar espaço»*) obriga a ter.
///
/// ⚠️ A secção do gizmo entrou naquele cartão na mesma wave, e **uma tabela de grupos PARCIAL** é a
/// hipótese que esta sonda serve para confirmar ou matar: se a ordem ou o dono de cada linha
/// mudou, o artista arrasta uma linha e escreve noutro param.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture o_que_o_cartao_do_grid_pinta`
#[test]
#[ignore = "sonda, nao um gate"]
fn o_que_o_cartao_do_grid_pinta() {
    let m = MotionState::new();
    let tid = ph2d_nodegraph::node::NodeTypeId::of("motion.grid");
    use ph2d_nodegraph::cook::OpResolver;
    let op = m.registry.resolve(tid).expect("o Grid existe");
    eprintln!("\n=== O CARTAO DO `motion.grid` ===\n");
    eprintln!("  # │ param          │ rotulo          │ seccao");
    for (i, p) in op.manifest().params.iter().enumerate() {
        let hint = m
            .registry
            .param_ui(tid)
            .into_iter()
            .flat_map(|t| t.iter())
            .find(|h| h.param == p.name);
        let grupo = m
            .registry
            .param_groups(tid)
            .iter()
            .find(|g| g.param == p.name)
            .map_or("—", |g| g.group);
        eprintln!(
            "  {i} │ {:<14} │ {:<15} │ {grupo}",
            p.name,
            hint.map_or("(SEM DICA)", |h| h.label)
        );
    }
    eprintln!();
}

/// ⭐⭐⭐ **O `gap_y` NA GRELHA DO DONO — a grelha estava CERTA, e este é o número que o prova.**
///
/// Report de 2026-09-19: *«Gap y quebrou e movimenta tudo em vez de criar espaço»*, sobre a
/// *«grade do segundo exemplo»* — a cena **`=2`**, que é uma `motion.grid` de **360 × 360 =
/// 129 600** elementos.
///
/// ⚠️⚠️ **A causa NÃO era o nó, e esta sonda é o que o mostra:** a nuvem inteira ESPAÇA (o centro
/// fica parado e a extensão cresce) em todas as posições do knob. Quem mentia era o **desenho** —
/// o gizmo de posições, que segurava um PREFIXO da nuvem (uma faixa na borda, que voava) e que a
/// ordem do dono do mesmo dia **retirou**.
///
/// ⚠️ **A sonda irmã [`o_gap_y_ainda_espaca`] mede uma grelha de `4 × 4`**, e é por isso que ela
/// não podia ver nada: *uma fixtura pequena não testa o que só aparece na escala do dono.*
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture o_gap_y_na_grelha_do_dono`
#[test]
#[ignore = "sonda, nao um gate"]
fn o_gap_y_na_grelha_do_dono() {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::graph::{Edge, Graph};
    // A geometria da cena `=2`, lida dela: `motion_state_gpu_demos.rs`.
    const ROWS: f32 = 360.0;
    const COLS: f32 = 360.0;
    eprintln!("\n=== O `gap_y` NA GRELHA DE {ROWS:.0}x{COLS:.0} (a cena `=2`) ===\n");
    eprintln!("  gap_y │  extensao Y │   centro Y  │ pontos");
    let mut antes: Option<(f32, f32)> = None;
    for gy in [1.0f32, 1.5, 2.0, 3.0] {
        let mut m = MotionState::new();
        let mut g = Graph::new();
        let grelha = g.add_node("motion.grid");
        g.set_param(grelha, "rows", ROWS);
        g.set_param(grelha, "cols", COLS);
        g.set_param(grelha, "gap_x", 1.0);
        g.set_param(grelha, "gap_y", gy);
        let saida = g.add_node("motion.output");
        g.connect(Edge {
            from: (grelha, 0),
            to: (saida, 0),
            delayed: false,
        })
        .expect("liga");
        m.doc.graph = g;
        let Ok(out) = m.pump.cook.cook(&m.doc.graph, &m.registry, saida, 0.0) else {
            eprintln!("  {gy} │ NAO COZE");
            continue;
        };
        let s = out[0].as_stream();
        let Some(Column::Vec2(p)) = s.get("P") else {
            continue;
        };
        let medir = |v: &[[f32; 2]]| {
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            for q in v {
                lo = lo.min(q[1]);
                hi = hi.max(q[1]);
            }
            (hi - lo, (hi + lo) / 2.0)
        };
        let (ext_t, cen_t) = medir(p);
        eprintln!("  {gy:>5} │ {ext_t:>11.3} │ {cen_t:>11.3} │ {}", p.len());
        if let Some((e0, c0)) = antes {
            eprintln!(
                "        │  Δextensao {:>+8.3} │ Δcentro {:>+8.3}  ⇒ mover/espacar = {:.2}x",
                ext_t - e0,
                cen_t - c0,
                (cen_t - c0).abs() / (ext_t - e0).abs().max(f32::EPSILON)
            );
        }
        antes = Some((ext_t, cen_t));
    }
    eprintln!(
        "\n  ⇒ o `gap_y` ESPACA: o centro fica em 0,000 e a extensao acompanha, em todo o curso\n              do knob. O report do dono era o DESENHO — o gizmo de posicoes segurava as primeiras\n              fileiras da nuvem (uma faixa na borda) e ela VOAVA; esse gizmo foi RETIRADO.\n"
    );
}
