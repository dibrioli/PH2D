//! **As SEÇÕES de um nó chegam ORDENADAS, e cada uma nomeia um param que ele declara** — o que
//! sobra do censo do tecto de linhas (doc 88 B3) depois de o painel lateral sair (doc 114 §13).
//!
//! ⛔⛔ **O TECTO SAIU DAQUI porque o consumidor dele saiu do app.** Este ficheiro tinha dois
//! gates e uma sonda sobre o `MAX_PARAM_ROWS`, e a falha que eles vigiavam era o `.take()` do
//! `paint_rows` **do painel** a descartar em silêncio a row acima do tecto. Com a pintura fora,
//! o censo por porta dá **zero** leitores de produto para aquele número na workspace inteira: *um
//! tecto cujo consumidor saiu não protege nada — ele fica a ser um número que gates verdes
//! continuam a afirmar sobre o vazio*, que é a forma mais cara de cobertura falsa.
//!
//! ⚠️ **E o cartão não herda a pergunta**, o que foi medido antes de cortar e não presumido: ele
//! não desenha uma fileira de slots com `.take()` — ele pinta as rows que o snapshot traz, e um
//! selector dele **cicla** (`ClickDoes::Cycle(labels.len())`) em vez de expor uma opção por
//! botão. Não há onde truncar, logo não há param nem opção que caia em silêncio.
//!
//! ⭐ **O que fica é do SNAPSHOT, e o snapshot é do cartão:** a ordenação por grupo e a
//! integridade da tabela de secções. Elas nunca foram sobre quem pinta — são sobre o que a
//! ponte entrega —, e é por isso que sobrevivem à morte do pintor.

use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor_core::ProjectSettings;

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
                bad.push(format!(
                    "{ty}: a seção {:?} nomeia {:?}",
                    g.group_key, g.param
                ));
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
/// apagar as tabelas. ⚠️ A lista foi colhida de uma sonda de contagem de linhas que SAIU com o
/// painel lateral (doc 114 §13) — são os nós de 9+ linhas, os que a parede de sliders de facto
/// machuca. *A lista fica porque o que ela nomeia é o produto; o que morreu foi a ferramenta que
/// a colheu, e quem a quiser refazer volta a contar `rows.len()` sobre o snapshot.*
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

/// ⛔⛔ **UM GESTO REVELA LINHAS QUE O ESTADO DE FÁBRICA NÃO TEM** — e medir o painel de fábrica
/// é medir o mais MAGRO.
///
/// Auditoria de seis lentes, doc 96 §4.5. Logo a seguir a um `add_node` **todos** os params
/// estão no default, logo todo `ParamGateAbove` está a esconder: o `source.lsystem` entregava
/// **32** linhas, e um gesto de slider (`Tropism` fora do zero) revelava a 33.ª.
///
/// ⚠️⚠️ **A SEGUNDA METADE deste gate SAIU com o painel lateral** (doc 114 §13): ela afirmava
/// que o censo do tecto reportava o número GORDO e não o magro, e aquele censo existia para
/// alimentar o `MAX_PARAM_ROWS`, cujo consumidor — o `.take()` do pintor — já não existe.
/// *Uma metade que perde o sujeito sai; a que mede o SNAPSHOT fica*, porque o cartão lê
/// exactamente as mesmas rows e a pergunta *«este param é alcançável?»* continua a ser dele.
///
/// ⚠️ O que fica é falsificável na mesma: a mutação é não acordar os limiares no `contar`, e o
/// gate lê `fabrica == gesto` com o número dos dois lados.
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
}
