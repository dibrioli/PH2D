//! Gates da ORDEM DA ONDA (ciclo 2, W2 — doc 105).

use super::*;

/// As posições `0..1` de uma fila de `n`, na ordem pedida.
fn fila(order: i32, n: u32, seed: u32) -> Vec<f32> {
    (0..n).map(|i| raw_at(order, i, n, seed)).collect()
}

/// ⭐⭐⭐ **A `Index` É BYTE-IDÊNTICA AO QUE SEMPRE FOI** — todo grafo autorado lê a mesma onda.
/// FALSIFICADO por a porta tocar no caminho de omissão.
#[test]
fn the_index_order_is_the_old_ramp_bit_for_bit() {
    for n in [1_u32, 2, 3, 7, 64] {
        for i in 0..n {
            let antigo = if n <= 1 {
                0.0
            } else {
                i as f32 / (n as f32 - 1.0)
            };
            assert_eq!(raw_at(ORDER_INDEX, i, n, 0), antigo, "n={n} i={i}");
        }
    }
    // E uma ordem FORA de alcance cai na de sempre — a lei do `Pick::of` e do `Transfer::of`.
    assert_eq!(raw_at(99, 3, 7, 0), raw_at(ORDER_INDEX, 3, 7, 0));
}

/// ⭐⭐ **`FROM CENTER` ARRANCA NO MEIO E ACABA NAS DUAS PONTAS** — e é SIMÉTRICA, que é a
/// promessa que o nome faz. FALSIFICADO por qualquer fórmula que não seja `|2i − (n−1)|`.
#[test]
fn from_center_starts_in_the_middle_and_is_symmetric() {
    let f = fila(ORDER_FROM_CENTER, 7, 0);
    assert_eq!(f[3], 0.0, "o do meio arranca primeiro: {f:?}");
    assert_eq!(f[0], 1.0, "e as pontas por ultimo");
    assert_eq!(f[6], 1.0);
    for i in 0..7 {
        assert_eq!(f[i], f[6 - i], "tem de ser espelhada: {f:?}");
    }
    // ⚠️ Numa fila PAR os DOIS do meio partilham o mínimo — é o que o olho espera.
    let p = fila(ORDER_FROM_CENTER, 6, 0);
    assert_eq!(p[2], p[3], "os dois do meio: {p:?}");
    assert!(p[2] < p[1], "e sao o minimo");
}

/// ⭐⭐⭐ **`FROM EDGES` NÃO É UMA ENTRADA — É `FROM CENTER` COM O `REVERSE`**, e o gate mede
/// isso: espelhar o `raw` do centro dá exactamente a onda que vem das pontas.
///
/// ⛔ É a razão de o enum ter três entradas e não cinco. FALSIFICADO por alguém acrescentar
/// `From Edges` ao enum: passariam a existir duas portas para a mesma pergunta.
#[test]
fn from_edges_is_from_center_mirrored_and_needs_no_entry() {
    let centro = fila(ORDER_FROM_CENTER, 9, 0);
    let pontas: Vec<f32> = centro.iter().map(|r| 1.0 - r).collect();
    assert_eq!(pontas[0], 0.0, "as pontas arrancam primeiro: {pontas:?}");
    assert_eq!(pontas[4], 1.0, "e o meio por ultimo");
    // E o enum tem MESMO três entradas — o controle que impede a lista de crescer sem medir.
    let hints = crate::PARAM_HINTS
        .iter()
        .find(|h| h.param == "order")
        .expect("o hint da ordem");
    let ph2d_node_registry::ParamWidget::Enum { labels } = hints.widget else {
        panic!("a ordem e' um enum")
    };
    assert_eq!(labels, &["Index", "From Center", "Random"]);
}

/// ⭐⭐ **`RANDOM` DÁ A CADA UM O SEU ATRASO, E A MESMA SEMENTE DÁ A MESMA FILA.**
///
/// ⚠️ Não é uma permutação (isso pediria uma ordenação — ver o doc do módulo): é um atraso
/// próprio por elemento. O que se afirma é o que importa: **espalha**, **é determinista**, e
/// **a semente muda**.
#[test]
fn random_is_deterministic_spread_and_the_seed_moves_it() {
    let a = fila(ORDER_RANDOM, 32, 7);
    assert_eq!(
        a,
        fila(ORDER_RANDOM, 32, 7),
        "a mesma semente, a mesma fila"
    );
    assert_ne!(a, fila(ORDER_RANDOM, 32, 8), "outra semente, outra fila");
    assert!(a.iter().all(|r| (0.0..1.0).contains(r)), "{a:?}");
    // Espalha: os 32 caem em pelo menos metade dos oito baldes de `1/8`.
    let mut baldes = [0_u32; 8];
    for r in &a {
        baldes[((r * 8.0) as usize).min(7)] += 1;
    }
    let cheios = baldes.iter().filter(|c| **c > 0).count();
    assert!(cheios >= 6, "espalhou por {cheios} de 8 baldes: {baldes:?}");
    // ⚠️ E o CONTROLE de que não é a rampa disfarçada: ela NÃO é monótona.
    assert!(
        a.windows(2).any(|w| w[1] < w[0]),
        "uma fila aleatoria nao e' crescente"
    );
}

/// ⚠️ **UMA FILA DE UM não tem ordem** — todas as três dão `0`, e é o caso que o `n−1` do
/// denominador tornaria uma divisão por zero.
#[test]
fn a_row_of_one_has_no_order() {
    for ord in [ORDER_INDEX, ORDER_FROM_CENTER, ORDER_RANDOM] {
        assert_eq!(raw_at(ord, 0, 1, 3), 0.0, "ordem {ord}");
        assert_eq!(raw_at(ord, 0, 0, 3), 0.0, "e uma fila vazia tambem");
    }
}

/// ⭐⭐ **O QUE O DISPOSITIVO FAZ É O QUE O RUST FAZ** — a lei está escrita duas vezes (aqui e no
/// WGSL do [`crate::kernel`]); o que não pode é divergirem sem ninguém ver.
#[test]
fn the_wgsl_says_what_the_rust_says() {
    let w = crate::kernel::GPU_KERNEL.wgsl_lib;
    for termo in [
        "sg_raw_at",
        "sg_hash01",
        "abs(2.0 * f32(i) - last) / last",
        "4294967296.0",
    ] {
        assert!(w.contains(termo), "o WGSL perdeu `{termo}`");
    }
    // ⚠️⚠️ **AS CONSTANTES DERIVAM-SE, NUNCA SE COPIAM** — a 1.ª redacção deste gate escrevia
    // os quatro mixers em decimal à mão, e **três dos quatro estavam errados**. O gate apanhou
    // o WGSL (que eu tinha copiado igualmente mal) e depois apanhou-se a si próprio.
    // *Um gate que compara duas cópias à mão não prova nada: ele tem de derivar um dos lados.*
    for m in [0x9E37_79B9_u32, 0x85EB_CA6B, 0x7FEB_352D, 0x846C_A68B] {
        assert!(w.contains(&format!("{m}u")), "o WGSL perdeu o mixer {m:#x}");
    }
    for p in ["order", "seed"] {
        assert!(
            crate::kernel::GPU_KERNEL.params.contains(&p),
            "o kernel nao recebe `{p}`"
        );
    }
}
