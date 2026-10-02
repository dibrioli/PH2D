//! ⭐⭐ **O ORÁCULO CORRIDO: o Godot 4.7.2 (MIT, RVO2)** sobre cenas NOSSAS (plano 30 §8.2, família F4).
//!
//! A fixtura `tests/fixtures/godot/desvio_f4.txt` é a saída de
//! `docs/Components/ferramentas/godot_nav_oraculo/desvio.gd` corrido sem interface em MALHA FECHADA: o
//! script é o dono do estado e grava, a cada quadro, o que entregou ao servidor (`IN`: posição,
//! velocidade de agora, velocidade preferida) e o que ele devolveu (`OUT`: a velocidade segura). ⇒ cada
//! passo é re-resolvido ISOLADO, com as entradas exactas dele — o erro não se acumula, e um passo
//! errado aponta-se pelo número.
//!
//! ⚠️ **O quadro 1 fica fora, MEDIDO:** duas corridas do Godot em processos diferentes deram respostas
//! diferentes no quadro 1 da cena `frente` (`-100` e `-89.17`) e iguais em todos os outros — o
//! servidor resolve o 1.º quadro antes de sincronizar a posição que lhe foi dada (a armadilha do
//! *«`agent_set_position` é diferido»* da pesquisa, doc 29 §6.1).

use ph2d_orca::{Agent, Crowd, Params, Regime, V2, Walls};

const FIXTURA: &str = include_str!("../fixtures/godot/desvio_f4.txt");

struct Passo {
    quadro: u32,
    entradas: Vec<(V2, V2, V2)>,
    saidas: Vec<Option<V2>>,
}

struct Cena {
    nome: String,
    raio: f64,
    vmax: f64,
    nd: f64,
    mn: usize,
    tau: f64,
    tau_w: f64,
    dt: f64,
    obs: Vec<Vec<V2>>,
    passos: Vec<Passo>,
}

fn nums(t: &[&str]) -> Vec<f64> {
    t.iter().map(|s| s.parse::<f64>().expect("número")).collect()
}

fn le() -> Vec<Cena> {
    let mut cenas: Vec<Cena> = Vec::new();
    for l in FIXTURA.lines() {
        let t: Vec<&str> = l.split_whitespace().collect();
        let Some(&cab) = t.first() else { continue };
        match cab {
            "CENA" => cenas.push(Cena {
                nome: t[1].to_owned(),
                raio: 0.0,
                vmax: 0.0,
                nd: 0.0,
                mn: 0,
                tau: 0.0,
                tau_w: 0.0,
                dt: 0.0,
                obs: Vec::new(),
                passos: Vec::new(),
            }),
            "PARAM" => {
                let c = cenas.last_mut().expect("PARAM antes de CENA");
                let n = nums(&t[1..]);
                (c.raio, c.vmax, c.nd, c.mn, c.tau, c.tau_w, c.dt) =
                    (n[0], n[1], n[2], n[3] as usize, n[4], n[5], n[6]);
            }
            "OBS" => {
                let n = nums(&t[1..]);
                let c = cenas.last_mut().expect("OBS antes de CENA");
                c.obs.push(n.chunks(2).map(|p| [p[0], p[1]]).collect());
            }
            "IN" => {
                let c = cenas.last_mut().expect("IN antes de CENA");
                let q: u32 = t[1].parse().expect("quadro");
                let i: usize = t[2].parse().expect("agente");
                let n = nums(&t[3..]);
                if c.passos.last().is_none_or(|p| p.quadro != q) {
                    c.passos.push(Passo {
                        quadro: q,
                        entradas: Vec::new(),
                        saidas: Vec::new(),
                    });
                }
                let p = c.passos.last_mut().expect("passo");
                assert_eq!(p.entradas.len(), i, "agentes fora de ordem em {} q{q}", c.nome);
                p.entradas.push(([n[0], n[1]], [n[2], n[3]], [n[4], n[5]]));
                p.saidas.push(None);
            }
            "OUT" => {
                let c = cenas.last_mut().expect("OUT antes de CENA");
                let q: u32 = t[1].parse().expect("quadro");
                let i: usize = t[2].parse().expect("agente");
                let n = nums(&t[3..]);
                let Some(p) = c.passos.iter_mut().rev().find(|p| p.quadro == q) else {
                    // O servidor responde aos agentes recém-criados no quadro 0, antes de qualquer
                    // entrada — e responde zero. Só esse quadro pode chegar aqui.
                    assert_eq!((q, n.as_slice()), (0, [0.0, 0.0].as_slice()), "OUT sem IN em {}", c.nome);
                    continue;
                };
                p.saidas[i] = Some([n[0], n[1]]);
            }
            _ => {}
        }
    }
    cenas
}

/// O pior desvio de uma cena (`px/s`, quadro, agente), separado pela distância ao TOQUE.
#[derive(Default)]
struct Pior {
    longe: (f64, u32, usize),
    toque: (f64, u32, usize),
    n: usize,
    n_toque: usize,
    n_apertado: usize,
}

/// Como o Godot SEQUENCIAL vê os vizinhos do agente `i` (ver o cabeçalho de [`piores`]).
fn vista_do_godot(c: &Cena, p: &Passo, i: usize) -> Vec<Agent> {
    p.entradas
        .iter()
        .zip(&p.saidas)
        .enumerate()
        .map(|(j, (&(pos, vel, pref), out))| {
            let (pos, vel) = match out {
                Some(o) if j < i => ([pos[0] + o[0] * c.dt, pos[1] + o[1] * c.dt], *o),
                _ => (pos, vel),
            };
            Agent {
                pos,
                vel,
                pref,
                radius: c.raio,
                max_speed: c.vmax,
                avoids: true,
            }
        })
        .collect()
}

/// ⚠️ **O Godot resolve EM SEQUÊNCIA** (Gauss-Seidel pela ordem de criação): o agente `i` vê os
/// anteriores JÁ andados neste quadro — posição `+ saída × dt` e a saída como velocidade — e os
/// seguintes como entraram. Medido na cena `frente`: com a fotografia comum (a do artigo, a NOSSA) o
/// pior passo erra `0,74 px/s` logo no quadro 25; com esta leitura, `0,000009`. ⇒ a ordem dele é
/// reproduzida AQUI para provar que os semi-planos e os programas lineares são os mesmos — e o produto
/// fica com a fotografia comum, que não depende de quem foi criado primeiro.
///
/// ⛔ **E só com o desvio dele numa LINHA de execução** (`avoidance_use_multiple_threads = false`,
/// o projecto ao lado do script): com as linhas de fábrica, o agente `1` lê o `0` ora antes, ora
/// depois, ora a MEIO da actualização dele (medido nos quadros 52, 85 e 169 da `frente`) — uma
/// corrida de dados, e duas corridas diferem. Numa linha só, duas corridas são iguais byte a byte.
fn piores() -> Vec<(String, Pior)> {
    let mut out = Vec::new();
    for c in le() {
        let walls = Walls::from_polygons(&c.obs);
        let params = Params {
            time_horizon: c.tau,
            time_horizon_walls: c.tau_w,
            neighbor_dist: Some(c.nd),
            max_neighbors: Some(c.mn),
            side_bias: 0.0,
        };
        let banda = BANDA_DE_TOQUE * 2.0 * c.raio;
        let mut pior = Pior::default();
        for p in &c.passos {
            for (i, s) in p.saidas.iter().enumerate() {
                // O último quadro de cada cena acaba antes da resposta.
                let Some(s) = s else { continue };
                let vista = vista_do_godot(&c, p, i);
                let me = vista[i].pos;
                let folga = vista
                    .iter()
                    .enumerate()
                    .filter(|&(j, _)| j != i)
                    .map(|(_, o)| ((o.pos[0] - me[0]).hypot(o.pos[1] - me[1]) - 2.0 * c.raio).abs())
                    .fold(f64::INFINITY, f64::min);
                let crowd = Crowd::new(vista, params);
                let w = (!c.obs.is_empty()).then_some((&walls, c.raio));
                let (v, regime) = crowd.solve(i, w, c.dt);
                let e = (v[0] - s[0]).hypot(v[1] - s[1]);
                pior.n += 1;
                pior.n_apertado += usize::from(regime == Regime::Dense);
                let alvo = if folga <= banda {
                    pior.n_toque += 1;
                    &mut pior.toque
                } else {
                    &mut pior.longe
                };
                if e > alvo.0 {
                    *alvo = (e, p.quadro, i);
                }
            }
        }
        out.push((c.nome, pior));
    }
    out
}

/// ⚠️ **A faixa de TOQUE**, em fracção da soma dos raios: `0,5 %` (`0,1 px` a raios de `10`).
///
/// Encostados, o semi-plano é MAL CONDICIONADO — a perna do cone sai de `√(d² − R²)` com `d² − R²`
/// perto de zero, e o ramo «já se tocam» divide a posição pelo TIQUE (`× 60`) — logo o `f32` do Godot
/// é amplificado. Medido sobre os `5 670` passos, pela folga ao vizinho mais perto: com a faixa a `0`,
/// o pior é `0,046 px/s`; a `0,25 %`, `0,0041`; a `0,5 %`, **`0,00068`** — o ruído de fora. Os sete
/// passos acima de `1e-3` têm TODOS um vizinho a menos de `0,083 px` do toque.
const BANDA_DE_TOQUE: f64 = 0.005;
/// Longe do toque: o pior foi `0,000695 px/s` (a `porta`), o ruído do `f32` do Godot a `~300 px`
/// (`2⁻²⁴ × 300 / dt ≈ 1e-3`). `2e-3` deixa folga de três vezes, e fica três ordens abaixo do erro de
/// UM semi-plano errado (a fotografia comum errava `0,74`).
const TOL_LONGE: f64 = 2e-3;
/// No toque: medido `0,046 px/s` (`0,05 %` da velocidade máxima).
const TOL_TOQUE: f64 = 0.1;

#[test]
fn cada_passo_do_godot_e_re_resolvido_igual() {
    let r = piores();
    for (nome, p) in &r {
        eprintln!(
            "{nome:>16}: longe {:.6} px/s (q{} a{}) · toque {:.6} (q{} a{}) em {} · apertados {} · de {} passos",
            p.longe.0, p.longe.1, p.longe.2, p.toque.0, p.toque.1, p.toque.2, p.n_toque, p.n_apertado, p.n
        );
    }
    assert_eq!(r.len(), 6, "as seis cenas da fixtura");
    for (nome, p) in &r {
        assert!(p.n > 400, "{nome}: só {} passos comparados", p.n);
        assert!(p.longe.0 <= TOL_LONGE, "{nome}: longe {:?}", p.longe);
        assert!(p.toque.0 <= TOL_TOQUE, "{nome}: no toque {:?}", p.toque);
    }
    // ⭐ A população: passos no toque E passos apertados (o 3D) existem, senão a barra do toque e a
    // paridade do 3D não mediriam nada.
    let toque: usize = r.iter().map(|(_, p)| p.n_toque).sum();
    let apertados: usize = r.iter().map(|(_, p)| p.n_apertado).sum();
    assert!(toque >= 100, "só {toque} passos no toque");
    assert!(apertados >= 100, "só {apertados} passos apertados");
}
