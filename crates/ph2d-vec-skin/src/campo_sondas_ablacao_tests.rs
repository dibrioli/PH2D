//! ⭐⭐⭐ **A LENTE A, 2.ª metade — O QUE O CAMPO E O PINCEL DE FACTO COMPRAM.**
//!
//! As sondas `0`–`3` do irmão [`super::campo_sondas_tests`] respondem *«de ONDE vem o peso de cada
//! ponto?»*; estas respondem *«e quanto é que ligar o campo, ou pousar uma mancha, MOVE?»* — a
//! ablação em quatro formas, a mancha do pincel e o alcance dela, e a tabela que um doc-comment do
//! produto publicava a partir de uma fixtura degenerada.
//!
//! ⚠️ Saíram por tecto de LOC (`1 053` contra `700`), por RESPONSABILIDADE.

use super::campo_sondas_tests::*;
use super::campo_tests::*;
use super::pesos::*;
use ph2d_vec_scene::VecPath;

/// ⭐⭐⭐ **SONDA 4 — A PORTA DE ABLAÇÃO bissecta?** — `PH2D_SKIN_CAMPO=0`.
///
/// ⚠️ **A porta é lida no SÍTIO DE CHAMADA e o valor viaja como o `Option<&CampoDoDominio>`**
/// (`campo.then_some(guardado.campo.as_ref()).flatten()`, em `ph2d_skeleton_live::skin_live`) ⇒
/// o A/B honesto é sobre o ARGUMENTO, que é exactamente o que a porta escolhe. ⛔ Pôr a env var
/// aqui seria um canal entre testes — o `cargo test` corre-os em threads do mesmo processo.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-vec-skin --lib diag_a_ablacao -- --nocapture
/// ```
#[test]
fn diag_a_ablacao_do_campo() {
    eprintln!("\n=== SONDA 4 — a ablação do campo, em quatro formas ===");
    eprintln!(
        "  lei_do_campo_activa() = {} (sem a env var posta)",
        crate::curva::lei_do_campo_activa()
    );
    for caso in casos() {
        let diag = diagonal(&caso.fonte);
        let corre = |com: bool| {
            let mut p = caso.fonte.clone();
            crate::curva::aplica_pela_curva_com(
                &caso.pele,
                &mut p,
                &caso.tabela,
                &[],
                true,
                com.then_some(&caso.campo),
            );
            p
        };
        let (sem, com) = (corre(false), corre(true));
        let (mut d_no, mut d_alca) = (Vec::new(), Vec::new());
        for c in 0..sem.contour_count() {
            let (Some((a, _)), Some((b, _))) = (sem.contour(c), com.contour(c)) else {
                continue;
            };
            for (x, y) in a.iter().zip(b) {
                d_no.push(dist(x.anchor, y.anchor));
                d_alca.push(dist(x.in_handle, y.in_handle));
                d_alca.push(dist(x.out_handle, y.out_handle));
            }
        }
        let (_, _, mn) = percentis(&mut d_no);
        let (a50, a90, amax) = percentis(&mut d_alca);
        eprintln!(
            "  {:<48} âncoras max {mn:.9} · alças {a50:.6}/{a90:.6}/{amax:.6} \
             (max {:.3} % da peça)",
            caso.nome,
            100.0 * amax / diag
        );
    }
}

/// ⭐⭐⭐ **SONDA 5 — UMA MANCHA DE PINCEL: onde é que ela aterra?**
///
/// A outra metade do report. O pincel de peso pousa uma [`Correccao`] no ESPAÇO; ela é somada
/// dentro da [`ph2d_skeleton::Skin::weights_corrected`], que corre **nos dois sítios** (na âncora,
/// pela lei dos nós; e em cada amostra, pela lei da curva). ⇒ a pergunta não é *«ela faz alguma
/// coisa?»* mas **de quanto, e em que metade do vértice**.
///
/// ⚠️ O raio é o de FÁBRICA do pincel (`40 px` a `100 ppm` ⇒ `0,40` de mundo), sobre uma barra que
/// tem `1,0` de altura.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-vec-skin --lib diag_a_mancha_do_pincel -- --nocapture
/// ```
#[test]
fn diag_a_mancha_do_pincel() {
    use ph2d_skeleton::{Correccao, Especie};
    eprintln!("\n=== SONDA 5 — a mancha do pincel: onde aterra, e de quanto ===");
    for caso in casos() {
        let diag = diagonal(&caso.fonte);
        // A junta do meio, na borda de baixo da peça — onde o artista corrigiria.
        let centro = [PASSO.mul_add(1.0, BASE[0]), 2.0];
        let dab = Correccao {
            tendon: 0,
            centro,
            raio: 0.40,
            especie: Especie::Soma(1.0),
        };
        let corre = |cs: &[Correccao]| {
            let mut p = caso.fonte.clone();
            crate::curva::aplica_pela_curva_com(
                &caso.pele,
                &mut p,
                &caso.tabela,
                cs,
                true,
                Some(&caso.campo),
            );
            p
        };
        let (sem, com) = (corre(&[]), corre(std::slice::from_ref(&dab)));
        let mut por_classe = [Vec::new(), Vec::new(), Vec::new()];
        for c in 0..sem.contour_count() {
            let (Some((a, _)), Some((b, _))) = (sem.contour(c), com.contour(c)) else {
                continue;
            };
            for (x, y) in a.iter().zip(b) {
                por_classe[0].push(dist(x.anchor, y.anchor));
                por_classe[1].push(dist(x.in_handle, y.in_handle));
                por_classe[2].push(dist(x.out_handle, y.out_handle));
            }
        }
        // E no DESENHO: o contorno cozido, amostrado denso.
        let mut no_desenho = Vec::new();
        for c in 0..sem.cooked().contour_count() {
            let (ca, cb) = (sem.cooked(), com.cooked());
            let (Some((a, fa)), Some((b, _))) = (ca.contour(c), cb.contour(c)) else {
                continue;
            };
            if a.len() != b.len() {
                continue;
            }
            let n = a.len();
            let segs = if fa { n } else { n.saturating_sub(1) };
            for k in 0..segs {
                let (u, v) = (cubica_de(a, k, n), cubica_de(b, k, n));
                for i in 0..=40 {
                    let t = f(i) / 40.0;
                    no_desenho.push(dist(eval_cubica(&u, t), eval_cubica(&v, t)));
                }
            }
        }
        let cols = por_classe.map(|mut v| percentis(&mut v));
        let (_, d90, dmax) = percentis(&mut no_desenho);
        eprintln!(
            "\n  ── {} — diagonal {diag:.4}, mancha em [{:.3}, {:.3}] r=0,40",
            caso.nome, centro[0], centro[1]
        );
        for (j, classe) in ["âncora", "alça in", "alça out"].iter().enumerate() {
            eprintln!(
                "     {classe:<10} p50 {:>10.6}  p90 {:>10.6}  max {:>10.6}  ({:.3} % da peça)",
                cols[j].0,
                cols[j].1,
                cols[j].2,
                100.0 * cols[j].2 / diag
            );
        }
        eprintln!(
            "     DESENHO (contorno cozido, denso)  p90 {d90:.6}  max {dmax:.6}  ({:.3} % da peça)",
            100.0 * dmax / diag
        );
    }
}

/// ⭐ **SONDA 6 — a mancha ALCANÇA alguma coisa?** O controlo da [`diag_a_mancha_do_pincel`]:
/// sem ele, um `0,000000` lê-se como *«a lei não faz nada»* quando pode ser *«a fixtura não a
/// alcança»*.
#[test]
fn diag_o_alcance_da_mancha() {
    use ph2d_skeleton::{Correccao, Especie};
    eprintln!("\n=== SONDA 6 — a mancha alcança? (controlo da sonda 5) ===");
    let centro = [PASSO.mul_add(1.0, BASE[0]), 2.0];
    for caso in casos() {
        let nos = caso.fonte.verts_all().count();
        let mut min_no = f64::MAX;
        let mut min_am = f64::MAX;
        let mut perto = ([0.0, 0.0], 0.0);
        for c in 0..caso.fonte.contour_count() {
            let Some((vs, fechado)) = caso.fonte.contour(c) else {
                continue;
            };
            let n = vs.len();
            for v in vs {
                min_no = min_no.min(dist(v.anchor, centro));
            }
            let segs = if fechado { n } else { n.saturating_sub(1) };
            for k in 0..segs {
                let cub = cubica_de(vs, k, n);
                for i in 0..crate::curva::AMOSTRAS {
                    let t = (f(i) + 0.5) / f(crate::curva::AMOSTRAS);
                    let p = eval_cubica(&cub, t);
                    let d = dist(p, centro);
                    if d < min_am {
                        min_am = d;
                        perto = (p, t);
                    }
                }
            }
        }
        // O `w` no ponto mais perto, com e sem a mancha.
        let dab = Correccao {
            tendon: 0,
            centro,
            raio: 0.40,
            especie: Especie::Soma(1.0),
        };
        let linha = caso.campo.linha(perto.0);
        let mut w0 = caso.pele.scratch();
        let mut w1 = caso.pele.scratch();
        caso.pele
            .weights_corrected(perto.0, linha.as_deref(), &mut w0, &[]);
        caso.pele
            .weights_corrected(perto.0, linha.as_deref(), &mut w1, &[dab]);
        eprintln!(
            "  {:<48} {nos:>3} nós · nó mais perto {min_no:.4} · amostra mais perta {min_am:.4} \
             (t={:.4}) · campo={} · w {:?} → {:?}",
            caso.nome,
            perto.1,
            if linha.is_some() { "Some" } else { "None" },
            w0.iter()
                .map(|x| (x * 1e4).round() / 1e4)
                .collect::<Vec<_>>(),
            w1.iter()
                .map(|x| (x * 1e4).round() / 1e4)
                .collect::<Vec<_>>(),
        );
    }
}

/// ⛔⛔⛔ **SONDA 7 — A TABELA PUBLICADA em [`crate::curva::lei_do_campo_activa`] foi medida numa
/// fixtura DEGENERADA?**
///
/// Ela diz *«rectângulo `40 × 10`, dois ossos … como o bind entrega (20 nós) · dobra `0,8` ⇒
/// `3,480` (`8,7 %`)»*. A [`pele`] deste ficheiro constrói os dois ossos pela
/// [`ph2d_skeleton::SkinBone::new`], que escreve `tendon: 0` nos DOIS ⇒ o
/// [`ph2d_skeleton::Skin::weights_from`] lê a MESMA casa da tabela para os dois ossos.
///
/// Esta sonda corre o MESMO A/B com as duas peles — a do ficheiro e a de tendões distintos.
#[test]
fn diag_a_tabela_publicada_saiu_de_uma_fixtura_degenerada() {
    eprintln!("\n=== SONDA 7 — a tabela publicada, com e sem o colapso de tendões ===");
    let eixos = eixos();
    let mut fonte = forma();
    let cortes = subdivide_como_o_bind(&mut fonte, &eixos);
    let campo = campo_do_caminho(&fonte, &eixos).expect("campo");
    let tabela = pesos_dos_pontos(&fonte, &campo);
    eprintln!(
        "  rectângulo 40x10 · {} nós ({cortes} cortes) · dobra 0,8 · «% da peça» = /W = 40",
        fonte.verts_all().count()
    );
    for (rot, pele_f) in [
        ("pele_degenerada     — tendões [0,0]", pele_degenerada(0.8)),
        ("pele_com_tendoes()  — tendões [0,1]", pele_com_tendoes(0.8)),
    ] {
        let corre = |com: bool| {
            let mut p = fonte.clone();
            crate::curva::aplica_pela_curva_com(
                &pele_f,
                &mut p,
                &tabela,
                &[],
                true,
                com.then_some(&campo),
            );
            p
        };
        let (sem, com) = (corre(false), corre(true));
        let mut d = Vec::new();
        let mut parados = 0usize;
        let mut total = 0usize;
        for c in 0..sem.contour_count() {
            let (Some((a, _)), Some((b, _))) = (sem.contour(c), com.contour(c)) else {
                continue;
            };
            for (x, y) in a.iter().zip(b) {
                d.push(dist(x.anchor, y.anchor));
                d.push(dist(x.in_handle, y.in_handle));
                d.push(dist(x.out_handle, y.out_handle));
            }
        }
        // Quantos pontos da FONTE o `w` deixa PARADOS (soma zero ⇒ a mistura devolve o ponto)?
        let mut w = pele_f.scratch();
        for c in 0..fonte.contour_count() {
            let Some((vs, _)) = fonte.contour(c) else {
                continue;
            };
            for (k, v) in vs.iter().enumerate() {
                let linha = linha_plana(&tabela, k, campo.ossos());
                pele_f.weights_corrected(v.anchor, Some(linha), &mut w, &[]);
                total += 1;
                if w.iter().sum::<f64>() <= 0.0 {
                    parados += 1;
                }
            }
        }
        let (p50, p90, max) = percentis(&mut d);
        eprintln!(
            "     {rot}  Δ(com−sem) p50 {p50:.6} p90 {p90:.6} max {max:.6} ({:.2} % de W) \
             · nós com w=0 (NÃO se movem): {parados}/{total}",
            100.0 * max / W
        );
    }
}

/// ⭐⭐⭐ **SONDA 8 — O CAMPO EM SI É SÃO?** O perfil do peso ao longo da barra.
///
/// Se o desenho já está a `0,05 %` do padrão-ouro (sonda 3), então uma deformação que o dono ache
/// má **não pode ser da fiação do peso** — ou é da LEI da mistura, ou é do próprio campo. Esta
/// sonda mede a segunda hipótese: a LARGURA DA TRANSIÇÃO de cada osso, em unidades da ALTURA da
/// peça. *Uma transição muito mais estreita que a peça vinca; uma muito mais larga borra a junta.*
#[test]
fn diag_o_perfil_do_campo_ao_longo_da_peca() {
    eprintln!("\n=== SONDA 8 — o campo em si: largura da transição ===");
    let mut fonte = barra_do_dono();
    let eixos = eixos_do_dono();
    subdivide_como_o_bind(&mut fonte, &eixos);
    let campo = campo_do_caminho(&fonte, &eixos).expect("campo");
    let n = campo.ossos();
    // A linha média da barra: y = 2,5, x de -8,2 a -1,8.
    let amostras: Vec<(f64, Vec<f64>)> = (0..=64)
        .filter_map(|i| {
            let x = (-1.8_f64 - -8.2).mul_add(f(i) / 64.0, -8.2);
            campo.linha([x, 2.5]).map(|l| (x, l))
        })
        .collect();
    eprintln!(
        "  {} amostras na linha média, {n} ossos · altura da peça = 1,0 · passo do osso = {PASSO:.4}",
        amostras.len()
    );
    for j in 0..n {
        // A largura em que o peso `j` vai de 0,1 a 0,9 — a transição.
        let (mut lo, mut hi) = (f64::NAN, f64::NAN);
        for (x, l) in &amostras {
            if l[j] >= 0.1 && lo.is_nan() {
                lo = *x;
            }
            if l[j] >= 0.9 {
                hi = *x;
            }
        }
        let pico = amostras.iter().map(|(_, l)| l[j]).fold(0.0_f64, f64::max);
        eprintln!(
            "     osso {j}: pico {pico:.4} · 0,1 em x={lo:>7.3} · 0,9 em x={hi:>7.3} · \
             transição {:.3} (= {:.2} alturas da peça)",
            (hi - lo).abs(),
            (hi - lo).abs()
        );
    }
    eprintln!("  perfil (x, pesos):");
    for (x, l) in amostras.iter().step_by(8) {
        eprintln!(
            "     x={x:>7.3}  {:?}",
            l.iter()
                .map(|v| (v * 1000.0).round() / 1000.0)
                .collect::<Vec<_>>()
        );
    }
}

/// A CURVA amostrada — `n` pontos por segmento cúbico, na ordem do contorno.
///
/// ⚠️ **Ela existe porque um ponto de controlo NÃO é o desenho:** uma alça que anda `δ` move a
/// curva no máximo `4/9 · δ` (o pico da base de Bernstein interior), logo medir alças **majora** o
/// que o artista vê — e medir âncoras **subestima**, porque elas podem não se mexer nada.
fn amostra_a_curva(p: &VecPath, n: usize) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    for c in 0..p.contour_count() {
        let Some((vs, fechado)) = p.contour(c) else {
            continue;
        };
        let pares = if fechado {
            vs.len()
        } else {
            vs.len().saturating_sub(1)
        };
        for i in 0..pares {
            let (a, b) = (&vs[i], &vs[(i + 1) % vs.len()]);
            for k in 0..n {
                let t = k as f64 / n as f64;
                let u = 1.0 - t;
                let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                out.push(std::array::from_fn(|d| {
                    w3.mul_add(
                        b.anchor[d],
                        w2.mul_add(
                            b.in_handle[d],
                            w1.mul_add(a.out_handle[d], w0 * a.anchor[d]),
                        ),
                    )
                }));
            }
        }
    }
    out
}

/// ⭐⭐⭐ **SONDA 8 — A TABELA PUBLICADA, RE-MEDIDA NAS TRÊS LINHAS.**
///
/// A [`crate::curva::lei_do_campo_activa`] publicava três números tirados da [`pele`] deste
/// ficheiro, que escreve `tendon: 0` em **ambos** os ossos ([`SkinBone::new`] não recebe tendão) —
/// as duas colunas do campo colapsam numa e cinco dos vinte nós ficam com `w = 0`, logo **não se
/// movem**. Esta sonda corre as MESMAS três linhas com a [`pele_com_tendoes`], que é a única
/// diferença entre as duas colunas.
///
/// ⚠️ **Ela imprime as duas colunas de propósito:** um número honesto ao lado do número que ele
/// substitui é o que impede a tabela de voltar a ser copiada à mão.
#[test]
fn diag_a_tabela_honesta_das_tres_linhas() {
    use ph2d_skeleton::{Correccao, Especie};
    eprintln!("\n=== SONDA 8 — a tabela do `lei_do_campo_activa`, re-medida ===");
    eprintln!("  «% da peça» = max / W = 40 · Δ = (com campo) − (sem campo)");
    eprintln!(
        "  {:<46} | {:^17} | {:^25}",
        "linha da tabela", "FIXTURA DEGENERADA", "FIXTURA HONESTA"
    );
    eprintln!(
        "  {:<46} | {:>7} {:>9} | {:>7} {:>7} {:>8}",
        "", "alças", "curva", "alças", "curva", "% de W"
    );

    let eixos = eixos();
    let dab = Correccao {
        tendon: 1,
        centro: [W * 0.5, 0.0],
        raio: 4.0,
        especie: Especie::Soma(1.0),
    };
    let linhas: [(&str, bool, f64, &[Correccao]); 3] = [
        (
            "como o artista desenha (4 nós) · dobra 0,8",
            false,
            0.8,
            &[],
        ),
        ("como o BIND entrega (20 nós) · dobra 0,8", true, 0.8, &[]),
        (
            "como o bind entrega · dobra 1,5 · peso pintado",
            true,
            1.5,
            std::slice::from_ref(&dab),
        ),
    ];

    for (rot, subdividir, dobra, correcoes) in linhas {
        let mut fonte = forma();
        if subdividir {
            subdivide_como_o_bind(&mut fonte, &eixos);
        }
        let campo = campo_do_caminho(&fonte, &eixos).expect("campo");
        let tabela = pesos_dos_pontos(&fonte, &campo);
        let mut col = Vec::new();
        for pele_f in [pele_degenerada(dobra), pele_com_tendoes(dobra)] {
            let corre = |com: bool| {
                let mut p = fonte.clone();
                crate::curva::aplica_pela_curva_com(
                    &pele_f,
                    &mut p,
                    &tabela,
                    correcoes,
                    true,
                    com.then_some(&campo),
                );
                p
            };
            let (sem, com) = (corre(false), corre(true));
            let (mut d, mut anc) = (Vec::new(), Vec::new());
            for c in 0..sem.contour_count() {
                let (Some((a, _)), Some((b, _))) = (sem.contour(c), com.contour(c)) else {
                    continue;
                };
                for (x, y) in a.iter().zip(b) {
                    anc.push(dist(x.anchor, y.anchor));
                    d.push(dist(x.anchor, y.anchor));
                    d.push(dist(x.in_handle, y.in_handle));
                    d.push(dist(x.out_handle, y.out_handle));
                }
            }
            // Os nós que a tabela deixa com soma zero — eles NÃO se movem, com ou sem campo.
            let mut w = pele_f.scratch();
            let (mut parados, mut total) = (0usize, 0usize);
            for c in 0..fonte.contour_count() {
                let Some((vs, _)) = fonte.contour(c) else {
                    continue;
                };
                for (k, v) in vs.iter().enumerate() {
                    let linha = linha_plana(&tabela, k, campo.ossos());
                    pele_f.weights_corrected(v.anchor, Some(linha), &mut w, correcoes);
                    total += 1;
                    if w.iter().sum::<f64>() <= 0.0 {
                        parados += 1;
                    }
                }
            }
            let mut curva: Vec<f64> = amostra_a_curva(&sem, 16)
                .into_iter()
                .zip(amostra_a_curva(&com, 16))
                .map(|(x, y)| dist(x, y))
                .collect();
            let (_, _, cmax) = percentis(&mut curva);
            let (_, _, max) = percentis(&mut d);
            let (_, _, amax) = percentis(&mut anc);
            col.push((max, 100.0 * max / W, parados, total, cmax, amax));
        }
        assert_eq!(
            col[1].5, 0.0,
            "as âncoras não se movem — é a lei que esta sonda documenta"
        );
        eprintln!(
            "  {rot:<46} | {:>7.3} {:>9.3} | {:>7.3} {:>7.3} {:>7.2}%",
            col[0].0,
            col[0].4,
            col[1].0,
            col[1].4,
            100.0 * col[1].4 / W
        );
    }
    eprintln!("  âncoras: 0,000 nas SEIS células — ligar o campo NÃO move um único nó.");
    eprintln!("  ⇒ a `curva` da fixtura HONESTA é a tabela que o doc publica.");
}
