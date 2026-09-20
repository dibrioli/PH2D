//! ⭐⭐⭐ **A LENTE B — as SONDAS e o GATE contra o padrão-ouro** (auditoria de 2026-09-20).
//!
//! ⚠️ **As RÉGUAS vivem no irmão [`super::ouro_reguas_tests`]**, e o corte é por RESPONSABILIDADE
//! (tecto de LOC, `1 759` contra `700`): um instrumento e uma medição são coisas diferentes, e
//! este repo já pagou a lição de um ficheiro em que as duas moram juntas — quem afina a régua
//! deixa de ver quantas medições dependem dela.

use super::ouro_reguas_tests::*;

/// ⭐⭐⭐ **SONDA B1 — A ESCADA DE DOBRAS na barra REAL da cena, contra o PADRÃO-OURO.**
///
/// A **tabela A** responde *«quão longe do padrão-ouro está o que o produto desenha?»*; a
/// **tabela B** responde a pergunta que decide a wave seguinte: *«e o padrão-ouro, ele PRÓPRIO é
/// bom?»* — área, largura do braço, vinco e auto-intersecção, com o REPOUSO como CONTROLO.
#[test]
fn diag_b_escada_de_dobras() {
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    let rest_anc = b_ancoras(&p.fonte);
    let area_rest = b_area(&rest);
    const EPS: f64 = 1e-7;

    let comp: Vec<f64> = {
        let n = rest.len();
        (0..n)
            .map(|i| {
                let (a, b) = (rest[i], rest[(i + 1) % n]);
                (a[0] - b[0]).hypot(a[1] - b[1])
            })
            .collect()
    };
    let n_zero = comp.iter().filter(|c| **c <= EPS).count();
    let vivo_min = comp
        .iter()
        .copied()
        .filter(|c| *c > EPS)
        .fold(f64::MAX, f64::min);
    let rectas = b_rectas(&rest);

    println!("\n{:=<128}", "");
    println!(
        "SONDA B1 · a escada de dobras na barra da cena, contra o PADRÃO-OURO (a lei da mídia IMAGEM)"
    );
    println!("{:=<128}", "");
    println!(
        "nós={} · amostras do contorno={} ({} no troço RECTO) · malha do campo={} vértices / {} tendões · ossos resolvidos={} · área de repouso={area_rest:.5}",
        p.fonte.verts_all().count(),
        rest.len(),
        rectas.len(),
        p.campo.malha.rest.len(),
        p.campo.ossos(),
        p.pele().len(),
    );
    println!(
        "segmentos de repouso DEGENERADOS (≤ {EPS:.0e}): {n_zero} de {} — a barra é uma CÁPSULA \
         (raio 0,5 = meia altura), logo as duas paredes dos topos têm comprimento ZERO; o menor \
         segmento VIVO mede {vivo_min:.6}.",
        rest.len()
    );
    let fora = rest.iter().filter(|q| p.campo.linha(**q).is_none()).count();
    #[expect(clippy::cast_precision_loss, reason = "contagens pequenas")]
    let pct_fora = fora as f64 / rest.len() as f64 * 100.0;
    println!(
        "amostras do contorno SEM triângulo no campo: {fora}/{} ({pct_fora:.2} %)",
        rest.len()
    );
    println!(
        "loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );

    let mut a = Vec::new();
    let mut b = Vec::new();
    // O CONTROLO: o REPOUSO, pelas mesmas réguas.
    let kr: Vec<f64> = {
        let k = b_menger(&rest, B_H);
        rectas.iter().map(|&i| k[i]).collect()
    };
    let (_, kr90, krmax) = b_pct(&mut kr.clone());
    let xr = b_auto(&rest, EPS);

    for graus in [0.0_f32, 20.0, 45.0, 70.0, 90.0, 120.0, 150.0] {
        p.dobra(graus);
        let pele = p.pele();
        let ouro = p.ouro(&pele, &rest);
        let ouro_anc = p.ouro(&pele, &rest_anc);
        let prod_path = p.produto(true, true);
        let prod = b_amostra(&prod_path);
        let prod_anc = b_ancoras(&prod_path);

        let (d50, d90, dmax) = {
            let (x, y, z) = b_perfil(&prod, &ouro);
            let (x2, y2, z2) = b_perfil(&ouro, &prod);
            (x.max(x2), y.max(y2), z.max(z2))
        };
        let nomax = ouro_anc
            .iter()
            .zip(&prod_anc)
            .map(|(u, v)| (u[0] - v[0]).hypot(u[1] - v[1]))
            .fold(0.0_f64, f64::max);
        let (_, q90, qmax) = b_pct(&mut b_quebra_nos(&prod_path));
        let move_se = ouro
            .iter()
            .zip(&rest)
            .map(|(u, v)| (u[0] - v[0]).hypot(u[1] - v[1]))
            .fold(0.0_f64, f64::max);
        a.push((graus, d50, d90, dmax, nomax, q90, qmax, move_se));

        let ko_t = b_menger(&ouro, B_H);
        let kp_t = b_menger(&prod, B_H);
        let mut kos: Vec<f64> = rectas.iter().map(|&i| ko_t[i]).collect();
        let mut kps: Vec<f64> = rectas.iter().map(|&i| kp_t[i]).collect();
        let (_, ko90, komax) = b_pct(&mut kos);
        let (_, kp90, kpmax) = b_pct(&mut kps);
        let mut lo = b_larguras(&rest, &ouro);
        let mut lp = b_larguras(&rest, &prod);
        let lomin = lo.iter().copied().fold(f64::MAX, f64::min);
        let lpmin = lp.iter().copied().fold(f64::MAX, f64::min);
        let (lo50, ..) = b_pct(&mut lo);
        let (lp50, ..) = b_pct(&mut lp);
        b.push((
            graus,
            b_area(&ouro) / area_rest,
            b_area(&prod) / area_rest,
            lo50,
            lomin,
            lp50,
            lpmin,
            ko90,
            komax,
            kp90,
            kpmax,
            b_auto(&ouro, EPS),
            b_auto(&prod, EPS),
        ));
    }

    println!(
        "\n── TABELA A · FIDELIDADE AO PADRÃO-OURO ── a barra tem 1,0 u de ESPESSURA e 7,0 de comprimento {:─<30}",
        ""
    );
    println!(
        "{:>6} {:>10} {:>10} {:>10} {:>10} {:>13} {:>11} {:>11}",
        "graus",
        "desv p50",
        "desv p90",
        "desv max",
        "% da esp",
        "erro nos NÓS",
        "quebra p90",
        "quebra max"
    );
    for (g, x, y, z, no, q90, qmax, mv) in &a {
        println!(
            "{g:>6.0} {x:>10.5} {y:>10.5} {z:>10.5} {:>9.2} % {no:>13.2e} {q90:>10.4}° {qmax:>10.4}°   (a dobra move a arte {mv:.3})",
            z * 100.0
        );
    }

    println!(
        "\n── TABELA B · QUALIDADE INTRÍNSECA ── o PADRÃO-OURO é bom? (o REPOUSO é o CONTROLO) {:─<43}",
        ""
    );
    println!(
        "{:>6} | {:>8} {:>8} | {:>8} {:>8} | {:>8} {:>8} | {:>7} {:>7} {:>8} | {:>7} {:>7} {:>8} | {:>4} {:>4}",
        "graus",
        "áreaOURO",
        "áreaPRD",
        "largOURO",
        "min",
        "largPRD",
        "min",
        "κ OU90",
        "κ máx",
        "quina°",
        "κ PR90",
        "κ máx",
        "quina°",
        "Xou",
        "Xpr"
    );
    println!(
        "{:>6} | {:>8} {:>8} | {:>8} {:>8} | {:>8} {:>8} | {:>7.3} {:>7.3} {:>8.2} | {:>7} {:>7} {:>8} | {:>4} {:>4}",
        "REST",
        "100.00%",
        "—",
        "1.0000",
        "1.0000",
        "—",
        "—",
        kr90,
        krmax,
        b_quina(krmax, B_H),
        "—",
        "—",
        "—",
        xr,
        "—"
    );
    for (g, ao, ap, lo50, lomin, lp50, lpmin, ko90, komax, kp90, kpmax, xo, xp) in &b {
        println!(
            "{g:>6.0} | {:>7.2}% {:>7.2}% | {lo50:>8.4} {lomin:>8.4} | {lp50:>8.4} {lpmin:>8.4} | \
             {ko90:>7.3} {komax:>7.3} {:>8.2} | {kp90:>7.3} {kpmax:>7.3} {:>8.2} | {xo:>4} {xp:>4}",
            ao * 100.0,
            ap * 100.0,
            b_quina(*komax, B_H),
            b_quina(*kpmax, B_H)
        );
    }
    println!(
        "\nκ = curvatura de MENGER à escala h={B_H} (rad/u), SÓ nas {} amostras cujo repouso está no \
         troço RECTO — a tampa da cápsula (raio 0,5) leria 2,0 e ficou de FORA. `quina°` é o mesmo \
         κ máx lido como ÂNGULO DE QUINA, que satura em 180°. X = pares de segmentos do contorno \
         que se CRUZAM.",
        rectas.len()
    );
    println!("{:=<128}", "");
}

/// ⭐⭐⭐ **SONDA B2 — A ATRIBUIÇÃO.** Parte o erro em parcelas e diz a percentagem de cada uma.
///
/// Cada linha é uma ABLAÇÃO pela porta do produto, medida contra o MESMO padrão-ouro.
#[test]
fn diag_b_atribuicao() {
    let mut p = b_palco(true);
    let mut q = b_palco(false); // o mundo de ANTES de 2026-09-19 — oito nós
    let rest = b_amostra(&p.fonte);

    println!("\n{:=<122}", "");
    println!(
        "SONDA B2 · a ATRIBUIÇÃO — quanto do erro é MODELO (nós/cúbica) e quanto é PROCEDIMENTO (o ajuste)"
    );
    println!("{:=<122}", "");
    println!(
        "nós: BIND={} · sem subdivisão={} · o padrão-ouro é SEMPRE o mesmo (o campo do bind subdividido)",
        p.fonte.verts_all().count(),
        q.fonte.verts_all().count()
    );
    println!(
        "{:>6} {:<34} {:>10} {:>10} {:>10} {:>10}",
        "graus", "configuração", "desv p50", "desv p90", "desv max", "% da esp"
    );
    println!("{:-<122}", "");

    for graus in [45.0_f32, 90.0, 120.0, 150.0] {
        p.dobra(graus);
        q.dobra(graus);
        let pele = p.pele();
        let ouro: Vec<[f64; 2]> = rest
            .iter()
            .map(|&x| b_ouro_pt(&pele, &p.campo, &p.correcoes, x).0)
            .collect();

        let (chao34, mesmo_t34, por_seg) = b_chao(&p.fonte, &pele, &p.campo, &p.correcoes);
        let (chao8, mesmo_t8, _) = b_chao(&q.fonte, &pele, &p.campo, &p.correcoes);

        let linhas: Vec<(String, Vec<[f64; 2]>)> = vec![
            (
                "PRODUTO (34 nós · curva · campo)".into(),
                b_amostra(&p.produto(true, true)),
            ),
            (
                "  ablação: PH2D_SKIN_CAMPO=0".into(),
                b_amostra(&p.produto(true, false)),
            ),
            (
                "  ablação: PH2D_SKIN_CURVE=0 (ingénua)".into(),
                b_amostra(&p.produto(false, true)),
            ),
            (
                "  8 nós · curva · campo".into(),
                b_amostra(&q.produto(true, true)),
            ),
            (
                "  8 nós · ingénua (antes de 19/09)".into(),
                b_amostra(&q.produto(false, true)),
            ),
            ("CHÃO do modelo · 34 nós".into(), chao34),
            ("CHÃO do modelo ·  8 nós".into(), chao8),
        ];

        let mut prod_max = 0.0_f64;
        let mut chao_max = 0.0_f64;
        for (i, (nome, poli)) in linhas.iter().enumerate() {
            let (x, y, z) = b_perfil(poli, &ouro);
            let (x2, y2, z2) = b_perfil(&ouro, poli);
            let (d50, d90, dmax) = (x.max(x2), y.max(y2), z.max(z2));
            if i == 0 {
                prod_max = dmax;
            }
            if i == 5 {
                chao_max = dmax;
            }
            println!(
                "{:>6} {nome:<34} {d50:>10.5} {d90:>10.5} {dmax:>10.5} {:>9.2} %",
                if i == 0 {
                    format!("{graus:.0}")
                } else {
                    String::new()
                },
                dmax * 100.0
            );
        }
        let fit = (prod_max - chao_max).max(0.0);
        println!(
            "{:>6} {:<34} MODELO {:>5.1} %  ·  PROCEDIMENTO {:>5.1} %   (chão mesmo-t: 34 nós {mesmo_t34:.5} · 8 nós {mesmo_t8:.5})",
            "",
            "→ atribuição do PRODUTO:",
            chao_max / prod_max * 100.0,
            fit / prod_max * 100.0
        );
        // Os segmentos onde o MODELO não chega — é lá que faltam nós.
        let mut ord: Vec<(usize, f64)> = por_seg.iter().copied().enumerate().collect();
        ord.sort_by(|a, b| b.1.total_cmp(&a.1));
        let cozido = p.fonte.cooked();
        let (v, _) = cozido.contour(0).expect("contorno");
        let piores: Vec<String> = ord
            .iter()
            .take(3)
            .map(|(k, e)| {
                let a = v[*k].anchor;
                format!("#{k} em ({:.2},{:.2}) → {e:.5}", a[0], a[1])
            })
            .collect();
        println!(
            "{:>6} {:<34} {}",
            "",
            "  piores segmentos do CHÃO:",
            piores.join("  ·  ")
        );
        println!("{:-<122}", "");
    }
    println!(
        "loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("{:=<122}", "");
}
