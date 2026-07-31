//! **A TINTA QUE O PERCURSO DERRUBA** — o defeito que a §22.6 rotulou de "saturação" e não era.
//!
//! ⚠️ **Filho de [`super`] de propósito:** o oráculo é o mesmo `true_area`, e as fixtures (`screen`,
//! `art`, `BLACK`) são as do binning. Um módulo irmão precisaria de uma segunda cópia da referência,
//! e duas referências para uma pergunta é como elas divergem.

use super::*;
use crate::binning::bin_segments;

/// 📏 **SONDA — a SATURAÇÃO aberta: de que o `1 − exp(−τ)` está feito no pixel que mais erra.**
///
/// ⚠️ **O defeito 3a, e a hipótese que ela existe para testar.** Em `hardness = 1` o perfil é um
/// degrau, então a resposta certa é **pura área** e o `1 − exp(−τ)` deveria valer 1 em todo pixel
/// tocado. Ele não vale: medido, 16-27/255 de falta. E a suspeita não é o `F_MAX` — é que a
/// cobertura está sendo atenuada **DUAS vezes pela mesma informação de borda**: uma no `edge`
/// (a área) e outra no `τ`, cujo arco encurta perto da silhueta pelo mesmo motivo geométrico.
///
/// Ela imprime, no pior pixel de cada cena, o `τ` cru e a saturação que ele produz — sem isso
/// "consertar a saturação" é escolher entre `F_MAX`, `SUB` e o empurrão do `p_eval` no escuro.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_the_saturation_is_made_of() {
    const N: u32 = 64;
    let (w, h) = (96.0_f32, 96.0_f32);
    let sc = screen(w, h);
    let cenas: [(&str, crate::pack::FlipGpuData); 3] = [
        (
            "flanco RETO (a borda mais simples que existe)",
            art(&[(&[[12.0, 48.0], [84.0, 48.0]], 14.0, false, BLACK)]),
        ),
        (
            "PONTA aguda",
            art(&[(&[[20.0, 20.0], [70.0, 62.0]], 5.0, false, BLACK)]),
        ),
        (
            "QUINA externa",
            art(&[(
                &[[16.0, 16.0], [80.0, 80.0], [48.0, 80.0], [48.0, 16.0]],
                9.0,
                false,
                BLACK,
            )]),
        ),
    ];
    // ⚠️ **O CONTROLE: o mesmo `sd` no meio do FLANCO.** Sem ele "τ = 0 num pixel raso" não
    // distingue *a lei erra na borda* de *a lei erra perto de uma PONTA*. Num traço inclinado o
    // `sd` assume valores arbitrários ao longo do flanco (num horizontal ele só cai em ±0,5, que é
    // por que o pior pixel das três cenas caiu numa tampa).
    {
        let g = art(&[(&[[20.0, 20.0], [70.0, 62.0]], 9.0, false, BLACK)]);
        let bins = bin_segments(&g, &sc, 16);
        let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
        println!("  CONTROLE — flanco de um traco inclinado (t estritamente interno):");
        let mut vistos = 0;
        for y in 0..h as u32 {
            for x in 0..w as u32 {
                let p = [x as f32 + 0.5, y as f32 + 0.5];
                let Some(ti) = bins.tile_of_pixel(p[0], p[1]) else {
                    continue;
                };
                let run = bins.segs_of(ti);
                if run.is_empty() {
                    continue;
                }
                let seg = run[0];
                let (pa, pb) = (g.points[seg.a as usize], g.points[seg.b as usize]);
                let (sa, sb) = (sc.point_px(pa.pos), sc.point_px(pb.pos));
                let (t, _, _) = closest_on_seg(p, sa, sb);
                if !(0.3..=0.7).contains(&t) {
                    continue;
                }
                let Some(sl) = stroke_silhouette(run, &g, &sc, style.tip, p) else {
                    continue;
                };
                if !(0.30..=0.49).contains(&sl.sd) || vistos >= 4 {
                    continue;
                }
                vistos += 1;
                let real = true_area(run, &g, &sc, style.tip, p, N);
                let cover = stroke_deposit(run, &g, &sc, p).map_or(0.0, |d| d.cover);
                println!(
                    "      sd={:+.3}  area={real:.4}  edge={:.4}  cover={cover:.4}  em ({:.1}, {:.1})",
                    sl.sd,
                    sl.planes.coverage(),
                    p[0],
                    p[1]
                );
            }
        }
    }
    println!(
        "  cena                                     sd      area    edge     tau     sat    cover"
    );
    for (nome, g) in &cenas {
        let bins = bin_segments(g, &sc, 16);
        let (mut pior, mut onde) = (0.0_f32, [0.0_f32, 0.0]);
        for y in 0..h as u32 {
            for x in 0..w as u32 {
                let p = [x as f32 + 0.5, y as f32 + 0.5];
                let Some(ti) = bins.tile_of_pixel(p[0], p[1]) else {
                    continue;
                };
                let run = bins.segs_of(ti);
                if run.is_empty() {
                    continue;
                }
                let style = crate::tau::StrokeStyle::of(&g.strokes[run[0].stroke as usize]);
                let real = true_area(run, g, &sc, style.tip, p, N);
                let cover = stroke_deposit(run, g, &sc, p).map_or(0.0, |d| d.cover);
                let d = (cover - real).abs();
                if d > pior {
                    pior = d;
                    onde = p;
                }
            }
        }
        let ti = bins.tile_of_pixel(onde[0], onde[1]).expect("dentro");
        let run = bins.segs_of(ti);
        let style = crate::tau::StrokeStyle::of(&g.strokes[run[0].stroke as usize]);
        let s = stroke_silhouette(run, g, &sc, style.tip, onde).expect("tocado");
        let real = true_area(run, g, &sc, style.tip, onde, N);
        // O MESMO empurrão que o `stroke_deposit` aplica — a sonda tem de perguntar onde ele
        // pergunta, senão mede um `τ` que o produto nunca computou.
        let p_eval = if s.sd > -0.5 && s.dist > 1e-6 {
            let f = (s.sd + 0.5) * 0.5 / s.dist;
            [
                onde[0] + (s.near[0] - onde[0]) * f,
                onde[1] + (s.near[1] - onde[1]) * f,
            ]
        } else {
            onde
        };
        let tau = crate::tau::stroke_tau(run, g, &sc, style, p_eval).map_or(0.0, |i| i.tau);
        let cover = stroke_deposit(run, g, &sc, onde).map_or(0.0, |d| d.cover);
        println!(
            "  {nome:40} {:+.3}  {real:.4}  {:.4}  {tau:6.3}  {:.4}  {cover:.4}   em ({:.1}, {:.1})",
            s.sd,
            s.planes.coverage(),
            1.0 - (-tau).exp(),
            onde[0],
            onde[1]
        );
    }
}

/// 📏 **SONDA — quanta tinta o percurso DERRUBA, e onde.**
///
/// ⚠️ **O defeito 3a não era saturação.** O `cover` não fica *baixo* nesses pixels: fica **ZERO**,
/// com `stroke_tau` devolvendo `None`. Mecanismo, aberto no pixel: perto de uma tampa ou de uma
/// junta o suporte do integrando encolhe (o pico está EM CIMA do extremo do segmento) — medido,
/// **0,121 px** de suporte contra um passo de quadratura de **0,35** ⇒ a regra do ponto médio não
/// enxerga nada, e a integral inteira dá 0. No FLANCO, com o MESMO `sd`, `cover == área` a quatro
/// casas: a lei está certa, a grade é que não resolve a janela.
///
/// Esta sonda mede o ALCANCE do defeito — quantos pixels e quanto — porque a cura (resolver a
/// janela em vez do segmento) muda os números do miolo também, e esse preço só se paga contra um
/// número.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_how_much_ink_the_walk_drops() {
    const N: u32 = 64;
    let (w, h) = (96.0_f32, 96.0_f32);
    let sc = screen(w, h);
    let cenas: [(&str, crate::pack::FlipGpuData); 3] = [
        (
            "traco reto (2 tampas)",
            art(&[(&[[12.0, 48.0], [84.0, 48.0]], 14.0, false, BLACK)]),
        ),
        (
            "L (2 tampas + 2 juntas)",
            art(&[(
                &[[16.0, 16.0], [80.0, 80.0], [48.0, 80.0], [48.0, 16.0]],
                9.0,
                false,
                BLACK,
            )]),
        ),
        (
            "zigue-zague (24 juntas)",
            art(&[(
                &(0..24)
                    .map(|k| [20.0 + k as f32 * 2.5, if k % 2 == 0 { 40.0 } else { 52.0 }])
                    .collect::<Vec<_>>(),
                6.0,
                false,
                BLACK,
            )]),
        ),
    ];
    println!(
        "  cena                        px c/ tinta   px DERRUBADOS   pior queda   soma perdida"
    );
    for (nome, g) in &cenas {
        let bins = bin_segments(g, &sc, 16);
        let (mut com_tinta, mut zerados, mut pior, mut soma) = (0_u32, 0_u32, 0.0_f32, 0.0_f32);
        for y in 0..h as u32 {
            for x in 0..w as u32 {
                let p = [x as f32 + 0.5, y as f32 + 0.5];
                let Some(ti) = bins.tile_of_pixel(p[0], p[1]) else {
                    continue;
                };
                let run = bins.segs_of(ti);
                if run.is_empty() {
                    continue;
                }
                let style = crate::tau::StrokeStyle::of(&g.strokes[run[0].stroke as usize]);
                let real = true_area(run, g, &sc, style.tip, p, N);
                if real <= 0.0 {
                    continue;
                }
                com_tinta += 1;
                let cover = stroke_deposit(run, g, &sc, p).map_or(0.0, |d| d.cover);
                if cover <= 0.0 && real > 0.02 {
                    if zerados == 0 {
                        let sl = stroke_silhouette(run, g, &sc, style.tip, p).expect("tocado");
                        println!(
                            "      1o ofensor ({:.1}, {:.1}): sd={:+.4} dist={:.4} area={real:.4}                              segs no tile={}",
                            p[0],
                            p[1],
                            sl.sd,
                            sl.dist,
                            run.len()
                        );
                    }
                    zerados += 1;
                    pior = pior.max(real);
                    soma += real;
                }
            }
        }
        println!(
            "  {nome:28} {com_tinta:6}     {zerados:8}      {:7.2}/255   {:8.2} px",
            pior * 255.0,
            soma
        );
    }
}

/// ⭐ **A CURA DO 3a, PINADA** — o que era um pin do DEFEITO virou o guard da correção.
///
/// A grade da quadratura passou a resolver a **JANELA** em vez do segmento (doc 12 §22.10), e as
/// duas metades que o pin antigo media trocaram de valor: o flanco continua exato e **a tampa
/// deixou de ser zero**.
///
/// ⚠️ **O flanco tem de ser INCLINADO.** Num traço horizontal o `sd` do flanco cai exatamente em
/// ±0,5 — nunca há pixel meio-coberto ali —, então uma fixture horizontal mede `área = 0` e o gate
/// afirma o vazio. É também a razão de os piores pixels das sondas terem caído todos numa TAMPA.
#[test]
fn the_walk_no_longer_drops_the_ink_at_a_cap() {
    let sc = screen(96.0, 96.0);
    let ver = |g: &crate::pack::FlipGpuData, p: [f32; 2]| {
        let bins = bin_segments(g, &sc, 16);
        let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
        let ti = bins.tile_of_pixel(p[0], p[1]).expect("dentro");
        let run = bins.segs_of(ti);
        let real = true_area(run, g, &sc, style.tip, p, 64);
        let cover = stroke_deposit(run, g, &sc, p).map_or(0.0, |d| d.cover);
        (real, cover)
    };
    // (1) O FLANCO segue exato — é o controle que separa *a lei erra na borda* de *a lei erra
    // perto de um EXTREMO*, e sem ele o número da outra metade não diz nada.
    let inclinado = art(&[(&[[20.0, 20.0], [70.0, 62.0]], 9.0, false, BLACK)]);
    let (real_flanco, cover_flanco) = ver(&inclinado, [47.5, 46.5]);
    assert!(
        real_flanco > 0.0 && (cover_flanco - real_flanco).abs() < 2.0 / 255.0,
        "o flanco deveria ser exato: area {real_flanco:.4} vs cover {cover_flanco:.4}"
    );
    // (2) A TAMPA deposita. Antes da cura este pixel media EXATAMENTE zero contra uma área de
    // 0,1028 — 26,21/255 de tinta derrubada.
    let reto = art(&[(&[[12.0, 48.0], [84.0, 48.0]], 14.0, false, BLACK)]);
    let (real_tampa, cover_tampa) = ver(&reto, [87.5, 41.5]);
    assert!(
        real_tampa > 0.05,
        "a fixture perdeu a tampa: area {real_tampa:.4} (o gate mediria o vazio)"
    );
    assert!(
        cover_tampa > 0.02,
        "⭐ A TINTA DA TAMPA VOLTOU A SER DERRUBADA: cover {cover_tampa:.4} contra area \
         {real_tampa:.4}. A grade voltou a ser ancorada no SEGMENTO?"
    );
}

/// ⭐ **O RESÍDUO QUE A LEI DE ÁREA CRIOU, pinado com o número — e ele NÃO é regressão.**
///
/// A lei de área (§22.7) enxerga um pixel cuja **QUINA** entra na silhueta mesmo com o centro a mais
/// de meio pixel de distância (`sd` até `√2/2`). O `p_eval` — que escolhe onde amostrar o PERFIL —
/// é 1-D ao longo da normal, e a derivação dele (*a parte coberta é `v ∈ [sd − ½, 0]`*) tem
/// intervalo **vazio** quando `sd > ½`: ele pousa FORA da silhueta, `τ` sai 0 e o depósito devolve
/// `None`.
///
/// ⚠️ **Não é regressão:** com a lei antiga esses pixels tinham `edge = 0,5 − sd ≤ 0` e eram
/// descartados no early-out — derrubados do mesmo jeito, só que sem ninguém saber. O que a lei de
/// área fez foi **tornar visível** a discordância entre a cobertura 2-D e a amostra 1-D.
///
/// Medido: **13 pixels de 1115** num zigue-zague de 24 juntas, área ≤ 2,9% (14,94/255).
///
/// ⚠️ **E em 2026-07-31 a pergunta foi FECHADA por medição, não por escolha:** as duas curas que a
/// §22.10 nomeou (capar o alcance dos planos; mover o `p_eval` para dentro) ganharam uma terceira,
/// melhor que as duas — amostrar no **CENTROIDE** da parte coberta, derivado do mesmo polígono da lei
/// de área. Ela foi construída e o oráculo supersampleado a **reprovou**: empate nos flancos, pior
/// nas junções (96,38 contra 61,86/255 de pior caso). Detalhe e tabela em
/// `measure_which_profile_sample_point_is_closer_to_the_truth`.
///
/// ⚠️ **O que aquela medição estabeleceu é maior que o veredito de uma cura:** o pior erro da lei que
/// SHIPA contra a verdade é **22 a 62/255**, e este resíduo vale **≤ 14,94**. *Ele é menor que o erro
/// da aproximação que o curaria.* Enquanto o ponto de amostra for UM ponto, mexer nele troca um
/// artefato pequeno e conhecido por um maior e difuso — a cura de verdade é atacar a aproximação
/// (supersamplear o perfil, ou um segundo tap), que é outra wave, com preço de device próprio.
#[test]
fn the_area_law_can_claim_a_corner_the_profile_cannot_sample_and_this_is_its_number() {
    let sc = screen(96.0, 96.0);
    let g = art(&[(
        &(0..24)
            .map(|k| [20.0 + k as f32 * 2.5, if k % 2 == 0 { 40.0 } else { 52.0 }])
            .collect::<Vec<_>>(),
        6.0,
        false,
        BLACK,
    )]);
    let bins = bin_segments(&g, &sc, 16);
    let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
    let p = [17.5_f32, 58.5];
    let ti = bins.tile_of_pixel(p[0], p[1]).expect("dentro");
    let run = bins.segs_of(ti);
    let sl = stroke_silhouette(run, &g, &sc, style.tip, p).expect("tocado");
    // A premissa declarada: o centro está FORA de meio pixel, então só a quina entra.
    assert!(
        sl.sd > 0.5 && sl.sd < core::f32::consts::FRAC_1_SQRT_2,
        "a fixture perdeu o fenomeno: sd = {} (precisa de meio-pixel a raiz(2)/2)",
        sl.sd
    );
    assert!(
        sl.planes.coverage() > 0.01,
        "a lei de area deveria enxergar a quina: {}",
        sl.planes.coverage()
    );
    assert!(
        stroke_deposit(run, &g, &sc, p).is_none(),
        "⭐ O RESIDUO DA QUINA FOI CORRIGIDO — atualize a nota da §22.10 do doc 12 e este gate."
    );
}

/// 📏 **SONDA — a borda macia ENDURECE quando o traço passa por cima de si mesmo?**
///
/// ⚠️ **É a pergunta do item 5 da fila (a terceira lei, o `Soft` do Krita), feita ao NOSSO motor
/// antes de portar lei nenhuma.** O doc 12 §2.4 mediu o defeito no Painter, cujo depósito é um
/// PRODUTO por-dab; o percurso não tem dabs — ele é `α = 1 − exp(−τ)` com `τ` integral. Mas a
/// álgebra é a mesma família: uma TAXA rumo a um teto. Se `τ` dobra a cada passada, a faixa em que
/// `α` sobe de 10% a 90% encolhe, e isso É o endurecimento.
///
/// A fixture é o gesto que o defeito exige — ir e voltar sobre a MESMA linha dentro de UM traço —,
/// com o pincel mais macio da faixa. Mede a largura da banda perpendicular.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_whether_a_soft_edge_hardens_within_one_stroke() {
    let (w, h) = (128.0_f32, 64.0_f32);
    let sc = screen(w, h);
    println!("  passadas | dureza |  alfa max |  banda HOJE  |  alfa max  |  banda com a 3a LEI");
    for hardness in [0.0_f32, 0.2, 0.5] {
        for passadas in [1_u32, 3, 15] {
            // Vai-e-volta sobre a MESMA linha, dentro de um traço só.
            let mut pts: Vec<[f32; 2]> = Vec::new();
            for k in 0..passadas {
                let (a, b) = if k % 2 == 0 {
                    (24.0, 104.0)
                } else {
                    (104.0, 24.0)
                };
                if k == 0 {
                    pts.push([a, 32.0]);
                }
                pts.push([b, 32.0]);
            }
            let mut g = art(&[(&pts, 18.0, false, BLACK)]);
            g.strokes[0].hardness = hardness;
            let bins = bin_segments(&g, &sc, 16);
            // Perfil perpendicular no MEIO do traço (longe das tampas).
            let coluna: Vec<f32> = (0..h as u32)
                .map(|y| {
                    let p = [64.5_f32, y as f32 + 0.5];
                    crate::binning::walk_pixel(&bins, &g, &sc, p)[3]
                })
                .collect();
            let amax = coluna.iter().copied().fold(0.0_f32, f32::max);
            // A banda: distância entre os cruzamentos de 10% e 90% do máximo, num lado só.
            let cruza = |alvo: f32| -> f32 {
                for y in 0..coluna.len() - 1 {
                    let (a, b) = (coluna[y], coluna[y + 1]);
                    if a < alvo && b >= alvo {
                        return y as f32 + (alvo - a) / (b - a);
                    }
                }
                0.0
            };
            let banda = cruza(0.9 * amax) - cruza(0.1 * amax);
            // ⚠️ **O que a TERCEIRA LEI daria aqui, computado na sonda e NÃO no produto.** O `Soft`
            // limita cada pixel pela cobertura do PRÓPRIO dab ali; no contínuo isso é o perfil
            // avaliado na distância ao caminho — `dn = dist/r`, com `r = dist − sd`. É um campo
            // LISO, então a ressalva do doc 12 §2.4 (*"o ponto fixo é o `max` que deu beading"*)
            // **não alcança o percurso**: o beading era estrutura por-dab ficando à vista, e aqui
            // não há dabs.
            let teto: Vec<f32> = (0..h as u32)
                .map(|y| {
                    let p = [64.5_f32, y as f32 + 0.5];
                    let Some(ti) = bins.tile_of_pixel(p[0], p[1]) else {
                        return 0.0;
                    };
                    let run = bins.segs_of(ti);
                    if run.is_empty() {
                        return 0.0;
                    }
                    let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
                    let Some(sl) = stroke_silhouette(run, &g, &sc, style.tip, p) else {
                        return 0.0;
                    };
                    let r = (sl.dist - sl.sd).max(1e-4);
                    let w = crate::tau::dab_weight(sl.dist / r, hardness);
                    let cov = stroke_deposit(run, &g, &sc, p).map_or(0.0, |d| d.cover);
                    cov.min(w * sl.planes.coverage())
                })
                .collect();
            let tmax = teto.iter().copied().fold(0.0_f32, f32::max);
            let cruza_t = |alvo: f32| -> f32 {
                for y in 0..teto.len() - 1 {
                    let (a, b) = (teto[y], teto[y + 1]);
                    if a < alvo && b >= alvo {
                        return y as f32 + (alvo - a) / (b - a);
                    }
                }
                0.0
            };
            let banda_t = cruza_t(0.9 * tmax) - cruza_t(0.1 * tmax);
            println!(
                "  {passadas:8} | {hardness:6} |  {amax:.4}   |  {banda:.3}       |  {tmax:.4}  \
                 |  {banda_t:.3}"
            );
        }
    }
}

/// ⭐ **A BORDA MACIA ENDURECE SOB AUTO-SOBREPOSIÇÃO, E ESTE É O NÚMERO** — o item 5 da fila, medido
/// no percurso.
///
/// ⚠️ **Descrição, não veredito.** Acumular ao passar por cima é o que tinta faz, e é o que o
/// build-up do GIMP entrega; a terceira lei (`Soft` do Krita) é um MODO, não um conserto. O gate
/// existe para que o número não precise ser re-derivado quando a decisão de produto vier.
///
/// A álgebra do percurso é a mesma família: `α = 1 − exp(−τ)` é uma TAXA rumo a um teto, então
/// dobrar `τ` a cada passada encolhe a faixa em que `α` sobe de 10% a 90%.
#[test]
fn a_soft_edge_hardens_when_the_stroke_crosses_itself_and_this_is_its_number() {
    let (w, h) = (128.0_f32, 64.0_f32);
    let sc = screen(w, h);
    let banda = |passadas: u32| -> f32 {
        let mut pts: Vec<[f32; 2]> = vec![[24.0, 32.0]];
        for k in 0..passadas {
            pts.push([if k % 2 == 0 { 104.0 } else { 24.0 }, 32.0]);
        }
        let mut g = art(&[(&pts, 18.0, false, BLACK)]);
        g.strokes[0].hardness = 0.0;
        let bins = bin_segments(&g, &sc, 16);
        let col: Vec<f32> = (0..h as u32)
            .map(|y| crate::binning::walk_pixel(&bins, &g, &sc, [64.5, y as f32 + 0.5])[3])
            .collect();
        let amax = col.iter().copied().fold(0.0_f32, f32::max);
        let cruza = |alvo: f32| -> f32 {
            for y in 0..col.len() - 1 {
                if col[y] < alvo && col[y + 1] >= alvo {
                    return y as f32 + (alvo - col[y]) / (col[y + 1] - col[y]);
                }
            }
            0.0
        };
        cruza(0.9 * amax) - cruza(0.1 * amax)
    };
    let (uma, quinze) = (banda(1), banda(15));
    // A premissa: com o pincel mais macio a borda de UMA passada é larga. Sem isto o gate mede a
    // razão entre dois números duros e não diz nada sobre maciez.
    assert!(
        uma > 3.0,
        "a fixture perdeu a maciez: banda de uma passada {uma:.3} px"
    );
    assert!(
        quinze < uma * 0.6,
        "⭐ O ENDURECIMENTO MUDOU — uma passada {uma:.3} px, quinze {quinze:.3}. Se a terceira lei \
         entrou, atualize a nota da §22.11 do doc 12; se nao, algo mexeu na lei da tinta."
    );
}

/// 📏 **SONDA — o ORÁCULO: qual das duas leis está mais perto da tinta que o pixel de fato tem.**
///
/// ⚠️ **Ela existe porque o preço sozinho não decide nada.** A sonda irmã mede *quanto* as duas leis
/// discordam; esta mede *qual acerta*. Sem ela, "a lei nova move 91,93/255 num pixel" é uma
/// acusação contra a lei nova — e pode ser exatamente o contrário.
///
/// A verdade é a definição do filtro-caixa: **a média da tinta sobre o pixel**, por supersampling
/// `N×N` (irmã do `true_area` do `aa_tests`, que faz o mesmo para a ÁREA). Cada sub-amostra é um
/// pixel infinitesimal ⇒ sem fator de área, só `(1 − exp(−τ)) · fade` avaliado ALI.
///
/// # ⛔ O CENTROIDE FOI CONSTRUÍDO E REJEITADO POR ESTA SONDA (2026-07-31) — não refaça
///
/// A §22.10 nomeou duas curas para o resíduo de quina e chamou as duas de decisão. **Uma terceira
/// foi construída**, e ela parecia estritamente melhor: amostrar o perfil no **CENTROIDE da parte
/// coberta**, derivado do MESMO polígono que a lei de área já recorta (`c_cob = −c_desc·A_desc/A_cob`).
/// Ela fecha o buraco por construção (a região é não-vazia exatamente quando a área é não-nula),
/// não capa o alcance dos planos, e substitui um modelo de FATIA — que erra duas vezes: a extensão
/// do quadrado na normal é `|nx|+|ny|`, não 1, e a densidade ao longo dela é um TRAPÉZIO, não uma
/// constante.
///
/// ⚠️ **O oráculo reprovou.** Contra a média supersampleada, o centroide é empate nos flancos e
/// **pior onde as passagens se sobrepõem**, que é exatamente onde o resíduo vive:
///
/// | cena / dureza | erro médio FATIA | CENTROIDE | pior FATIA | pior CENTROIDE |
/// |---|---|---|---|---|
/// | flanco 0° / 0,8 | 0,89 | 0,87 | 23,07 | 23,07 |
/// | flanco 45° / 0,8 | **2,77** | 2,87 | 28,19 | 28,19 |
/// | zigue-zague / 0,8 | **3,39** | 4,45 | **61,86** | **96,38** |
///
/// O mecanismo: numa junção a região coberta é uma **UNIÃO** de passagens, possivelmente não-convexa,
/// e o centroide dela não representa ninguém — enquanto a fatia, ancorada na normal da passagem MAIS
/// PRÓXIMA, ao menos amostra onde a passagem dominante manda.
///
/// ⚠️ **E o número que fecha o item 3c está NESTA tabela, não na do resíduo:** o pior erro da lei que
/// shipa contra a verdade é **22 a 62/255**, e o resíduo de quina que se queria curar vale
/// **≤ 14,94/255 em 13 pixels de 1115**. *O artefato é menor que o erro da aproximação que o
/// curaria* — mover o ponto de amostra troca um artefato pequeno e conhecido por um maior e difuso.
/// Curar o resíduo de verdade exige atacar a aproximação inteira (supersamplear o perfil, ou um
/// segundo tap), que é outra wave e tem preço de device próprio.
///
/// Uma variante intermediária também foi medida e é pior que as duas: manter a fatia e só corrigir o
/// SUPORTE dela (`r = (|nx|+|ny|)/2` no lugar do ½) move **65,77/255** no zigue-zague e **21,48** até
/// num flanco a 0° — ela herda a densidade uniforme, e a de uma borda a 45° é um triângulo.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_which_profile_sample_point_is_closer_to_the_truth() {
    const N: u32 = 24;
    let sc = screen(96.0, 96.0);
    let cenas: [(&str, crate::pack::FlipGpuData); 3] = [
        (
            "flanco a 0 graus  ",
            art(&[(&[[12.0, 48.0], [84.0, 48.0]], 14.0, false, BLACK)]),
        ),
        (
            "flanco a 45 graus ",
            art(&[(&[[16.0, 16.0], [80.0, 80.0]], 14.0, false, BLACK)]),
        ),
        (
            "zigue-zague (24 j)",
            art(&[(
                &(0..24)
                    .map(|k| [20.0 + k as f32 * 2.5, if k % 2 == 0 { 40.0 } else { 52.0 }])
                    .collect::<Vec<_>>(),
                6.0,
                false,
                BLACK,
            )]),
        ),
    ];
    println!("\n=== QUANTO A LEI QUE SHIPA ERRA (contra a media supersampleada 24x24) ===");
    println!("  cena | dureza | erro MEDIO (/255) | pior pixel (/255)");
    for (nome, g) in &cenas {
        for dureza in [1.0_f32, 0.8, 0.3] {
            let mut g = g.clone();
            for s in &mut g.strokes {
                s.hardness = dureza;
            }
            let bins = bin_segments(&g, &sc, 16);
            let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
            let (mut soma_f, mut n_px) = (0.0_f64, 0_usize);
            let mut pior_f = 0.0_f32;
            for y in 0..96 {
                for x in 0..96 {
                    let p = [x as f32 + 0.5, y as f32 + 0.5];
                    let Some(ti) = bins.tile_of_pixel(p[0], p[1]) else {
                        continue;
                    };
                    let run = bins.segs_of(ti);
                    let Some(sl) = stroke_silhouette(run, &g, &sc, style.tip, p) else {
                        continue;
                    };
                    let edge = sl.planes.coverage();
                    if edge <= 0.0 {
                        continue;
                    }
                    // A VERDADE: a média da tinta sobre o pixel, sub-amostra a sub-amostra.
                    let mut soma = 0.0_f32;
                    for j in 0..N {
                        for i in 0..N {
                            let q = [
                                p[0] - 0.5 + (i as f32 + 0.5) / N as f32,
                                p[1] - 0.5 + (j as f32 + 0.5) / N as f32,
                            ];
                            let Some(ti_q) = bins.tile_of_pixel(q[0], q[1]) else {
                                continue;
                            };
                            soma += crate::tau::stroke_tau(bins.segs_of(ti_q), &g, &sc, style, q)
                                .map_or(0.0, |k| (1.0 - (-k.tau).exp()) * k.fade);
                        }
                    }
                    let verdade = soma / (N * N) as f32;
                    let cobrir = |pe: [f32; 2]| -> f32 {
                        crate::tau::stroke_tau(run, &g, &sc, style, pe)
                            .map_or(0.0, |i| (1.0 - (-i.tau).exp()) * edge.min(1.0) * i.fade)
                    };
                    let pe_fatia = if sl.sd > -0.5 && sl.dist > 1e-6 {
                        let f = (sl.sd + 0.5) * 0.5 / sl.dist;
                        [
                            p[0] + (sl.near[0] - p[0]) * f,
                            p[1] + (sl.near[1] - p[1]) * f,
                        ]
                    } else {
                        p
                    };
                    let e_f = (cobrir(pe_fatia) - verdade).abs() * 255.0;
                    soma_f += f64::from(e_f);
                    pior_f = pior_f.max(e_f);
                    n_px += 1;
                }
            }
            let d = n_px.max(1) as f64;
            println!(
                "  {nome} | {dureza:6.1} | {:17.2} | {pior_f:17.2}",
                soma_f / d
            );
        }
    }
}

/// 📏 **SONDA — a terceira lei ANULA o Self Overlap?** (a pergunta que faltava ao item 5)
///
/// ⚠️ **É um fato novo sobre a decisão, e ele não estava no §22.11.** No percurso, `τ` é uma
/// INTEGRAL sobre todas as passagens do ladrilho — então um traço que cruza a si mesmo acumula por
/// construção: **o Self Overlap não é um toggle aqui, é o motor**. (No rasterizador ele era um bit
/// no `flip.wgsl`, e foi shipado como feature em 2026-07-27.)
///
/// A terceira lei limita cada pixel pela cobertura do PRÓPRIO bico ali. Isso não capa só o
/// endurecimento da borda: capa **a acumulação inteira**, inclusive a do cruzamento. Se o
/// cruzamento deixar de escurecer, a 3ª lei não é *um modo ao lado* do que existe — ela **desliga
/// uma feature que já shipa**, e a decisão de look passa a ter duas metades em vez de uma.
///
/// Um X de UM traço só, opacidade 0,5, **com o `FLAG_SELF_OVERLAP` ligado** (com tinta opaca não há
/// o que acumular — o §2.4 já dizia; e sem a flag o traço é uma passagem só, ganho 1,00×).
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_whether_the_third_law_also_switches_off_the_self_overlap() {
    let sc = screen(64.0, 64.0);
    println!("\n=== A 3a LEI E O SELF OVERLAP (X de UM traco, opacidade 0,5) ===");
    println!(
        "  dureza | braco HOJE | cruzamento HOJE | ganho | braco 3a LEI | cruz 3a LEI | ganho"
    );
    for dureza in [1.0_f32, 0.5, 0.0] {
        // ⚠️ **A forma é a do `crossing_x` do `cover_tests`, e a diferença é load-bearing:** o
        // cruzamento tem de cair no MEIO de duas pernas. Um X desenhado como *vai, volta, sobe*
        // cruza num VÉRTICE, onde as passagens são contíguas — a partição não vê duas, e a sonda
        // mede 1,00× sobre um motor que funciona (foi o que aconteceu na 1ª tentativa).
        let pernas = [[12.0, 12.0], [52.0, 52.0], [52.0, 12.0], [12.0, 52.0]];
        let mut pts: Vec<[f32; 2]> = Vec::new();
        for w in pernas.windows(2) {
            for k in 0..24 {
                let t = k as f32 / 24.0;
                pts.push([
                    w[0][0] + (w[1][0] - w[0][0]) * t,
                    w[0][1] + (w[1][1] - w[0][1]) * t,
                ]);
            }
        }
        pts.push(pernas[3]);
        let mut g = art(&[(&pts, 9.0, false, BLACK)]);
        for p in &mut g.points {
            p.opacity = 0.5;
        }
        for s in &mut g.strokes {
            s.hardness = dureza;
            // ⚠️ **A FLAG, sem a qual a fixture não contém o fenômeno.** O `art` não a liga, e sem
            // ela o percurso trata o traço como UMA passagem: medido, ganho 1,00× em toda dureza —
            // que é o toggle DESLIGADO fazendo o que promete, não a ausência da feature.
            s.flags |= crate::pack::FLAG_SELF_OVERLAP;
        }
        let bins = bin_segments(&g, &sc, 16);
        let style = crate::tau::StrokeStyle::of(&g.strokes[0]);
        // A cobertura de hoje, e a que a 3ª lei daria (o teto do próprio bico ali).
        // ⚠️ **O ALFA do pixel, não o `cover`** — é ele que responde *"o cruzamento escureceu?"*. O
        // `cover` é geometria × tinta; o `opacity` entra DEPOIS dele, no alfa da cor (`tau.rs`: a
        // regra do GP, *um traço a opacity 0,5 não escurece sobre si mesmo*). Ler o `cover` mede a
        // pergunta errada e devolve 1,0000 em tudo.
        let par = |p: [f32; 2]| -> (f32, f32) {
            let hoje = crate::binning::walk_pixel(&bins, &g, &sc, p)[3];
            let Some(ti) = bins.tile_of_pixel(p[0], p[1]) else {
                return (hoje, 0.0);
            };
            let run = bins.segs_of(ti);
            let Some(sl) = stroke_silhouette(run, &g, &sc, style.tip, p) else {
                return (hoje, 0.0);
            };
            let r = (sl.dist - sl.sd).max(1e-4);
            // O teto da 3ª lei, na mesma unidade: a cobertura do próprio bico ali, vezes a
            // opacidade (que é o que um dab isolado depositaria).
            let teto = crate::tau::dab_weight(sl.dist / r, dureza) * 0.5 * sl.planes.coverage();
            (hoje, hoje.min(teto))
        };
        // O BRAÇO: um ponto do meio de uma perna, longe do cruzamento.
        let (braco_h, braco_t) = par([20.5, 20.5]);
        // O CRUZAMENTO: onde as duas passagens se encontram, no meio das duas.
        let (cruz_h, cruz_t) = par([32.5, 32.5]);
        println!(
            "  {dureza:6.1} | {:10.4} | {:15.4} | {:5.2}x | {:12.4} | {:11.4} | {:5.2}x",
            braco_h,
            cruz_h,
            cruz_h / braco_h.max(1e-6),
            braco_t,
            cruz_t,
            cruz_t / braco_t.max(1e-6)
        );
    }
}
