//! **A cena `=108` MOSTRA o que diz que mostra** — a metade executável do «se a linha não
//! aparecer, PARE».
//!
//! ⚠️ O anúncio dela promete quatro leituras ao Enio (*"a 2 e a 3 têm de sair diferentes"*,
//! *"a 4 verga"*, …). Uma promessa impressa no terminal é uma frase; estes gates são o que a
//! torna uma afirmação — e o que a mantém verdadeira no dia em que alguém mexer no nó.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::value::CookValue;

/// Coze o documento da cena e devolve a nuvem de posições de cada planta.
/// As nuvens de posições no instante `t`.
fn plants_at(t: f64) -> Vec<Vec<[f32; 2]>> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registam");
    let mut doc = MotionDoc::default();
    let sinks = build_lsystem_demo_document(&mut doc, &reg).expect("a cena monta");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(
            |s| match &cook.cook(&doc.graph, &reg, *s, t).expect("coze")[0] {
                CookValue::Instances(st) => match st.get("P") {
                    Some(Column::Vec2(v)) => v.clone(),
                    _ => Vec::new(),
                },
                other => panic!("esperava instancias, veio {other:?}"),
            },
        )
        .collect()
}

fn plants() -> Vec<Vec<[f32; 2]>> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registam");
    let mut doc = MotionDoc::default();
    let sinks = build_lsystem_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), PLANTS.len(), "uma sink por planta");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(
            |s| match &cook.cook(&doc.graph, &reg, *s, 0.0).expect("coze")[0] {
                CookValue::Instances(st) => match st.get("P") {
                    Some(Column::Vec2(v)) => v.clone(),
                    _ => panic!("a planta tem de trazer P"),
                },
                other => panic!("esperava instancias, veio {other:?}"),
            },
        )
        .collect()
}

/// **Toda planta desenha alguma coisa** — o controlo mais barato e o que apanha uma cena que
/// monta um grafo e cozinha o vazio.
///
/// ⚠️ **Medida no PICO do ciclo, e não em `t = 0`.** O tamanho da 5.ª é função do relógio (o
/// `Generations` dela vem de um LFO), então num instante qualquer ela pode estar legitimamente
/// pequena — a 1.ª redacção deste gate lia `t = 0`, apanhava-a a meio do ciclo com 8 elementos
/// e acusava a cena de estar vazia. *Uma régua fixa num instante mede o relógio, não a planta.*
#[test]
fn every_plant_in_the_scene_actually_grows() {
    let peak: Vec<usize> = (0..12).map(|k| plants_at(f64::from(k) * 0.4)).fold(
        vec![0usize; PLANTS.len()],
        |acc, now| {
            acc.iter()
                .zip(&now)
                .map(|(a, p)| (*a).max(p.len()))
                .collect()
        },
    );
    for (k, n) in peak.iter().enumerate() {
        assert!(
            *n > 20,
            "a planta {} saiu com {n} elementos no pico",
            PLANTS[k].label
        );
    }
}

/// ⭐ **A 2 e a 3 são a MESMA gramática com sementes diferentes, e TÊM de divergir.**
///
/// É a leitura nº 2 do anúncio. Sem ela, uma estocástica partida (um sorteio que ignora a
/// semente) desenharia duas plantas gémeas e a cena continuaria bonita.
#[test]
fn the_two_stochastic_plants_are_not_twins() {
    let p = plants();
    assert_eq!(
        PLANTS[1].rules, PLANTS[2].rules,
        "as duas tem de partilhar a gramatica, senao a diferenca nao e' da semente"
    );
    assert_ne!(
        PLANTS[1].seed, PLANTS[2].seed,
        "e tem de diferir na semente"
    );
    let same = p[1].len() == p[2].len() && p[1].iter().zip(&p[2]).all(|(a, b)| a == b);
    assert!(!same, "as duas plantas estocasticas sairam gemeas");
}

/// ⭐ **A 4 é a 1 com gravidade, e ela VERGA.**
///
/// A régua é o `x` mais à direita: com o tropismo a puxar para baixo, as pontas caem e a
/// envergadura muda. As duas partilham tudo menos o tropismo — que é o que faz disto uma
/// afirmação sobre o tropismo.
#[test]
fn the_gravity_plant_hangs_lower_than_the_one_it_copies() {
    let p = plants();
    assert_eq!(
        PLANTS[0].rules, PLANTS[3].rules,
        "a 4 e' a 1, com um numero mudado"
    );
    assert_eq!(PLANTS[0].tropism, 0.0);
    // ⚠️ **POSITIVO puxa PARA a direcção declarada** (que aqui aponta para baixo). Esta linha
    // dizia `< 0.0` e a cena tinha o sinal trocado — os dois erros concordavam, e a planta
    // «com gravidade» saía mais direita do que a sem. *Duas coisas erradas da mesma maneira
    // não se acusam uma à outra: quem as separou foi a régua GEOMÉTRICA abaixo.*
    assert!(
        PLANTS[3].tropism > 0.0,
        "a 4 tem de puxar PARA a direccao do tropismo"
    );
    assert_eq!(
        PLANTS[3].angle, PLANTS[0].angle,
        "e o angulo tem de ser o mesmo"
    );
    // ⚠️ **A régua é o CENTROIDE, e a 1.ª foi o ponto mais alto — que não mede vergar.**
    // Numa árvore simétrica o topo é alcançado pelo caminho mais VERTICAL, e é exactamente
    // aí que o tropismo é mais fraco: a lei do ABOP é `α = e·(H × T)`, e o produto vectorial
    // ANULA-SE quando o ramo já aponta ao longo da gravidade. O topo mexeu-se `0,005` e o
    // gate acusou o produto de estar partido. *Vergar é a massa descer, não a ponta.*
    let mid = |v: &Vec<[f32; 2]>| v.iter().map(|q| q[1]).sum::<f32>() / v.len() as f32;
    let span = |v: &Vec<[f32; 2]>| {
        v.iter().map(|q| q[1]).fold(f32::MIN, f32::max)
            - v.iter().map(|q| q[1]).fold(f32::MAX, f32::min)
    };
    let (a, b) = (mid(&p[0]), mid(&p[3]));
    // ⚠️ **A barra é uma FRACÇÃO da altura da planta, não um número solto.** A cena é um
    // demo: o que ela tem de provar é que a diferença se VÊ, e «vê-se» mede-se contra o
    // tamanho da coisa. Um limiar absoluto seria re-precificado por qualquer mexida no `Step`.
    let bar = span(&p[0]) * 0.1;
    assert!(
        b < a - bar,
        "a massa da planta com gravidade tem de descer >{bar:.3}: {a} contra {b}"
    );
}

/// **A espessura chega ao canvas** — o `!` da gramática, visto na coluna `size`.
///
/// Sem isto, um `size` constante desenharia a árvore como uma nuvem de pontos iguais: a
/// leitura *"o tronco e' grosso e as pontas sao finas"* do anúncio ficaria falsa e nada
/// vermelho o diria.
#[test]
fn the_trunk_is_thicker_than_the_twigs_all_the_way_to_the_sink() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registam");
    let mut doc = MotionDoc::default();
    let sinks = build_lsystem_demo_document(&mut doc, &reg).expect("a cena monta");
    let mut cook = Cook::new();
    let CookValue::Instances(st) = &cook.cook(&doc.graph, &reg, sinks[0], 0.0).expect("coze")[0]
    else {
        panic!("instancias")
    };
    let Some(Column::Vec2(size)) = st.get("size") else {
        panic!("a planta tem de trazer size")
    };
    let (min, max) = size.iter().fold((f32::MAX, f32::MIN), |(lo, hi), s| {
        (lo.min(s[0]), hi.max(s[0]))
    });
    assert!(
        max > min * 4.0,
        "a espessura tem de variar da raiz a' ponta: {min} a {max}"
    );
}
/// ⭐⭐ **A QUINTA PLANTA ANDA COM O RELÓGIO** — a leitura mais importante da cena, e a que
/// ninguém estava a afirmar.
///
/// ⚠️ **Este gate nasceu de um report de «não há movimento»** (Enio, 2026-08-28). O report era
/// sobre OUTRA cena, mas a pergunta ficou de pé: *o que é que aqui prova que a planta 5 cresce?*
/// Nada. A cena imprimia a promessa no terminal e o resto era confiança.
///
/// A régua é a CONTAGEM de elementos: com o `Generations` ligado a um relógio, a planta ganha e
/// perde gerações inteiras ao longo do ciclo, e a contagem tem de variar. E o CONTROLE são as
/// outras quatro, que têm de ficar **exactamente** paradas — senão o que se estaria a medir era
/// o cook a ser não-determinista.
#[test]
fn only_the_fifth_plant_moves_with_the_clock() {
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::value::CookValue;
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registam");
    let mut doc = MotionDoc::default();
    let sinks = build_lsystem_demo_document(&mut doc, &reg).expect("a cena monta");
    // ⚠️ **UM cozinhador para toda a varredura, e é a régua do APP.** Um `Cook::new()` por
    // instante nunca devolve nada de velho — foi essa a cegueira que deixou o `motion.sub_uv`
    // congelado passar por todos os gates (2026-08-28). Aqui a planta 5 tem de andar com o
    // MESMO memo que o app usa.
    let mut cook = Cook::new();
    let counts_at = |cook: &mut Cook, t: f64| -> Vec<usize> {
        sinks
            .iter()
            .map(
                |s| match &cook.cook(&doc.graph, &reg, *s, t).expect("coze")[0] {
                    CookValue::Instances(st) => st.count(),
                    other => panic!("esperava instancias, veio {other:?}"),
                },
            )
            .collect()
    };
    let base = counts_at(&mut cook, 0.0);
    let mut moved = vec![false; sinks.len()];
    for k in 1..=8 {
        let now = counts_at(&mut cook, f64::from(k) * 0.35);
        for (i, (a, b)) in base.iter().zip(&now).enumerate() {
            if a != b {
                moved[i] = true;
            }
        }
    }
    let last = sinks.len() - 1;
    assert!(
        moved[last],
        "a planta 5 tem de CRESCER com o relogio — o `Generations` dela esta' ligado a um LFO"
    );
    for (i, m) in moved.iter().enumerate().take(last) {
        assert!(!m, "a planta {} devia estar parada e mexeu-se", i + 1);
    }
}

/// ⭐⭐ **A 5.ª PLANTA NUNCA SOME** — a lei que o report de 2026-08-28 enuncia.
///
/// ⚠️ *"o tronco pisca uma vez"*. Medido: no fundo do ciclo o relógio levava o `Generations` a
/// **zero**, e zero gerações é o axioma por derivar — um módulo mudo. A planta ficava com UM
/// elemento (a raiz, que não desenha) e desaparecia, uma vez por volta.
///
/// A régua é a CONTAGEM ao longo de um ciclo inteiro, amostrada fino: em nenhum instante ela
/// pode cair ao osso. E o CONTROLE é ela de facto respirar — uma planta presa no máximo também
/// nunca sumiria, e não seria o que a coluna promete.
#[test]
fn the_growing_plant_never_blinks_out() {
    let counts: Vec<usize> = (0..=40)
        .map(|k| plants_at(f64::from(k) * 0.2)[4].len())
        .collect();
    let low = *counts.iter().min().expect("amostrou");
    let high = *counts.iter().max().expect("amostrou");
    assert!(
        low > 1,
        "a planta desapareceu (ficou com {low} elemento) — o fundo do relogio leva as \
         geracoes a zero: {counts:?}"
    );
    assert!(
        high > low * 4,
        "e o CONTROLE: ela tem de RESPIRAR, senao o gate acima e' o de uma planta parada \
         no maximo: {counts:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────
// O MODO — a cena passou a ter os DOIS (2026-08-29).
// ─────────────────────────────────────────────────────────────────────────────────────────

/// O nó `source.lsystem` de cada coluna, na ordem das colunas.
fn lsystem_nodes(g: &ph2d_nodegraph::graph::Graph) -> Vec<NodeId> {
    (0..g.nodes().len() as u32)
        .map(NodeId)
        .filter(|n| g.node(*n).is_some_and(|i| i.type_name == "source.lsystem"))
        .collect()
}

fn scene_graph() -> ph2d_nodegraph::graph::Graph {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registam");
    let mut doc = MotionDoc::default();
    build_lsystem_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph
}

/// ⭐⭐⭐ **CADA PLANTA DECLARA O MODO EM QUE FOI AUTORADA** — o gate sem o qual esta cena
/// passaria a mostrar cinco vezes a mesma coisa, calada.
///
/// ⚠️ Desde 2026-08-29 o default do nó é `Guided`, e no guiado **o texto não é lido**. Uma
/// coluna que escrevesse a gramática sem declarar `Grammar` compilaria, cozinharia, desenharia
/// uma planta perfeitamente boa — a errada. *Um default que muda re-pergunta a toda fixtura
/// que dependia do antigo.*
///
/// ⚠️ **E a metade GUIADA é afirmada ao contrário**: ela não pode ter texto nenhum, senão o
/// gate seguinte (a identidade ao bit) estaria a comparar duas gramáticas em vez de comparar
/// os sliders com a gramática.
#[test]
fn every_plant_declares_the_grammar_mode_it_authors() {
    let g = scene_graph();
    let nodes = lsystem_nodes(&g);
    assert_eq!(nodes.len(), PLANTS.len(), "um no' por coluna");
    let mut guided_seen = 0usize;
    for (k, n) in nodes.iter().enumerate() {
        let mode = g
            .node_param_overrides(*n)
            .and_then(|m| m.get(ls::param::MODE))
            .copied()
            .unwrap_or(ls::MODE_GUIDED as f32)
            .round() as i32;
        let has_text = |key: &str| {
            g.node_text_param_overrides(*n)
                .and_then(|m| m.get(key))
                .is_some_and(|v| !v.trim().is_empty())
        };
        if PLANTS[k].guided {
            guided_seen += 1;
            assert_eq!(mode, ls::MODE_GUIDED, "a coluna {k} devia ser guiada");
            assert!(
                !has_text(ls::AXIOM_PARAM) && !has_text(ls::RULES_PARAM),
                "a coluna guiada {k} escreveu texto que ninguem le' — e a leitura dela deixa \
                 de ser sobre os sliders"
            );
        } else {
            assert_eq!(mode, ls::MODE_GRAMMAR, "a coluna {k} autora uma gramatica");
            assert!(
                has_text(ls::AXIOM_PARAM) && has_text(ls::RULES_PARAM),
                "a coluna {k} diz `Grammar` e nao escreveu gramatica nenhuma"
            );
        }
    }
    // ⚠️ **O CONTROLE, e é ele que torna a cena uma demonstração dos DOIS modos**: sem pelo
    // menos uma coluna guiada, o modo que o artista de facto encontra não está no ecrã.
    assert_eq!(
        guided_seen, 1,
        "a cena tem de conter EXACTAMENTE uma coluna guiada — zero esconde o modo default, \
         e mais do que uma gasta uma coluna que isola outra dimensao"
    );
}

/// ⭐⭐⭐ **A PLANTA GUIADA DA CENA DESENHA, AO BIT, O QUE A GRAMÁTICA DE FÁBRICA DESENHA.**
///
/// ⚠️ É a afirmação mais forte que esta wave pode fazer: *os sliders não são uma segunda
/// planta parecida — eles exprimem a MESMA*. O guiado emite `A(s*length_scale)` e a fábrica
/// traz o literal `0.7`; com o slider em [`GUIDED_LENGTH_SCALE`] as duas expressões são a
/// mesma, e a derivação e a tartaruga fazem as mesmas contas na mesma ordem.
///
/// ⚠️ **Ao BIT e não «parecido»**: uma barra frouxa aceitaria outra associação de
/// multiplicações, que é como o `rig.fk` já apanhou 1 ULP nesta crate. E a coluna 4 depende
/// disto — ela copia a 1 por gramática, e o gate da gravidade compara as duas.
///
/// ⚠️⚠️ **A 1.ª redacção deste gate SOBREVIVEU a apagar a linha do `length_scale` da CENA**
/// (mutação MS9): ela montava os dois lados com `probe_build` e a constante escrita à mão, e
/// por isso media a **LEI** — que continuava verdadeira — em vez da cena. *Um gate sobre uma
/// cena tem de cozinhar a cena;* aqui o lado guiado sai do grafo real e o oráculo lê os
/// params **do mesmo grafo**, para que a única coisa escrita à mão seja a gramática que a
/// coluna 4 usa.
#[test]
fn the_guided_plant_draws_exactly_what_the_factory_grammar_draws() {
    let k = PLANTS
        .iter()
        .position(|p| p.guided)
        .expect("ha' uma guiada");
    let (axiom, rules) = (PLANTS[k].axiom, PLANTS[k].rules);

    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registam");
    let mut doc = MotionDoc::default();
    build_lsystem_demo_document(&mut doc, &reg).expect("a cena monta");
    let node = lsystem_nodes(&doc.graph)[k];

    // O lado GUIADO é a cena, cozida como o app a coze.
    let mut cook = Cook::new();
    let guided = match &cook.cook(&doc.graph, &reg, node, 0.0).expect("coze")[0] {
        CookValue::Instances(st) => st.clone(),
        other => panic!("esperava instancias, veio {other:?}"),
    };

    // E o ORÁCULO lê os params DO MESMO NÓ — só a gramática é que vem de fora.
    let over = doc
        .graph
        .node_param_overrides(node)
        .cloned()
        .unwrap_or_default();
    let gens = over
        .get(ls::param::GENERATIONS)
        .copied()
        .expect("a cena poe as geracoes");
    let mut authored: Vec<(&str, f32)> = vec![(ls::param::MODE, ls::MODE_GRAMMAR as f32)];
    for name in [
        ls::param::SEED,
        ls::param::TROPISM,
        ls::param::ANGLE,
        ls::param::WIDTH,
        ls::param::STEP,
        ls::param::LENGTH_SCALE,
    ] {
        if let Some(v) = over.get(name) {
            authored.push((name, *v));
        }
    }
    let oracle = ls::probe_build(axiom, rules, gens, &authored);

    assert_eq!(
        guided.count(),
        oracle.count(),
        "as duas contagens tem de bater: {} contra {}",
        guided.count(),
        oracle.count()
    );
    let p = |s: &ph2d_nodegraph::attr::Stream| match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("P"),
    };
    for (i, (u, w)) in p(&guided).iter().zip(p(&oracle).iter()).enumerate() {
        assert_eq!(
            u.map(f32::to_bits),
            w.map(f32::to_bits),
            "o elemento {i} difere: {u:?} contra {w:?} — a coluna guiada da cena deixou de \
             exprimir a gramatica de fabrica"
        );
    }
}
