//! Os gates das cenas `=130`/`=131` (doc 125): pela BOMBA do produto (as formas publicadas antes do
//! cozimento do quadro), a corda sem `Collide` atravessa-se, com ele segura o diâmetro sem esticar,
//! e o gesto do dono (ligar o botão, ou religar o Shape à porta `shape`) é o que o faz.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::{Edge, Graph};

const TIQUES: u64 = 240;

fn vec2(s: &Stream, nome: &str) -> Vec<[f32; 2]> {
    match s.get(nome) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// O pior par que o REPOUSO não liga (a coluna `rest` que a corda publica).
pub(crate) fn pior_solto(s: &Stream, raio: f32) -> f32 {
    let (p, rest) = (vec2(s, "P"), vec2(s, "rest"));
    assert_eq!(p.len(), rest.len(), "a corda publica o repouso");
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

fn arco(s: &Stream) -> f32 {
    vec2(s, "P")
        .windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .sum()
}

/// Um quadro da bomba da CPU, com as formas publicadas antes (sem o `publish_all` o `source.shape`
/// não existe e o teste mede outra coisa).
pub(crate) fn quadro(m: &mut MotionState, t: u64) {
    #[expect(clippy::cast_precision_loss, reason = "um tique")]
    crate::motion_externals::publish_all(m, t as f64 / 60.0);
    let sinks = m.sinks.clone();
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
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
}

/// `(pior solto na 2.ª metade, MEDIANA do pior solto na 2.ª metade, maior arco na 2.ª metade,
/// FRAÇÃO dos quadros da 2.ª metade com contas atravessadas — o pior solto abaixo do raio)` de cada
/// corda.
fn marcha(m: &mut MotionState, cordas: &[NodeId], raio: f32) -> Vec<(f32, f32, f32, f32)> {
    let mut series = vec![Vec::new(); cordas.len()];
    let mut arcos = vec![0.0f32; cordas.len()];
    for t in 0..TIQUES {
        quadro(m, t);
        if t < TIQUES / 2 {
            continue;
        }
        for (i, c) in cordas.iter().enumerate() {
            let s = m.pump.cook.peek(*c).expect("a corda cozeu")[0]
                .as_stream()
                .clone();
            series[i].push(pior_solto(&s, raio));
            arcos[i] = arcos[i].max(arco(&s));
        }
    }
    series
        .into_iter()
        .zip(arcos)
        .map(|(mut v, a)| {
            let min = v.iter().copied().fold(f32::MAX, f32::min);
            #[expect(clippy::cast_precision_loss, reason = "quadros")]
            let cruza = v.iter().filter(|d| **d < raio).count() as f32 / v.len() as f32;
            v.sort_by(f32::total_cmp);
            (min, v[v.len() / 2], a, cruza)
        })
        .collect()
}

/// A cena `=130` com o `Collide` de cada Shape dado (`None` = o autorado).
fn cena(r: Receita, collide: [Option<bool>; 2]) -> Vec<(f32, f32, f32, f32)> {
    let mut m = MotionState::new();
    let (saidas, cordas, formas) = monta(&mut m.doc, &m.registry, r).expect("a cena monta");
    for (f, c) in formas.iter().zip(collide) {
        if let Some(c) = c {
            m.doc.graph.set_param(
                *f,
                ph2d_node_motion_shape::param::COLLIDE,
                f32::from(u8::from(c)),
            );
        }
    }
    m.sinks = saidas;
    marcha(&mut m, &cordas, r.raio)
}

/// ⭐⭐ **A cena ensina o que acontece** (`CLAUDE.md` §5.0): a laranja (sem `Collide`) passa a 2.ª
/// metade com as contas umas dentro das outras, a verde nunca desce de `0,9·D` e não estica.
#[test]
fn the_left_rope_crosses_and_the_right_one_holds() {
    let v = cena(RECEITA, [None, None]);
    let ((_, _, arco_nua, cruza), (com, _, arco_com, _)) = (v[0], v[1]);
    eprintln!(
        "cena 130: sem Collide cruza {:.0}% dos quadros · com (minimo) {com:.4} · arco {arco_nua:.3} -> {arco_com:.3}",
        cruza * 100.0
    );
    let d = 2.0 * RECEITA.raio;
    assert!(
        cruza >= 0.25,
        "CONTROLO: a laranja atravessa-se pelo menos num quadro em cada quatro ({cruza:.2})"
    );
    assert!(
        com >= d * 0.9,
        "a verde segura o diametro: {com:.4} < {:.4}",
        d * 0.9
    );
    assert!(
        arco_com <= arco_nua * 1.10,
        "a verde NAO estica: {arco_com:.3} contra {arco_nua:.3}"
    );
}

/// ⭐⭐ **O GESTO do dono** (o passo 3 do anúncio): ligar o `Collide` do Shape da esquerda faz a
/// corda dele segurar — o controlo está no CARTÃO, no catálogo.
#[test]
fn turning_on_the_left_shapes_collide_makes_the_rope_hold() {
    let v = cena(RECEITA, [Some(true), None]);
    let d = 2.0 * RECEITA.raio;
    eprintln!("cena 130, a esquerda ligada: {:.4}", v[0].0);
    assert!(
        v[0].0 >= d * 0.9,
        "ligar o Collide da esquerda segura: {:.4}",
        v[0].0
    );
}

/// ⭐⭐⭐ **O Bug #16 pela fixtura do ORÁCULO** (`rope_thickness`: `24` nós, raio `0,5`, a cabeça a
/// `3,0·sin(2πt/0,6)`, `60 Hz`), pela porta do artista: `Shape (Collide) → Verlet Rope`. A régua é
/// a do oráculo (`ferramentas/godot_oracle_corrente/`, Godot, MIT): com a lei, a corrente dele
/// segura `0,9957·D` na 2.ª metade e não estica.
#[test]
fn the_rope_reads_its_pieces_collider_and_does_not_cross() {
    let oraculo: f32 =
        include_str!("../../../docs/Motion Nodes/ferramentas/godot_oracle_corrente/saida.txt")
            .lines()
            .find(|l| l.contains("modo=lei "))
            .and_then(|l| {
                l.split_whitespace()
                    .find_map(|c| c.strip_prefix("pior_nao_ligado_2a_metade="))
            })
            .and_then(|v| v.parse().ok())
            .expect("a saida do oraculo traz o modo `lei`");
    let fixtura = Receita {
        raio: 0.5,
        periodo: 0.6,
        curso: 3.0,
        de_lado: false,
        gravidade: 9.8,
        ..RECEITA
    };
    let v = cena(fixtura, [None, None]);
    let ((nua, _, arco_nua, _), (com, _, arco_com, _)) = (v[0], v[1]);
    eprintln!(
        "Bug #16: nua {nua:.4} · com o Collide {com:.4} (oraculo {oraculo}) · arco {arco_nua:.3} -> {arco_com:.3}"
    );
    let d = 2.0 * fixtura.raio;
    assert!(
        nua < d * 0.5,
        "CONTROLO: a corda nua atravessa-se ({nua:.4})"
    );
    assert!(oraculo >= d * 0.9, "o oraculo segura com a lei ({oraculo})");
    assert!(
        com >= d * 0.9,
        "a corda segura o diametro: {com:.4} < {:.4}",
        d * 0.9
    );
    assert!(
        arco_com <= arco_nua * 1.10,
        "a corda NAO estica: {arco_com:.3} contra {arco_nua:.3}"
    );
}

/// ⭐⭐ **A `=131`: o `Collide` depois da corda é um botão que DIZ o que fazer** (doc 125 §2), e o
/// gesto que o aviso pede — o fio do Shape para a porta `shape` da corda — cala o aviso e segura.
#[test]
fn the_shape_after_the_rope_advises_and_rewiring_it_holds() {
    let mut m = MotionState::new();
    let (saida, corda, shape) = monta_depois(&mut m.doc, &m.registry).expect("a cena monta");
    let aviso = |g: &Graph, reg: &ph2d_node_registry::NodeRegistry| {
        ph2d_motion_diagnose::diagnose(g, reg)
            .into_iter()
            .find(|d| d.node == shape)
            .map(|d| d.deficit)
    };
    assert_eq!(
        aviso(&m.doc.graph, &m.registry),
        Some(ph2d_motion_diagnose::Deficit::ColisorMudo("shape")),
        "o cartao do Shape avisa"
    );
    m.sinks = vec![saida];
    let antes = marcha(&mut m, &[corda], RECEITA.raio)[0];
    // O GESTO: o fio do Shape para a porta `shape` da corda.
    m.doc
        .graph
        .connect(Edge {
            from: (shape, 0),
            to: (corda, 3),
            delayed: false,
        })
        .expect("o fio do gesto");
    assert_eq!(aviso(&m.doc.graph, &m.registry), None, "o aviso cala-se");
    let mut m2 = MotionState::new();
    m2.doc.graph = m.doc.graph.clone();
    m2.sinks = vec![saida];
    let depois = marcha(&mut m2, &[corda], RECEITA.raio)[0];
    let d = 2.0 * RECEITA.raio;
    eprintln!(
        "cena 131: antes cruza {:.0}% · depois do fio (minimo) {:.4}",
        antes.3 * 100.0,
        depois.0
    );
    assert!(
        antes.3 >= 0.25,
        "CONTROLO: antes do fio a corda atravessa-se ({:.2})",
        antes.3
    );
    assert!(
        depois.0 >= d * 0.9,
        "depois do fio a corda segura: {:.4}",
        depois.0
    );
}

/// A sonda que escolheu a [`RECEITA`]: o raio das contas.
#[test]
#[ignore = "sonda: as receitas da cena 130 — corra à mão com --nocapture"]
fn sonda_as_receitas_da_corda() {
    for (periodo, curso, de_lado, gravidade) in [
        (0.6f32, 3.0f32, false, 9.8f32),
        (1.0, 3.0, false, 0.0),
        (1.0, 3.0, true, 0.0),
        (1.3, 3.0, true, 0.0),
        (1.3, 2.0, true, 0.0),
        (1.6, 3.0, true, 0.0),
    ] {
        for raio in [0.12f32, 0.15] {
            let r = Receita {
                raio,
                periodo,
                curso,
                de_lado,
                gravidade,
                ..RECEITA
            };
            let v = cena(r, [None, None]);
            println!(
                "periodo {periodo} curso {curso} de lado {de_lado} g {gravidade} raio {raio:.2}: sem (min {:.4} mediana {:.4} arco {:.3} cruza {:.0}%) · com (min {:.4} arco {:.3} cruza {:.0}%) · D {:.2}",
                v[0].0,
                v[0].1,
                v[0].2,
                v[0].3 * 100.0,
                v[1].0,
                v[1].2,
                v[1].3 * 100.0,
                2.0 * raio
            );
        }
    }
}
