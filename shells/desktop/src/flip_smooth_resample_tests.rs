//! Os gates da REAMOSTRAGEM (a Catmull-Rom centrípeta do `flip_smooth`) e a sonda que os
//! mediu. Irmão por LOC cap (HR-18, 600) — segue módulo FILHO via `#[path]`, então
//! `use super::*` alcança os privados do `flip_smooth`.

use super::*;

/// A distância do ponto `p` à POLILINHA que o artista de fato desenhou.
fn dist_to_polyline(p: Vec2, pts: &[Vec2]) -> f32 {
    let mut best = f32::MAX;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let ab = Vec2::new(b.x - a.x, b.y - a.y);
        let l2 = ab.x * ab.x + ab.y * ab.y;
        let t = if l2 < 1e-12 {
            0.0
        } else {
            (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0)
        };
        let d = Vec2::new(p.x - a.x - t * ab.x, p.y - a.y - t * ab.y);
        best = best.min((d.x * d.x + d.y * d.y).sqrt());
    }
    best
}

/// O maior GIRO (graus) entre segmentos consecutivos da saida -- o detector direto de
/// CUSPIDE: uma cuspide e uma reversao de ~180 graus, e e exatamente a propriedade que a
/// parametrizacao centripeta prova nao existir (Yuksel et al. 2011).
fn max_turn_deg(pts: &[Vec2]) -> f32 {
    let mut worst = 0.0f32;
    for w in pts.windows(3) {
        let a = Vec2::new(w[1].x - w[0].x, w[1].y - w[0].y);
        let b = Vec2::new(w[2].x - w[1].x, w[2].y - w[1].y);
        let la = (a.x * a.x + a.y * a.y).sqrt();
        let lb = (b.x * b.x + b.y * b.y).sqrt();
        if la < 1e-9 || lb < 1e-9 {
            continue;
        }
        let c = ((a.x * b.x + a.y * b.y) / (la * lb)).clamp(-1.0, 1.0);
        worst = worst.max(c.acos().to_degrees());
    }
    worst
}

/// O comprimento de arco de uma polilinha.
fn arc(pts: &[Vec2]) -> f32 {
    pts.windows(2)
        .map(|w| {
            let d = Vec2::new(w[1].x - w[0].x, w[1].y - w[0].y);
            (d.x * d.x + d.y * d.y).sqrt()
        })
        .sum()
}

/// As fixtures das duas metades desta wave — o CONTROLE (espaçamento par, passa nos dois
/// mundos) e três graus de espaçamento DESIGUAL com giro GENTIL, que é o que o RDP produz.
/// UMA lista, dois consumidores: o gate e a sonda medem o MESMO traço.
fn fixtures() -> [(&'static str, Vec<Vec2>); 4] {
    [
        (
            "CONTROLE  espacamento par, giro gentil",
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(10.0, 0.0),
                Vec2::new(20.0, 3.0),
                Vec2::new(30.0, 9.0),
            ],
        ),
        (
            "desigual 50:1  giro gentil (dot 0.71)",
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(100.0, 0.0),
                Vec2::new(102.0, 2.0),
                Vec2::new(104.0, 20.0),
            ],
        ),
        (
            "desigual 20:1  giro gentil",
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(60.0, 0.0),
                Vec2::new(63.0, 3.0),
                Vec2::new(66.0, 12.0),
            ],
        ),
        (
            "desigual 8:1   giro gentil",
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(40.0, 0.0),
                Vec2::new(45.0, 5.0),
                Vec2::new(50.0, 15.0),
            ],
        ),
    ]
}

/// **Suavizar não pode entortar MAIS do que o traço já entortava** — e nenhuma barriga
/// pode ser maior que o span mais curto em que ela vive.
///
/// As duas metades são PROPRIEDADES do que "suavizar" significa, não modelos da
/// implementação: um alisador redistribui o giro de uma quina por vários vértices, então
/// o giro de CADA junta da saída é no máximo o giro da quina da entrada (com igualdade
/// quando o span é curto demais para subdividir — daí o `<=` e a folga de meio grau, que é
/// ruído de `f32`, não margem escolhida). E o desvio à polilinha que o artista desenhou
/// mede a BARRIGA: uma que passa do span mais curto é tinta num lugar onde ninguém passou.
///
/// ⚠️ **O regime que isto pega não é a quina — é o ESPAÇAMENTO DESIGUAL com giro GENTIL**,
/// que é o que o RDP produz (ele apaga os pontos do trecho reto e mantém os da curva). O
/// CONTROLE é o espaçamento par: ele passa nos dois mundos, então são os desiguais que
/// carregam o gate. Medido, giro máximo da saída — centrípeta vs uniforme:
/// par `4,7° / 4,8°` · `8:1` `14,5° / 46,5°` · `20:1` `19,4° / 135,3°` ·
/// `50:1` **`29,6° / 158,5°`** (uma reversão de 158° É uma cúspide) — e o desvio no `50:1`
/// vai de `1,51` para `5,55` num span de corda `2,83`.
///
/// Mutação que sangra: `knot_step` devolver `1.0` (α = 0, a parametrização UNIFORME).
#[test]
fn the_resampled_curve_never_bends_harder_than_the_polyline_it_smooths() {
    for (name, pts) in fixtures() {
        let prs = vec![1.0f32; pts.len()];
        let (out, _) = resample_smooth(&pts, &prs, 2.0, 2.0 * 0.125);
        let turn_in = max_turn_deg(&pts);
        let turn_out = max_turn_deg(&out);
        assert!(
            turn_out <= turn_in + 0.5,
            "{name}: a reamostragem entortou MAIS que a polilinha \
             — giro de saída {turn_out:.1}°, entrada {turn_in:.1}° (cúspide da uniforme)"
        );
        let shortest = pts
            .windows(2)
            .map(|w| {
                let d = Vec2::new(w[1].x - w[0].x, w[1].y - w[0].y);
                (d.x * d.x + d.y * d.y).sqrt()
            })
            .fold(f32::MAX, f32::min);
        let dev = out
            .iter()
            .map(|&p| dist_to_polyline(p, &pts))
            .fold(0.0f32, f32::max);
        assert!(
            dev <= shortest,
            "{name}: a barriga ({dev:.3}) passou do span mais curto ({shortest:.3})"
        );
    }
}

/// SONDA — o comentário do `centered_tangent` afirma que *"uniforme basta porque as
/// quinas são tratadas à parte"*. A afirmação é sobre um número: a Catmull-Rom
/// UNIFORME dispara com ESPAÇAMENTO DESIGUAL, e não só em quina — e espaçamento
/// desigual é o que o RDP PRODUZ (ele apaga os pontos do trecho reto e mantém os
/// da curva). Rode com `--nocapture`.
#[test]
fn measure_the_uniform_parameterisation_on_uneven_spacing() {
    let cases = fixtures();
    println!("  caso                                  | desvio max | inflacao | giro max");
    for (name, pts) in cases {
        let prs = vec![1.0f32; pts.len()];
        let (out, _) = resample_smooth(&pts, &prs, 2.0, 2.0 * 0.125);
        let dev = out
            .iter()
            .map(|&p| dist_to_polyline(p, &pts))
            .fold(0.0f32, f32::max);
        println!(
            "  {name:37} | {dev:10.3} | {:8.3} | {:7.1}",
            arc(&out) / arc(&pts),
            max_turn_deg(&out),
        );
    }
}

/// SONDA — **quantos pontos a reamostragem inventa, e quantos deles não dizem nada.**
///
/// O passo é FIXO em arco (`0,4 × largura`), então um span RETO é subdividido com a mesma
/// densidade de uma curva fechada. E span reto longo é o que o RDP DEIXA (ele existe para
/// colapsar o trecho reto a dois pontos) — ou seja, o par RDP→reamostragem apaga pontos de
/// uma reta e depois repõe pontos na MESMA reta. Cada um custa um quad na GPU e, pior, um
/// SLOT na lista de vizinhos (o orçamento da fita local existe porque ela satura).
///
/// "Não diz nada" = o ponto está a menos de `1e-3` da corda do span que ele subdivide, isto
/// é: apagá-lo não moveria a tinta.
#[test]
fn measure_how_many_resampled_points_say_nothing() {
    let mut cases: Vec<(&str, Vec<Vec2>)> = vec![
        (
            "reta longa (o que o RDP deixa)",
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(50.0, 0.0),
                Vec2::new(100.0, 0.0),
            ],
        ),
        (
            "L: duas retas longas + quina",
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(60.0, 0.0),
                Vec2::new(60.0, 60.0),
            ],
        ),
    ];
    // Um arco de verdade: 9 pontos num quarto de circulo de raio 40 (todo ponto informa).
    let arc_pts: Vec<Vec2> = (0..=8)
        .map(|i| {
            let a = i as f32 / 8.0 * std::f32::consts::FRAC_PI_2;
            Vec2::new(40.0 * a.cos(), 40.0 * a.sin())
        })
        .collect();
    cases.push(("arco de 90 graus (9 pontos)", arc_pts));
    cases.extend(fixtures());

    // ⚠️ Os pontos de ENTRADA estão na polilinha por definição, então contam como
    // "mudos" sempre — o DESPERDÍCIO é o que sobra depois de descontá-los.
    println!("  caso                                  | entra | sai  | desperdicio");
    for (name, pts) in cases {
        let prs = vec![1.0f32; pts.len()];
        let (out, _) = resample_smooth(&pts, &prs, 2.0, 2.0 * 0.125);
        // Um ponto de saida e MUDO se cai sobre a corda do span de entrada mais proximo.
        let mute = out
            .iter()
            .filter(|&&p| dist_to_polyline(p, &pts) < 1e-3)
            .count();
        println!(
            "  {name:37} | {:5} | {:4} | {:11}",
            pts.len(),
            out.len(),
            mute.saturating_sub(pts.len()),
        );
    }
}

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
        let suave = super::active_smooth(&cru, 0.0);
        let keep = super::simplify_rdp(&suave, tol);
        let pts: Vec<Vec2> = keep.iter().map(|&i| suave[i]).collect();
        let prs = vec![1.0_f32; pts.len()];
        let (finais, _) = super::resample_smooth(&pts, &prs, 0.4 * espessura, tol);
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
        let suave = super::active_smooth(&cru, smoothing);
        let keep = super::simplify_to_curve(&suave, tol, 0.4 * espessura);
        let pts: Vec<Vec2> = keep.iter().map(|&i| suave[i]).collect();
        let prs = vec![1.0_f32; pts.len()];
        let (finais, _) = super::resample_smooth(&pts, &prs, 0.4 * espessura, tol);
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
        let suave = super::active_smooth(&cru, smoothing);
        let keep = super::simplify_rdp(&suave, tol);
        let pts: Vec<Vec2> = keep.iter().map(|&i| suave[i]).collect();
        let prs = vec![1.0_f32; pts.len()];
        let (finais, _) = super::resample_smooth(&pts, &prs, 0.4 * espessura, tol);
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
    for fracao in [0.05_f32, 0.03, 0.02, 0.01] {
        let tol = fracao * espessura;
        let mut linha = format!("  {fracao:6} |");
        let mut ms = 0.0_f64;
        for (idx, cru) in [&gancho, &liso].iter().enumerate() {
            let suave = super::active_smooth(cru, 0.5);
            let t0 = std::time::Instant::now();
            let keep = super::simplify_to_curve(&suave, tol, 0.4 * espessura);
            if idx == 0 {
                ms = t0.elapsed().as_secs_f64() * 1000.0;
            }
            let pts: Vec<Vec2> = keep.iter().map(|&i| suave[i]).collect();
            let prs = vec![1.0_f32; pts.len()];
            let (finais, _) = super::resample_smooth(&pts, &prs, 0.4 * espessura, tol);
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
        let (finais, _) = super::resample_smooth(&pts, &prs, passo, tol);
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
    let d_curva = desvio(
        &gancho,
        &super::simplify_to_curve(&gancho, tol, passo),
        &gancho,
    );
    let d_corda = desvio(&gancho, &super::simplify_rdp(&gancho, tol), &gancho);
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
    let n = super::simplify_to_curve(&liso, tol, passo).len();
    assert!(
        n < liso.len() / 10,
        "o arco liso guardou {n} pontos de {} — a queixa de 2026-07-18 voltou",
        liso.len()
    );
}
