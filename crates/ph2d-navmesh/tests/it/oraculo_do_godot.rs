//! ⭐⭐ **O ORÁCULO CORRIDO: o Godot 4.7.2 (MIT)** sobre cenas NOSSAS (plano 30 §8.2, famílias F1 + F2).
//!
//! A fixtura `tests/fixtures/godot/corpus_f1_f2.txt` é a saída de
//! `docs/Components/ferramentas/godot_nav_oraculo/corpus.gd` corrido sem interface, e traz a PRÓPRIA
//! geometria de cada cena (`REG`/`OBS`) — este gate lê-a de lá, nunca a repete (duas cópias de uma
//! cena divergem no dia em que uma mudar).
//!
//! Três leis, uma por bloco:
//!
//! 1. **F1 — a construção em ESQUADRIA dá a malha do Godot**: a mesma área, a menos do ruído medido
//!    do `f32` dele.
//! 2. **F2 — o nosso caminho nunca é mais longo que o dele** (o A\*+funil dele é óptimo só no
//!    corredor; o Polyanya é óptimo, gate S1) — e é estritamente mais curto em pelo menos um par, senão
//!    a vantagem declarada não estaria medida.
//! 3. **A divergência DECLARADA existe**: com o recuo REDONDO a área andável é maior e nenhum caminho
//!    é mais longo que o da esquadria (§2.3, S3).

use ph2d_nav::{Polyanya, V2};
use ph2d_navmesh::{Corner, Params, Shape, build};

const FIXTURA: &str = include_str!("../fixtures/godot/corpus_f1_f2.txt");

/// A barra sai da MEDIÇÃO: nas 28 combinações o pior desvio de área contra o Godot foi **`0`**
/// (os vértices das cenas são exactos em `f32` e a nossa grelha de `2⁻¹⁶` contém-nos). Fica o ruído da
/// soma em `f64` de áreas de `~10⁵` (`~1e-11`), com folga de cinco ordens — uma barra larga é onde
/// uma régua errada sobrevive.
const AREA_TOL: f64 = 1e-6;
/// O caminho dele começa e acaba em pontos re-escritos em `f32` (`50.000007629` por `50`): `1e-4`.
const LEN_TOL: f64 = 1e-3;

struct Cena {
    nome: String,
    raio: f64,
    regiao: Vec<V2>,
    obs: Vec<Vec<V2>>,
    area: f64,
    pares: Vec<(V2, V2, f64)>,
}

fn pontos(s: &str) -> Vec<V2> {
    s.split_whitespace()
        .map(|p| {
            let mut it = p.split(',').map(|x| x.parse::<f64>().expect("número"));
            [it.next().expect("x"), it.next().expect("y")]
        })
        .collect()
}

fn cenas() -> Vec<Cena> {
    let mut out: Vec<Cena> = Vec::new();
    for linha in FIXTURA.lines() {
        if let Some(r) = linha.strip_prefix("CENA ") {
            let mut it = r.split_whitespace();
            let nome = it.next().expect("nome").to_string();
            let raio = it.nth(1).expect("raio").parse().expect("raio");
            out.push(Cena {
                nome,
                raio,
                regiao: Vec::new(),
                obs: Vec::new(),
                area: 0.0,
                pares: Vec::new(),
            });
        } else if let Some(r) = linha.strip_prefix("REG ") {
            out.last_mut().expect("CENA antes").regiao = pontos(r);
        } else if let Some(r) = linha.strip_prefix("OBS ") {
            out.last_mut().expect("CENA antes").obs.push(pontos(r));
        } else if let Some(r) = linha.strip_prefix("AREA ") {
            out.last_mut().expect("CENA antes").area = r.trim().parse().expect("área");
        } else if let Some(r) = linha.strip_prefix("PATH ") {
            let v: Vec<&str> = r.split_whitespace().collect();
            let n = |i: usize| v[i].parse::<f64>().expect("número");
            let len = n(5);
            out.last_mut()
                .expect("CENA antes")
                .pares
                .push(([n(0), n(1)], [n(2), n(3)], len));
        }
    }
    out
}

fn construir(c: &Cena, corner: Corner) -> ph2d_navmesh::Built {
    let obs: Vec<Shape> = c.obs.iter().map(|o| Shape::Convex(o.clone())).collect();
    build(
        &c.regiao,
        &obs,
        &Params {
            agent_radius: c.raio,
            corner,
            ..Params::default()
        },
    )
    .unwrap_or_else(|e| panic!("{} r={}: {e:?}", c.nome, c.raio))
}

#[test]
fn a_fixtura_tem_as_sete_cenas_nos_quatro_raios() {
    let cs = cenas();
    assert_eq!(cs.len(), 28, "7 cenas × 4 raios");
    assert!(
        cs.iter()
            .all(|c| c.regiao.len() == 4 && !c.obs.is_empty() && !c.pares.is_empty())
    );
    assert!(
        FIXTURA.contains("Godot 4.7.2"),
        "o cabeçalho diz que versão do oráculo correu"
    );
}

#[test]
fn f1_a_esquadria_da_a_area_do_godot() {
    let mut pior = 0.0f64;
    for c in cenas() {
        let b = construir(&c, Corner::Miter);
        let d = (b.mesh.area() - c.area).abs();
        pior = pior.max(d);
        assert!(
            d <= AREA_TOL,
            "{} r={}: nossa {} contra Godot {} (Δ {d})",
            c.nome,
            c.raio,
            b.mesh.area(),
            c.area
        );
    }
    eprintln!("F1: pior Δ de área contra o Godot = {pior:e}");
}

#[test]
fn f2_o_nosso_caminho_nunca_e_mais_longo_que_o_do_godot() {
    let mut s = Polyanya::new();
    let mut mais_curtos = 0;
    let mut comparados = 0;
    for c in cenas() {
        let b = construir(&c, Corner::Miter);
        for &(a, z, len_godot) in &c.pares {
            // Um ponto que o recuo engoliu é PROJECTADO, como o Godot faz (família F3).
            let (a2, _) = b.mesh.nearest_point(a, None).expect("malha não vazia");
            let (z2, _) = b.mesh.nearest_point(z, None).expect("malha não vazia");
            let Ok(p) = s.find_path(&b.mesh, a2, z2) else {
                // Ilhas diferentes: o Godot devolve um caminho parcial (outra família, F3).
                continue;
            };
            comparados += 1;
            assert!(
                p.length <= len_godot + LEN_TOL,
                "{} r={}: {a:?}→{z:?}: nosso {} contra Godot {len_godot}",
                c.nome,
                c.raio,
                p.length
            );
            if p.length < len_godot - 0.5 {
                mais_curtos += 1;
            }
        }
    }
    assert!(comparados >= 90, "só {comparados} pares comparados");
    assert!(
        mais_curtos >= 1,
        "nenhum par ficou mais curto que o do Godot — a vantagem S1 não está medida"
    );
    eprintln!("F2: {comparados} pares, {mais_curtos} mais curtos que o Godot por mais de 0,5");
}

#[test]
fn o_recuo_redondo_diverge_da_esquadria_a_favor_do_agente() {
    let mut s = Polyanya::new();
    for c in cenas().into_iter().filter(|c| c.raio > 0.0) {
        let m = construir(&c, Corner::Miter);
        let r = construir(&c, Corner::Round);
        assert!(
            r.mesh.area() > m.mesh.area(),
            "{} r={}: o redondo tem de andar mais",
            c.nome,
            c.raio
        );
        for &(a, z, _) in &c.pares {
            let (Some(_), Some(_)) = (m.mesh.locate(a), m.mesh.locate(z)) else {
                continue;
            };
            let (Ok(pm), Ok(pr)) = (s.find_path(&m.mesh, a, z), s.find_path(&r.mesh, a, z)) else {
                continue;
            };
            assert!(
                pr.length <= pm.length + 1e-9,
                "{} r={}: redondo {} > esquadria {}",
                c.nome,
                c.raio,
                pr.length,
                pm.length
            );
        }
    }
}
