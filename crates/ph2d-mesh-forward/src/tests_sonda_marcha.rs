//! ⭐⭐ **A paralaxe de PERTO, na CPU** — os raios do cromo do oráculo de perto
//! ([`crate::tests_reflexo_perto`]) sobre a captura lida de volta: quem acerta uma vizinha e quem falha,
//! e em que direcção a leitura cai, por esquema — o ponto fixo de [`PARALAXE`] contra a BUSCA ao longo
//! do raio sobre a distância guardada (Szirmay-Kalos, Aszódi, Lazányi e Premecz 2005, *Approximate
//! ray-tracing on the GPU with distance impostors*): passos iguais em ângulo no arco que a recta
//! projecta do centro (`λ(θ) = |q| sen Θ / sen(Θ − θ)`), e a bissecção no primeiro passo que fica atrás.

use crate::gpu::sondas_impl::PARALAXE;
use crate::tests_chao_tapa::metal;
use crate::tests_contacto::norm;
use crate::tests_reflexo_perto::{PERTO, desenha, desenhista, oraculo};
use crate::tests_sonda_cpu::{Nivel, acerta_em, camada, le};

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A busca: `passos` iguais em ângulo e `refino` bissecções; `None` = o raio não acerta nada.
pub(crate) fn marcha(
    dist: &[Nivel],
    c: [f32; 3],
    p: [f32; 3],
    r: [f32; 3],
    passos: u32,
    refino: u32,
    espessura: f32,
) -> Option<[f32; 3]> {
    let q = [0, 1, 2].map(|e| p[e] - c[e]);
    let ql = dot(q, q).sqrt();
    let qh = q.map(|x| x / ql);
    let ct = dot(qh, r).clamp(-1.0, 1.0);
    let th = ct.acos();
    if th < 1.0e-4 {
        return None;
    }
    let w = norm([0, 1, 2].map(|e| r[e] - ct * qh[e]));
    let u = |a: f32| [0, 1, 2].map(|e| a.cos() * qh[e] + a.sin() * w[e]);
    let lam = |a: f32| ql * th.sin() / (th - a).sin();
    let atras = |a: f32| {
        let g = le(dist, u(a), 0.0);
        g[1] > 0.5 && lam(a) >= g[0] / g[1]
    };
    let (mut ant, mut frente) = (0.0f32, true);
    for k in 1..=passos {
        let a = th * k as f32 / (passos + 1) as f32;
        let at = atras(a);
        if at && frente {
            let (mut lo, mut hi) = (ant, a);
            for _ in 0..refino {
                let m = 0.5 * (lo + hi);
                if atras(m) {
                    hi = m;
                } else {
                    lo = m;
                }
            }
            let g = le(dist, u(hi), 0.0);
            if lam(hi) <= g[2] / g[1].max(1.0e-6) * (1.0 + espessura) {
                return Some(u(hi));
            }
        }
        frente = !at;
        ant = a;
    }
    None
}

/// ⭐ **Sonda** (imprime): nos raios do cromo de perto, por esquema, os que acertam uma vizinha e leem
/// nada, os que falham e leem vizinha, e o ângulo médio/máximo nos que acertam.
#[test]
#[ignore = "precisa de aparelho"]
fn sonda_da_paralaxe_de_perto() {
    let Some(mut fw) = desenhista(&PERTO) else {
        return;
    };
    let _ = desenha(&PERTO, &mut fw, metal(0.0), true);
    let dist = camada(&fw, 1);
    let c = PERTO.pecas[0].0;
    let vista = norm(PERTO.de.map(|x| -x));
    let casos: Vec<_> = oraculo(&PERTO)
        .iter()
        .map(|p| {
            let fn_ = dot(vista, p.n);
            let r: [f32; 3] = std::array::from_fn(|e| vista[e] - 2.0 * fn_ * p.n[e]);
            let alvo = PERTO.pecas[1..]
                .iter()
                .filter_map(|q| acerta_em(*q, p.p, r))
                .min_by(f32::total_cmp)
                .map(|t| norm([0, 1, 2].map(|e| p.p[e] + t * r[e] - c[e])));
            (p.p, r, alvo)
        })
        .collect();
    let cob = |d: [f32; 3]| le(&dist, d, 0.0)[1];
    let relata = |nome: &str, f: &dyn Fn([f32; 3], [f32; 3]) -> Option<[f32; 3]>| {
        let (mut n, mut s, mut pior, mut nada, mut nf, mut falsas) =
            (0usize, 0.0f32, 0.0f32, 0, 0, 0);
        for (p, r, alvo) in &casos {
            let d = f(*p, *r);
            match alvo {
                Some(a) => {
                    n += 1;
                    match d.filter(|d| cob(*d) > 0.5) {
                        Some(d) => {
                            let ang = dot(d, *a).clamp(-1.0, 1.0).acos().to_degrees();
                            s += ang;
                            pior = pior.max(ang);
                        }
                        None => nada += 1,
                    }
                }
                None => {
                    nf += 1;
                    falsas += usize::from(d.is_some_and(|d| cob(d) > 0.5));
                }
            }
        }
        eprintln!(
            "{nome}: {n} acertam · leem nada {nada} · ângulo médio {:.3}°, máx {pior:.2}° · {nf} falham, leem \
             vizinha {falsas}",
            s / (n - nada).max(1) as f32
        );
    };
    relata("ponto fixo de produção", &|p, r| {
        Some(crate::tests_sonda_cpu::paralaxe(&dist, c, p, r, &PARALAXE))
    });
    for (passos, refino, esp) in [
        (16u32, 6u32, 0.02f32),
        (24, 6, 0.0),
        (24, 6, 0.02),
        (24, 6, 0.05),
        (32, 6, 0.02),
    ] {
        relata(
            &format!("busca com FUNDO {passos} passos + {refino} bissecções, folga {esp}"),
            &|p, r| marcha(&dist, c, p, r, passos, refino, esp),
        );
    }
}

/// Sonda (imprime): com `PH2D_REFLEXO_FOTOS=<pasta>`, os níveis da cor da captura do cromo do PAR
/// (`nivel_k.pgm`, a cobertura) — o pré-filtro à vista.
#[test]
#[ignore = "precisa de aparelho"]
fn sonda_niveis_da_captura() {
    let Ok(pasta) = std::env::var("PH2D_REFLEXO_FOTOS") else {
        return;
    };
    let v = &crate::tests_reflexo_perto::PAR;
    let Some(mut fw) = desenhista(v) else {
        return;
    };
    let _ = desenha(v, &mut fw, metal(0.3), true);
    for k in 0..crate::gpu::sondas_impl::NIVEIS {
        let t = fw.le_sonda(0, k).expect("a captura lê-se");
        let w = crate::gpu::sondas_impl::LADO >> k;
        let mut f = format!("P5 {w} {w} 255\n").into_bytes();
        // A COBERTURA (`a`): onde a captura vê vizinha.
        f.extend(t.iter().map(|c| (c[3].clamp(0.0, 1.0) * 255.0) as u8));
        let _ = std::fs::write(format!("{pasta}/nivel_{k}.pgm"), f);
    }
}

/// Sonda (imprime): com `PH2D_REFLEXO_FOTOS=<pasta>`, a cobertura que a leitura da CPU (o gémeo de
/// `sonda_le`) dá a cada pixel do cromo do PAR a `0,3`, na direcção reflectida crua (`par_cpu.pgm`) — a
/// leitura e os dados, sem a paralaxe.
#[test]
#[ignore = "precisa de aparelho"]
fn sonda_leitura_cpu_do_par() {
    let Ok(pasta) = std::env::var("PH2D_REFLEXO_FOTOS") else {
        return;
    };
    let v = &crate::tests_reflexo_perto::PAR;
    let Some(mut fw) = desenhista(v) else {
        return;
    };
    let _ = desenha(v, &mut fw, metal(0.3), true);
    let cor = camada(&fw, 0);
    let vista = norm(v.de.map(|x| -x));
    let lod = 0.3 * (crate::gpu::sondas_impl::NIVEIS - 1) as f32;
    let mut img = vec![0u8; 512 * 512];
    for p in oraculo(v) {
        let fn_ = dot(vista, p.n);
        let r: [f32; 3] = std::array::from_fn(|e| vista[e] - 2.0 * fn_ * p.n[e]);
        img[(p.j * 512 + p.i) as usize] = (le(&cor, r, lod)[3].clamp(0.0, 1.0) * 255.0) as u8;
    }
    let mut f = b"P5 512 512 255\n".to_vec();
    f.extend_from_slice(&img);
    let _ = std::fs::write(format!("{pasta}/par_cpu.pgm"), f);
}
