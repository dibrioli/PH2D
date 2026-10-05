//! A10 — a ponta do traço na ponta do VINCO. ⚠️ **A régua é a CONVERGÊNCIA:** a mesma lei com a
//! malha fina cada vez mais fina; a referência é o lado `32` (o desvio da recta cai `4×` por dobra,
//! a `32` fica `~10⁻³` do de hoje). Mede-se onde acaba cada trecho do traço contra a referência.

use super::super::super::frente::{DIV_FIXO, TOL_FIXA};
use super::{LARGURA, desenho_aqui, polilinhas};
use crate::skin_desenho::SkinDesenhado;

/// O desenho numa thread NOVA (o memo do quadro é por thread) com o lado `div` (`None` = a lei).
fn desenho(graus: f32, div: Option<usize>) -> SkinDesenhado {
    desenho_tol(graus, div, None)
}

/// O [`desenho`] com a tolerância da malha fina forçada (`None` = a da lei).
fn desenho_tol(graus: f32, div: Option<usize>, tol: Option<f64>) -> SkinDesenhado {
    std::thread::scope(|s| {
        s.spawn(|| {
            DIV_FIXO.with(|c| c.set(div));
            TOL_FIXA.with(|c| c.set(tol));
            desenho_aqui(graus)
        })
        .join()
        .expect("thread do desenho")
    })
}

/// As pontas dos trechos ABERTOS da camada do traço (um fechado inteiro não tem ponta).
fn pontas(d: &SkinDesenhado) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    for x in d.values() {
        let Some(t) = &x.traco else { continue };
        let mut abertos = t.clone();
        abertos.subpaths.retain(|c| !c.closed);
        if abertos.closed {
            abertos.verts.clear();
        }
        for l in polilinhas(&abertos, false) {
            if l.len() > 2 {
                out.extend([l[0], l[l.len() - 1]]);
            }
        }
    }
    out
}

/// Para cada ponta de `a`, a distância à ponta mais perto de `b`, em larguras (ordenadas).
fn desvios(a: &[[f64; 2]], b: &[[f64; 2]]) -> Vec<f64> {
    let mut v: Vec<f64> = a
        .iter()
        .map(|p| {
            b.iter()
                .map(|q| (p[0] - q[0]).hypot(p[1] - q[1]))
                .fold(f64::MAX, f64::min)
                / LARGURA
        })
        .collect();
    v.sort_by(f64::total_cmp);
    v
}

/// ⭐ **SONDA — a ponta do vinco converge?** Por pose, o lado fixo `1…16` e a lei contra o `32`:
/// as pontas, o maior desvio em larguras e o tempo do desenho.
#[test]
#[ignore = "sonda: imprime"]
fn diag_a_ponta_do_vinco_converge() {
    for graus in [110f32, 130.0, 150.0, 160.0, 170.0, 175.0] {
        let refe = pontas(&desenho(graus, Some(32)));
        println!("  {graus}°: {} pontas na referência", refe.len());
        for (div, tol) in [
            (Some(1), None),
            (Some(2), None),
            (Some(4), None),
            (Some(8), None),
            (Some(16), None),
            (None, Some(0.05)),
            (None, Some(0.1)),
            (None, Some(0.2)),
            (None, Some(0.4)),
        ] {
            let t0 = std::time::Instant::now();
            let d = desenho_tol(graus, div, tol);
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            let p = pontas(&d);
            let (ida, volta) = (desvios(&p, &refe), desvios(&refe, &p));
            let max = ida.iter().chain(&volta).copied().fold(0.0, f64::max);
            println!(
                "    lado {div:?} tol {tol:?}: {} pontas · maior desvio {max:.3} larg. · {ms:.1} ms · {:.2?}",
                p.len(),
                ida
            );
        }
    }
}

/// ⭐ **SONDA — o preço da malha fina**, µs por forma por quadro (duas poses alternadas, o memo
/// nunca acerta), o lado `1` (a malha do campo) contra a lei.
#[test]
#[ignore = "sonda: imprime"]
fn diag_o_preco_da_malha_fina() {
    use crate::skin_desenho::Leis;
    let (mut sim, mut scene, map, id, [_, ponta]) = crate::skin_live::tests::palco();
    {
        let p = scene.path_mut(id).expect("path");
        p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
            LARGURA,
        ));
        p.effects = vec![ph2d_vec_scene::effect::FxEntry::new(
            ph2d_vec_scene::effect::PathEffect::Repeat(ph2d_vec_scene::fx_repeat::RepeatSpec {
                copies_x: 1.0,
                move_x: 0.0,
                copies_y: 2.0,
                move_y: 60.0,
                spin: 5.0,
                orbit: 0.0,
            }),
        )];
    }
    assert_eq!(
        crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None),
        1
    );
    let mut n = 0_u32;
    for (pa, pb) in [(110f32, 150f32), (160.0, 175.0), (0.0, 30.0)] {
        let mut quadro = |div: Option<usize>| {
            DIV_FIXO.with(|c| c.set(div));
            let mut sc = scene.clone();
            let t = std::time::Instant::now();
            let mut k = 0_u32;
            while t.elapsed().as_millis() < 400 {
                n += 1;
                sim.world_mut()
                    .get_mut::<ph2d_ecs::Transform>(ponta)
                    .expect("Transform")
                    .rotation = if n.is_multiple_of(2) { pa } else { pb }.to_radians();
                let _ = crate::skin_live::recook_leis(&sim, &mut sc, Leis::do_ambiente());
                k += 1;
            }
            t.elapsed().as_secs_f64() * 1e6 / f64::from(k)
        };
        let velha = quadro(Some(1));
        let mut txt = format!("malha do campo {velha:.1}");
        for tol in [0.05, 0.1, 0.2, 0.4] {
            TOL_FIXA.with(|c| c.set(Some(tol)));
            let us = quadro(None);
            let (pais, verts, subs) = super::super::super::frente::ULTIMA_FINA.with(std::cell::Cell::get);
            txt.push_str(&format!(" · tol {tol} → {us:.1} ({pais} tri → {verts} v / {subs} sub)"));
        }
        TOL_FIXA.with(|c| c.set(None));
        println!(
            "  {pa}°↔{pb}°: µs/forma/quadro: {txt} · loadavg {}",
            std::fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .trim()
        );
    }
}

/// O `d` de SVG dos contornos de `p` (y para baixo).
fn d_de(p: &ph2d_vec_scene::VecPath) -> String {
    use std::fmt::Write as _;
    let mut d = String::new();
    for l in polilinhas(p, false) {
        for (i, q) in l.iter().enumerate() {
            let _ = write!(d, "{}{:.5} {:.5} ", if i == 0 { 'M' } else { 'L' }, q[0], -q[1]);
        }
    }
    d
}

/// ⭐ **SONDA — a FOTO do vinco** a `160°` e `170°`: a malha do campo e a lei, inteira e ampliada
/// na ponta que mais muda. SVG em `target/prova/vinco`.
#[test]
#[ignore = "sonda: escreve SVG"]
fn diag_a_foto_do_vinco() {
    let saida = "target/prova/vinco";
    std::fs::create_dir_all(saida).expect("pasta");
    for graus in [160f32, 170.0] {
        let (velho, novo) = (desenho(graus, Some(1)), desenho(graus, None));
        let (pv, pn) = (pontas(&velho), pontas(&novo));
        let alvo = pv
            .iter()
            .max_by(|a, b| {
                let d = |p: &&[f64; 2]| {
                    pn.iter()
                        .map(|q| (p[0] - q[0]).hypot(p[1] - q[1]))
                        .fold(f64::MAX, f64::min)
                };
                d(a).total_cmp(&d(b))
            })
            .copied()
            .unwrap_or_default();
        for (nome, d) in [("campo", &velho), ("fina", &novo)] {
            let mut corpo = String::new();
            for x in d.values() {
                corpo.push_str(&format!(
                    "<path d='{}' fill='#bbbbbb' fill-rule='nonzero'/>",
                    d_de(&x.forma)
                ));
                if let Some(t) = &x.traco {
                    corpo.push_str(&format!(
                        "<path d='{}' fill='none' stroke='#8b0000' stroke-width='{LARGURA}' \
                         stroke-linecap='round' stroke-linejoin='round'/>",
                        d_de(t)
                    ));
                }
            }
            let r = 4.0 * LARGURA;
            for (sufixo, vb) in [
                ("inteira", "-10 -80 120 110".to_string()),
                (
                    "ampliada",
                    format!("{} {} {} {}", alvo[0] - r, -alvo[1] - r, 2.0 * r, 2.0 * r),
                ),
            ] {
                let f = format!("{saida}/vinco_{graus}_{nome}_{sufixo}.svg");
                std::fs::write(
                    &f,
                    format!(
                        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='{vb}' width='1200' \
                         height='1200'><rect x='-1000' y='-1000' width='3000' height='3000' \
                         fill='white'/>{corpo}</svg>"
                    ),
                )
                .expect("svg");
                println!("  {f}");
            }
        }
    }
}
