//! ⭐⭐⭐ **O REFLEXO PELA TELA** — o 5.º report do dono (04/10): o ALUMÍNIO a `0` mostrava a verde, um VAZIO
//! e só depois a azul. O vazio é a parte da azul que cada ponto do espelho vê e o CENTRO da captura não (tapada
//! pela verde): o limite de UMA captura por peça. Esta régua mede só esses px; o controlo, os que acertam a
//! mesma azul e o centro vê.

use crate::tests_chao_tapa::metal;
use crate::tests_reflexo_junta::{foto, mede};
use crate::tests_reflexo_perto::{
    FOTO2, LADO, Px, VAZIO, Vista, desenha, desenhista, oraculo, visto_do_centro, vizinha_refletida,
};

/// Os px (índices em `px`) cujo raio reflectido, pela geometria, acerta uma vizinha com o ponto `escondido`
/// do centro do espelho (ou visto, o controlo) — a mais de `2 px` de outra classe e da silhueta.
pub(crate) fn zona(v: &Vista, px: &[Px], escondido: bool) -> Vec<usize> {
    let l = LADO as i32;
    let mut mapa: Vec<Option<(bool, bool)>> = vec![None; (l * l) as usize];
    for p in px {
        mapa[(p.j * LADO + p.i) as usize] =
            Some((vizinha_refletida(v, p).is_some(), visto_do_centro(v, p)));
    }
    let at = |x: i32, y: i32| {
        ((0..l).contains(&x) && (0..l).contains(&y))
            .then(|| mapa[(y * l + x) as usize])
            .flatten()
    };
    let alvo = Some((true, !escondido));
    px.iter()
        .enumerate()
        .filter(|(_, p)| {
            let (i, j) = (p.i as i32, p.j as i32);
            (-2..=2).all(|dy| (-2..=2).all(|dx| at(i + dx, j + dy) == alvo))
        })
        .map(|(q, _)| q)
        .collect()
}

/// ⏱ **Sonda: o espelho mostra a vizinha que o centro dele não vê?** — nítido e `0,05`. CONTROLO: onde o centro
/// vê a vizinha. ⚠️ Medido (04/10): VERMELHA (`362 px`, `|Δ|` `0,218`; controlo `0,0086`) — o crescente do
/// cromo junto da verde, que o centro do alumínio não vê; mas ele está FORA da tela nesta vista (o olho do app
/// de perto), logo nenhum reflexo pela tela o conserta. Fica sonda até haver cura (a 2.ª camada na captura).
#[test]
#[ignore = "sonda: precisa de aparelho"]
fn sonda_do_vazio() {
    let v = &VAZIO;
    let Some(mut fw) = desenhista(v) else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    let px = oraculo(v);
    let (vazio, visto) = (zona(v, &px, true), zona(v, &px, false));
    let mut falhas = Vec::new();
    for (rug, cols) in [(0.0f32, (0usize, 2usize)), (v.rug2, (1, 3))] {
        let viz = desenha(v, &mut fw, metal(rug), true);
        let solo = desenha(v, &mut fw, metal(rug), false);
        let m = mede(&px, &vazio, (&viz, &solo), cols);
        let c = mede(&px, &visto, (&viz, &solo), cols);
        foto(&px, &vazio, (&viz, &solo), cols, &format!("vazio_{rug}"));
        foto(
            &px,
            &visto,
            (&viz, &solo),
            cols,
            &format!("vazio_visto_{rug}"),
        );
        println!(
            "SONDA vazio, espelho {rug}: a azul que o centro NÃO vê {} px · |Δ| médio {:.4} · |Δ| > 0,2: {} — \
             CONTROLO, a que o centro vê {} px · {:.4} · {}",
            m.0, m.1, m.2, c.0, c.1, c.2
        );
        if !(m.1 < BARRA.0 && m.2 <= BARRA.1) {
            falhas.push(rug);
        }
    }
    println!("SONDA vazio: acima da barra {BARRA:?} em {falhas:?}");
}

/// `(|Δ| médio no vazio, px com |Δ| > 0,2)` — a barra que a cura terá de passar (a do controlo, `0,0086 / 48`, folgada).
const BARRA: (f32, usize) = (0.03, 50);

/// ⏱ **Sonda (CPU, sem placa): quanto do vazio o OLHO vê** — numa pose (o espelho primeiro) e um olho: dos
/// px do espelho cujo raio reflectido acerta uma vizinha num ponto que o centro NÃO vê, quantos acertam um
/// ponto que o olho vê (o que o reflexo pela tela pode consertar; o resto só uma 2.ª camada na captura).
#[test]
#[ignore = "sonda: à mão"]
fn sonda_do_vazio_a_vista() {
    use crate::tests_contacto::{Peca, norm};
    use crate::tests_sonda_cpu::acerta_em;
    let d3 = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let primeiro = |ps: &[Peca], o: [f32; 3], d: [f32; 3], fora: Option<usize>| {
        ps.iter()
            .enumerate()
            .filter(|(k, _)| Some(*k) != fora)
            .filter_map(|(k, q)| acerta_em(*q, o, d).map(|t| (t, k)))
            .min_by(|a, b| a.0.total_cmp(&b.0))
    };
    let normal = |(c, _, caixa): Peca, x: [f32; 3]| {
        let d: [f32; 3] = std::array::from_fn(|e| x[e] - c[e]);
        if caixa {
            let e = (0..3)
                .max_by(|a, b| d[*a].abs().total_cmp(&d[*b].abs()))
                .unwrap_or(0);
            std::array::from_fn(|i| if i == e { d[e].signum() } else { 0.0 })
        } else {
            norm(d)
        }
    };
    // Visto de `o` (o centro ou o olho): a face virada para lá e nada no caminho.
    let visto = |ps: &[Peca], o: [f32; 3], x: [f32; 3], k: usize, fora: Option<usize>| {
        let a: [f32; 3] = std::array::from_fn(|e| x[e] - o[e]);
        let dist = d3(a, a).sqrt();
        let u = a.map(|y| y / dist);
        d3(normal(ps[k], x), u) < 0.0
            && primeiro(ps, o, u, fora).is_none_or(|(t, j)| j == k || t > dist - 1.0e-3)
    };
    let cena = |azul: [f32; 3]| -> Vec<Peca> {
        vec![
            ([0.0, 0.2, 1.35], 0.2, false),
            ([0.0, 0.18, 0.72], 0.18, true),
            (azul, 0.15, true),
            ([0.0, 0.3, 0.0], 0.3, false),
            ([0.0, 0.2, -0.7], 0.2, false),
            ([0.55, 0.17, -0.25], 0.17, false),
        ]
    };
    let olho = |yaw: f32| {
        let (p, d) = (0.52f32, 2.222f32);
        [
            d * p.cos() * yaw.sin(),
            d * p.sin(),
            d * p.cos() * yaw.cos(),
        ]
    };
    let mut casos: Vec<(String, Vec<Peca>, [f32; 3])> = vec![
        (
            "pose do dono, olho de perto".into(),
            cena([-0.5, 0.15, 0.6]),
            [-0.3533, 0.4209, 1.1953],
        ),
        (
            "foto 2 do dono (azul à frente da verde)".into(),
            cena([-0.55, 0.15, 0.75]),
            olho(4.6),
        ),
    ];
    for azul in [
        [-0.5, 0.15, 0.6],
        [-0.3, 0.15, 0.3],
        [-0.45, 0.15, 0.95],
        [0.3, 0.15, 0.3],
    ] {
        for yaw in [3.3f32, 3.8, 4.3, 4.8, 5.3] {
            casos.push((format!("azul {azul:?}, yaw {yaw}"), cena(azul), olho(yaw)));
        }
    }
    for (nome, ps, e) in &casos {
        let (c, r, _) = ps[0];
        let a: [f32; 3] = std::array::from_fn(|k| c[k] - e[k]);
        let dist = d3(a, a).sqrt();
        let f = a.map(|y| y / dist);
        let up = norm([0, 1, 2].map(|k| [0.0, 1.0, 0.0][k] - f[1] * f[k]));
        let rr = [
            f[1] * up[2] - f[2] * up[1],
            f[2] * up[0] - f[0] * up[2],
            f[0] * up[1] - f[1] * up[0],
        ];
        let k_ = 1.05 * r / (dist * dist - r * r).sqrt();
        let n = 300;
        let (mut espelho, mut acerta, mut esc, mut esc_olho) = (0, 0, 0, 0);
        for j in 0..n {
            for i in 0..n {
                let (x, y) = (
                    (i as f32 + 0.5) / n as f32 * 2.0 - 1.0,
                    (j as f32 + 0.5) / n as f32 * 2.0 - 1.0,
                );
                let d = norm(std::array::from_fn(|k| f[k] + k_ * (x * rr[k] + y * up[k])));
                let Some((t, 0)) = primeiro(ps, *e, d, None) else {
                    continue;
                };
                espelho += 1;
                let p: [f32; 3] = std::array::from_fn(|k| e[k] + t * d[k]);
                let nn = norm(std::array::from_fn(|k| p[k] - c[k]));
                let dn = d3(d, nn);
                let rf: [f32; 3] = std::array::from_fn(|k| d[k] - 2.0 * dn * nn[k]);
                let Some((s, k)) = primeiro(ps, p, rf, Some(0)) else {
                    continue;
                };
                acerta += 1;
                let xx: [f32; 3] = std::array::from_fn(|q| p[q] + s * rf[q]);
                if !visto(ps, c, xx, k, Some(0)) {
                    esc += 1;
                    esc_olho += usize::from(visto(ps, *e, xx, k, None));
                }
            }
        }
        println!(
            "SONDA {nome}: espelho {espelho} px · acerta vizinha {acerta} · escondido do centro {esc} · \
             desses, o olho vê {esc_olho}"
        );
    }
}

/// ⏱ **Sonda: a pose da 2.ª foto do dono contra o Cycles** — a foto `nossa | Cycles | erro` de todo o
/// espelho (`PH2D_REFLEXO_FOTOS`), e o `|Δ|` onde o raio acerta uma vizinha e onde não acerta nada.
#[test]
#[ignore = "sonda: precisa de aparelho"]
fn sonda_da_foto_do_dono() {
    let v = &FOTO2;
    let Some(mut fw) = desenhista(v) else {
        return;
    };
    let px = oraculo(v);
    let (vazio, visto) = (zona(v, &px, true), zona(v, &px, false));
    let nada: Vec<usize> = (0..px.len())
        .filter(|k| vizinha_refletida(v, &px[*k]).is_none())
        .collect();
    for (rug, cols) in [(0.0f32, (0usize, 2usize)), (v.rug2, (1, 3))] {
        let viz = desenha(v, &mut fw, metal(rug), true);
        let solo = desenha(v, &mut fw, metal(rug), false);
        let (a, b, c) = (
            mede(&px, &vazio, (&viz, &solo), cols),
            mede(&px, &visto, (&viz, &solo), cols),
            mede(&px, &nada, (&viz, &solo), cols),
        );
        foto(&px, &visto, (&viz, &solo), cols, &format!("foto2_{rug}"));
        println!(
            "SONDA foto 2, espelho {rug}: escondido do centro {} px {:.4} / {} · visto {} px {:.4} / {} · \
             sem vizinha {} px {:.4} / {}",
            a.0, a.1, a.2, b.0, b.1, b.2, c.0, c.1, c.2
        );
    }
}

/// A BASE (índices em `px`): o raio reflectido acerta uma vizinha a menos de [`BASE`] do chão num ponto que o
/// centro do espelho vê, a mais de `2 px` da silhueta dele — a faixa fina da base de uma peça pousada.
pub(crate) fn base(v: &Vista, px: &[Px]) -> Vec<usize> {
    let l = LADO as i32;
    let mut no_espelho = vec![false; (l * l) as usize];
    for p in px {
        no_espelho[(p.j * LADO + p.i) as usize] = true;
    }
    let dentro = |i: i32, j: i32| {
        (-2..=2).all(|dy| {
            (-2..=2).all(|dx| {
                let (x, y) = (i + dx, j + dy);
                (0..l).contains(&x) && (0..l).contains(&y) && no_espelho[(y * l + x) as usize]
            })
        })
    };
    px.iter()
        .enumerate()
        .filter(|(_, p)| {
            let Some(k) = vizinha_refletida(v, p) else {
                return false;
            };
            let f = v.vista_em(p.p);
            let fn_ = f[0] * p.n[0] + f[1] * p.n[1] + f[2] * p.n[2];
            let r: [f32; 3] = std::array::from_fn(|e| f[e] - 2.0 * fn_ * p.n[e]);
            let t = crate::tests_sonda_cpu::acerta_em(v.pecas[k + 1], p.p, r).unwrap_or(0.0);
            p.p[1] + t * r[1] < BASE && visto_do_centro(v, p) && dentro(p.i as i32, p.j as i32)
        })
        .map(|(q, _)| q)
        .collect()
}

/// A altura da faixa da base, em mundo.
const BASE: f32 = 0.03;

/// ⭐⭐⭐ **A base de uma vizinha pousada não se salta** — o report 5 do dono (04/10): de perto, a borda de baixo
/// do reflexo da caixa azul em ESCADA e com dentes escuros. Medido: a base (o ponto acertado a `y < 0,011`), vista
/// de raspão do centro, cabia entre dois passos da busca, e por trás dela aparecia a sombra do chão junto à caixa.
/// Nítido e `0,05`, na vista do dono (`VAZIO`) e na de perto da cena 42 (`PERTO`).
#[test]
#[ignore = "precisa de aparelho"]
fn a_base_de_uma_vizinha_pousada_nao_se_salta() {
    let mut falhas = Vec::new();
    for (v, nome, barra) in [
        (&VAZIO, "vazio", BARRA_BASE),
        (
            &crate::tests_reflexo_perto::PERTO,
            "perto",
            BARRA_BASE_PERTO,
        ),
    ] {
        let Some(mut fw) = desenhista(v) else {
            eprintln!("sem aparelho — o gate não corre aqui");
            return;
        };
        let px = oraculo(v);
        let b = base(v, &px);
        for (rug, cols) in [(0.0f32, (0usize, 2usize)), (v.rug2, (1, 3))] {
            let viz = desenha(v, &mut fw, metal(rug), true);
            let solo = desenha(v, &mut fw, metal(rug), false);
            let m = mede_em(&px, &b, (&viz, &solo), cols, 0.1);
            foto(&px, &b, (&viz, &solo), cols, &format!("base_{nome}_{rug}"));
            eprintln!(
                "base {nome}, espelho {rug}: {} px · |Δ| médio {:.4} · |Δ| > 0,1: {}",
                m.0, m.1, m.2
            );
            assert!(m.0 > 300, "a base encolheu: {} px", m.0);
            if !(m.1 < barra.0 && m.2 <= barra.1) {
                falhas.push(format!("{nome} {rug}"));
            }
        }
    }
    assert!(
        falhas.is_empty(),
        "a base de uma vizinha pousada saltou-se: {falhas:?}"
    );
}

/// `(px, |Δ| médio, px com |Δ| > grosso)` da razão `viz/solo` contra a do Cycles nos px `quais`.
fn mede_em(
    px: &[Px],
    quais: &[usize],
    (viz, solo): (&[u8], &[u8]),
    (cv, cs): (usize, usize),
    grosso: f32,
) -> (usize, f32, usize) {
    let lin = crate::tests_contacto::linear;
    let (mut n, mut s, mut g) = (0usize, 0.0f32, 0usize);
    for &k in quais {
        let p = &px[k];
        let q = (p.j * LADO + p.i) as usize;
        let (lv, ls) = (lin(viz[q * 4 + 1]), lin(solo[q * 4 + 1]));
        if ls < 0.05 || p.col[cs] < 0.05 {
            continue;
        }
        let e = (lv / ls - p.col[cv] / p.col[cs]).abs();
        n += 1;
        s += e;
        g += usize::from(e > grosso);
    }
    (n, s / n.max(1) as f32, g)
}

/// `(|Δ| médio na base, px com |Δ| > 0,1)` na vista do dono. Medido (04/10): a busca a saltar a base
/// `0,0266 / 363` e `0,0244 / 290` (nítido, `0,05`); com o chão a acabar o arco `0,0158 / 110` e `0,0191 / 125`
/// — o resto é a LUZ da face da azul virada para a verde (o Cycles escurece-a mais), sem salto nenhum.
const BARRA_BASE: (f32, usize) = (0.022, 160);

/// A mesma na vista de perto da cena 42: antes `0,0279 / 38` e `0,0227 / 31`; depois `0,0186 / 18` e `0,0157 / 11`.
const BARRA_BASE_PERTO: (f32, usize) = (0.022, 25);
