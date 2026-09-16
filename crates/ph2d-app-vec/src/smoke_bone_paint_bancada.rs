//! ⏱️⭐⭐⭐ **A BANCADA DAS DUAS LEIS DE PELE** — irmã do [`super`] pelo tecto de LOC, cortada por
//! RESPONSABILIDADE: *a cena demonstra e os gates dela guardam-na* é uma pergunta, *quanto cada lei
//! de pesos entrega e quanto custa* é outra.
//!
//! ⚠️ Ela vive dentro do módulo de testes do irmão (`#[path]`), e é de propósito: as fixturas são as
//! dele (`cena`, `campo_da_cena`, `posadas_da_cena`, `ponto_do_produto`). *Uma cópia delas divergiria
//! no primeiro ajuste, e a bancada passaria a medir uma cena que o dono não vê.*

use super::*;

/// ⏱️⭐⭐⭐ **A BANCADA DAS DUAS LEIS (`--ignored`)** — a tabela que decidiu esta jornada, e que
/// qualquer pessoa pode voltar a correr.
///
/// ```text
/// cargo test -p ph2d-app-vec --lib -- --ignored --nocapture sonda_a_cena_nas_duas_leis
/// ```
///
/// Ela imprime três coisas, e cada uma respondeu a uma pergunta:
///
/// 1. **as duas leis lado a lado**, com a coluna do VAZAMENTO — é ali que a `strength = 2,0`
///    aparece pelo que é (`26 px`), e que as duas linhas do padrão-ouro saem **idênticas**,
///    provando que o alcance ficou inerte;
/// 2. **graduada contra uniforme** — ⭐⭐ e o resultado INVERTEU um defeito registado: a
///    graduação da grelha do bind era *anti-correlacionada* com o erro da lei euclidiana (fina
///    no eixo do osso, grossa na borda, que é onde o *bump* normalizado explodia) e é
///    **correlacionada** com o do padrão-ouro (que varia depressa junto das restrições, isto é,
///    junto dos eixos). Medido: a graduada entrega menos faceta com `~30 %` MENOS peças.
///    *Trocar a lei curou a grelha.*
/// 3. **a faceta contra a densidade** — `O(h^1,2)`, entre `O(h)` e `O(h²)`: o padrão-ouro é
///    `C¹` mas não `C²` na fronteira do conjunto activo (é o preço das caixas, que é o que
///    compra a localidade), logo a malha nunca o segue com a ordem cheia.
///
/// ⚠️ **Acima de `load ~5` os relógios desta workstation não valem nada** (`CLAUDE.md` §5.0) —
/// mas aqui nenhuma coluna é um relógio: são todas geometria, e são determinísticas.
#[test]
#[ignore = "bancada: imprime a tabela das duas leis, sem barra"]
fn sonda_a_cena_nas_duas_leis() {
    fn medir(
        altura: u32,
        forca: Option<f64>,
        usar_pesos: bool,
        grelha: ph2d_poly2d::GridOptions,
    ) -> (f64, f64, f64, f64, usize) {
        let (sim, e) = cena_com(altura, forca, grelha);
        let sm = ph2d_skeleton_live::skin_image::skinned_mesh_of(&sim, e).expect("malha");
        let (p2l, pele) = ph2d_skeleton_live::skin_image::deform_field(&sim, e, sm.mesh.size, PPM)
            .expect("campo");
        let ossos = if usar_pesos { sm.ossos() } else { 0 };
        let mut w = pele.scratch();
        let campo = |q: [f64; 2], pesos: &[f64], w: &mut Vec<f64>| {
            let p = p2l.apply(q);
            if ossos == 0 {
                pele.point(p, w)
            } else {
                pele.point_with(p, pesos, w)
            }
        };
        let posadas: Vec<[f64; 2]> = sm
            .mesh
            .rest
            .iter()
            .enumerate()
            .map(|(v, &q)| campo(q, if ossos == 0 { &[] } else { sm.pesos_de(v) }, &mut w))
            .collect();
        // faceta
        let mut w2 = pele.scratch();
        let d = ph2d_poly2d::deviation_attrs(
            &sm.mesh,
            &posadas,
            if ossos == 0 { &[] } else { &sm.pesos },
            ossos,
            &mut |q, pesos| campo(q, pesos, &mut w2),
        ) * PX_POR_METRO;
        // triangulos invertidos
        let area = |t: &[u32; 3], p: &[[f64; 2]]| {
            let (a, b, c) = (p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]);
            (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])
        };
        // ⚠️ O REPOUSO tem de atravessar a MESMA regua: a malha esta' em pixels (y para
        // BAIXO) e a posada em metros de mundo (y para CIMA), logo comparar sinais crus le^
        // 100% de inversao sobre uma cena perfeita.
        let repouso_mapeado: Vec<[f64; 2]> = sm.mesh.rest.iter().map(|&q| p2l.apply(q)).collect();
        let invertidos = sm
            .mesh
            .tris
            .iter()
            .filter(|t| area(t, &repouso_mapeado) * area(t, &posadas) <= 0.0)
            .count();
        let inv_pct = 100.0 * invertidos as f64 / sm.mesh.tris.len() as f64;
        // esticao maximo por aresta
        let mut estica = 0.0_f64;
        for t in &sm.mesh.tris {
            for k in 0..3 {
                let (i, j) = (t[k] as usize, t[(k + 1) % 3] as usize);
                let r = (sm.mesh.rest[i][0] - sm.mesh.rest[j][0])
                    .hypot(sm.mesh.rest[i][1] - sm.mesh.rest[j][1]);
                let a = (posadas[i][0] - posadas[j][0]).hypot(posadas[i][1] - posadas[j][1]);
                if r > 0.0 {
                    // a malha esta' em pixels, a posada em metros: normaliza pela regua
                    estica = estica.max(a * f64::from(PPM) / r);
                }
            }
        }
        // circulo DESENHADO: baricentrico na malha
        let centro = [f64::from(super::LARGURA_PX) / 2.0, f64::from(altura) / 2.0];
        let r = f64::from(altura) * 0.4;
        let n = 720;
        let mut pts = Vec::with_capacity(n);
        for i in 0..n {
            let th = std::f64::consts::TAU * i as f64 / n as f64;
            let q = [centro[0] + r * th.cos(), centro[1] + r * th.sin()];
            let mut melhor: Option<(f64, [f64; 2])> = None;
            for t in &sm.mesh.tris {
                let (a, b, c) = (
                    sm.mesh.rest[t[0] as usize],
                    sm.mesh.rest[t[1] as usize],
                    sm.mesh.rest[t[2] as usize],
                );
                let den = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
                if den.abs() < 1e-12 {
                    continue;
                }
                let u = ((q[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (q[1] - a[1])) / den;
                let v = ((b[0] - a[0]) * (q[1] - a[1]) - (q[0] - a[0]) * (b[1] - a[1])) / den;
                let fora = (-u).max(-v).max(u + v - 1.0).max(0.0);
                let (pa, pb, pc) = (
                    posadas[t[0] as usize],
                    posadas[t[1] as usize],
                    posadas[t[2] as usize],
                );
                let p = [
                    (1.0 - u - v) * pa[0] + u * pb[0] + v * pc[0],
                    (1.0 - u - v) * pa[1] + u * pb[1] + v * pc[1],
                ];
                if melhor.is_none_or(|(f, _)| fora < f) {
                    melhor = Some((fora, p));
                }
            }
            pts.push(melhor.expect("ha' triangulos").1);
        }
        let c = pts
            .iter()
            .fold([0.0, 0.0], |a, p| [a[0] + p[0], a[1] + p[1]]);
        let c = [c[0] / n as f64, c[1] / n as f64];
        let (mut rmin, mut rmax) = (f64::INFINITY, 0.0_f64);
        for p in &pts {
            let d = (p[0] - c[0]).hypot(p[1] - c[1]);
            rmin = rmin.min(d);
            rmax = rmax.max(d);
        }
        (d, inv_pct, estica, rmax / rmin, sm.mesh.tris.len())
    }

    let g = ph2d_poly2d::GridOptions::default();
    println!("lei                        forca  pecas  faceta_px  inv%   estica  circulo  VAZAM");
    for (nome, usar) in [("euclidiana", false), ("BBW guardado", true)] {
        for f in [None, Some(2.0)] {
            let (d, inv, est, cir, n) = medir(super::ALTURA_PX, f, usar, g);
            let vaz = vazamento_com(f, usar);
            let rot = f.map_or_else(|| "fabrica".to_string(), |x| format!("{x:.2}"));
            println!(
                "{nome:<26} {rot:>5}  {n:>5}  {d:>9.2}  {inv:>4.2}  {est:>6.3}  {cir:>7.4}  {vaz:>5.2}"
            );
        }
    }
    // ⭐⭐⭐ A DOBRA DURA, nas DUAS leis, sobre a cena do PRODUTO — a pergunta que a mesa do
    // oráculo levantou (lá, com DOIS ossos e a arte `2,4×` mais alta que um osso é longo, o
    // padrão-ouro vira `9`–`11 %` de arte do avesso a `90°`+).
    println!("\n-- A DOBRA DURA: quanta arte vira do AVESSO, nas duas leis --");
    println!("graus/junta   euclidiana(fabrica)   PADRAO-OURO");
    for graus in [25.0_f32, 45.0, 60.0, 90.0] {
        let (sim, e) = cena_dobrada(
            super::ALTURA_PX,
            None,
            ph2d_poly2d::GridOptions::default(),
            graus,
        );
        let (sm, p2l, pele) = campo_da_cena(&sim, e);
        let inv = |usar: bool| -> f64 {
            let mut w = pele.scratch();
            let pos: Vec<[f64; 2]> = sm
                .mesh
                .rest
                .iter()
                .enumerate()
                .map(|(v, &q)| {
                    let p = p2l.apply(q);
                    if usar {
                        pele.point_with(p, sm.pesos_de(v), &mut w)
                    } else {
                        pele.point(p, &mut w)
                    }
                })
                .collect();
            let rep: Vec<[f64; 2]> = sm.mesh.rest.iter().map(|&q| p2l.apply(q)).collect();
            let (mut mau, mut tot) = (0.0, 0.0);
            for t in &sm.mesh.tris {
                let a = |q: &[[f64; 2]]| {
                    let (x, y, z) = (q[t[0] as usize], q[t[1] as usize], q[t[2] as usize]);
                    (y[0] - x[0]) * (z[1] - x[1]) - (y[1] - x[1]) * (z[0] - x[0])
                };
                let (s0, s1) = (a(&rep), a(&pos));
                tot += s0.abs();
                if s0 * s1 <= 0.0 {
                    mau += s0.abs();
                }
            }
            100.0 * mau / tot
        };
        println!("{graus:<13} {:>17.2}% {:>13.2}%", inv(false), inv(true));
    }

    println!("\n-- BBW: o que a DOBRA compra e o que ela custa (a partir do REPOUSO) --");
    println!("graus  esticao_max  esticao_p99  circulo  faceta_px");
    for graus in [0.0_f32, 6.0, 10.0, 15.0, 25.0, 35.0, 45.0] {
        let (sim, e) = cena_dobrada(
            super::ALTURA_PX,
            None,
            ph2d_poly2d::GridOptions::default(),
            graus,
        );
        let (sm, p2l, pele) = campo_da_cena(&sim, e);
        let posadas = posadas_da_cena(&sm, p2l, &pele);
        let mut razoes: Vec<f64> = Vec::new();
        for t in &sm.mesh.tris {
            for k in 0..3 {
                let (i, j) = (t[k] as usize, t[(k + 1) % 3] as usize);
                let r = (sm.mesh.rest[i][0] - sm.mesh.rest[j][0])
                    .hypot(sm.mesh.rest[i][1] - sm.mesh.rest[j][1]);
                let a = (posadas[i][0] - posadas[j][0]).hypot(posadas[i][1] - posadas[j][1]);
                if r > 0.0 {
                    razoes.push(a * f64::from(PPM) / r);
                }
            }
        }
        razoes.sort_by(f64::total_cmp);
        let (maxi, p99) = (
            *razoes.last().expect("ha' arestas"),
            razoes[razoes.len() * 99 / 100],
        );
        let mut w2 = pele.scratch();
        let faceta = ph2d_poly2d::deviation_attrs(
            &sm.mesh,
            &posadas,
            &sm.pesos,
            sm.ossos(),
            &mut |q, pesos| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut w2),
        ) * PX_POR_METRO;
        // ⛔ O CONTROLO: em repouso o mapa E' a identidade, e as quatro colunas dizem-no.
        println!(
            "{graus:<6} {maxi:>11.4} {p99:>12.4} {:>8.4} {faceta:>10.3}",
            circulo_dobrado(graus)
        );
    }
    println!("\n-- BBW: GRADUADA contra UNIFORME, a contagem de pecas semelhante --");
    for (fine, coarse) in [
        (10.0, 26.0),
        (18.0, 18.0),
        (7.0, 18.0),
        (12.5, 12.5),
        (5.0, 13.0),
        (9.0, 9.0),
    ] {
        let gg = ph2d_poly2d::GridOptions {
            fine,
            coarse,
            ..ph2d_poly2d::GridOptions::default()
        };
        let (d, _inv, est, cir, n) = medir(super::ALTURA_PX, None, true, gg);
        let rot = if (fine - coarse).abs() < 1e-9 {
            "UNIFORME"
        } else {
            "graduada"
        };
        println!(
            "{rot} fine={fine:<5} coarse={coarse:<5} pecas={n:<6} faceta={d:>6.2} estica={est:.3} circulo={cir:.4}"
        );
    }
    println!("\n-- BBW: a faceta contra a DENSIDADE da malha do bind --");
    for (fine, coarse) in [
        (10.0, 26.0),
        (7.0, 18.0),
        (5.0, 13.0),
        (3.5, 9.0),
        (2.5, 6.5),
    ] {
        let gg = ph2d_poly2d::GridOptions {
            fine,
            coarse,
            ..ph2d_poly2d::GridOptions::default()
        };
        let (d, inv, est, cir, n) = medir(super::ALTURA_PX, None, true, gg);
        println!(
            "fine={fine:<5} coarse={coarse:<5} pecas={n:<6} faceta={d:>6.2} inv={inv:.2} estica={est:.3} circulo={cir:.4}"
        );
    }
}
