//! A9 — a ponta de um corte do traço acerta no cruzamento desenhado.
//!
//! ⚠️ **A régua não usa a malha nem a chave de osso:** só a geometria DESENHADA — a ponta de um
//! trecho do traço que mora sobre o contorno fecha-se a que distância do cruzamento mais perto dos
//! contornos FECHADOS desenhados (a borda onde o traço de trás passa por baixo da frente)? Entre
//! `0,05` e `1` largura é um tique; mais longe é outra coisa (a ponta do vinco, aberto A10).

use super::super::SkinDesenhado;
use crate::skin_desenho::Leis;
use crate::skin_live::tests::palco;
use ph2d_vec_scene::VecPath;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};

const LARGURA: f64 = 0.5;

/// O [`desenho_aqui`] numa thread NOVA, com ou sem o encaixe — ⚠️ o memo do quadro é por thread e
/// guarda o desenho por forma: na mesma thread o 2.º desenho era o do 1.º (o gate lia a lei
/// desligada dos dois lados).
fn desenho(graus: f32, sem_encaixe: bool) -> SkinDesenhado {
    std::thread::scope(|s| {
        s.spawn(|| {
            super::SEM_ENCAIXE.with(|c| c.set(sem_encaixe));
            desenho_aqui(graus)
        })
        .join()
        .expect("thread do desenho")
    })
}

/// Duas cópias da barra `40 × 10` (a 2.ª girada `5°`), traço `0,5`, presas e a ponta a `graus`.
pub(super) fn desenho_aqui(graus: f32) -> SkinDesenhado {
    let (mut sim, mut scene, map, id, [_, ponta]) = palco();
    {
        let p = scene.path_mut(id).expect("path");
        p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
            LARGURA,
        ));
        p.effects = vec![FxEntry::new(PathEffect::Repeat(
            ph2d_vec_scene::fx_repeat::RepeatSpec {
                copies_x: 1.0,
                move_x: 0.0,
                copies_y: 2.0,
                move_y: 60.0,
                spin: 5.0,
                orbit: 0.0,
            },
        ))];
    }
    assert_eq!(
        crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None),
        1
    );
    sim.world_mut()
        .get_mut::<ph2d_ecs::Transform>(ponta)
        .expect("Transform")
        .rotation = graus.to_radians();
    crate::skin_live::recook_leis(&sim, &mut scene.clone(), Leis::do_ambiente())
}

pub(super) fn polilinhas(p: &VecPath, so_fechados: bool) -> Vec<Vec<[f64; 2]>> {
    (0..p.contour_count())
        .filter_map(|c| p.contour(c))
        .filter(|(v, f)| v.len() > 1 && (*f || !so_fechados))
        .map(|(v, fechado)| {
            let m = if fechado { v.len() } else { v.len() - 1 };
            let mut out: Vec<[f64; 2]> = (0..m)
                .flat_map(|k| {
                    let (a, b) = (v[k], v[(k + 1) % v.len()]);
                    let c = [a.anchor, a.out_handle, b.in_handle, b.anchor];
                    (0..64)
                        .map(move |i| super::super::super::frente::avalia(&c, f64::from(i) / 64.0))
                })
                .collect();
            out.push(if fechado {
                v[0].anchor
            } else {
                v[v.len() - 1].anchor
            });
            out
        })
        .collect()
}

/// Os cruzamentos entre troços não vizinhos: `(contorno, troço, fracção no troço)`, os dois lados.
fn cruzamentos(pl: &[Vec<[f64; 2]>]) -> Vec<(usize, usize, f64)> {
    let t: Vec<(usize, usize, [f64; 2], [f64; 2])> = pl
        .iter()
        .enumerate()
        .flat_map(|(c, l)| {
            l.windows(2)
                .enumerate()
                .map(move |(i, w)| (c, i, w[0], w[1]))
        })
        .collect();
    let mut out = Vec::new();
    for (x, &(ca, ia, a0, a1)) in t.iter().enumerate() {
        for &(cb, ib, b0, b1) in &t[x + 1..] {
            let n = pl[ca].len() - 1;
            if ca == cb && (ia.abs_diff(ib) <= 1 || ia.abs_diff(ib) >= n - 1) {
                continue;
            }
            let (r, s) = (
                [a1[0] - a0[0], a1[1] - a0[1]],
                [b1[0] - b0[0], b1[1] - b0[1]],
            );
            let den = r[0] * s[1] - r[1] * s[0];
            if den.abs() < 1e-18 {
                continue;
            }
            let q = [b0[0] - a0[0], b0[1] - a0[1]];
            let (u, v) = (
                (q[0] * s[1] - q[1] * s[0]) / den,
                (q[0] * r[1] - q[1] * r[0]) / den,
            );
            if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) {
                out.push((ca, ia, u));
                out.push((cb, ib, v));
            }
        }
    }
    out
}

/// A distância AO LONGO do contorno fechado (em larguras) de cada ponta de trecho do traço sobre ele
/// ao cruzamento mais perto, até `3` larguras (além disso, `3`). ⚠️ Ao longo e não em linha recta:
/// numa dobra em grampo o cruzamento do OUTRO braço fica perto pelo ar (lia `0,14` uma ponta de vinco).
fn pontas(d: &SkinDesenhado) -> Vec<f64> {
    let mut out = Vec::new();
    for x in d.values() {
        let Some(t) = &x.traco else { continue };
        let contorno = polilinhas(&x.forma, true);
        let cruz = cruzamentos(&contorno);
        let perto = |p: [f64; 2]| -> (usize, usize, f64, f64) {
            let mut m = (0, 0, 0.0, f64::MAX);
            for (c, l) in contorno.iter().enumerate() {
                for (i, s) in l.windows(2).enumerate() {
                    let ab = [s[1][0] - s[0][0], s[1][1] - s[0][1]];
                    let l2 = (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-18);
                    let f = (((p[0] - s[0][0]) * ab[0] + (p[1] - s[0][1]) * ab[1]) / l2)
                        .clamp(0.0, 1.0);
                    let dd = (p[0] - s[0][0] - f * ab[0]).hypot(p[1] - s[0][1] - f * ab[1]);
                    if dd < m.3 {
                        m = (c, i, f, dd);
                    }
                }
            }
            m
        };
        // Só os trechos ABERTOS (um fechado inteiro à vista não tem ponta: a «ponta» é a emenda).
        let mut abertos = t.clone();
        abertos.subpaths.retain(|c| !c.closed);
        if abertos.closed {
            abertos.verts.clear();
        }
        for l in polilinhas(&abertos, false) {
            if l.len() < 3 || perto(l[l.len() / 2]).3 > 1e-4 {
                continue;
            }
            for p in [l[0], l[l.len() - 1]] {
                let (c, i, f, _) = perto(p);
                let pl = &contorno[c];
                let n = pl.len() - 1;
                let comp = |j: usize| (pl[j + 1][0] - pl[j][0]).hypot(pl[j + 1][1] - pl[j][1]);
                let mut melhor = 3.0 * LARGURA;
                for &(cc, j, g) in cruz.iter().filter(|x| x.0 == c) {
                    let _ = cc;
                    // O arco de (i, f) a (j, g) pelos dois sentidos da volta.
                    let (mut ida, mut k, mut a) = (0.0, i, f);
                    for _ in 0..n {
                        if k == j && g >= a {
                            ida += (g - a) * comp(k);
                            break;
                        }
                        ida += (1.0 - a) * comp(k);
                        (k, a) = ((k + 1) % n, 0.0);
                    }
                    let (mut volta, mut k, mut a) = (0.0, i, f);
                    for _ in 0..n {
                        if k == j && g <= a {
                            volta += (a - g) * comp(k);
                            break;
                        }
                        volta += a * comp(k);
                        (k, a) = ((k + n - 1) % n, 1.0);
                    }
                    melhor = melhor.min(ida.min(volta));
                }
                out.push(melhor / LARGURA);
            }
        }
    }
    out
}

/// ⭐⭐⭐ **GATE — nenhuma ponta de corte fica a um TIQUE do cruzamento** (`0,05`–`1` largura), de
/// `110°` a `170°`. ⛔ **O CONTROLO:** sem o encaixe há tiques (MEDIDO na `=6`: `0,13`–`0,98`).
///
/// ⚠️ De `10` em `10` graus desde a lei do meio-ângulo (2026-10-06): com ela os tiques sem o
/// encaixe caem a `120°`, `136°`–`142°`, `152°` e `156°`–`164°` (`1`/`1`/`5` a `120`/`140`/`160`), e
/// a régua de `20` em `20` (`110`/`130`/`150`/`170`, `0`/`2`/`2`/`0` no círculo) passava ENTRE eles.
#[test]
fn nenhuma_ponta_de_corte_fica_a_um_tique_do_cruzamento() {
    let tiques = |v: &[f64]| v.iter().filter(|d| **d > 0.05 && **d < 1.0).count();
    let (mut com, mut sem, mut no_cruzamento) = (0, 0, 0);
    for graus in [110f32, 120.0, 130.0, 140.0, 150.0, 160.0, 170.0] {
        let (antes, depois) = (
            pontas(&desenho(graus, true)),
            pontas(&desenho(graus, false)),
        );
        println!(
            "  {graus}°: tiques sem o encaixe {} · com {} · pontas {} {depois:.2?}",
            tiques(&antes),
            tiques(&depois),
            depois.len()
        );
        // ⭐ E o que fica a MAIS de uma largura de um cruzamento (a ponta do vinco) não se mexe.
        let longe = |v: &[f64]| {
            let mut l: Vec<String> = v
                .iter()
                .filter(|d| **d >= 1.0)
                .map(|d| format!("{d:.3}"))
                .collect();
            l.sort();
            l
        };
        assert_eq!(
            longe(&antes),
            longe(&depois),
            "a {graus}° o encaixe mexeu numa ponta longe de um cruzamento"
        );
        sem += tiques(&antes);
        com += tiques(&depois);
        no_cruzamento += depois.iter().filter(|d| **d <= 0.05).count();
    }
    assert!(sem > 0, "o CONTROLO: sem o encaixe não há tique nenhum");
    assert!(
        no_cruzamento > 0,
        "o CONTROLO: nenhuma ponta cai num cruzamento"
    );
    assert_eq!(
        com, 0,
        "{com} pontas de corte ficaram a um tique do cruzamento"
    );
}

/// SONDA — a foto SVG desta fixtura a `SONDA_G` graus (`target/prova/cruza.svg`): o contorno a azul
/// fino, a camada do traço a vermelho.
#[test]
#[ignore = "sonda: escreve SVG"]
fn diag_a_foto_da_fixtura() {
    use std::fmt::Write as _;
    let g: f32 = std::env::var("SONDA_G")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(130.0);
    let d = desenho(g, std::env::var("SONDA_SEM").is_ok());
    let mut s = String::from(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='-30 -40 90 70' width='2400' height='1867'><rect x='-30' y='-40' width='90' height='70' fill='white'/>",
    );
    for x in d.values() {
        for (p, cor, w) in [
            (Some(&x.forma), "#1e66ff", 0.08),
            (x.traco.as_ref(), "#8b0000", LARGURA),
        ] {
            let Some(p) = p else { continue };
            for l in polilinhas(p, false) {
                let pts: Vec<String> = l
                    .iter()
                    .map(|q| format!("{:.4},{:.4}", q[0], -q[1]))
                    .collect();
                let _ = write!(
                    s,
                    "<polyline points='{}' fill='none' stroke='{cor}' stroke-width='{w}' stroke-opacity='0.8'/>",
                    pts.join(" ")
                );
            }
        }
    }
    s.push_str("</svg>");
    std::fs::create_dir_all("target/prova").expect("pasta");
    std::fs::write("target/prova/cruza.svg", s).expect("svg");
}

/// O `x` do ponto da base da barra (o segmento `0`, nós de canto) no parâmetro `u`, e o inverso.
fn x_da_base(u: f64) -> f64 {
    let c = [[0.0, 0.0], [0.0, 0.0], [10.0, 0.0], [10.0, 0.0]];
    super::super::super::frente::avalia(&c, u)[0]
}

fn u_da_base(x: f64) -> f64 {
    let (mut a, mut b) = (0.0, 1.0);
    for _ in 0..60 {
        let m = 0.5 * (a + b);
        if x_da_base(m) < x { a = m } else { b = m }
    }
    0.5 * (a + b)
}

/// Um polígono recto (alças sobre as âncoras) como contorno.
fn poligono(pts: &[[f64; 2]], closed: bool) -> ph2d_vec_scene::Contour {
    ph2d_vec_scene::Contour {
        verts: pts
            .iter()
            .map(|&p| ph2d_vec_scene::VecVertex::corner(p))
            .collect(),
        closed,
    }
}

/// A barra `[0, 10] × [0, 1]` e duas tiras verticais que lhe cruzam a base em `x = 5,2 / 5,3` e
/// `5,6 / 5,7`; `barra_fechada` diz como o ASSADO marca a barra (a fonte marca-a sempre fechada).
fn barra_e_tiras(barra_fechada: bool) -> (VecPath, VecPath) {
    let tira = |x: f64| {
        poligono(
            &[[x, -1.0], [x + 0.1, -1.0], [x + 0.1, 2.0], [x, 2.0]],
            true,
        )
    };
    let barra = [[0.0, 0.0], [10.0, 0.0], [10.0, 1.0], [0.0, 1.0]];
    let de = |fechada: bool| {
        let b = poligono(&barra, fechada);
        VecPath {
            verts: b.verts,
            closed: fechada,
            subpaths: vec![tira(5.2), tira(5.6)],
            ..VecPath::default()
        }
    };
    (de(barra_fechada), de(true))
}

/// ⭐⭐ **GATE — a ponta vai ao cruzamento MAIS PERTO ao longo do contorno**, não a outro da janela
/// (M11 sobrevivia: as fixturas tinham um cruzamento por janela). A ponta em `x = 5` tem `4`
/// cruzamentos a menos de uma largura `1`; vai para `x = 5,2`.
#[test]
fn a_ponta_vai_ao_cruzamento_mais_perto() {
    let (d, fonte) = barra_e_tiras(true);
    let b = super::Bordas::de(&d, &fonte, 1.0);
    let x = x_da_base(b.encaixa(0, u_da_base(5.0), 1.0));
    assert!(
        (x - 5.2).abs() < 5e-3,
        "a ponta foi a x = {x} (o cruzamento mais perto é 5,2)"
    );
    let parada = x_da_base(b.encaixa(0, u_da_base(5.0), 0.1));
    assert!(
        (parada - 5.0).abs() < 1e-9,
        "o CONTROLO: fora do alcance não se mexe ({parada})"
    );
}

/// ⭐⭐ **GATE — quem diz que um contorno é fechado é a FONTE** (M13 sobrevivia): com o assado a marcar
/// a barra aberta, as bordas continuam a ver a volta e a ponta encaixa igual.
#[test]
fn o_fecho_das_bordas_e_o_da_fonte() {
    let (d, fonte) = barra_e_tiras(false);
    assert!(
        !d.contour(0).expect("barra").1,
        "o CONTROLO: o assado marca a barra aberta"
    );
    let x = x_da_base(super::Bordas::de(&d, &fonte, 1.0).encaixa(0, u_da_base(5.0), 1.0));
    assert!(
        (x - 5.2).abs() < 5e-3,
        "com o assado aberto a ponta ficou em x = {x}"
    );
}

/// ⭐⭐ **GATE — um trecho que o encaixe inverte SAI** (era todo tique; M14 sobrevivia): as pontas em
/// `x = 5,29` e `5,31` vão as duas para `5,3`. ⛔ O CONTROLO: um trecho que só encolhe fica.
#[test]
fn um_trecho_que_o_encaixe_inverte_sai() {
    let (d, fonte) = barra_e_tiras(true);
    let b = super::Bordas::de(&d, &fonte, 1.0);
    assert_eq!(
        b.encaixa_trecho(0, (u_da_base(5.29), u_da_base(5.31)), 4.0, 1.0),
        None
    );
    let fica = b
        .encaixa_trecho(0, (u_da_base(5.1), u_da_base(5.35)), 4.0, 1.0)
        .expect("o CONTROLO: o trecho fica");
    let (x0, x1) = (x_da_base(fica.0), x_da_base(fica.1));
    assert!(
        (x0 - 5.2).abs() < 5e-3 && (x1 - 5.3).abs() < 5e-3,
        "{x0} {x1}"
    );
}

#[path = "skin_desenho_camadas_vinco_tests.rs"]
mod vinco;
