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
fn pior_solto(s: &Stream) -> f32 {
    let (p, rest) = (vec2(s, "P"), vec2(s, "rest"));
    assert_eq!(p.len(), rest.len(), "o corpo publica o repouso");
    let mut pior = f32::MAX;
    for i in 0..p.len() {
        for j in (i + 1)..p.len() {
            let dr = (rest[j][0] - rest[i][0]).hypot(rest[j][1] - rest[i][1]);
            if dr >= 2.0 * RAIO {
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

/// A cena pela bomba da CPU: `([(pior solto na 2.ª metade, extensão no fim); 2], discos por saída,
/// a MEDIANA do pior solto da gelatina SEM colisor na 2.ª metade — o que uma foto
/// num instante qualquer apanha (o mínimo é um instante).
fn marcha_com_mediana(r: Receita) -> ([(f32, f32); 2], Vec<usize>, f32) {
    let mut serie: Vec<f32> = Vec::new();
    let mut m = MotionState::new();
    let (saidas, corpos, _) = monta_com(&mut m.doc, &m.registry, r).expect("a cena monta");
    m.sinks = saidas.clone();
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    let mut res = [(f32::MAX, 0.0f32); 2];
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
                let p = pior_solto(&s);
                res[i].0 = res[i].0.min(p);
                if i == 0 {
                    serie.push(p);
                }
            }
            res[i].1 = extensao(&s);
        }
    }
    let discos = saidas
        .iter()
        .map(|s| m.pump.cook.peek(*s).map_or(0, |v| v[0].as_stream().count()))
        .collect();
    serie.sort_by(f32::total_cmp);
    let mediana = serie.get(serie.len() / 2).copied().unwrap_or(f32::MAX);
    (res, discos, mediana)
}

/// ⭐⭐ **A cena ensina o que acontece** (`CLAUDE.md` §5.0) — medido pelo que uma FOTO apanha: a
/// laranja passa a 2.ª metade com discos encavalitados (a MEDIANA do pior par solto, não um
/// instante), a verde nunca desce do diâmetro; e cada partícula é um disco na tela.
#[test]
fn the_left_body_crosses_and_the_right_one_holds() {
    let ([(_, _), (com, _)], discos, mediana) = marcha_com_mediana(RECEITA);
    eprintln!("cena 129: sem colisor (mediana) {mediana:.4} · com colisor (minimo) {com:.4}");
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
    assert_eq!(discos, vec![malha; 2], "um disco por particula, nas duas");
}

/// **A sonda das receitas** — a que escolheu a [`RECEITA`]: a gelatina tem de se DOBRAR sem
/// colisor (o fenómeno à vista) e não explodir.
#[test]
#[ignore = "sonda: as receitas da cena 129 — corra à mão com --nocapture"]
fn sonda_as_receitas() {
    for clusters in [1.0f32, 3.0, 4.0] {
        for rigidez in [0.03f32, 0.1] {
            for ima in [200.0f32, 400.0] {
                let r = Receita {
                    rigidez,
                    ima,
                    ima_y: 0.5,
                    curso: 2.0,
                    iteracoes: 8.0,
                    sub_passos: 8.0,
                    clusters,
                };
                let ([(sem, e_sem), (com, e_com)], d, med) = marcha_com_mediana(r);
                println!(
                    "RECEITA {r:?}: sem min {sem:.4} MEDIANA {med:.4} (ext {e_sem:.3}) · com \
                     {com:.4} (ext {e_com:.3}) · discos {d:?}"
                );
            }
        }
    }
}
