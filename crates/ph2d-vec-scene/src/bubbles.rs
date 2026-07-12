//! **Balões** (fala · pensamento · nuvem · explosão · chave) — módulo irmão de
//! [`crate::shapes`].
//!
//! Autorado no espaço unitário ([`crate::space`]): `u` 0→1 da esquerda para a direita,
//! `v` 0→1 do TOPO para a base (a convenção SVG/OOXML de onde as fórmulas vêm). A versão
//! anterior foi escrita em coordenadas de MUNDO (Y-para-cima) com a cabeça na convenção de
//! tela — e todo rabicho nascia apontando para o céu. Aqui não há sinal para errar.
//!
//! ## O que muda em relação ao OOXML (e por quê)
//!
//! A geometria pré-definida do ECMA-376 é a referência — mas em três pontos ela é **ruim**
//! e foi deliberadamente NÃO portada:
//!
//! 1. **O rabo do balão retangular** (`wedgeRectCallout`) prende a base numa grade grossa
//!    de 1/12 da aresta, não tem largura de base ajustável, e é um triângulo de lados
//!    retos. Aqui o rabo é o [`tail_flanks`]: duas cúbicas com raiz misturada
//!    (`ROOT_BIAS`) e bico em agulha — o rabo de quadrinho.
//! 2. **O bico pode sair da caixa** (`ahXY` sem limite). Incompatível com o gizmo, que lê
//!    a bbox. Aqui o corpo cede uma **faixa reservada** (`room`) e o bico vive DENTRO
//!    dela: o rabo não tem como vazar (prova em `speech_rect`).
//! 3. **A nuvem é uma tabela de 11 arcos** escrita à mão (que ainda por cima transborda a
//!    própria caixa). Ela foi decodificada numericamente: todo endpoint de arco está
//!    **exatamente sobre as duas elipses vizinhas** — ou seja, os joins são as
//!    **interseções círculo-círculo**. A nuvem é a fronteira da UNIÃO de discos numa
//!    espinha fechada. É esse algoritmo que está aqui, generalizado (com raio de vale ρ,
//!    que leva a cúspide do OOXML a um contorno G1 de verdade).
//!
//! ## Contenção: medir a curva, não o casco
//!
//! Toda forma daqui termina em [`crate::space::fit`] — o ajuste afim medido na bbox
//! **VERDADEIRA** da curva (âncoras + extremos internos das Béziers). Afim aplicado a uma
//! Bézier é exato nos pontos de controle: nenhuma tangência se perde, nenhum join deixa de
//! ser G1, e a bbox passa a **ser** a caixa do gesto. É o mesmo truque que o OOXML aplica à
//! mão na `star5` — aqui é calculado, para qualquer parâmetro.
//!
//! ## Determinismo
//!
//! Zero RNG em tempo de cozimento. A irregularidade da nuvem e da explosão são **tabelas
//! fixas** indexadas por `i % N`. Uma forma que muda de silhueta a cada re-cook (cada tecla
//! no painel, cada undo, cada reload) seria inutilizável.

use crate::VecPath;
use crate::bubbles_ring::{
    Uvv, place, refit, rot_q, seal, sector, sharp, stitch, tail_flanks, uv_arc, uv_straighten,
    v_add, v_len, v_mul, v_norm, v_sub,
};
use crate::space::{Unit, Uv, add_sub, clamp_window, closed, fit, open};

// ─────────────────────────────────────────────────────────────────────────────
// Balão de fala retangular
// ─────────────────────────────────────────────────────────────────────────────

/// Empurra uma quina do round-rect: o arco de 90°, ou **uma quina viva** quando o raio é
/// nulo (arco de raio zero deixaria duas âncoras coincidentes no path editável).
fn rr_corner(out: &mut Vec<Uvv>, c: Uv, p: f64, q: f64, start_deg: f64, vivo: Uv) {
    if p > 1e-9 && q > 1e-9 {
        out.extend(uv_arc(c, p, q, start_deg, 90.0));
    } else {
        out.push(sharp(vivo));
    }
}

/// **Balão de fala retangular.** Round-rect + rabo, num contorno ÚNICO e provadamente
/// simples.
///
/// O bico é um ponto 2D livre (`tip_u`, `tip_v`) — como nos apps de verdade, e não a grade
/// de 1/12 do OOXML. O setor do bico escolhe por qual lado o rabo sai; a forma é sempre
/// construída com o rabo à DIREITA e girada no fim ([`rot_q`]), então há **um** caminho de
/// código para os quatro lados.
///
/// **Contenção, e por que o contorno não pode se auto-intersectar:** o corpo cede a faixa
/// `[1−room, 1]` e o bico é clampado dentro dela. Todos os pontos de controle dos dois
/// flancos têm `u ≥ 1−room` (a raiz mistura duas direções de `u` não-negativo e o handle do
/// bico só recua na direção do eixo), logo — pelo casco convexo — o rabo INTEIRO vive na
/// faixa e jamais reentra no corpo. Os dois flancos, por sua vez, saem do bico para lados
/// opostos do eixo. O contorno é simples por construção.
///
/// `radius` é MUNDO (a quina é um círculo na tela, não uma elipse); `base_w` e `room` são
/// frações da caixa.
#[must_use]
pub fn speech_rect(
    a: [f64; 2],
    b: [f64; 2],
    radius: f64,
    tip_u: f64,
    tip_v: f64,
    base_w: f64,
    room: f64,
) -> VecPath {
    let u = Unit::of(a, b);
    let (hw, hh) = ((b[0] - a[0]).abs() * 0.5, (b[1] - a[1]).abs() * 0.5);
    let room = room.clamp(0.05, 0.6);
    let w = 1.0 - room; // largura do corpo (canônico: o rabo sai à direita)

    let k = sector((tip_u.clamp(0.0, 1.0), tip_v.clamp(0.0, 1.0)));
    let mut t = rot_q((tip_u.clamp(0.0, 1.0), tip_v.clamp(0.0, 1.0)), (4 - k) % 4);
    t.0 = clamp_window(t.0, w, 1.0); // A FAIXA: o bico não tem como sair da caixa

    // O raio vem em MUNDO; nos dois eixos de autoria ele vira um par (para a quina ser
    // circular na tela). O quarto-de-volta troca os eixos, então os raios trocam junto.
    let (mut ru, mut rv) = if hw > 1e-12 && hh > 1e-12 {
        (radius.max(0.0) / (2.0 * hw), radius.max(0.0) / (2.0 * hh))
    } else {
        (0.0, 0.0)
    };
    if k % 2 == 1 {
        std::mem::swap(&mut ru, &mut rv);
    }
    // Satura preservando o círculo (o MESMO fator nos dois eixos). O teto é 0,45 do
    // semi-eixo, não 0,5: sobra sempre uma aresta reta onde a base do rabo assenta.
    let s = 1.0_f64
        .min(0.45 * w / ru.max(1e-12))
        .min(0.45 / rv.max(1e-12));
    let (p, q) = (ru * s, rv * s);

    // Base do rabo na aresta direita. Nunca cavalga as quinas arredondadas, e ela SEGUE o
    // bico lateralmente — é o que mantém o rabo perpendicular ao corpo (e sem gancho).
    //
    // **A janela COLAPSA** quando a base pedida é maior que a aresta reta que sobrou entre
    // as duas quinas: aí `hb = (1 − 2q)/2`, e o piso `q + hb` e o teto `1 − q − hb` são o
    // MESMO número — em matemática real. Em ponto flutuante o teto cai 1 ulp abaixo do
    // piso e o `f64::clamp` explode (foi o pânico do smoke: `0.5` vs `0.49999999999999994`).
    // Com a janela colapsada não há intervalo, há um PONTO: a base ocupa a aresta inteira.
    let hb = 0.5 * base_w.clamp(0.05, 0.6).min(1.0 - 2.0 * q);
    let cy = clamp_window(t.1, q + hb, 1.0 - q - hb);
    let (pa, pb) = ((w, cy - hb), (w, cy + hb));
    let n_out: Uv = (1.0, 0.0);
    let (a_out, tip, b_in) = tail_flanks(pa, pb, t, n_out, n_out);

    // A caminhada, horária na tela: quina sup-esq → aresta de cima → quina sup-dir →
    // aresta direita (com o rabo emendado NO MEIO dela) → quina inf-dir → aresta de baixo
    // → quina inf-esq → aresta esquerda.
    let mut ring: Vec<Uvv> = Vec::with_capacity(16);
    rr_corner(&mut ring, (p, q), p, q, 180.0, (0.0, 0.0));
    let i_top = ring.len() - 1;
    rr_corner(&mut ring, (w - p, q), p, q, 270.0, (w, 0.0));
    uv_straighten(&mut ring, i_top);
    let i_right = ring.len() - 1;
    ring.push(Uvv {
        a: pa,
        i: pa,
        o: a_out,
        cusp: true,
    });
    ring.push(tip);
    let i_b = ring.len();
    ring.push(Uvv {
        a: pb,
        i: b_in,
        o: pb,
        cusp: true,
    });
    uv_straighten(&mut ring, i_right);
    rr_corner(&mut ring, (w - p, 1.0 - q), p, q, 0.0, (w, 1.0));
    uv_straighten(&mut ring, i_b);
    let i_bot = ring.len() - 1;
    rr_corner(&mut ring, (p, 1.0 - q), p, q, 90.0, (0.0, 1.0));
    uv_straighten(&mut ring, i_bot);
    // A aresta esquerda fecha do último para o primeiro (o `uv_straighten` não dá a volta).
    let last = ring.len() - 1;
    ring[last].o = ring[last].a;
    ring[last].cusp = true;
    ring[0].i = ring[0].a;
    ring[0].cusp = true;

    let mut path = closed(place(&u, k, &ring));
    fit(&u, &mut path);
    path
}

// ─────────────────────────────────────────────────────────────────────────────
// Balão de fala oval
// ─────────────────────────────────────────────────────────────────────────────

/// **Balão de fala oval.** Elipse + o mesmo rabo, num contorno único.
///
/// O que o OOXML acerta aqui — e que é preciso manter — é achar a base do rabo pelo **ângulo
/// PARAMÉTRICO** da elipse (`atan2(D.v/b, D.u/a)`), não pelo polar: com o polar a base
/// desliza para longe do bico assim que a elipse achata. A normal externa também é
/// paramétrica (`(cos t / a, sin t / b)` normalizado), e **não** `(cos t, sin t)`.
///
/// O corpo percorre o caminho LONGO (`360° − 2·wedge`) de `P(tB)` a `P(tA)`; o rabo fecha o
/// resto. `wedge` é a meia-abertura angular da base (o spec usa 11°).
#[must_use]
pub fn speech_oval(
    a: [f64; 2],
    b: [f64; 2],
    tip_u: f64,
    tip_v: f64,
    wedge: f64,
    room: f64,
) -> VecPath {
    let u = Unit::of(a, b);
    let room = room.clamp(0.05, 0.6);
    let w = 1.0 - room;
    let k = sector((tip_u.clamp(0.0, 1.0), tip_v.clamp(0.0, 1.0)));
    let mut t = rot_q((tip_u.clamp(0.0, 1.0), tip_v.clamp(0.0, 1.0)), (4 - k) % 4);
    t.0 = clamp_window(t.0, w, 1.0);

    let half = wedge.clamp(2.0, 45.0).to_radians();
    let c: Uv = (w * 0.5, 0.5);
    let (ra, rb) = (w * 0.5, 0.5); // a elipse INSCRITA no corpo
    let d = v_sub(t, c);
    let pang = (d.1 / rb).atan2(d.0 / ra);
    let (ta, tb) = (pang - half, pang + half);
    let pt = |x: f64| (c.0 + ra * x.cos(), c.1 + rb * x.sin());
    let nrm = |x: f64| v_norm((x.cos() / ra, x.sin() / rb));
    let (pa, pb) = (pt(ta), pt(tb));
    let (a_out, tip, b_in) = tail_flanks(pa, pb, t, nrm(ta), nrm(tb));

    let mut ring = uv_arc(c, ra, rb, tb.to_degrees(), 360.0 - 2.0 * half.to_degrees());
    let last = ring.len() - 1; // === P(tA): o corpo CHEGA aqui
    ring[last].o = a_out;
    ring[last].cusp = true;
    ring.push(tip);
    ring[0].i = b_in; // === P(tB): o corpo SAI daqui (o fecho do contorno)
    ring[0].cusp = true;

    let mut path = closed(place(&u, k, &ring));
    fit(&u, &mut path);
    path
}

// ─────────────────────────────────────────────────────────────────────────────
// Nuvem — a cadeia de arcos (o algoritmo-ouro)
// ─────────────────────────────────────────────────────────────────────────────

/// A irregularidade **MEDIDA** da nuvem do ECMA-376 (`wR / média(wR)` dos 11 arcos). Tabela
/// FIXA, indexada por `i % 11` para qualquer N: sem RNG, sem estado, idêntica a cada cook.
const CLOUD_JITTER: [f64; 11] = [
    1.191, 0.940, 0.770, 0.856, 0.940, 1.195, 1.020, 1.191, 1.362, 0.769, 0.766,
];
/// Gira qual bolha cai onde (o spec nasce por volta daqui).
const CLOUD_PHASE_DEG: f64 = 200.0;
/// Raio da bolha ÷ passo da espinha. Abaixo de 0,5 as vizinhas **não se sobrepõem** e o join
/// não existe — é o piso do parâmetro, não um gosto.
const CLOUD_PUFF: f64 = 0.62;

/// O anel da nuvem: bolhas circulares numa espinha, unidas por círculos de VALE achados por
/// **trilateração** (o join é a interseção círculo-círculo, exatamente o que a tabela do
/// OOXML esconde).
///
/// Para cada par vizinho resolve-se o centro `Q` do círculo de vale (raio ρ), externamente
/// tangente aos dois:
///
/// ```text
/// d = |P₁−P₀| ;  R₀ = r₀+ρ ;  R₁ = r₁+ρ
/// a = (R₀² − R₁² + d²) / 2d ;  h = √(R₀² − a²)      ← exige R₀+R₁ > d (a SOBREPOSIÇÃO)
/// Q = P₀ + a·û ± h·n̂     → a raiz mais LONGE do centro (a de fora)
/// ```
///
/// Os pontos de tangência são colineares com os centros (`T = P + r·normalize(Q−P)`) — é
/// **isso** que torna o join G1: a bolha vira à esquerda, o vale vira à direita, e os dois
/// compartilham a reta tangente.
///
/// * `ρ = 0` ⇒ a trilateração degenera EXATAMENTE na interseção círculo-círculo, os dois
///   pontos de tangência coincidem e o vale some ⇒ a cúspide clássica (= OOXML).
/// * `ρ > 0` ⇒ nuvem G1 de verdade.
///
/// Autorado na caixa QUADRADA `[0,1]²` (espinha circular ⇒ passo equiangular **é**
/// equidistante em arco) e depois ajustado por afim: é literalmente o que o OOXML faz — os
/// 11 arcos dele têm todos a mesma razão `hR/wR = 1.362`, ou seja, a nuvem foi autorada em
/// CÍRCULOS e escalada anisotropicamente.
fn cloud_ring(bumps: usize, puff: f64, valley: f64, irregularity: f64) -> Vec<Uvv> {
    let n = bumps.clamp(5, 24);
    let puff = puff.clamp(0.52, 0.9);
    let rho = valley.clamp(0.0, 0.06);
    let irreg = irregularity.clamp(0.0, 1.5);
    let c: Uv = (0.5, 0.5);

    let jit = |i: usize| 1.0 + irreg * (CLOUD_JITTER[i % 11] - 1.0);
    let jit_max = (0..n).map(jit).fold(1e-6_f64, f64::max);

    // O raio médio é `puff · perímetro(espinha) / N`, e a espinha é recuada pelo raio MÁXIMO
    // (para os discos caberem antes do refit) — um ponto fixo. Na caixa quadrada a espinha é
    // um CÍRCULO, e o ponto fixo tem forma fechada:
    //     s = 0.5 − puff·2π·s·jit_max/N   ⇒   s = 0.5 / (1 + 2π·puff·jit_max/N)
    // Fixar `r_mean` (em vez de derivá-lo de N) é o erro que explode a forma: a espinha
    // cresce, os vãos passam de `rᵢ + rⱼ` e **não existe interseção**.
    let s = 0.5 / (1.0 + std::f64::consts::TAU * puff * jit_max / n as f64);
    let r_mean = puff * std::f64::consts::TAU * s / n as f64;

    let step = std::f64::consts::TAU / n as f64;
    let phase = CLOUD_PHASE_DEG.to_radians();
    let cen: Vec<Uv> = (0..n)
        .map(|i| {
            let (sn, cs) = (phase + step * i as f64).sin_cos();
            (c.0 + s * cs, c.1 + s * sn)
        })
        .collect();
    let mut rad: Vec<f64> = (0..n).map(|i| r_mean * jit(i)).collect();

    // Passe de segurança: o jitter pode afastar duas bolhas pequenas além da sobreposição
    // (0,769 + 0,766 é o par ruim da tabela). Cresce TODOS os raios pelo maior déficit —
    // determinístico, e garante que o join sempre existe.
    let mut grow = 1.0_f64;
    for i in 0..n {
        let j = (i + 1) % n;
        let d = v_len(v_sub(cen[j], cen[i]));
        let need = (d - 2.0 * rho) * 1.02;
        let have = rad[i] + rad[j];
        if have > 1e-12 && have < need {
            grow = grow.max(need / have);
        }
    }
    for r in &mut rad {
        *r *= grow;
    }

    // Os centros de vale (trilateração), um por par vizinho.
    let valleys: Vec<Uv> = (0..n)
        .map(|i| {
            let j = (i + 1) % n;
            let (p0, p1) = (cen[i], cen[j]);
            let d = v_len(v_sub(p1, p0)).max(1e-9);
            let (r0, r1) = (rad[i] + rho, rad[j] + rho);
            let am = (r0 * r0 - r1 * r1 + d * d) / (2.0 * d);
            let h = (r0 * r0 - am * am).max(0.0).sqrt();
            let uh = v_mul(v_sub(p1, p0), 1.0 / d);
            let nh = (-uh.1, uh.0);
            let base = v_add(p0, v_mul(uh, am));
            let (qa, qb) = (v_add(base, v_mul(nh, h)), v_sub(base, v_mul(nh, h)));
            // a raiz de FORA: a de dentro cavaria o vale para dentro da nuvem
            if v_len(v_sub(qa, c)) >= v_len(v_sub(qb, c)) {
                qa
            } else {
                qb
            }
        })
        .collect();

    let t_out = |i: usize| v_add(cen[i], v_mul(v_norm(v_sub(valleys[i], cen[i])), rad[i]));
    let t_in = |i: usize| {
        let prev = (i + n - 1) % n;
        v_add(cen[i], v_mul(v_norm(v_sub(valleys[prev], cen[i])), rad[i]))
    };
    let ang = |p: Uv, o: Uv| (p.1 - o.1).atan2(p.0 - o.0).to_degrees();

    let mut ring: Vec<Uvv> = Vec::with_capacity(n * 4);
    for i in 0..n {
        let (ti, to) = (t_in(i), t_out(i));
        let sweep = sweep_pos(ang(to, cen[i]) - ang(ti, cen[i]));
        stitch(
            &mut ring,
            &uv_arc(cen[i], rad[i], rad[i], ang(ti, cen[i]), sweep),
            rho <= 0.0,
        );
        if rho > 0.0 {
            let j = (i + 1) % n;
            let q = valleys[i];
            let sweep = sweep_short(ang(t_in(j), q) - ang(to, q));
            stitch(&mut ring, &uv_arc(q, rho, rho, ang(to, q), sweep), false);
        }
    }
    seal(&mut ring, rho <= 0.0);
    refit(&mut ring, [0.0, 0.0], [1.0, 1.0]);
    ring
}

/// Normaliza um sweep para `(0°, 360°]` — a bolha é CONVEXA (varre para fora).
fn sweep_pos(d: f64) -> f64 {
    if !d.is_finite() {
        return 0.0;
    }
    let m = d.rem_euclid(360.0);
    if m <= 1e-9 { 360.0 } else { m }
}

/// Normaliza um sweep para `(−180°, 180°]` — o vale é CÔNCAVO e vai pelo caminho CURTO.
fn sweep_short(d: f64) -> f64 {
    if !d.is_finite() {
        return 0.0;
    }
    let m = d.rem_euclid(360.0);
    if m > 180.0 { m - 360.0 } else { m }
}

/// **Nuvem.** Ver [`cloud_ring`]: bolhas numa espinha, unidas por trilateração.
/// `puff` = raio da bolha ÷ passo da espinha (**> 0,5** ou não há sobreposição);
/// `valley` = o raio do filete do vale (0 = cúspide do OOXML, > 0 = contorno G1);
/// `irregularity` escala a tabela de jitter medida do spec.
#[must_use]
pub fn cloud(
    a: [f64; 2],
    b: [f64; 2],
    bumps: f64,
    puff: f64,
    valley: f64,
    irregularity: f64,
) -> VecPath {
    let u = Unit::of(a, b);
    let n = bumps.clamp(5.0, 24.0).round() as usize;
    let ring = cloud_ring(n, puff, valley, irregularity);
    let mut path = closed(place(&u, 0, &ring));
    fit(&u, &mut path);
    path
}

// ─────────────────────────────────────────────────────────────────────────────
// Balão de pensamento
// ─────────────────────────────────────────────────────────────────────────────

/// A faixa que o corpo cede para a corrente de bolhas.
const THOUGHT_ROOM: f64 = 0.34;
/// Raios da corrente: o spec (`cloudCallout`) usa 600/1200/1800 em `ss/21600`, que é
/// **exatamente** `ss/36` … `ss/12` para N = 3.
const BUBBLE_R_MIN: f64 = 1.0 / 36.0;
const BUBBLE_R_MAX: f64 = 1.0 / 12.0;

/// **Balão de pensamento.** A nuvem + a corrente de bolhas que desce até quem pensa.
///
/// A corrente do `cloudCallout` parece um punhado de constantes mágicas. Não é: é uma
/// **cadeia de vãos IGUAIS**. Decodificada, `g9 = 600 + 2·1200 + 2·1800` é o "sólido"
/// (`r₀ + 2·Σ_{i≥1} rᵢ`) e `g11 = (g8 − g9)/3` é a folga repartida em **três vãos iguais** —
/// bolha₀↔bolha₁, bolha₁↔bolha₂ e bolha₂↔nuvem. Generalizando para N:
///
/// ```text
/// rᵢ    = r_min + (r_max − r_min)·i/(N−1)     (i = 0 é a MENOR, no BICO)
/// sólido = r₀ + 2·Σ_{i≥1} rᵢ
/// vão    = (|E − T| − sólido) / N              (N vãos: N−1 entre bolhas + 1 até a nuvem)
/// c₀ = T ;  cᵢ = cᵢ₋₁ + (rᵢ₋₁ + vão + rᵢ)·û
/// ```
///
/// Duas armadilhas, ambas verificadas: `D = T − C` (e **não** `C − T`, que ancora a corrente
/// do lado errado da nuvem), e o ponto `E` sai da elipse que **circunscreve o corpo**, não da
/// espinha da nuvem (a espinha é recuada, e a última bolha nasceria dentro dela).
#[must_use]
pub fn thought(
    a: [f64; 2],
    b: [f64; 2],
    tip_u: f64,
    tip_v: f64,
    bubbles: f64,
    bumps: f64,
) -> VecPath {
    let u = Unit::of(a, b);
    let n_bub = bubbles.clamp(1.0, 6.0).round() as usize;
    let n_bump = bumps.clamp(5.0, 24.0).round() as usize;
    let w = 1.0 - THOUGHT_ROOM;

    // A bolha 0 é centrada NO bico: sem este clamp ela vaza a caixa (um bico em v = 0,99
    // com r_min = 1/36 chega a 1,008 — a asserção pegou).
    let tip0 = (
        tip_u.clamp(BUBBLE_R_MIN, 1.0 - BUBBLE_R_MIN),
        tip_v.clamp(BUBBLE_R_MIN, 1.0 - BUBBLE_R_MIN),
    );
    let k = sector(tip0);
    let mut t = rot_q(tip0, (4 - k) % 4);
    t.0 = clamp_window(t.0, w, 1.0 - BUBBLE_R_MIN);

    let mut ring = cloud_ring(n_bump, CLOUD_PUFF, 0.0, 1.0);
    refit(&mut ring, [0.0, 0.0], [w, 1.0]);

    let c: Uv = (w * 0.5, 0.5);
    let (ra, rb) = (w * 0.5, 0.5);
    let d = v_sub(t, c); // do CENTRO para o BICO — invertido, a corrente ancora do lado oposto
    let pang = (d.1 / rb).atan2(d.0 / ra);
    let e = (c.0 + ra * pang.cos(), c.1 + rb * pang.sin());
    let dir = v_norm(v_sub(e, t));
    let span = v_len(v_sub(e, t));

    let mut r: Vec<f64> = (0..n_bub)
        .map(|i| {
            if n_bub == 1 {
                BUBBLE_R_MIN
            } else {
                BUBBLE_R_MIN + (BUBBLE_R_MAX - BUBBLE_R_MIN) * i as f64 / (n_bub - 1) as f64
            }
        })
        .collect();
    let solid = |r: &[f64]| r[0] + 2.0 * r[1..].iter().sum::<f64>();
    if solid(&r) > span * 0.85 && solid(&r) > 1e-12 {
        let k = span * 0.85 / solid(&r);
        for x in &mut r {
            *x *= k;
        }
    }
    let gap = (span - solid(&r)) / n_bub as f64;

    let mut path = closed(place(&u, k, &ring));
    let mut centre = t;
    for i in 0..n_bub {
        if i > 0 {
            centre = v_add(centre, v_mul(dir, r[i - 1] + gap + r[i]));
        }
        let mut circle = uv_arc(centre, r[i], r[i], 0.0, 360.0);
        circle.pop(); // a volta inteira repete o primeiro vértice
        add_sub(&mut path, place(&u, k, &circle), true);
    }
    // `NonZero` (o default) FUNDE a nuvem e as bolhas num preenchimento só — que é o desenho
    // certo. `EvenOdd` faria de cada bolha um furo.
    fit(&u, &mut path);
    path
}

// ─────────────────────────────────────────────────────────────────────────────
// Explosão / selo irregular
// ─────────────────────────────────────────────────────────────────────────────

/// Jitter RADIAL fixo (24 entradas, `i % 24`) — a irregularidade do `irregularSeal1`.
const BURST_R_JIT: [f64; 24] = [
    1.00, 0.92, 1.06, 0.88, 1.03, 0.95, 1.10, 0.90, 0.98, 1.07, 0.93, 1.02, 0.96, 1.09, 0.89, 1.04,
    0.99, 0.91, 1.05, 0.94, 1.08, 0.97, 1.01, 0.87,
];
/// Jitter ANGULAR fixo. O maior |valor| é 0,33 e o parâmetro vai a 0,9 ⇒ o desvio máximo é
/// 0,297 do passo, **menor que meio passo**: os ângulos nunca trocam de ordem, então o
/// polígono é estrelado em torno do centro e **não pode** se auto-intersectar.
const BURST_A_JIT: [f64; 24] = [
    0.10, -0.22, 0.31, -0.05, 0.18, -0.30, 0.07, 0.26, -0.14, 0.02, -0.27, 0.21, -0.09, 0.29,
    -0.18, 0.13, 0.24, -0.25, 0.05, -0.11, 0.33, -0.03, 0.16, -0.20,
];
/// A amplitude da tabela radial — normaliza `r_jit` para uma fração legível.
const BURST_R_SPAN: f64 = 0.115;
/// A primeira ponta olha para CIMA.
const BURST_PHASE_DEG: f64 = -90.0;

/// **Explosão / grito** (o `irregularSeal` do spec, parametrizado).
///
/// Segmentos RETOS são o desenho **certo** aqui — a explosão tem arestas retas; não é um
/// polígono *aproximando* um arco. O que o OOXML dá são duas tabelas de vértices escritas à
/// mão (24 e 28 pontos); decompondo-as em polar sai o gerador: raios alternados
/// (externo 0,98 / interno 0,556 ⇒ razão ≈ 0,565), passo angular médio de 15° com jitter, e
/// um ajuste final para preencher a caixa — que aqui é o [`crate::space::fit`].
///
/// A irregularidade vem de **tabelas fixas** indexadas pelo vértice: a silhueta é a mesma a
/// cada cook, undo e reload.
#[must_use]
pub fn burst(a: [f64; 2], b: [f64; 2], spikes: f64, inner: f64, r_jit: f64, a_jit: f64) -> VecPath {
    let u = Unit::of(a, b);
    let k = spikes.clamp(5.0, 30.0).round() as usize;
    let inner = inner.clamp(0.2, 0.9);
    let rj = r_jit.clamp(0.0, 0.6);
    let aj = a_jit.clamp(0.0, 0.9);
    let n = k * 2;
    let step = 360.0 / n as f64;

    let ring: Vec<Uvv> = (0..n)
        .map(|i| {
            let base = if i % 2 == 0 { 1.0 } else { inner };
            let rad = base * (1.0 + rj * (BURST_R_JIT[i % 24] - 1.0) / BURST_R_SPAN);
            let ang =
                (BURST_PHASE_DEG + step * i as f64 + aj * BURST_A_JIT[i % 24] * step).to_radians();
            let (s, c) = ang.sin_cos();
            sharp((0.5 + 0.5 * rad * c, 0.5 + 0.5 * rad * s))
        })
        .collect();

    let mut path = closed(place(&u, 0, &ring));
    fit(&u, &mut path);
    path
}

// ─────────────────────────────────────────────────────────────────────────────
// Chave
// ─────────────────────────────────────────────────────────────────────────────

/// **Chave** `{` — a anotação de diagrama. É uma forma ABERTA (uma linha de centro para
/// TRAÇAR), e isso é a semântica que o spec esconde: o `<path>` preenchível do `leftBrace` é
/// um chamariz (fecha pela aresta direita e encerra uma região sem sentido); o glifo de
/// verdade é o segundo path, `fill="none"`.
///
/// Quatro quartos de elipse + dois talos retos:
///
/// ```text
/// M (1,1) → arco → (stem, 1−arm) → talo → (stem, pinch+arm) → arco → (0, pinch)  ← o BICO
///                 (0, pinch) → arco → (stem, pinch−arm) → talo → (stem, arm) → arco → (1,0)
/// ```
///
/// O bico em `(0, pinch)` é uma **cúspide de 180° verdadeira**: os dois arcos são tangentes
/// à horizontal ali e o sentido de percurso inverte. É isso que faz um `{` em vez de um `S`.
///
/// Clamp (do próprio `maxAdj1` do spec): os dois pares de arcos não podem se sobrepor, logo
/// `arm ≤ ½·min(pinch, 1−pinch)` — e daí a contenção é automática.
#[must_use]
pub fn brace(a: [f64; 2], b: [f64; 2], arm: f64, pinch: f64, stem: f64) -> VecPath {
    let u = Unit::of(a, b);
    let sx = stem.clamp(0.15, 0.85);
    let py = pinch.clamp(0.1, 0.9);
    let am = arm.max(1e-3).min(0.5 * py.min(1.0 - py));

    let mut ring = uv_arc((1.0, 1.0 - am), 1.0 - sx, am, 90.0, 90.0); // ponta de baixo
    ring.push(sharp((sx, py + am))); // talo de baixo
    uv_straighten(&mut ring, 1);
    stitch(&mut ring, &uv_arc((0.0, py + am), sx, am, 0.0, -90.0), true);
    stitch(
        &mut ring,
        &uv_arc((0.0, py - am), sx, am, 90.0, -90.0),
        true,
    ); // o BICO
    let i_stem = ring.len() - 1;
    ring.push(sharp((sx, am))); // talo de cima
    uv_straighten(&mut ring, i_stem);
    stitch(
        &mut ring,
        &uv_arc((1.0, am), 1.0 - sx, am, 180.0, 90.0),
        true,
    );

    let mut path = open(place(&u, 0, &ring));
    fit(&u, &mut path);
    path
}

#[cfg(test)]
#[path = "bubbles_tests.rs"]
mod tests;
