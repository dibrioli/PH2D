//! **As sondas e os portões do ROTEADOR de cenas** — o censo que só existe porque há uma
//! porta por onde um gate consegue MONTAR uma cena e perguntar ([`super::build_level`]).
//!
//! Irmão do `motion_state_demo_router.rs`, cortado pelo teto de 600 LOC da shell e por
//! ASSUNTO: lá mora *que documento o ambiente pediu*, aqui *o que uma cena montada mede*.

use super::*;

#[test]
fn no_conference_scene_ships_a_setup_hole() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let (mut built, mut bad) = (0usize, Vec::new());
    for level in 1..=MAX_DEMO_LEVEL {
        let mut doc = MotionDoc::default();
        if build_level(Some(&level.to_string()), &mut doc, &reg).is_empty() {
            continue;
        }
        built += 1;
        let d = ph2d_motion_diagnose::diagnose(&doc.graph, &reg);
        if !d.is_empty() {
            bad.push(format!("=`{level}`: {d:?}"));
        }
        // ⛔⛔ **A REDE MAIS LARGA, e ela estava a UMA LINHA de distância.** O `diagnose` só
        // conhece `MissingSource` e `MissingInput` — ele é **cego a `UnknownParam`**, que é o
        // `set_param` com o nome errado: um NO-OP silencioso que monta, cozinha, mede o custo
        // certo e entrega outra cena. Medido pela auditoria de 2026-08-27: quatro cenas escreviam
        // `spacing` num `motion.grid` que declara `gap_x`/`gap_y`, e as grades saíam **2× mais
        // largas** do que quem as autorou pediu. *O gate que existia para esta classe varria UMA
        // cena — a `glow_dirt` — e o doc dele dizia cobrir «os nós que ela venha a ganhar».*
        if let Err(e) = doc.graph.validate(&reg) {
            bad.push(format!("=`{level}` VALIDATE: {e:?}"));
        }
    }
    assert!(
        bad.is_empty(),
        "cenas com buraco de setup:\n{}",
        bad.join("\n")
    );
    // ⚠️ CONTROLE: sem isto o portão passa por VÁCUO no dia em que o
    // `build_level` deixar de montar (um `_ =>` que engula tudo, um refactor do
    // env). Uma varredura que não acha nada não prova nada.
    assert!(built >= 60, "controle: a varredura montou só {built} cenas");
}

/// ⛔⛔ **A CURA DAS 107 CENAS NÃO TINHA GATE NENHUM, E A MUTAÇÃO SOBREVIVIA.**
///
/// A família `PH2D_GPU_COOK_DEMO` precisa da ferramenta `motion` em mãos; sem ela as 107 montam,
/// imprimem a legenda e desenham **zero**. A cura vive num `if` em `render_loop/mod.rs` e nada a
/// observava: trocá-lo por `if false` deixava **4500 testes verdes com a contagem idêntica**
/// (auditoria de 2026-08-27).
///
/// ⚠️ **A régua é o CALL SITE, porque é lá que a cura mora** — a rota do `ToolRegistry` real
/// atravessa uma janela e não é alcançável daqui. Um censo textual é fraco como oráculo e é
/// **exacto** contra esta mutação: apagar a chamada apaga o nome.
#[test]
fn the_demo_family_takes_the_motion_tool_and_the_call_site_says_so() {
    let src = include_str!("render_loop/mod.rs");
    for needle in [
        "demo_wants_the_motion_tool(",
        "ToolId::new(\"motion\")",
        "demo_tool_forced",
    ] {
        assert!(
            src.contains(needle),
            "o `render_loop` deixou de {needle:?} — as 107 cenas voltam a desenhar NADA, \
             e nenhum outro gate deste repo o diria"
        );
    }
}

/// **A LEI DO PREDICADO: a ferramenta é tomada quando há CENA, não quando há variável.**
///
/// ⚠️ `PH2D_GPU_COOK_DEMO=0` cai no braço `_ => Vec::new()` do roteador. Com a 1.ª versão do
/// predicado (`var_os(..).is_some()`) ele trocava o app para o Motion **com a tela vazia** — e
/// contradizia o `gpu_enabled_from_env` do mesmo ficheiro, onde `0` significa **desligar**.
#[test]
fn the_tool_is_taken_only_when_the_router_actually_built_a_scene() {
    use super::wants_the_motion_tool as wants;
    assert!(
        wants(true, 1),
        "env posta e cena montada: toma a ferramenta"
    );
    assert!(
        !wants(true, 0),
        "`=0` (ou lixo) monta ZERO sinks — tomar o canvas ali da' uma tela em branco"
    );
    assert!(!wants(false, 2), "sem a env, o documento e' do artista");
}

/// **`MAX_DEMO_LEVEL` TEM DE SER O MAIOR BRAÇO DO `match`, E O NÚMERO CONTA-SE DO FONTE.**
///
/// ⚠️ **Baixá-lo a `106` sobrevivia a 4500 testes** (auditoria de 2026-08-27): as três travessias
/// que varrem as cenas param nele, então a cena nova deixa de ser diagnosticada **e o controle
/// não pisca** — ele é `built >= 60` sobre `built = 107`, ou seja **47 de folga**.
///
/// ⚠️ E o gate que o §5 do `CLAUDE.md` diz cobrir esta família — `no_two_smoke_scenes_claim_the
/// _same_level` — lê o `build_smoke_router.rs`, que é o roteador **VETORIAL**. Ele nunca leu este
/// ficheiro, em nenhum dia. *Um ponteiro para um gate não é o gate.*
#[test]
fn the_sweep_ceiling_is_the_highest_arm_the_router_actually_has() {
    let src = include_str!("motion_state_demo_router.rs");
    let highest = src
        .match_indices("Some(\"")
        .filter_map(|(i, _)| {
            let rest = &src[i + 6..];
            let end = rest.find('"')?;
            rest[..end].parse::<u32>().ok()
        })
        .max()
        .expect("o roteador tem bracos numerados");
    assert_eq!(
        MAX_DEMO_LEVEL, highest,
        "o teto da varredura e o maior braco do `match` discordam — as cenas entre eles \
         existem e nunca sao diagnosticadas"
    );
}

/// ⭐⭐⭐ **O QUE ALIMENTA A PORTA `shape` DE UM CARIMBO É UMA FORMA, EM TODA CENA** — report do
/// Enio, 2026-09-06: *«você colocou grid entrando em Shape de Duplicator! Essa aplicação é
/// correta?»*
///
/// Não era, e o censo irmão respondeu à pergunta seguinte: **três** cenas faziam o mesmo. Curar
/// a que ele apontou e deixar as outras é a armadilha de curar METADE de uma família.
///
/// ⚠️ **Nenhum gate de TIPOS pode apanhar isto:** as duas portas do `motion.duplicator` são o
/// mesmo `Domain::Instances`, então uma grelha ali compila, cozinha e desenha o ladrilho de
/// omissão. O que ela faz de errado é ENSINAR — quem lê o grafo aprende que uma forma se faz com
/// uma grelha de 1×1.
///
/// ⚠️ **A lista é uma CATRACA com as duas metades** (a lei do `CLAUDE.md` §5.0): uma fonte nova
/// entra aqui de propósito, com uma linha a dizer porquê — e uma que já não é usada por cena
/// nenhuma **tem de sair**, senão a lista vira licença.
#[test]
fn every_stamps_shape_port_is_fed_by_a_thing_to_draw() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    /// **O QUE É «uma coisa para desenhar»** — as fontes que trazem APARÊNCIA (geometria, uma
    /// sprite, glifos), e não um ARRANJO, que só traz lugares.
    const FONTES: &[&str] = &[
        // Geometria vectorial viva. O doc dele nomeia esta composição à letra.
        "source.shape",
    ];
    let mut vistas: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut maus: Vec<String> = Vec::new();
    let mut carimbos = 0usize;
    for level in 1..=MAX_DEMO_LEVEL {
        let mut doc = MotionDoc::default();
        if build_level(Some(&level.to_string()), &mut doc, &reg).is_empty() {
            continue;
        }
        let g = &doc.graph;
        for n in g
            .nodes()
            .iter()
            .filter(|n| n.type_name == "motion.duplicator")
        {
            carimbos += 1;
            let Some(e) = g.edges().iter().find(|e| e.to.0 == n.id && e.to.1 == 0) else {
                maus.push(format!("=`{level}`: um carimbo com a porta `shape` VAZIA"));
                continue;
            };
            let raiz = g
                .nodes()
                .iter()
                .find(|m| m.id == shape_origin(g, e.from.0))
                .map(|m| m.type_name.clone())
                .unwrap_or_default();
            vistas.insert(raiz.clone());
            if !FONTES.contains(&raiz.as_str()) {
                maus.push(format!("=`{level}`: a porta `shape` nasce num `{raiz}`"));
            }
        }
    }
    assert!(
        carimbos >= 15,
        "controle: a varredura achou {carimbos} carimbos"
    );
    assert!(
        maus.is_empty(),
        "uma GRELHA (ou outro arranjo) alimenta a porta de FORMA de um carimbo — ela desenha o \
         ladrilho de omissao e ensina o idioma errado a quem le^ o grafo:\n{}",
        maus.join("\n")
    );
    // A metade da obsolescência: uma fonte declarada que nenhuma cena usa é licença parada.
    let orfas: Vec<&&str> = FONTES.iter().filter(|f| !vistas.contains(**f)).collect();
    assert!(
        orfas.is_empty(),
        "a lista declara fontes que cena nenhuma usa — apague-as: {orfas:?}"
    );
}

/// Sobe pela porta 0 até um nó SEM entradas — a fonte daquele braço. Entre a fonte e o carimbo
/// há transformes e tints, então olhar o vizinho imediato não responderia.
pub(super) fn shape_origin(
    g: &ph2d_nodegraph::graph::Graph,
    mut id: ph2d_nodegraph::graph::NodeId,
) -> ph2d_nodegraph::graph::NodeId {
    for _ in 0..32 {
        match g.edges().iter().find(|e| e.to.0 == id && e.to.1 == 0) {
            Some(e) => id = e.from.0,
            None => break,
        }
    }
    id
}
