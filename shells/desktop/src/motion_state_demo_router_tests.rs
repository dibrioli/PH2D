//! **As sondas e os portões do ROTEADOR de cenas** — o censo que só existe porque há uma
//! porta por onde um gate consegue MONTAR uma cena e perguntar ([`super::build_level`]).
//!
//! Irmão do `motion_state_demo_router.rs`, cortado pelo teto de 600 LOC da shell e por
//! ASSUNTO: lá mora *que documento o ambiente pediu*, aqui *o que uma cena montada mede*.

use super::*;

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
/// `PH2D_LAYOUT_LEVEL=73 cargo test -p ph2d-host-desktop --bins
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
        let sinks = build_level(Some(&level.to_string()), &mut state.doc, &state.registry);
        if sinks.is_empty() {
            continue;
        }
        crate::render_loop::motion_externals::publish_all(&mut state, 0.0);
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
/// `PH2D_LAYOUT_LEVEL=75 cargo test -p ph2d-host-desktop --bins
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
        let sinks = build_level(Some(level), &mut doc, &reg);
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
    assert!(wants(true, 1), "env posta e cena montada: toma a ferramenta");
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
