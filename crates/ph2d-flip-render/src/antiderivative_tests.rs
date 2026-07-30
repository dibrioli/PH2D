//! 📏 **A ANTIDERIVADA — medir o risco antes de construir** (§21.5 do `12_novo_motor_pesquisa.md`).
//!
//! A substituição `s = r·u` tira o `r` de dentro de `∫ f(dn) ds`, então existe um `H(hardness, y, u)`
//! universal e o laço de quadratura colapsa em duas leituras e uma subtração. Este arquivo mede o que
//! isso CUSTA em exatidão antes de alguém construir a tabela.
//!
//! ⚠️ **Irmão do `tau_tests.rs`, pendurado sob `binning::tests` pelo mesmo motivo:** as fixtures
//! (`screen`, `art`, `push_tapered`, `BLACK`) são as do binning, e duplicá-las daria duas cenas para
//! uma pergunta.

use super::*;
use crate::binning::{BinSeg, ScreenSpace, bin_segments};
use crate::pack::FlipGpuData;

/// `H(y, u) = ∫₀^u f(√(y² + v²)) dv` — a antiderivada universal da §21.5, por quadratura FINA.
///
/// ⚠️ Isto **não é** a LUT: é o ORÁCULO dela. A pergunta desta sonda é a do risco 1 — a substituição
/// `s = r·u` supõe `r` CONSTANTE no segmento, e um traço de pressão viola isso. A resolução da tabela
/// é outra pergunta, e medir as duas juntas não diria qual erra.
///
/// ⚠️ **O ERRO DELE CRESCE COM `u`, e foi isso que fez a 1ª medição parecer não-monótona:** são `n`
/// amostras sobre `[0, u]`, então o passo é `u/n` e o erro `∝ (u/n)²`. Consumido como
/// `H(u1) − H(u0)` — duas grades DIFERENTES para números quase iguais — ele não cancela, e é
/// **por-CHAMADA**: subdividir em `k` pedaços soma `2k` erros desses enquanto o erro do `r`
/// congelado cai com `1/k²`, então a soma tem um MÍNIMO e depois sobe. Medido: por diferença
/// `20,01 → 4,38 → 2,15 → 4,53`; pelo pedaço direto ([`h_piece`]) `20,01 → 4,38 → 0,94 → 0,21`.
/// A premissa da wave é a segunda linha.
fn h_exact(y: f32, u: f32, prof: crate::tau::DabProfile) -> f64 {
    if u == 0.0 {
        return 0.0;
    }
    let n = 4000;
    let (a, b) = (0.0_f64, f64::from(u));
    let h = (b - a) / f64::from(n);
    let mut acc = 0.0_f64;
    for k in 0..n {
        let v = a + (f64::from(k) + 0.5) * h;
        let dn = (f64::from(y) * f64::from(y) + v * v).sqrt();
        acc += f64::from(crate::tau::f_of(dn as f32, prof)) * h;
    }
    acc
}

/// `∫_{u0}^{u1} f(√(y² + v²)) dv` **DIRETO** — o mesmo integrando do [`h_exact`], mas sobre o pedaço
/// em vez de por DIFERENÇA de duas antiderivadas.
///
/// ⚠️ **É o que separa os dois erros que a sonda misturava.** O `h_exact` gasta `n` amostras sobre
/// `[0, u]`, então o erro dele **cresce com `u`** — e a antiderivada o consome como `H(u1) − H(u0)`,
/// dois números quase iguais computados em GRADES DIFERENTES, cujos erros não se cancelam. Subdividir
/// não encolhe esse erro: ele é por-CHAMADA, então `k` pedaços somam `2k` erros independentes
/// enquanto o erro do `r` congelado cai com `1/k²`. Medir o pedaço direto (erro `∝ (Δu/n)²`, que
/// encolhe COM o pedaço) deixa só o `r` congelado — a pergunta que a wave de fato tem.
fn h_piece(y: f32, u0: f32, u1: f32, prof: crate::tau::DabProfile) -> f64 {
    let n = 4000;
    let (a, b) = (f64::from(u0), f64::from(u1));
    let h = (b - a) / f64::from(n);
    let mut acc = 0.0_f64;
    for k in 0..n {
        let v = a + (f64::from(k) + 0.5) * h;
        let dn = (f64::from(y) * f64::from(y) + v * v).sqrt();
        acc += f64::from(crate::tau::f_of(dn as f32, prof)) * h;
    }
    acc
}

/// `τ` de **REFERÊNCIA** — a MESMA lei do [`crate::tau::stroke_tau`], com a quadratura 16× mais fina
/// (`SUB_REF` contra o `SUB = 4` que shipa) e somada em `f64`.
///
/// ⚠️ **Ele existe porque a 1ª versão desta sonda media contra o ORÁCULO ERRADO.** Ela comparava a
/// antiderivada com o que SHIPA, e o que shipa tem **viés próprio** — o doc do `SUB` diz que 4
/// *"satura por cima e morde por baixo"*, com −53/255 medidos na TAMPA. Contra um alvo enviesado o
/// resíduo `|A_k − Q|` **não pode cair abaixo de `|e_Q|`** e pode CRESCER quando o erro da
/// antiderivada atravessa o do alvo — que é exactamente o não-monótono que apareceu (k=8 pior que
/// k=4). Medir as DUAS rotas contra a verdade separa os dois erros, e é a única forma de a pergunta
/// *"a antiderivada é boa o bastante?"* ter resposta.
///
/// ⚠️ **Não inclui o meio dab do `end_dab`** — ele é termo FECHADO (`0.5·f_of`), idêntico em
/// qualquer rota, e a sonda mede fora das tampas de propósito, onde ele vale 0.
fn tau_reference(
    run: &[BinSeg],
    g: &FlipGpuData,
    sc: &ScreenSpace,
    prof: crate::tau::DabProfile,
    p: [f32; 2],
) -> f64 {
    const SUB_REF: f32 = 64.0;
    let mut tau = 0.0_f64;
    for seg in run {
        let (pa, pb) = (g.points[seg.a as usize], g.points[seg.b as usize]);
        let (sa, sb) = (sc.point_px(pa.pos), sc.point_px(pb.pos));
        let (ra, rb) = (sc.radius_px(pa.width), sc.radius_px(pb.width));
        let v = [sb[0] - sa[0], sb[1] - sa[1]];
        let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
        if len <= 1e-6 {
            continue;
        }
        let reach = crate::dabs::dab_reach(crate::tau::TipShape::Continuous, ra.max(rb));
        let Some((t0, t1)) = crate::dabs::seg_window(p, sa, sb, reach) else {
            continue;
        };
        let pitch_min = (crate::tau::PAINTER_SPACING * 2.0 * ra.min(rb)).max(0.25);
        let ds = pitch_min / SUB_REF;
        let n = (len / ds).ceil().max(1.0);
        let step = len / n;
        let i0 = (t0 * len / step - 0.5).floor().max(0.0) as u32;
        let i1 = (t1 * len / step - 0.5).ceil().clamp(0.0, n - 1.0) as u32;
        for i in i0..=i1 {
            let t = ((i as f32 + 0.5) * step / len).clamp(0.0, 1.0);
            let s = [sa[0] + v[0] * t, sa[1] + v[1] * t];
            let r = (ra * (1.0 - t) + rb * t).max(1e-4);
            let dn = ((p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2)).sqrt() / r;
            let pitch = (crate::tau::PAINTER_SPACING * 2.0 * r).max(0.25);
            tau += f64::from(crate::tau::d_tau_of(dn, prof, step, r, pitch));
        }
    }
    tau
}

/// `τ` de um traço via a ANTIDERIVADA, com `r` congelado no MEIO de cada segmento.
fn tau_via_antiderivative(
    run: &[BinSeg],
    g: &FlipGpuData,
    sc: &ScreenSpace,
    prof: crate::tau::DabProfile,
    p: [f32; 2],
    k: u32,
    direto: bool,
) -> f64 {
    let mut tau = 0.0_f64;
    for seg in run {
        let (pa, pb) = (g.points[seg.a as usize], g.points[seg.b as usize]);
        let (sa, sb) = (sc.point_px(pa.pos), sc.point_px(pb.pos));
        let v = [sb[0] - sa[0], sb[1] - sa[1]];
        let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
        if len <= 1e-6 {
            continue;
        }
        let dir = [v[0] / len, v[1] / len];
        let w = [p[0] - sa[0], p[1] - sa[1]];
        let t_foot = w[0] * dir[0] + w[1] * dir[1];
        let perp = (w[0] * (-dir[1]) + w[1] * dir[0]).abs();
        let (ra, rb) = (sc.radius_px(pa.width), sc.radius_px(pb.width));
        // ⚠️ **A antiderivada é EXATA para `r` constante**, então a cura do risco 1 é SUBDIVIDIR: `k`
        // pedaços com `r` congelado no meio de cada um. Duas leituras por pedaço, contra ~40 amostras
        // da quadratura — o `k` que fecha o erro é o que decide se a wave vale.
        for j in 0..k {
            let (fa, fb) = (j as f32 / k as f32, (j + 1) as f32 / k as f32);
            let (sa_j, sb_j) = (len * fa, len * fb);
            let r = ra + (rb - ra) * (fa + fb) * 0.5;
            let pitch = (crate::tau::PAINTER_SPACING * 2.0 * r).max(0.25);
            let y = perp / r;
            let (u0, u1) = ((sa_j - t_foot) / r, (sb_j - t_foot) / r);
            let dh = if direto {
                h_piece(y, u0, u1, prof)
            } else {
                h_exact(y, u1, prof) - h_exact(y, u0, prof)
            };
            tau += f64::from(r / pitch) * dh;
        }
    }
    tau
}

/// 📏 **SONDA — o RISCO 1 da §21.5: a antiderivada supõe `r` constante no segmento.**
///
/// Três rotas contra a MESMA referência fina ([`tau_reference`]), em três regimes de largura:
///
/// 1. **a quadratura que SHIPA** — o controle. Ela sai em `0,00–0,01/255` no corpo do traço, e é
///    isso que torna a referência confiável (o viés de `−53` do `SUB = 4` é da TAMPA, que esta
///    sonda exclui de propósito).
/// 2. **`H` por DIFERENÇA** — carrega o erro do oráculo, que **cresce com `u`** (ver [`h_exact`]).
/// 3. **pedaço DIRETO** ([`h_piece`]) — só o `r` congelado, que é a premissa da wave.
///
/// ⚠️ **A 1ª versão desta sonda tinha só a rota 2 e comparava contra o que SHIPA**, então media as
/// duas coisas somadas contra um alvo que eu supunha enviesado — e a conclusão saiu *"2,15 em k=4, e
/// k=8 é PIOR"*, que é uma afirmação sobre o meu oráculo vestida de afirmação sobre o desenho. O que
/// a wave precisa saber é a rota 3: **`0,94/255` em k=4 · `0,21` em k=8**, caindo `O(1/k²)`.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_whether_the_antiderivative_survives_a_varying_radius() {
    let (w, h) = (96.0, 64.0);
    let sc = screen(w, h);
    let prof = crate::tau::DabProfile {
        hardness: 0.5,
        airbrush: false,
    };
    let style = crate::tau::StrokeStyle {
        profile: prof,
        tip: crate::tau::TipShape::Continuous,
    };
    // ⚠️ **A fixture afilada é REAMOSTRADA, e a 1ª versão desta sonda não era:** com 2 pontos o `r`
    // varia 8× dentro de UM segmento, o que não é regime nenhum do produto — o `resample_smooth`
    // densifica a `0,4 × largura`, ou seja segmentos de ~`0,8r`. Medir no traço de 2 pontos respondia
    // sobre um desenho que o motor nunca vê (o erro saía 74,6 em τ).
    // ⚠️ **O passo é `0,4 × largura` LOCAL — e a 2ª versão desta sonda usava passo UNIFORME**, o que
    // no fim fino dá segmentos de `3,2r` em vez de `0,8r`: ela media uma reamostragem que o produto
    // não produz. É a mesma convenção que o `measure_ribbon_budget` do `neighbors_tests` já usa
    // (`step = 0,8·R` com `R` local), porque é o que o `resample_smooth` faz.
    let densificado = |xs: (f32, f32), ws: (f32, f32)| -> FlipGpuData {
        let (mut pts, mut wds) = (Vec::new(), Vec::new());
        let mut x = xs.0;
        loop {
            let t = (x - xs.0) / (xs.1 - xs.0);
            let wl = ws.0 + (ws.1 - ws.0) * t;
            pts.push([x, 32.0]);
            wds.push(wl);
            if x >= xs.1 - 1e-4 {
                break;
            }
            x = (x + 0.4 * wl).min(xs.1);
        }
        let mut g = FlipGpuData::default();
        push_tapered(&mut g, &pts, &wds);
        g
    };
    println!(
        "  cena                           o pior |Δα| (de 255) contra a REFERÊNCIA fina (SUB 64)"
    );
    for (nome, g) in [
        (
            "reto, largura CONSTANTE",
            art(&[(&[[12.0, 32.0], [84.0, 32.0]], 12.0, false, BLACK)]),
        ),
        ("afilado 24->3, 2 PONTOS (irreal)", {
            let mut g = FlipGpuData::default();
            push_tapered(&mut g, &[[12.0, 32.0], [84.0, 32.0]], &[24.0, 3.0]);
            g
        }),
        (
            "afilado 24->3, REAMOSTRADO",
            densificado((12.0, 84.0), (24.0, 3.0)),
        ),
    ] {
        let bins = bin_segments(&g, &sc, 16);
        let (mut pior_q, mut pior_a, mut pior_d, mut n) =
            (0.0_f64, [0.0_f64; 4], [0.0_f64; 4], 0u32);
        for y in 0..h as u32 {
            for x in 0..w as u32 {
                let p = [x as f32 + 0.5, y as f32 + 0.5];
                let Some(ti) = bins.tile_of_pixel(p[0], p[1]) else {
                    continue;
                };
                let run = bins.segs_of(ti);
                if run.is_empty() {
                    continue;
                }
                // ⚠️ **Fora das TAMPAS.** A quadratura que shipa soma o meio dab do `end_dab` (§13) e
                // a antiderivada não o tem; incluir a ponta mediria a AUSÊNCIA desse termo (≈ F_MAX/2
                // = 8 em τ, o que a 1ª versão desta sonda reportou como 9,0) em vez da substituição.
                let st = g.strokes[0];
                let p0 = sc.point_px(g.points[st.first_point as usize].pos);
                let pn = sc.point_px(g.points[(st.first_point + st.point_count - 1) as usize].pos);
                let r0 = sc.radius_px(g.points[st.first_point as usize].width) + 2.0;
                let rn = sc
                    .radius_px(g.points[(st.first_point + st.point_count - 1) as usize].width)
                    + 2.0;
                let perto = |q: [f32; 2], raio: f32| {
                    (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) <= raio * raio
                };
                if perto(p0, r0) || perto(pn, rn) {
                    continue;
                }
                let Some(ink) = crate::tau::stroke_tau(run, &g, &sc, style, p) else {
                    continue;
                };
                // A VERDADE deste pixel, e é contra ela que as duas rotas respondem.
                let a_ref = 1.0 - (-tau_reference(run, &g, &sc, prof, p).max(0.0)).exp();
                let a_q = 1.0 - (-f64::from(ink.tau)).exp();
                pior_q = pior_q.max((a_q - a_ref).abs() * 255.0);
                for (ki, kk) in [1_u32, 2, 4, 8].into_iter().enumerate() {
                    for (dst, direto) in [(&mut pior_a, false), (&mut pior_d, true)] {
                        let alvo = tau_via_antiderivative(run, &g, &sc, prof, p, kk, direto);
                        let a_h = 1.0 - (-alvo.max(0.0)).exp();
                        dst[ki] = dst[ki].max((a_h - a_ref).abs() * 255.0);
                    }
                }
                n += 1;
            }
        }
        println!("  {nome}  ({n} px)   a QUADRATURA que shipa: {pior_q:.2}");
        println!(
            "      H por DIFERENCA (erro do ORACULO)     k=1 {:6.2}  k=2 {:6.2}  k=4 {:6.2}  k=8 \
             {:6.2}",
            pior_a[0], pior_a[1], pior_a[2], pior_a[3]
        );
        println!(
            "      pedaco DIRETO (so o `r` congelado)    k=1 {:6.2}  k=2 {:6.2}  k=4 {:6.2}  k=8 \
             {:6.2}",
            pior_d[0], pior_d[1], pior_d[2], pior_d[3]
        );
    }
}
