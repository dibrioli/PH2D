//! ⏱️ **A MESA DO ORÁCULO** — irmã da [`super::bancada`] pelo tecto de LOC, cortada por
//! RESPONSABILIDADE: *o que a lei nova entrega na CENA do dono* é uma pergunta, *como ela se sai na
//! fixtura onde as SEIS leis foram julgadas em 2026-09-14* é outra.
//!
//! A fixtura e as duas réguas são as de [`docs/Skeleton/oraculo/`](../../../docs/Skeleton/oraculo/README.md):
//! dois ossos de `2` ao longo de `+x`, arte de `4 × 4,8`, malha `48 × 48` — e as colunas são a
//! **dobra** (quanta arte vira do avesso) e o **segue** (quanto da rotação mandada a arte cumpre).
//!
//! ⚠️ **Ela existe porque o §0.0 a exige:** aquela mesa **recusou os pesos harmónicos** (`11,3 %` de
//! dobra contra `3,1 %` do que shipava), e o padrão-ouro é um vizinho deles. *Quem move o número que
//! tornava algo inalcançável tem de reconferir a nota.*

use super::bancada::interpola;
use super::{Handle, Options, bounded_biharmonic};
use ph2d_affine::Xform;
use ph2d_poly2d::Mesh2d;

// ─────────────────────────────────────────────────────────────────────────────────────────────
// A FIXTURA DO ORÁCULO (`docs/Skeleton/oraculo/`) — dois ossos, e as DUAS réguas dele.
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// O arco da fixtura do oráculo: dois ossos de `2` ao longo de `+x`, arte de `4 × 4,8`.
pub(super) const ORA_ARCO: f64 = 4.0;
/// A meia-altura da arte do oráculo.
const ORA_META: f64 = 2.4;

/// Uma lei de pesos qualquer, para a mesa comparar — `ponto ↦ os pesos dele`.
type LeiDePesos<'a> = Box<dyn Fn([f64; 2]) -> Vec<f64> + 'a>;

/// A malha do oráculo: `48 × 48` sobre a caixa `[0, 4] × [−2,4, 2,4]`.
pub(super) fn ora_malha(n: usize) -> Mesh2d {
    let mut m = Mesh2d {
        rest: Vec::new(),
        tris: Vec::new(),
        size: [1, 1],
    };
    for j in 0..=n {
        for i in 0..=n {
            m.rest.push([
                ORA_ARCO * i as f64 / n as f64,
                -ORA_META + 2.0 * ORA_META * j as f64 / n as f64,
            ]);
        }
    }
    let id = |i: usize, j: usize| (j * (n + 1) + i) as u32;
    for j in 0..n {
        for i in 0..n {
            m.tris.push([id(i, j), id(i + 1, j), id(i + 1, j + 1)]);
            m.tris.push([id(i, j), id(i + 1, j + 1), id(i, j + 1)]);
        }
    }
    m
}

/// As poses do oráculo: o 2.º osso roda `total`, o 1.º fica.
pub(super) fn ora_poses(total: f64) -> Vec<Xform> {
    let l = ORA_ARCO / 2.0;
    let (mut ox, mut oy, mut fi) = (0.0, 0.0, 0.0);
    let mut out = Vec::new();
    for i in 0..2 {
        if i > 0 {
            fi += total;
        }
        let (s, c) = fi.sin_cos();
        // `p ↦ R·(p − i·L) + o` — exactamente o `aplica` do `fidelidade.py`.
        out.push(Xform([c, s, -s, c, ox, oy]).compose_after_shift(-(i as f64) * l));
        ox += l * c;
        oy += l * s;
    }
    out
}

/// `R·(p + dx) + o` escrito como um afim — o mesmo que o oráculo aplica.
trait ShiftAfim {
    fn compose_after_shift(self, dx: f64) -> Xform;
}
impl ShiftAfim for Xform {
    fn compose_after_shift(self, dx: f64) -> Xform {
        let Xform([a, b, c, d, e, f]) = self;
        Xform([a, b, c, d, a.mul_add(dx, e), b.mul_add(dx, f)])
    }
}

/// Os pesos do *bump* euclidiano do oráculo (raio `r`), num ponto.
pub(super) fn ora_bump(p: [f64; 2], r: f64) -> Vec<f64> {
    let l = ORA_ARCO / 2.0;
    let eixos = [([0.0, 0.0], [l, 0.0]), ([l, 0.0], [2.0 * l, 0.0])];
    let (mut w, mut soma, mut perto, mut pd) = (vec![0.0; 2], 0.0, 0usize, f64::INFINITY);
    for (i, (a, b)) in eixos.iter().enumerate() {
        let d2 = ph2d_skeleton::dist2_to_segment(p, *a, *b);
        if d2 < pd {
            (perto, pd) = (i, d2);
        }
        let x2 = d2 / (r * r);
        let v = if x2 < 1.0 { (1.0 - x2).powi(2) } else { 0.0 };
        w[i] = v;
        soma += v;
    }
    if soma > 0.0 {
        for v in &mut w {
            *v /= soma;
        }
    } else {
        w[perto] = 1.0;
    }
    w
}

/// As DUAS réguas do oráculo: `(dobra %, a arte segue em graus)`.
fn ora_reguas(m: &Mesh2d, pesos: &dyn Fn([f64; 2]) -> Vec<f64>, total: f64) -> (f64, f64) {
    let poses = ora_poses(total);
    ora_reguas_de(m, &|p| {
        let w = pesos(p);
        let mut o = [0.0, 0.0];
        for (k, &wi) in w.iter().enumerate() {
            if wi == 0.0 {
                continue;
            }
            let q = poses[k].apply(p);
            o[0] += wi * q[0];
            o[1] += wi * q[1];
        }
        o
    })
}

/// ⭐ **As DUAS réguas do oráculo sobre um CAMPO qualquer** — `(dobra %, a arte segue em graus)`.
///
/// ⚠️ Ela existe porque a mesa dos centros de rotação mede outra LEI DE MISTURA sobre os mesmos
/// pesos: sem esta porta, ela teria de copiar as duas réguas, e uma cópia diverge no primeiro
/// ajuste.
pub(super) fn ora_reguas_de(m: &Mesh2d, pos: &dyn Fn([f64; 2]) -> [f64; 2]) -> (f64, f64) {
    let p: Vec<[f64; 2]> = m.rest.iter().map(|&v| pos(v)).collect();
    // DOBRA: a fracção da área com o sinal invertido contra o repouso.
    let (mut inv, mut tot) = (0.0, 0.0);
    for t in &m.tris {
        let s = |q: &[[f64; 2]]| {
            let (a, b, c) = (q[t[0] as usize], q[t[1] as usize], q[t[2] as usize]);
            (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
        };
        let (s0, s1) = (s(&m.rest), s(&p));
        tot += s0.abs();
        if s1 * s0 <= 0.0 {
            inv += s0.abs();
        }
    }
    // SEGUE: quanto a arte sobre a METADE FINAL do 2.º osso rodou, contra o que se mandou.
    let l = ORA_ARCO / 2.0;
    let ponta = pos([2.0 * l, 0.0]);
    let mut angs = Vec::new();
    for (k, v) in m.rest.iter().enumerate() {
        if v[0] < l * 1.5 {
            continue;
        }
        let r0 = [v[0] - 2.0 * l, v[1]];
        let r1 = [p[k][0] - ponta[0], p[k][1] - ponta[1]];
        let (n0, n1) = (r0[0].hypot(r0[1]), r1[0].hypot(r1[1]));
        if n0 < 1e-9 || n1 < 1e-9 {
            continue;
        }
        let cruz = r0[0] * r1[1] - r0[1] * r1[0];
        let dot = r0[0] * r1[0] + r0[1] * r1[1];
        angs.push(cruz.atan2(dot));
    }
    angs.sort_by(f64::total_cmp);
    let mediana = angs[angs.len() / 2];
    (100.0 * inv / tot, mediana.to_degrees())
}

/// ⏱️⭐⭐⭐ **O PADRÃO-OURO NA MESA DO ORÁCULO** (`--ignored`) — a re-medição que o §0.0 exige.
///
/// ⛔⛔ **Uma recusa MEDIDA responde a UMA pergunta, e esta linha mudou a lei por baixo dela.** A
/// tabela das SEIS leis (`docs/Skeleton/oraculo/`, 2026-09-14) recusou os **pesos harmónicos** com
/// `11,3 %` de dobra contra `3,1 %` do que shipava, e o padrão-ouro é um vizinho deles. *Quem move
/// o número que tornava algo inalcançável tem de reconferir a nota.*
#[test]
#[ignore = "bancada: a mesa do oraculo, sem barra"]
fn bancada_na_mesa_do_oraculo() {
    let m = ora_malha(48);
    let l = ORA_ARCO / 2.0;
    let handles = vec![
        Handle {
            a: [0.0, 0.0],
            b: [l, 0.0],
        },
        Handle {
            a: [l, 0.0],
            b: [2.0 * l, 0.0],
        },
    ];
    // ⭐ A VARREDURA da folga da junta: o número tem de sair de um joelho, nunca de uma escolha.
    println!("\nfolga  presos          60°            90°           120°           150°");
    for folga in [0.0_f64, 1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0] {
        let opts = Options {
            folga_da_junta: folga,
            ..Options::default()
        };
        let Some(ww) = bounded_biharmonic(&m, &handles, opts) else {
            println!("{folga:<6} — nao resolve");
            continue;
        };
        let malha = ora_malha(48);
        print!("{folga:<6} {:>6}", ww.report.presos);
        for g in [60.0_f64, 90.0, 120.0, 150.0] {
            let (dobra, segue) = ora_reguas(
                &malha,
                &|p| interpola(&m, &ww.por_vertice, p).unwrap_or_else(|| ora_bump(p, l)),
                g.to_radians(),
            );
            print!("  {dobra:>5.2}% /{segue:>6.1}°");
        }
        println!();
    }

    let w = bounded_biharmonic(&m, &handles, Options::default()).expect("resolve");
    println!(
        "\nBBW (folga de fabrica): {} vertices, {} presos, residuo {:.2e}, convergiu={}",
        w.report.vertices, w.report.presos, w.report.residuo, w.report.convergiu
    );
    // ⭐ O PASSO A PASSO (§3.5 do protocolo): o peso do 1.º osso ao longo da linha do meio — é ele
    // que diz se a lei é uma FUNÇÃO ESCADA na junta, que foi o que reprovou os harmónicos.
    // ⛔⛔ **A 1.ª redacção amostrava `y = 0`, que é a LINHA PRESA** — ela lia a condição de
    // Dirichlet e não a solução, e eu concluí «o padrão-ouro é uma escada». *Uma régua que amostra
    // exactamente onde o problema é fixo mede o que eu escrevi, não o que a energia resolveu.*
    for y in [0.0_f64, 0.3, 0.8, 1.6, 2.3] {
        println!(
            "\nx (em y={y:.1})   bump(raio=osso)   PADRAO-OURO{}",
            if y == 0.0 { "   <- a LINHA PRESA" } else { "" }
        );
        for x in [1.40_f64, 1.70, 1.90, 2.00, 2.10, 2.30, 2.60] {
            let p = [x, y];
            let b = ora_bump(p, l)[0];
            let g = interpola(&m, &w.por_vertice, p).map_or(f64::NAN, |v| v[0]);
            println!("{x:<14.2} {b:>15.4} {g:>13.4}");
        }
    }
    // E o maior GRADIENTE vertical do peso, que é a grandeza que o oráculo citou.
    let grad = |pesos: &dyn Fn([f64; 2]) -> Vec<f64>| -> f64 {
        let mut pior = 0.0_f64;
        let h = 0.05;
        for j in 0..=40 {
            for i in 0..=40 {
                let p = [
                    ORA_ARCO * f64::from(i) / 40.0,
                    -ORA_META + 2.0 * ORA_META * f64::from(j) / 40.0,
                ];
                if p[1] - h < -ORA_META || p[1] + h > ORA_META {
                    continue;
                }
                let a = pesos([p[0], p[1] - h])[0];
                let b = pesos([p[0], p[1] + h])[0];
                pior = pior.max((b - a).abs() / (2.0 * h));
            }
        }
        pior
    };
    let malha = ora_malha(48);
    println!(
        "\nmaior gradiente VERTICAL do peso: bump {:.3} | PADRAO-OURO {:.3}",
        grad(&|p| ora_bump(p, l)),
        grad(&|p| interpola(&m, &w.por_vertice, p).unwrap_or_else(|| ora_bump(p, l)))
    );
    // ⭐⭐⭐ **O CONTROLO QUE TORNA A RÉGUA DA DOBRA HONESTA: quanta arte é RÍGIDA.**
    //
    // ⛔⛔ Uma região presa a UM osso só (peso exactamente `0` ou `1`) **não pode inverter-se** — ela
    // não se deforma. ⇒ uma lei que deixa metade da arte rígida ganha a régua da dobra por NÃO FAZER
    // NADA, que é o mesmo defeito da lei de alcance largo, com outra cara. *A régua da dobra precisa
    // desta coluna ao lado, senão ela premeia a rigidez.*
    let rigida = |pesos: &dyn Fn([f64; 2]) -> Vec<f64>| -> f64 {
        let malha = ora_malha(48);
        let mut duros = 0usize;
        for p in &malha.rest {
            let w = pesos(*p);
            if w.iter().any(|v| *v >= 1.0 - 1e-9) {
                duros += 1;
            }
        }
        100.0 * duros as f64 / malha.rest.len() as f64
    };
    println!(
        "\narte RIGIDA (peso exactamente 0 ou 1): bump {:.1}% | PADRAO-OURO {:.1}%",
        rigida(&|p| ora_bump(p, l)),
        rigida(&|p| interpola(&m, &w.por_vertice, p).unwrap_or_else(|| ora_bump(p, l)))
    );

    println!(
        "\nlei                                 60°            90°           120°           150°"
    );
    let leis: Vec<(&str, LeiDePesos<'_>)> = vec![
        (
            "bump raio = osso (o que shipava)",
            Box::new(|p| ora_bump(p, l)),
        ),
        (
            "bump raio 2,08x a arte",
            Box::new(|p| ora_bump(p, 2.08 * 2.0 * ORA_META)),
        ),
        (
            "PADRAO-OURO (BBW)",
            Box::new(move |p| interpola(&m, &w.por_vertice, p).unwrap_or_else(|| ora_bump(p, l))),
        ),
    ];
    for (nome, pesos) in &leis {
        print!("{nome:<32}");
        for g in [60.0_f64, 90.0, 120.0, 150.0] {
            let (dobra, segue) = ora_reguas(&malha, pesos.as_ref(), g.to_radians());
            print!("  {dobra:>5.2}% /{segue:>6.1}°");
        }
        println!();
    }
}
