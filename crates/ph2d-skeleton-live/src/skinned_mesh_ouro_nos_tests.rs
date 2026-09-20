//! ⭐⭐⭐ **ONDE O ERRO NASCE — a atribuição ao SUBSTRATO, e o que cada wave comprou.**
//!
//! As sondas do irmão [`super::ouro_tests`] respondem *«quão longe do ouro?»*; estas respondem
//! *«de ONDE vem o que sobra?»* — o vinco e a coerência da média, o CHÃO do modelo em função da
//! contagem de nós, a mídia IMAGEM na mesma cena, onde a subdivisão do bind põe os nós, e o que
//! cada wave desta linha comprou de facto.
//!
//! ⚠️ Saiu do irmão por tecto de LOC (`812` contra `700`), por RESPONSABILIDADE.

use super::ouro_reguas_tests::*;

/// ⭐⭐⭐ **SONDA B3 — O MECANISMO.** Onde nasce o defeito, porquê, e qual é o TECTO.
#[test]
fn diag_b_onde_nasce() {
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);

    println!("\n{:=<126}", "");
    println!("SONDA B3 · o MECANISMO — onde nasce o vinco, e qual é o tecto");
    println!("{:=<126}", "");
    println!(
        "juntas dos ossos (repouso, x): {:?}",
        [-8.2_f64, -8.2 + 6.4 / 3.0, -8.2 + 2.0 * 6.4 / 3.0, -1.8]
            .map(|x| (x * 100.0).round() / 100.0)
    );

    // ── (1) O VINCO (Menger, troço RECTO) e a COERÊNCIA da média em círculo ────────────
    let rectas = b_rectas(&rest);
    println!(
        "\n── (1) VINCO por MENGER (h={B_H}, só no troço RECTO: {} de {} amostras) + a DEGENERESCÊNCIA da média em círculo {:─<8}",
        rectas.len(),
        rest.len(),
        ""
    );
    println!(
        "{:>6} | {:>7} {:>7} {:>7} {:>8} | {:>7} {:>7} {:>7} {:>8} | {:>8} {:>8} | {:>7} {:>7}",
        "graus",
        "κ OURO",
        "p90",
        "máx",
        "quina°",
        "κ PRD",
        "p90",
        "máx",
        "quina°",
        "coer p50",
        "coer MIN",
        "κ>2 ou",
        "κ>2 pr"
    );
    let mut piores = Vec::new();
    for graus in [20.0_f32, 45.0, 70.0, 90.0, 120.0, 150.0] {
        p.dobra(graus);
        let pele = p.pele();
        let ouro = p.ouro(&pele, &rest);
        let prod = b_amostra(&p.produto(true, true));
        let kot = b_menger(&ouro, B_H);
        let kpt = b_menger(&prod, B_H);
        let ko: Vec<f64> = rectas.iter().map(|&i| kot[i]).collect();
        let kp: Vec<f64> = rectas.iter().map(|&i| kpt[i]).collect();
        let (ko50, ko90, komax) = b_pct(&mut ko.clone());
        let (kp50, kp90, kpmax) = b_pct(&mut kp.clone());
        let no = ko.iter().filter(|x| **x > 2.0).count();
        let np = kp.iter().filter(|x| **x > 2.0).count();

        // A coerência |Σ w·(cos θ, sin θ)| — `1` = todos os ossos concordam, `0` = a média em
        // círculo DEGENERA e a lei cai na mistura linear (o candy-wrapper).
        let mut coer: Vec<f64> = Vec::with_capacity(rest.len());
        for &x in &rest {
            let mut w = pele.scratch();
            let linha = p
                .campo
                .linha(x)
                .unwrap_or_else(|| b_mais_proximo(&p.campo, x));
            pele.weights_corrected(x, Some(&linha), &mut w, &p.correcoes);
            let (mut sx, mut sy) = (0.0_f64, 0.0_f64);
            for (bn, &pw) in pele.bones().iter().zip(w.iter()) {
                let t = bn.angulo_da_pose();
                sx = pw.mul_add(t.cos(), sx);
                sy = pw.mul_add(t.sin(), sy);
            }
            coer.push(sx.hypot(sy));
        }
        let cmin = coer.iter().copied().fold(f64::MAX, f64::min);
        let (c50, ..) = b_pct(&mut coer.clone());

        println!(
            "{graus:>6.0} | {ko50:>7.3} {ko90:>7.3} {komax:>7.3} {:>8.2} | {kp50:>7.3} {kp90:>7.3} {kpmax:>7.3} {:>8.2} | {c50:>8.4} {cmin:>8.4} | {no:>7} {np:>7}",
            b_quina(komax, B_H),
            b_quina(kpmax, B_H)
        );

        let iko = ko
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map_or(0, |(i, _)| i);
        let ikp = kp
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map_or(0, |(i, _)| i);
        let ic = coer
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.total_cmp(b.1))
            .map_or(0, |(i, _)| i);
        piores.push((
            graus,
            rest[rectas[iko]],
            komax,
            rest[rectas[ikp]],
            kpmax,
            rest[ic],
            cmin,
        ));
    }
    println!("\n  onde (posição de REPOUSO; as juntas estão em x = -6,07 e x = -3,93):");
    for (g, ro, k, rp, kpv, rc, c) in &piores {
        println!(
            "  {g:>5.0}°  pior vinco OURO κ={k:>7.3} em ({:>6.2},{:>5.2})  ·  pior vinco PRODUTO κ={kpv:>7.3} em ({:>6.2},{:>5.2})  ·  pior coerência {c:.4} em ({:>6.2},{:>5.2})",
            ro[0], ro[1], rp[0], rp[1], rc[0], rc[1]
        );
    }

    // ── (2) O TECTO: o CHÃO do modelo em função da contagem de nós ──────────────────────
    println!(
        "\n── (2) O TECTO — o CHÃO do modelo se cada segmento for partido em M {:─<50}",
        ""
    );
    println!(
        "{:>6} | {:>12} {:>12} {:>12} {:>12} {:>12}",
        "graus", "M=1 (34 nós)", "M=2 (68)", "M=4 (136)", "M=8 (272)", "PRODUTO hoje"
    );
    #[expect(clippy::cast_precision_loss, reason = "índices pequenos")]
    for graus in [45.0_f32, 90.0, 120.0, 150.0] {
        p.dobra(graus);
        let pele = p.pele();
        let cozido = p.fonte.cooked();
        let (v, _) = cozido.contour(0).expect("contorno");
        let n = v.len();
        let mut col = Vec::new();
        for m in [1usize, 2, 4, 8] {
            let mut pior = 0.0_f64;
            for k in 0..n {
                let c = b_cub(v, k);
                for j in 0..m {
                    let (t0, t1) = (j as f64 / m as f64, (j + 1) as f64 / m as f64);
                    let ts: Vec<f64> = (0..=B_N)
                        .map(|i| (i as f64 / B_N as f64).mul_add(t1 - t0, t0))
                        .collect();
                    let ouro: Vec<[f64; 2]> = ts
                        .iter()
                        .map(|&t| b_ouro_pt(&pele, &p.campo, &p.correcoes, b_eval(&c, t)).0)
                        .collect();
                    // Reparametrizado em [0,1] no sub-intervalo.
                    let loc: Vec<f64> = (0..=B_N).map(|i| i as f64 / B_N as f64).collect();
                    let best = b_melhor_cubica(&ouro, &loc);
                    for (i, &t) in loc.iter().enumerate() {
                        let q = b_eval(&best, t);
                        pior = pior.max((q[0] - ouro[i][0]).hypot(q[1] - ouro[i][1]));
                    }
                }
            }
            col.push(pior);
        }
        let ouro_full = p.ouro(&pele, &rest);
        let prod = b_amostra(&p.produto(true, true));
        let (_, _, dprod) = b_perfil(&prod, &ouro_full);
        println!(
            "{graus:>6.0} | {:>12.5} {:>12.5} {:>12.5} {:>12.5} {:>12.5}",
            col[0], col[1], col[2], col[3], dprod
        );
    }

    // ── (3) A MÍDIA IMAGEM na MESMA cena — a malha densa, triângulo a triângulo ─────────
    println!(
        "\n── (3) A MÍDIA IMAGEM na mesma cena — distorção de área POR TRIÂNGULO {:─<48}",
        ""
    );
    println!(
        "{:>6} | {:>10} {:>10} {:>10} {:>10} {:>10}",
        "graus", "área total", "p50 tri", "p10 tri", "min tri", "invertidos"
    );
    let malha = p.campo.malha.clone();
    let locais: Vec<[f64; 2]> = (0..malha.rest.len())
        .map(|i| p.campo.local_do_vertice(i).expect("régua"))
        .collect();
    let tri_area = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| {
        ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])) * 0.5
    };
    let rest_tri: Vec<f64> = malha
        .tris
        .iter()
        .map(|t| {
            tri_area(
                locais[t[0] as usize],
                locais[t[1] as usize],
                locais[t[2] as usize],
            )
        })
        .collect();
    let soma_rest: f64 = rest_tri.iter().map(|x| x.abs()).sum();
    for graus in [45.0_f32, 90.0, 120.0, 150.0] {
        p.dobra(graus);
        let pele = p.pele();
        let pos: Vec<[f64; 2]> = (0..malha.rest.len())
            .map(|i| {
                let q = locais[i];
                let mut w = pele.scratch();
                let linha = p.campo.linha_do_vertice(i).map(<[f64]>::to_vec);
                pele.weights_corrected(q, linha.as_deref(), &mut w, &p.correcoes);
                pele.blend(q, &w)
            })
            .collect();
        let mut razoes = Vec::new();
        let mut invertidos = 0usize;
        let mut soma = 0.0_f64;
        for (t, r0) in malha.tris.iter().zip(&rest_tri) {
            let a = tri_area(pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]);
            soma += a.abs();
            if r0.abs() > 1e-12 {
                razoes.push(a / *r0);
                if (a / *r0) < 0.0 {
                    invertidos += 1;
                }
            }
        }
        razoes.sort_by(f64::total_cmp);
        #[expect(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "percentil"
        )]
        let q = |f: f64| razoes[(((razoes.len() - 1) as f64) * f).round() as usize];
        println!(
            "{graus:>6.0} | {:>9.2} % {:>10.4} {:>10.4} {:>10.4} {:>10}",
            soma / soma_rest * 100.0,
            q(0.5),
            q(0.1),
            razoes[0],
            invertidos
        );
    }
    println!(
        "\nloadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("{:=<126}", "");
}

/// ⭐⭐ **SONDA B6 — ONDE A SUBDIVISÃO PÕE OS NÓS, e onde o erro MORA.**
///
/// A [`crate::subdivisao`] parte por um passo UNIFORME (o osso mais curto a dividir por `3`). Esta
/// sonda cruza o espaçamento dos nós com o CHÃO do modelo por segmento e com a distância à junta
/// mais próxima — é a régua da hipótese *«a subdivisão entrega nós a menos nos sítios errados»*.
#[test]
fn diag_b_onde_a_subdivisao_poe_os_nos() {
    let mut p = b_palco(true);
    let juntas = [-8.2_f64, -8.2 + 6.4 / 3.0, -8.2 + 2.0 * 6.4 / 3.0, -1.8];
    println!("\n{:=<110}", "");
    println!("SONDA B6 · onde a subdivisão põe os nós — e onde o erro mora");
    println!("{:=<110}", "");
    println!(
        "juntas em x = {:?}",
        juntas.map(|x| (x * 100.0).round() / 100.0)
    );

    let v: Vec<ph2d_vec_scene::VecVertex> = p
        .fonte
        .cooked()
        .contour(0)
        .map(|(x, _)| x.to_vec())
        .expect("contorno");
    let dist_junta = |x: f64| {
        juntas
            .iter()
            .map(|j| (j - x).abs())
            .fold(f64::MAX, f64::min)
    };

    for alvo_y in [3.0_f64, 2.0] {
        let mut xs: Vec<f64> = v
            .iter()
            .filter(|w| (w.anchor[1] - alvo_y).abs() < 1e-9)
            .map(|w| w.anchor[0])
            .collect();
        xs.sort_by(f64::total_cmp);
        let passos: Vec<f64> = xs.windows(2).map(|w| w[1] - w[0]).collect();
        let pmin = passos.iter().copied().fold(f64::MAX, f64::min);
        let pmax = passos.iter().copied().fold(0.0_f64, f64::max);
        println!(
            "\n  aresta y={alvo_y}: {} nós, passo min={pmin:.4} máx={pmax:.4} (razão {:.2}× ⇒ {})",
            xs.len(),
            pmax / pmin,
            if pmax / pmin < 1.5 {
                "UNIFORME"
            } else {
                "graduado"
            }
        );
        println!(
            "    x dos nós: {}",
            xs.iter()
                .map(|x| format!("{x:.2}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }

    // O CHÃO por segmento contra a distância à junta.
    println!("\n  o CHÃO do modelo por segmento, contra a distância à junta mais próxima:");
    println!(
        "  {:>6} | {:>26} | {:>26}",
        "graus", "segs a ≤0,4 da junta", "segs a >0,4 da junta"
    );
    for graus in [70.0_f32, 90.0, 120.0, 150.0] {
        p.dobra(graus);
        let pele = p.pele();
        let (_, _, por_seg) = b_chao(&p.fonte, &pele, &p.campo, &p.correcoes);
        let (mut perto, mut longe) = (Vec::new(), Vec::new());
        for (k, e) in por_seg.iter().enumerate() {
            let a = v[k].anchor;
            let b = v[(k + 1) % v.len()].anchor;
            let d = dist_junta((a[0] + b[0]) * 0.5);
            if d <= 0.4 {
                perto.push(*e)
            } else {
                longe.push(*e)
            }
        }
        let (p50a, _, maxa) = b_pct(&mut perto.clone());
        let (p50b, _, maxb) = b_pct(&mut longe.clone());
        println!(
            "  {graus:>6.0} | n={:>3} p50={p50a:.5} máx={maxa:.5} | n={:>3} p50={p50b:.5} máx={maxb:.5}",
            perto.len(),
            longe.len()
        );
    }
    println!(
        "\nloadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("{:=<110}", "");
}

/// ⭐⭐ **SONDA B7 — O QUE CADA WAVE COMPROU, dobra a dobra, contra o PADRÃO-OURO.**
///
/// ⛔ Ela existe porque o dono reportou *«não houve nenhuma melhora na deformação do vetor»* sobre
/// a wave do CAMPO (2026-09-20), e a sonda que a casa tinha media a diferença entre DUAS SAÍDAS
/// NOSSAS — nunca a distância ao padrão-ouro.
#[test]
fn diag_b_o_que_cada_wave_comprou() {
    let mut p = b_palco(true);
    let mut q = b_palco(false);
    let rest = b_amostra(&p.fonte);
    println!("\n{:=<118}", "");
    println!(
        "SONDA B7 · o que cada wave comprou — desvio MÁXIMO ao padrão-ouro, em % da ESPESSURA da barra"
    );
    println!("{:=<118}", "");
    println!(
        "{:>6} | {:>12} {:>12} {:>12} | {:>10} {:>10} {:>10}",
        "graus", "8nós ingénua", "8nós curva", "34nós curva", "+SUBDIV", "+CURVA", "+CAMPO"
    );
    println!(
        "{:>6} | {:>12} {:>12} {:>12} | {:>10} {:>10} {:>10}",
        "", "(pré-19/09)", "", "sem campo", "×", "×", "×"
    );
    for graus in [
        20.0_f32, 45.0, 70.0, 90.0, 110.0, 120.0, 130.0, 140.0, 150.0, 160.0,
    ] {
        p.dobra(graus);
        q.dobra(graus);
        let pele = p.pele();
        let ouro = p.ouro(&pele, &rest);
        let d = |path: &ph2d_vec_scene::VecPath| -> f64 {
            let poli = b_amostra(path);
            let (_, _, a) = b_perfil(&poli, &ouro);
            let (_, _, b) = b_perfil(&ouro, &poli);
            a.max(b)
        };
        let v8n = d(&q.produto(false, true));
        let v8c = d(&q.produto(true, true));
        let v34s = d(&p.produto(true, false));
        let v34 = d(&p.produto(true, true));
        println!(
            "{graus:>6.0} | {:>11.2}% {:>11.2}% {:>11.2}% | {:>9.1}× {:>9.1}× {:>9.2}×",
            v8n * 100.0,
            v8c * 100.0,
            v34s * 100.0,
            v8c / v34s.max(1e-12),
            v8n / v8c.max(1e-12),
            v34s / v34.max(1e-12)
        );
    }
    println!(
        "\n  a coluna `+SUBDIV` é (8 nós curva) ÷ (34 nós curva sem campo); `+CURVA` é (8 ingénua) ÷ \
         (8 curva); `+CAMPO` é (34 sem campo) ÷ (34 com campo). Um número ABAIXO de 1,0 quer dizer \
         que a wave PIOROU o desenho."
    );
    println!("\n  PRODUTO de hoje (34 nós · curva · campo), em % da espessura:");
    for graus in [
        20.0_f32, 45.0, 70.0, 90.0, 110.0, 120.0, 130.0, 140.0, 150.0, 160.0,
    ] {
        p.dobra(graus);
        let pele = p.pele();
        let ouro = p.ouro(&pele, &rest);
        let poli = b_amostra(&p.produto(true, true));
        let (_, _, a) = b_perfil(&poli, &ouro);
        let (_, _, b) = b_perfil(&ouro, &poli);
        let sc = b_amostra(&p.produto(true, false));
        let (_, _, c) = b_perfil(&sc, &ouro);
        let (_, _, e) = b_perfil(&ouro, &sc);
        println!(
            "    {graus:>5.0}°  com campo {:>7.2}%   ·   sem campo {:>7.2}%   ⇒  {}",
            a.max(b) * 100.0,
            c.max(e) * 100.0,
            if a.max(b) <= c.max(e) {
                "o campo AJUDA"
            } else {
                "o campo PIORA"
            }
        );
    }
    println!(
        "\nloadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("{:=<118}", "");
}
