//! Os gates da cena `=129`: a fixtura CONTÉM o fenómeno (a gelatina sem colisor atravessa-se) e
//! a com colisor segura o diâmetro sem inchar — pela BOMBA do produto (as formas publicadas e as
//! ilhas sub-passadas antes do cozimento do quadro).

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};

const TIQUES: u64 = 360;

fn vec2(s: &Stream, nome: &str) -> Vec<[f32; 2]> {
    match s.get(nome) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// O pior par que o REPOUSO não liga (lido da coluna `rest` que o corpo publica).
fn pior_solto(s: &Stream, raio: f32) -> f32 {
    let (p, rest) = (vec2(s, "P"), vec2(s, "rest"));
    assert_eq!(p.len(), rest.len(), "o corpo publica o repouso");
    let mut pior = f32::MAX;
    for i in 0..p.len() {
        for j in (i + 1)..p.len() {
            let dr = (rest[j][0] - rest[i][0]).hypot(rest[j][1] - rest[i][1]);
            if dr >= 2.0 * raio - 1e-4 {
                pior = pior.min((p[j][0] - p[i][0]).hypot(p[j][1] - p[i][1]));
            }
        }
    }
    pior
}

fn extensao(s: &Stream) -> f32 {
    let p = vec2(s, "P");
    let mut e = 0.0f32;
    for a in &p {
        for b in &p {
            e = e.max((a[0] - b[0]).hypot(a[1] - b[1]));
        }
    }
    e
}

/// A cena pela bomba da CPU, com o `Collide` do Shape de cada coluna dado (`None` = o autorado):
/// `([(pior solto na 2.ª metade, extensão no fim); 2], discos por saída, [mediana do pior solto; 2])`.
/// A MEDIANA é o que uma foto num instante qualquer apanha (o mínimo é um instante).
fn marcha_com(r: Receita, collide: [Option<bool>; 2]) -> ([(f32, f32); 2], Vec<usize>, [f32; 2]) {
    let mut m = MotionState::new();
    let (saidas, corpos, formas) = monta_com(&mut m.doc, &m.registry, r).expect("a cena monta");
    for (f, c) in formas.iter().zip(collide) {
        if let Some(c) = c {
            m.doc.graph.set_param(
                *f,
                ph2d_node_motion_shape::param::COLLIDE,
                f32::from(u8::from(c)),
            );
        }
    }
    m.sinks = saidas.clone();
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    let mut res = [(f32::MAX, 0.0f32); 2];
    let mut series = [Vec::new(), Vec::new()];
    for t in 0..TIQUES {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        crate::motion_externals::publish_all(&mut m, t as f64 / 60.0);
        let sinks = m.sinks.clone();
        let _ = m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &sinks,
            t,
            |t| t as f64 / 60.0,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
        for (i, b) in corpos.iter().enumerate() {
            let s = m.pump.cook.peek(*b).expect("o corpo cozeu")[0]
                .as_stream()
                .clone();
            if t >= TIQUES / 2 {
                let p = pior_solto(&s, RAIO);
                res[i].0 = res[i].0.min(p);
                series[i].push(p);
            }
            res[i].1 = extensao(&s);
        }
    }
    let discos = saidas
        .iter()
        .map(|s| m.pump.cook.peek(*s).map_or(0, |v| v[0].as_stream().count()))
        .collect();
    let mediana = |v: &mut Vec<f32>| {
        v.sort_by(f32::total_cmp);
        v.get(v.len() / 2).copied().unwrap_or(f32::MAX)
    };
    let (m0, m1) = (mediana(&mut series[0]), mediana(&mut series[1]));
    (res, discos, [m0, m1])
}

/// ⭐⭐ **A cena ensina o que acontece** (`CLAUDE.md` §5.0), medido pelo que uma FOTO apanha: a
/// laranja (o `Collide` do Shape desligado) passa a 2.ª metade com as peças encavalitadas, a verde
/// (ligado) nunca desce do diâmetro; e cada partícula é uma peça na tela.
#[test]
fn the_left_body_crosses_and_the_right_one_holds() {
    let ([_, (com, _)], discos, [mediana, _]) = marcha_com(RECEITA, [None, None]);
    eprintln!("cena 129: sem Collide (mediana) {mediana:.4} · com Collide (minimo) {com:.4}");
    let d = 2.0 * RAIO;
    assert!(
        mediana < d * 0.5,
        "CONTROLO: a laranja tem de se atravessar ({mediana:.4})"
    );
    assert!(
        com >= d * 0.9,
        "a verde segura o diametro: {com:.4} < {:.4}",
        d * 0.9
    );
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a malha"
    )]
    let malha = (LADO * LADO) as usize;
    assert_eq!(discos, vec![malha; 2], "uma peca por particula, nas duas");
}

/// ⭐⭐⭐ **O GESTO DO DONO** (report de 08/10: *«o colisor estava desligado nas duas. ao ligar não
/// vi diferença»*): ligar o `Collide` do Shape da ESQUERDA fá-la segurar como a verde — e é a
/// única coisa que muda.
#[test]
fn turning_on_the_left_shapes_collide_makes_it_hold() {
    let ([(esquerda, _), (direita, _)], _, _) = marcha_com(RECEITA, [Some(true), None]);
    eprintln!(
        "cena 129 com o Collide da esquerda ligado: esquerda {esquerda:.4} · direita {direita:.4}"
    );
    let d = 2.0 * RAIO;
    assert!(
        esquerda >= d * 0.9,
        "ligar o Collide do Shape tem de segurar: {esquerda:.4}"
    );
    assert!(
        (esquerda - direita).abs() < 1e-6,
        "as duas colunas sao a MESMA cena menos o botao: {esquerda} contra {direita}"
    );
}

/// **A sonda das receitas** — a que escolheu a [`RECEITA`].
#[test]
#[ignore = "sonda: as receitas da cena 129 — corra à mão com --nocapture"]
fn sonda_as_receitas() {
    for clusters in [1.0f32, 3.0] {
        for sub_passos in [4.0f32, 8.0] {
            for ima in [200.0f32, 400.0] {
                let r = Receita {
                    rigidez: 0.03,
                    ima,
                    ima_y: 0.5,
                    curso: 2.0,
                    sub_passos,
                    clusters,
                };
                let ([(_, e0), (com, e1)], d, [med, _]) = marcha_com(r, [None, None]);
                println!(
                    "RECEITA {r:?}: sem Collide mediana {med:.4} (ext {e0:.3}) · com {com:.4} \
                     (ext {e1:.3}) · pecas {d:?}"
                );
            }
        }
    }
}

/// Que colisor as peças da porta `shape` declaram (diagnóstico).
#[test]
#[ignore = "sonda: o colisor que as pecas declaram — corra à mão com --nocapture"]
fn sonda_o_colisor_das_pecas() {
    let mut m = MotionState::new();
    let (saidas, corpos, _) = monta(&mut m.doc, &m.registry).expect("a cena monta");
    m.sinks = saidas;
    crate::motion_externals::publish_all(&mut m, 0.0);
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    let sinks = m.sinks.clone();
    let _ = m.pump.advance_or_scrub_scoped(
        &m.doc.graph,
        &m.registry,
        &sinks,
        0,
        |t| t as f64 / 60.0,
        m.default_uv_rect,
        m.default_size,
        &scopes,
    );
    for b in corpos {
        let s = m.pump.cook.peek(b).expect("corpo")[0].as_stream().clone();
        let c = ph2d_contact::colisores(&s);
        let nomes: Vec<&String> = s.columns().map(|(n, _)| n).collect();
        println!(
            "PECAS colunas {nomes:?} · colisores {:?}",
            c.map(|v| v
                .iter()
                .take(2)
                .map(|c| c.map(|c| (c.alcance(), c.forma)))
                .collect::<Vec<_>>())
        );
    }
}

/// O corpo do Bug #15 (`5 × 5`, passo `0,30`, raio `0,28`, a âncora a varrer `±2,0` em `0,7 s`,
/// `substeps 4`) FEITO DE PEÇAS, com o `Collide` do Shape dado. Devolve
/// `(pior solto na 2.ª metade, extensão no fim)`, pela bomba do app.
fn bug15_de_pecas(collide: bool) -> (f32, f32) {
    use ph2d_nodegraph::graph::{Edge, Graph};
    let mut m = MotionState::new();
    let g: &mut Graph = &mut m.doc.graph;
    let liga = |g: &mut Graph, a, ap, b, bp, pre| {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed: pre,
        })
        .expect("liga");
    };
    let sb = g.add_node("motion.soft_body".to_string());
    for (p, v) in [
        ("gravity", 9.8),
        ("stiffness", 0.3),
        ("damping", 0.02),
        ("substeps", 4.0),
    ] {
        g.set_param(sb, p, v);
    }
    let lfo = g.add_node("value.lfo".to_string());
    g.set_param(lfo, "period", 0.7);
    g.set_param(lfo, "amplitude", 2.0);
    liga(g, lfo, 0, sb, 0, false);
    liga(g, sb, 0, sb, 2, true);
    let grelha = g.add_node("motion.grid".to_string());
    for (p, v) in [("rows", 5.0), ("cols", 5.0), ("gap_x", 0.3), ("gap_y", 0.3)] {
        g.set_param(grelha, p, v);
    }
    use ph2d_node_motion_shape::param as p;
    let forma = g.add_node("source.shape".to_string());
    g.set_param(forma, p::SIZE, RAIO_DO_BUG15);
    g.set_param(forma, p::COLLIDER_SHAPE, 1.0);
    g.set_param(forma, p::COLLIDE, f32::from(u8::from(collide)));
    let dup = g.add_node("motion.duplicator".to_string());
    liga(g, forma, 0, dup, 0, false);
    liga(g, grelha, 0, dup, 1, false);
    liga(g, dup, 0, sb, 3, false);
    let saida = g.add_node("motion.output".to_string());
    liga(g, sb, 0, saida, 0, false);
    m.sinks = vec![saida];
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    let (mut pior, mut ext) = (f32::MAX, 0.0);
    for t in 0..180u64 {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        crate::motion_externals::publish_all(&mut m, t as f64 / 60.0);
        let sinks = m.sinks.clone();
        let _ = m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &sinks,
            t,
            |t| t as f64 / 60.0,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
        let s = m.pump.cook.peek(sb).expect("corpo")[0].as_stream().clone();
        if t >= 90 {
            pior = pior.min(pior_solto(&s, RAIO_DO_BUG15));
        }
        ext = extensao(&s);
    }
    (pior, ext)
}

/// O raio da fixtura do Bug #15 — MAIOR que meio passo (`0,30`): os vizinhos sobrepõem-se no
/// repouso, que é o caso da lei.
const RAIO_DO_BUG15: f32 = 0.28;

/// ⭐⭐ **O corpo de peças com o `Collide` do Shape segura o diâmetro e NÃO incha** (doc 124 §6) —
/// a lei do Bug #15 pela porta que o dono escolheu: os pares que o repouso sobrepõe são do corpo.
#[test]
fn a_body_of_pieces_holds_the_diameter_and_does_not_swell() {
    let (sem, e_sem) = bug15_de_pecas(false);
    let (com, e_com) = bug15_de_pecas(true);
    eprintln!("pecas: pior solto {sem:.4} -> {com:.4} · extensao {e_sem:.4} -> {e_com:.4}");
    let d = 2.0 * RAIO_DO_BUG15;
    assert!(
        sem < d * 0.95,
        "CONTROLO: sem o Collide o corpo atravessa-se ({sem:.4})"
    );
    assert!(
        com >= d * 0.9,
        "o Collide do Shape segura o diametro: {com:.4}"
    );
    assert!(
        (e_com - e_sem).abs() <= e_sem * 0.05,
        "o corpo NAO incha: {e_sem:.4} -> {e_com:.4}"
    );
}

/// ⭐ **As linhas que o tutorial 09 manda mexer EXISTEM nos cartões da cena** (`CLAUDE.md` §5.0: um
/// passo que manda procurar uma linha que a tela não mostra faz o dono concluir que o programa está
/// partido): `Collide` e `Collider Shape` no Shape, `Substeps` no Soft Body.
#[test]
fn the_rows_the_tutorial_names_are_on_the_cards() {
    const TUTORIAL: &str =
        include_str!("../../../docs/Motion Nodes/tutoriais/src/09_coisas_que_se_seguram.html");
    let mut m = MotionState::new();
    let _ = crate::motion_demo_legend::monta("129", &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    // ⚠️ O `Collider Shape` só aparece DEPOIS de ligar o `Collide` (a linha é condicionada) — é
    // por isso que o tutorial diz «ligue Collide e ponha o Collider Shape…» por esta ordem, e o
    // gate o lê no cartão da DIREITA (ligado).
    let linhas_de = |tipo: &str| -> Vec<Vec<&'static str>> {
        let ids: Vec<u32> = m
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == tipo)
            .map(|n| n.id.0)
            .collect();
        snap.nodes
            .iter()
            .filter(|v| ids.contains(&v.id))
            .map(|v| v.params.iter().map(|c| c.hint.label).collect())
            .collect()
    };
    let (formas, corpos) = (linhas_de("source.shape"), linhas_de("motion.soft_body"));
    assert_eq!(
        (formas.len(), corpos.len()),
        (2, 2),
        "CONTROLO: os quatro cartoes foram lidos"
    );
    for (cartao, l) in [
        (&formas[0], "Collide"),
        (&formas[1], "Collide"),
        (&formas[1], "Collider Shape"),
        (&corpos[0], "Substeps"),
        (&corpos[1], "Substeps"),
    ] {
        assert!(
            cartao.contains(&l),
            "o tutorial manda mexer em `{l}` e o cartao mostra {cartao:?}"
        );
        assert!(
            TUTORIAL.contains(&format!("<b>{l}</b>")),
            "o tutorial nomeia `{l}`"
        );
    }
}
