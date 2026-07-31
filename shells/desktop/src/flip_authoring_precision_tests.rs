//! **A PRECISÃO DA AUTORIA** — quanto do gesto do artista sobrevive até o traço guardado.
//!
//! ⚠️ Irmão do `flip_smooth_resample_tests.rs`, e o corte é por ASSUNTO: lá se mede a reamostragem
//! (o que o render recebe), aqui se mede o **gesto** (o que a mão fez contra o que ficou guardado).
//!
//! Os três estágios, medidos por ablação POR ENTRADA (os knobs, nunca instrumentação):
//! **captura** (`MIN_SAMPLE_PX`) · **smoothing** (`DEFAULT_SMOOTHING`) · **simplificação**.

use super::super::{Vec2, active_smooth, resample_smooth, simplify_rdp, simplify_to_curve};

/// 📏 **SONDA — quão longe o traço GUARDADO fica do que a mão fez.**
///
/// ⚠️ **O report do Enio (2026-07-30, com screenshot e setas) é sobre AUTORIA, não render:**
/// *"poucos pontos são gerados por traço na ferramenta Draw e precisamos de mais precisão"* — as
/// setas apontam da polilinha guardada para onde o traço de fato está.
///
/// ⚠️ **E a constante já foi ajustada DUAS vezes em direções OPOSTAS:** `0,0008` produziu *"muitos
/// pontos muito próximos e até sobrepostos"* (2026-07-18) e `0,05` produz *"poucos pontos"* (hoje).
/// Um terceiro ajuste da mesma constante é o sinal de que o **modelo** está errado, não o valor
/// ([[feedback_a_new_remedy_makes_the_old_one_double_counting]]) — então a sonda mede o que o
/// modelo de fato erra, e não só quantos pontos sobram.
///
/// **A suspeita, escrita antes de medir:** o RDP mede `perp_dist` até a **CORDA RETA**, mas o que
/// se desenha depois é uma **Catmull-Rom** pelos pontos guardados. Simplificador e reconstrutor
/// discordam sobre o que os pontos guardados significam — e a tolerância está sendo cobrada contra
/// uma curva que ninguém desenha.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_how_far_the_stored_stroke_is_from_the_hand() {
    // Um arco de mão: espiral aberta com curvatura variável, amostrada a ~2 px como o produto.
    let cru: Vec<Vec2> = (0..400)
        .map(|k| {
            let t = k as f32 / 399.0;
            let a = t * 2.4 * core::f32::consts::PI;
            let r = 30.0 + 55.0 * t;
            Vec2::new(160.0 + r * a.cos(), 160.0 + r * a.sin())
        })
        .collect();
    let espessura = 12.0_f32;
    println!(
        "  fracao |  pts guardados |  desvio da MAO (px)  | % da espessura | pts finais (pos-resample)"
    );
    for fracao in [0.0008_f32, 0.01, 0.02, 0.05, 0.10] {
        let tol = fracao * espessura;
        let suave = active_smooth(&cru, 0.0);
        let keep = simplify_rdp(&suave, tol);
        let pts: Vec<Vec2> = keep.iter().map(|&i| suave[i]).collect();
        let prs = vec![1.0_f32; pts.len()];
        let (finais, _) = resample_smooth(&pts, &prs, 0.4 * espessura, tol);
        // O desvio que o artista VÊ: a distância de cada amostra crua à polilinha final.
        // ⚠️ Ao SEGMENTO, não à reta infinita — a `perp_dist` do produto é a da reta (é o que o
        // RDP quer), e usá-la aqui mediria distância a prolongamentos que ninguém desenha.
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
        let mut pior = 0.0_f32;
        for q in &cru {
            let mut best = f32::MAX;
            for w in finais.windows(2) {
                best = best.min(dist_seg(*q, w[0], w[1]));
            }
            pior = pior.max(best);
        }
        println!(
            "  {fracao:6} |  {:6}        |      {pior:8.4}        |    {:6.2} %     |  {:6}",
            pts.len(),
            pior / espessura * 100.0,
            finais.len()
        );
    }
}

/// 📏 **SONDA — de que o desvio é feito: SMOOTHING ou SIMPLIFICAÇÃO?**
///
/// ⚠️ **A sonda irmã não continha o fenômeno.** Numa espiral suave e com `smoothing = 0` o desvio
/// no valor que shipa é 0,33 px (2,7% da espessura) — longe do que a foto do Enio mostra. Faltavam
/// as duas coisas que o caso real tem: o **gancho fechado** e o **`DEFAULT_SMOOTHING = 0,5`**.
///
/// Ablação **por ENTRADA** (os knobs, não instrumentação): cada linha desliga um estágio.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_the_stroke_deviation_is_made_of() {
    let espessura = 12.0_f32;
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
    // O GANCHO da foto: entra, curva fechado (raio ~1,5 espessuras) e volta.
    let mut cru: Vec<Vec2> = Vec::new();
    for k in 0..90 {
        cru.push(Vec2::new(40.0 + k as f32 * 2.0, 40.0));
    }
    let (cx, cy, rr) = (220.0_f32, 58.0, 18.0);
    for k in 1..=60 {
        let a = -core::f32::consts::FRAC_PI_2 + k as f32 / 60.0 * core::f32::consts::PI;
        cru.push(Vec2::new(cx + rr * a.cos(), cy + rr * a.sin()));
    }
    for k in 1..=90 {
        cru.push(Vec2::new(220.0 - k as f32 * 2.0, 76.0));
    }
    println!(
        "  cenario                               |  pts  | desvio da MAO (px) | % da espessura"
    );
    // A LEI NOVA, na mesma cena: refinamento contra a curva desenhada.
    for (smoothing, fracao) in [(0.5_f32, 0.05_f32), (0.0, 0.05)] {
        let tol = fracao * espessura;
        let suave = active_smooth(&cru, smoothing);
        let keep = simplify_to_curve(&suave, tol, 0.4 * espessura);
        let pts: Vec<Vec2> = keep.iter().map(|&i| suave[i]).collect();
        let prs = vec![1.0_f32; pts.len()];
        let (finais, _) = resample_smooth(&pts, &prs, 0.4 * espessura, tol);
        let mut pior = 0.0_f32;
        for q in &cru {
            let mut best = f32::MAX;
            for w in finais.windows(2) {
                best = best.min(dist_seg(*q, w[0], w[1]));
            }
            pior = pior.max(best);
        }
        println!(
            "  CONTRA A CURVA (smoothing {smoothing} · sim {fracao})   | {:5} |     {pior:8.4}    \
             |   {:6.2} %",
            pts.len(),
            pior / espessura * 100.0
        );
    }
    let casos: [(&str, f32, f32); 5] = [
        ("PRODUTO (smoothing 0,5 · sim 0,05)", 0.5, 0.05),
        ("so' o SMOOTHING (sim ~0)", 0.5, 0.0008),
        ("so' a SIMPLIFICACAO (smoothing 0)", 0.0, 0.05),
        ("nenhum dos dois", 0.0, 0.0008),
        ("smoothing 0,25 · sim 0,02", 0.25, 0.02),
    ];
    for (nome, smoothing, fracao) in casos {
        let tol = fracao * espessura;
        let suave = active_smooth(&cru, smoothing);
        let keep = simplify_rdp(&suave, tol);
        let pts: Vec<Vec2> = keep.iter().map(|&i| suave[i]).collect();
        let prs = vec![1.0_f32; pts.len()];
        let (finais, _) = resample_smooth(&pts, &prs, 0.4 * espessura, tol);
        let mut pior = 0.0_f32;
        for q in &cru {
            let mut best = f32::MAX;
            for w in finais.windows(2) {
                best = best.min(dist_seg(*q, w[0], w[1]));
            }
            pior = pior.max(best);
        }
        println!(
            "  {nome:36}  | {:5} |     {pior:8.4}       |   {:6.2} %",
            pts.len(),
            pior / espessura * 100.0
        );
    }
}

/// 📏 **SONDA — o que a tolerância compra AGORA que a métrica é honesta, e quanto custa.**
///
/// Com o erro cobrado contra a curva desenhada, o número passa a significar o que diz: *"o traço
/// guardado não se afasta da mão mais que isto"*. A tabela é o joelho + o relógio (o ajuste roda
/// por frame no preview ao vivo, então o custo é parte da decisão).
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_the_tolerance_buys_now_that_the_metric_is_honest() {
    let espessura = 12.0_f32;
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
    // O gancho da foto + um arco liso, para a tabela mostrar as DUAS pontas da queixa.
    let mut gancho: Vec<Vec2> = Vec::new();
    for k in 0..90 {
        gancho.push(Vec2::new(40.0 + k as f32 * 2.0, 40.0));
    }
    for k in 1..=60 {
        let a = -core::f32::consts::FRAC_PI_2 + k as f32 / 60.0 * core::f32::consts::PI;
        gancho.push(Vec2::new(220.0 + 18.0 * a.cos(), 58.0 + 18.0 * a.sin()));
    }
    for k in 1..=90 {
        gancho.push(Vec2::new(220.0 - k as f32 * 2.0, 76.0));
    }
    let liso: Vec<Vec2> = (0..240)
        .map(|k| {
            let t = k as f32 / 239.0;
            Vec2::new(40.0 + t * 300.0, 60.0 + 40.0 * (t * 1.6).sin())
        })
        .collect();
    println!("  fracao |  GANCHO pts/desvio%  |  LISO pts/desvio%  |  ms por ajuste (gancho)");
    for fracao in [0.05_f32, 0.02, 0.01, 0.005, 0.0025, 0.001] {
        let tol = fracao * espessura;
        let mut linha = format!("  {fracao:6} |");
        let mut ms = 0.0_f64;
        for (idx, cru) in [&gancho, &liso].iter().enumerate() {
            let suave = active_smooth(cru, 0.5);
            let t0 = std::time::Instant::now();
            let keep = simplify_to_curve(&suave, tol, 0.4 * espessura);
            if idx == 0 {
                ms = t0.elapsed().as_secs_f64() * 1000.0;
            }
            let pts: Vec<Vec2> = keep.iter().map(|&i| suave[i]).collect();
            let prs = vec![1.0_f32; pts.len()];
            let (finais, _) = resample_smooth(&pts, &prs, 0.4 * espessura, tol);
            let mut pior = 0.0_f32;
            for q in cru.iter() {
                let mut best = f32::MAX;
                for w in finais.windows(2) {
                    best = best.min(dist_seg(*q, w[0], w[1]));
                }
                pior = pior.max(best);
            }
            linha.push_str(&format!(
                "   {:4} / {:5.2}%     |",
                pts.len(),
                pior / espessura * 100.0
            ));
        }
        println!("{linha}   {ms:.3}");
    }
}

/// ⭐ **O TRAÇO GUARDADO SEGUE A MÃO — e o oráculo é a MÃO, não o simplificador.**
///
/// ⚠️ **As duas metades, porque uma sozinha não diz nada:** um gancho fechado tem de ficar PRECISO
/// (era o report do Enio de 2026-07-30) **e** um arco liso tem de ficar ECONÔMICO (era o report
/// dele de 2026-07-18, *"muitos pontos muito próximos e até sobrepostos"*). É a mesma constante que
/// as duas queixas disputavam, e é por isso que o gate afirma as duas ao mesmo tempo: com a
/// tolerância cobrada contra a corda **não existe valor que satisfaça as duas**.
#[test]
fn the_stored_stroke_follows_the_hand_without_hoarding_points() {
    let espessura = 12.0_f32;
    let tol = 0.02 * espessura;
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
    let desvio = |cru: &[Vec2], keep: &[usize], suave: &[Vec2]| -> f32 {
        let pts: Vec<Vec2> = keep.iter().map(|&i| suave[i]).collect();
        let prs = vec![1.0_f32; pts.len()];
        let (finais, _) = resample_smooth(&pts, &prs, passo, tol);
        let mut pior = 0.0_f32;
        for q in cru {
            let mut best = f32::MAX;
            for w in finais.windows(2) {
                best = best.min(dist_seg(*q, w[0], w[1]));
            }
            pior = pior.max(best);
        }
        pior
    };
    // (1) O GANCHO — precisão. Sem smoothing, para o gate medir a SIMPLIFICAÇÃO e não o filtro.
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
    let d_curva = desvio(&gancho, &simplify_to_curve(&gancho, tol, passo), &gancho);
    let d_corda = desvio(&gancho, &simplify_rdp(&gancho, tol), &gancho);
    assert!(
        d_curva < 0.5 * espessura * 0.05,
        "o gancho guardado esta' a {:.2} % da espessura da mao",
        d_curva / espessura * 100.0
    );
    // A metade que torna o número acima um FATO sobre a lei nova: a corda erra mais na mesma cena.
    assert!(
        d_corda > d_curva * 1.5,
        "a corda deveria errar mais no gancho: corda {d_corda:.4} vs curva {d_curva:.4}"
    );
    // (2) O ARCO LISO — economia. A queixa oposta, que a mesma constante já causou.
    let liso: Vec<Vec2> = (0..240)
        .map(|k| {
            let t = k as f32 / 239.0;
            Vec2::new(40.0 + t * 300.0, 60.0 + 40.0 * (t * 1.6).sin())
        })
        .collect();
    let n = simplify_to_curve(&liso, tol, passo).len();
    assert!(
        n < liso.len() / 10,
        "o arco liso guardou {n} pontos de {} — a queixa de 2026-07-18 voltou",
        liso.len()
    );
}

/// 📏 **SONDA — o que sobra depois da cura da simplificação: a CAPTURA e o SMOOTHING.**
///
/// ⚠️ *"Melhor, mas ainda não tão bom como o Blender"* (Enio, 2026-07-30). O `simplify_rdp` é porte
/// do `paint.cc` do próprio Blender, então a diferença não está NELE — está nos dois estágios que
/// vêm ANTES e que são nossos:
///
/// - **`MIN_SAMPLE_PX = 2,0`** — a captura DESCARTA todo movimento menor que 2 px de tela. O
///   Blender faz o oposto: o *Input Samples* dele INTERPOLA amostras a mais entre eventos;
/// - **`DEFAULT_SMOOTHING = 0,5`** — que são **12 iterações** de binomial `[1,2,1]/4`. Um
///   passa-baixa desse tamanho não remove só tremor: ele encolhe curva fechada.
///
/// ⚠️ **O oráculo aqui NÃO é "desvio da mão"** para o smoothing — alisar é para desviar do cru (o
/// tremor). O que se mede é o que o artista chama de imprecisão: **o ápice do gancho ANDANDO** e o
/// raio da curva ABRINDO.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_the_capture_and_the_smoothing_cost() {
    let espessura = 12.0_f32;
    let (cx, cy, rr) = (220.0_f32, 58.0, 18.0);
    // O gancho, amostrado FINO (a mão de verdade, antes do nosso corte de captura).
    let mut fino: Vec<Vec2> = (0..360)
        .map(|k| Vec2::new(40.0 + k as f32 * 0.5, 40.0))
        .collect();
    for k in 1..=240 {
        let a = -core::f32::consts::FRAC_PI_2 + k as f32 / 240.0 * core::f32::consts::PI;
        fino.push(Vec2::new(cx + rr * a.cos(), cy + rr * a.sin()));
    }
    for k in 1..=360 {
        fino.push(Vec2::new(220.0 - k as f32 * 0.5, 76.0));
    }
    // O ápice do gancho na mão: o ponto mais à direita.
    let apice_mao = fino.iter().fold(f32::MIN, |m, p| m.max(p.x));

    println!("  (1) SMOOTHING — quanto o apice do gancho ANDA (captura fina, sem simplificar)");
    println!("      smoothing | iteracoes |  apice anda (px) | % da espessura");
    for smoothing in [0.0_f32, 0.1, 0.25, 0.5, 0.75] {
        let suave = active_smooth(&fino, smoothing);
        let apice = suave.iter().fold(f32::MIN, |m, p| m.max(p.x));
        let anda = apice_mao - apice;
        println!(
            "      {smoothing:9} | {:9} |     {anda:8.4}     |   {:6.2} %",
            (smoothing * 24.0).round() as u32,
            anda / espessura * 100.0
        );
    }

    println!("\n  (2) CAPTURA — quanto o corte de `MIN_SAMPLE_PX` custa (smoothing 0, sim 0,02)");
    println!("      corte px |  amostras |  desvio da MAO (px) | % da espessura");
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
    for corte in [0.5_f32, 1.0, 2.0, 4.0] {
        // A captura do produto: só aceita se andou >= corte.
        let mut cap: Vec<Vec2> = vec![fino[0]];
        for q in &fino[1..] {
            let last = *cap.last().expect("nao vazio");
            let d = Vec2::new(q.x - last.x, q.y - last.y);
            if (d.x * d.x + d.y * d.y).sqrt() >= corte {
                cap.push(*q);
            }
        }
        // ⚠️ **O pen-up PROMOVE a pendente** (`FlipDraw::take`), e o espelho tem de fazer o mesmo:
        // sem isto a sonda mede um produto que não existe mais — foi ela que achou o defeito
        // (o pior desvio caía EXATAMENTE no último ponto), e um espelho que não acompanha a cura
        // segue reportando o defeito curado.
        let fim = *fino.last().expect("nao vazio");
        if *cap.last().expect("nao vazio") != fim {
            cap.push(fim);
        }
        let tol = 0.02 * espessura;
        let keep = simplify_to_curve(&cap, tol, 0.4 * espessura);
        let pts: Vec<Vec2> = keep.iter().map(|&i| cap[i]).collect();
        let prs = vec![1.0_f32; pts.len()];
        let (finais, _) = resample_smooth(&pts, &prs, 0.4 * espessura, tol);
        let (mut pior, mut onde) = (0.0_f32, Vec2::new(0.0, 0.0));
        for q in &fino {
            let mut best = f32::MAX;
            for w in finais.windows(2) {
                best = best.min(dist_seg(*q, w[0], w[1]));
            }
            if best > pior {
                pior = best;
                onde = *q;
            }
        }
        // ⚠️ **ONDE**, porque `1,0000` exato em dois cortes diferentes é redondo demais para ser
        // publicado sem se olhar — número redondo é geometria exata ou fixture, e os dois são
        // indistinguíveis sem abrir.
        println!(
            "      {corte:8} |  {:8} |     {pior:8.4}        |   {:6.2} %   em ({:.2}, {:.2})",
            cap.len(),
            pior / espessura * 100.0,
            onde.x,
            onde.y
        );
    }
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
