//! **O CACHE DO AJUSTE É O AJUSTE** — os gates do [`FitCache`].
//!
//! ⚠️ Irmão do módulo do orçamento, e o corte é por PERGUNTA: lá se mede *o que o ajuste custa e
//! como ele reparte o teto*; aqui, ***se guardar a resposta muda a resposta***. Os dois gates
//! centrais desta wave não são redundantes e é útil dizer por quê:
//!
//! - o de **identidade** prova que o cache não mente — e ele fica VERDE sobre um cache que nunca
//!   reusa nada (identidade é trivial quando não há reuso);
//! - o de **razão** prova que ele de fato poupa trabalho — e ele fica VERDE sobre um cache que
//!   reusa nós errados (rápido e mentiroso é rápido).
//!
//! Só os dois juntos afirmam a wave. Por isso o de identidade **declara a própria premissa**
//! (`reaproveitados`): sem esse número ele viraria vácuo em silêncio no dia em que alguém
//! enfraquecesse o `congelados`.

use super::super::super::{FitCache, Vec2, active_smooth, resample_smooth, simplify_to_curve};

/// A mão do artista: um serpenteado com TREMOR, amostrado como a captura amostra. (Cópia da
/// fixture do módulo do orçamento — as duas suítes têm de poder ser lidas em separado.)
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

const ESPESSURA: f32 = 12.0;
fn regras() -> (f32, f32) {
    (0.0025 * ESPESSURA, 0.4 * ESPESSURA)
}

/// ⭐ **O CACHE DEVOLVE EXATAMENTE O QUE O AJUSTE DEVOLVERIA** — índice a índice, quadro a quadro,
/// pelo pipeline do PRODUTO.
///
/// ⚠️ **O `active_smooth` é parte da fixture, não cenário.** Ele é quem torna o problema difícil: a
/// entrada do ajuste é o array SUAVIZADO, e uma amostra nova **reescreve a cauda dele** (o kernel é
/// local, mas as pontas são ancoradas). Um cache que confiasse em *"as amostras são append-only"*
/// reusaria nós decididos sobre números que já não existem — e não falharia: devolveria um traço
/// plausível. Alimentar o cache com o cru esconderia exatamente isso.
///
/// ⚠️ **A premissa é declarada:** o gate exige que o cache tenha REUSADO de verdade. Sem isso a
/// mutação *"nunca reaproveite nada"* passaria por aqui — e ela é a que mata a wave inteira.
#[test]
fn the_cached_fit_is_the_fit() {
    let (tol, passo) = regras();
    let cru = mao(900);
    let mut cache = FitCache::default();
    let (mut total_reuso, mut total_nos, mut quadros) = (0_usize, 0_usize, 0_usize);
    for n in 3..=cru.len() {
        let suave = active_smooth(&cru[..n], 0.5);
        let esperado = simplify_to_curve(&suave, tol, passo);
        let obtido = cache.simplify(&suave, tol, passo).to_vec();
        assert_eq!(
            obtido,
            esperado,
            "com {n} amostras o cache divergiu do ajuste (reusou {} nos)",
            cache.reaproveitados()
        );
        total_reuso += cache.reaproveitados();
        total_nos += esperado.len();
        quadros += 1;
    }
    println!("  {quadros} quadros | {total_reuso} nos reusados de {total_nos}");
    // A premissa: sem reuso o gate acima é verdadeiro por vácuo.
    assert!(
        total_reuso * 10 > total_nos * 9,
        "o cache reusou so' {total_reuso} de {total_nos} nos — o gate de identidade estaria \
         medindo um cache que refaz tudo"
    );
}

/// ⭐ **E ELE SOBREVIVE A UMA ENTRADA HOSTIL** — mesmo prefixo, CAUDA DIFERENTE.
///
/// ⚠️ **É este o caso que separa *medir* de *prometer*.** Um cache que guardasse só `n` e confiasse
/// no chamador aceitaria as duas entradas como "a mesma coisa, mais comprida" e reusaria a fronteira
/// inteira. Aqui a cauda vira uma QUINA — a decisão dos nós vizinhos à fronteira lê um span à frente
/// (o `espelho`), então ela muda —, e o cache tem de descobrir isso sozinho.
///
/// Mutação que sangra: tirar a condição de alcance do `congelados`.
#[test]
fn the_cache_survives_a_tail_that_changes_the_neighbouring_decisions() {
    let (tol, passo) = regras();
    let reto: Vec<Vec2> = (0..400)
        .map(|k| Vec2::new(40.0 + k as f32 * 2.0, 300.0))
        .collect();
    // Mesmo prefixo (300), cauda que vira uma quina fechada.
    let mut quina = reto[..300].to_vec();
    for k in 1..=100 {
        quina.push(Vec2::new(640.0, 300.0 + k as f32 * 2.0));
    }
    let mut cache = FitCache::default();
    let _ = cache.simplify(&reto, tol, passo);
    let obtido = cache.simplify(&quina, tol, passo).to_vec();
    assert_eq!(
        obtido,
        simplify_to_curve(&quina, tol, passo),
        "o cache reusou nos decididos sobre a OUTRA cauda (reusou {})",
        cache.reaproveitados()
    );
}

/// ⭐ **Mexer na ESPESSURA joga o cache fora** — `tol` e `passo` são dois dos três números de que a
/// resposta é função, e o terceiro (os pontos) já é verificado.
#[test]
fn changing_the_rules_throws_the_cache_away() {
    let (tol, passo) = regras();
    let cru = active_smooth(&mao(400), 0.5);
    let mut cache = FitCache::default();
    let _ = cache.simplify(&cru, tol, passo);
    for (t, p) in [(tol * 4.0, passo), (tol, passo * 2.0)] {
        let obtido = cache.simplify(&cru, t, p).to_vec();
        assert_eq!(
            obtido,
            simplify_to_curve(&cru, t, p),
            "o cache respondeu com os nos da regra ANTERIOR (tol {t}, passo {p})"
        );
        // E o cache volta ao normal na regra seguinte — sem estado preso.
        let _ = cache.simplify(&cru, tol, passo);
    }
}

/// ⭐ **O QUADRO DO PREVIEW PASSA A CUSTAR A CAUDA, NÃO O TRAÇO** — a entrega da wave.
///
/// ⚠️ **É uma RAZÃO entre as duas rotas na MESMA corrida, nunca um kill de milissegundos.** Um bar
/// de relógio mede o PERFIL do build (a lição que o gate irmão do orçamento pagou: 21,65 ms em
/// debug contra 1,92 em release, sobre o mesmo código) — uma razão entre dois caminhos medidos lado
/// a lado é imune ao perfil e à carga da máquina.
///
/// ⚠️ E ela é medida sobre o **quadro inteiro** (`active_smooth` + ajuste + reamostragem), que é o
/// que o artista espera, e não sobre o ajuste isolado — que exageraria o ganho, porque o
/// `active_smooth` continua rodando por quadro e **não** é cacheado.
#[test]
fn the_cache_makes_a_preview_frame_cost_the_tail_not_the_stroke() {
    let (tol, passo) = regras();
    let cru = mao(3000);
    // Um quadro do preview, das amostras à curva densa — as duas rotas, o mesmo trabalho.
    let quadro_sem = |n: usize| {
        let suave = active_smooth(&cru[..n], 0.5);
        let keep = simplify_to_curve(&suave, tol, passo);
        let pts: Vec<Vec2> = keep.iter().map(|&i| suave[i]).collect();
        resample_smooth(&pts, &vec![1.0; pts.len()], passo, tol)
            .0
            .len()
    };
    let quadro_com = |cache: &mut FitCache, n: usize| {
        let suave = active_smooth(&cru[..n], 0.5);
        let keep = cache.simplify(&suave, tol, passo).to_vec();
        let pts: Vec<Vec2> = keep.iter().map(|&i| suave[i]).collect();
        resample_smooth(&pts, &vec![1.0; pts.len()], passo, tol)
            .0
            .len()
    };
    // Aquece as duas rotas (o alocador tem memória entre chamadas — a lição do pen-up).
    let mut cache = FitCache::default();
    for n in (2400..2500).step_by(10) {
        let _ = quadro_sem(n);
        let _ = quadro_com(&mut cache, n);
    }
    let medir = |f: &mut dyn FnMut(usize)| {
        let t0 = std::time::Instant::now();
        for n in 2500..2600 {
            f(n);
        }
        t0.elapsed().as_secs_f64() * 1000.0
    };
    let ms_sem = medir(&mut |n| {
        let _ = quadro_sem(n);
    });
    let mut cache = FitCache::default();
    let _ = quadro_com(&mut cache, 2499);
    let ms_com = medir(&mut |n| {
        let _ = quadro_com(&mut cache, n);
    });
    let razao = ms_sem / ms_com.max(1e-9);
    println!("  100 quadros: sem cache {ms_sem:.2} ms | com cache {ms_com:.2} ms | {razao:.1}x");
    // A premissa: o traço TEM de exigir muitos nos, senão a razão mede o vazio.
    assert!(
        simplify_to_curve(&active_smooth(&cru, 0.5), tol, passo).len() > 200,
        "a fixture nao exige pontos — o gate mediria o vazio"
    );
    assert!(
        razao > 3.0,
        "o cache poupou so' {razao:.1}x ({ms_sem:.2} -> {ms_com:.2} ms em 100 quadros) — ele \
         voltou a re-decidir o traco inteiro?"
    );
}

/// 📏 **SONDA — de que é feito um quadro do preview, com e sem o cache.**
///
/// A irmã do `measure_what_a_live_preview_frame_is_made_of`, agora com as duas rotas lado a lado:
/// é ela que diz quanto do quadro ainda é do `active_smooth` (que NÃO é cacheado) e quanto sobrou
/// no ajuste.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_preview_frame_with_and_without_the_cache() {
    let (tol, passo) = regras();
    println!("  amostras | smooth | ajuste SEM | ajuste COM | nos reusados | quadro sem | com");
    for n in [1200_usize, 3000, 9000] {
        let cru = mao(n);
        // O cache chega ao quadro `n` como chegaria no produto: um quadro por amostra.
        let mut cache = FitCache::default();
        for k in (3..n).step_by(1) {
            let s = active_smooth(&cru[..k], 0.5);
            let _ = cache.simplify(&s, tol, passo);
        }
        let t0 = std::time::Instant::now();
        let suave = active_smooth(&cru, 0.5);
        let t1 = std::time::Instant::now();
        let sem = simplify_to_curve(&suave, tol, passo);
        let t2 = std::time::Instant::now();
        let com = cache.simplify(&suave, tol, passo).to_vec();
        let t3 = std::time::Instant::now();
        assert_eq!(sem, com);
        let ms = |a: std::time::Instant, b: std::time::Instant| (b - a).as_secs_f64() * 1000.0;
        println!(
            "  {n:8} | {:6.3} | {:10.3} | {:10.3} | {:12} | {:10.3} | {:.3}",
            ms(t0, t1),
            ms(t1, t2),
            ms(t2, t3),
            cache.reaproveitados(),
            ms(t0, t1) + ms(t1, t2),
            ms(t0, t1) + ms(t2, t3)
        );
    }
}
