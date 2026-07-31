//! **O ORÇAMENTO DO AJUSTE** — quanto ele custa e como ele reparte o teto.
//!
//! ⚠️ Irmão do [`super`], e o corte é por ORÁCULO: lá o desvio se mede em fração da espessura;
//! aqui em **milissegundos** (o ajuste roda por frame) e em **distribuição de pontos** (o teto não
//! pode cortar por lugar). Os dois modos de falha — lento, enviesado — são invisíveis a uma
//! medição de forma.

use super::super::super::{Vec2, resample_smooth, simplify_to_curve};

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
/// ⚠️ **A suspeita, escrita antes de medir, e ela é sobre a ORDEM da minha fila.** O ajuste
/// divide-e-conquista com um `Vec` e `pop()` — ou seja uma PILHA (LIFO). Ao partir `(a, b)` ele
/// empilha `(a, idx)` e depois `(idx, b)`, então o próximo a sair é `(idx, b)`: a recursão desce
/// pela metade da DIREITA primeiro. Quando o `MAX_FIT_POINTS` fecha a porta, o que sobra na pilha
/// sem ser processado são justamente os spans da ESQUERDA — o **começo do traço**.
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

/// ⭐ **O TETO CORTA QUALIDADE, NÃO UM PEDAÇO DO TRAÇO** (report do Enio, 2026-07-30: *"se o traço é
/// muito longo o início do traço perde pontos e deforma"*).
///
/// ⚠️ **A fila já foi uma PILHA**, e uma pilha faz a recursão descer inteira pela metade da direita
/// antes de tocar a esquerda. Enquanto o teto não morde, tanto faz — quando morde, o orçamento fica
/// todo no FIM. Medido com o teto forçado a 64: pontos por terço **`[1, 0, 63]`**, 1594 % de desvio
/// no primeiro terço.
///
/// ⚠️ **E é por isso que este gate FORÇA o teto.** Com o teto de produção (512) e um traço pedindo
/// 495, ele só mordia nas últimas 17 inserções e a distribuição saía `[166, 165, 164]` — de
/// aparência perfeitamente inocente. *Um guard que quase não morde esconde a forma do que acontece
/// quando ele morde.*
#[test]
fn the_cap_costs_quality_everywhere_not_the_start_of_the_stroke() {
    let espessura = 12.0_f32;
    let n = 9000_usize;
    let cru: Vec<Vec2> = (0..n)
        .map(|k| {
            let t = k as f32 / (n - 1) as f32;
            Vec2::new(
                40.0 + t * 3000.0,
                300.0 + 90.0 * (t * 26.0).sin() + 30.0 * (t * 61.0).cos(),
            )
        })
        .collect();
    const TETO: usize = 64;
    let keep = super::super::fit::simplify_to_curve_capped(&cru, 0.0025 * espessura, 4.8, TETO);
    // A premissa: o teto TEM de morder, senão o gate mede um caso que não existe.
    assert_eq!(
        keep.len(),
        TETO,
        "o teto nao mordeu — a fixture nao contem o caso"
    );
    let mut por_terco = [0_usize; 3];
    for &i in &keep {
        por_terco[(i * 3 / n).min(2)] += 1;
    }
    let magro = *por_terco.iter().min().expect("tres tercos");
    assert!(
        magro * 5 >= TETO,
        "um terco do traco levou so' {magro} de {TETO} pontos ({por_terco:?}) — o teto voltou a \
         cortar por LUGAR em vez de por qualidade"
    );
}
