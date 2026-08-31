//! **Todo param de todo nó CHEGA ao painel** — o censo do teto de linhas (doc 88, B3).
//!
//! Irmão do `range_tests` (a ESCALA de um valor) e do `unit_tests` (a UNIDADE dele): este mede
//! se o valor **aparece**. Um param acima do `MAX_PARAM_ROWS` não é desenhado nem registrado —
//! o `.take()` do `paint_rows` o descarta —, então ele existe no modelo, o cook o lê, e o
//! artista não tem gesto nenhum que o alcance. É a falha silenciosa que as quatro condições de
//! UI proíbem, e a única testemunha possível é um censo sobre o registry inteiro: nenhum gate
//! por-nó a veria, porque cada um usa a fixture do seu próprio nó.
//!
//! ⚠️ O teto é um recurso de verdade — o `populate` do painel registra **21 widgets por slot**
//! —, então ele não pode simplesmente sumir; o que ele pode é ser **medido** (§0). A sonda
//! abaixo imprime o censo; o gate o mantém honesto.

use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;
use ph2d_panel_motion_params::MAX_PARAM_ROWS;

/// Quantas linhas de painel cada tipo de nó do registry produz, do maior para o menor.
///
/// ⚠️ Conta as linhas do SNAPSHOT, não os `ParamSpec` do manifesto: um nó emite também as
/// linhas de text param (Curve / Gradient / Palette / Text / Source / Channels) **antes** do
/// laço do manifesto, e é a soma que disputa os slots. Contar o manifesto responderia a outra
/// pergunta e reportaria um teto folgado demais.
fn row_census() -> Vec<(&'static str, usize)> {
    let mut motion = MotionState::new();
    let types: Vec<&'static str> = motion.registry.manifests().map(|m| m.name).collect();
    let mut census: Vec<(&'static str, usize)> = types
        .into_iter()
        .map(|ty| {
            let node = motion.doc.graph.add_node(ty);
            // ⛔⛔ **ACORDA OS GATES DE LIMIAR ANTES DE CONTAR** (doc 96 §4.5).
            //
            // Um `ParamGateAbove` esconde enquanto a grandeza que o decide está em zero — e um
            // nó recém-criado tem TODOS os params no default. O censo media logo a seguir ao
            // `add_node` e por isso contava o `source.lsystem` em **32** linhas; **um gesto de
            // slider** (`Tropism` fora do zero) revela a 33.ª, que é o `MAX_PARAM_ROWS`
            // **exacto**. ⇒ a folga real era **ZERO** e o censo reportava `1`.
            //
            // ⚠️ *Um censo que mede o estado de fábrica mede o painel mais MAGRO que aquele nó
            // consegue ter* — e o teto existe para o mais GORDO. Empurrar cada `when` de limiar
            // um passo acima do limiar é o que torna a contagem a do pior caso alcançável.
            for g in motion
                .registry
                .param_gates_above(
                    motion
                        .doc
                        .graph
                        .node(node)
                        .map_or_else(|| unreachable!("acabou de ser criado"), |i| i.type_id()),
                )
                .into_iter()
                .flatten()
            {
                motion.doc.graph.set_param(node, g.when, g.above + 1.0);
            }
            ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
            let n = build_params_snapshot(&motion, ProjectSettings::default())
                .map_or(0, |s| s.rows.len());
            (ty, n)
        })
        .collect();
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    census.sort_by_key(|&(_, n)| std::cmp::Reverse(n));
    census
}

/// **Nenhum param fica fora da tela.**
///
/// Nasceu VERMELHO contra o teto de 8 que shipava: o `field.remap` produz linhas acima dele, e
/// as excedentes eram descartadas em silêncio. A mutação que o prova é baixar
/// `MAX_PARAM_ROWS` de volta — o gate nomeia o nó e a contagem, em vez de dizer só "falhou".
#[test]
fn the_panel_shows_every_param_of_every_node() {
    let census = row_census();
    let over: Vec<String> = census
        .iter()
        .filter(|(_, n)| *n > MAX_PARAM_ROWS)
        .map(|(ty, n)| format!("{ty} ({n} linhas)"))
        .collect();
    assert!(
        over.is_empty(),
        "estes nós têm mais linhas que MAX_PARAM_ROWS ({MAX_PARAM_ROWS}), e o excedente é \
         descartado pelo `.take()` do paint_rows — o param existe e o artista não o alcança: \
         {over:?}"
    );
}

/// **E o teto não é folgado a ponto de não medir nada.**
///
/// A metade oposta, e ela não é cerimônia: sem isto, "conserte o gate acima" tem uma resposta
/// trivial e errada — pôr o teto em 256 e pagar 5376 registros de widget no `populate` por um
/// número que ninguém mediu. O teto é o pior caso medido mais folga de uma família; se o censo
/// cair muito abaixo dele, é sinal de que ele foi escolhido em vez de medido.
#[test]
fn the_row_cap_is_measured_not_guessed() {
    let census = row_census();
    let worst = census.first().copied().expect("o registry não é vazio");
    assert!(
        worst.1 <= MAX_PARAM_ROWS,
        "o pior nó ({} com {} linhas) não cabe no teto {MAX_PARAM_ROWS}",
        worst.0,
        worst.1
    );
    assert!(
        MAX_PARAM_ROWS <= worst.1 * 2,
        "o teto {MAX_PARAM_ROWS} é mais que o dobro do pior nó medido ({} com {} linhas) — \
         cada slot custa 21 registros de widget no populate, então isto é orçamento gasto \
         num número que ninguém mediu",
        worst.0,
        worst.1
    );
}

/// A SONDA: imprime o censo inteiro, para o número do teto sair de uma medição.
/// `cargo test -p ph2d-host-desktop measure_the_param_row_census -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição, não gate"]
fn measure_the_param_row_census() {
    let census = row_census();
    println!("\n=== LINHAS DE PAINEL POR NÓ (teto atual: {MAX_PARAM_ROWS}) ===");
    for (ty, n) in census.iter().take(20) {
        let flag = if *n > MAX_PARAM_ROWS {
            "  <-- CORTADO"
        } else {
            ""
        };
        println!("{n:3}  {ty}{flag}");
    }
    let over = census.iter().filter(|(_, n)| *n > MAX_PARAM_ROWS).count();
    println!(
        "--- {} tipos no total, {} acima do teto\n",
        census.len(),
        over
    );
}

/// **As SEÇÕES agrupam as rows, e a ordem é a que a tabela declara.**
///
/// Um nó com grupos tem de entregar as rows já ORDENADAS (soltas primeiro, depois grupo a
/// grupo) e dizer onde cada seção começa. Sem a ordenação o pintor desenharia o mesmo
/// cabeçalho várias vezes, intercalado — que é pior que lista plana.
#[test]
fn a_grouped_node_delivers_its_rows_sorted_with_the_loose_ones_first() {
    let mut motion = MotionState::new();
    let node = motion.doc.graph.add_node("field.remap");
    ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
    let snap = build_params_snapshot(&motion, ProjectSettings::default()).expect("o nó existe");

    assert!(
        !snap.sections.is_empty(),
        "o field.remap declara grupos — sem seções a tabela não chegou ao painel"
    );
    // As soltas vêm antes da primeira seção, e são os essenciais (a transferência).
    let first = snap.sections[0].1;
    assert!(
        first > 0,
        "os params SEM grupo têm de vir antes de toda seção — é onde os essenciais moram"
    );
    // Os índices são crescentes e cada seção começa onde uma row começa.
    let mut prev = 0;
    for (title, at) in &snap.sections {
        assert!(*at > prev || *at == first, "{title}: seções fora de ordem");
        assert!(
            *at < snap.rows.len(),
            "{title}: seção depois do fim das rows"
        );
        prev = *at;
    }
    // E nenhuma seção repete: rows do mesmo grupo são contíguas.
    let titles: Vec<&String> = snap.sections.iter().map(|(t, _)| t).collect();
    let mut uniq = titles.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(
        titles.len(),
        uniq.len(),
        "um grupo apareceu duas vezes — as rows dele não estão contíguas: {titles:?}"
    );
    // E nenhuma seção fica VAZIA: um cabeçalho sem rows embaixo é a seção-morta, irmã do
    // botão-morto — ele desenha, dobra, e não esconde nada.
    for (k, (title, at)) in snap.sections.iter().enumerate() {
        let end = snap
            .sections
            .get(k + 1)
            .map_or(snap.rows.len(), |(_, next)| *next);
        assert!(
            end > *at,
            "a seção {title} não tem row nenhuma embaixo dela"
        );
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **Toda entrada de `ParamGroup` nomeia um param que o nó DECLARA.**
///
/// Um nome errado numa tabela de seções não falha em lugar nenhum: o `param_group` não acha a
/// entrada, a row fica solta, e "solta" é exatamente o que uma escolha deliberada também
/// parece. O nó continua compilando, o painel continua pintando, e o param que devia estar
/// numa seção fica na parede — que é o problema que a seção existe para resolver.
///
/// Por isso o censo é sobre o REGISTRY inteiro, e não sobre um nó: cada gate por-nó usa a
/// fixture do próprio nó, e a tabela do sétimo nasce sem testemunha nenhuma.
///
/// O conjunto aceito é *params do manifesto* ∪ *nomes das rows do snapshot* — a união porque um
/// text param (curva, gradiente, paleta, fórmula) produz row e **não** aparece no manifesto,
/// e agrupá-lo é legítimo.
#[test]
fn every_param_group_entry_names_a_param_the_node_declares() {
    let mut motion = MotionState::new();
    let types: Vec<(&'static str, ph2d_nodegraph::node::NodeTypeId)> = motion
        .registry
        .manifests()
        .map(|m| (m.name, m.id))
        .collect();
    let mut bad: Vec<String> = Vec::new();
    for (ty, id) in types {
        let groups = motion.registry.param_groups(id);
        if groups.is_empty() {
            continue;
        }
        let node = motion.doc.graph.add_node(ty);
        ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
        let mut declared: std::collections::BTreeSet<String> = Default::default();
        if let Some(snap) = build_params_snapshot(&motion, ProjectSettings::default()) {
            for row in &snap.rows {
                declared.extend(row.params().iter().map(|p| (*p).to_string()));
            }
        }
        if let Some(op) = {
            use ph2d_nodegraph::cook::OpResolver;
            motion.registry.resolve(id)
        } {
            declared.extend(op.manifest().params.iter().map(|p| p.name.to_string()));
        }
        // ⚠️⚠️ **E os HINTS, que é onde os TEXT params vivem** — o `ParamSpec` é `f32`, então
        // um `axiom`/`rules`/`curve` não está no manifesto, e o snapshot acima só traz as rows
        // do estado em que este nó acabou de nascer. Um param **gateado** (o `source.lsystem`
        // esconde a gramática no modo `Guided`, que é o default) não aparece em nenhum dos
        // dois, e o censo acusava a seção dele de nomear um param inexistente.
        // *Um censo que mede um estado não pode julgar uma tabela que vale em todos.*
        if let Some(hints) = motion.registry.param_ui(id) {
            declared.extend(hints.iter().map(|h| h.param.to_string()));
        }
        for g in groups {
            if !declared.contains(g.param) {
                bad.push(format!("{ty}: a seção {:?} nomeia {:?}", g.group, g.param));
            }
        }
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    assert!(
        bad.is_empty(),
        "estas entradas de ParamGroup nomeiam params que o nó não tem — a row fica SOLTA e \
         nada acusa: {bad:?}"
    );
}

/// **E os nós que a medição nomeou de fato entregam seções.**
///
/// A metade oposta do gate acima: sem ela, "conserte os nomes" tem a resposta trivial de
/// apagar as tabelas. A lista sai da sonda `measure_the_param_row_census` — são os nós de 9+
/// linhas, os que a parede de sliders de fato machuca.
#[test]
fn the_nodes_the_census_named_all_ship_sections() {
    let mut motion = MotionState::new();
    for ty in [
        "field.remap",
        "motion.emitter",
        "motion.boids",
        "field.radial_sweep",
        "value.pattern",
        "motion.spline_wrap",
        "motion.distribute_curve",
    ] {
        let node = motion.doc.graph.add_node(ty);
        ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
        let snap = build_params_snapshot(&motion, ProjectSettings::default())
            .unwrap_or_else(|| panic!("{ty} existe no registry"));
        assert!(
            !snap.sections.is_empty(),
            "{ty} tem {} linhas e nenhuma seção — é a parede plana que o doc 88 B3 ataca",
            snap.rows.len()
        );
        // E sobra algo SOLTO: uma seção que engole o nó inteiro põe todo controle atrás de um
        // clique, que troca uma parede por uma porta trancada.
        assert!(
            snap.sections[0].1 > 0,
            "{ty} agrupou TODOS os params — nenhum essencial ficou solto na frente",
        );
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **Só a régua ESCOLHIDA é oferecida** — o `time_mode` do oscilador (doc 88 B3).
///
/// `frequency` e `bpm` são a MESMA grandeza em duas unidades. Mostrar as duas seria pior que
/// um botão morto: seriam dois números na tela discordando sobre um só valor, sem nada dizendo
/// qual manda — e o cook lê exatamente um deles.
///
/// As duas metades (presença E ausência) num gate só, porque cada uma sozinha tem resposta
/// trivial: "sempre mostre os dois" passa na presença, "nunca mostre nenhum" passa na ausência.
#[test]
fn the_oscillator_offers_only_the_time_ruler_it_uses() {
    let mut motion = MotionState::new();
    let node = motion.doc.graph.add_node("motion.oscillator");
    ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);

    let names = |motion: &MotionState| -> Vec<String> {
        build_params_snapshot(motion, ProjectSettings::default())
            .expect("o no existe")
            .rows
            .iter()
            .flat_map(|r| r.params().into_iter().map(|p| p.to_string()))
            .collect()
    };

    // Segundos (o default): o Hz aparece, o BPM não.
    let secs = names(&motion);
    assert!(secs.iter().any(|p| p == "frequency"), "{secs:?}");
    assert!(!secs.iter().any(|p| p == "bpm"), "{secs:?}");

    // BPM: exatamente o inverso.
    motion.doc.graph.set_param(node, "time_mode", 1.0);
    let bpm = names(&motion);
    assert!(bpm.iter().any(|p| p == "bpm"), "{bpm:?}");
    assert!(!bpm.iter().any(|p| p == "frequency"), "{bpm:?}");

    // E o SELETOR está sempre lá — a régua se escolhe, então o controle que a escolhe não
    // pode desaparecer com a escolha (seria a única porta de volta).
    assert!(
        secs.iter().any(|n| n == "time_mode"),
        "o seletor sumiu em Seconds"
    );
    assert!(
        bpm.iter().any(|n| n == "time_mode"),
        "o seletor sumiu em BPM"
    );
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// ⛔⛔ **O CENSO MEDE O PAINEL MAIS GORDO QUE UM GESTO ALCANÇA, e não o de fábrica.**
///
/// Auditoria de seis lentes, doc 96 §4.5. O [`row_census`] media logo a seguir ao `add_node`, e
/// ali **todos** os params estão no default — logo todo `ParamGateAbove` está a esconder. O
/// `source.lsystem` contava **32** linhas; **um gesto de slider** (`Tropism` fora do zero)
/// revelava a 33.ª, que era o `MAX_PARAM_ROWS` **exacto**: a folga real era **zero** e o censo
/// reportava `1`.
///
/// ⚠️⚠️ **E o defeito seguinte seria SILENCIOSO:** o `.take(MAX_PARAM_ROWS)` do pintor descarta
/// a linha excedente sem dizer nada, e o gate que devia acusar mede o estado de fábrica — onde
/// ela ainda cabe.
///
/// ⚠️ **Sem este gate, a cura do censo é infalsificável:** acordar os limiares mudou o número
/// (`30 → 31`) e nenhuma asserção dependia dele. *Uma correcção que nenhum gate lê é uma
/// declaração com um número mais bonito.*
#[test]
fn the_census_measures_the_fattest_panel_a_gesture_can_reach() {
    let mut motion = MotionState::new();
    let ty = ph2d_node_source_lsystem::MANIFEST.name;
    let id = ph2d_node_source_lsystem::MANIFEST.id;
    let acima = motion
        .registry
        .param_gates_above(id)
        .into_iter()
        .flatten()
        .count();
    // ⚠️ O controlo do próprio gate: sem um gate de limiar não há nada para acordar, e ele
    // passaria a medir duas vezes a mesma coisa.
    assert!(
        acima > 0,
        "`{ty}` deixou de ter gates de limiar — este gate perdeu o sujeito, escolha outro nó"
    );

    let contar = |motion: &mut MotionState, acordar: bool| {
        let node = motion.doc.graph.add_node(ty);
        if acordar {
            for g in motion
                .registry
                .param_gates_above(id)
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
            {
                motion.doc.graph.set_param(node, g.when, g.above + 1.0);
            }
        }
        ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
        build_params_snapshot(motion, ProjectSettings::default()).map_or(0, |s| s.rows.len())
    };
    let fabrica = contar(&mut motion, false);
    let gesto = contar(&mut motion, true);
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    assert!(
        gesto > fabrica,
        "acordar os {acima} limiar(es) de `{ty}` não revelou linha nenhuma ({fabrica} contra \
         {gesto}) — o censo está a medir o painel de fábrica, que é o mais MAGRO"
    );
    // E o censo do produto tem de reportar o número GORDO, não o magro.
    let do_censo = row_census()
        .into_iter()
        .find(|(n, _)| *n == ty)
        .map_or(0, |(_, n)| n);
    assert_eq!(
        do_censo, gesto,
        "o censo reporta {do_censo} e o pior caso alcançável é {gesto}"
    );
}
