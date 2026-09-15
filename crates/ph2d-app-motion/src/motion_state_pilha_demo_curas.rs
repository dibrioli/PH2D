//! ⭐⭐⭐ **AS CURAS DO ZUMBIDO, MEDIDAS E REFUTADAS** — as oito cadeias do **doc 109 §8.7–§8.14**,
//! do report do dono de 2026-09-15 (*«vibram muito»* · *«rotacionam como se fossem círculos»*).
//!
//! Irmão do [`super::tremor`] pelo tecto de LOC (HR-18) e por ASSUNTO: lá mede-se **o defeito**
//! (quanto a pilha mexe, e se treme ou gira); aqui mede-se **o que o curaria**, e o veredito de cada
//! candidata.
//!
//! ⚠️⚠️ **Toda sonda daqui corre VÁRIAS REALIZAÇÕES do mesmo monte**, e a razão é o
//! [`probe_o_ruido_entre_realizacoes`]: um monte de 25 quadrados a cair é CAÓTICO, e uma única
//! queda não distingue uma lei de um sorteio. *Sem o piso de ruído, toda varredura de constante lê
//! ruído como tendência* — a armadilha que a `line/quadextract` pagou em cinco realizações da mesma
//! escultura.

use super::tremor::mediana;
use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

/// **SONDA — o RUÍDO entre realizações do mesmo monte.**
///
/// ⚠️⚠️ **Um monte de 25 quadrados a cair é CAÓTICO:** mudar qualquer constante muda o arranjo
/// inteiro, e comparar uma realização com outra é comparar **cenas diferentes**, não leis. Esta
/// sonda mede a dispersão das duas grandezas quando **nada na lei muda** — só o sítio onde as peças
/// nascem, por um epsilon. *Sem este número, qualquer varredura de constantes lê ruído como
/// tendência* (a armadilha que a `line/quadextract` pagou em cinco realizações da mesma escultura).
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_o_ruido_entre_realizacoes -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_ruido_entre_realizacoes() {
    eprintln!("\n  desvio do berço | balanço pior (°/tique) | giro líquido pior (°)");
    eprintln!("  ----------------|------------------------|----------------------");
    let (mut bs, mut gs) = (Vec::new(), Vec::new());
    for k in 0..7 {
        #[expect(clippy::cast_precision_loss, reason = "um indice pequeno")]
        let eps = (k as f32 - 3.0) * 1e-3;
        let (b, g) = realizacao(eps);
        eprintln!("  {eps:>15.4} | {b:>22.4} | {g:>21.2}");
        bs.push(b);
        gs.push(g);
    }
    let faixa = |v: &[f32]| {
        let (lo, hi) = (
            v.iter().copied().fold(f32::MAX, f32::min),
            v.iter().copied().fold(0.0_f32, f32::max),
        );
        (lo, hi, mediana(v))
    };
    let (bl, bh, bm) = faixa(&bs);
    let (gl, gh, gm) = faixa(&gs);
    eprintln!(
        "\n  balanço pior : {bl:.3} .. {bh:.3} (p50 {bm:.3}) — amplitude {:.1}×",
        bh / bl.max(1e-6)
    );
    eprintln!(
        "  giro líquido : {gl:.2} .. {gh:.2} (p50 {gm:.2}) — amplitude {:.1}×",
        gh / gl.max(1e-6)
    );
    eprintln!(
        "\n  ⚠️ toda varredura de constante tem de bater ESTA amplitude para dizer alguma coisa."
    );
}

/// Uma realização do monte com o berço deslocado `eps`: `(balanço pior, giro líquido pior)`.
fn realizacao(eps: f32) -> (f32, f32) {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    // A grelha de CIMA da metade da direita — a que larga as peças.
    let altos: Vec<NodeId> = state
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "motion.transform")
        .map(|n| n.id)
        .collect();
    if let Some(alto) = altos.last() {
        let x = state
            .doc
            .graph
            .node_param_overrides(*alto)
            .and_then(|o| o.get("offset_x").copied())
            .unwrap_or(0.0);
        state.doc.graph.set_param(*alto, "offset_x", x + eps);
    }
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let sink = sinks[1];
    let (mut passos, mut anterior) = (Vec::<Vec<f32>>::new(), Vec::<f32>::new());
    let (mut liquido, mut n) = (Vec::<f32>::new(), 0_usize);
    for k in 0..=174_u64 {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if let Some(Column::Scalar(r)) = s.get("rot") {
            if liquido.len() != r.len() {
                liquido = vec![0.0; r.len()];
                n = r.len();
            }
            if t >= 2.0 && anterior.len() == r.len() {
                passos.push(
                    r.iter()
                        .zip(&anterior)
                        .map(|(a, b)| (a - b).abs())
                        .collect(),
                );
                for i in 0..r.len() {
                    liquido[i] += r[i] - anterior[i];
                }
            }
            anterior = r.clone();
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    let balanco = (0..n)
        .map(|i| mediana(&passos.iter().map(|l| l[i]).collect::<Vec<_>>()))
        .fold(0.0_f32, f32::max);
    (balanco, liquido.iter().fold(0.0_f32, |a, v| a.max(v.abs())))
}

/// **SONDA — a ALTERNÂNCIA ITERADA, testada SEM uma linha de motor** (ordem do dono, 2026-09-15:
/// *«muda o contacto de dono»*; hipótese no doc 109 §8.10).
///
/// ⭐⭐⭐ O zumbido é um ciclo entre **dois conjuntos de restrições** — as peças umas contra as
/// outras (`sim.step`) e as peças contra o mundo (`sim.collide`) — resolvidos **uma vez cada, em
/// sequência**, sem nenhum saber do outro. Projecções alternadas convergem *se a alternância for
/// ITERADA*; uma volta só oscila.
///
/// ⚠️ **E isso exprime-se no GRAFO:** `passo → taça → passo → taça`. O segundo `sim.step` lê
/// `dt = 0` (o primeiro já escreveu o `sim_t` deste tique), logo **não integra** — ele só volta a
/// separar as peças contra as vizinhas depois de a taça as ter mexido. *Se a hipótese estiver
/// certa, o zumbido cai sem que o produto mude uma linha; se não estiver, poupou-se a cirurgia.*
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_a_alternancia_iterada -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_a_alternancia_iterada() {
    eprintln!("\n  voltas de alternância | balanço pior (°/tique) | giro líquido pior (°)");
    eprintln!("  ----------------------|------------------------|----------------------");
    for voltas in [1_usize, 2, 4] {
        let (mut bs, mut gs) = (Vec::new(), Vec::new());
        for k in 0..5 {
            #[expect(clippy::cast_precision_loss, reason = "um indice pequeno")]
            let eps = (k as f32 - 2.0) * 1e-3;
            let (b, g) = realizacao_com_voltas(eps, voltas);
            bs.push(b);
            gs.push(g);
        }
        bs.sort_by(f32::total_cmp);
        gs.sort_by(f32::total_cmp);
        eprintln!(
            "  {voltas:>21} | {:>8.3} .. {:>8.3}      | {:>8.2} .. {:>8.2}",
            bs[0],
            bs[bs.len() - 1],
            gs[0],
            gs[gs.len() - 1]
        );
    }
    eprintln!("\n  ⚠️ o piso de ruído com 1 volta é 3,79..5,69 e 24,30..28,36 (probe_o_ruido_*).");
}

/// Uma realização com a cadeia `passo → taça` repetida `voltas` vezes dentro do mesmo tique.
fn realizacao_com_voltas(eps: f32, voltas: usize) -> (f32, f32) {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    if voltas > 1 {
        encadeia_mais_voltas(&mut state, voltas - 1);
    }
    let altos: Vec<NodeId> = state
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "motion.transform")
        .map(|n| n.id)
        .collect();
    if let Some(alto) = altos.last() {
        let x = state
            .doc
            .graph
            .node_param_overrides(*alto)
            .and_then(|o| o.get("offset_x").copied())
            .unwrap_or(0.0);
        state.doc.graph.set_param(*alto, "offset_x", x + eps);
    }
    crate::motion_shape_gen::publish(&mut state, 0.0);
    mede_balanco_e_giro(&mut state, sinks[1])
}

/// Insere mais `extra` pares `sim.step → sim.collide` entre a taça da DIREITA e a zona dela.
///
/// ⚠️ Os pares novos copiam os params do original — um `sim.collide` com outro obstáculo seria
/// outra cena, e um `sim.step` com outro relógio integraria duas vezes.
fn encadeia_mais_voltas(state: &mut MotionState, extra: usize) {
    use ph2d_nodegraph::graph::Edge;
    let g = &mut state.doc.graph;
    // A taça da DIREITA é a última `sim.collide` do grafo (a metade esquerda não colide).
    let Some(taca) = g
        .nodes()
        .iter()
        .filter(|n| n.type_name == "sim.collide")
        .map(|n| n.id)
        .next_back()
    else {
        return;
    };
    let Some(passo) = g
        .nodes()
        .iter()
        .filter(|n| n.type_name == "sim.step")
        .map(|n| n.id)
        .next_back()
    else {
        return;
    };
    // Onde a taça entrega hoje: a zona, porta 1.
    let Some(destino) = g.edges().iter().find(|e| e.from.0 == taca).map(|e| e.to) else {
        return;
    };
    let params_de = |g: &ph2d_nodegraph::graph::Graph, n: NodeId| -> Vec<(String, f32)> {
        g.node_param_overrides(n)
            .map(|o| o.iter().map(|(k, v)| (k.clone(), *v)).collect())
            .unwrap_or_default()
    };
    let (pp, pt) = (params_de(g, passo), params_de(g, taca));
    g.disconnect(destino.0, destino.1);
    let mut anterior = taca;
    for _ in 0..extra {
        let p2 = g.add_node("sim.step");
        for (k, v) in &pp {
            g.set_param(p2, k, *v);
        }
        let t2 = g.add_node("sim.collide");
        for (k, v) in &pt {
            g.set_param(t2, k, *v);
        }
        let _ = g.connect(Edge {
            from: (anterior, 0),
            to: (p2, 0),
            delayed: false,
        });
        let _ = g.connect(Edge {
            from: (p2, 0),
            to: (t2, 0),
            delayed: false,
        });
        anterior = t2;
    }
    let _ = g.connect(Edge {
        from: (anterior, 0),
        to: destino,
        delayed: false,
    });
}

/// Corre a cena até `2,9 s` e devolve `(balanço pior, giro líquido pior)` da corrente de `sink`.
fn mede_balanco_e_giro(state: &mut MotionState, sink: NodeId) -> (f32, f32) {
    let (mut passos, mut anterior) = (Vec::<Vec<f32>>::new(), Vec::<f32>::new());
    let (mut liquido, mut n) = (Vec::<f32>::new(), 0_usize);
    for k in 0..=174_u64 {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if let Some(Column::Scalar(r)) = s.get("rot") {
            if liquido.len() != r.len() {
                liquido = vec![0.0; r.len()];
                n = r.len();
            }
            if t >= 2.0 && anterior.len() == r.len() {
                passos.push(
                    r.iter()
                        .zip(&anterior)
                        .map(|(a, b)| (a - b).abs())
                        .collect(),
                );
                for i in 0..r.len() {
                    liquido[i] += r[i] - anterior[i];
                }
            }
            anterior = r.clone();
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    let balanco = (0..n)
        .map(|i| mediana(&passos.iter().map(|l| l[i]).collect::<Vec<_>>()))
        .fold(0.0_f32, f32::max);
    (balanco, liquido.iter().fold(0.0_f32, |a, v| a.max(v.abs())))
}

/// **SONDA — as duas alavancas de PRODUTO que nunca foram medidas contra o zumbido:** o ATRITO
/// entre peças (a cena não escreve material, logo elas são gelo) e o ARRASTO ANGULAR do passo
/// (nasce em `1`, que é *sem arrasto*).
///
/// ⚠️ Uma cura de produto que resolva isto vale mais que uma de motor: ela não muda lei nenhuma e
/// não toca em nenhuma cena que já exista.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_as_alavancas_de_produto -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_as_alavancas_de_produto() {
    eprintln!(
        "\n  atrito peças | arrasto angular | balanço pior (°/tique) | giro líquido pior (°)"
    );
    eprintln!("  -------------|-----------------|------------------------|----------------------");
    for (mu, arrasto) in [
        (0.0_f32, 1.0_f32),
        (0.6, 1.0),
        (0.0, 0.9),
        (0.0, 0.6),
        (0.6, 0.6),
    ] {
        let (mut bs, mut gs) = (Vec::new(), Vec::new());
        for k in 0..5 {
            #[expect(clippy::cast_precision_loss, reason = "um indice pequeno")]
            let eps = (k as f32 - 2.0) * 1e-3;
            let (b, g) = realizacao_de_produto(eps, mu, arrasto);
            bs.push(b);
            gs.push(g);
        }
        bs.sort_by(f32::total_cmp);
        gs.sort_by(f32::total_cmp);
        eprintln!(
            "  {mu:>12.2} | {arrasto:>15.2} | {:>8.3} .. {:>8.3}      | {:>8.2} .. {:>8.2}",
            bs[0],
            bs[bs.len() - 1],
            gs[0],
            gs[gs.len() - 1]
        );
    }
    eprintln!("\n  ⚠️ o piso de ruído da cena como shipa é 3,79..5,69 e 24,30..28,36.");
}

/// Uma realização com o atrito das peças e o arrasto angular do passo escritos no cartão.
fn realizacao_de_produto(eps: f32, mu: f32, arrasto: f32) -> (f32, f32) {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    let tipo = |state: &MotionState, t: &str| -> Vec<NodeId> {
        state
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == t)
            .map(|n| n.id)
            .collect()
    };
    if let Some(forma) = tipo(&state, "source.shape").last().copied() {
        state.doc.graph.set_param(forma, param::FRICTION, mu);
    }
    if let Some(passo) = tipo(&state, "sim.step").last().copied() {
        state.doc.graph.set_param(passo, "angular_damping", arrasto);
    }
    if let Some(alto) = tipo(&state, "motion.transform").last().copied() {
        let x = state
            .doc
            .graph
            .node_param_overrides(alto)
            .and_then(|o| o.get("offset_x").copied())
            .unwrap_or(0.0);
        state.doc.graph.set_param(alto, "offset_x", x + eps);
    }
    crate::motion_shape_gen::publish(&mut state, 0.0);
    mede_balanco_e_giro(&mut state, sinks[1])
}

/// **SONDA — quantos contactos do monte são FACE-COM-FACE?** (a pergunta que decide se o manifesto
/// de dois pontos vale para esta cena, doc 109 §8.5.)
///
/// ⚠️⚠️ **Um segundo ponto de apoio só existe onde há um TRECHO**, e um trecho só existe entre duas
/// faces quase paralelas. Entre uma quina e uma face o contacto é um ponto **por geometria**, e
/// nenhuma lei o pode desdobrar. ⇒ *se o monte assentar às três pancadas, o manifesto não tem onde
/// agir, e construí-lo seria curar um caso que esta cena não tem.*
///
/// A régua é o desalinhamento dos eixos das duas caixas, **módulo 90°** (um quadrado tem essa
/// simetria): `0°` = faces paralelas, `45°` = quina contra face.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_quantos_contactos_sao_face_a_face -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_quantos_contactos_sao_face_a_face() {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let sink = sinks[1];
    let mut ultimo = None;
    for k in 0..=174_u64 {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if k == 174 {
            ultimo = Some(s);
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    let s = ultimo.expect("o stream final");
    let cols = ph2d_contact::colisores(&s).expect("a metade da direita declara colisor");
    let p = match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("sem P"),
    };
    let rot = match s.get("rot") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => vec![0.0; p.len()],
    };
    let mut desalinhos = Vec::new();
    for i in 0..p.len() {
        for j in (i + 1)..p.len() {
            let (Some(a), Some(b)) = (cols[i], cols[j]) else {
                continue;
            };
            if ph2d_contact::contato(&a, p[i], &b, p[j], (i + j) % 2 == 0).is_none() {
                continue;
            }
            // O desalinhamento dos eixos, módulo 90°, dobrado para `0..45`.
            let d = (rot[i] - rot[j]).abs() % 90.0;
            desalinhos.push(if d > 45.0 { 90.0 - d } else { d });
        }
    }
    desalinhos.sort_by(f32::total_cmp);
    let n = desalinhos.len();
    let flush = desalinhos.iter().filter(|d| **d < 5.0).count();
    let quase = desalinhos.iter().filter(|d| **d < 15.0).count();
    eprintln!("\n  {n} contactos no monte assente");
    eprintln!(
        "  desalinho p10/p50/p90: {:.1}° / {:.1}° / {:.1}°",
        desalinhos[n / 10],
        desalinhos[n / 2],
        desalinhos[n * 9 / 10]
    );
    eprintln!(
        "  face-com-face (< 5°) : {flush} = {:.0}%",
        100.0 * flush as f32 / n as f32
    );
    eprintln!(
        "  quase        (< 15°) : {quase} = {:.0}%",
        100.0 * quase as f32 / n as f32
    );
    eprintln!("\n  ⚠️ num monte com desalinho UNIFORME 0..45 esperar-se-ia 11% e 33%.");
}

/// **SONDA — a W0: as peças da `=114` são MESMO gelo?**
///
/// ⚠️⚠️ O doc 109 §8.5 e o doc 111 §2 afirmam que sim (*«a cena não escreve material nenhum»*). Esta
/// sonda lê a coluna do stream COZIDO em vez de a supor — *uma ausência afirmada sem olhar a coluna
/// é um palpite com cara de medição*.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_as_pecas_sao_gelo -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_as_pecas_sao_gelo() {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let s = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, sinks[1], 1.0)
        .expect("cozinha")[0]
        .as_stream()
        .clone();
    for col in [
        ph2d_nodegraph::attr::FRICTION_COLUMN,
        ph2d_nodegraph::attr::BOUNCE_COLUMN,
        ph2d_nodegraph::attr::ROLLING_COLUMN,
    ] {
        match s.get(col) {
            Some(Column::Scalar(v)) => eprintln!(
                "  {col:<10} PRESENTE, {} linhas, valor[0] = {:.3}",
                v.len(),
                v.first().copied().unwrap_or(f32::NAN)
            ),
            _ => eprintln!("  {col:<10} AUSENTE  ⇒ o par cai no `Material::LISO`"),
        }
    }
    let mats = ph2d_contact::materiais(&s);
    eprintln!(
        "\n  `ph2d_contact::materiais` devolve {} ⇒ μ do par = {:.3}",
        if mats.is_some() { "Some" } else { "None" },
        mats.as_ref()
            .and_then(|m| m.first())
            .map_or(0.0, |m| ph2d_contact::atrito::mu(m.atrito, m.atrito))
    );
}

/// O `y` MEDIANO das peças da direita no fim — a régua que separa *«assentou»* de *«congelou»*.
fn altura_mediana_final() -> f32 {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let mut ys = Vec::new();
    for k in 0..=174_u64 {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sinks[1], t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if k == 174
            && let Some(Column::Vec2(p)) = s.get("P")
        {
            ys = p.iter().map(|q| q[1]).collect();
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    mediana(&ys)
}

/// **SONDA — W2 do [doc 111]: os CONTACTOS sobrevivem ao tique seguinte?**
///
/// ⭐⭐⭐ É o facto que decide se a obra encomendada é sequer APLICÁVEL. O `λ` acumulado (*warm
/// starting*) precisa de uma chave estável entre tiques: se as feições mudarem todas, **não há o que
/// aquecer**, e um `λ` guardado numa chave que mudou é **pior que nenhum** — ele aquece o contacto
/// errado.
///
/// A chave medida é o PAR `(id menor, id maior)`, que é o mais grosseiro possível: se nem ela
/// sobreviver, uma chave com a FEIÇÃO dentro sobrevive ainda menos.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_os_contactos_sobrevivem -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_os_contactos_sobrevivem() {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let mut anterior: Vec<(usize, usize)> = Vec::new();
    let (mut vivos, mut novos, mut tiques) = (0_usize, 0_usize, 0_usize);
    for k in 0..=174_u64 {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sinks[1], t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if t >= 2.0
            && let (Some(Column::Vec2(p)), Some(cols)) = (s.get("P"), ph2d_contact::colisores(&s))
        {
            let mut agora = Vec::new();
            for i in 0..p.len() {
                for j in (i + 1)..p.len() {
                    if let (Some(a), Some(b)) = (cols[i], cols[j])
                        && ph2d_contact::contato(&a, p[i], &b, p[j], (i + j) % 2 == 0).is_some()
                    {
                        agora.push((i, j));
                    }
                }
            }
            if !anterior.is_empty() {
                vivos += agora.iter().filter(|c| anterior.contains(c)).count();
                novos += agora.iter().filter(|c| !anterior.contains(c)).count();
                tiques += 1;
            }
            anterior = agora;
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    let total = vivos + novos;
    eprintln!("\n  sobre {tiques} tiques da janela assente:");
    eprintln!("  contactos que SOBREVIVEM ao tique anterior: {vivos}");
    eprintln!("  contactos NOVOS                           : {novos}");
    eprintln!(
        "  ⇒ taxa de sobrevivência: {:.1} %",
        100.0 * vivos as f32 / total.max(1) as f32
    );
    eprintln!("\n  ⚠️ abaixo de ~80 % nao ha o que aquecer: doc 111 §5 W2.");
}
