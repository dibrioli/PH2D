//! ⭐⭐ **A AUDITORIA DO GRUPO DA SIMULAÇÃO** (ciclo 5, passo 2 — doc 103 §5).
//!
//! *«Deixar a física decidir»*: até aqui o artista disse **onde** cada coisa está, **como** ela
//! anda e **quem** é afectado. Este grupo entrega a última decisão a uma lei — e o que ele pede
//! em troca é um **relógio**.
//!
//! ⚠️ **O grupo é meio LISTA e meio FAMÍLIA**, e a metade família é derivada do registry: uma
//! `force.*` que nasça amanhã entra na auditoria sozinha. *Uma lista escrita à mão envelhece em
//! silêncio no dia em que a família cresce.*
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture audit_the_sim_group
//! ```

/// A metade escrita: os nós do ciclo que **não** são uma família.
const NOMEADOS: [&str; 6] = [
    "sim.zone",
    "sim.spawn",
    "sim.step",
    "sim.lifetime",
    "sim.collide",
    "motion.integrate",
];

/// O grupo inteiro — os nomeados mais **toda** a família `force.*`, pedida ao registry.
pub fn grupo() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = NOMEADOS.to_vec();
    v.extend(crate::motion_ciclo_probe::familia("force."));
    v
}

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn audit_the_sim_group() {
    crate::motion_ciclo_probe::retrato(&grupo());
}

/// **OS PARAMS DE CADA NÓ DO GRUPO** — o que a auditoria compara contra as referências.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_each_sim_node_offers() {
    crate::motion_ciclo_probe::params_de(&grupo());
}

/// **O QUE O CARTÃO PINTA**, com o rótulo da tela.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_the_sim_card_shows() {
    crate::motion_ciclo_probe::cartao(&grupo());
}

/// **OS NOMES dos cartões** — o que o tutorial terá de escrever.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_sim_card_names() {
    crate::motion_ciclo_probe::nomes(&grupo());
}

/// **O VOCABULÁRIO do grupo** — dois nós que guardam a mesma pergunta chamam-lhe o mesmo nome?
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_sim_vocabulary() {
    crate::motion_ciclo_probe::vocabulario(&grupo());
}

/// ⚠️ **A FAMÍLIA `force.*` NÃO ESTÁ VAZIA, e o grupo cresce com ela.**
///
/// ⛔ Sem este piso, um `familia("force.")` que devolvesse nada (um prefixo mal escrito, um
/// registo que mudou de nome) deixaria as cinco sondas acima a auditar **seis** nós em vez de
/// doze — e todas passariam, caladas.
#[test]
fn the_force_family_is_derived_and_not_empty() {
    let forcas = crate::motion_ciclo_probe::familia("force.");
    assert!(
        forcas.len() >= 6,
        "so' {} no(s) `force.*` -- o prefixo ou o registo mudaram: {forcas:?}",
        forcas.len()
    );
    let g = grupo();
    assert_eq!(
        g.len(),
        NOMEADOS.len() + forcas.len(),
        "o grupo tem de ser os nomeados MAIS a familia inteira"
    );
}

/// ⭐⭐⭐ **QUEM DESTE GRUPO OCUPA LUGAR NO MUNDO E NÃO TEM ALÇA?**
///
/// ⚠️ **É a mesma pergunta que abriu o ciclo 4**, e ela vale por grupo: um nó com `center_x` e
/// `center_y` é uma coisa **posta** algures, e pô-la com dois sliders é caçar a posição. A régua
/// é derivada — quem declara o vocabulário espacial contra quem o
/// [`crate::field_gizmo::spec_for`] conhece.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture which_sim_nodes_have_a_place
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn which_sim_nodes_have_a_place() {
    use ph2d_nodegraph::cook::OpResolver;
    let m = crate::motion_state::MotionState::new();
    eprintln!("\n  nó                        | centro | raio | ângulo | alça de canvas");
    eprintln!("  --------------------------|--------|------|--------|---------------");
    for nome in grupo() {
        let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
        let Some(op) = m.registry.resolve(tid) else {
            continue;
        };
        let tem = |p: &str| op.manifest().params.iter().any(|s| s.name == p);
        let centro = tem("center_x") && tem("center_y");
        if !centro {
            continue;
        }
        eprintln!(
            "  {nome:<26} |  sim   | {:^4} | {:^6} | {}",
            if tem("radius") { "sim" } else { "—" },
            if tem("angle") || tem("rotation") {
                "sim"
            } else {
                "—"
            },
            if crate::field_gizmo::spec_for(tid, &|_| 0.0).is_some() {
                "sim"
            } else {
                "⛔ NAO"
            }
        );
    }
    eprintln!();
}

// ---------------------------------------------------------------------------------------------
// W3 — as SEÇÕES do grupo, e o piso que a própria casa desenhou.
// ---------------------------------------------------------------------------------------------

/// **O menor cartão que esta casa alguma vez julgou valer uma secção**, em params.
///
/// ⛔ **Não é um número escolhido — é MEDIDO**, e o teste abaixo volta a medi-lo a cada
/// corrida. Ele saiu de `field.box`, a wave do ciclo 4, e é o piso do vale entre os dois
/// lados: abaixo dele a casa mantém tudo em fila (a lei já escrita no `field.box`: *numa
/// carta de 3, 4 ou 6 rows dois cabeçalhos organizam menos do que ocupam*, porque
/// `band_len = params + secções` e uma secção aberta custa **+1 fileira**).
const SECTION_FLOOR: usize = 9;

/// ⭐⭐⭐ **NENHUM CARTÃO GRANDE DESTE GRUPO É UMA PAREDE DE SLIDERS.**
///
/// A pergunta do ciclo 4 (o `field.box` a pintar nove rows em fila ao lado de um irmão
/// idêntico que já as agrupava) vale por grupo, e aqui ela acusava **três**:
/// `force.attractor` (11) · `force.curl` (11) · `force.wind` (12), com o `sim.collide` (15)
/// já arrumado desde a folha 13.
///
/// ## ⛔ A catraca tem CENSO DE OBSOLESCÊNCIA, e é a primeira metade do teste
///
/// *Uma catraca sem censo não desce: ela vira licença* (`CLAUDE.md` §5.0). Aqui o censo é o
/// próprio piso: [`SECTION_FLOOR`] tem de continuar a ser **o menor cartão com secções de
/// todo o registo**. No dia em que outra linha agrupar um cartão mais pequeno, a casa terá
/// baixado a própria régua — e este gate diz isso em voz alta, em vez de derivar em silêncio
/// e acender o grupo inteiro sem explicação.
#[test]
fn no_big_card_in_this_group_is_a_wall_of_sliders() {
    use ph2d_nodegraph::cook::OpResolver;
    let m = crate::motion_state::MotionState::new();

    // 1. O CENSO: o piso ainda descreve a casa?
    let menor = m
        .registry
        .manifests()
        .filter(|man| !man.params.is_empty())
        .filter(|man| !m.registry.param_groups(man.id).is_empty())
        .map(|man| man.params.len())
        .min()
        .expect("a casa tem pelo menos um no com seccoes");
    assert_eq!(
        menor, SECTION_FLOOR,
        "o menor cartao COM seccoes da casa passou a ter {menor} params -- a casa mexeu na \
         propria regua, entao o piso deste gate tem de ser re-medido e a decisao escrita ao \
         lado (doc 108 W3), nunca ajustada em silencio"
    );

    // 2. A LEI: no grupo, quem chega ao piso tem secções.
    let paredes: Vec<(&str, usize)> = grupo()
        .into_iter()
        .filter_map(|nome| {
            let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
            let op = m.registry.resolve(tid)?;
            let n = op.manifest().params.len();
            (n >= SECTION_FLOOR && m.registry.param_groups(tid).is_empty()).then_some((nome, n))
        })
        .collect();
    assert!(
        paredes.is_empty(),
        "estes nos do grupo pintam {SECTION_FLOOR}+ params em fila, sem uma unica seccao -- \
         o artista tem de ler todos para achar um: {paredes:?}"
    );
}

/// ⚠️ **E o inverso: uma secção que o cartão nunca pinta é pior que nenhuma.**
///
/// Cada linha de [`ph2d_node_registry::ParamGroup`] nomeia um param, e um nome que o
/// manifesto não declara (uma chave renomeada, um param apagado) fica **muda** — a secção
/// existe na tabela e nunca aparece, e nada no ecrã o diz.
#[test]
fn every_section_of_this_group_names_a_param_that_exists() {
    use ph2d_nodegraph::cook::OpResolver;
    let m = crate::motion_state::MotionState::new();
    let mut orfaos: Vec<String> = Vec::new();
    for nome in grupo() {
        let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
        let Some(op) = m.registry.resolve(tid) else {
            continue;
        };
        let man = op.manifest();
        for g in m.registry.param_groups(tid) {
            if !man.params.iter().any(|p| p.name == g.param) {
                orfaos.push(format!("{nome}::{} (seccao `{}`)", g.param, g.group));
            }
        }
    }
    assert!(
        orfaos.is_empty(),
        "estas linhas de seccao nomeiam params que o manifesto nao tem, logo nunca pintam: \
         {orfaos:?}"
    );
}

/// **O QUE OS CARTÕES DA CENA `=113` MOSTRAM** — a fonte dos nomes que o tutorial escreve.
///
/// ⚠️ **Não é o mesmo que [`what_the_sim_card_shows`]:** ali cada nó está nos DEFAULTS dele;
/// aqui está como a cena o autora, e o `sim.collide` da cena é um `Box` — logo o cartão mostra
/// `Box Width`/`Box Height` onde o default mostraria `Radius`. *Um tutorial nomeia o que o dono
/// vê, e o que ele vê é a CENA.*
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_the_sim_scene_cards_show() {
    let mut m = crate::motion_state::MotionState::new();
    let _ = crate::motion_demo_legend::monta("113", &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor::ProjectSettings::default(),
        &mut snap,
    );
    for v in &snap.nodes {
        let rows: Vec<&str> = v.params.iter().map(|c| c.hint.label).collect();
        eprintln!("  {:<18} | {}", v.display_name, rows.join(" · "));
        if !v.sections.is_empty() {
            let secs: Vec<String> = v
                .sections
                .iter()
                .map(|s| format!("{}@{}", s.title, s.at))
                .collect();
            eprintln!("  {:<18} > secções: {}", "", secs.join(" · "));
        }
    }
}

/// A FONTE do tutorial deste ciclo — lida para que os nomes do gate e os do texto não possam
/// divergir em silêncio.
const TUTORIAL: &str = include_str!("../../../docs/Motion Nodes/tutoriais/src/05_simulacao.html");

/// ⭐⭐⭐ **CADA PASSO DO TUTORIAL É POSSÍVEL NO APP** (ciclo 5, passo 7 — doc 103 §1).
///
/// ⛔⛔ **Um passo que manda clicar numa linha AFIRMA que ela está na lista**, e a casa já pagou
/// por escrever um passo impossível ([memória](../../project-memory/feedback_a_smoke_step_that_names_a_panel_row_must_prove_the_row_is_in_the_list.md)).
/// No ciclo 4 esta mesma régua apanhou **seis** erros meus antes do dono os ver.
///
/// ⚠️ **Ela corre sobre a CENA e não sobre os defaults**, e neste ciclo a diferença morde: o
/// `sim.collide` da `=113` é um `Box`, logo o cartão mostra `Box Width`/`Box Height` onde o
/// default mostraria `Radius`. *O dono vê a cena.*
#[test]
fn every_row_the_sim_tutorial_names_is_on_the_card() {
    let mut m = crate::motion_state::MotionState::new();
    let _ = crate::motion_demo_legend::monta("113", &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor::ProjectSettings::default(),
        &mut snap,
    );

    // O que o tutorial manda procurar: (título do cartão, linhas dentro dele).
    let pedidos: &[(&str, &[&str])] = &[
        (
            "Collider",
            &[
                "Shape",
                "Center X",
                "Center Y",
                "Box Width",
                "Box Height",
                "Angle",
            ],
        ),
        ("Wind", &["Angle", "Strength", "Acts As", "Gust"]),
        ("Simulation Zone", &["Life Cycle"]),
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
        // ⚠️ **E o título tem de estar de facto NO TUTORIAL** — senão esta lista é uma promessa
        // sobre um texto que não a faz, e ela envelhece calada.
        assert!(
            TUTORIAL.contains(titulo),
            "o gate defende o cartao `{titulo}` e o tutorial nunca o nomeia"
        );
    }

    // ⚠️ **AS SECÇÕES que o tutorial manda ABRIR também são uma afirmação.**
    for (titulo, seccao) in [("Wind", "Gust"), ("Wind", "Timing")] {
        let v = snap
            .nodes
            .iter()
            .find(|v| v.display_name == titulo)
            .expect("o cartao existe (verificado acima)");
        let secs: Vec<&str> = v.sections.iter().map(|s| s.title).collect();
        assert!(
            secs.contains(&seccao),
            "o tutorial manda abrir a seccao `{seccao}` no cartao `{titulo}`, e ele tem {secs:?}"
        );
    }
}

/// ⭐⭐⭐ **O PREÇO DO GRUPO** (ciclo 5, passo 5 — doc 103 §1).
///
/// ⚠️ **As `force.*` são `Pure` e acumulam em `accel`** — sozinhas na cadeia elas não movem um
/// pixel. A tabela mede-as **na cadeia do produto**, `grid → <nó> → output`, que é onde o
/// planeador decide se a coisa fica no dispositivo.
///
/// ⚠️ **Nenhuma leitura desta workstation vale nada acima de `load ~5`** (`CLAUDE.md` §5.0), e é
/// por isso que a porta imprime o `/proc/loadavg` na primeira linha: *uma tabela de relógio sem a
/// carga ao lado não é uma medição, é um número*.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture measure_the_sim_group
/// ```
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn measure_the_sim_group() {
    let lado: f32 = std::env::var("PH2D_LADO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(320.0);
    crate::motion_ciclo_probe::tabela(&grupo(), lado);
}

/// ⭐⭐⭐ **A TABELA DE PREÇO RECUSA-SE A PÔR UM NÚMERO SOBRE UM STREAM VAZIO.**
///
/// ⛔⛔ **Achado na medição deste ciclo:** a régua que decidia *«esta linha leva número?»*
/// perguntava se o nó **DECLARA** precisar de outra porta — e o `sim.zone` e o `sim.spawn` não
/// declaram nenhuma. Resultado: a tabela imprimiu `9,29 ms · 0,38×` e um veredito de
/// dispositivo sobre **zero objectos**. *Um número sobre um stream vazio não é um número
/// pequeno: é a ausência de medição com cara de medição.*
///
/// ⭐ A declaração é um **proxy**; a contagem é o **facto**
/// ([memória](../../project-memory/feedback_a_proxy_predicate_that_becomes_constant_leaves_a_vacuous_assertion.md)).
///
/// ⚠️ **A lei é uma IMPLICAÇÃO, não uma lista:** *emitiu zero ⇒ recusada*. Nomear aqui quais
/// nós vêm vazios pinaria o comportamento de hoje, e o dia em que o `sim.spawn` passasse a
/// emitir nesta cadeia o gate reprovaria sobre uma melhoria.
///
/// ⚠️ **Com os dois CONTROLOS**, senão a implicação é vácua nos dois sentidos: tem de haver no
/// grupo pelo menos um nó que é medido e pelo menos um que é recusado.
#[test]
fn the_price_table_refuses_to_price_an_empty_stream() {
    // Uma grade minúscula: o que se mede aqui é a REGRA, não o relógio.
    const LADO: f32 = 4.0;
    let reg = crate::motion_state::MotionState::new().registry;
    let (mut medidos, mut recusados) = (0usize, 0usize);
    for nome in grupo() {
        let d = crate::motion_ciclo_probe::cook_com(LADO, Some(nome), 1);
        match crate::motion_ciclo_probe::porque_nao_medir(&reg, nome, d.n) {
            Some(_) => recusados += 1,
            None => {
                medidos += 1;
                assert!(
                    d.n > 0,
                    "`{nome}` emitiu ZERO objectos e mesmo assim a tabela ia por-lhe um relogio \
                     e um veredito de dispositivo -- um numero sobre um stream vazio nao e' um \
                     numero pequeno, e' a ausencia de medicao com cara de medicao"
                );
            }
        }
    }
    assert!(
        medidos > 0,
        "nenhum no' do grupo foi dado como mensuravel -- a implicacao ficou vacua"
    );
    assert!(
        recusados > 0,
        "nenhum no' do grupo foi recusado -- se nada e' recusado, esta regra nao esta' a apanhar \
         nada e o controlo dela nao prova coisa nenhuma"
    );
}
