use super::{ANGULOS, BORDO, Perfil, RUGOSIDADES, alfa_da_calote, pela_calote, pelo_lobo};
use crate::{Ceu, Embarcado, Panorama};

fn luma(c: [f32; 3]) -> f64 {
    0.2126 * f64::from(c[0]) + 0.7152 * f64::from(c[1]) + 0.0722 * f64::from(c[2])
}

fn unit(d: [f64; 3]) -> [f64; 3] {
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    d.map(|v| v / l)
}

fn dir_de(elev_graus: f64, azim_graus: f64) -> [f64; 3] {
    let (e, a) = (elev_graus.to_radians(), azim_graus.to_radians());
    [e.cos() * a.cos(), e.sin(), e.cos() * a.sin()]
}

/// Um céu chapado `fundo` a `1024 × 512` (o tamanho dos embarcados) com DISCOS uniformes
/// `(direcção, raio em graus, radiância)` pintados pelo centro de cada texel.
fn com_discos(fundo: [f32; 3], discos: &[([f64; 3], f64, [f32; 3])]) -> Panorama {
    let mut p = Panorama {
        largura: 1024,
        altura: 512,
        rgb: vec![fundo; 1024 * 512],
    };
    for y in 0..p.altura {
        for x in 0..p.largura {
            let d = p.direcao(x, y).map(f64::from);
            for (c, r, l) in discos {
                if angulo(d, *c) <= r.to_radians() {
                    p.rgb[(y * p.largura + x) as usize] = *l;
                }
            }
        }
    }
    p
}

/// ⭐ **O forno do sol:** um céu chapado não tem sol, e o céu sem sol é ELE, ao bit.
#[test]
fn um_ceu_chapado_nao_tem_sol() {
    let p = Panorama::chapado([0.3, 0.7, 1.9]);
    let (sem, sol) = p.separa_sol();
    assert!(sol.is_none());
    assert_eq!(sem, p);
}

/// ⭐⭐ **O sol é ACHADO onde está, com a energia que tem, e só ele:** dois discos de direcção, raio e
/// cor conhecidos — o forte vira o sol (direcção ao centésimo de grau, raio ao texel, a energia do
/// excesso), o fraco fica no céu INTACTO, e `céu sem sol + excesso = o panorama` texel a texel.
#[test]
fn o_sol_e_achado_e_so_ele() {
    let fundo = [0.5, 0.6, 0.7];
    let (forte, fraco) = (dir_de(40.0, 30.0), dir_de(15.0, -120.0));
    let (lf, lw) = ([5000.0, 4500.0, 4000.0], [300.0, 300.0, 300.0]);
    let p = com_discos(fundo, &[(forte, 1.0, lf), (fraco, 2.0, lw)]);
    let (sem, sol) = p.separa_sol();
    let sol = sol.expect("há um sol");
    let erro_dir = angulo(sol.dir.map(f64::from), forte).to_degrees();
    assert!(erro_dir < 0.05, "direcção errada por {erro_dir:.3}°");
    // O 2.º momento de um disco uniforme de raio θ é θ²/2: o raio volta (a grelha erra meio texel).
    let raio = f64::from(sol.raio).to_degrees();
    assert!((raio - 1.0).abs() < 0.1, "raio {raio:.3}° (pintado 1°)");
    // Texel a texel: o fraco intacto; o forte preso ao limiar com a cor dele; a soma é o panorama.
    let limiar = super::LIMIAR * luma(p.media());
    let mut energia = [0.0f64; 3];
    for y in 0..p.altura {
        let om = p.angulo_solido(y);
        for x in 0..p.largura {
            let i = (y * p.largura + x) as usize;
            let d = p.direcao(x, y).map(f64::from);
            let (a, b) = (p.rgb[i], sem.rgb[i]);
            if angulo(d, forte) <= 1.0f64.to_radians() {
                assert!((luma(b) - limiar).abs() < 1.0e-3 * limiar, "não preso ao limiar");
                let r = |c: [f32; 3]| c[0] / c[2];
                assert!((r(a) - r(b)).abs() < 1.0e-5, "a cor mudou");
            } else {
                assert_eq!(a, b, "um texel fora do sol mudou ({x}, {y})");
            }
            for k in 0..3 {
                energia[k] += f64::from(a[k] - b[k]) * om;
            }
        }
    }
    for k in 0..3 {
        let rel = (f64::from(sol.energia[k]) - energia[k]).abs() / energia[k];
        assert!(rel < 1.0e-4, "energia do canal {k}: {} contra {}", sol.energia[k], energia[k]);
    }
}

/// ⭐⭐ **A tabela conserva a energia em TODA a rugosidade:** o núcleo é simétrico em `(r, l)` e está
/// normalizado pelo hemisfério, logo `∫ T(α, ψ(r)) dω_r = Ω_calote` — para o espelho, para as
/// linhas amostradas pelo lóbulo e para as integradas sobre a calote. Controlo: sem a
/// normalização certa a linha `α = 1` sai do sítio.
#[test]
fn a_tabela_conserva_a_energia() {
    for raio_graus in [0.3, 1.0, 5.0] {
        let p = Perfil::de(f64::from(raio_graus as f32).to_radians());
        let om = p.angulo_solido();
        let t = super::tabela(&p);
        let sol = super::Sol {
            dir: [0.0, 1.0, 0.0],
            raio: 0.0,
            radiancia: [1.0; 3],
            energia: [1.0; 3],
            fracao: 0.0,
            tabela: t,
        };
        for ri in 0..RUGOSIDADES {
            let a = (ri as f32 / (RUGOSIDADES - 1) as f32).powi(2);
            // ∫ T dω = 2π ∫ T d(cos ψ), com cos ψ = 1 − u², u ∈ [0, √2] (o eixo da tabela, mais fino).
            const N: usize = 1 << 16;
            let du = std::f64::consts::SQRT_2 / N as f64;
            let s: f64 = (0..N)
                .map(|i| {
                    let u = (i as f64 + 0.5) * du;
                    f64::from(sol.tabela_em(a, (1.0 - u * u) as f32)) * 2.0 * u * du
                })
                .sum::<f64>()
                * std::f64::consts::TAU;
            let rel = (s - om).abs() / om;
            // ⚠️ Duas faixas MEDIDAS (03/10): onde o disco ou o lóbulo cobre 8 passos da tabela, a
            // energia fica a `≤ 0,13 %`; abaixo disso (um sol de `0,3°` quase espelho) a tabela só o
            // representa a `≤ 5 %` — o regime do texel de `0,35°` do próprio atlas.
            let resolvido = p.suporte.max(2.0 * f64::from(a)) >= 8.0 * super::PASSO;
            let tecto = if resolvido { 0.003 } else { 0.06 };
            assert!(rel < tecto, "raio {raio_graus}°, α {a:.5}: ∫T = {s:.6e} contra Ω {om:.6e} ({:.2} %)", 100.0 * rel);
        }
    }
}

/// ⭐ **As duas quadraturas concordam onde a tabela passa de uma para a outra** (o lóbulo estreito
/// amostrado, a calote integrada) — senão a tabela tem um degrau em `√α`.
#[test]
fn as_duas_quadraturas_concordam_no_corte() {
    for raio_graus in [0.3f64, 1.0, 5.0] {
        let p = Perfil::de(raio_graus.to_radians());
        let a = alfa_da_calote(&p);
        let a2 = a * a;
        let pico = pela_calote(&p, a2, 0).max(pelo_lobo(&p, a2, 0));
        for ai in [0, 1, 2, 4, 8, 16, 32, 64] {
            let (l, c) = (pelo_lobo(&p, a2, ai), pela_calote(&p, a2, ai));
            assert!(
                (l - c).abs() < 0.02 * pico,
                "raio {raio_graus}°, α {a:.5}, ψ-índice {ai}: lóbulo {l:.5} × calote {c:.5}"
            );
        }
    }
}

/// ⭐⭐⭐ **CONSERVAÇÃO: céu sem sol + sol = o céu inteiro**, pela lei que o material usa (o atlas do
/// céu sem o disco e a tabela do sol), medida contra a SOMA EXACTA sobre o panorama (`verdade`) —
/// em direcções espalhadas e junto do sol, nas rugosidades da convolução exacta e na irradiância.
///
/// ⚠️ **A régua NÃO é o atlas do céu inteiro** (03/10): ele guarda o sol num texel de `3–4°` e
/// desloca-o meio texel — `1–5 %` de erro onde o sol pesa, MAIOR que o da separação. ⇒ a separação
/// tem de errar MENOS que ele. Controlo: sem o sol o erro é a energia dele.
#[test]
fn o_ceu_sem_sol_mais_o_sol_e_o_ceu() {
    use rayon::prelude::*;
    for e in [Embarcado::Cidade, Embarcado::Nascer] {
        let p = e.panorama();
        let (inteiro, partido) = (Ceu::novo(&p), Ceu::com_sol(&p));
        let sol = partido.sol().expect("este céu tem sol");
        let piso = 0.02 * luma(p.media());
        let s = sol.dir.map(f64::from);
        let mut dirs = fibonacci(96);
        // 32 direcções a menos de 20° do sol.
        for (k, d) in fibonacci(400).into_iter().enumerate() {
            let c = f64::from(d[0]) * s[0] + f64::from(d[1]) * s[1] + f64::from(d[2]) * s[2];
            if c > 20.0f64.to_radians().cos() && k % 2 == 0 {
                dirs.push(d);
            }
        }
        let rel = |a: [f32; 3], b: [f32; 3]| ((luma(a) - luma(b)).abs() / (luma(b) + piso)) as f32;
        let erros: Vec<(f32, f32, f32)> = dirs
            .par_iter()
            .flat_map_iter(|d| {
                [0.1f32, 0.25, 0.5, 1.0].into_iter().map(|a| {
                    let v = crate::tests::verdade(&p, *d, a);
                    let c = partido.radiance(*d, a);
                    let t = sol.radiance(*d, a);
                    let soma = [c[0] + t[0], c[1] + t[1], c[2] + t[2]];
                    (rel(soma, v), rel(inteiro.radiance(*d, a), v), rel(c, v))
                })
            })
            .collect();
        let q = |f: &dyn Fn(&(f32, f32, f32)) -> f32| {
            let mut v: Vec<f32> = erros.iter().map(f).collect();
            v.sort_by(f32::total_cmp);
            (v[v.len() / 2], v[(v.len() - 1) * 99 / 100], v[v.len() - 1])
        };
        let (sp, ip, cp) = (q(&|e| e.0), q(&|e| e.1), q(&|e| e.2));
        eprintln!("{e:?}: separado p50 {:.3} p99 {:.3} máx {:.3} | atlas inteiro p50 {:.3} p99 {:.3} máx {:.3} | sem o sol máx {:.3}", sp.0, sp.1, sp.2, ip.0, ip.1, ip.2, cp.2);
        assert!(sp.1 <= ip.1 && sp.2 <= ip.2, "{e:?}: a separação erra mais que o atlas inteiro");
        assert!(sp.2 < 0.02, "{e:?}: céu sem sol + sol erra {:.2} %", 100.0 * sp.2);
        assert!(cp.2 > 0.2, "{e:?}: o controlo (sem o sol) devia errar muito: {:.2} %", 100.0 * cp.2);
    }
}

fn fibonacci(n: usize) -> Vec<[f32; 3]> {
    let phi = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());
    (0..n)
        .map(|i| {
            let y = 1.0 - 2.0 * (i as f32 + 0.37) / n as f32;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let a = phi * i as f32 + 0.11;
            [r * a.cos(), y, r * a.sin()]
        })
        .collect()
}

#[test]
fn o_bordo_tem_dois_passos_da_tabela() {
    // A tabela nunca representa uma aresta mais dura do que o próprio passo (`ANGULOS`).
    assert!((BORDO - 4.0 / (ANGULOS - 1) as f64).abs() < 1.0e-15);
    let _ = unit([1.0, 0.0, 0.0]);
}

fn angulo(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0).acos()
}

/// ⚙️ **O instrumento do sol** — por céu: a luz média, o pico, e para cada limiar (em múltiplos da
/// média) quantos texels passam, o ângulo sólido deles, a fracção da energia e da irradiância do
/// chão que o EXCESSO leva, a elevação do centróide e o raio do disco uniforme com o mesmo 2.º
/// momento (`θ = √(2·⟨ψ²⟩)`).
#[test]
#[ignore = "instrumento: corre à mão com --nocapture"]
fn instrumento_sol() {
    for e in Embarcado::TODOS {
        let p: Panorama = e.panorama();
        let (mut potencia, mut chao, mut pico, mut onde) = (0.0f64, 0.0f64, 0.0f64, (0, 0));
        for y in 0..p.altura {
            let om = p.angulo_solido(y);
            for x in 0..p.largura {
                let l = luma(p.rgb[(y * p.largura + x) as usize]);
                potencia += l * om;
                chao += l * om * f64::from(p.direcao(x, y)[1].max(0.0));
                if l > pico {
                    pico = l;
                    onde = (x, y);
                }
            }
        }
        let media = potencia / (4.0 * std::f64::consts::PI);
        if let (_, Some(s)) = p.separa_sol() {
            eprintln!(
                "{:>10} SOL: altura {:.1}° raio {:.2}° leva {:.1} % · radiância {:.0}",
                e.chave(),
                f64::from(s.dir[1]).asin().to_degrees(),
                f64::from(s.raio).to_degrees(),
                100.0 * s.fracao,
                luma(s.radiancia)
            );
        }
        let dp = p.direcao(onde.0, onde.1);
        eprintln!(
            "{:>10} {}x{} média {media:.4} pico {pico:.1} ({:.0}× a média) elev {:.1}°",
            e.chave(),
            p.largura,
            p.altura,
            pico / media,
            f64::from(dp[1]).asin().to_degrees()
        );
        for k in [4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0, 1024.0] {
            let t = k * media;
            let (mut n, mut om_t, mut exc, mut exc_chao) = (0u32, 0.0f64, 0.0f64, 0.0f64);
            let mut c = [0.0f64; 3];
            for y in 0..p.altura {
                let om = p.angulo_solido(y);
                for x in 0..p.largura {
                    let l = luma(p.rgb[(y * p.largura + x) as usize]);
                    if l > t {
                        let d = p.direcao(x, y).map(f64::from);
                        n += 1;
                        om_t += om;
                        exc += (l - t) * om;
                        exc_chao += (l - t) * om * d[1].max(0.0);
                        for i in 0..3 {
                            c[i] += (l - t) * om * d[i];
                        }
                    }
                }
            }
            if n == 0 {
                continue;
            }
            let lc = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
            let cd = c.map(|v| v / lc);
            let mut m2 = 0.0f64;
            for y in 0..p.altura {
                let om = p.angulo_solido(y);
                for x in 0..p.largura {
                    let l = luma(p.rgb[(y * p.largura + x) as usize]);
                    if l > t {
                        let a = angulo(p.direcao(x, y).map(f64::from), cd);
                        m2 += (l - t) * om * a * a;
                    }
                }
            }
            eprintln!(
                "    T={k:>6}×  texels {n:>6}  Ω {om_t:.5} sr  excesso {:.1} % da energia, {:.1} % do chão · elev {:.1}° · θ(2.º momento) {:.2}° · θ(Ω) {:.2}°",
                100.0 * exc / potencia,
                100.0 * exc_chao / chao,
                cd[1].asin().to_degrees(),
                (2.0 * m2 / exc).sqrt().to_degrees(),
                (1.0 - om_t / std::f64::consts::TAU).acos().to_degrees()
            );
        }
    }
}
