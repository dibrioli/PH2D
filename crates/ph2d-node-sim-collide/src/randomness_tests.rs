//! Gates da **ALEATORIEDADE DA RESTITUIÇÃO** (doc 89 folha 13, célula do `sim.collide`).
//!
//! A lei tem quatro metades: em `0` nada muda (e *nada* aqui é bit-a-bit), ela só TIRA, a
//! chave é a IDENTIDADE do elemento e não a posição dele, e sem `id` a posição é a resposta.

use super::*;

const REST: f32 = 0.8;

/// Um stream de `n` elementos a cair sobre o chão, com `id` autorado.
fn falling(n: usize, ids: Option<&[f32]>) -> Stream {
    let mut s = Stream::new(n);
    s.set("P", Column::Vec2(vec![[0.0, -1.0]; n]));
    s.set("vel", Column::Vec2(vec![[0.0, -4.0]; n]));
    if let Some(v) = ids {
        s.set("id", Column::Scalar(v.to_vec()));
    }
    s
}

/// A velocidade vertical de cada elemento depois de um passo contra o chão.
fn bounced(s: &Stream, randomness: f32, seed: u32) -> Vec<f32> {
    let out = collide(
        s,
        SHAPE_PLANE,
        0.0,
        [0.0, 0.0],
        0.0,
        REST,
        0.0,
        (RADIUS_POINT, 0.0, 0.0),
        plane_normal(0.0),
        (randomness, seed),
    );
    match out.get("vel") {
        Some(Column::Vec2(v)) => v.iter().map(|q| q[1]).collect(),
        _ => panic!("o passe escreve `vel`"),
    }
}

/// ⭐ **O CONTROLE, e ele é sobre o VALOR, não sobre um caminho:** `1 − 0·h` é `1` em
/// IEEE-754 para todo `h` finito, então a restituição efectiva é **o mesmo número** que o
/// nó sempre usou — não «indistinguível», o mesmo.
#[test]
fn zero_randomness_is_the_authored_restitution_bit_for_bit() {
    for rest in [0.0f32, 0.25, 0.8, 1.0] {
        for seed in [0u32, 7, 999] {
            for key in [0u32, 1, 4242] {
                assert_eq!(
                    element_restitution(rest, 0.0, seed, key).to_bits(),
                    rest.to_bits(),
                    "rest {rest}, seed {seed}, key {key}"
                );
            }
        }
    }
    // E a mesma afirmação no stream inteiro: doze elementos, todos com o mesmo salto.
    let ids: Vec<f32> = (0..12).map(|i| i as f32).collect();
    let v = bounced(&falling(12, Some(&ids)), 0.0, 7);
    assert!(
        v.windows(2).all(|w| w[0].to_bits() == w[1].to_bits()),
        "sem aleatoriedade todos saltam igual: {v:?}"
    );
}

/// A aleatoriedade **espalha**: com ela no máximo, doze elementos não podem sair todos com o
/// mesmo salto — e nenhum pode sair a saltar MAIS do que o autorado.
#[test]
fn the_randomness_spreads_the_bounce_and_never_raises_it() {
    let ids: Vec<f32> = (0..12).map(|i| i as f32).collect();
    let base = bounced(&falling(12, Some(&ids)), 0.0, 3);
    let spread = bounced(&falling(12, Some(&ids)), 1.0, 3);
    let distinct = {
        let mut v: Vec<u32> = spread.iter().map(|x| x.to_bits()).collect();
        v.sort_unstable();
        v.dedup();
        v.len()
    };
    assert!(
        distinct >= 8,
        "so' {distinct} saltos distintos em 12: {spread:?}"
    );
    for (i, (b, s)) in base.iter().zip(&spread).enumerate() {
        // Sobe = velocidade positiva; «saltar mais» é um número MAIOR.
        assert!(
            *s <= *b + 1e-6,
            "elemento {i}: com acaso saltou {s} contra os {b} autorados -- a lei so' TIRA"
        );
    }
}

/// ⚠️ **A chave é a IDENTIDADE, não a posição.** Baralhar a lista (mantendo os `id`) tem de
/// dar a cada elemento o MESMO salto — senão pôr um `motion.sort` no meio redistribuiria
/// quão saltitante cada partícula é, que é a lei que o `pick` do `motion.duplicator` já paga.
#[test]
fn the_bounce_travels_with_the_id_not_with_the_place_in_the_list() {
    let straight: Vec<f32> = (0..6).map(|i| i as f32).collect();
    let reversed: Vec<f32> = straight.iter().rev().copied().collect();
    let a = bounced(&falling(6, Some(&straight)), 1.0, 5);
    let mut b = bounced(&falling(6, Some(&reversed)), 1.0, 5);
    b.reverse();
    // ⚠️ **A metade JUSTA, e sem ela este gate passava com a lei morta:** se a aleatoriedade
    // não fizer nada, todos saltam igual e a invariância à ordem é trivialmente verdadeira.
    // A prova de mutação apanhou exactamente isso — *uma afirmação que mutação nenhuma mata é
    // uma afirmação sobre nada*.
    assert!(
        a.windows(2).any(|w| w[0].to_bits() != w[1].to_bits()),
        "o espalhamento tem de estar VIVO para esta invariancia dizer algo: {a:?}"
    );
    assert_eq!(a, b, "o salto seguiu a posicao em vez do `id`");
}

/// Sem coluna `id`, a posição é a única resposta disponível — e tem de ser ela, não zero
/// para toda a gente (que daria a todos a mesma sorte, com nome de acaso).
#[test]
fn without_an_id_column_the_place_is_the_key() {
    let v = bounced(&falling(8, None), 1.0, 2);
    let distinct = {
        let mut x: Vec<u32> = v.iter().map(|q| q.to_bits()).collect();
        x.sort_unstable();
        x.dedup();
        x.len()
    };
    assert!(distinct >= 6, "so' {distinct} saltos distintos em 8: {v:?}");
}

/// A semente escolhe QUAL espalhamento, e a mesma semente repete-o — é o que faz um scrub
/// reproduzir a cena.
#[test]
fn the_seed_chooses_the_scatter_and_the_same_seed_repeats_it() {
    let ids: Vec<f32> = (0..10).map(|i| i as f32).collect();
    let a = bounced(&falling(10, Some(&ids)), 1.0, 1);
    let b = bounced(&falling(10, Some(&ids)), 1.0, 1);
    let c = bounced(&falling(10, Some(&ids)), 1.0, 2);
    assert_eq!(
        a, b,
        "a mesma semente e' o mesmo espalhamento, toda cozedura"
    );
    assert_ne!(
        a, c,
        "sementes diferentes escolhem espalhamentos diferentes"
    );
}
