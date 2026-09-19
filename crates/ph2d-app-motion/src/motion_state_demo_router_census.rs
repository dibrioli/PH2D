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
