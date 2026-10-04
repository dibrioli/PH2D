//! ⭐⭐ **As capturas lidas de volta** — o gémeo da leitura (`sonda_le.wgsl`) na CPU, sobre os níveis que
//! a placa escreveu: a captura vê as vizinhas na direcção e à distância certas (contra a geometria do
//! oráculo), e a paralaxe põe o reflexo onde o raio as acerta.

use crate::gpu::sondas_impl::{LADO, NIVEIS, PARALAXE};
use crate::tests_chao_tapa::PECAS;
use crate::tests_contacto::{Peca, norm};
use crate::tests_reflexo::{DE, desenha, desenhista, fosca, oraculo};

/// Um nível de uma camada lido de volta.
pub(crate) struct Nivel {
    w: u32,
    t: Vec<[f32; 4]>,
}

/// A leitura bilinear da placa (com a beira presa).
fn bilinear(n: &Nivel, uv: [f32; 2]) -> [f32; 4] {
    let w = n.w as i32;
    let (x, y) = (uv[0] * n.w as f32 - 0.5, uv[1] * n.w as f32 - 0.5);
    let (x0, y0) = (x.floor(), y.floor());
    let (tx, ty) = (x - x0, y - y0);
    let at = |i: i32, j: i32| n.t[(j.clamp(0, w - 1) * w + i.clamp(0, w - 1)) as usize];
    let (a, b) = (at(x0 as i32, y0 as i32), at(x0 as i32 + 1, y0 as i32));
    let (c, d) = (
        at(x0 as i32, y0 as i32 + 1),
        at(x0 as i32 + 1, y0 as i32 + 1),
    );
    std::array::from_fn(|e| {
        let cima = a[e] + (b[e] - a[e]) * tx;
        let baixo = c[e] + (d[e] - c[e]) * tx;
        cima + (baixo - cima) * ty
    })
}

fn uv(d: [f32; 3], k: u32) -> [f32; 2] {
    let (a, b) = ph2d_sky::oct(d);
    let w = (LADO >> k) as f32;
    [
        (1.0 + (a * 0.5 + 0.5) * (w - 2.0)) / w,
        (1.0 + (b * 0.5 + 0.5) * (w - 2.0)) / w,
    ]
}

/// `sonda_le`: o nível contínuo, cada um na coordenada dele.
pub(crate) fn le(niveis: &[Nivel], d: [f32; 3], lod: f32) -> [f32; 4] {
    let l = lod.clamp(0.0, (NIVEIS - 1) as f32);
    let k0 = (l as u32).min(NIVEIS - 2);
    let a = bilinear(&niveis[k0 as usize], uv(d, k0));
    let b = bilinear(&niveis[k0 as usize + 1], uv(d, k0 + 1));
    std::array::from_fn(|e| a[e] + (b[e] - a[e]) * (l - k0 as f32))
}

/// O texel MAIS PRÓXIMO do nível `k` (sem filtro).
pub(crate) fn texel(niveis: &[Nivel], d: [f32; 3], k: u32) -> [f32; 4] {
    let n = &niveis[k as usize];
    let p = uv(d, k);
    let w = n.w as i32;
    let at = |s: f32| ((s * n.w as f32).floor() as i32).clamp(0, w - 1);
    n.t[(at(p[1]) * w + at(p[0])) as usize]
}

/// A camada `camada` em todos os níveis.
pub(crate) fn camada(fw: &crate::Forward, camada: u32) -> Vec<Nivel> {
    (0..NIVEIS)
        .map(|k| Nivel {
            w: LADO >> k,
            t: fw.le_sonda(camada, k).expect("a captura lê-se"),
        })
        .collect()
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// O raio `o + t·d` acerta a peça `q` a que distância (`t > 0`)?
pub(crate) fn acerta_em((c, r, caixa): Peca, o: [f32; 3], d: [f32; 3]) -> Option<f32> {
    let q = [0, 1, 2].map(|e| o[e] - c[e]);
    if caixa {
        let (mut t0, mut t1) = (1.0e-5f32, f32::INFINITY);
        for e in 0..3 {
            if d[e].abs() < 1.0e-9 {
                if q[e].abs() > r {
                    return None;
                }
                continue;
            }
            let (a, b) = ((-r - q[e]) / d[e], (r - q[e]) / d[e]);
            t0 = t0.max(a.min(b));
            t1 = t1.min(a.max(b));
        }
        return (t0 <= t1).then_some(t0);
    }
    let b = dot(q, d);
    let disc = b * b - (dot(q, q) - r * r);
    if disc < 0.0 {
        return None;
    }
    let (t0, t1) = (-b - disc.sqrt(), -b + disc.sqrt());
    if t0 > 1.0e-5 {
        Some(t0)
    } else if t1 > 1.0e-5 {
        Some(t1)
    } else {
        None
    }
}

/// A primeira peça (que não `menos`) que o raio acerta.
fn primeira(o: [f32; 3], d: [f32; 3], menos: usize) -> Option<f32> {
    PECAS
        .iter()
        .enumerate()
        .filter(|(k, _)| *k != menos)
        .filter_map(|(_, q)| acerta_em(*q, o, d))
        .min_by(f32::total_cmp)
}

/// ⭐⭐ **A captura vê as vizinhas na direcção e à distância certas** — a captura da esfera pousada,
/// texel a texel no nível `0`, contra o raio do centro dela: onde ele acerta uma vizinha (e os texels à
/// volta também), cobertura `1` e a distância da geometria; onde ninguém, cobertura `0`.
#[test]
#[ignore = "precisa de aparelho"]
fn a_captura_ve_as_vizinhas() {
    let Some(mut fw) = desenhista() else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    let f = fosca();
    let _ = desenha(&mut fw, &[0, 1, 2], &[f, f, f], false);
    let dist = camada(&fw, 3);
    let c = PECAS[1].0;
    let w = LADO as i32;
    let dir = |x: f32, y: f32| {
        ph2d_sky::de_oct(
            (x - 1.0) / (w as f32 - 2.0) * 2.0 - 1.0,
            (y - 1.0) / (w as f32 - 2.0) * 2.0 - 1.0,
        )
    };
    let (mut dentro, mut fora, mut erro_cob, mut erro_d, mut pior_d) =
        (0usize, 0usize, 0usize, 0.0f32, 0.0f32);
    for y in 1..w - 1 {
        for x in 1..w - 1 {
            let hits: Vec<Option<f32>> =
                [(0.5, 0.5), (-0.5, 0.0), (1.5, 0.0), (0.0, -0.5), (0.0, 1.5)]
                    .iter()
                    .map(|&(dx, dy)| primeira(c, dir(x as f32 + dx, y as f32 + dy), 1))
                    .collect();
            let t = dist[0].t[(y * w + x) as usize];
            if hits.iter().all(Option::is_some) {
                dentro += 1;
                let d = t[0] / t[1].max(1.0e-6);
                let e = (d - hits[0].unwrap_or(0.0)).abs() / hits[0].unwrap_or(1.0);
                erro_d += e;
                pior_d = pior_d.max(e);
                erro_cob += usize::from(t[1] < 0.99);
            } else if hits.iter().all(Option::is_none) {
                fora += 1;
                erro_cob += usize::from(t[1] > 0.01);
            }
        }
    }
    eprintln!(
        "{dentro} texels a ver uma vizinha, {fora} a ver nada · cobertura errada {erro_cob} · distância \
         |Δ|/d média {:.4}, máx {pior_d:.4}",
        erro_d / dentro.max(1) as f32
    );
    assert!(dentro > 200 && fora > 5000, "a captura mal vê a cena");
    assert!(
        erro_cob * 200 < dentro + fora,
        "a cobertura não é a da geometria: {erro_cob}"
    );
    assert!(
        erro_d / (dentro as f32) < 0.01 && pior_d < 0.05,
        "a distância não é a da geometria"
    );
}

/// A paralaxe com um passo por nível em `lods` (do grosso ao fino) — a de `sonda_le.wgsl` quando
/// `lods` é a dela.
pub(crate) fn paralaxe(
    dist: &[Nivel],
    c: [f32; 3],
    p: [f32; 3],
    r: [f32; 3],
    lods: &[f32],
) -> [f32; 3] {
    let q = [0, 1, 2].map(|e| p[e] - c[e]);
    let mut d = r;
    // O primeiro passo: a distância média em toda a volta (`sonda_esfera`: o nível mais largo nos seis
    // eixos e em `r`).
    let topo = (NIVEIS - 1) as f32;
    let eixos = [
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
    ];
    let esfera = eixos.iter().fold(le(dist, r, topo), |a, e| {
        let g = le(dist, *e, topo);
        [a[0] + g[0], a[1] + g[1], 0.0, 0.0]
    });
    let passo = |g: [f32; 4], d: &mut [f32; 3]| {
        let dd = g[0] / g[1].max(1.0e-6);
        let b = dot(q, r);
        let disc = b * b - (dot(q, q) - dd * dd);
        let t = disc.max(0.0).sqrt() - b;
        if disc >= 0.0 && t > 0.0 {
            // `smoothstep(0, COBERTURA_PLENA, cobertura)` — o peso do passo (o de `sonda_le.wgsl`).
            let x = (g[1] / crate::gpu::sondas_impl::COBERTURA_PLENA).clamp(0.0, 1.0);
            let w = x * x * (3.0 - 2.0 * x);
            let novo = norm([0, 1, 2].map(|e| q[e] + t * r[e]));
            *d = norm([0, 1, 2].map(|e| d[e] + (novo[e] - d[e]) * w));
        }
    };
    passo(esfera, &mut d);
    for &lod in lods {
        let g = le(dist, d, lod);
        passo(g, &mut d);
    }
    d
}

/// ⭐⭐ **A paralaxe acerta onde o raio bate** — nos pixels do espelho do oráculo, o ângulo entre a
/// direcção que a leitura usa e a do ponto acertado visto do centro da captura (raios que acertam uma
/// vizinha), e a cobertura lida no fim: os que acertam e leem nada, os que falham e leem vizinha. Os
/// outros esquemas imprimem-se ao lado (a tabela da escolha, em [`PARALAXE`]); o de produção afirma.
#[test]
#[ignore = "precisa de aparelho"]
fn a_paralaxe_acerta_onde_o_raio_bate() {
    let Some(mut fw) = desenhista() else {
        return;
    };
    let f = fosca();
    let _ = desenha(&mut fw, &[0, 1, 2], &[f, f, f], false);
    let (pontos, _) = oraculo();
    let vista = norm(DE.map(|x| -x));
    let esquemas: &[&[f32]] = &[
        &PARALAXE,
        &[],
        &[4.0, 2.0, 1.0, 0.0],
        &[5.0, 3.0, 1.0, 0.0],
        &[5.0, 3.0, 2.0, 1.0, 0.0],
        &[5.0, 4.0, 2.0, 1.0, 0.0],
    ];
    let mut producao = Vec::new();
    for (espelho, camada_d) in [(1usize, 3u32), (0, 1)] {
        let dist = camada(&fw, camada_d);
        let c = PECAS[espelho].0;
        let mut casos = Vec::new();
        for p in pontos.iter().filter(|p| p.obj == espelho) {
            let fn_ = dot(vista, p.n);
            let r: [f32; 3] = std::array::from_fn(|e| vista[e] - 2.0 * fn_ * p.n[e]);
            // O ponto acertado, e se o CENTRO da captura o vê (senão nenhuma captura de um ponto o tem).
            let alvo = primeira(p.p, r, espelho).map(|t| {
                let x = [0, 1, 2].map(|e| p.p[e] + t * r[e] - c[e]);
                let l = dot(x, x).sqrt();
                let visto = primeira(c, norm(x), espelho).is_some_and(|u| u > l - 1.0e-4);
                (norm(x), visto)
            });
            casos.push((p.p, r, alvo));
        }
        for lods in esquemas {
            let (mut n, mut s, mut pior, mut sem_viz, mut falsas, mut nf) =
                (0usize, 0.0f32, 0.0f32, 0usize, 0usize, 0usize);
            let (mut escondidos, mut sem_viz_visto) = (0usize, 0usize);
            for (p, r, alvo) in &casos {
                let d = paralaxe(&dist, c, *p, *r, lods);
                let cob = le(&dist, d, 0.0)[1];
                match alvo {
                    Some((a, visto)) => {
                        escondidos += usize::from(!visto);
                        sem_viz_visto += usize::from(*visto && cob < 0.5);
                        let ang = dot(d, *a).clamp(-1.0, 1.0).acos().to_degrees();
                        s += ang;
                        pior = pior.max(ang);
                        n += 1;
                        sem_viz += usize::from(cob < 0.5);
                    }
                    None => {
                        nf += 1;
                        falsas += usize::from(cob > 0.5);
                    }
                }
            }
            let media = s / n.max(1) as f32;
            eprintln!(
                "espelho {espelho} lods {lods:?}: {n} acertam ({escondidos} num ponto que o centro não vê) · \
                 ângulo médio {:.2}°, máx {pior:.2}° · leem nada {sem_viz} (dos vistos {sem_viz_visto}) · \
                 {nf} falham, leem vizinha {falsas}",
                media
            );
            if std::ptr::eq(*lods, &PARALAXE[..]) {
                producao.push((espelho, n, nf, media, pior, sem_viz_visto, falsas));
            }
        }
    }
    for (espelho, n, nf, media, pior, sem_viz_visto, falsas) in producao {
        // Medido (04/10, octaedro `256`): ver a tabela em [`PARALAXE`].
        let (barra_media, barra_pior, sem_viz_max) = if espelho == 1 {
            (0.1, 1.0, 0)
        } else {
            (0.3, 4.0, 200)
        };
        assert!(n > 200 && nf > 4000, "o oráculo mudou: {n} / {nf} raios");
        assert!(
            media < barra_media && pior < barra_pior,
            "a paralaxe do espelho {espelho} afastou-se: {media:.3}° / {pior:.2}°"
        );
        assert!(
            sem_viz_visto <= sem_viz_max && falsas * 100 < nf,
            "o espelho {espelho} lê o que não está lá: {sem_viz_visto} sem vizinha, {falsas} falsas"
        );
    }
}
