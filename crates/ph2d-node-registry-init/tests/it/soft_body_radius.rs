//! **O CORPO MOLE TEM RAIO DE PARTÍCULA — a célula dizia *"não rodei"*, e agora rodou.**
//!
//! O gêmeo exacto do [`rope_thickness`]. A folha 03 do doc 89 fecha a célula do
//! `motion.soft_body` com uma frase e nenhum número:
//!
//! > *"**A MESMA cadeia do rope** (`soft_body.out --pre--> motion.collide → soft_body.state`) —
//! > `motion.collide` é Pure e repassa `sb_vel`/`sim_t`. **Não rodei**"*
//!
//! ⚠️ **Uma célula que diz *"não rodei"* não é uma omissão nem uma recusa: é uma pergunta
//! sem resposta**, e o veredito dela (`omissão/ergonomia — o teste decide`) estava escrito à
//! espera de uma medição. A cadeia corre e o corpo continua a simular. ⛔ O ciclo 14 leu o corpo a
//! INCHAR (`1,92 → 2,76`) como *«o que um particle radius faz»*, e o oráculo refutou-o: era o
//! colisor a empurrar os vizinhos de malha contra o corpo (Bug #15, doc 124 §2). A lei de hoje:
//! os pares que o repouso sobrepõe não colidem, o contacto entra na velocidade, o laço sub-tica.
//!
//! ⇒ **não falta capacidade, falta o GESTO** — hoje são dois nós e duas arestas ligadas à mão,
//! exactamente como na corda. É a mesma conclusão e pelo mesmo mecanismo.
//!
//! ⚠️ **A régua NÃO é a distância entre vizinhos de malha.** Um corpo mole segura os vizinhos
//! a `spacing` de propósito, e `spacing` é menor que o diâmetro do colisor. A régua são os pares
//! NÃO ligados no repouso (os que o repouso já põe a `≥ 2·raio`), à saída do colisor e no que se
//! desenha — e a extensão, que NÃO pode inchar.
//!
//! ⚠️ **E o `advance_tick` é o que faz a aresta `pre` CARREGAR estado.** Sem ele a cadeia coze
//! o mesmo tique para sempre e as duas versões saem bit a bit idênticas — a armadilha que
//! reprovou duas fixtures do gate da corda antes de ele medir alguma coisa.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

const ROWS: f32 = 5.0;
const COLS: f32 = 5.0;
/// O passo da malha em repouso — e o número contra o qual o raio do colisor é grande.
const SPACING: f32 = 0.30;
/// ⚠️ **O raio é MAIOR que meio passo de propósito:** um colisor que já coubesse entre as
/// partículas não teria nada que empurrar, e o gate ficaria verde por vácuo.
const RADIUS: f32 = 0.28;
const TICKS: usize = 180;
const DT: f64 = 1.0 / 60.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .expect("wire");
}

/// Um corpo mole a balançar, com ou sem o colisor no laço de estado.
///
/// ⚠️ **A porta de estado é a `2`** — as `0`/`1` são `anchor_x`/`anchor_y` e a `3` é a `shape`.
/// Ligar o `pre` na porta errada é aceite pelo `connect` sem uma palavra.
fn body(collide: bool) -> (Graph, NodeId) {
    body_com(collide, 0.6, 1.0)
}

/// O corpo com a rigidez e os sub-passos (o param do corpo, lido pelo SEQUENCIADOR) dados.
fn body_com(collide: bool, stiffness: f32, substeps: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let sb = g.add_node("motion.soft_body");
    g.set_param(sb, "rows", ROWS);
    g.set_param(sb, "cols", COLS);
    g.set_param(sb, "spacing", SPACING);
    g.set_param(sb, "gravity", 9.8);
    g.set_param(sb, "stiffness", stiffness);
    g.set_param(sb, "damping", 0.02);
    g.set_param(sb, "substeps", substeps);

    // A âncora varre de lado: um corpo pendurado e PARADO nunca se dobra sobre si próprio, e
    // foi essa a 1ª fixture reprovada do gate da corda.
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", 0.7);
    g.set_param(lfo, "amplitude", 2.0);
    wire(&mut g, lfo, 0, sb, 0, false);

    if collide {
        let col = g.add_node("motion.collide");
        g.set_param(col, "radius", RADIUS);
        g.set_param(col, "iterations", 8.0);
        // ⚠️ O `pre` mora na aresta que ENTRA no colisor: é ela que quebra o ciclo, e o
        // colisor é `Pure` ⇒ não carimba `sim_t`, então o corpo ainda vê o `dt` do próprio
        // relógio no tique seguinte.
        wire(&mut g, sb, 0, col, 0, true);
        wire(&mut g, col, 0, sb, 2, false);
    } else {
        wire(&mut g, sb, 0, sb, 2, true);
    }
    (g, sb)
}

/// A pose no fim de `TICKS`.
fn settle(g: &Graph, reg: &NodeRegistry, node: NodeId) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let mut last = Stream::new(0);
    for k in 0..TICKS {
        let playhead = k as f64 * DT;
        let out = cook.cook(g, reg, node, playhead).expect("o corpo coze");
        let CookValue::Instances(s) = &out[0] else {
            panic!("a saida e um stream")
        };
        last = s.clone();
        // A ordem do produto: cozer e DEPOIS avançar o tique (doc 123 §4).
        cook.advance_tick(g, reg, playhead)
            .expect("o tique avanca o `pre`");
    }
    match last.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// A maior distância entre duas partículas — o quanto o corpo ocupa.
fn extent(p: &[[f32; 2]]) -> f32 {
    let mut d = 0.0_f32;
    for a in p {
        for b in p {
            d = d.max((a[0] - b[0]).hypot(a[1] - b[1]));
        }
    }
    d
}

/// ⭐⭐ **O colisor segura o diâmetro e o corpo NÃO incha** — o Bug #15 fechado pela lei do
/// padrão-ouro (doc 124 §2): os pares que o repouso já sobrepõe não colidem (o filtro do FleX), o
/// contacto entra na velocidade do corpo, e o laço sub-tica (`substeps = 4`, o param do corpo,
/// marchado pela porta do pump).
///
/// ⚠️ **A régua é a do ORÁCULO** (`ferramentas/godot_oracle_corpo_mole/`, Godot `4.7.2`, MIT, a
/// MESMA fixtura): com a lei, o pior par não ligado fica em `0,5563` (o diâmetro menos a folga do
/// solver) e a extensão é a do corpo nu (`1,81 → 1,80`); colidir também os ligados reproduz o
/// `2,76` que o ciclo 14 leu como *«o que um particle radius faz»* — era a assinatura do defeito.
#[test]
fn the_collider_holds_the_diameter_and_the_body_does_not_swell() {
    const STIFFNESS: f32 = 0.3;
    const SUBSTEPS: f32 = 4.0;
    // O número do ORÁCULO, lido da saída gravada (`corre.sh` regrava-a): o pior não-ligado na 2.ª
    // metade com a lei. Correr o oráculo de novo é mudar este gate.
    let oraculo: f32 =
        include_str!("../../../../docs/Motion Nodes/ferramentas/godot_oracle_corpo_mole/saida.txt")
            .lines()
            .find(|l| l.contains("modo=lei "))
            .and_then(|l| {
                l.split_whitespace()
                    .find_map(|c| c.strip_prefix("pior_nao_ligado_2a_metade="))
            })
            .and_then(|v| v.parse().ok())
            .expect("a saida do oraculo traz o modo `lei`");
    let reg = registry();
    let (g0, sb0) = body_com(false, STIFFNESS, SUBSTEPS);
    let (g1, sb1) = body_com(true, STIFFNESS, SUBSTEPS);
    let [_, nu_corpo, nu_ligado, nu_ext] = reguas_com_sub_passos(&g0, &reg, sb0, None, 0);
    let [colisor, corpo, ligado, ext] =
        reguas_com_sub_passos(&g1, &reg, sb1, Some(NodeId(sb1.0 + 2)), 0);
    println!(
        "Bug #15 · nu: corpo {nu_corpo:.4} ligado {nu_ligado:.4} ext {nu_ext:.4} · com colisor: \
         colisor {colisor:.4} corpo {corpo:.4} ligado {ligado:.4} ext {ext:.4}"
    );
    let d = 2.0 * RADIUS;
    // O CONTROLO: sem o colisor a fixtura CONTÉM o fenómeno — o corpo atravessa-se.
    assert!(
        nu_corpo < d * 0.95,
        "CONTROLO: o corpo nu tem de se atravessar ({nu_corpo:.4})"
    );
    assert!(
        colisor >= oraculo,
        "a saida do colisor segura o diametro: {colisor:.4} < {oraculo}"
    );
    assert!(
        corpo >= d * 0.9,
        "o corpo desenhado nao se atravessa: {corpo:.4} < {:.4}",
        d * 0.9
    );
    assert!(
        ligado >= nu_ligado * 0.95,
        "o colisor nao esmaga a malha: ligado {ligado:.4} contra {nu_ligado:.4} no corpo nu"
    );
    assert!(
        (ext - nu_ext).abs() <= nu_ext * 0.05,
        "o corpo NAO incha: extensao {ext:.4} contra {nu_ext:.4} no corpo nu"
    );
}

/// Os pares que o REPOUSO já põe a menos de `2·RADIUS` — os quatro vizinhos e as diagonais da
/// malha (`0,30` e `0,42` contra o diâmetro `0,56`). É a mesma definição que o oráculo usa
/// (`docs/Motion Nodes/ferramentas/godot_oracle_corpo_mole/`) e que o colisor passa a filtrar.
fn ligados_no_repouso(i: usize, j: usize) -> bool {
    let (c, r) = (COLS as usize, |k: usize| k / COLS as usize);
    let dx = (i % c) as f32 - (j % c) as f32;
    let dy = r(i) as f32 - r(j) as f32;
    (dx * dx + dy * dy).sqrt() * SPACING < 2.0 * RADIUS
}

/// O pior par NÃO ligado no repouso, e o pior ligado.
fn pior_par(p: &[[f32; 2]]) -> (f32, f32) {
    let (mut solto, mut ligado) = (f32::MAX, f32::MAX);
    for i in 0..p.len() {
        for j in (i + 1)..p.len() {
            let d = (p[i][0] - p[j][0]).hypot(p[i][1] - p[j][1]);
            if ligados_no_repouso(i, j) {
                ligado = ligado.min(d);
            } else {
                solto = solto.min(d);
            }
        }
    }
    (solto, ligado)
}

fn posicoes(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// ⭐ **A régua do Bug #15, nas DUAS saídas** — a do colisor (o que ele promete) e a do corpo (o que
/// se desenha) —, tique a tique, na ordem do produto.
#[test]
#[ignore = "sonda: o Bug #15 medido — corra à mão com --nocapture"]
fn sonda_o_colisor_no_laco_do_corpo_mole() {
    let reg = registry();
    let (g0, sb0) = body(false);
    let (g1, sb1) = body(true);
    let col = NodeId(sb1.0 + 2);
    assert_eq!(
        g1.nodes()
            .iter()
            .find(|n| n.id == col)
            .map(|n| n.type_name.as_str()),
        Some("motion.collide"),
        "o terceiro no' do grafo com colisor e' o colisor"
    );
    let bare = settle(&g0, &reg, sb0);
    let mut cook = Cook::new();
    let (mut pior_col, mut pior_corpo) = (f32::MAX, f32::MAX);
    let (mut fim_col, mut fim_corpo, mut fim_lig, mut fim) = (0.0, 0.0, 0.0, Vec::new());
    for k in 0..TICKS {
        let playhead = k as f64 * DT;
        let out = cook.cook(&g1, &reg, sb1, playhead).expect("o corpo coze");
        let CookValue::Instances(s) = &out[0] else {
            panic!("stream")
        };
        let corpo = posicoes(s);
        let colisor = match cook.peek(col) {
            Some([CookValue::Instances(c), ..]) => posicoes(c),
            _ => panic!("o colisor cozeu neste tique"),
        };
        let (pc, _) = pior_par(&colisor);
        let (pb, lig) = pior_par(&corpo);
        if k >= TICKS / 2 {
            pior_col = pior_col.min(pc);
            pior_corpo = pior_corpo.min(pb);
        }
        (fim_col, fim_corpo, fim_lig, fim) = (pc, pb, lig, corpo);
        cook.advance_tick(&g1, &reg, playhead).expect("avanca");
    }
    println!(
        "BUG15 diametro {:.4} · SAIDA DO COLISOR pior nao-ligado fim {fim_col:.4} / 2a metade \
         {pior_col:.4} · SAIDA DO CORPO pior nao-ligado fim {fim_corpo:.4} / 2a metade \
         {pior_corpo:.4} · ligado min fim {fim_lig:.4} (passo {SPACING}) · extensao nu {:.4} -> \
         com colisor {:.4}",
        2.0 * RADIUS,
        extent(&bare),
        extent(&fim)
    );
}

/// **A porta do PRODUTO**: o pump pergunta as ilhas ao grafo (`substep_islands`) e sub-passa cada
/// uma ANTES do cozimento do quadro (`ph2d_eval_motion::substep_zones`). `forcar > 0` impõe o ritmo
/// à mão (a sonda das variantes); `0` usa o que o grafo declara.
fn sub_passa(cook: &mut Cook, g: &Graph, reg: &NodeRegistry, playhead: f64, forcar: u32) {
    let Some(antes) = cook.prev_playhead() else {
        return;
    };
    for ilha in ph2d_nodegraph::cook::substep_islands(g, reg) {
        let n = if forcar > 0 { forcar } else { ilha.substeps };
        cook.substep(g, reg, ilha.root, antes, playhead, n)
            .expect("sub-passos");
    }
}

/// As réguas do Bug #15 com `n` sub-passos por quadro, pela porta do produto (`Cook::substep`,
/// a que o pump corre antes do cozimento do quadro): `(colisor 2.ª metade, corpo 2.ª metade,
/// ligado mín. na 2.ª metade, extensão no fim)`.
fn reguas_com_sub_passos(
    g: &Graph,
    reg: &NodeRegistry,
    sb: NodeId,
    col: Option<NodeId>,
    n: u32,
) -> [f32; 4] {
    let mut cook = Cook::new();
    let (mut pc, mut pb, mut pl, mut fim) = (f32::MAX, f32::MAX, f32::MAX, Vec::new());
    for k in 0..TICKS {
        let playhead = k as f64 * DT;
        sub_passa(&mut cook, g, reg, playhead, n);
        let out = cook.cook(g, reg, sb, playhead).expect("coze");
        let CookValue::Instances(s) = &out[0] else {
            panic!("stream")
        };
        let corpo = posicoes(s);
        if k >= TICKS / 2 {
            let (b, l) = pior_par(&corpo);
            pb = pb.min(b);
            pl = pl.min(l);
            if let Some(col) = col
                && let Some([CookValue::Instances(c), ..]) = cook.peek(col)
            {
                pc = pc.min(pior_par(&posicoes(c)).0);
            }
        }
        fim = corpo;
        cook.advance_tick(g, reg, playhead).expect("avanca");
    }
    [pc, pb, pl, extent(&fim)]
}

/// ⭐ **Sub-passos: o contacto resolvido no tempo** (doc 124 §2) — a fixtura a `1`, `2`, `4` e `8`
/// sub-passos, a duas rigidezes, com e sem colisor. O corpo DECLARA os sub-passos (o param dele) e
/// a marcha é a do pump.
#[test]
#[ignore = "sonda: os sub-passos do Bug #15 — corra à mão com --nocapture"]
fn sonda_o_colisor_com_sub_passos() {
    let reg = registry();
    for stiffness in [0.6f32, 0.3] {
        for n in [1u32, 2, 4, 8] {
            #[expect(
                clippy::cast_precision_loss,
                reason = "um numero pequeno de sub-passos"
            )]
            let (g0, sb0) = body_com(false, stiffness, n as f32);
            #[expect(
                clippy::cast_precision_loss,
                reason = "um numero pequeno de sub-passos"
            )]
            let (g1, sb1) = body_com(true, stiffness, n as f32);
            let nu = reguas_com_sub_passos(&g0, &reg, sb0, None, 0);
            let com = reguas_com_sub_passos(&g1, &reg, sb1, Some(NodeId(sb1.0 + 2)), 0);
            println!(
                "SUBPASSOS stiffness={stiffness} n={n} · nu: corpo {:.4} ligado {:.4} ext {:.4} · \
                 com colisor: colisor {:.4} corpo {:.4} ligado {:.4} ext {:.4}",
                nu[1], nu[2], nu[3], com[0], com[1], com[2], com[3]
            );
        }
    }
}

/// ⚠️ **E o corpo continua a SIMULAR com o colisor no laço** — a metade que o `moved` acima
/// não cobre: uma cadeia pode diferir da nua e ainda assim estar congelada.
#[test]
fn the_body_keeps_simulating_with_the_collider_in_the_loop() {
    let reg = registry();
    let (g, sb) = body(true);
    let mut cook = Cook::new();
    let mut poses: Vec<Vec<[f32; 2]>> = Vec::new();
    for k in 0..TICKS {
        let playhead = k as f64 * DT;
        let out = cook.cook(&g, &reg, sb, playhead).expect("coze");
        let CookValue::Instances(s) = &out[0] else {
            panic!("stream")
        };
        if k == TICKS / 2 || k == TICKS - 1 {
            match s.get("P") {
                Some(Column::Vec2(v)) => poses.push(v.clone()),
                _ => panic!("P"),
            }
        }
        cook.advance_tick(&g, &reg, playhead).expect("avanca");
    }
    assert_eq!(poses.len(), 2, "duas fotografias");
    let d = poses[0]
        .iter()
        .zip(&poses[1])
        .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
        .fold(0.0_f32, f32::max);
    assert!(
        d > 1e-3,
        "o corpo congelou com o colisor no laco (maior desvio entre metade e fim: {d:.6})"
    );
    // E nada explodiu.
    for (i, q) in poses[1].iter().enumerate() {
        assert!(
            q[0].is_finite() && q[1].is_finite() && q[0].abs() < 1e3 && q[1].abs() < 1e3,
            "particula {i} fugiu para {q:?}"
        );
    }
}
