//! ⭐⭐ **A AUDITORIA DO GRUPO DO FOCO — os CAMPOS** (ciclo 4, passo 2 — doc 103 §5).
//!
//! *«Nem todos ao mesmo tempo»*: este grupo é o que decide **quem** um animador ou um
//! deformador afecta, e **quanto**. Um `motion.falloff` sem campo é um interruptor; com campo é
//! um pincel.
//!
//! ⚠️ A régua é a mesma dos ciclos 1–3 e vive numa porta só
//! ([`crate::motion_ciclo_probe`]) — o que muda entre ciclos é a lista de nós, nunca o
//! instrumento.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture audit_the_field_group
//! ```

/// Os sete do ciclo 4 (doc 103 §5).
pub(crate) const GRUPO: [&str; 7] = [
    "motion.falloff",
    "field.box",
    "field.radial_sweep",
    "field.index_range",
    "field.remap",
    "field.combine",
    "field.shape",
];

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn audit_the_field_group() {
    crate::motion_ciclo_probe::retrato(&GRUPO);
}

/// **OS PARAMS DE CADA NÓ DO GRUPO, um a um** — o que a auditoria compara contra as
/// referências. ⚠️ Sem esta lista, «falta X» é um palpite.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture what_each_field_offers
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_each_field_offers() {
    crate::motion_ciclo_probe::params_de(&GRUPO);
}

/// ⭐⭐ **AS ROWS QUE O CARTÃO DE FACTO PINTA** — ver a porta.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture what_the_field_card_shows
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_the_field_card_shows() {
    crate::motion_ciclo_probe::cartao(&GRUPO);
}

/// ⭐⭐ **O VOCABULÁRIO do grupo** — dois campos que guardam a mesma pergunta chamam-lhe o mesmo
/// nome? É o achado §2.3 do ciclo 3 (*«seis vocabulários para onde é o centro»*) virado régua.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture the_field_vocabulary
/// ```
/// Os NOMES que os cartões pintam — o que o tutorial tem de escrever.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_field_card_names() {
    crate::motion_ciclo_probe::nomes(&GRUPO);
}

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_field_vocabulary() {
    crate::motion_ciclo_probe::vocabulario(&GRUPO);
}

/// ⭐⭐⭐ **DOIS IRMÃOS QUE PARTILHAM UM PARAM PÕEM-NO NO MESMO SÍTIO** (ciclo 4, W2).
///
/// A população é **derivada**: os nós do grupo que declaram secções com o **mesmo
/// vocabulário** — o `field.box` e o `field.radial_sweep` falam `Placement`/`Falloff`, o
/// `field.remap` fala `Range`/`Output` e responde a outras perguntas, logo não entra na
/// comparação. ⚠️ Sem esse recorte o gate acusaria o `invert` (que no `remap` vive em `Range`)
/// e mandaria alinhar duas coisas que não são a mesma.
///
/// ⚠️⚠️ **«O mesmo sítio» inclui FICAR SOLTO, e a 1.ª redacção não o dizia.** Ela comparava só
/// os params que os dois **agrupavam** — e a mutação que devolvia o `soft` do `field.box` para
/// fora de toda secção **SOBREVIVEU**, porque um param solto simplesmente saía da população.
/// *Um censo que só olha o que foi declarado é cego a uma omissão*, e a omissão era exactamente
/// a divergência que esta wave veio curar. Hoje o «sítio» de um param partilhado é o título da
/// secção **ou `(solto)`**, e os dois têm de bater.
///
/// ⛔ **Ele já se pagou:** a 1.ª redacção da tabela do `field.box` deixava o `soft` solto, e o
/// irmão põe-no em `Falloff` — *quando o objectivo é alinhar dois irmãos, a autoridade é o
/// irmão*.
#[test]
fn the_two_spatial_boxes_group_a_shared_param_the_same_way() {
    use std::collections::BTreeMap;
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry");
    // nó -> (param -> sítio), só para quem fala o vocabulário ESPACIAL.
    let mut casas: Vec<(&str, BTreeMap<&str, &str>)> = Vec::new();
    for nome in GRUPO {
        let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
        let grupos = reg.param_groups(tid);
        if !grupos.iter().any(|g| g.group == "Placement") {
            continue;
        }
        let man = {
            use ph2d_nodegraph::cook::OpResolver;
            let Some(op) = reg.resolve(tid) else { continue };
            op.manifest()
        };
        let mut casa: BTreeMap<&str, &str> = BTreeMap::new();
        for spec in man.params {
            let onde = grupos
                .iter()
                .find(|g| g.param == spec.name)
                .map_or("(solto)", |g| g.group);
            casa.insert(spec.name, onde);
        }
        casas.push((nome, casa));
    }
    assert!(
        casas.len() >= 2,
        "so' {} no(s) com o vocabulario espacial -- o gate compara IRMAOS",
        casas.len()
    );
    let (n0, c0) = &casas[0];
    let mut partilhados = 0usize;
    for (n, c) in &casas[1..] {
        for (param, onde) in c {
            let Some(onde0) = c0.get(param) else { continue };
            partilhados += 1;
            assert_eq!(
                onde, onde0,
                "`{param}` vive em `{onde}` no {n} e em `{onde0}` no {n0} -- dois irmaos com o \
                 mesmo vocabulario te^m de o arrumar igual (e `(solto)` e' um sitio)"
            );
        }
    }
    // Piso contra o vácuo: dois irmãos sem param nenhum em comum não provariam nada.
    assert!(
        partilhados >= 4,
        "so' {partilhados} param(s) partilhado(s) -- a comparacao ficou vazia"
    );
}

/// ⭐⭐⭐ **O PREÇO DO GRUPO** (ciclo 4, passo 5 — doc 103 §1).
///
/// ⚠️ **Um campo é `Pure` e escreve a coluna `falloff`** — sozinho na cadeia ele não move um
/// pixel. A tabela mede-o **na cadeia do produto**, `grid → <campo> → output`, que é onde o
/// planeador decide se a coisa fica no dispositivo.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture measure_the_field_group
/// ```
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn measure_the_field_group() {
    let lado: f32 = std::env::var("PH2D_LADO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(320.0);
    crate::motion_ciclo_probe::tabela(&GRUPO, lado);
}

/// ⭐ **O DESPERTAR TEM DE ACORDAR** também neste grupo — ver a porta partilhada.
#[test]
fn waking_a_field_takes_it_off_the_identity() {
    let mudos = crate::motion_ciclo_probe::quem_o_despertar_nao_acorda(&GRUPO);
    assert!(
        mudos.is_empty(),
        "o despertar nao mexeu nestes: {mudos:?} -- ou o widget deles nao esta' na lista de \
         controlos continuos, ou a fraccao poe o no' de volta na identidade"
    );
}

/// ⭐⭐ **A CENA `=112` CONSTRÓI, É ALCANÇÁVEL E FICA NO DISPOSITIVO** (ciclo 4, passo 7).
///
/// ⚠️ **As três perguntas são independentes**, e a casa já pagou por confundi-las: um grafo que
/// coze não é um grafo que se VÊ (a 1.ª redacção da cena `=111` punha o pano a `300` unidades,
/// fora do alcance do zoom, e o report foi *«funciona nos nós mas não aparece no canvas»*), e um
/// grafo que se vê não é um grafo que o dispositivo reivindica.
#[test]
fn the_focus_scene_builds_and_stays_on_the_device() {
    let mut m = crate::motion_state::MotionState::new();
    let (sinks, _) = crate::motion_demo_legend::monta("112", &mut m.doc, &m.registry);
    let sink = *sinks.first().expect("a cena 112 tem um sink");

    // 1. Ela COZE, e produz as peças que promete.
    let out = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, sink, 0.0)
        .expect("coze");
    let st = out[0].as_stream();
    assert_eq!(st.count(), 6400, "80 x 80 pecas");

    // 2. O campo MORDE — nem todas as peças têm o mesmo tamanho. ⚠️ É a régua da premissa do
    //    ciclo (*«nem todos ao mesmo tempo»*): sem ela a cena podia crescer o pano inteiro.
    let Some(ph2d_nodegraph::attr::Column::Vec2(size)) = st.get("size") else {
        panic!("a cena escreve `size` -- o `motion.scale` e' quem le^ o campo");
    };
    let (mut menor, mut maior) = (f32::MAX, f32::MIN);
    for s in size {
        menor = menor.min(s[0]);
        maior = maior.max(s[0]);
    }
    assert!(
        maior > menor * 1.5,
        "a mancha tem de se ver: menor {menor} maior {maior} -- se forem iguais, o campo nao \
         esta' a ser lido"
    );

    // 3. E a cadeia inteira é REIVINDICADA pelo dispositivo.
    assert!(
        ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, sink).is_fully_gpu(),
        "a cena do ciclo 4 tem de correr no dispositivo -- os sete campos menos um la' estao"
    );
}

/// A FONTE do tutorial deste ciclo — lida para que os nomes do gate e os do texto não possam
/// divergir em silêncio.
const TUTORIAL: &str = include_str!("../../../docs/Motion Nodes/tutoriais/src/04_campos.html");

/// ⭐⭐⭐ **CADA PASSO DO TUTORIAL É POSSÍVEL NO APP** (ciclo 4, passo 7 — doc 103 §1).
///
/// ⛔⛔ **Um passo que manda clicar numa linha AFIRMA que ela está na lista**, e a casa já pagou
/// por escrever um passo impossível ([memória](../../project-memory/feedback_a_smoke_step_that_names_a_panel_row_must_prove_the_row_is_in_the_list.md)).
/// Este gate lê a **fonte do tutorial** e verifica, contra a cena `=112` de verdade, que cada
/// nome que ele manda o dono procurar existe: o **título do cartão** e a **linha** dentro dele.
///
/// ⚠️ **E ele apanhou-me a mim antes do dono:** a 1.ª redacção dizia *«o cartão `Falloff`, o
/// SEGUNDO da fila»* — ele é o **terceiro** (`grid · scale · falloff · remap · scale · output`),
/// e o `Field Remap` que eu chamava de terceiro é o quarto. *Contar cartões de cabeça é
/// exactamente o que esta régua existe para impedir.*
#[test]
fn every_row_the_tutorial_names_is_on_the_card() {
    let mut m = crate::motion_state::MotionState::new();
    let _ = crate::motion_demo_legend::monta("112", &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::render_loop::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor::ProjectSettings::default(),
        &mut snap,
    );

    // O que o tutorial manda procurar: (título do cartão, linhas dentro dele).
    let pedidos: &[(&str, &[&str])] = &[
        ("Falloff", &["Shape", "Radius", "Center X", "Center Y"]),
        ("Remap", &["Curvature", "Multiplier"]),
    ];
    for (titulo, linhas) in pedidos {
        let v = snap
            .nodes
            .iter()
            .find(|v| v.display_name == *titulo)
            .unwrap_or_else(|| {
                let havia: Vec<&str> = snap.nodes.iter().map(|v| v.display_name.as_str()).collect();
                panic!(
                    "o tutorial manda procurar o cartao `{titulo}` e a cena nao tem nenhum com \
                     esse nome -- ha': {havia:?}"
                )
            });
        let rows: Vec<&str> = v.params.iter().map(|c| c.hint.label).collect();
        for l in *linhas {
            assert!(
                rows.contains(l),
                "o tutorial manda clicar em `{l}` no cartao `{titulo}`, e o cartao mostra {rows:?}"
            );
        }
    }

    // ⚠️ **E os nomes que o tutorial usa FORA da cena** — os dois nós de duas portas, que a §5
    // dele nomeia. Um cartão pinta-se pelo `display_name`, que **não** é o `type_name`: o
    // `field.remap` é `Remap`, o `field.combine` é `Combine Fields`. A 1.ª redacção deste
    // tutorial escreveu «Field Remap», «Field Box», «Field Combine» e «Field Shape» — **quatro**
    // nomes que não existem no ecrã.
    for (tipo, esperado) in [
        ("field.box", "Box"),
        ("field.radial_sweep", "Radial Sweep"),
        ("field.index_range", "Index Range"),
        ("field.combine", "Combine Fields"),
        ("field.shape", "Shape Field"),
    ] {
        let mut d = crate::motion_state::MotionState::new();
        let id = d.doc.graph.add_node(tipo.to_string());
        let snap = ph2d_panel_motion_graph::snapshot_from(&d.doc.graph, &d.registry);
        let nome = snap
            .nodes
            .iter()
            .find(|v| v.id == id.0)
            .map_or("(sem cartao)", |v| v.display_name.as_str());
        assert_eq!(
            nome, esperado,
            "o tutorial chama-lhe `{esperado}` e o cartao pinta-se `{nome}`"
        );
        // ⚠️ **E o nome esperado tem de estar de facto NO TUTORIAL** — senão esta lista é um
        // espelho meu, e alguém pode editar o texto sem que nada acuse. Com as duas metades, o
        // par «o que o app pinta» ⟷ «o que o dono lê» não pode divergir em silêncio.
        assert!(
            TUTORIAL.contains(esperado),
            "o gate diz que o cartao se chama `{esperado}` e o tutorial nao o menciona -- um dos \
             dois envelheceu"
        );
    }

    // ⚠️ E a linha `Rotation` NÃO pode estar lá no estado em que a cena abre — o tutorial ensina
    // que ela **aparece** ao trocar a forma, e um passo que promete uma aparição sobre algo que
    // já estava lá ensina o contrário do que acontece.
    let falloff = snap
        .nodes
        .iter()
        .find(|v| v.display_name == "Falloff")
        .expect("o cartao");
    let rows: Vec<&str> = falloff.params.iter().map(|c| c.hint.label).collect();
    assert!(
        !rows.contains(&"Rotation"),
        "a cena abre com `Rotation` ja' no cartao -- o passo 4 promete que ela APARECE"
    );
}

/// ⚠️ **SONDA de resposta ao dono (09/09): «o Index Range tem como rodar o campo?»**
///
/// Ele **não** tem — e não pode ter, porque não é um campo ESPACIAL: escolhe por **posto**, não
/// por posição (é por isso que o gizmo de canvas devolve `None` para ele). O que responde à
/// pergunta por baixo — *«quero a faixa a correr numa direcção que eu escolho»* — é o
/// `motion.sort` a montante, com o `Axis Angle`. **Medido** (grelha 12×12, faixa `0,25..0,75`):
///
/// ```text
/// axis_angle   0° → 50 apanhados · x -2.50..2.50 · y -5.50..5.50   (tira VERTICAL)
/// axis_angle  90° → 50 apanhados · x -5.50..5.50 · y -2.50..2.50   (tira HORIZONTAL)
/// ```
///
/// ⚠️ **E a 1.ª corrida desta sonda leu «zero diferença»** — as duas caixas davam a grelha
/// inteira — porque eu não pus o `key` do sort no modo espacial: no default (`Radial`) o
/// `axis_angle` não é lido. *Um param no default mede o param desligado*, e a sonda quase
/// respondeu ao dono que a composição não funciona.
///
/// ⛔ **O preço da composição, nomeado:** o `motion.sort` **reordena o stream** (toda coluna
/// viaja com a permutação), então quem está à frente de quem muda. Para uma faixa espacial
/// rodada **sem** mexer na ordem, o nó é o `field.box` — fino e rodado —, que desde a W1 tem alça
/// no canvas.
#[test]
#[ignore = "sonda de resposta — corra à mão"]
fn does_a_sort_axis_turn_the_index_range_band() {
    use ph2d_nodegraph::graph::Edge;
    let colher = |graus: f32| -> Vec<[f32; 2]> {
        let mut m = crate::motion_state::MotionState::new();
        let g = &mut m.doc.graph;
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 12.0);
        g.set_param(grid, "cols", 12.0);
        let sort = g.add_node("motion.sort");
        // ⚠️ **O `key` TEM de ser o modo espacial** (`1` = X): no default (`Radial`) o
        // `axis_angle` não é lido, e a 1.ª corrida desta sonda leu «zero diferença» sobre um
        // param desligado — a armadilha de sempre.
        g.set_param(sort, "key", 1.0);
        g.set_param(sort, "axis_angle", graus);
        let faixa = g.add_node("field.index_range");
        let cresce = g.add_node("motion.scale");
        g.set_param(cresce, "amount", 3.0);
        let out = g.add_node("motion.output");
        for (a, b) in [(grid, sort), (sort, faixa), (faixa, cresce), (cresce, out)] {
            g.connect(Edge {
                from: (a, 0),
                to: (b, 0),
                delayed: false,
            })
            .expect("liga");
        }
        let saida = m
            .pump
            .cook
            .cook(&m.doc.graph, &m.registry, out, 0.0)
            .expect("coze");
        let st = saida[0].as_stream();
        let p = match st.get("P") {
            Some(ph2d_nodegraph::attr::Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        };
        let s = match st.get("size") {
            Some(ph2d_nodegraph::attr::Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        };
        // Só os que a faixa apanhou (size grande), pela POSIÇÃO — a ordem do stream muda.
        let maior = s.iter().fold(0.0f32, |a, q| a.max(q[0]));
        p.into_iter()
            .zip(s)
            .filter(|(_, t)| t[0] > maior * 0.9)
            .map(|(q, _)| q)
            .collect()
    };
    for graus in [0.0f32, 90.0] {
        let apanhados = colher(graus);
        let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for q in &apanhados {
            x0 = x0.min(q[0]);
            x1 = x1.max(q[0]);
            y0 = y0.min(q[1]);
            y1 = y1.max(q[1]);
        }
        eprintln!(
            "  axis_angle {graus:>5.0}° → {} apanhados · x {x0:.2}..{x1:.2} · y {y0:.2}..{y1:.2}",
            apanhados.len()
        );
    }
}
