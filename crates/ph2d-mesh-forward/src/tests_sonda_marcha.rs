//! ⭐⭐ **A paralaxe de PERTO, na CPU** — os raios do cromo do oráculo de perto
//! ([`crate::tests_reflexo_perto`]) sobre a captura lida de volta: quem acerta uma vizinha e quem falha,
//! e em que direcção a leitura cai, por esquema — o ponto fixo de [`PARALAXE`] contra a BUSCA ao longo
//! do raio sobre a distância guardada (Szirmay-Kalos, Aszódi, Lazányi e Premecz 2005, *Approximate
//! ray-tracing on the GPU with distance impostors*): passos iguais em ângulo no arco que a recta
//! projecta do centro (`λ(θ) = |q| sen Θ / sen(Θ − θ)`), e a bissecção no primeiro passo que fica atrás.

use crate::gpu::sondas_impl::PARALAXE;
use crate::tests_chao_tapa::metal;
use crate::tests_contacto::norm;
use crate::tests_reflexo_perto::{DE, PECAS, desenha, desenhista, oraculo};
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
    let Some(mut fw) = desenhista() else {
        return;
    };
    let _ = desenha(&mut fw, metal(0.0), true);
    let dist = camada(&fw, 1);
    let c = PECAS[0].0;
    let vista = norm(DE.map(|x| -x));
    let casos: Vec<_> = oraculo()
        .iter()
        .map(|p| {
            let fn_ = dot(vista, p.n);
            let r: [f32; 3] = std::array::from_fn(|e| vista[e] - 2.0 * fn_ * p.n[e]);
            let alvo = PECAS[1..]
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
