//! **A SONDA QUE DECIDE A LEI DO MOTOR NOVO** (doc 12 §3.1, passo 1 da wave).
//!
//! O motor proposto (C4) troca `perfil(min distância)` pela **integral de arco**:
//!
//! ```text
//! τ(p) = (1/pitch) · ∫_caminho f(dn(s,p)) ds        f(d) = −ln(1 − dab(d))
//! α(p) = 1 − exp(−τ(p))
//! ```
//!
//! Isso é o **limite contínuo** da fileira de dabs do Painter: se os dabs são compostos por
//! `over` (`c ← 1−(1−c)(1−w)`), então `1−α = Π(1−w_k)`, e `ln` transforma o produto na soma
//! que a integral aproxima. **É por isso que a soma é ADITIVA e o blending pode ser `Add`.**
//!
//! ⚠️ **O RISCO QUE ESTA SONDA MEDE** (doc 12 §3.1, risco 2): o oráculo `painter_deposit_sized`
//! é uma soma **FINITA** (dabs a `0,1×diâmetro`); a integral é o **LIMITE** dela. Os dois não
//! são a mesma coisa, e a diferença tem de ser um número conhecido ANTES de virar critério de
//! aceitação — senão o gate mede o limite e o produto mede a soma.
//!
//! Roda na CPU, sem adapter. `cargo test -p ph2d-flip-render --release --test integral_law -- --ignored --nocapture`

const PAINTER_SPACING: f32 = 0.10;
const W: u32 = 96;
const H: u32 = 96;

/// O perfil de **UM DAB** do Painter — a função REAL dele, não uma reescrita
/// (o mesmo helper do `painter_look.rs`; a 2ª porta divergiria).
fn painter_weight(dn: f32, hardness: f32) -> f32 {
    let h = hardness.clamp(0.0, 1.0);
    if h >= 1.0 {
        return f32::from(dn < 1.0);
    }
    let remapped = ((dn - h) / (1.0 - h)).clamp(0.0, 1.0);
    ph2d_painter_brush::Falloff::Smooth.weight(remapped)
}

/// **O ORÁCULO** — dabs a `pitch` de arco, compostos por `over`. Cópia verbatim da lei do
/// `painter_look.rs::painter_deposit_sized` (amostra no CENTRO do pixel, decisão medida lá).
fn painter_deposit(pts: &[(f32, f32)], r: f32, hardness: f32) -> Vec<f32> {
    let pitch = (PAINTER_SPACING * 2.0 * r).max(0.25);
    let mut dabs: Vec<(f32, f32)> = vec![pts[0]];
    let mut carry = 0.0_f32;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let seg = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        if seg <= 1e-6 {
            continue;
        }
        let mut t = pitch - carry;
        while t <= seg {
            let f = t / seg;
            dabs.push((a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f));
            t += pitch;
        }
        carry = (carry + seg) % pitch;
    }
    let mut cov = vec![0.0_f32; (W * H) as usize];
    for &(dx, dy) in &dabs {
        let x0 = ((dx - r).floor().max(0.0)) as u32;
        let x1 = ((dx + r).ceil().min(W as f32 - 1.0)) as u32;
        let y0 = ((dy - r).floor().max(0.0)) as u32;
        let y1 = ((dy + r).ceil().min(H as f32 - 1.0)) as u32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let dn = ((px - dx).powi(2) + (py - dy).powi(2)).sqrt() / r;
                if dn >= 1.0 {
                    continue;
                }
                let w = painter_weight(dn, hardness);
                let c = &mut cov[(y * W + x) as usize];
                *c = 1.0 - (1.0 - *c) * (1.0 - w);
            }
        }
    }
    cov
}

/// O teto de `f`. `dab → 1` faz `f = −ln(0) → ∞`; o núcleo duro é opaco e `α` satura, então
/// qualquer teto acima de ~16 é indistinguível em u8 (`1 − e^{−16} = 1 − 1,1e−7`).
/// ⚠️ Este número é do MOTOR (a precisão do alvo `τ`), e está aqui para ser medido, não
/// escolhido: a §"o teto de τ" abaixo varre-o.
const F_MAX: f32 = 16.0;

fn f_of(dn: f32, hardness: f32) -> f32 {
    let w = painter_weight(dn, hardness);
    if w >= 1.0 {
        return F_MAX;
    }
    (-(1.0 - w).ln()).min(F_MAX)
}

/// **A LEI PROPOSTA** — `α = 1 − exp(−τ)`, `τ` integrado por arco com `sub` amostras por pitch.
/// Quadratura do ponto médio (a que um kernel de GPU faria por segmento).
///
/// `caps`: liga o **TERMO DE FRONTEIRA**, e ⚠️ **a medição REPROVOU a hipótese que ele encarna** —
/// esta doc registra o resultado, não a expectativa.
///
/// A hipótese era Euler–Maclaurin: `Σ_{k=0}^{N} g(k·h) = (1/h)·∫g ds + ½·(g(0)+g(L)) + O(h)`, ou
/// seja **meio dab em cada extremo**, já que uma ponta só tem arco de UM lado.
///
/// **Medido (`measure_whether_one_boundary_coefficient_fits_every_hardness`): não fecha.** Com
/// `k = 0,5` a tampa faz OVERSHOOT (−101 → +87). Varrendo `k` e medindo **na região da tampa**, o
/// erro salta de −54 para +40 **sem passar por zero** — o pior pixel TROCA DE LUGAR em vez de
/// encolher. Logo não é um coeficiente errado: a fileira FINITA põe um **disco** na ponta e a
/// integral contínua tem ponta **macia**; são formas diferentes, não amplitudes diferentes.
///
/// ⇒ O cap é **primitivo geométrico**, e vai para a wave de caps/joins (`03 §8`). Este parâmetro
/// fica só como a sonda que provou isso.
fn integral_law_caps(pts: &[(f32, f32)], r: f32, hardness: f32, sub: u32, caps: bool) -> Vec<f32> {
    integral_law_k(pts, r, hardness, sub, if caps { 0.5 } else { 0.0 })
}

/// A lei com o coeficiente de fronteira `k` explícito (0 = sem termo, 0,5 = Euler–Maclaurin).
fn integral_law_k(pts: &[(f32, f32)], r: f32, hardness: f32, sub: u32, k: f32) -> Vec<f32> {
    let pitch = (PAINTER_SPACING * 2.0 * r).max(0.25);
    let ds = pitch / sub as f32;
    let mut tau = vec![0.0_f32; (W * H) as usize];
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let seg = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        if seg <= 1e-6 {
            continue;
        }
        let n = (seg / ds).ceil().max(1.0) as u32;
        let step = seg / n as f32;
        for i in 0..n {
            // ponto médio da i-ésima sub-amostra (⚠️ `k` é o coeficiente de fronteira — não sombrear)
            let t = (i as f32 + 0.5) * step / seg;
            let (sx, sy) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            let x0 = ((sx - r).floor().max(0.0)) as u32;
            let x1 = ((sx + r).ceil().min(W as f32 - 1.0)) as u32;
            let y0 = ((sy - r).floor().max(0.0)) as u32;
            let y1 = ((sy + r).ceil().min(H as f32 - 1.0)) as u32;
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let px = x as f32 + 0.5;
                    let py = y as f32 + 0.5;
                    let dn = ((px - sx).powi(2) + (py - sy).powi(2)).sqrt() / r;
                    if dn >= 1.0 {
                        continue;
                    }
                    tau[(y * W + x) as usize] += f_of(dn, hardness) * (step / pitch);
                }
            }
        }
    }
    if k > 0.0 {
        // O termo de fronteira: `k` dab em cada EXTREMO do traço.
        for &(ex, ey) in &[pts[0], pts[pts.len() - 1]] {
            let x0 = ((ex - r).floor().max(0.0)) as u32;
            let x1 = ((ex + r).ceil().min(W as f32 - 1.0)) as u32;
            let y0 = ((ey - r).floor().max(0.0)) as u32;
            let y1 = ((ey + r).ceil().min(H as f32 - 1.0)) as u32;
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let px = x as f32 + 0.5;
                    let py = y as f32 + 0.5;
                    let dn = ((px - ex).powi(2) + (py - ey).powi(2)).sqrt() / r;
                    if dn >= 1.0 {
                        continue;
                    }
                    tau[(y * W + x) as usize] += k * f_of(dn, hardness);
                }
            }
        }
    }
    tau.iter().map(|&t| 1.0 - (-t).exp()).collect()
}

/// A lei SEM o termo de fronteira (o que a sonda mediu primeiro).
fn integral_law(pts: &[(f32, f32)], r: f32, hardness: f32, sub: u32) -> Vec<f32> {
    integral_law_caps(pts, r, hardness, sub, false)
}

fn q8(v: f32) -> i32 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as i32
}

/// Pior desvio em níveis de 255, e onde.
fn worst(a: &[f32], b: &[f32]) -> (i32, (u32, u32), u32) {
    let (mut d, mut at, mut n) = (0i32, (0u32, 0u32), 0u32);
    for y in 0..H {
        for x in 0..W {
            let i = (y * W + x) as usize;
            let e = q8(a[i]) - q8(b[i]);
            if e.abs() > 8 {
                n += 1;
            }
            if e.abs() > d.abs() {
                d = e;
                at = (x, y);
            }
        }
    }
    (d, at, n)
}

// ————————————————————————————————— as figuras —————————————————————————————————

fn straight(r: f32) -> Vec<(f32, f32)> {
    let _ = r;
    vec![(12.0, 48.0), (84.0, 48.0)]
}

/// A quina afiada — onde o modelo de distância precisava de remendo.
fn corner() -> Vec<(f32, f32)> {
    vec![(16.0, 20.0), (60.0, 48.0), (16.0, 76.0)]
}

/// O auto-cruzamento: o X que o Enio desenha no smoke.
fn crossing() -> Vec<(f32, f32)> {
    vec![(20.0, 20.0), (76.0, 76.0), (76.0, 20.0), (20.0, 76.0)]
}

/// A estrela de UM traço — a figura do oráculo do Enio.
fn star() -> Vec<(f32, f32)> {
    let (cx, cy, outer) = (48.0_f32, 48.0, 34.0);
    let mut c: Vec<(f32, f32)> = (0..5)
        .map(|k| {
            let a = -std::f32::consts::FRAC_PI_2 + (k as f32) * 4.0 * std::f32::consts::PI / 5.0;
            (cx + outer * a.cos(), cy + outer * a.sin())
        })
        .collect();
    c.push(c[0]);
    c
}

/// Reamostra um caminho num passo dado — a MÃO LENTA (a densidade que matou o motor atual).
fn densify(pts: &[(f32, f32)], step: f32) -> Vec<(f32, f32)> {
    let mut out = vec![pts[0]];
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let seg = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        let n = (seg / step).ceil().max(1.0) as usize;
        for k in 1..=n {
            let t = k as f32 / n as f32;
            out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
        }
    }
    out
}

// ————————————————————————————————— as medições —————————————————————————————————

/// **A PERGUNTA 1: a integral reproduz o depósito finito do Painter?**
/// Se não reproduzir, o C4 morre aqui — antes de uma linha de GPU.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_the_integral_against_the_painters_finite_deposit() {
    println!("\n=== A INTEGRAL vs O DEPOSITO FINITO DO PAINTER (o oraculo) ===");
    println!("    quadratura: 32 sub-amostras por pitch\n");
    println!("  figura       r     h     pior   onde         px>8");
    for (name, pts) in [
        ("reta", straight(7.0)),
        ("quina", corner()),
        ("cruz", crossing()),
        ("estrela", star()),
    ] {
        for r in [7.0_f32, 14.0] {
            for h in [0.0_f32, 0.2, 0.4, 0.7, 0.9] {
                let want = painter_deposit(&pts, r, h);
                let got = integral_law(&pts, r, h, 32);
                let (d, at, n) = worst(&got, &want);
                println!("  {name:<10} {r:>4.0}  {h:>4.1}  {d:>+5}   {at:?}   {n:>4}");
            }
        }
        println!();
    }
}

/// **A PERGUNTA 2: a lei é função do CAMINHO, não da densidade da polilinha?**
/// É a lei que o `sampling_invariance.rs` pina no motor atual, e a que o penhasco quebrou.
/// A integral é por ARCO, então a resposta tem de ser um número CONSTANTE.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_the_integral_is_a_fact_of_the_path_not_of_the_polyline_density() {
    println!("\n=== A LEI: a MESMA figura, de 0,80·r a 0,04·r de passo ===");
    let r = 7.0_f32;
    for h in [0.4_f32, 0.7] {
        let base = star();
        let want = painter_deposit(&densify(&base, 0.8 * r), r, h);
        for k in [0.80_f32, 0.40, 0.20, 0.10, 0.05, 0.04] {
            let pts = densify(&base, k * r);
            let got = integral_law(&pts, r, h, 32);
            let (d, _, n) = worst(&got, &want);
            println!(
                "  h={h:.1} passo={k:.2}xr ({:>4} segmentos)  desvio {d:+4}  ({n} px > 8)",
                pts.len() - 1
            );
        }
        println!();
    }
}

/// **A PERGUNTA 3: quantas sub-amostras por pitch o kernel precisa?**
/// É o custo do motor: cada sub-amostra é trabalho no fragment. O número que
/// SATURA aqui é o que o kernel vai usar — medido, não escolhido.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_how_much_quadrature_the_kernel_needs() {
    println!("\n=== QUADRATURA: sub-amostras por pitch vs fidelidade ===");
    let r = 7.0_f32;
    for (name, pts) in [("quina", corner()), ("estrela", star())] {
        for h in [0.4_f32, 0.7] {
            let want = painter_deposit(&pts, r, h);
            print!("  {name:<8} h={h:.1}: ");
            for sub in [1_u32, 2, 4, 8, 16, 32, 64] {
                let got = integral_law(&pts, r, h, sub);
                let (d, _, _) = worst(&got, &want);
                print!("{sub}→{d:+4}  ");
            }
            println!();
        }
    }
    println!("\n  (sub=1 é UMA amostra por dab do Painter — o piso teórico da soma finita)");
}

/// **A PERGUNTA 3b: ONDE mora o desvio — tampa, junta, ou CORPO?**
///
/// A pergunta que decide se o C4 vive: um desvio no **corpo** condena a lei; um desvio só nas
/// **tampas** é a wave de caps/joins que o `03 §8` já tem na fila (e que a baseline de hoje
/// também acusa no motor ATUAL — doc 12 §1). Sem esta separação o número agregado mente.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_where_the_deviation_lives_cap_join_or_body() {
    println!("\n=== ONDE MORA O DESVIO: tampa · junta · CORPO ===");
    println!(
        "    (tampa = a menos de r de um EXTREMO · junta = a menos de r de um vértice interno)\n"
    );
    println!("  figura     h      TAMPA        JUNTA        CORPO");
    for (name, pts) in [
        ("reta", straight(7.0)),
        ("quina", corner()),
        ("cruz", crossing()),
        ("estrela", star()),
    ] {
        for h in [0.0_f32, 0.4, 0.7] {
            let r = 7.0_f32;
            let want = painter_deposit(&pts, r, h);
            let got = integral_law(&pts, r, h, 32);
            let ends = [pts[0], pts[pts.len() - 1]];
            let inner = &pts[1..pts.len() - 1];
            let near = |p: (f32, f32), q: (f32, f32)| {
                (p.0 - q.0).powi(2) + (p.1 - q.1).powi(2) <= (r * 1.05).powi(2)
            };
            let (mut cap, mut join, mut body) = (0i32, 0i32, 0i32);
            for y in 0..H {
                for x in 0..W {
                    let i = (y * W + x) as usize;
                    let e = q8(got[i]) - q8(want[i]);
                    let p = (x as f32 + 0.5, y as f32 + 0.5);
                    let slot = if ends.iter().any(|&q| near(p, q)) {
                        &mut cap
                    } else if inner.iter().any(|&q| near(p, q)) {
                        &mut join
                    } else {
                        &mut body
                    };
                    if e.abs() > slot.abs() {
                        *slot = e;
                    }
                }
            }
            println!("  {name:<8} {h:>4.1}  {cap:>+6}      {join:>+6}      {body:>+6}");
        }
        println!();
    }
    println!("  ⚠️ a estrela é FECHADA: os dois 'extremos' são o mesmo ponto (uma tampa só).");
}

/// **A PERGUNTA 3c: o TERMO DE FRONTEIRA fecha a tampa?**
/// Euler–Maclaurin diz que falta `½·(f(início) + f(fim))`. Ou fecha, ou a lei tem outro buraco.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_whether_the_boundary_term_closes_the_cap() {
    println!("\n=== O TERMO DE FRONTEIRA (meio dab por extremo) ===\n");
    println!("  figura     h     SEM caps   COM caps    corpo(COM)");
    for (name, pts) in [
        ("reta", straight(7.0)),
        ("quina", corner()),
        ("cruz", crossing()),
        ("estrela", star()),
    ] {
        for h in [0.0_f32, 0.2, 0.4, 0.7] {
            let r = 7.0_f32;
            let want = painter_deposit(&pts, r, h);
            let sem = integral_law_caps(&pts, r, h, 32, false);
            let com = integral_law_caps(&pts, r, h, 32, true);
            let (d_sem, _, _) = worst(&sem, &want);
            let (d_com, _, _) = worst(&com, &want);
            // o corpo, com o termo ligado
            let ends = [pts[0], pts[pts.len() - 1]];
            let inner = &pts[1..pts.len() - 1];
            let near = |p: (f32, f32), q: (f32, f32)| {
                (p.0 - q.0).powi(2) + (p.1 - q.1).powi(2) <= (r * 1.05).powi(2)
            };
            let mut body = 0i32;
            for y in 0..H {
                for x in 0..W {
                    let i = (y * W + x) as usize;
                    let p = (x as f32 + 0.5, y as f32 + 0.5);
                    if ends.iter().any(|&q| near(p, q)) || inner.iter().any(|&q| near(p, q)) {
                        continue;
                    }
                    let e = q8(com[i]) - q8(want[i]);
                    if e.abs() > body.abs() {
                        body = e;
                    }
                }
            }
            println!("  {name:<8} {h:>4.1}  {d_sem:>+7}   {d_com:>+7}    {body:>+7}");
        }
        println!();
    }
}

/// **A PERGUNTA 3d: EXISTE um coeficiente de fronteira que sirva?**
///
/// Se um único `k` fecha a tampa em toda dureza, o cap é um **TERMO** (barato, uma linha).
/// Se não existe, a tampa é **GEOMETRIA** — a fileira finita do Painter põe um DISCO no
/// primeiro dab e a integral contínua tem uma ponta MACIA; nesse caso o cap vira primitivo
/// explícito, que é a wave de caps/joins que o `03 §8` já tem na fila.
/// ⚠️ Mede a REGIÃO DA TAMPA, não o pior global (que muda de lugar quando o termo entra).
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_whether_one_boundary_coefficient_fits_every_hardness() {
    println!("\n=== VARREDURA DO COEFICIENTE DE FRONTEIRA (pior desvio NA TAMPA) ===\n");
    let r = 7.0_f32;
    for (name, pts) in [("reta", straight(7.0)), ("quina", corner())] {
        println!("  {name}:");
        print!("     h \\ k  ");
        for k in [0.0_f32, 0.15, 0.25, 0.35, 0.5] {
            print!("{k:>8.2}");
        }
        println!();
        for h in [0.0_f32, 0.2, 0.4, 0.7, 0.9] {
            let want = painter_deposit(&pts, r, h);
            print!("     {h:>4.1}   ");
            for k in [0.0_f32, 0.15, 0.25, 0.35, 0.5] {
                let got = integral_law_k(&pts, r, h, 32, k);
                let ends = [pts[0], pts[pts.len() - 1]];
                let mut cap = 0i32;
                for y in 0..H {
                    for x in 0..W {
                        let p = (x as f32 + 0.5, y as f32 + 0.5);
                        if !ends.iter().any(|&q| {
                            (p.0 - q.0).powi(2) + (p.1 - q.1).powi(2) <= (r * 1.05).powi(2)
                        }) {
                            continue;
                        }
                        let i = (y * W + x) as usize;
                        let e = q8(got[i]) - q8(want[i]);
                        if e.abs() > cap.abs() {
                            cap = e;
                        }
                    }
                }
                print!("{cap:>+8}");
            }
            println!();
        }
        println!();
    }
}

/// **A PERGUNTA 4: onde pode ficar o teto de `τ`?**
/// Decide a precisão do alvo (r16float vs r32float) — doc 12 §3.1 risco 1.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_where_the_tau_ceiling_can_sit() {
    println!("\n=== O TETO DE tau: qual F_MAX ainda satura em u8? ===");
    let r = 7.0_f32;
    let pts = star();
    for h in [0.4_f32, 0.7] {
        let want = painter_deposit(&pts, r, h);
        // tau maximo observado com o teto atual
        let pitch = PAINTER_SPACING * 2.0 * r;
        let mut tau_max = 0.0_f32;
        for w in pts.windows(2) {
            let (a, b) = (w[0], w[1]);
            let seg = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            let n = (seg / (pitch / 32.0)).ceil() as u32;
            let step = seg / n as f32;
            for k in 0..n {
                let t = (k as f32 + 0.5) * step / seg;
                let (sx, sy) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                let _ = (sx, sy);
            }
        }
        // o tau no MIOLO do traco (dn = 0): f(0) por dab, integrado sobre o comprimento
        let core = f_of(0.0, h);
        let total: f32 = pts
            .windows(2)
            .map(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt())
            .sum();
        tau_max = tau_max.max(core * (2.0 * r) / pitch);
        let got = integral_law(&pts, r, h, 32);
        let (d, _, _) = worst(&got, &want);
        println!(
            "  h={h:.1}: f(dn=0)={core:.2}  tau no miolo ~{tau_max:.1}  \
             (caminho {total:.0} px)  desvio {d:+4}"
        );
    }
    println!(
        "\n  1 - e^-16 = {:.9}  ⇒ satura em u8",
        1.0 - (-16.0_f32).exp()
    );
    println!("  1 - e^-8  = {:.9}", 1.0 - (-8.0_f32).exp());
    println!("  1 - e^-6  = {:.9}", 1.0 - (-6.0_f32).exp());
}
