//! ⭐⭐⭐ **NENHUM `Collide` VISÍVEL FICA MUDO SEM O ARTISTA SABER PORQUÊ** (doc 125 §2) — o censo
//! que liga o botão, pela bomba do produto, e exige EFEITO no que se desenha OU um conselho
//! ([`Deficit::ColisorMudo`]) no cartão dele.
//!
//! ⭐ Desde o doc 115 §33 há UMA chave só: o `Collide` da forma. O passe do fim arma-se sozinho em
//! todo sink, logo o `Output` já não tem botão — e um `Collide` que chega a um sink tem sempre
//! leitor. As montagens são as que o artista faz com as peças que se sobrepõem.

use super::corda_com_colisor_demo_tests::{pior_solto, quadro};
use crate::motion_state::MotionState;
use ph2d_motion_diagnose::Deficit;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

const RAIO: f32 = 0.15;
const TIQUES: u64 = 120;

fn liga(g: &mut Graph, a: NodeId, b: NodeId, porta: u16, pre: bool) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, porta),
        delayed: pre,
    })
    .expect("liga");
}

fn shape(g: &mut Graph) -> NodeId {
    use ph2d_node_motion_shape::param as p;
    let s = g.add_node("source.shape".to_string());
    g.set_param(s, p::SIZE, RAIO);
    g.set_param(s, p::COLLIDER_SHAPE, 1.0);
    s
}

fn grelha(g: &mut Graph) -> NodeId {
    let n = g.add_node("motion.grid".to_string());
    for (p, v) in [("rows", 4.0), ("cols", 4.0), ("gap_x", 0.2), ("gap_y", 0.2)] {
        g.set_param(n, p, v);
    }
    n
}

/// Uma montagem: o grafo, o Shape cujo `Collide` se liga e o Output.
struct Montagem {
    nome: &'static str,
    g: Graph,
    shape: NodeId,
    saida: NodeId,
}

/// `Grid → Duplicator ← Shape → Output` — sem simulação.
fn estatica() -> Montagem {
    let mut g = Graph::new();
    let (gr, s) = (grelha(&mut g), shape(&mut g));
    let dup = g.add_node("motion.duplicator".to_string());
    liga(&mut g, s, dup, 0, false);
    liga(&mut g, gr, dup, 1, false);
    let saida = g.add_node("motion.output".to_string());
    liga(&mut g, dup, saida, 0, false);
    Montagem {
        nome: "estatica",
        g,
        shape: s,
        saida,
    }
}

/// O corpo mole da `=129` (a receita MEDIDA que se dobra: um íman dentro dele), com as peças a
/// ENTRAR (`pecas_dentro`) ou a vesti-lo DEPOIS. O raio fica abaixo de meio passo: no repouso nada
/// se sobrepõe, e o que o passe do fim vê são as dobras.
fn corpo(pecas_dentro: bool, raio: f32) -> Montagem {
    use super::super::corpo_com_colisor_demo::{PASSO, RECEITA as R};
    let mut g = Graph::new();
    let s = shape(&mut g);
    g.set_param(s, ph2d_node_motion_shape::param::SIZE, raio);
    let sb = g.add_node("motion.soft_body".to_string());
    for (p, v) in [
        ("rows", 7.0),
        ("cols", 7.0),
        ("spacing", PASSO),
        ("gravity", 9.0),
        ("stiffness", R.rigidez),
        ("damping", 0.03),
        ("pin", 1.0),
        ("substeps", R.sub_passos),
        ("clusters", R.clusters),
    ] {
        g.set_param(sb, p, v);
    }
    let lfo = g.add_node("value.lfo".to_string());
    g.set_param(lfo, "period", 1.8);
    g.set_param(lfo, "amplitude", R.curso);
    liga(&mut g, lfo, sb, 0, false);
    let ima = g.add_node("force.attractor".to_string());
    g.set_param(ima, "target_y", R.ima_y);
    g.set_param(ima, "strength", R.ima);
    g.set_param(ima, "radius", 3.0);
    liga(&mut g, sb, ima, 0, true);
    liga(&mut g, ima, sb, 2, false);
    let saida = g.add_node("motion.output".to_string());
    if pecas_dentro {
        let gr = g.add_node("motion.grid".to_string());
        for (p, v) in [
            ("rows", 7.0),
            ("cols", 7.0),
            ("gap_x", PASSO),
            ("gap_y", PASSO),
        ] {
            g.set_param(gr, p, v);
        }
        let dup = g.add_node("motion.duplicator".to_string());
        liga(&mut g, s, dup, 0, false);
        liga(&mut g, gr, dup, 1, false);
        liga(&mut g, dup, sb, 3, false);
        liga(&mut g, sb, saida, 0, false);
    } else {
        let dup = g.add_node("motion.duplicator".to_string());
        liga(&mut g, s, dup, 0, false);
        liga(&mut g, sb, dup, 1, false);
        liga(&mut g, dup, saida, 0, false);
    }
    Montagem {
        nome: if pecas_dentro {
            "corpo, pecas dentro"
        } else {
            "corpo, vestido depois"
        },
        g,
        shape: s,
        saida,
    }
}

/// Abaixo de meio passo da `=129` (`0,30`): no repouso as peças não se tocam.
const RAIO_DO_CORPO: f32 = 0.14;
/// Acima de meio passo: os vizinhos de malha sobrepõem-se no repouso (o caso da lei).
const RAIO_SOBREPOSTO: f32 = 0.28;

/// A corda da `=130` (a receita MEDIDA: sem gravidade a corda nua cruza-se em todos os quadros),
/// com o Shape na porta `shape` (`dentro`) ou a vesti-la DEPOIS.
fn corda(dentro: bool) -> Montagem {
    use super::super::corda_com_colisor_demo::RECEITA as R;
    let mut g = Graph::new();
    let s = shape(&mut g);
    g.set_param(s, ph2d_node_motion_shape::param::SIZE, R.raio);
    let c = g.add_node("motion.verlet_rope".to_string());
    for (p, v) in [
        ("count", R.contas),
        ("length", R.comprimento),
        ("gravity", R.gravidade),
        ("iterations", 16.0),
    ] {
        g.set_param(c, p, v);
    }
    let lfo = g.add_node("value.lfo".to_string());
    g.set_param(lfo, "period", R.periodo);
    g.set_param(lfo, "amplitude", R.curso);
    liga(&mut g, lfo, c, u16::from(R.de_lado), false);
    liga(&mut g, c, c, 2, true);
    let saida = g.add_node("motion.output".to_string());
    if dentro {
        liga(&mut g, s, c, 3, false);
        liga(&mut g, c, saida, 0, false);
    } else {
        let dup = g.add_node("motion.duplicator".to_string());
        liga(&mut g, s, dup, 0, false);
        liga(&mut g, c, dup, 1, false);
        liga(&mut g, dup, saida, 0, false);
    }
    Montagem {
        nome: if dentro {
            "corda, shape dentro"
        } else {
            "corda, vestida depois"
        },
        g,
        shape: s,
        saida,
    }
}

/// O que o Output DESENHA ao fim de `tiques` quadros, com o `Collide` do Shape dado — pela porta do
/// produto (`o_que_o_sink_desenha`, o passe do fim incluído). `passe = false` devolve a corrente
/// COZIDA, antes do passe (a régua do «o passe não incha o corpo»).
fn desenho(m: &Montagem, shape_on: bool, passe: bool, tiques: u64) -> Stream {
    let mut st = MotionState::new();
    st.doc.graph = m.g.clone();
    let g = &mut st.doc.graph;
    g.set_param(
        m.shape,
        ph2d_node_motion_shape::param::COLLIDE,
        f32::from(u8::from(shape_on)),
    );
    st.sinks = vec![m.saida];
    for t in 0..tiques {
        quadro(&mut st, t);
    }
    let cozido = st.pump.cook.peek(m.saida).expect("o output cozeu")[0]
        .as_stream()
        .clone();
    if !passe {
        return cozido;
    }
    ph2d_eval_motion::o_que_o_sink_desenha(&st.doc.graph, m.saida, &cozido)
        .0
        .unwrap_or(cozido)
}

fn posicoes(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn muda(a: &Stream, b: &Stream) -> bool {
    let (a, b) = (posicoes(a), posicoes(b));
    a.len() != b.len()
        || a.iter()
            .zip(&b)
            .any(|(p, q)| (p[0] - q[0]).abs() > 1e-5 || (p[1] - q[1]).abs() > 1e-5)
}

fn conselho(m: &Montagem, shape_on: bool, no: NodeId) -> Option<Deficit> {
    let st = MotionState::new();
    let mut g = m.g.clone();
    g.set_param(
        m.shape,
        ph2d_node_motion_shape::param::COLLIDE,
        f32::from(u8::from(shape_on)),
    );
    ph2d_motion_diagnose::diagnose(&g, &st.registry)
        .into_iter()
        .find(|d| d.node == no && matches!(d.deficit, Deficit::ColisorMudo(_)))
        .map(|d| d.deficit)
}

/// ⭐⭐⭐ **O CENSO**: cada `Collide`, ligado sozinho, MUDA o desenho ou ACONSELHA no cartão.
#[test]
fn every_collide_alone_has_an_effect_or_an_advice() {
    let montagens = [
        estatica(),
        corpo(true, RAIO_DO_CORPO),
        corpo(false, RAIO_DO_CORPO),
        corda(true),
        corda(false),
    ];
    let mut linhas = Vec::new();
    for m in &montagens {
        let base = desenho(m, false, true, TIQUES);
        // ⭐ O botão SOZINHO — e desde a §33 ele é o único: o Output arma o passe por si.
        let efeito = muda(&base, &desenho(m, true, true, TIQUES));
        let aviso = conselho(m, true, m.shape);
        linhas.push(format!(
            "{:24} Shape : efeito {efeito:5} · conselho {aviso:?}",
            m.nome
        ));
        assert!(
            efeito || aviso.is_some(),
            "`{}`: o Collide do Shape nao faz nada e o cartao nao diz porque",
            m.nome
        );
        // ⚠️ O CONTROLO de que a montagem contém sobreposição: ligado, o desenho MUDA — senão «sem
        // efeito» seria verdade por vácuo. Desde a §33 isto vale em TODA montagem que chega a um
        // sink, com ou sem conselho (o passe do fim separa o desenho sozinho).
        assert!(
            efeito,
            "CONTROLO `{}`: com o Collide do Shape o desenho tem de mudar",
            m.nome
        );
    }
    eprintln!("{}", linhas.join("\n"));
}

/// ⭐ **E nenhum conselho onde o botão FUNCIONA** — as peças dentro de um corpo, e o botão numa
/// montagem estática (o passe do fim lê-o sozinho): o aviso a soar num grafo certo seria ruído (a
/// lição do `NeverDrawn`).
#[test]
fn a_working_collide_is_never_advised() {
    for m in [corpo(true, RAIO_DO_CORPO), corda(true)] {
        assert_eq!(
            conselho(&m, true, m.shape),
            None,
            "`{}`: o Shape funciona",
            m.nome
        );
    }
    let m = estatica();
    assert_eq!(
        conselho(&m, true, m.shape),
        None,
        "estatica: o Shape tem leitor — o passe automatico do Output"
    );
}

/// ⭐⭐ **O passe do fim não incha um corpo de peças** (doc 125 §2, o filtro do Bug #15 no passe):
/// com o `Collide` ligado (o passe do fim arma-se sozinho), o desenho guarda os vizinhos de repouso
/// onde o corpo os pôs — comparado com a corrente cozida ANTES do passe. O raio
/// é o do Bug #15 (`0,28`, MAIOR que meio passo): os vizinhos sobrepõem-se no repouso.
#[test]
fn the_end_pass_does_not_swell_a_body_of_pieces() {
    let m = corpo(true, RAIO_SOBREPOSTO);
    let so_corpo = desenho(&m, true, false, TIQUES);
    let com_passe = desenho(&m, true, true, TIQUES);
    let extensao = |s: &Stream| {
        let p = posicoes(s);
        let mut e = 0.0f32;
        for a in &p {
            for b in &p {
                e = e.max((a[0] - b[0]).hypot(a[1] - b[1]));
            }
        }
        e
    };
    let (e0, e1) = (extensao(&so_corpo), extensao(&com_passe));
    eprintln!(
        "passe sobre o corpo de pecas: extensao {e0:.4} -> {e1:.4} · pior solto {:.4} -> {:.4}",
        pior_solto(&so_corpo, RAIO_SOBREPOSTO),
        pior_solto(&com_passe, RAIO_SOBREPOSTO)
    );
    assert!(
        (e1 - e0).abs() <= e0 * 0.05,
        "o passe do fim incha o corpo desenhado: {e0:.4} -> {e1:.4}"
    );
}
