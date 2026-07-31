//! **O ORÇAMENTO DO AJUSTE** — quanto ele custa e como ele reparte o teto.
//!
//! ⚠️ Irmão do [`super`], e o corte é por ORÁCULO: lá o desvio se mede em fração da espessura; aqui
//! em **estabilidade** (um ponto já posto não pode ser re-decidido) e em **custo** (o ajuste roda
//! por frame). Os dois modos de falha — o traço se mexendo sozinho, o frame ficando lento — são
//! invisíveis a uma medição de forma.

use super::super::super::{Vec2, active_smooth, resample_smooth, simplify_to_curve};

/// A mão do artista: um serpenteado com TREMOR, amostrado como a captura amostra.
fn mao(n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|k| {
            let t = k as f32 / (n - 1) as f32;
            let j = ((k * 12347) % 1000) as f32 / 1000.0 - 0.5;
            Vec2::new(
                40.0 + t * 3000.0 + j * 0.6,
                300.0 + 90.0 * (t * 26.0).sin() + 30.0 * (t * 61.0).cos() + j * 0.6,
            )
        })
        .collect()
}

/// 📏 **SONDA — quantas vezes um ponto JÁ POSTO é mexido enquanto a mão continua?**
///
/// ⚠️ **Esta é a medição do report do Enio** (*"o traço deve parar de reconstruir a curva toda; os
/// pontos já postos não devem ser modificados"*), e ela é sobre TEMPO, não sobre uma foto: a sonda
/// irmã compara UM prefixo contra UM total e mede ~1 % da espessura — pequeno. O que o artista vê é
/// esse 1 % **mudando a cada frame**, sobre um traço que ele já considera pronto.
///
/// Roda o pipeline do PRODUTO (`active_smooth` → ajuste) quadro a quadro e conta em quantos deles o
/// conjunto de pontos guardados **atrás do cursor** mudou.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_how_often_an_already_placed_point_is_moved() {
    let espessura = 12.0_f32;
    let (tol, passo) = (0.0025 * espessura, 0.4 * espessura);
    // Quantas amostras atrás do cursor já contam como "posto" — folgado sobre o raio do
    // `active_smooth` (12 iterações no Smoothing default), senão a sonda mediria o alisamento.
    const ATRAS: usize = 40;
    let cru = mao(1200);
    println!("  frames | frames com o passado MEXIDO | pontos mexidos (total) | pior indice");
    let (mut frames, mut mexidos, mut trocas, mut pior) = (0, 0, 0_usize, 0_usize);
    let mut anterior: Vec<usize> = Vec::new();
    for n in 200..=cru.len() {
        let s = active_smooth(&cru[..n], 0.5);
        let keep = simplify_to_curve(&s, tol, passo);
        let corte = n.saturating_sub(ATRAS);
        let passado: Vec<usize> = keep.iter().copied().filter(|&i| i < corte).collect();
        if !anterior.is_empty() {
            let vivos: Vec<usize> = anterior.iter().copied().filter(|&i| i < corte).collect();
            if vivos != passado[..passado.len().min(vivos.len())] {
                mexidos += 1;
                let d = vivos
                    .iter()
                    .zip(passado.iter())
                    .position(|(a, b)| a != b)
                    .unwrap_or(vivos.len().min(passado.len()));
                trocas += 1;
                pior = pior.max(vivos.get(d).copied().unwrap_or(0));
            }
        }
        anterior = keep;
        frames += 1;
    }
    println!("  {frames:6} | {mexidos:27} | {trocas:22} | {pior:11}");
}

/// 📏 **SONDA — o custo do ajuste cresce como com o COMPRIMENTO do traço?**
///
/// ⚠️ **A pergunta que precede triplicar os pontos.** O ajuste guloso roda **por frame** no preview
/// ao vivo (a porta é a mesma do bake, de propósito), e ele é `O(k · n · m)` — pontos guardados ×
/// amostras cruas × segmentos da reconstrução. Triplicar `k` num traço LONGO multiplica os três.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_how_the_fit_cost_grows_with_the_stroke() {
    let espessura = 12.0_f32;
    println!("  fracao  | amostras |  pts  |   ms por ajuste");
    for fracao in [0.02_f32, 0.0025] {
        for n in [240_usize, 600, 1200] {
            // Um serpenteado longo: curvatura o tempo todo, o pior caso honesto.
            let cru: Vec<Vec2> = (0..n)
                .map(|k| {
                    let t = k as f32 / (n - 1) as f32;
                    Vec2::new(
                        40.0 + t * 900.0,
                        200.0 + 70.0 * (t * 9.0).sin() + 20.0 * (t * 23.0).cos(),
                    )
                })
                .collect();
            let tol = fracao * espessura;
            let t0 = std::time::Instant::now();
            let keep = simplify_to_curve(&cru, tol, 0.4 * espessura);
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            println!("  {fracao:7} |  {n:7}  | {:5} |   {ms:8.3}", keep.len());
        }
    }
}

/// ⭐ **O AJUSTE RECONSTRÓI A VIZINHANÇA, NÃO O TRAÇO** — e este é o único gate que pode ver isso,
/// porque a diferença entre as duas versões é **só o relógio**: elas dão o mesmo conjunto de pontos.
///
/// ⚠️ **A medição original não continha o fenômeno.** Ela usou o gancho (duas pernas retas, ~13
/// pontos) e mediu 0,2-0,4 ms; num traço LONGO e serpenteado a versão que reconstruía o traço
/// inteiro a cada inserção custava **27 ms/frame já na tolerância antiga** e 64 na de hoje — e ele
/// roda **por frame** no preview ao vivo.
///
/// A barra é kill de wall-clock com folga de ~7× sobre o medido (0,72 ms), porque o modo de falha
/// que ela vigia é de ORDEM DE GRANDEZA (voltar a reconstruir o traço inteiro custa 90×), não de
/// afinação.
#[test]
fn the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke() {
    let espessura = 12.0_f32;
    let n = 1200_usize;
    let cru: Vec<Vec2> = (0..n)
        .map(|k| {
            let t = k as f32 / (n - 1) as f32;
            Vec2::new(
                40.0 + t * 900.0,
                200.0 + 70.0 * (t * 9.0).sin() + 20.0 * (t * 23.0).cos(),
            )
        })
        .collect();
    let tol = 0.0025 * espessura;
    // A premissa da fixture, declarada: o traço tem de EXIGIR muitos pontos, senão o gate mede um
    // ajuste que nem trabalha.
    let t0 = std::time::Instant::now();
    let keep = simplify_to_curve(&cru, tol, 0.4 * espessura);
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    assert!(
        keep.len() > 60,
        "a fixture nao exige pontos ({} guardados) — o gate mediria o vazio",
        keep.len()
    );
    assert!(
        ms < 5.0,
        "o ajuste custou {ms:.2} ms num traco de {n} amostras — a reconstrucao voltou a ser do \
         TRACO INTEIRO? (medido local: 0,72 ms; global: 64 ms)"
    );
}

/// 📏 **SONDA — num traço LONGO, o COMEÇO perde pontos?** (report do Enio, 2026-07-30)
///
/// ⚠️ **A suspeita foi REFUTADA por esta sonda, e ela fica pelo negativo.** A hipótese era sobre a
/// ordem da fila do ajuste divide-e-conquista (uma PILHA desce pela metade da direita primeiro, e
/// sob um teto global o orçamento acabava todo no FIM). A distribuição medida saiu **uniforme**, o
/// teto **nunca mordia** (376 de 512), e o defeito era outro: o ajuste re-decidia o traço INTEIRO a
/// cada frame. Hoje nem a pilha nem o teto existem — sobrou a sonda, que segue sendo o jeito de ver
/// se algum trecho do traço está sendo mal servido.
///
/// Mede o desvio por TERÇO do traço, em comprimentos crescentes.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_whether_a_long_stroke_starves_its_beginning() {
    let espessura = 12.0_f32;
    let tol = 0.0025 * espessura;
    let passo = 0.4 * espessura;
    let dist_seg = |q: Vec2, a: Vec2, b: Vec2| -> f32 {
        let ab = Vec2::new(b.x - a.x, b.y - a.y);
        let len2 = ab.x * ab.x + ab.y * ab.y;
        let t = if len2 < 1e-12 {
            0.0
        } else {
            (((q.x - a.x) * ab.x + (q.y - a.y) * ab.y) / len2).clamp(0.0, 1.0)
        };
        let (dx, dy) = (q.x - (a.x + ab.x * t), q.y - (a.y + ab.y * t));
        (dx * dx + dy * dy).sqrt()
    };
    println!("  amostras |  pts  | desvio no 1o terco | 2o terco | 3o terco  (% da espessura)");
    for n in [600_usize, 2000, 5000, 9000] {
        let cru: Vec<Vec2> = (0..n)
            .map(|k| {
                let t = k as f32 / (n - 1) as f32;
                Vec2::new(
                    40.0 + t * 3000.0,
                    300.0 + 90.0 * (t * 26.0).sin() + 30.0 * (t * 61.0).cos(),
                )
            })
            .collect();
        let keep = simplify_to_curve(&cru, tol, passo);
        let pts: Vec<Vec2> = keep.iter().map(|&i| cru[i]).collect();
        let prs = vec![1.0_f32; pts.len()];
        let (finais, _) = resample_smooth(&pts, &prs, passo, tol);
        let mut piores = [0.0_f32; 3];
        for (i, q) in cru.iter().enumerate() {
            let mut best = f32::MAX;
            for w in finais.windows(2) {
                best = best.min(dist_seg(*q, w[0], w[1]));
            }
            let terco = (i * 3 / n).min(2);
            piores[terco] = piores[terco].max(best);
        }
        // ⚠️ **A distribuição é a medição direta do report** — "o começo perde pontos" é sobre
        // QUANTOS pontos cada trecho ficou, e o desvio pode esconder isso quando a curvatura é
        // uniforme (a fixture pune todo trecho igual).
        let mut por_terco = [0_usize; 3];
        for &i in &keep {
            por_terco[(i * 3 / n).min(2)] += 1;
        }
        println!(
            "  {n:8} | {:5} |      {:7.2} %       | {:6.2} % | {:6.2} %   | pts por terco              {por_terco:?}",
            keep.len(),
            piores[0] / espessura * 100.0,
            piores[1] / espessura * 100.0,
            piores[2] / espessura * 100.0
        );
    }
}

/// 📏 **SONDA — o COMEÇO do traço se mexe enquanto a mão continua desenhando?**
///
/// ⚠️ **A hipótese da pilha LIFO foi REFUTADA** (a distribuição de pontos sai uniforme,
/// `[166, 165, 164]`), então "o início perde pontos e deforma" tem de ser outra coisa — e
/// *deforma* é a palavra que aponta: o ajuste é **global e roda a cada frame** do preview. Se
/// acrescentar amostras no FIM muda quais pontos ele escolhe no COMEÇO, o artista vê o pedaço já
/// desenhado **se mexer** enquanto ele continua.
///
/// Mede exatamente isso: reconstrói o prefixo sozinho, reconstrói o traço inteiro, e compara as
/// duas sobre o MESMO prefixo.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_whether_the_start_moves_while_the_hand_keeps_drawing() {
    let espessura = 12.0_f32;
    let tol = 0.0025 * espessura;
    let passo = 0.4 * espessura;
    let dist_seg = |q: Vec2, a: Vec2, b: Vec2| -> f32 {
        let ab = Vec2::new(b.x - a.x, b.y - a.y);
        let len2 = ab.x * ab.x + ab.y * ab.y;
        let t = if len2 < 1e-12 {
            0.0
        } else {
            (((q.x - a.x) * ab.x + (q.y - a.y) * ab.y) / len2).clamp(0.0, 1.0)
        };
        let (dx, dy) = (q.x - (a.x + ab.x * t), q.y - (a.y + ab.y * t));
        (dx * dx + dy * dy).sqrt()
    };
    let curva = |cru: &[Vec2]| -> Vec<Vec2> {
        let keep = simplify_to_curve(cru, tol, passo);
        let pts: Vec<Vec2> = keep.iter().map(|&i| cru[i]).collect();
        let prs = vec![1.0_f32; pts.len()];
        resample_smooth(&pts, &prs, passo, tol).0
    };
    println!("  total |  prefixo | pts(prefixo) | pts(total) | o COMECO ANDOU (% da espessura)");
    for n in [1200_usize, 3000, 6000, 12000] {
        let cru: Vec<Vec2> = (0..n)
            .map(|k| {
                let t = k as f32 / (n - 1) as f32;
                Vec2::new(
                    40.0 + t * 3000.0,
                    300.0 + 90.0 * (t * 26.0).sin() + 30.0 * (t * 61.0).cos(),
                )
            })
            .collect();
        let corte = n / 3;
        let prefixo = &cru[..corte];
        let so_prefixo = curva(prefixo);
        let inteiro = curva(&cru);
        // Quanto a curva do prefixo ANDOU quando o resto do traço chegou.
        let mut andou = 0.0_f32;
        for q in &so_prefixo {
            let mut best = f32::MAX;
            for w in inteiro.windows(2) {
                best = best.min(dist_seg(*q, w[0], w[1]));
            }
            andou = andou.max(best);
        }
        println!(
            "  {n:5} |  {corte:6}  |    {:6}    |   {:6}   |     {:7.2} %",
            simplify_to_curve(prefixo, tol, passo).len(),
            simplify_to_curve(&cru, tol, passo).len(),
            andou / espessura * 100.0
        );
    }
}

/// 📏 **SONDA — de que é feito um FRAME do preview ao vivo, por estágio.**
///
/// ⚠️ **A pergunta que decide se "parar de reconstruir a curva toda" ainda vale uma wave.** A lei
/// nova já garante que o RESULTADO não muda (`measure_how_often_an_already_placed_point_is_moved`:
/// 762 → 0 frames), mas o TRABALHO segue sendo refeito do zero a cada frame. Quanto ele custa é o
/// que diz se um cache incremental compra alguma coisa — e ele só é seguro por causa da lei.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_a_live_preview_frame_is_made_of() {
    let espessura = 12.0_f32;
    let (tol, passo) = (0.0025 * espessura, 0.4 * espessura);
    println!("  amostras | active_smooth | ajuste | reamostragem |  total  |  pts");
    for n in [1200_usize, 3000, 9000] {
        let cru = mao(n);
        let t0 = std::time::Instant::now();
        let s = active_smooth(&cru, 0.5);
        let t1 = std::time::Instant::now();
        let keep = simplify_to_curve(&s, tol, passo);
        let t2 = std::time::Instant::now();
        let pts: Vec<Vec2> = keep.iter().map(|&i| s[i]).collect();
        let (finais, _) = resample_smooth(&pts, &vec![1.0; pts.len()], passo, tol);
        let t3 = std::time::Instant::now();
        let ms = |a: std::time::Instant, b: std::time::Instant| (b - a).as_secs_f64() * 1000.0;
        println!(
            "  {n:8} |    {:7.3}    | {:6.3} |    {:7.3}   | {:7.3} | {:5}",
            ms(t0, t1),
            ms(t1, t2),
            ms(t2, t3),
            ms(t0, t3),
            finais.len()
        );
    }
}

/// 📏 **SONDA — a margem de aceitação: quanto ela compra em precisão e quanto custa em pontos.**
///
/// A estimativa do erro é aproximada por construção (a tangente do fim do span sai de um palpite
/// sobre onde o próximo nó cai), então o erro DESENHADO passa da tolerância. Esta sonda varre a
/// margem contra as duas metades que o gate de precisão afirma ao mesmo tempo.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_the_acceptance_margin_buys() {
    let espessura = 12.0_f32;
    let (tol, passo) = (0.02 * espessura, 0.4 * espessura);
    let dist_seg = |q: Vec2, a: Vec2, b: Vec2| -> f32 {
        let ab = Vec2::new(b.x - a.x, b.y - a.y);
        let len2 = ab.x * ab.x + ab.y * ab.y;
        let t = if len2 < 1e-12 {
            0.0
        } else {
            (((q.x - a.x) * ab.x + (q.y - a.y) * ab.y) / len2).clamp(0.0, 1.0)
        };
        let (dx, dy) = (q.x - (a.x + ab.x * t), q.y - (a.y + ab.y * t));
        (dx * dx + dy * dy).sqrt()
    };
    // O gancho da fixture de precisão, verbatim.
    let mut gancho: Vec<Vec2> = (0..90)
        .map(|k| Vec2::new(40.0 + k as f32 * 2.0, 40.0))
        .collect();
    for k in 1..=60 {
        let a = -core::f32::consts::FRAC_PI_2 + k as f32 / 60.0 * core::f32::consts::PI;
        gancho.push(Vec2::new(220.0 + 18.0 * a.cos(), 58.0 + 18.0 * a.sin()));
    }
    for k in 1..=90 {
        gancho.push(Vec2::new(220.0 - k as f32 * 2.0, 76.0));
    }
    let arco: Vec<Vec2> = (0..240)
        .map(|k| {
            let a = k as f32 / 239.0 * core::f32::consts::PI;
            Vec2::new(300.0 + 200.0 * a.cos(), 300.0 + 200.0 * a.sin())
        })
        .collect();
    println!("  margem | gancho (% da espessura) | pts do gancho | pts do arco liso");
    for safety in [1.0_f32, 0.8, 0.65, 0.5] {
        let keep = super::super::fit::fit_tuned(&gancho, tol, passo, safety);
        let pts: Vec<Vec2> = keep.iter().map(|&i| gancho[i]).collect();
        let (finais, _) = resample_smooth(&pts, &vec![1.0; pts.len()], passo, tol);
        let mut pior = 0.0_f32;
        for q in &gancho {
            let mut best = f32::MAX;
            for w in finais.windows(2) {
                best = best.min(dist_seg(*q, w[0], w[1]));
            }
            pior = pior.max(best);
        }
        let n_arco = super::super::fit::fit_tuned(&arco, tol, passo, safety).len();
        println!(
            "  {safety:6.2} |          {:6.2}         |     {:5}     |     {n_arco:5}",
            pior / espessura * 100.0,
            keep.len()
        );
    }
}

/// 📏 **SONDA — a quantas amostras atrás do cursor um ponto CONGELA?**
///
/// A franja provisória é o span sob a caneta (o [`espelho`] olha um span à frente), então a margem
/// não é "dois nós" e sim um comprimento — e quem diz o número é a medição, não eu.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_how_far_behind_the_pen_a_point_freezes() {
    let espessura = 12.0_f32;
    let (tol, passo) = (0.0025 * espessura, 0.4 * espessura);
    let cru = mao(900);
    let mut anterior: Vec<usize> = Vec::new();
    let mut pior = 0_usize;
    for n in 60..=cru.len() {
        let keep = simplify_to_curve(&cru[..n], tol, passo);
        if !anterior.is_empty() {
            // O nó mais ANTIGO que mudou de valor entre os dois frames.
            for (i, (a, b)) in anterior.iter().zip(keep.iter()).enumerate() {
                if a != b {
                    pior = pior.max(n - 1 - anterior[i.saturating_sub(1)]);
                    break;
                }
            }
            if keep.len() < anterior.len() {
                pior = pior.max(n - 1 - anterior[keep.len().saturating_sub(1)]);
            }
        }
        anterior = keep;
    }
    println!("  a re-decisao mais funda ficou a {pior} amostras atras do cursor");
}

/// ⭐ **UM PONTO QUE A MÃO JÁ DEIXOU PARA TRÁS NUNCA É RE-DECIDIDO** — o gate do report do Enio
/// (2026-07-30, terceira rodada: *"os pontos já postos não devem ser modificados"*).
///
/// ⚠️ **É a propriedade, não a amplitude.** A sonda irmã que compara um prefixo contra um total mede
/// ~1 % da espessura e por isso passou por pequena nas duas rodadas anteriores; o que o artista vê é
/// esse 1 % **mudando a cada frame**. Medido no ajuste divide-e-conquista: **762 de 1001 frames**
/// mexiam no passado. A lei nova (guloso da esquerda para a direita) leva isso a **zero por
/// construção**, e é isso que este gate afirma.
///
/// ⚠️ **A folga é medida, e a UNIDADE dela é a do artista: amostras atrás do cursor.** Não são "dois
/// nós" — o [`espelho`] olha um span à frente, então a franja provisória é um COMPRIMENTO. Medido
/// (`measure_how_far_behind_the_pen_a_point_freezes`): a re-decisão mais funda fica a **12
/// amostras** do cursor; a barra é **40**, que é também a folga sobre o raio do `active_smooth`
/// (12 iterações no Smoothing default) — os dois efeitos vivem na mesma franja.
///
/// Mutação que sangra: voltar a fila do ajuste para a partição por pior erro GLOBAL.
#[test]
fn the_fit_never_re_decides_a_point_the_hand_already_left_behind() {
    const FRANJA: usize = 40;
    let espessura = 12.0_f32;
    let (tol, passo) = (0.0025 * espessura, 0.4 * espessura);
    let cru = mao(900);
    let mut anterior: Vec<usize> = Vec::new();
    let mut conferidos = 0_usize;
    for n in 60..=cru.len() {
        let keep = simplify_to_curve(&cru[..n], tol, passo);
        if !anterior.is_empty() {
            let corte = (n - 1).saturating_sub(FRANJA);
            let firme: Vec<usize> = anterior.iter().copied().filter(|&i| i <= corte).collect();
            assert!(
                keep.len() >= firme.len() && keep[..firme.len()] == firme[..],
                "com {n} amostras o ajuste re-decidiu o passado (corte em {corte})\n  \
                 firme: {:?}\n  agora: {:?}",
                &firme[firme.len().saturating_sub(6)..],
                &keep[..firme.len().min(keep.len())]
                    [firme.len().min(keep.len()).saturating_sub(6)..]
            );
            conferidos += firme.len();
        }
        anterior = keep;
    }
    // A premissa da fixture, declarada: sem nós conferidos o gate é verde por vácuo.
    assert!(conferidos > 100_000, "so' {conferidos} nos conferidos");
}

/// ⭐ **E o traço LONGO não paga um teto** — a contagem de pontos é governada pelo piso
/// anti-redundância, que é uma pergunta **por span**, e não por um orçamento global.
///
/// ⚠️ **Um teto global contradiz a lei acima e por isso FOI REMOVIDO** (`MAX_FIT_POINTS`, e com ele
/// o gate `the_cap_costs_quality_everywhere_not_the_start_of_the_stroke`): sob um teto, o mesmo
/// começo de traço recebe orçamentos diferentes conforme o traço cresce — que é exatamente o defeito
/// que o Enio reportou. Não é economia: é que a pergunta não pode ser feita globalmente.
///
/// O que sobra a vigiar é o preço, e ele é uma **RAZÃO entre dois comprimentos**, nunca wall-clock.
///
/// ⚠️ **A primeira versão era um kill de milissegundos e ela reprovou em DEBUG** (21,65 ms contra
/// 1,92 em release): um bar de relógio mede o PERFIL do build, não o código — a lição que o ADR-0124
/// já tinha pago no editor de áudio. O modo de falha real aqui é de FORMA: a varredura do span é
/// `O(span²)` no pior caso, então uma regressão aparece como o custo **deixando de ser linear** no
/// comprimento do traço. Triplicar as amostras tem de triplicar o custo (bar 5×), não multiplicá-lo
/// por nove — e uma razão é imune ao perfil e à carga da máquina.
#[test]
fn a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget() {
    let espessura = 12.0_f32;
    let (tol, passo) = (0.0025 * espessura, 0.4 * espessura);
    // ⚠️ **Pelo pipeline do PRODUTO.** Alimentar o ajuste com o tremor CRU mede uma cena que não
    // existe: o `active_smooth` roda antes dele, sempre. Sem alisar, a mesma fixture pede 3968
    // pontos (o tremor é 10× a tolerância, então quase toda amostra vira nó) — número honesto sobre
    // uma entrada que ninguém entrega.
    let medir = |n: usize| -> (usize, f64) {
        let cru = active_smooth(&mao(n), 0.5);
        let t0 = std::time::Instant::now();
        let keep = simplify_to_curve(&cru, tol, passo);
        (keep.len(), t0.elapsed().as_secs_f64() * 1000.0)
    };
    let (pts_curto, ms_curto) = medir(3000);
    let (pts, ms) = medir(9000);
    println!("  3000 -> {pts_curto} pts / {ms_curto:.2} ms | 9000 -> {pts} pts / {ms:.2} ms");
    // A premissa: o traço TEM de pedir mais pontos que o teto que existia (512), senão o gate mede
    // um caso em que o teto nunca teria mordido.
    assert!(
        pts > 512,
        "a fixture nao passa do teto antigo ({pts} pontos) — o gate mediria o vazio"
    );
    let razao = ms / ms_curto.max(1e-6);
    assert!(
        razao < 5.0,
        "3x as amostras custaram {razao:.2}x (linear seria 3, quadratico 9) — a varredura do span \
         voltou a ser O(span^2)?  {ms_curto:.2} -> {ms:.2} ms"
    );
}
