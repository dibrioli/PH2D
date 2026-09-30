//! ⭐⭐⭐ **SONDA — o CONTACTO no cotovelo** (2026-09-29): onde o contorno da pele se cruza, o que
//! a borda da malha faz, e a SILHUETA que acabou por ser a cura (a porta de produto é a
//! [`ph2d_vec_boolean::resolve_overlap`], ligada em `skin_desenho::calcula`).
//!
//! ⚠️ Irmã da `arap_sonda_tests`, de onde vem o palco (a malha do domínio com o alvo da lei) e as
//! duas curas RECUSADAS — o corte é o tecto de LOC, e a fronteira é a pergunta: lá *«uma correcção
//! na malha cura?»*, aqui *«o que é que o artista vê, e o que o cura?»*.

use super::arap_sonda_tests::*;
use super::ouro_reguas_tests::*;

/// Os pares de segmentos que se cruzam, com o índice de amostra de cada um.
fn cruzes(poli: &[[f64; 2]]) -> Vec<(usize, usize)> {
    let n = poli.len();
    let sinal = |o: [f64; 2], u: [f64; 2], v: [f64; 2]| {
        ((u[0] - o[0]) * (v[1] - o[1]) - (v[0] - o[0]) * (u[1] - o[1])).signum()
    };
    let mut out = Vec::new();
    for i in 0..n {
        let (a1, a2) = (poli[i], poli[(i + 1) % n]);
        for j in (i + 2)..n {
            if i == 0 && j == n - 1 {
                continue;
            }
            let (b1, b2) = (poli[j], poli[(j + 1) % n]);
            if sinal(a1, a2, b1) != sinal(a1, a2, b2) && sinal(b1, b2, a1) != sinal(b1, b2, a2) {
                out.push((i, j));
            }
        }
    }
    out
}

#[test]
#[ignore = "sonda: ONDE o contorno se cruza, e se o sítio está dentro da malha"]
fn diag_onde_o_contorno_cruza() {
    let mut p = b_palco(false);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    let rest = b_amostra_com(&p.fonte, 64);
    let pc0 = palco(&p);
    let (mut xmin, mut xmax, mut ymin, mut ymax) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for v in &pc0.rest {
        xmin = xmin.min(v[0]);
        xmax = xmax.max(v[0]);
        ymin = ymin.min(v[1]);
        ymax = ymax.max(v[1]);
    }
    println!(
        "  malha: {} vértices, caixa x {xmin:.3}..{xmax:.3} y {ymin:.3}..{ymax:.3}",
        pc0.rest.len()
    );
    for g in [100.0_f32, 130.0] {
        p.dobra_em_s(g);
        let pc = palco(&p);
        let (x, _) = desdobra(&pc, 0.1, 0.01);
        let c1 = contorno(&p, &pc, &x, &rest);
        for (i, j) in cruzes(&c1) {
            let fora = |k: usize| {
                localiza(&pc, rest[k]).map_or(f64::NAN, |(_, l)| (-l[0]).max(-l[1]).max(-l[2]))
            };
            println!(
                "  S {g}°: cruza {i} {:?} (fora {:.3}) com {j} {:?} (fora {:.3})",
                rest[i],
                fora(i),
                rest[j],
                fora(j)
            );
        }
    }
}

/// Escreve um SVG com a malha (triângulos virados a vermelho) e o contorno, para OLHAR.
fn svg(caminho: &str, pc: &Palco, x: &[[f64; 2]], cont: &[[f64; 2]], alvo: [f64; 2], meio: f64) {
    use std::fmt::Write as _;
    let esc = 600.0 / (2.0 * meio);
    let tx = |p: [f64; 2]| ((p[0] - alvo[0] + meio) * esc, (alvo[1] + meio - p[1]) * esc);
    let mut s = String::from(
        "<svg xmlns='http://www.w3.org/2000/svg' width='600' height='600'><rect width='600' \
         height='600' fill='white'/>",
    );
    let d = dets(&pc.rest, x, &pc.tris);
    for (t, dt) in pc.tris.iter().zip(&d) {
        let (a, b, c) = (
            tx(x[t[0] as usize]),
            tx(x[t[1] as usize]),
            tx(x[t[2] as usize]),
        );
        let cor = if *dt <= 0.0 { "#f44" } else { "none" };
        let _ = write!(
            s,
            "<polygon points='{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}' fill='{cor}' fill-opacity='0.5' \
             stroke='#aac' stroke-width='0.5'/>",
            a.0, a.1, b.0, b.1, c.0, c.1
        );
    }
    let _ = write!(
        s,
        "<polyline fill='none' stroke='black' stroke-width='1.5' points='"
    );
    for q in cont.iter().chain(cont.first()) {
        let (u, v) = tx(*q);
        let _ = write!(s, "{u:.1},{v:.1} ");
    }
    s.push_str("'/></svg>");
    std::fs::write(caminho, s).expect("svg");
}

#[test]
#[ignore = "sonda: desenha a lei e a correcção no cotovelo, para olhar"]
fn diag_desenha_o_cotovelo() {
    let dir = std::env::var("PH2D_SONDA_DIR").unwrap_or_else(|_| "/tmp".into());
    let mut p = b_palco(false);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    let rest = b_amostra_com(&p.fonte, 64);
    for g in [100.0_f32, 130.0] {
        p.dobra_em_s(g);
        let pc = palco(&p);
        let (x, _) = desdobra(&pc, 0.1, 0.01);
        let c0 = contorno(&p, &pc, &pc.lei, &rest);
        let c1 = contorno(&p, &pc, &x, &rest);
        // o cotovelo 1: a junta em repouso ~ (-6.07, 2.5); o alvo é onde a lei a leva
        let j = {
            let pele = p.pele();
            let mut w = pele.scratch();
            pele.weights_corrected([-6.07, 2.0], None, &mut w, &[]);
            pele.blend([-6.07, 2.0], &w)
        };
        svg(
            &format!("{dir}/cotovelo_{g}_lei.svg"),
            &pc,
            &pc.lei,
            &c0,
            j,
            1.2,
        );
        svg(
            &format!("{dir}/zoom_{g}_lei.svg"),
            &pc,
            &pc.lei,
            &c0,
            j,
            0.35,
        );
        svg(
            &format!("{dir}/zoom_{g}_barreira.svg"),
            &pc,
            &x,
            &c1,
            j,
            0.35,
        );
        svg(
            &format!("{dir}/cotovelo_{g}_barreira.svg"),
            &pc,
            &x,
            &c1,
            j,
            1.2,
        );
    }
}

/// A soma dos ângulos dos triângulos à volta de cada vértice da BORDA, no repouso e deformada.
fn angulos_da_borda(pc: &Palco, x: &[[f64; 2]]) -> Vec<(usize, f64, f64)> {
    use std::collections::BTreeMap;
    let mut arestas: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for t in &pc.tris {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            *arestas.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    let mut borda = vec![false; pc.rest.len()];
    for (&(a, b), &c) in &arestas {
        if c == 1 {
            borda[a as usize] = true;
            borda[b as usize] = true;
        }
    }
    let ang = |p: &[[f64; 2]], t: [u32; 3], k: usize| {
        let (o, u, v) = (
            p[t[k] as usize],
            p[t[(k + 1) % 3] as usize],
            p[t[(k + 2) % 3] as usize],
        );
        let (e1, e2) = (sub(u, o), sub(v, o));
        cruz(e1, e2).atan2(e1[0].mul_add(e2[0], e1[1] * e2[1]))
    };
    let mut soma0 = vec![0.0_f64; pc.rest.len()];
    let mut soma1 = vec![0.0_f64; pc.rest.len()];
    for &t in &pc.tris {
        for k in 0..3 {
            soma0[t[k] as usize] += ang(&pc.rest, t, k);
            soma1[t[k] as usize] += ang(x, t, k);
        }
    }
    (0..pc.rest.len())
        .filter(|&i| borda[i])
        .map(|i| (i, soma0[i].to_degrees(), soma1[i].to_degrees()))
        .collect()
}

#[test]
#[ignore = "sonda: os vértices da borda que DÃO A VOLTA (soma dos ângulos > 360°)"]
fn diag_a_borda_da_a_volta() {
    let mut p = b_palco(false);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    for g in [90.0_f32, 100.0, 110.0, 130.0] {
        p.dobra_em_s(g);
        let pc = palco(&p);
        let (x, _) = desdobra(&pc, 0.1, 0.01);
        for (nome, pos) in [("lei", &pc.lei), ("barreira", &x)] {
            let mut maus: Vec<(usize, f64, f64)> = angulos_da_borda(&pc, pos)
                .into_iter()
                .filter(|&(_, _, s)| s > 360.0 || s < 0.0)
                .collect();
            maus.sort_by(|a, b| b.2.total_cmp(&a.2));
            println!(
                "  S {g}° {nome}: {} vértices de borda a dar a volta · pior {:?}",
                maus.len(),
                maus.first()
            );
        }
    }
}

/// O contorno como o artista o vê: preenchido (regra não-zero) e com traço.
fn svg_arte(caminho: &str, cont: &[[f64; 2]], alvo: [f64; 2], meio: f64) {
    use std::fmt::Write as _;
    let esc = 400.0 / (2.0 * meio);
    let tx = |p: [f64; 2]| ((p[0] - alvo[0] + meio) * esc, (alvo[1] + meio - p[1]) * esc);
    let mut s = String::from(
        "<svg xmlns='http://www.w3.org/2000/svg' width='400' height='400'><rect width='400' \
         height='400' fill='white'/><path fill='#e8a95a' fill-rule='nonzero' stroke='#222' \
         stroke-width='3' stroke-linejoin='round' d='M",
    );
    for q in cont {
        let (u, v) = tx(*q);
        let _ = write!(s, "{u:.1},{v:.1} L");
    }
    s.truncate(s.len() - 2);
    s.push_str(" Z'/></svg>");
    std::fs::write(caminho, s).expect("svg");
}

#[test]
#[ignore = "sonda: a ARTE (preenchida e com traço) no cotovelo, lei contra barreira"]
fn diag_desenha_a_arte() {
    let dir = std::env::var("PH2D_SONDA_DIR").unwrap_or_else(|_| "/tmp".into());
    let mut p = b_palco(false);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    let rest = b_amostra_com(&p.fonte, 64);
    for g in [90.0_f32, 100.0, 110.0, 130.0] {
        p.dobra_em_s(g);
        let pc = palco(&p);
        let (x, _) = desdobra(&pc, 0.1, 0.01);
        let c0 = contorno(&p, &pc, &pc.lei, &rest);
        let c1 = contorno(&p, &pc, &x, &rest);
        let j = {
            let pele = p.pele();
            let mut w = pele.scratch();
            pele.weights_corrected([-6.07, 2.0], None, &mut w, &[]);
            pele.blend([-6.07, 2.0], &w)
        };
        svg_arte(&format!("{dir}/arte_{g}_lei.svg"), &c0, j, 0.8);
        svg_arte(&format!("{dir}/arte_{g}_barreira.svg"), &c1, j, 0.8);
    }
}

/// A SILHUETA de uma polilinha fechada: `A ∪ ∅` pelo motor exacto, regra NÃO-ZERO — a região que o
/// preenchimento pinta. Devolve os contornos que saem.
fn silhueta(cont: &[[f64; 2]]) -> Vec<Vec<[f64; 2]>> {
    use ph2d_vec_scene::{FillRule, VecPath, VecVertex};
    let p = VecPath {
        verts: cont.iter().map(|&q| VecVertex::corner(q)).collect(),
        closed: true,
        fill_rule: FillRule::NonZero,
        ..VecPath::default()
    };
    let vazio = VecPath {
        closed: true,
        ..VecPath::default()
    };
    let t = std::time::Instant::now();
    let out = ph2d_vec_boolean::apply(&p, &vazio, ph2d_vec_boolean::BoolOp::Union);
    SILHUETA_US.with(|c| c.set(c.get() + t.elapsed().as_secs_f64() * 1e6));
    let mut aneis = Vec::new();
    for q in &out {
        let cz = q.cooked();
        let mut k = 0;
        while let Some((v, _)) = cz.contour(k) {
            let mut anel = Vec::new();
            for i in 0..v.len() {
                let c = b_cub(v, i);
                for s in 0..8 {
                    anel.push(b_eval(&c, f64::from(s) / 8.0));
                }
            }
            aneis.push(anel);
            k += 1;
        }
    }
    aneis
}

thread_local! {
    static SILHUETA_US: std::cell::Cell<f64> = const { std::cell::Cell::new(0.0) };
}

#[test]
#[ignore = "sonda: a SILHUETA do contorno deformado — o contacto resolvido pela regra do preenchimento"]
fn diag_a_silhueta_do_cotovelo() {
    let dir = std::env::var("PH2D_SONDA_DIR").unwrap_or_else(|_| "/tmp".into());
    let mut p = b_palco(false);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    let rest = b_amostra_com(&p.fonte, 64);
    for g in [60.0_f32, 90.0, 100.0, 110.0, 130.0, 150.0] {
        p.dobra_em_s(g);
        let pc = palco(&p);
        let c0 = contorno(&p, &pc, &pc.lei, &rest);
        SILHUETA_US.with(|c| c.set(0.0));
        let s0 = silhueta(&c0);
        let us = SILHUETA_US.with(std::cell::Cell::get);
        let (x, _) = desdobra(&pc, 0.1, 0.01);
        let c1 = contorno(&p, &pc, &x, &rest);
        let s1 = silhueta(&c1);
        let cr = |a: &Vec<Vec<[f64; 2]>>| a.iter().map(|r| b_auto(r, 1e-7)).sum::<usize>();
        let area = |a: &Vec<Vec<[f64; 2]>>| {
            a.iter()
                .map(|r| ph2d_poly2d::signed_area(r))
                .sum::<f64>()
                .abs()
        };
        println!(
            "  S {g:>4}°: lei cruzes {} → silhueta {} contorno(s) {} cruzes, área {:.4} (lei {:.4}) · \
             barreira→silhueta {} contorno(s) {} cruzes · {us:.0} µs",
            b_auto(&c0, 1e-7),
            s0.len(),
            cr(&s0),
            area(&s0),
            ph2d_poly2d::signed_area(&c0).abs(),
            s1.len(),
            cr(&s1),
        );
        let j = {
            let pele = p.pele();
            let mut w = pele.scratch();
            pele.weights_corrected([-6.07, 2.0], None, &mut w, &[]);
            pele.blend([-6.07, 2.0], &w)
        };
        if let Some(a) = s0.first() {
            svg_arte(&format!("{dir}/sil_{g}_lei.svg"), a, j, 0.8);
        }
        if let Some(a) = s1.first() {
            svg_arte(&format!("{dir}/sil_{g}_barreira.svg"), a, j, 0.8);
        }
    }
}

/// Rasteriza a malha deformada com uma TEXTURA de teste (listras + contorno escuro) — o que uma
/// imagem presa pintaria, na ordem dos triângulos. PPM.
fn raster(caminho: &str, pc: &Palco, x: &[[f64; 2]], alvo: [f64; 2], meio: f64) {
    const N: usize = 400;
    let esc = N as f64 / (2.0 * meio);
    let mut img = vec![[255u8; 3]; N * N];
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for v in &pc.rest {
        x0 = x0.min(v[0]);
        x1 = x1.max(v[0]);
        y0 = y0.min(v[1]);
        y1 = y1.max(v[1]);
    }
    let tex = |p: [f64; 2]| -> [u8; 3] {
        let borda = (p[1] - y0).min(y1 - p[1]).min(p[0] - x0).min(x1 - p[0]);
        if borda < 0.06 {
            return [30, 30, 30];
        }
        #[expect(clippy::cast_possible_truncation, reason = "sonda")]
        let faixa = ((p[0] * 6.0).floor() as i64).rem_euclid(2);
        if faixa == 0 {
            [232, 169, 90]
        } else {
            [214, 120, 60]
        }
    };
    for t in &pc.tris {
        let (a, b, c) = (t[0] as usize, t[1] as usize, t[2] as usize);
        let q = |i: usize| {
            [
                (x[i][0] - alvo[0] + meio) * esc,
                (alvo[1] + meio - x[i][1]) * esc,
            ]
        };
        let (qa, qb, qc) = (q(a), q(b), q(c));
        let area = cruz(sub(qb, qa), sub(qc, qa));
        if area.abs() < 1e-12 {
            continue;
        }
        let lo = |k: usize| qa[k].min(qb[k]).min(qc[k]).floor().max(0.0);
        let hi = |k: usize| qa[k].max(qb[k]).max(qc[k]).ceil().min(N as f64 - 1.0);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "sonda"
        )]
        for py in lo(1) as usize..=hi(1) as usize {
            for px in lo(0) as usize..=hi(0) as usize {
                let p = [px as f64 + 0.5, py as f64 + 0.5];
                let l0 = cruz(sub(qb, p), sub(qc, p)) / area;
                let l1 = cruz(sub(qc, p), sub(qa, p)) / area;
                let l2 = 1.0 - l0 - l1;
                if l0 < 0.0 || l1 < 0.0 || l2 < 0.0 {
                    continue;
                }
                let r = [
                    l0 * pc.rest[a][0] + l1 * pc.rest[b][0] + l2 * pc.rest[c][0],
                    l0 * pc.rest[a][1] + l1 * pc.rest[b][1] + l2 * pc.rest[c][1],
                ];
                let mut cor = tex(r);
                // ⚠️ O ecrã tem o `y` para BAIXO: um triângulo são no mundo tem área NEGATIVA aqui.
                if area > 0.0 {
                    cor = [cor[0], cor[1] / 3, 200]; // virado: a textura ESPELHADA, marcada a roxo
                }
                img[py * N + px] = cor;
            }
        }
    }
    let mut s = format!("P6 {N} {N} 255\n").into_bytes();
    for p in img {
        s.extend_from_slice(&p);
    }
    std::fs::write(caminho, s).expect("ppm");
}

#[test]
#[ignore = "sonda: a IMAGEM presa no cotovelo forte — dobras e textura espelhada"]
fn diag_a_imagem_no_cotovelo() {
    let dir = std::env::var("PH2D_SONDA_DIR").unwrap_or_else(|_| "/tmp".into());
    let mut p = b_palco(false);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    for g in [90.0_f32, 110.0, 130.0] {
        p.dobra_em_s(g);
        let pc = palco(&p);
        let (x, _) = desdobra(&pc, 0.1, 0.01);
        let j = {
            let pele = p.pele();
            let mut w = pele.scratch();
            pele.weights_corrected([-6.07, 2.0], None, &mut w, &[]);
            pele.blend([-6.07, 2.0], &w)
        };
        raster(&format!("{dir}/img_{g}_lei.ppm"), &pc, &pc.lei, j, 0.8);
        raster(&format!("{dir}/img_{g}_barreira.ppm"), &pc, &x, j, 0.8);
    }
}
