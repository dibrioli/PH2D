//! ⭐⭐ **O BANCO DE CENÁRIOS** (plano 30 §2.7, a queixa Q7 — *«o ORCA entala-se nos cantos e nos
//! empates simétricos»*, Sunshine-Hill, Game AI Pro 3): as seis cenas do oráculo do Godot corridas em
//! MALHA FECHADA pela nossa lei, cada uma com três réguas — ninguém se sobrepõe, ninguém entra numa
//! parede, todos chegam.
//!
//! As cenas são as do `desvio.gd` (o mesmo `px`, raio `10`, `100 px/s`, horizontes `1 s`), e o
//! desfecho do Godot nelas está MEDIDO na fixtura — é contra ele que a vantagem se declara.

use ph2d_orca::{Agent, Crowd, Params, V2, Walls};

const R: f64 = 10.0;
const VMAX: f64 = 100.0;
const DT: f64 = 1.0 / 60.0;
const PASSOS: usize = 900;

struct Cena {
    nome: &'static str,
    agentes: Vec<(V2, V2)>,
    paredes: Vec<Vec<V2>>,
    /// ⚠️ Um ponto de PASSAGEM comum antes do alvo — o que o caminho planeado daria. Sem ele a
    /// velocidade pedida aponta em linha recta para o alvo, e quem vem de lado entala-se na quina da
    /// porta: é o mínimo local que o ORCA sozinho não sai, e que no produto o Polyanya já resolveu.
    passagem: Option<V2>,
}

fn caixa(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<V2> {
    vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
}

/// O círculo de `n`, cada um para o ponto oposto. ⚠️ Sem `cos`/`sin` (a cerca da crate vale nos testes
/// que medem a lei): os oito pontos de um octógono saem de `√½`.
fn circulo8() -> Vec<(V2, V2)> {
    let h = 0.5_f64.sqrt();
    let u: [V2; 8] = [
        [1.0, 0.0],
        [h, h],
        [0.0, 1.0],
        [-h, h],
        [-1.0, 0.0],
        [-h, -h],
        [0.0, -1.0],
        [h, -h],
    ];
    u.iter()
        .map(|d| {
            (
                [200.0 + 120.0 * d[0], 150.0 + 120.0 * d[1]],
                [200.0 - 120.0 * d[0], 150.0 - 120.0 * d[1]],
            )
        })
        .collect()
}

fn cenas() -> Vec<Cena> {
    vec![
        Cena {
            nome: "frente",
            agentes: vec![([50.0, 150.0], [350.0, 150.0]), ([350.0, 150.0], [50.0, 150.0])],
            paredes: vec![],
            passagem: None,
        },
        Cena {
            nome: "frente_desviado",
            agentes: vec![([50.0, 150.0], [350.0, 150.0]), ([350.0, 153.0], [50.0, 153.0])],
            paredes: vec![],
            passagem: None,
        },
        Cena {
            nome: "cruzamento",
            agentes: vec![
                ([50.0, 50.0], [350.0, 250.0]),
                ([350.0, 250.0], [50.0, 50.0]),
                ([350.0, 50.0], [50.0, 250.0]),
                ([50.0, 250.0], [350.0, 50.0]),
            ],
            paredes: vec![],
            passagem: None,
        },
        Cena {
            nome: "circulo8",
            agentes: circulo8(),
            paredes: vec![],
            passagem: None,
        },
        Cena {
            nome: "corredor",
            agentes: vec![([60.0, 150.0], [340.0, 150.0]), ([340.0, 152.0], [60.0, 152.0])],
            paredes: vec![caixa(100.0, 90.0, 300.0, 120.0), caixa(100.0, 180.0, 300.0, 210.0)],
            passagem: None,
        },
        Cena {
            nome: "porta",
            agentes: vec![
                // ⚠️ No `desvio.gd` os alvos distam `10 px` e dois são o MESMO ponto — com `2r = 20` não
                // cabem todos, e «todos chegam» seria impossível. O passo a passo do oráculo não depende
                // disso; aqui os alvos distam `25 px`.
                ([100.0, 150.0], [320.0, 150.0]),
                ([80.0, 120.0], [320.0, 125.0]),
                ([80.0, 180.0], [320.0, 175.0]),
                ([50.0, 150.0], [345.0, 150.0]),
            ],
            paredes: vec![caixa(190.0, 0.0, 210.0, 125.0), caixa(190.0, 175.0, 210.0, 300.0)],
            passagem: Some([200.0, 150.0]),
        },
    ]
}

/// A velocidade que o agente QUER: direito ao alvo, e a travar no último passo (a regra do script).
fn pref(pos: V2, alvo: V2) -> V2 {
    let d = [alvo[0] - pos[0], alvo[1] - pos[1]];
    let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
    if l > VMAX * DT {
        [d[0] / l * VMAX, d[1] / l * VMAX]
    } else {
        [d[0] / DT, d[1] / DT]
    }
}

/// A distância de `p` ao polígono convexo `poly` (negativa dentro).
fn dist_poly(poly: &[V2], p: V2) -> f64 {
    let n = poly.len();
    let mut dentro = true;
    let mut best = f64::INFINITY;
    for k in 0..n {
        let a = poly[k];
        let b = poly[(k + 1) % n];
        let ab = [b[0] - a[0], b[1] - a[1]];
        let ap = [p[0] - a[0], p[1] - a[1]];
        if ab[0] * ap[1] - ab[1] * ap[0] < 0.0 {
            dentro = false;
        }
        let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1])).clamp(0.0, 1.0);
        let q = [a[0] + ab[0] * t - p[0], a[1] + ab[1] * t - p[1]];
        best = best.min((q[0] * q[0] + q[1] * q[1]).sqrt());
    }
    if dentro { -best } else { best }
}

struct Desfecho {
    /// O quadro em que TODOS chegaram (a menos de 1 px), se chegaram.
    chegaram: Option<usize>,
    /// A menor distância entre dois centros.
    min_par: f64,
    /// A menor distância de um centro a uma parede.
    min_parede: f64,
}

fn corre(c: &Cena, side_bias: f64) -> Desfecho {
    let walls = Walls::from_polygons(&c.paredes);
    let params = Params {
        side_bias,
        ..Params::PRODUCT
    };
    let mut pos: Vec<V2> = c.agentes.iter().map(|a| a.0).collect();
    let mut vel: Vec<V2> = vec![[0.0, 0.0]; pos.len()];
    let mut passou: Vec<bool> = vec![c.passagem.is_none(); pos.len()];
    let mut out = Desfecho {
        chegaram: None,
        min_par: f64::INFINITY,
        min_parede: f64::INFINITY,
    };
    for passo in 0..PASSOS {
        let agentes: Vec<Agent> = (0..pos.len())
            .map(|i| Agent {
                pos: pos[i],
                vel: vel[i],
                pref: pref(
                    pos[i],
                    match c.passagem {
                        Some(p) if !passou[i] => p,
                        _ => c.agentes[i].1,
                    },
                ),
                radius: R,
                max_speed: VMAX,
                avoids: true,
            })
            .collect();
        let mut crowd = Crowd::new(agentes, params);
        let tem = !c.paredes.is_empty();
        let novas: Vec<V2> = crowd.solve_all(|_| tem.then_some((&walls, R)), DT);
        for i in 0..pos.len() {
            if let Some(p) = c.passagem
                && ((pos[i][0] - p[0]).powi(2) + (pos[i][1] - p[1]).powi(2)).sqrt() < 2.0 * R
            {
                passou[i] = true;
            }
            vel[i] = novas[i];
            pos[i] = [pos[i][0] + vel[i][0] * DT, pos[i][1] + vel[i][1] * DT];
        }
        for i in 0..pos.len() {
            for j in i + 1..pos.len() {
                let d = ((pos[i][0] - pos[j][0]).powi(2) + (pos[i][1] - pos[j][1]).powi(2)).sqrt();
                out.min_par = out.min_par.min(d);
            }
            for p in &c.paredes {
                out.min_parede = out.min_parede.min(dist_poly(p, pos[i]));
            }
        }
        let todos = (0..pos.len()).all(|i| {
            let a = c.agentes[i].1;
            ((pos[i][0] - a[0]).powi(2) + (pos[i][1] - a[1]).powi(2)).sqrt() < 1.0
        });
        if todos && out.chegaram.is_none() {
            out.chegaram = Some(passo + 1);
        }
    }
    out
}

/// ⚠️ **A folga da sobreposição**, em px: o ORCA garante o desvio no tempo CONTÍNUO e o passo é
/// discreto, logo um par pode invadir-se no máximo o que se anda num tique a separar-se (o ramo «já se
/// tocam» separa-os no tique SEGUINTE). Medido nas seis cenas com o peso do produto: NENHUMA invasão —
/// o pior par a `2r + 0,0001 px`, a pior parede a `r + 0,006 px`. `1e-3` deixa o arredondamento passar
/// e fica três ordens abaixo de um passo (`100 px/s × dt = 1,7 px`).
const FOLGA: f64 = 1e-3;

#[test]
fn ninguem_se_sobrepoe_ninguem_entra_na_parede_e_todos_chegam() {
    for c in cenas() {
        let d = corre(&c, ph2d_orca::SIDE_BIAS);
        eprintln!(
            "{:>16}: chegaram {:?} · par mín {:.6} px (2r = {}) · parede mín {:.6} px",
            c.nome,
            d.chegaram,
            d.min_par,
            2.0 * R,
            d.min_parede
        );
        assert!(d.chegaram.is_some(), "{}: nem todos chegaram em {PASSOS} quadros", c.nome);
        assert!(d.min_par >= 2.0 * R - FOLGA, "{}: sobreposição {}", c.nome, d.min_par);
        assert!(d.min_parede >= R - FOLGA, "{}: dentro da parede {}", c.nome, d.min_parede);
    }
}

/// ⭐ **O CONTROLO: sem a preferência de lado, os dois frente a frente PARAM** — como no Godot, medido
/// na fixtura (a cena `frente` dele acaba os `240` quadros a `25 px` um do outro). Sem este controlo a
/// régua de cima não provava que é o peso que desfaz o empate.
#[test]
fn sem_a_preferencia_de_lado_o_frente_a_frente_para_como_no_godot() {
    let frente = cenas().into_iter().find(|c| c.nome == "frente").expect("a cena");
    let d = corre(&frente, 0.0);
    assert_eq!(d.chegaram, None);
    assert!(d.min_par >= 2.0 * R - FOLGA, "parados, mas sem se sobrepor: {}", d.min_par);
}
