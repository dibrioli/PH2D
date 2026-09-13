//! Os gates do contacto entre peças (doc 109 W2).

use super::*;

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// Um inteiro espalhado em `[0, 1)` — determinístico, para a nuvem ser a mesma em toda corrida.
fn acaso(i: u32, faixa: u32) -> f32 {
    let mut h = i.wrapping_mul(0x9e37_79b9) ^ faixa.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    #[expect(clippy::cast_precision_loss, reason = "24 bits cabem num f32")]
    let v = (h >> 8) as f32 / 16_777_216.0;
    v
}

/// Uma nuvem APERTADA com o que a lei tem de saber tratar: raios mistos, pinos, peças sem colisor
/// e dois pares de centros coincidentes.
fn nuvem(n: u32) -> (Vec<[f32; 2]>, Vec<f32>, Vec<f32>) {
    let lado = f64::from(n).sqrt().ceil();
    #[expect(clippy::cast_possible_truncation, reason = "um lado de grelha pequeno")]
    #[expect(clippy::cast_sign_loss, reason = "idem")]
    let lado = lado as u32;
    let mut p = Vec::new();
    let mut r = Vec::new();
    let mut w = Vec::new();
    for i in 0..n {
        #[expect(clippy::cast_precision_loss, reason = "coordenadas de fixture")]
        let (gx, gy) = ((i % lado) as f32, (i / lado) as f32);
        p.push([
            (gx + acaso(i, 1) - 0.5) * 0.3,
            (gy + acaso(i, 2) - 0.5) * 0.3,
        ]);
        // 1 em 11 sem colisor; os outros entre 0,1 e 0,3.
        r.push(if i % 11 == 0 {
            0.0
        } else {
            0.1 + 0.2 * acaso(i, 3)
        });
        // 1 em 7 é um pino.
        w.push(if i % 7 == 3 { 0.0 } else { 1.0 });
    }
    // Centros coincidentes, de propósito: é o único ramo sem normal.
    p[5] = p[4];
    p[20] = p[19];
    (p, r, w)
}

/// ⭐⭐⭐ **A grelha dá OS MESMOS BITS que todos-os-pares.**
///
/// ⚠️ Igualdade exacta e não tolerância: a promessa do cabeçalho é a ORDEM das somas, e uma ordem
/// trocada dá um resultado a poucos ULP do certo — que uma tolerância engoliria e o gate não veria.
///
/// ⛔ **O que este gate NÃO prova: a lei do par.** Os dois caminhos chamam a mesma `corrigida`, então
/// uma mutação dentro dela muda os dois lados por igual e eles continuam a concordar — medido: o eixo
/// dos centros coincidentes sem o respeito ao índice menor/maior SOBREVIVEU aqui. Ele prova a ordem e
/// o conjunto de vizinhos; a lei tem gates próprios abaixo.
#[test]
fn the_grid_gives_the_same_bits_as_all_pairs() {
    let (p0, r, w) = nuvem(400);
    let mut grelha = p0.clone();
    let mut todos = p0.clone();
    separate(&mut grelha, &r, &w, 8);
    separate_all_pairs(&mut todos, &r, &w, 8);
    // O controlo: a nuvem de facto se mexeu, senão a igualdade seria de duas identidades.
    let mexeu = (0..p0.len()).filter(|&i| p0[i] != todos[i]).count();
    assert!(
        mexeu > 200,
        "a nuvem tinha de estar apertada: so' {mexeu} pecas se mexeram"
    );
    for i in 0..p0.len() {
        assert_eq!(
            (grelha[i][0].to_bits(), grelha[i][1].to_bits()),
            (todos[i][0].to_bits(), todos[i][1].to_bits()),
            "peca {i}: grelha {:?} contra todos-os-pares {:?}",
            grelha[i],
            todos[i]
        );
    }
}

/// ⭐ **Duas peças sobrepostas assentam à SOMA dos raios**, e o ponto médio do par não se mexe.
#[test]
fn two_overlapping_pieces_settle_at_the_sum_of_their_radii() {
    let mut p = vec![[-0.1, 0.0], [0.1, 0.0]];
    separate(&mut p, &[0.3, 0.6], &[1.0, 1.0], 8);
    assert!((dist(p[0], p[1]) - 0.9).abs() < 1e-4, "{p:?}");
    assert!(
        ((p[0][0] + p[1][0]) * 0.5).abs() < 1e-6,
        "o meio do par ficou: {p:?}"
    );
}

/// ⭐ **Duas peças no MESMO ponto separam-se em sentidos opostos, até à soma dos raios.**
///
/// ⚠️ É o único ramo sem normal, e nasceu de uma mutação SOBREVIVENTE: com a normal a ignorar qual
/// dos dois é o índice menor, as duas peças eram empurradas para o mesmo lado e nunca se separavam —
/// e o gate da grelha, que partilha a lei do par, continuava verde.
#[test]
fn two_coincident_pieces_split_apart_in_opposite_directions() {
    let mut p = vec![[0.25, -0.5], [0.25, -0.5]];
    separate(&mut p, &[0.5, 0.5], &[1.0, 1.0], 8);
    assert!((dist(p[0], p[1]) - 1.0).abs() < 1e-4, "{p:?}");
    assert!(
        ((p[0][0] + p[1][0]) * 0.5 - 0.25).abs() < 1e-6
            && ((p[0][1] + p[1][1]) * 0.5 + 0.5).abs() < 1e-6,
        "e o meio do par fica onde os dois estavam: {p:?}"
    );
}

/// ⭐ **Uma peça sem colisor é TRANSPARENTE**: não empurra nem é empurrada.
#[test]
fn a_piece_without_a_collider_is_transparent() {
    let p0 = vec![[0.0, 0.0], [0.05, 0.0]];
    let mut p = p0.clone();
    separate(&mut p, &[0.5, 0.0], &[1.0, 1.0], 8);
    assert_eq!(p, p0, "nenhuma das duas se mexe");
}

/// ⭐ **Um pino é OBSTÁCULO**: não se move, e a outra peça sai inteira do caminho.
#[test]
fn a_pinned_piece_does_not_move_and_the_other_goes_around_it() {
    let mut p = vec![[0.0, 0.0], [0.2, 0.0]];
    separate(&mut p, &[0.5, 0.5], &[0.0, 1.0], 8);
    assert_eq!(p[0], [0.0, 0.0], "o pino ficou");
    assert!(
        (dist(p[0], p[1]) - 1.0).abs() < 1e-4,
        "a livre saiu toda: {p:?}"
    );
}

/// ⭐ **Uma peça sem vizinhos sai com os MESMOS bits** — a lei não toca no que não tem contacto.
#[test]
fn a_piece_with_no_neighbour_is_untouched_to_the_bit() {
    let p0 = vec![[0.123_456_7, -9.876_543], [50.0, 50.0]];
    let mut p = p0.clone();
    separate(&mut p, &[0.4, 0.4], &[1.0, 1.0], 8);
    assert_eq!(p, p0);
}
