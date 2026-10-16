//! ⭐⭐ **Toda cena de ciclo que ensina APARÊNCIA desenha FORMAS** (Bug #20; `CLAUDE.md` §0.8).
//!
//! Desde a lei do dono de 2026-09-19 (`72cdfa6f71`, `SinkStyle::so_com_forma`) uma corrente sem
//! forma desenha só os marcadores de ponto — e um `Tint`, uma `Color Ramp` ou um `Trail` sobre ela
//! pintam colunas que ninguém desenha. A `=118` (16/09) nasceu três dias antes da lei e ficou a
//! prometer laranja, arco-íris e rastos sobre cruzes azuis até 15/10.
//!
//! A régua é a do PRODUTO: cada cena coze pela porta do quadro (`o_quadro`, rota da CPU) e cada
//! sink cujo cone tem um cartão de aparência tem de sair com aparência ([`tem_aparencia`]: uma
//! geometria ou um ladrilho) — a mesma pergunta que a lei faz antes de desenhar.

use crate::motion_state::MotionState;
use ph2d_eval_motion::tem_aparencia;
use ph2d_nodegraph::graph::NodeId;

/// Os cartões que decidem como uma PEÇA se parece (o grupo do tutorial 7, menos o `Glow`, que é
/// um passe de tela e acende sem peças).
const APARENCIA: &[&str] = &[
    "motion.tint",
    "motion.color_ramp",
    "motion.color_array",
    "motion.trail",
    "motion.strobe",
    "motion.sub_uv",
    "fx.drop_shadow",
    "fx.rgb_split",
];

const QUADROS: u32 = 8;

/// `(sink, cartões de aparência no cone, desenha formas?)` de cada sink da cena `n`.
fn sinks_da_cena(n: &str) -> Vec<(NodeId, Vec<String>, bool)> {
    let mut m = MotionState::new();
    // ⚠️ Pela porta `monta` (a TRAVA da legenda): o `build_level` direto publicava a legenda por
    // cima da de quem varre as fichas em paralelo (`tapa` leu a do `=112` na `=40`, 15/10).
    let _ = crate::motion_demo_legend::monta(n, &mut m.doc, &m.registry);
    sinks_de(m)
}

/// O mesmo, para um documento já montado em `m` — coze `QUADROS` quadros pela porta do quadro e
/// lê cada sink VISÍVEL (o `o_quadro` recalcula-os).
fn sinks_de(mut m: MotionState) -> Vec<(NodeId, Vec<String>, bool)> {
    let mut playhead = ph2d_core::Playhead::new(1.0 / 60.0);
    playhead.play();
    for _ in 0..QUADROS {
        playhead.advance_ticks(1);
        crate::motion_bridge::quadro::o_quadro(
            &mut m,
            None,
            &playhead,
            1.0 / 60.0,
            crate::motion_bridge::quadro::MaoDoEditor::default(),
        );
    }
    m.sinks
        .clone()
        .into_iter()
        .map(|s| {
            let g = &m.doc.graph;
            let efeitos: Vec<String> = ph2d_nodegraph::cook::upstream_cone(g, s)
                .into_iter()
                .filter_map(|k| g.node(k).map(|i| i.type_name.clone()))
                .filter(|t| APARENCIA.contains(&t.as_str()))
                .collect();
            let desenha = m
                .pump
                .cook
                .peek(s)
                .and_then(|v| v.first())
                .is_some_and(|v| tem_aparencia(v.as_stream()));
            (s, efeitos, desenha)
        })
        .collect()
}

/// O CONTROLO das duas réguas: uma cena SÓ DE PONTOS (`grid → output`), montada aqui — a que as
/// `=111`–`=113`, `=116` e quatro panos da `=119` eram até 15/10. Ela tem de ler como «não desenha».
fn so_pontos() -> Vec<(NodeId, Vec<String>, bool)> {
    use ph2d_nodegraph::graph::Edge;
    let mut m = MotionState::new();
    let g = &mut m.doc.graph;
    let grid = g.add_node("motion.grid");
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (grid, 0),
        to: (out, 0),
        delayed: false,
    })
    .expect("grid -> output");
    sinks_de(m)
}

/// ⭐⭐ **O GATE.** ⚠️ Piso de população: pelo menos 10 sinks com aparência em 6 cenas (medido
/// 15/10: `=118`, `=133`–`=136`, `=138`, `=140`, `=142`) — `0 pedidos` lê-se igual a `0 defeitos`.
/// ⚠️ Controlo: a cena só de pontos ([`so_pontos`]) não desenha forma nenhuma — a régua distingue
/// um sink de pontos de um de formas.
#[test]
fn toda_cena_de_ciclo_que_pinta_aparencia_desenha_formas() {
    let mut mudos = Vec::new();
    let (mut sinks_com_efeito, mut cenas_com_efeito) = (0usize, 0usize);
    for n in super::CICLOS {
        let sinks = sinks_da_cena(n);
        assert!(!sinks.is_empty(), "a cena ={n} monta e tem sinks");
        let com: Vec<_> = sinks.iter().filter(|(_, e, _)| !e.is_empty()).collect();
        if !com.is_empty() {
            cenas_com_efeito += 1;
        }
        for (s, efeitos, desenha) in com {
            sinks_com_efeito += 1;
            if !desenha {
                mudos.push(format!("={n} sink {s:?} {efeitos:?}"));
            }
        }
    }
    assert!(
        mudos.is_empty(),
        "sinks de cena de ciclo com cartao de aparencia que desenham so' MARCADORES (sem forma): \
         {mudos:#?}"
    );
    assert!(
        sinks_com_efeito >= 10 && cenas_com_efeito >= 6,
        "piso: so' {sinks_com_efeito} sink(s) com aparencia em {cenas_com_efeito} cena(s) -- a \
         regua varre o vazio"
    );
    let controlo = so_pontos();
    assert!(
        !controlo.is_empty() && controlo.iter().all(|(_, e, d)| e.is_empty() && !d),
        "controlo: a cena so' de pontos nao pinta aparencia: {controlo:?}"
    );
}

/// ⭐⭐⭐ **TODA cena de ciclo desenha FORMAS em TODO sink visível** (`CLAUDE.md` §0.8, Bug #20) — e
/// não só as que têm um cartão de aparência: uma cena de ciclo é o smoke que o dono segue, e um
/// sink só de pontos mostra-lhe cruzinhas azuis (os marcadores) onde a lição devia estar. A
/// varredura de 15/10 achou cinco: `=111`, `=112`, `=113`, `=116` (nenhum sink) e `=119` (4 de 6).
///
/// ⚠️ Piso de população: TODAS as cenas da [`super::CICLOS`] e pelo menos 56 sinks (contados a
/// 15/10) — uma cena que deixasse de montar, ou um `o_quadro` que deixasse de ver sinks, leria
/// «zero mudos». ⚠️ Controlo: a cena só de pontos ([`so_pontos`]) lê como muda.
#[test]
fn toda_cena_de_ciclo_desenha_formas_em_todo_sink_visivel() {
    let mut mudos = Vec::new();
    let (mut sinks_vistos, mut cenas) = (0usize, 0usize);
    for n in super::CICLOS {
        let sinks = sinks_da_cena(n);
        assert!(!sinks.is_empty(), "a cena ={n} monta e tem sinks");
        cenas += 1;
        sinks_vistos += sinks.len();
        let sem: Vec<_> = sinks
            .iter()
            .filter(|(_, _, d)| !d)
            .map(|(s, _, _)| *s)
            .collect();
        if !sem.is_empty() {
            mudos.push(format!(
                "={n}: {} de {} sink(s) {sem:?}",
                sem.len(),
                sinks.len()
            ));
        }
    }
    assert!(
        mudos.is_empty(),
        "cenas de ciclo com sinks que desenham so' MARCADORES (sem forma): {mudos:#?}"
    );
    assert!(
        cenas == super::CICLOS.len() && sinks_vistos >= 56,
        "piso: {sinks_vistos} sink(s) em {cenas} cena(s) -- a regua varre o vazio"
    );
    let controlo = so_pontos();
    assert!(
        !controlo.is_empty() && controlo.iter().all(|(_, _, d)| !d),
        "controlo: a cena so' de pontos tem de ler como muda: {controlo:?}"
    );
}

/// A fonte dos anúncios das cenas `=111`–`=120` (as de `=130` em diante anunciam no ficheiro de
/// cada uma, e entram no gate acima pelo grafo).
const ANUNCIOS: &str = include_str!("motion_state_demo_announce.rs");

/// As cenas cujo ANÚNCIO promete cor — o texto do `eprintln!` dela (do marcador `[cena N]` ao fim
/// da chamada), nunca o ficheiro inteiro, que tem os doc-comments das vizinhas.
fn anuncios_que_prometem_cor() -> Vec<String> {
    let mut fora = Vec::new();
    for n in super::CICLOS {
        let marca = format!("[cena {n}]");
        let Some(i) = ANUNCIOS.find(&marca) else {
            continue;
        };
        let resto = &ANUNCIOS[i..];
        let bloco = resto.find("\n    );").map_or(resto, |j| &resto[..j]);
        // ⚠️ Sem «rasto»: o `arrasto` da `=114` casava.
        if [
            "cor ",
            "COR ",
            "cores",
            "arco-iris",
            "laranja",
            "cauda",
            "CAUDA",
        ]
        .iter()
        .any(|w| bloco.contains(w))
        {
            fora.push((*n).to_string());
        }
    }
    fora
}

/// ⭐⭐ **A metade do TEXTO: toda cena cujo anúncio promete cor desenha formas em TODOS os sinks.**
/// ⚠️ Piso: a `=118` é uma delas — senão a extracção partiu-se e o gate varre o vazio.
#[test]
fn todo_anuncio_que_promete_cor_tem_formas_em_todo_pano() {
    let cenas = anuncios_que_prometem_cor();
    assert!(
        cenas.iter().any(|n| n == "118"),
        "piso: a =118 promete cor e a extraccao nao a viu: {cenas:?}"
    );
    for n in &cenas {
        let sinks = sinks_da_cena(n);
        let sem: Vec<_> = sinks.iter().filter(|(_, _, d)| !d).collect();
        assert!(
            sem.is_empty(),
            "o anuncio da ={n} promete cor e {} de {} pano(s) desenham so' marcadores: {sem:?}",
            sem.len(),
            sinks.len()
        );
    }
}

/// **SONDA — a ROTA de cada cena:** os tipos que o plano do produto deixa na CPU (as fronteiras).
/// `[]` = a placa de ponta a ponta; `["source.shape"]` = híbrida, só a forma na CPU.
/// ⚠️ Lê só o PLANO: as cercas do `cook_gpu` e o `gpu_route` decidem depois dele (a `=113`, a `=120`,
/// a `=134` e a `=139` planeiam com fronteiras e correm TODAS no processador). A rota que o terminal
/// diz é a do `sonda_rota_de_cada_cena_do_roteador` (`motion_state_demo_rota_anunciada_tests.rs`).
#[test]
#[ignore = "sonda: corra à mão"]
fn sonda_rota_das_cenas_de_ciclo() {
    for n in super::CICLOS {
        let mut m = MotionState::new();
        let (sinks, _) = crate::motion_demo_legend::monta(n, &mut m.doc, &m.registry);
        crate::motion_externals::publish_all(&mut m, 0.0);
        let com = rota(&mut m, &sinks);
        // O CONTROLO: a mesma cena com cada carimbo (`Shape → Duplicator`) arrancado — a rota de
        // antes da forma.
        let carimbos: Vec<NodeId> = m
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|i| i.type_name == "motion.duplicator")
            .map(|i| i.id)
            .filter(|d| {
                m.doc.graph.input_edge(*d, 0).is_some_and(|(f, _, _)| {
                    m.doc
                        .graph
                        .node(f)
                        .is_some_and(|i| i.type_name == "source.shape")
                })
            })
            .collect();
        for d in carimbos {
            let g = &mut m.doc.graph;
            let Some((pontos, porta, _)) = g.input_edge(d, 1) else {
                continue;
            };
            let forma = g.input_edge(d, 0).map(|e| e.0);
            let saidas: Vec<_> = g
                .edges()
                .iter()
                .filter(|e| e.from.0 == d)
                .copied()
                .collect();
            for e in saidas {
                g.disconnect(e.to.0, e.to.1);
                let _ = g.connect(ph2d_nodegraph::graph::Edge {
                    from: (pontos, u16::try_from(porta).expect("porta")),
                    ..e
                });
            }
            g.remove_node(d);
            if let Some(f) = forma {
                g.remove_node(f);
            }
        }
        let sem = rota(&mut m, &sinks);
        eprintln!("={n}: na CPU {com:?} · sem carimbo {sem:?}");
    }
}

fn rota(m: &mut MotionState, sinks: &[NodeId]) -> Vec<String> {
    let plano = crate::motion_bridge::gpu::plano_do_produto_para(m, sinks, 0.0);
    let mut fora: Vec<String> = plano
        .boundaries
        .iter()
        .filter_map(|(k, _)| m.doc.graph.node(*k).map(|i| i.type_name.clone()))
        .collect();
    fora.sort();
    fora.dedup();
    fora
}

/// **SONDA — a família inteira:** por cena, quantos sinks, quantos desenham forma, e os cartões de
/// aparência de cada um.
#[test]
#[ignore = "sonda: corra à mão"]
fn sonda_formas_das_cenas_de_ciclo() {
    for n in super::CICLOS {
        let sinks = sinks_da_cena(n);
        let formas = sinks.iter().filter(|(_, _, d)| *d).count();
        let efeitos: Vec<_> = sinks
            .iter()
            .filter(|(_, e, _)| !e.is_empty())
            .map(|(_, e, d)| format!("{}{e:?}", if *d { "" } else { "MUDO " }))
            .collect();
        eprintln!(
            "={n}: {} sinks, {formas} com forma · {efeitos:?}",
            sinks.len()
        );
    }
}
