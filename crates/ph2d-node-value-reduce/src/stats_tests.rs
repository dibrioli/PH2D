//! Os gates da LEI — as oito agregações e as duas portas que as escopam.

use crate::stats::{Mode, aggregate, group_of, reduce_field, selected, variance};
use ph2d_nodegraph::gpu::ReduceOp;

/// **O oráculo CONGELADO** — as quatro agregações exactamente como shipavam
/// antes deste grupo, copiadas verbatim.
///
/// ⚠️ Chamar a função sob teste para computar o que se espera dela é o gate
/// sempre-verde que esta casa já documentou três vezes; e um `pub(crate)` sem
/// chamador seria uma SEGUNDA resposta esperando alguém chamá-la, então ela vive
/// sob `cfg(test)`.
fn frozen_aggregate(field: &[f32], mode: Mode) -> f32 {
    match mode {
        Mode::Sum => ReduceOp::Sum.cpu(field),
        Mode::Mean => {
            let n = field.len() as f32;
            if n > 0.0 {
                ReduceOp::Sum.cpu(field) / n
            } else {
                0.0
            }
        }
        Mode::Min => ReduceOp::Min.cpu(field),
        Mode::Max => ReduceOp::Max.cpu(field),
        _ => unreachable!("o oráculo só conhece os quatro que shipavam"),
    }
}

/// **Os quatro modos que já shipavam são BIT a BIT o que eram** — o controle do
/// grupo. Os quatro novos entram por baixo deles, não por dentro.
#[test]
fn the_four_original_modes_are_bit_identical_to_what_shipped() {
    let fields: [&[f32]; 5] = [
        &[1.0, 2.0, 3.0, 4.0],
        &[-3.5, 0.25, 17.0],
        &[7.0],
        &[0.1, 0.2, 0.3, -0.4, 1e6, -1e-6],
        &[2.0; 33],
    ];
    for f in fields {
        for mode in [Mode::Sum, Mode::Mean, Mode::Min, Mode::Max] {
            assert_eq!(
                aggregate(f, mode).to_bits(),
                frozen_aggregate(f, mode).to_bits(),
                "{mode:?} moveu-se sobre {f:?}"
            );
        }
    }
}

/// **`Range` é a extensão, e é EXACTA** — `max − min` não passa por soma
/// nenhuma, então não carrega o ε que os dois `Sum` carregam.
#[test]
fn range_is_the_extent_and_it_is_exact() {
    assert_eq!(aggregate(&[3.0, -1.0, 7.5, 2.0], Mode::Range), 8.5);
    assert_eq!(aggregate(&[5.0; 4], Mode::Range), 0.0, "campo chato: zero");
    let f = [1e7, 1e7 + 1.0];
    assert_eq!(
        aggregate(&f, Mode::Range).to_bits(),
        (aggregate(&f, Mode::Max) - aggregate(&f, Mode::Min)).to_bits()
    );
}

/// **A variância mede a DISPERSÃO e o desvio é a raiz dela** — num campo cujo
/// desvio se sabe de cabeça: `[2, 4, 4, 4, 5, 5, 7, 9]` tem média 5, variância
/// populacional 4 e desvio 2.
#[test]
fn the_variance_and_the_deviation_measure_the_spread() {
    let f = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    assert_eq!(aggregate(&f, Mode::Mean), 5.0);
    assert!((aggregate(&f, Mode::Variance) - 4.0).abs() < 1e-4);
    assert!((aggregate(&f, Mode::StdDev) - 2.0).abs() < 1e-4);
    // Um campo constante não tem dispersão nenhuma — e com a fórmula de dois
    // passos isso é EXACTO por construção (cada desvio é `v − v = 0`), sem clamp
    // nenhum a fazer: o `sqrt` do `StdDev` nunca vê um argumento negativo.
    for c in [0.0f32, 1.0, -3.5, 1e5] {
        let f = [c; 16];
        assert_eq!(aggregate(&f, Mode::Variance), 0.0, "constante {c}");
        assert_eq!(aggregate(&f, Mode::StdDev), 0.0, "constante {c}");
        assert!(aggregate(&f, Mode::StdDev).is_finite());
    }
}

/// **A variância é a POPULACIONAL (÷N), não a amostral (÷N−1)** — e a diferença
/// não é académica: com `N = 1` a amostral divide por ZERO.
#[test]
fn the_variance_is_the_population_one() {
    let f = [1.0, 3.0]; // média 2 · populacional 1 · amostral 2
    assert!((aggregate(&f, Mode::Variance) - 1.0).abs() < 1e-5);
    assert_eq!(aggregate(&[42.0], Mode::Variance), 0.0);
    assert!(aggregate(&[42.0], Mode::StdDev).is_finite());
}

/// **A mediana é o do MEIO, e não a média** — o gate que os separa é um campo
/// com um outlier: `[1, 2, 3, 4, 1000]` tem mediana 3 e média 202.
#[test]
fn the_median_is_the_middle_not_the_mean() {
    let f = [1.0, 2.0, 3.0, 4.0, 1000.0];
    assert_eq!(aggregate(&f, Mode::Median), 3.0);
    assert!(aggregate(&f, Mode::Mean) > 200.0, "a média cede ao outlier");
    // Contagem PAR: a média dos dois centrais.
    assert_eq!(aggregate(&[1.0, 2.0, 8.0, 10.0], Mode::Median), 5.0);
    // A ordem de entrada não importa (é um rank, não uma varredura).
    assert_eq!(aggregate(&[1000.0, 3.0, 1.0, 4.0, 2.0], Mode::Median), 3.0);
}

/// **Um conjunto VAZIO devolve zero em TODOS os modos** — e é o `Min`/`Max` que
/// torna isto load-bearing: a identidade deles é `±∞`, e com uma máscara que não
/// selecciona nada esse infinito seria DIFUNDIDO para um campo de N elementos.
#[test]
fn an_empty_set_has_no_aggregate_and_answers_zero() {
    for mode in [
        Mode::Sum,
        Mode::Mean,
        Mode::Min,
        Mode::Max,
        Mode::Range,
        Mode::Variance,
        Mode::StdDev,
        Mode::Median,
    ] {
        let a = aggregate(&[], mode);
        assert_eq!(a, 0.0, "{mode:?} sobre o vazio");
        assert!(a.is_finite(), "{mode:?} sobre o vazio tem de ser finito");
    }
}

/// **Sem máscara e sem grupo, a saída é a de sempre** — o CONTROLE das duas
/// portas: elas não podem custar nada a quem não as liga.
#[test]
fn without_the_ports_the_output_is_the_broadcast_aggregate() {
    let f = [1.0, 2.0, 3.0, 4.0];
    for mode in [Mode::Sum, Mode::Mean, Mode::Min, Mode::Max, Mode::Range] {
        let out = reduce_field(&f, mode, &[], &[]);
        assert_eq!(out, vec![aggregate(&f, mode); 4], "{mode:?}");
    }
}

/// **A máscara escolhe quem é CONTADO, e todos recebem o número** — a lei que
/// mantém a cadeia `reduce → math(Subtract)` viva para o campo inteiro. Um
/// elemento excluído continua a receber o agregado; o que ele não faz é entrar
/// nele.
#[test]
fn the_mask_picks_who_is_counted_never_who_is_answered() {
    let f = [1.0, 2.0, 3.0, 100.0];
    let m = [1.0, 1.0, 1.0, 0.0];
    let out = reduce_field(&f, Mode::Mean, &m, &[]);
    assert_eq!(out, vec![2.0; 4], "a média dos três, difundida aos quatro");
    // E o extremo: o 100 excluído não pode ser o máximo.
    assert_eq!(reduce_field(&f, Mode::Max, &m, &[]), vec![3.0; 4]);
}

/// **Uma máscara de comprimento 1 DIFUNDE** — a convenção `ReadBroadcast` da
/// engine: uma chave liga ou desliga o conjunto inteiro. E desligada, o conjunto
/// fica vazio, que é a lei do zero.
#[test]
fn a_length_one_mask_switches_the_whole_set() {
    let f = [1.0, 2.0, 3.0];
    assert_eq!(reduce_field(&f, Mode::Sum, &[1.0], &[]), vec![6.0; 3]);
    assert_eq!(reduce_field(&f, Mode::Sum, &[0.0], &[]), vec![0.0; 3]);
    assert!(selected(&[], 99), "ausente ⇒ todos");
    assert!(
        selected(&[7.0], 99),
        "comprimento 1 difunde, em qualquer índice"
    );
    assert!(!selected(&[1.0, 0.0], 1));
    assert!(!selected(&[1.0, 1.0], 5), "fora do alcance ⇒ excluído");
}

/// **Cada grupo recebe o SEU agregado** — a redução segmentada. E os ids são
/// arredondados, então um campo contínuo (o que um `value.*` produz) parte em
/// bins sem um nó de quantização no meio.
#[test]
fn each_group_gets_its_own_aggregate() {
    let f = [1.0, 3.0, 10.0, 20.0, 100.0];
    let g = [0.0, 0.0, 1.0, 1.0, 2.0];
    assert_eq!(
        reduce_field(&f, Mode::Mean, &[], &g),
        vec![2.0, 2.0, 15.0, 15.0, 100.0]
    );
    // `1.4` e `0.6` arredondam para o MESMO bin.
    assert_eq!(group_of(&[0.6, 1.4], 0), 1);
    assert_eq!(group_of(&[0.6, 1.4], 1), 1);
    assert_eq!(group_of(&[], 3), 0, "ausente ⇒ um grupo só");
    assert_eq!(group_of(&[f32::NAN], 0), 0, "não-finito ⇒ grupo zero");
}

/// **Máscara e grupo COMPÕEM** — a estatística de cada bin corre sobre os
/// membros seleccionados dele, e um bin sem nenhum recebe zero pela lei do
/// conjunto vazio (não a identidade `±∞` do operador).
#[test]
fn the_mask_and_the_group_compose() {
    let f = [1.0, 100.0, 10.0, 20.0];
    let g = [0.0, 0.0, 1.0, 1.0];
    let m = [1.0, 0.0, 0.0, 0.0]; // só o primeiro do grupo 0
    let out = reduce_field(&f, Mode::Min, &m, &g);
    assert_eq!(out[0], 1.0);
    assert_eq!(out[1], 1.0, "o excluído recebe o número do seu grupo");
    assert_eq!(out[2], 0.0, "o grupo 1 ficou vazio ⇒ zero, nunca +∞");
    assert!(out[2].is_finite() && out[3].is_finite());
}

/// **Um grupo só é o mundo de antes** — o CONTROLE da porta `group`: com todos
/// os elementos no mesmo bin, o resultado tem de ser o agregado difundido.
#[test]
fn one_group_is_the_ungrouped_world() {
    let f = [1.0, 2.0, 3.0, 4.0, 5.0];
    for mode in [Mode::Sum, Mode::Mean, Mode::Median, Mode::StdDev] {
        assert_eq!(
            reduce_field(&f, mode, &[], &[3.0; 5]),
            reduce_field(&f, mode, &[], &[]),
            "{mode:?}"
        );
    }
}

/// **Os oito índices são os que o documento guarda** — renumerá-los re-aponta em
/// silêncio todo grafo salvo, e os quatro primeiros são os que já shipavam.
#[test]
fn the_mode_indices_are_the_saved_face() {
    for (i, m) in [
        Mode::Sum,
        Mode::Mean,
        Mode::Min,
        Mode::Max,
        Mode::Range,
        Mode::Variance,
        Mode::StdDev,
        Mode::Median,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(Mode::from_param(i as f32), m);
        assert_eq!(Mode::from_param(i as f32 + 0.4), m, "arredonda para si");
    }
    assert_eq!(Mode::from_param(99.0), Mode::Sum, "fora da lista ⇒ Sum");
}

/// **A fórmula de UM passo, congelada como CONTRA-oráculo** — `E[v²] − E[v]²`,
/// exactamente a que este grupo construiu, mediu e descartou.
///
/// ⚠️ Ela não vive no produto: existe aqui para o gate abaixo poder afirmar *por
/// quanto* ela erra, em vez de a rejeição ficar sendo uma frase.
fn one_pass_variance(field: &[f32]) -> f32 {
    let n = field.len() as f32;
    let mean = field.iter().fold(0.0f32, |a, b| a + b) / n;
    let sumsq = field.iter().fold(0.0f32, |a, v| a + v * v);
    (sumsq / n - mean * mean).max(0.0)
}

/// **A variância de DOIS passos resolve onde a de um passo se desfaz** — o gate
/// que justifica a recusa do device, com os dois lados medidos lado a lado.
///
/// O caso que fecha a discussão é o campo **CONSTANTE**: a resposta certa é zero
/// e a mais fácil que existe, e o um passo reporta um desvio de dezenas.
#[test]
fn the_two_pass_variance_resolves_where_the_one_pass_falls_apart() {
    // Um campo de desvio EXACTAMENTE 1, deslocado para a média `mu`.
    let spread_one = |mu: f32| -> Vec<f32> {
        (0..64)
            .map(|i| mu + if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect()
    };
    for mu in [0.0f32, 1.0, 100.0, 1e3, 1e4, 1e5, 1e6] {
        let f = spread_one(mu);
        assert!(
            (variance(&f) - 1.0).abs() < 1e-4,
            "dois passos têm de dar 1 em mu={mu}, deu {}",
            variance(&f)
        );
    }
    // E os dois pontos em que o um passo JÁ tinha ido embora (medidos):
    // mu=1e3 devolve 0,25 de variância (metade do desvio) e mu=1e5 devolve 3072.
    assert!(
        (one_pass_variance(&spread_one(1e3)) - 1.0).abs() > 0.1,
        "se o um passo passar a acertar em 1e3, a medição que motivou a recusa \
         deixou de ser verdade e a nota tem de ser reconferida"
    );
    // O caso que decide: um campo CONSTANTE não tem dispersão nenhuma.
    for mu in [1.0f32, 100.0, 1e4, 1e5, 1e6] {
        let f = vec![mu; 64];
        assert_eq!(variance(&f), 0.0, "constante em {mu}: dois passos");
    }
    assert!(
        one_pass_variance(&vec![1e5f32; 64]).sqrt() > 10.0,
        "o um passo inventa dezenas de desvio num campo constante — é este \
         número que põe Variance/StdDev fora do device"
    );
}

/// **A sonda que produziu a tabela do doc-comment** — o desvio que cada fórmula
/// reporta, por magnitude do campo.
#[test]
#[ignore = "sonda: cargo test -p ph2d-node-value-reduce -- --ignored --nocapture"]
fn measure_where_the_one_pass_variance_stops_resolving() {
    println!(
        "{:>10} {:>12} {:>12} {:>12}",
        "media", "sigma real", "1 passo", "2 passos"
    );
    for mu in [0.0f32, 1.0, 10.0, 100.0, 1e3, 1e4, 1e5, 1e6] {
        let f: Vec<f32> = (0..64)
            .map(|i| mu + if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        println!(
            "{mu:>10} {:>12} {:>12.6} {:>12.6}",
            1.0,
            one_pass_variance(&f).sqrt(),
            variance(&f).sqrt()
        );
    }
    println!("--- campo CONSTANTE (sigma real = 0) ---");
    for mu in [1.0f32, 100.0, 1e4, 1e5, 1e6] {
        let f = vec![mu; 64];
        println!(
            "{mu:>10} {:>12} {:>12.6} {:>12.6}",
            0.0,
            one_pass_variance(&f).sqrt(),
            variance(&f).sqrt()
        );
    }
}
