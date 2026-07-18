//! Gates da EDT. O oráculo é a **força bruta** — a definição literal da distância —,
//! nunca uma propriedade derivada: uma EDT que "parece certa" é exatamente o defeito
//! que uma aproximação por chanfro produz, e ele passa despercebido em qualquer teste
//! de forma redonda.

use super::sq_distance_to_set;

/// A definição, ao pé da letra: `min` sobre TODO ponto do conjunto.
fn brute_force(w: usize, h: usize, set: &[bool]) -> Vec<u32> {
    let mut out = vec![u32::MAX; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut best = u32::MAX;
            for (sy, sx) in (0..h).flat_map(|sy| (0..w).map(move |sx| (sy, sx))) {
                if !set[sy * w + sx] {
                    continue;
                }
                let dx = x.abs_diff(sx) as u32;
                let dy = y.abs_diff(sy) as u32;
                best = best.min(dx * dx + dy * dy);
            }
            out[y * w + x] = best;
        }
    }
    out
}

/// Ruído determinístico (splitmix64 — o mesmo hash-determinismo do resto da engine;
/// nada de `thread_rng`, HR-5).
fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// **O gate que morde: a EDT é EXATA.**
///
/// Varre formatos de grade e densidades de conjunto (inclusive um único ponto e
/// quase-tudo-preenchido) e compara **pixel a pixel** com a força bruta. Uma
/// aproximação por chanfro — a implementação "óbvia" — falha aqui na diagonal, que é
/// justamente onde o trapped-ball decide se a bola passa por um vão.
#[test]
fn the_transform_is_exact_against_brute_force() {
    let mut seed = 0x5EED_1234_u64;
    for &(w, h) in &[(1, 1), (1, 9), (9, 1), (7, 5), (16, 16), (23, 11)] {
        for &fill_num in &[1u64, 3, 8, 25, 60, 95] {
            let mut set = vec![false; w * h];
            let mut any = false;
            for s in set.iter_mut() {
                *s = splitmix(&mut seed) % 100 < fill_num;
                any |= *s;
            }
            if !any {
                set[(h / 2) * w + w / 2] = true;
            }
            let got = sq_distance_to_set(w, h, |i| set[i]);
            let want = brute_force(w, h, &set);
            assert_eq!(
                got, want,
                "EDT != forca bruta em {w}x{h}, densidade {fill_num}%"
            );
        }
    }
}

/// Um ponto só: a distância é o raio ao quadrado, e isso é verificável de cabeça.
#[test]
fn a_single_point_gives_the_squared_radius() {
    let (w, h) = (5, 5);
    let d = sq_distance_to_set(w, h, |i| i == 2 * w + 2); // centro
    assert_eq!(d[2 * w + 2], 0, "o proprio ponto esta a distancia 0");
    assert_eq!(d[2 * w + 3], 1, "vizinho lateral");
    assert_eq!(d[3 * w + 3], 2, "vizinho diagonal = 1+1");
    assert_eq!(d[0], 8, "o canto (0,0) esta a (2,2) => 4+4");
}

/// **Conjunto vazio = infinitamente longe, NUNCA zero.**
///
/// Zero significa "está no conjunto". Se a EDT devolvesse zero para um grid sem
/// fronteira nenhuma, o trapped-ball concluiria que a bola não cabe em lugar nenhum —
/// e o modo inteiro ficaria inerte, sem erro e sem sintoma. É o par do
/// [[feedback_absence_gate_needs_a_presence_sibling]]: o valor de "não há resposta"
/// tem de ser distinguível do valor de "a resposta é 0".
///
/// ⚠️ E o número tem de ser `u32::MAX` de verdade: a sentinela interna
/// (`4·(max²+1)`) **cabe num `u32`**, então uma conversão ingênua a entregaria como
/// se fosse uma distância medida.
#[test]
fn an_empty_set_is_infinitely_far_not_zero() {
    let d = sq_distance_to_set(8, 8, |_| false);
    assert!(
        d.iter().all(|&v| v == u32::MAX),
        "conjunto vazio tem de dar u32::MAX em todo pixel, veio {:?}",
        &d[..4]
    );
}

/// **A exatidão sobrevive além da faixa de inteiros do `f32`.**
///
/// É este gate que justifica o `u32` do buffer. `f32` só representa inteiros exatos
/// até 2²⁴ = 16.777.216; a distância medida aqui é 5999² = 35.988.001, que é ÍMPAR e
/// está acima de 2²⁵ — guardá-la em `f32` a arredondaria para um múltiplo de 4. O
/// resultado ficaria *quase* certo, que é a categoria de bug mais cara desta engine.
#[test]
fn the_transform_stays_exact_past_the_f32_integer_range() {
    let (w, h) = (6000, 2);
    let d = sq_distance_to_set(w, h, |i| i == 0);
    let far = d[w - 1]; // (5999, 0)
    assert_eq!(far, 5999 * 5999, "a distancia longa tem de ser EXATA");
    assert_ne!(
        far as f32 as u32, far,
        "premissa do gate: este valor NAO e representavel em f32 \
         (se passasse a ser, o gate deixou de provar o que diz)"
    );
}

// ⚠️ **Um gate de espelhamento foi escrito aqui e REMOVIDO** (2026-07-18), e vale
// registrar por quê para ninguém o reescrever achando que falta.
//
// A intenção era pegar troca de eixo entre as passadas separáveis. Ele nasceu VERDE e
// **nenhuma das três mutações o derrubou** — inclusive a que inverte a leitura da
// linha, porque inverter a leitura é *ele próprio* um espelhamento: os dois lados da
// comparação se movem juntos e a igualdade se mantém sobre um resultado errado.
//
// E a propriedade que ele pretendia guardar já está guardada, com um oráculo mais
// forte: o `the_transform_is_exact_against_brute_force` varre grades **não-quadradas**
// de propósito (9×1, 1×9, 7×5, 23×11), e força bruta é a DEFINIÇÃO — uma troca de eixo
// não sobrevive a ela. Um gate que mutação nenhuma mata, e cuja propriedade um gate
// mais forte já cobre, é verde decorativo: ele só aumenta a conta de testes e a
// confiança, sem aumentar a proteção.
