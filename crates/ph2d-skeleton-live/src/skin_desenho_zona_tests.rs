//! A ZONA DA DOBRA — onde a pele começa a dobrar sobre si mesma (report do dono de 2026-09-30, três
//! fotos com a junta de cima entre `84°` e `89°` e a de baixo a `~125°`).

use super::{Forma, com_e_sem_contacto_em, diagonal};
use ph2d_vec_scene::VecPath;

type P = [f64; 2];

fn cubica(p: &VecPath, i: usize) -> [P; 4] {
    let n = p.verts.len();
    let (c, q) = (&p.verts[i], &p.verts[(i + 1) % n]);
    [c.anchor, c.out_handle, q.in_handle, q.anchor]
}

fn d1(c: &[P; 4], t: f64) -> P {
    let u = 1.0 - t;
    let f = |k: usize| {
        3.0 * u * u * (c[1][k] - c[0][k])
            + 6.0 * u * t * (c[2][k] - c[1][k])
            + 3.0 * t * t * (c[3][k] - c[2][k])
    };
    [f(0), f(1)]
}

fn d2(c: &[P; 4], t: f64) -> P {
    let u = 1.0 - t;
    let f = |k: usize| {
        6.0 * u * (c[2][k] - 2.0 * c[1][k] + c[0][k])
            + 6.0 * t * (c[3][k] - 2.0 * c[2][k] + c[1][k])
    };
    [f(0), f(1)]
}

fn ponto(c: &[P; 4], t: f64) -> P {
    let u = 1.0 - t;
    let f = |k: usize| {
        u * u * u * c[0][k]
            + 3.0 * u * u * t * c[1][k]
            + 3.0 * u * t * t * c[2][k]
            + t * t * t * c[3][k]
    };
    [f(0), f(1)]
}

/// O menor raio de curvatura CÔNCAVA do contorno de fora, amostrado por segmento (e onde fica).
pub(super) fn raio_concavo_minimo(p: &VecPath, so: impl Fn(P) -> bool) -> (f64, P) {
    let n = p.verts.len();
    let sinal: f64 = (0..n)
        .map(|i| {
            let (a, b) = (p.verts[i].anchor, p.verts[(i + 1) % n].anchor);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
        .signum();
    let mut melhor = (f64::INFINITY, [0.0, 0.0]);
    for i in 0..n {
        let c = cubica(p, i);
        for k in 0..=64 {
            let t = f64::from(k) / 64.0;
            let (v, a) = (d1(&c, t), d2(&c, t));
            let cruz = v[0] * a[1] - v[1] * a[0];
            let nv = v[0].hypot(v[1]);
            if nv < 1e-12 {
                continue;
            }
            if cruz * sinal < 0.0 {
                let r = nv * nv * nv / cruz.abs();
                let q = ponto(&c, t);
                if r < melhor.0 && so(q) {
                    melhor = (r, q);
                }
            }
        }
    }
    melhor
}

/// 📏 SONDA — `PH2D_SONDA_ZONA=1`: varre a junta de cima de `70°` a `100°` com a de baixo a
/// `125°`, e imprime por ângulo: se cruza, as ilhas, a maior viragem côncava e o menor raio côncavo
/// (em raios do vinco).
#[test]
fn diag_a_zona_da_dobra() {
    if std::env::var("PH2D_SONDA_ZONA").is_err() {
        return;
    }
    let baixo: f32 = std::env::var("PH2D_SONDA_G")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(125.0);
    let mut g = 70.0_f32;
    while g <= 100.0 {
        let (sem, com) = com_e_sem_contacto_em(baixo, Forma::Dono(g));
        let r = ph2d_vec_boolean::overlap::RAIO_DO_VINCO * diagonal(&com);
        let ilhas: Vec<String> = com
            .subpaths
            .iter()
            .map(|c| {
                let v = VecPath {
                    verts: c.verts.clone(),
                    closed: true,
                    ..VecPath::default()
                };
                format!("{:.2e}", ph2d_vec_boolean::area(&v) / (r * r))
            })
            .collect();
        let vs = &com.verts;
        let (mut vmax, mut imax) = (0.0_f64, 0);
        for i in 0..vs.len() {
            let v = ph2d_vec_boolean::overlap::viragem_do_vertice(vs, i).unwrap_or(0.0);
            if v > vmax {
                (vmax, imax) = (v, i);
            }
        }
        // Só o canto de CIMA (a junta das fotos): a de baixo e o assado têm feições próprias.
        let cima = |q: P| q[1] > 3.5;
        let (rc, onde) = raio_concavo_minimo(&com, cima);
        let (rs, _) = raio_concavo_minimo(&sem, cima);
        println!(
            "g2 {g:5.1} cruza {} | verts {} | viragem max {vmax:6.1} em v{imax} {:?} | \
             raio côncavo min {:.3}r em ({:.3},{:.3}) (sem: {:.3}r) | ilhas {:?}",
            u8::from(com != sem),
            vs.len(),
            vs[imax].anchor.map(|x| (x * 1000.0).round() / 1000.0),
            rc / r,
            onde[0],
            onde[1],
            rs / r,
            ilhas
        );
        g += 1.0;
    }
}

/// A maior VIRAGEM de uma zona côncava mais apertada que `limite` (em graus), e onde: o contorno de
/// fora amostrado a `64` por segmento, as arestas que viram para DENTRO mais depressa que `limite`
/// agrupadas em corridas, e a viragem somada de cada uma. ⚠️ É a régua da bola medida por OUTRO
/// caminho (as tangentes da cúbica, não as da porta), e é por isso que o gate a usa.
pub(super) fn maior_zona_apertada(p: &VecPath, limite: f64) -> (f64, P) {
    let n = p.verts.len();
    let mut am: Vec<(P, P)> = Vec::new();
    for i in 0..n {
        let c = cubica(p, i);
        for k in 0..=64 {
            let t = f64::from(k) / 64.0;
            let v = d1(&c, t);
            let l = v[0].hypot(v[1]);
            if l > 1e-9 {
                am.push((ponto(&c, t), [v[0] / l, v[1] / l]));
            }
        }
    }
    let m = am.len();
    let sinal: f64 = (0..m)
        .map(|i| {
            let (a, b) = (am[i].0, am[(i + 1) % m].0);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
        .signum();
    let (mut pior, mut onde, mut soma, mut ini) = (0.0_f64, [0.0; 2], 0.0_f64, [0.0; 2]);
    for i in 0..=m {
        let (a, b) = (am[i % m], am[(i + 1) % m]);
        let dth = (a.1[0] * b.1[1] - a.1[1] * b.1[0]).atan2(a.1[0] * b.1[0] + a.1[1] * b.1[1]);
        let ds = (b.0[0] - a.0[0]).hypot(b.0[1] - a.0[1]);
        if dth * sinal < -1e-9 && ds < limite * dth.abs() {
            if soma == 0.0 {
                ini = a.0;
            }
            soma += dth.abs();
        } else {
            if soma > pior {
                (pior, onde) = (soma, ini);
            }
            soma = 0.0;
        }
    }
    (pior.to_degrees(), onde)
}

/// Os pontos do contorno de fora, `16` por segmento.
pub(super) fn pontos(p: &VecPath) -> Vec<P> {
    (0..p.verts.len())
        .flat_map(|i| {
            let c = cubica(p, i);
            (0..16).map(move |k| ponto(&c, f64::from(k) / 16.0))
        })
        .collect()
}

/// A distância de Hausdorff entre dois contornos amostrados.
pub(super) fn hausdorff(a: &[P], b: &[P]) -> f64 {
    let lado = |x: &[P], y: &[P]| {
        x.iter()
            .map(|p| {
                y.iter()
                    .map(|q| (p[0] - q[0]).hypot(p[1] - q[1]))
                    .fold(f64::INFINITY, f64::min)
            })
            .fold(0.0, f64::max)
    };
    lado(a, b).max(lado(b, a))
}

/// 📏 SONDA — `PH2D_SONDA_PASSO=1`: de grau em grau, quanto a SILHUETA anda contra quanto o
/// DESENHO anda (Hausdorff), na pose das fotos.
#[test]
fn diag_o_passo_da_silhueta() {
    if std::env::var("PH2D_SONDA_PASSO").is_err() {
        return;
    }
    let mut ant: Option<(Vec<P>, Vec<P>)> = None;
    let mut g = 60.0_f32;
    while g <= 150.0 {
        let (sem, com) = com_e_sem_contacto_em(125.0, Forma::Dono(g));
        let (ps, pc) = (pontos(&sem), pontos(&com));
        if let Some((a_s, a_c)) = &ant {
            let (hs, hc) = (hausdorff(a_s, &ps), hausdorff(a_c, &pc));
            println!(
                "g2 {g:5.1}  desenho {hs:.4}  silhueta {hc:.4}  razão {:.2}",
                hc / hs
            );
        }
        ant = Some((ps, pc));
        g += 1.0;
    }
}

/// ⭐⭐⭐ **GATE — nenhum canto da silhueta é mais apertado que a bola, ANTES e DEPOIS do
/// contacto** (report do dono de 2026-09-30, três fotos com a junta de cima a `84°`–`89°`: *«ainda
/// inconsistente… não é progressivo… artefatos circulares»*).
///
/// Nas quatro formas de dobra, de `60°` a `150°` (a pose das fotos de grau em grau): **(1)** a
/// silhueta é um PONTO FIXO da porta — rolar a bola outra vez não muda nada; **(2)** o menor raio
/// côncavo do contorno é pelo menos `0,9 r`, ou é o do desenho (uma feição que a bola não tocou,
/// porque a viragem dela cabe na solda). ⚠️ A F40 falhava a (2) de `90°` a `92°` (`0,053`/`0,033`/
/// `0,017 r` contra `0,022`/`0,009`/`0,002 r` do desenho): o arredondamento só corria no
/// cruzamento, e ali a pele aperta até ao bico SEM se cruzar. Piso de população: a bola TEM de ter
/// mexido em alguma dobra da faixa sem cruzamento, senão o gate varreria silhuetas intocadas.
#[test]
fn nenhum_canto_da_silhueta_e_mais_apertado_que_a_bola() {
    let mut casos: Vec<(Forma, f32)> = Vec::new();
    for forma in [Forma::C, Forma::Z, Forma::Uma] {
        let mut g = 60.0_f32;
        while g <= 150.0 {
            casos.push((forma, g));
            g += 5.0;
        }
    }
    let mut g = 60.0_f32;
    while g <= 150.0 {
        casos.push((Forma::Dono(g), 125.0));
        g += 1.0;
    }
    let mut antes_do_cruzamento = 0;
    for (forma, g) in casos {
        let (sem, com) = com_e_sem_contacto_em(g, forma);
        let r = ph2d_vec_boolean::overlap::RAIO_DO_VINCO * diagonal(&sem);
        assert!(
            ph2d_vec_boolean::silhueta_da_pele(&com).is_none(),
            "{forma:?} {g}°: rolar a bola outra vez mudou a silhueta — ela não é um ponto fixo"
        );
        let (vira, onde) = maior_zona_apertada(&com, 0.9 * r);
        assert!(
            vira < 12.0,
            "{forma:?} {g}°: uma zona côncava mais apertada que a bola vira {vira:.1}° em {onde:?} \
             — um canto ficou"
        );
        if com != sem
            && ph2d_vec_boolean::resolve_overlap(&sem).is_none()
            && maior_zona_apertada(&sem, 0.9 * r).0 >= 12.0
        {
            antes_do_cruzamento += 1;
        }
    }
    assert!(
        antes_do_cruzamento >= 1,
        "a bola não mexeu em nenhuma dobra antes do cruzamento — o gate não viu o fenómeno"
    );
}
