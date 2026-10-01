//! ⭐⭐⭐ **A BOLA QUE ROLA — nenhum canto interno da pele fica mais apertado que um raio.**
//!
//! # O defeito, medido (report do dono de 2026-09-30, três fotos da junta de cima a `84°`–`89°`)
//!
//! *«melhor mas ainda inconsistente. veja que o ângulo da linha arredonda demais, não é progressivo.
//! e veja que ainda produz artefatos circulares»* — e a sonda `diag_a_zona_da_dobra` mostrou que as
//! três fotos são UMA lei a faltar, não três defeitos:
//! - **Antes do contacto a pele já aperta até ao BICO.** O raio côncavo do lado de dentro vai de
//!   `0,12` raio a `85°` para `0,002` a `92°` (a dobra do mapa, `det J = |1 − θ̄′·r|`) **sem o
//!   contorno se cruzar** — e o arredondamento da F40 só corria no cruzamento. O bico é a foto 1, e
//!   o fundo a aparecer dentro dele, com a ponta redonda da junta do traço, é o «artefato circular»
//!   da foto 3.
//! - **O cruzamento LIGA e DESLIGA de grau para grau** (`88°`–`89°` cruza, `90°`–`92°` não, `93°`
//!   cruza): o canto saltava de bico para arco e de volta.
//! - **E o arco de um vinco raso era ENORME** (o piso `r` da F40 dava raio `r/tan(α/2)`), que é o
//!   «arredonda demais» da foto 2.
//!
//! # A lei: o FECHO morfológico por uma bola de raio `r`, só no lado CÔNCAVO
//!
//! Uma bola de raio `r` rola por FORA do contorno; onde ela não cabe (um canto, um bico, uma curva
//! mais apertada que `r`), o contorno passa a ser o arco dela. Isto é **contínuo na forma** — o
//! canto aperta aos poucos com a dobra, fica em `r` quando passaria dele, e a curva é a mesma antes
//! e depois de o contorno se cruzar —, e degenera nos casos que já estavam certos: um vinco de
//! viragem `α` recebe o arco tangente a `r·tan(α/2)` de cada lado (o da F40, sem o piso); uma curva
//! mais larga que `r` fica **byte-idêntica**.
//!
//! O centro da bola é onde as duas paralelas do contorno a `r` para FORA se cruzam (a cauda de
//! andorinha do offset), e os pontos de toque são os pés dele na curva. Só o que é CÔNCAVO
//! semeia — uma quina convexa nunca é tocada, e a régua da área o prova (o fecho só ACRESCENTA).
//!
//! ⛔ **As quinas do ARTISTA ficam** (um nó do desenho que já virava assim, acima de
//! [`crate::overlap::PAREDE_MINIMA`]): elas não semeiam e a procura não passa por elas.
//! ⚠️ **Divergência DECLARADA:** uma curva côncava LISA do desenho em repouso mais apertada que `r`
//! (`1 %` da diagonal) também é alargada até `r` — sem correspondência entre o desenho assado e o de
//! repouso, a bola não sabe que ela já era assim. Numa forma sem curvas côncavas assim apertadas (a
//! barra da cena, o `RoundRect`) o repouso sai byte-idêntico.

use kurbo::{CubicBez, ParamCurve, ParamCurveDeriv, Point, Vec2};
use ph2d_vec_scene::VecVertex;

/// Quantas amostras por segmento lê a bola — `32`. O assado junto da junta tem segmentos de
/// `~0,01` numa peça de `4 m`, e as peças longas `~0,9`: a `32` o passo mais longo é `0,7` raio.
const AMOSTRAS: usize = 32;

/// Até onde a bola procura o toque de cada lado da zona apertada, em raios — `32`, a distância do
/// toque num vinco de `176°` (`tan(88°) ≈ 29`). ⚠️ `8` (um vinco de `165°`) foi a 1.ª redacção e o
/// entalhe do gate (`166°`, o da pose das fotos) ficava FORA dela: o vinco não era tocado. Um bico
/// ainda mais fechado é um bico de verdade, e a procura não atravessa a peça atrás dele.
const ALCANCE: f64 = 32.0;

/// Só é APERTADA uma curva abaixo de `0,99` do raio — o arco da própria bola é cúbico, e a cúbica
/// de um quarto de círculo fica até `2,7e-4` para dentro dele: com o limiar em `1` rolar a bola
/// outra vez sobre o arco dela trocava-o por um arco quase igual, e o fecho deixava de ser
/// IDEMPOTENTE (há gate).
const APERTO: f64 = 0.99;

/// Um ponto do contorno a menos de `FOLGA_DA_BOLA · r` do centro põe a bola DENTRO da forma — `0,999`.
/// Os toques ficam a `r` exacto e as amostras vizinhas deles a `≥ r`; a folga é o erro do centro
/// das cordas amostradas (`~1e-4 r`, medido na dobra em C a `135°`) com uma ordem de margem.
const FOLGA_DA_BOLA: f64 = 0.999;

/// Uma amostra do contorno.
#[derive(Clone, Copy)]
struct Amostra {
    seg: usize,
    t: f64,
    p: Point,
    tan: Vec2,
    /// O comprimento acumulado ao longo da poligonal das amostras.
    s: f64,
}

/// Um trecho trocado pelo arco da bola.
#[derive(Clone, Copy, Debug)]
struct Vao {
    seg_a: usize,
    t_a: f64,
    seg_b: usize,
    t_b: f64,
    centro: Point,
}

fn unit(v: Vec2) -> Option<Vec2> {
    let l = v.hypot();
    (l > 1e-12).then(|| v / l)
}

fn pt(a: [f64; 2]) -> Point {
    Point::new(a[0], a[1])
}

fn arr(p: Point) -> [f64; 2] {
    [p.x, p.y]
}

fn segmento(verts: &[VecVertex], j: usize) -> CubicBez {
    let n = verts.len();
    let (c, q) = (&verts[j], &verts[(j + 1) % n]);
    CubicBez::new(
        pt(c.anchor),
        pt(c.out_handle),
        pt(q.in_handle),
        pt(q.anchor),
    )
}

/// A tangente de avanço no parâmetro `t` do segmento `j` — pela derivada, e nas pontas (ou onde a
/// derivada se anula) pela que o traço lê.
fn tangente(verts: &[VecVertex], j: usize, t: f64) -> Vec2 {
    let n = verts.len();
    let fallback = || {
        let (ent, sai) = if t < 0.5 {
            (
                None,
                crate::overlap::tangentes_do_vertice(verts, j).map(|x| x.1),
            )
        } else {
            (
                crate::overlap::tangentes_do_vertice(verts, (j + 1) % n).map(|x| x.0),
                None,
            )
        };
        let d = ent.or(sai).unwrap_or([1.0, 0.0]);
        Vec2::new(d[0], d[1])
    };
    if t <= 0.0 || t >= 1.0 {
        return fallback();
    }
    unit(segmento(verts, j).deriv().eval(t).to_vec2()).unwrap_or_else(fallback)
}

#[path = "bola_toque.rs"]
mod toque;
use toque::{ArcoDoNo, cruza_segmentos, pe};

/// **O vértice `i` é uma QUINA DO ARTISTA?** — está sobre um nó de `protegidos` (ao bit) e vira AGORA
/// mais que a `PAREDE_MINIMA` e não mais que a viragem que esse nó tinha (+ `VINCO_MINIMO`). A regra
/// de quem fica em quina, com dois leitores: a bola e o desfazer dos ganchos ([`crate::gancho`]).
pub(crate) fn quina_do_artista(
    verts: &[VecVertex],
    i: usize,
    protegidos: &[([f64; 2], f64)],
) -> bool {
    let a = verts[i].anchor;
    let vira = crate::overlap::viragem_do_vertice(verts, i).unwrap_or(0.0);
    protegidos.iter().any(|(o, v)| {
        (o[0] - a[0]).abs() <= 1e-12 * (1.0 + a[0].abs())
            && (o[1] - a[1]).abs() <= 1e-12 * (1.0 + a[1].abs())
            // ⚠️ Não há `v > PAREDE_MINIMA` à parte, e não é descuido: as duas linhas de baixo
            // já o implicam (`v ≥ vira − 1° > 14°`), e a mutação que o apagava SOBREVIVEU.
            // ⚠️ E AINDA é uma quina: a união pode deixar liso um nó que no desenho virava
            // (medido na dobra em C a `135°`: um nó de mais de `15°` sai a `0,5°`), e uma
            // parede ali prendia a procura a `0,019` do vinco — que ficava em quina.
            && vira > crate::overlap::PAREDE_MINIMA
            && vira <= v + crate::overlap::VINCO_MINIMO
    })
}

/// ⭐⭐⭐ **Rola a bola de raio `raio` por fora do contorno fechado `verts`** e troca cada zona
/// côncava onde ela não cabe pelo arco dela. `protegidos` são os nós do desenho com a viragem que
/// tinham (as quinas do artista); `solda` é a escala abaixo da qual uma troca não se faz (um vão
/// mais curto que a `solda` é ruído do assado, e trocá-lo deixaria um segmento na solda).
///
/// Nada a trocar ⇒ devolve `verts` intacto, ao bit.
#[must_use]
pub fn rola_a_bola(
    verts: Vec<VecVertex>,
    protegidos: &[([f64; 2], f64)],
    raio: f64,
    solda: f64,
) -> Vec<VecVertex> {
    rola(verts, protegidos, raio, solda, Lado::Fora)
}

/// ⭐⭐ **A mesma bola a rolar por DENTRO do contorno** — o que o fecho da forma faz num BURACO
/// (F44, report do dono de 2026-10-01 com três fotos: *«quando uma parte do membro se sobrepõe a
/// outra formando uma ilha, nessa ilha as quinas ainda não estão corretas»*).
///
/// Um buraco é vazio por DENTRO do contorno dele, logo os cantos dele que são CONVEXOS como forma
/// são os vincos CÔNCAVOS da pele — é ali que a junta do traço abre o espinho para dentro do
/// preenchimento. O lado de fora da bola sai da orientação do PRÓPRIO contorno (pela área dele), e
/// é por isso que esta porta existe em vez de inverter a ordem dos vértices: inverter não muda o
/// lado que a área diz.
#[must_use]
pub fn rola_a_bola_por_dentro(
    verts: Vec<VecVertex>,
    protegidos: &[([f64; 2], f64)],
    raio: f64,
    solda: f64,
) -> Vec<VecVertex> {
    rola(verts, protegidos, raio, solda, Lado::Dentro)
}

/// De que lado do contorno o vazio está: por FORA (o contorno de fora de uma forma) ou por DENTRO
/// (um buraco).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lado {
    Fora,
    Dentro,
}

#[allow(clippy::too_many_lines)]
fn rola(
    verts: Vec<VecVertex>,
    protegidos: &[([f64; 2], f64)],
    raio: f64,
    solda: f64,
    lado: Lado,
) -> Vec<VecVertex> {
    let n = verts.len();
    if n < 3 || !raio.is_finite() || raio <= 0.0 {
        return verts;
    }
    let parede: Vec<bool> = (0..n)
        .map(|i| quina_do_artista(&verts, i, protegidos))
        .collect();
    // As amostras: `AMOSTRAS + 1` por segmento, com as duas pontas — a do fim com a tangente de
    // ENTRADA do nó seguinte, a do começo com a de SAÍDA. Um vinco fica assim como uma aresta de
    // comprimento zero entre duas tangentes.
    let k1 = AMOSTRAS + 1;
    let m = n * k1;
    let mut am: Vec<Amostra> = Vec::with_capacity(m);
    for j in 0..n {
        let c = segmento(&verts, j);
        for k in 0..=AMOSTRAS {
            #[allow(clippy::cast_precision_loss)]
            let t = k as f64 / AMOSTRAS as f64;
            let p = if k == 0 {
                c.p0
            } else if k == AMOSTRAS {
                c.p3
            } else {
                c.eval(t)
            };
            am.push(Amostra {
                seg: j,
                t,
                p,
                tan: tangente(&verts, j, t),
                s: 0.0,
            });
        }
    }
    for i in 1..m {
        am[i].s = am[i - 1].s + (am[i].p - am[i - 1].p).hypot();
    }
    let perimetro = am[m - 1].s + (am[0].p - am[m - 1].p).hypot();
    // A orientação: o lado de FORA é a direita num contorno anti-horário.
    let area2: f64 = (0..m)
        .map(|i| am[i].p.to_vec2().cross(am[(i + 1) % m].p.to_vec2()))
        .sum();
    let sinal = match lado {
        Lado::Fora => area2.signum(),
        Lado::Dentro => -area2.signum(),
    };
    if sinal == 0.0 {
        return verts;
    }
    let fora: Vec<Point> = am
        .iter()
        .map(|a| a.p + Vec2::new(a.tan.y, -a.tan.x) * (sinal * raio))
        .collect();
    // ⭐ F46 (o entalhe estreito do braço dobrado de volta, medido a `(176°, 100°)`): quando a bola
    // pousa nas DUAS bordas da boca de uma fenda mais estreita que ela, os toques são dois nós
    // CONVEXOS, e o centro é onde os dois ARCOS de raio `raio` à volta deles se cruzam. A corda
    // entre as normais de um nó passa por DENTRO do arco: o cruzamento de duas cordas punha o centro
    // perto demais da boca, a bola nunca estava vazia, e a fenda ficava aberta.
    let arco_de = |e: usize| -> Option<ArcoDoNo> {
        let (a, b) = (&am[e], &am[(e + 1) % m]);
        if a.seg == b.seg {
            return None;
        }
        let dth = a.tan.cross(b.tan).atan2(a.tan.dot(b.tan));
        (dth * sinal > 1e-9).then(|| {
            let n = |t: Vec2| Vec2::new(t.y, -t.x) * sinal;
            ArcoDoNo {
                c: b.p,
                n0: n(a.tan),
                n1: n(b.tan),
            }
        })
    };
    // A aresta `e` (amostra `e` → `e + 1`) sobre um nó que é quina do artista é parede.
    let aresta_parede = |e: usize| e % k1 == AMOSTRAS && parede[(e / k1 + 1) % n];
    // As sementes: arestas CÔNCAVAS mais apertadas que `raio`.
    let apertada: Vec<bool> = (0..m)
        .map(|e| {
            if aresta_parede(e) {
                return false;
            }
            let (a, b) = (&am[e], &am[(e + 1) % m]);
            let dth = a.tan.cross(b.tan).atan2(a.tan.dot(b.tan));
            dth * sinal < -1e-9 && (b.p - a.p).hypot() < APERTO * raio * dth.abs()
        })
        .collect();
    if !apertada.contains(&true) {
        return verts;
    }
    // As corridas de arestas apertadas seguidas, `[lo, hi]` em índices de aresta (sem volta).
    let mut corridas: Vec<(usize, usize)> = Vec::new();
    let mut e = 0;
    while e < m {
        if apertada[e] {
            let lo = e;
            while e + 1 < m && apertada[e + 1] {
                e += 1;
            }
            corridas.push((lo, e));
        }
        e += 1;
    }
    let dist_s = |de: usize, ate: usize| {
        // O comprimento de `de` para `ate` avançando.
        let d = am[ate].s - am[de].s;
        if d >= 0.0 { d } else { d + perimetro }
    };
    // ⚠️ F44: cada lado procura no máximo MEIA VOLTA. Num contorno curto (uma ilha de perímetro
    // `0,89` contra um alcance de `32 r ≈ 1,2`, medido) o lado esquerdo comia a volta inteira e o
    // direito ficava com ZERO arestas — não havia par onde a bola pousar e o canto ficava em bico.
    // O contorno de fora não muda ao bit: o perímetro de uma curva fechada é pelo menos o dobro da
    // maior largura dela, logo `≥ √2·diagonal = 141 r` contra os `64 r` que as duas janelas somam.
    let janela = (ALCANCE * raio).min(0.5 * perimetro);
    // Procura o toque da bola a volta da corrida `[lo, hi]`.
    let procura = |lo: usize, hi: usize| -> Option<Vao> {
        let mut esq: Vec<usize> = Vec::new();
        let mut i = lo;
        loop {
            i = (i + m - 1) % m;
            if aresta_parede(i) || i == hi || dist_s((i + 1) % m, lo) > janela {
                break;
            }
            esq.push(i);
        }
        let mut dir: Vec<usize> = Vec::new();
        let mut j = hi;
        loop {
            j = (j + 1) % m;
            if aresta_parede(j) || j == lo || esq.contains(&j) || dist_s((hi + 1) % m, j) > janela {
                break;
            }
            dir.push(j);
        }
        let mut candidatos: Vec<(f64, usize, f64, usize, f64, Point)> = Vec::new();
        for &i in &esq {
            let ci = dist_s((i + 1) % m, lo);
            for &j in &dir {
                let custo = ci + dist_s((hi + 1) % m, j);
                let (pi, qi) = (fora[i], fora[(i + 1) % m]);
                let (pj, qj) = (fora[j], fora[(j + 1) % m]);
                // ⭐ F46: o offset de um nó CONVEXO é um ARCO de raio `raio` à volta dele, nunca a
                // corda entre as duas normais (ver [`arco_de`]).
                match (arco_de(i), arco_de(j)) {
                    (None, None) => {
                        if let Some((u, w)) = cruza_segmentos(pi, qi, pj, qj) {
                            candidatos.push((custo, i, u, j, w, pi + (qi - pi) * u));
                        }
                    }
                    (Some(ai), None) => {
                        for (c, w) in ai.com_segmento(pj, qj, raio) {
                            candidatos.push((custo, i, 0.0, j, w, c));
                        }
                    }
                    (None, Some(aj)) => {
                        for (c, u) in aj.com_segmento(pi, qi, raio) {
                            candidatos.push((custo, i, u, j, 0.0, c));
                        }
                    }
                    (Some(ai), Some(aj)) => {
                        for c in ai.com_arco(&aj, raio) {
                            candidatos.push((custo, i, 0.0, j, 0.0, c));
                        }
                    }
                }
            }
        }
        // ⛔⛔ **A bola tem de estar VAZIA** — nenhum ponto do contorno dentro dela. O cruzamento
        // mais BARATO é o toque certo num vinco simples, e num vinco com um DENTE convexo por
        // dentro (mais estreito que a bola) ele pousa num dos vales e atravessa o outro lado:
        // medido, dois arcos que se cruzam por cima do dente e uma meia-volta de `180°`. A bola
        // certa pousa nos dois flancos e engole o dente — é o candidato seguinte que está vazio.
        candidatos.sort_by(|a, b| a.0.total_cmp(&b.0));
        let vazia = |c: Point| am.iter().all(|a| (a.p - c).hypot() >= FOLGA_DA_BOLA * raio);
        let refina = |&(_, i, u, j, w, c): &(f64, usize, f64, usize, f64, Point)| -> Vao {
            let pe_de = |e: usize, f: f64| -> (usize, f64) {
                let (a, b) = (&am[e], &am[(e + 1) % m]);
                if a.seg != b.seg {
                    // A aresta de um nó: o toque é o próprio nó.
                    return (b.seg, 0.0);
                }
                let dt = 1.0 / AMOSTRAS as f64;
                let t0 = a.t + (b.t - a.t) * f;
                let cub = segmento(&verts, a.seg);
                (a.seg, pe(&cub, c, (t0 - dt).max(0.0), (t0 + dt).min(1.0)))
            };
            let (seg_a, mut t_a) = pe_de(i, u);
            let (seg_b, mut t_b) = pe_de(j, w);
            let mut c = c;
            // ⭐ O centro EXACTO: o ponto a `raio` para fora dos DOIS toques ao mesmo tempo, por Newton
            // sobre `(t_a, t_b)`. ⚠️ O cruzamento das paralelas AMOSTRADAS (cordas) deixa o centro
            // desviado, o arco sai um nada mais apertado que `raio`, e rolar a bola outra vez trocava
            // o arco dela — medido na dobra em C a `135°`, um toque a andar `1,5e-4`. Um toque num
            // NÓ (`t = 0` de uma aresta de nó) não se refina: ali o toque é o próprio nó.
            let para_fora = |seg: usize, t: f64| {
                let tg = tangente(&verts, seg, t);
                segmento(&verts, seg).eval(t) + Vec2::new(tg.y, -tg.x) * (sinal * raio)
            };
            let nos_a = am[i].seg != am[(i + 1) % m].seg;
            let nos_b = am[j].seg != am[(j + 1) % m].seg;
            if !nos_a && !nos_b {
                let (mut ta, mut tb) = (t_a, t_b);
                let mut ok = false;
                for _ in 0..24 {
                    let f = para_fora(seg_a, ta) - para_fora(seg_b, tb);
                    if f.hypot() < 1e-13 * raio.max(1.0) {
                        ok = true;
                        break;
                    }
                    let h = 1e-7;
                    let da =
                        (para_fora(seg_a, (ta + h).min(1.0 - 1e-12)) - para_fora(seg_a, ta)) / h;
                    let db =
                        (para_fora(seg_b, tb) - para_fora(seg_b, (tb + h).min(1.0 - 1e-12))) / h;
                    let det = da.cross(db);
                    if det.abs() < 1e-300 {
                        break;
                    }
                    // Resolve `da·x + db·y = −f`.
                    let x = -f.cross(db) / det;
                    let y = -da.cross(f) / det;
                    ta = (ta + x).clamp(1e-12, 1.0 - 1e-12);
                    tb = (tb + y).clamp(1e-12, 1.0 - 1e-12);
                }
                let novo = para_fora(seg_a, ta);
                if ok && (novo - c).hypot() < raio {
                    (t_a, t_b, c) = (ta, tb, novo);
                }
            }
            Vao {
                seg_a,
                t_a,
                seg_b,
                t_b,
                centro: c,
            }
        };
        if let Some(k) = candidatos.iter().find(|k| vazia(k.5)) {
            return Some(refina(k));
        }
        // ⭐ F46 (o fundo do vinco do braço dobrado de volta, medido a `(176°, 142°)`): quando a bola
        // pousa nas duas paredes de um canal que se fecha, o centro tirado das CORDAS amostradas
        // fica a `0,2 %`–`0,3 %` de raio da curva — mais que a `FOLGA_DA_BOLA` — e TODOS os
        // candidatos liam-se ocupados: o canal inteiro ficava aberto até ao fundo. ⇒ só quando
        // nenhum passa, cada candidato é julgado pelo centro EXACTO. ⚠️ A ordem é load-bearing: com
        // um candidato vazio pelas cordas a resposta é a de sempre, ao bit.
        candidatos.iter().map(refina).find(|v| vazia(v.centro))
    };
    let ponto = |seg: usize, t: f64| segmento(&verts, seg).eval(t);
    let g = |seg: usize, t: f64| seg as f64 + t;
    let dentro = |v: &Vao, x: f64| {
        let (a, b) = (g(v.seg_a, v.t_a), g(v.seg_b, v.t_b));
        if a <= b {
            x > a && x < b
        } else {
            x > a || x < b
        }
    };
    let mut vaos: Vec<Vao> = Vec::new();
    let mut fontes: Vec<(usize, usize)> = Vec::new();
    for &(lo, hi) in &corridas {
        // Uma corrida cuja viragem toda cabe na solda é ruído do assado — não se procura.
        let giro: f64 = (lo..=hi)
            .map(|e| {
                let (a, b) = (&am[e], &am[(e + 1) % m]);
                a.tan.cross(b.tan).atan2(a.tan.dot(b.tan)).abs()
            })
            .sum();
        // ⚠️ O corte é UMA solda de corda (vinco abaixo de `~5,7°`), não duas: com duas, um vinco
        // de `9°` da união ficava em quina (medido na dobra em C a `125°`); as micro-quinas do
        // assado (`1,4°`–`1,9°`) ficam abaixo de uma com folga (`0,033 r` contra `0,1 r`).
        if 2.0 * raio * (0.5 * giro.min(3.1)).tan() < solda {
            continue;
        }
        if let Some(v) = procura(lo, hi)
            && (ponto(v.seg_a, v.t_a) - ponto(v.seg_b, v.t_b)).hypot() >= solda
        {
            vaos.push(v);
            fontes.push((lo, hi));
        }
    }
    // Duas zonas apertadas vizinhas podem achar a MESMA bola (medido na dobra em Z a `80°`): um vão
    // contido noutro sai.
    let contido = |a: &Vao, b: &Vao| {
        let (xa, xb) = (g(b.seg_a, b.t_a), g(b.seg_b, b.t_b));
        let dentro_ou_ponta =
            |x: f64| dentro(a, x) || x == g(a.seg_a, a.t_a) || x == g(a.seg_b, a.t_b);
        dentro_ou_ponta(xa) && dentro_ou_ponta(xb)
    };
    let mut k = 0;
    while k < vaos.len() {
        let engolido = (0..vaos.len()).any(|o| {
            o != k && contido(&vaos[o], &vaos[k]) && (o < k || !contido(&vaos[k], &vaos[o]))
        });
        if engolido {
            vaos.remove(k);
            fontes.remove(k);
        } else {
            k += 1;
        }
    }
    // Dois vãos que se sobrepõem são UMA reentrância: procura-se de novo a partir das duas.
    let mut mudou = true;
    let mut voltas = 0;
    while mudou && voltas < n {
        mudou = false;
        voltas += 1;
        'fora: for x in 0..vaos.len() {
            for y in 0..vaos.len() {
                if x == y {
                    continue;
                }
                let (a, b) = (vaos[x], vaos[y]);
                if dentro(&a, g(b.seg_a, b.t_a)) || dentro(&a, g(b.seg_b, b.t_b)) {
                    // ⚠️ A corrida junta vai da MAIS À ESQUERDA à MAIS À DIREITA das duas — a 1.ª
                    // redacção tomava o começo de uma e o fim da outra, e quando o vão maior era o
                    // de índice mais alto a procura ficava com `lo > hi` e perdia os DOIS (medido a
                    // `91°`: um vinco de `134°` com uma quina de `13°` colada, e nenhum arco).
                    let (lo, hi) = (fontes[x].0.min(fontes[y].0), fontes[x].1.max(fontes[y].1));
                    let (x, y) = (x.max(y), x.min(y));
                    vaos.remove(x);
                    fontes.remove(x);
                    vaos.remove(y);
                    fontes.remove(y);
                    let r = procura(lo, hi);
                    if let Some(v) = r {
                        vaos.push(v);
                        fontes.push((lo, hi));
                    }
                    mudou = true;
                    break 'fora;
                }
            }
        }
    }
    if vaos.is_empty() {
        return verts;
    }
    vaos.sort_by(|a, b| g(a.seg_a, a.t_a).total_cmp(&g(b.seg_a, b.t_a)));
    // A partida: um nó fora de todo vão — e, quando TODOS os nós estão dentro de um (F44: uma ilha
    // triangular, onde cada nó é um canto e cada canto é um vão), o FIM do arco do primeiro vão,
    // que por construção não está dentro de nenhum. ⛔ Antes, sem nó livre a bola desistia e a
    // ilha ficava com os três bicos.
    let livre = (0..n).find(|&k| {
        vaos.iter()
            .all(|v| !dentro(v, g(k, 0.0)) && g(v.seg_a, v.t_a) != g(k, 0.0))
    });
    let (s0, t0) = livre.map_or((vaos[0].seg_b, vaos[0].t_b), |k| (k, 0.0));
    // O percurso: pedaços de cúbica em ordem, cada um com o vértice que o começa.
    let mut pedacos: Vec<(CubicBez, Option<usize>)> = Vec::new();
    let (mut seg, mut t) = (s0, t0);
    for _ in 0..(4 * n + 4 * vaos.len() + 8) {
        let proximo = vaos
            .iter()
            .filter(|v| v.seg_a == seg && v.t_a >= t)
            .min_by(|a, b| a.t_a.total_cmp(&b.t_a))
            .copied();
        let origem = (t == 0.0).then_some(seg);
        if let Some(v) = proximo {
            if v.t_a - t > 1e-12 {
                let c = segmento(&verts, seg);
                let c = if t == 0.0 && v.t_a == 1.0 {
                    c
                } else {
                    c.subsegment(t..v.t_a)
                };
                pedacos.push((c, origem));
            }
            arco(&verts, &v, &mut pedacos);
            seg = v.seg_b;
            t = v.t_b;
        } else {
            if 1.0 - t > 1e-12 {
                let c = segmento(&verts, seg);
                let c = if t == 0.0 { c } else { c.subsegment(t..1.0) };
                pedacos.push((c, origem));
            }
            seg = (seg + 1) % n;
            t = 0.0;
        }
        if seg == s0 && t == t0 {
            break;
        }
    }
    let q = pedacos.len();
    if q < 3 {
        return verts;
    }
    (0..q)
        .map(|k| {
            let (c, origem) = pedacos[k];
            let ant = pedacos[(k + q - 1) % q].0;
            let mut v = verts[origem.unwrap_or(s0)];
            let (anchor, out_handle, in_handle) = (arr(c.p0), arr(c.p1), arr(ant.p2));
            let igual = origem.is_some()
                && v.anchor == anchor
                && v.out_handle == out_handle
                && v.in_handle == in_handle;
            v.anchor = anchor;
            v.out_handle = out_handle;
            v.in_handle = in_handle;
            if !igual {
                v.kind = crate::classify(&v);
            }
            v
        })
        .collect()
}

/// O arco da bola de `v` em pedaços de no máximo `45°`, tangente à curva nos dois toques.
fn arco(verts: &[VecVertex], v: &Vao, pedacos: &mut Vec<(CubicBez, Option<usize>)>) {
    let a = segmento(verts, v.seg_a).eval(v.t_a);
    let b = segmento(verts, v.seg_b).eval(v.t_b);
    let ta = tangente(verts, v.seg_a, v.t_a);
    let c = v.centro;
    let (ra, rb) = ((a - c).hypot(), (b - c).hypot());
    // O sentido de rotação é o que sai de `a` na tangente da curva.
    let giro = if (a - c).cross(ta) >= 0.0 { 1.0 } else { -1.0 };
    let (ang_a, ang_b) = ((a - c).atan2(), (b - c).atan2());
    let tau = std::f64::consts::TAU;
    let varre = ((ang_b - ang_a) * giro).rem_euclid(tau);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    // ⚠️ Pedaços de `45°` e não de `90°`: a curvatura da cúbica de um quarto de círculo desce a
    // `0,992` do raio (medido), colada ao limiar [`APERTO`], e rolar a bola outra vez lia o arco
    // dela como apertado; a de `45°` fica em `0,9995`.
    let partes = ((varre / std::f64::consts::FRAC_PI_4).ceil() as usize).max(1);
    #[allow(clippy::cast_precision_loss)]
    let passo = varre / partes as f64;
    let no = |k: usize| -> (Point, Vec2, f64) {
        // ⚠️ A tangente é SEMPRE a do círculo, também nos toques: num toque refinado ela é a da
        // curva a `1e-13`, e num toque que caiu num NÓ que já é quina (a bola pousada nela) a
        // tangente da curva ali NÃO é a do círculo — com ela o 1.º pedaço saía torto (raio
        // `0,98 r`) e rolar outra vez trocava-o (medido na junta única a `125°`). A quina fica.
        let tangente_do_circulo = |p: Point| {
            let d = (p - c).normalize();
            Vec2::new(-d.y, d.x) * giro
        };
        if k == 0 {
            return (a, tangente_do_circulo(a), ra);
        }
        if k == partes {
            return (b, tangente_do_circulo(b), rb);
        }
        #[allow(clippy::cast_precision_loss)]
        let f = k as f64 / partes as f64;
        let r = ra + (rb - ra) * f;
        let ang = ang_a + giro * passo * k as f64;
        let d = Vec2::new(ang.cos(), ang.sin());
        (c + d * r, Vec2::new(-d.y, d.x) * giro, r)
    };
    let alca = (4.0 / 3.0) * (0.25 * passo).tan();
    for k in 0..partes {
        let (p0, t0, r0) = no(k);
        let (p1, t1, r1) = no(k + 1);
        pedacos.push((
            CubicBez::new(p0, p0 + t0 * (alca * r0), p1 - t1 * (alca * r1), p1),
            None,
        ));
    }
}

#[cfg(test)]
#[path = "bola_tests.rs"]
mod tests;
