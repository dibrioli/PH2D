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
//! Semântica (a GPU tem de responder a MESMA pergunta): os CORPOS que brilham (polígonos: sprites e
//! formas) são OPACOS (o raio termina no primeiro); uma LÂMPADA (disco, a `Luz`) é luz e não corpo —
//! o raio que a atravessa soma a radiância dela e segue (smoke do dono, 10/10); um oclusor multiplica
//! por `1 − opacidade` a cada ENTRADA do raio; o céu chega pelas direções que escapam. Por que este método e não quadratura: decidido pelos gates
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
    /// `Some` = emissor.
    emissao: Option<([f64; 3], f64)>,
    /// O emissor é um CORPO (termina o raio)? A lâmpada (disco) não é.
    opaco: bool,
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
                opaco: false,
                transmissao: 1.0,
            }),
            Emissor::Poligono {
                vertices,
                radiancia,
                alcance_m,
            } => objetos.push(Objeto {
                fronteira: Fronteira::Poligono(pontos(vertices)),
                emissao: Some((rgb(*radiancia), f64::from(*alcance_m))),
                opaco: true,
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
            opaco: false,
            transmissao: 1.0 - f64::from(o.opacidade),
        });
    }
    (objetos, pontuais)
}

/// O que um raio encontra: o corpo que o termina (índice e distância), a transmissão até lá, e as
/// LÂMPADAS que ele atravessa antes (índice e a transmissão até a entrada em cada uma).
struct Raio {
    acerto: Option<(usize, f64)>,
    transmissao: f64,
    lampadas: Vec<(usize, f64)>,
}

/// Onde o raio entra no objeto: `0` partindo de dentro; `None` se não o cruza.
fn entrada(c: &[f64]) -> Option<f64> {
    if c.len() % 2 == 1 {
        Some(0.0)
    } else {
        c.first().copied()
    }
}

fn lancar(objetos: &[Objeto], o: [f64; 2], d: [f64; 2], buf: &mut Vec<Vec<f64>>) -> Raio {
    buf.resize_with(objetos.len(), Vec::new);
    let mut acerto: Option<(usize, f64)> = None;
    for (k, obj) in objetos.iter().enumerate() {
        cruzamentos(&obj.fronteira, o, d, &mut buf[k]);
        if obj.emissao.is_some()
            && obj.opaco
            && let Some(t) = entrada(&buf[k])
            && acerto.is_none_or(|(_, ta)| t < ta)
        {
            acerto = Some((k, t));
        }
    }
    let limite = acerto.map_or(f64::INFINITY, |(_, t)| t);
    let lampadas = objetos
        .iter()
        .enumerate()
        .filter(|(_, obj)| obj.emissao.is_some() && !obj.opaco)
        .filter_map(|(k, _)| entrada(&buf[k]).filter(|&t| t < limite).map(|t| (k, t)))
        .map(|(k, t)| (k, transmissao_ate(objetos, buf, t)))
        .collect();
    Raio {
        acerto,
        transmissao: transmissao_ate(objetos, buf, limite),
        lampadas,
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

/// A radiância de um emissor e o peso angular dele em `[a, b]` visto de `ponto`: a largura, ou, com
/// alcance finito, a `queda` integrada (Gauss–Kronrod) à distância da entrada.
fn peso_do_emissor(obj: &Objeto, ponto: [f64; 2], a: f64, b: f64) -> ([f64; 3], f64) {
    let (l, alcance) = obj.emissao.expect("um emissor");
    if alcance == f64::INFINITY {
        return (l, b - a);
    }
    let mut c = Vec::new();
    let peso = integra::gauss_kronrod(a, b, &mut |th| {
        cruzamentos(&obj.fronteira, ponto, [th.cos(), th.sin()], &mut c);
        queda(entrada(&c).unwrap_or(0.0), alcance)
    });
    (l, peso)
}

/// A luz que uma superfície DE FRENTE (sprite ou vetor, no plano da cena) recebe em `ponto`:
/// `F / 2π`, a radiância média sobre a volta — o forno branco (plano §2.3). O receptor multiplica
/// o albedo linear por isto.
#[must_use]
pub fn luz_em(cena: &CenaDeLuz, ponto: [f64; 2]) -> [f64; 3] {
    fluencia(cena, ponto).map(|f| f / TAU)
}

/// O objeto é um corpo OPACO (τ dentro = 0): um que brilha (polígono) ou um oclusor de opacidade 1.
fn opaco(o: &Objeto) -> bool {
    o.opaco || (o.emissao.is_none() && o.transmissao <= 0.0)
}

/// ⭐ **O interior de um opaco recebe a luz do EXTERIOR mais próximo** (F3, W5a): dentro de um corpo
/// opaco a resposta é a do ponto de fora mais perto — logo além do pé de `p` na aresta opaca mais
/// próxima cujo pé não cai dentro de outro opaco. Fora de todo opaco, `p`.
fn fora_dos_opacos(objetos: &[Objeto], p: [f64; 2]) -> [f64; 2] {
    let mut buf = Vec::new();
    let mut dentro_de_algum = |q: [f64; 2]| {
        objetos
            .iter()
            .filter(|o| opaco(o))
            .any(|o| geometria::dentro(&o.fronteira, q, &mut buf))
    };
    if !dentro_de_algum(p) {
        return p;
    }
    let mut melhor: Option<(f64, [f64; 2])> = None;
    for o in objetos.iter().filter(|o| opaco(o)) {
        let Fronteira::Poligono(v) = &o.fronteira else {
            continue;
        };
        for pe in geometria::pes_nas_arestas(v, p) {
            let d = (pe[0] - p[0]).hypot(pe[1] - p[1]);
            if d <= 0.0 || melhor.is_some_and(|(m, _)| m <= d) {
                continue;
            }
            // 1 nm além da fronteira, para o lado de fora.
            let q = [
                pe[0] + 1e-9 * (pe[0] - p[0]) / d,
                pe[1] + 1e-9 * (pe[1] - p[1]) / d,
            ];
            if !dentro_de_algum(q) {
                melhor = Some((d, q));
            }
        }
    }
    melhor.map_or(p, |(_, q)| q)
}

/// A fluência (radiância integrada sobre o ângulo) que chega a `ponto`, por canal. Dentro de um
/// corpo opaco, a do exterior mais próximo ([`fora_dos_opacos`]).
#[must_use]
pub fn fluencia(cena: &CenaDeLuz, ponto: [f64; 2]) -> [f64; 3] {
    let (objetos, pontuais) = montar(cena);
    let ponto = fora_dos_opacos(&objetos, ponto);
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
        // O fundo (o céu, ou o corpo que termina o raio) e cada lâmpada atravessada antes dele.
        let (fundo, peso) = match raio.acerto {
            None => (ceu, b - a),
            Some((k, _)) => peso_do_emissor(&objetos[k], ponto, a, b),
        };
        let mut somar = |l: [f64; 3], w: f64| {
            for ch in 0..3 {
                total[ch] += l[ch] * w;
            }
        };
        somar(fundo, raio.transmissao * peso);
        for &(k, tr) in &raio.lampadas {
            let (l, peso) = peso_do_emissor(&objetos[k], ponto, a, b);
            somar(l, tr * peso);
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
