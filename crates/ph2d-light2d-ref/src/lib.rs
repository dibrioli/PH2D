//! **O gabarito da luz 2D** — a resposta CERTA, devagar, na CPU, em `f64`, determinística.
//!
//! A fluência num ponto é `∫ L(θ)·T(θ) dθ` sobre a volta inteira. O método é EXATO, sem Monte
//! Carlo: os ângulos onde a resposta pode mudar de natureza (vértices, tangentes aos discos,
//! cruzamentos de fronteiras que envolvem um emissor, cruzamentos com o círculo do alcance) partem
//! a volta em intervalos dentro dos quais quem o raio acerta e quantas vezes ele entra em cada
//! oclusor NÃO mudam. Basta então UM raio no meio de cada intervalo. Com radiância uniforme e
//! alcance infinito o integrando é constante no intervalo (soma exata das larguras); com alcance
//! finito integra-se a `queda` por Gauss–Kronrod adaptativo.
//!
//! Semântica (a GPU tem de responder a MESMA pergunta): emissores com área são OPACOS (o raio
//! termina no primeiro); um oclusor multiplica por `1 − opacidade` a cada ENTRADA do raio; o céu
//! chega pelas direções que escapam. Por que este método e não quadratura: decidido pelos gates
//! G1/G2 (ver o handoff da W1).

mod geometria;
mod integra;
pub mod queda;

use std::f64::consts::TAU;

use geometria::{Fronteira, angulo, cruzamentos, intersecoes};
use ph2d_light2d_ecs::{CenaDeLuz, Emissor};
use queda::queda;

struct Objeto {
    fronteira: Fronteira,
    /// `Some` = emissor (opaco).
    emissao: Option<([f64; 3], f64)>,
    /// `1 − opacidade` (só para oclusores).
    transmissao: f64,
}

struct Pontual {
    centro: [f64; 2],
    intensidade: [f64; 3],
    alcance: f64,
}

fn rgb(c: [f32; 3]) -> [f64; 3] {
    c.map(f64::from)
}

fn pontos(v: &[[f32; 2]]) -> Vec<[f64; 2]> {
    v.iter()
        .map(|p| [f64::from(p[0]), f64::from(p[1])])
        .collect()
}

fn montar(cena: &CenaDeLuz) -> (Vec<Objeto>, Vec<Pontual>) {
    let mut objetos = Vec::new();
    let mut pontuais = Vec::new();
    for e in &cena.emissores {
        match e {
            Emissor::Disco {
                centro,
                raio,
                radiancia,
                alcance_m,
            } => objetos.push(Objeto {
                fronteira: Fronteira::Disco {
                    c: [f64::from(centro[0]), f64::from(centro[1])],
                    r: f64::from(*raio),
                },
                emissao: Some((rgb(*radiancia), f64::from(*alcance_m))),
                transmissao: 0.0,
            }),
            Emissor::Poligono {
                vertices,
                radiancia,
                alcance_m,
            } => objetos.push(Objeto {
                fronteira: Fronteira::Poligono(pontos(vertices)),
                emissao: Some((rgb(*radiancia), f64::from(*alcance_m))),
                transmissao: 0.0,
            }),
            Emissor::Ponto {
                centro,
                intensidade,
                alcance_m,
            } => pontuais.push(Pontual {
                centro: [f64::from(centro[0]), f64::from(centro[1])],
                intensidade: rgb(*intensidade),
                alcance: f64::from(*alcance_m),
            }),
        }
    }
    for o in &cena.oclusores {
        objetos.push(Objeto {
            fronteira: Fronteira::Poligono(pontos(&o.vertices)),
            emissao: None,
            transmissao: 1.0 - f64::from(o.opacidade),
        });
    }
    (objetos, pontuais)
}

/// O que um raio encontra: o emissor que o termina (índice e distância) e a transmissão até lá.
struct Raio {
    acerto: Option<(usize, f64)>,
    transmissao: f64,
}

fn lancar(objetos: &[Objeto], o: [f64; 2], d: [f64; 2], buf: &mut Vec<Vec<f64>>) -> Raio {
    buf.resize_with(objetos.len(), Vec::new);
    let mut acerto: Option<(usize, f64)> = None;
    for (k, obj) in objetos.iter().enumerate() {
        cruzamentos(&obj.fronteira, o, d, &mut buf[k]);
        if obj.emissao.is_some() {
            let c = &buf[k];
            let t = if c.len() % 2 == 1 {
                Some(0.0)
            } else {
                c.first().copied()
            };
            if let Some(t) = t
                && acerto.is_none_or(|(_, ta)| t < ta)
            {
                acerto = Some((k, t));
            }
        }
    }
    let limite = acerto.map_or(f64::INFINITY, |(_, t)| t);
    Raio {
        acerto,
        transmissao: transmissao_ate(objetos, buf, limite),
    }
}

/// O produto de `1 − opacidade` por ENTRADA em cada oclusor, antes de `limite`, sobre os
/// cruzamentos já lançados em `buf`.
fn transmissao_ate(objetos: &[Objeto], buf: &[Vec<f64>], limite: f64) -> f64 {
    let mut transmissao = 1.0;
    for (obj, c) in objetos.iter().zip(buf) {
        if obj.emissao.is_some() {
            continue;
        }
        // Começando fora, as entradas são os cruzamentos de índice par; dentro, a partida conta
        // como uma e as seguintes são as de índice ímpar.
        let dentro = c.len() % 2 == 1;
        let entradas = usize::from(dentro)
            + c.iter()
                .skip(usize::from(dentro))
                .step_by(2)
                .take_while(|&&t| t < limite)
                .count();
        transmissao *= obj
            .transmissao
            .powi(i32::try_from(entradas).unwrap_or(i32::MAX));
    }
    transmissao
}

/// A fluência (radiância integrada sobre o ângulo) que chega a `ponto`, por canal.
#[must_use]
pub fn fluencia(cena: &CenaDeLuz, ponto: [f64; 2]) -> [f64; 3] {
    let (objetos, pontuais) = montar(cena);
    let ceu = rgb(cena.ceu);
    let mut buf = Vec::new();
    let mut total = [0.0_f64; 3];

    for intervalo in eventos(&objetos, ponto).windows(2) {
        let (a, b) = (intervalo[0], intervalo[1]);
        if b <= a {
            continue;
        }
        let meio = 0.5 * (a + b);
        let raio = lancar(&objetos, ponto, [meio.cos(), meio.sin()], &mut buf);
        let (radiancia, peso) = match raio.acerto {
            None => (ceu, b - a),
            Some((k, _)) => {
                let (l, alcance) = objetos[k].emissao.expect("acerto é emissor");
                let peso = if alcance == f64::INFINITY {
                    b - a
                } else {
                    let f = &objetos[k].fronteira;
                    let mut c = Vec::new();
                    integra::gauss_kronrod(a, b, &mut |th| {
                        cruzamentos(f, ponto, [th.cos(), th.sin()], &mut c);
                        let t = if c.len() % 2 == 1 {
                            0.0
                        } else {
                            c.first().copied().unwrap_or(0.0)
                        };
                        queda(t, alcance)
                    })
                };
                (l, peso)
            }
        };
        let w = raio.transmissao * peso;
        for ch in 0..3 {
            total[ch] += radiancia[ch] * w;
        }
    }

    for q in &pontuais {
        let v = [q.centro[0] - ponto[0], q.centro[1] - ponto[1]];
        let r = v[0].hypot(v[1]);
        if r == 0.0 {
            for (tot, &i) in total.iter_mut().zip(&q.intensidade) {
                if i > 0.0 {
                    *tot = f64::INFINITY;
                }
            }
            continue;
        }
        let raio = lancar(&objetos, ponto, [v[0] / r, v[1] / r], &mut buf);
        if raio.acerto.is_some_and(|(_, t)| t < r) {
            continue;
        }
        // Até ao ponto, só contam as entradas ANTES dele.
        let w = transmissao_ate(&objetos, &buf, r) * queda(r, q.alcance) / r;
        for (tot, &i) in total.iter_mut().zip(&q.intensidade) {
            *tot += i * w;
        }
    }
    total
}

/// Os ângulos-evento vistos de `p`, ordenados em `[0, 2π]`, com `0` e `2π` nas pontas.
fn eventos(objetos: &[Objeto], p: [f64; 2]) -> Vec<f64> {
    let mut ang = vec![0.0, TAU];
    for obj in objetos {
        match &obj.fronteira {
            Fronteira::Poligono(v) => ang.extend(v.iter().map(|&q| angulo(p, q))),
            Fronteira::Disco { c, r } => {
                let d = (c[0] - p[0]).hypot(c[1] - p[1]);
                if d > *r {
                    let base = (c[1] - p[1]).atan2(c[0] - p[0]);
                    let delta = (r / d).asin();
                    ang.push((base - delta).rem_euclid(TAU));
                    ang.push((base + delta).rem_euclid(TAU));
                }
            }
        }
    }
    let mut pts = Vec::new();
    for (k, e) in objetos.iter().enumerate() {
        let Some((_, alcance)) = e.emissao else {
            continue;
        };
        for (j, o) in objetos.iter().enumerate() {
            if j != k {
                intersecoes(&e.fronteira, &o.fronteira, &mut pts);
            }
        }
        if alcance.is_finite() {
            let circulo = Fronteira::Disco { c: p, r: alcance };
            intersecoes(&e.fronteira, &circulo, &mut pts);
        }
    }
    ang.extend(pts.iter().map(|&q| angulo(p, q)));
    ang.sort_by(f64::total_cmp);
    ang.dedup();
    ang
}
