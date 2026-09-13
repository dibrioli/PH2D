//! Os gates do contacto entre peças (doc 109 W2 e §5).

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

/// Discos centrados de raios `r` — `0` é uma peça sem colisor.
fn discos(r: &[f32]) -> Vec<Option<Colisor>> {
    r.iter()
        .map(|&r| (r > 0.0).then(|| Colisor::disco(r)))
        .collect()
}

/// Uma nuvem APERTADA com o que a lei tem de saber tratar: discos e caixas (giradas e fora do
/// centro), pinos, peças sem colisor e dois pares de centros coincidentes.
fn nuvem(n: u32) -> (Vec<[f32; 2]>, Vec<Option<Colisor>>, Vec<f32>) {
    let lado = f64::from(n).sqrt().ceil();
    #[expect(clippy::cast_possible_truncation, reason = "um lado de grelha pequeno")]
    #[expect(clippy::cast_sign_loss, reason = "idem")]
    let lado = lado as u32;
    let mut p = Vec::new();
    let mut c = Vec::new();
    let mut w = Vec::new();
    for i in 0..n {
        #[expect(clippy::cast_precision_loss, reason = "coordenadas de fixture")]
        let (gx, gy) = ((i % lado) as f32, (i / lado) as f32);
        p.push([
            (gx + acaso(i, 1) - 0.5) * 0.3,
            (gy + acaso(i, 2) - 0.5) * 0.3,
        ]);
        c.push(match i % 11 {
            // 1 em 11 sem colisor.
            0 => None,
            // 3 em 11 caixas, giradas e com o centro fora de `P`.
            k if k % 3 == 0 => declarado(
                None,
                Some([0.05 + 0.1 * acaso(i, 3), 0.05 + 0.1 * acaso(i, 4)]),
                [0.05 * (acaso(i, 5) - 0.5), 0.0],
                [1.0, 1.0],
                360.0 * acaso(i, 6),
            ),
            _ => Some(Colisor::disco(0.1 + 0.2 * acaso(i, 3))),
        });
        // 1 em 7 é um pino.
        w.push(if i % 7 == 3 { 0.0 } else { 1.0 });
    }
    // Centros coincidentes, de propósito: é o único ramo sem normal.
    p[5] = p[4];
    p[20] = p[19];
    (p, c, w)
}

/// ⭐⭐⭐ **A grelha dá OS MESMOS BITS que todos-os-pares** — com discos E caixas.
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
    let (p0, c, w) = nuvem(400);
    assert!(
        c.iter()
            .flatten()
            .any(|c| matches!(c.forma, Forma::Caixa { .. })),
        "a nuvem tem de ter caixas"
    );
    let mut grelha = p0.clone();
    let mut todos = p0.clone();
    separate(&mut grelha, &c, &w, 8);
    separate_all_pairs(&mut todos, &c, &w, 8);
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
    separate(&mut p, &discos(&[0.3, 0.6]), &[1.0, 1.0], 8);
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
    separate(&mut p, &discos(&[0.5, 0.5]), &[1.0, 1.0], 8);
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
    separate(&mut p, &discos(&[0.5, 0.0]), &[1.0, 1.0], 8);
    assert_eq!(p, p0, "nenhuma das duas se mexe");
}

/// ⭐ **Um pino é OBSTÁCULO**: não se move, e a outra peça sai inteira do caminho.
#[test]
fn a_pinned_piece_does_not_move_and_the_other_goes_around_it() {
    let mut p = vec![[0.0, 0.0], [0.2, 0.0]];
    separate(&mut p, &discos(&[0.5, 0.5]), &[0.0, 1.0], 8);
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
    separate(&mut p, &discos(&[0.4, 0.4]), &[1.0, 1.0], 8);
    assert_eq!(p, p0);
}

const SEM_GIRO: [f32; 2] = [1.0, 0.0];

/// ⭐⭐⭐ **Duas CAIXAS lado a lado encostam pela FACE** — à soma das meias larguras, e não à soma
/// dos círculos à volta delas (o `41 %` de ar do report do doc 109 §5).
#[test]
fn two_boxes_side_by_side_touch_face_to_face() {
    let c = vec![
        Some(Colisor::caixa([0.3, 1.0], SEM_GIRO)),
        Some(Colisor::caixa([0.5, 1.0], SEM_GIRO)),
    ];
    let mut p = vec![[-0.1, 0.0], [0.1, 0.0]];
    separate(&mut p, &c, &[1.0, 1.0], 8);
    assert!((p[1][0] - p[0][0] - 0.8).abs() < 1e-4, "{p:?}");
    assert_eq!((p[0][1], p[1][1]), (0.0, 0.0), "e nenhuma saiu na vertical");
    // O CONTROLO: os círculos à volta das duas afastá-las-iam muito mais.
    let circulos =
        c[0].map(|c| c.alcance()).unwrap_or(0.0) + c[1].map(|c| c.alcance()).unwrap_or(0.0);
    assert!(
        p[1][0] - p[0][0] < circulos * 0.5,
        "{p:?} contra {circulos}"
    );
}

/// ⭐⭐ **Uma caixa pousada noutra sai pelo eixo de MENOR sobreposição** — sobe, não escorrega para o
/// lado.
#[test]
fn a_stacked_box_is_pushed_along_the_axis_of_least_overlap() {
    let c = vec![
        Some(Colisor::caixa([1.0, 0.5], SEM_GIRO)),
        Some(Colisor::caixa([1.0, 0.5], SEM_GIRO)),
    ];
    let mut p = vec![[0.0, 0.0], [0.2, 0.9]];
    separate(&mut p, &c, &[0.0, 1.0], 8);
    assert_eq!(p[0], [0.0, 0.0], "o pino ficou");
    assert!(
        (p[1][0] - 0.2).abs() < 1e-6 && (p[1][1] - 1.0).abs() < 1e-5,
        "a de cima assentou na face: {p:?}"
    );
}

/// ⭐⭐ **Uma caixa GIRADA colide pelos eixos DELA** — duas caixas deitadas a 90° ficam em pé, e
/// encostam pela largura que o giro lhes deu.
#[test]
fn a_turned_box_collides_along_its_own_axes() {
    let em_pe = [0.0, 1.0];
    let c = vec![
        Some(Colisor::caixa([1.0, 0.2], em_pe)),
        Some(Colisor::caixa([1.0, 0.2], em_pe)),
    ];
    let mut p = vec![[-0.1, 0.0], [0.1, 0.0]];
    separate(&mut p, &c, &[1.0, 1.0], 8);
    assert!(
        (p[1][0] - p[0][0] - 0.4).abs() < 1e-4,
        "em pe' sao 0,4 de largo: {p:?}"
    );
}

/// ⭐⭐ **Um disco pousa na FACE de uma caixa** — não no círculo à volta dela.
#[test]
fn a_disc_rests_on_the_face_of_a_box() {
    let c = vec![
        Some(Colisor::caixa([1.0, 0.1], SEM_GIRO)),
        Some(Colisor::disco(0.2)),
    ];
    let mut p = vec![[0.0, 0.0], [0.9, 0.25]];
    separate(&mut p, &c, &[0.0, 1.0], 8);
    assert!(
        (p[1][0] - 0.9).abs() < 1e-6 && (p[1][1] - 0.3).abs() < 1e-5,
        "{p:?}"
    );
}

/// ⭐ **Um disco cujo centro ENTROU na caixa sai pela face mais próxima.**
#[test]
fn a_disc_that_entered_a_box_leaves_by_the_nearest_face() {
    let c = vec![
        Some(Colisor::caixa([1.0, 0.5], SEM_GIRO)),
        Some(Colisor::disco(0.1)),
    ];
    let mut p = vec![[0.0, 0.0], [0.2, 0.4]];
    separate(&mut p, &c, &[0.0, 1.0], 8);
    assert!(
        (p[1][0] - 0.2).abs() < 1e-6 && (p[1][1] - 0.6).abs() < 1e-5,
        "{p:?}"
    );
}

/// ⭐ **Duas caixas no MESMO ponto separam-se em sentidos opostos** — pelo eixo de menor
/// sobreposição, com o meio do par no sítio.
#[test]
fn two_coincident_boxes_split_apart_in_opposite_directions() {
    let c = vec![
        Some(Colisor::caixa([0.5, 1.0], SEM_GIRO)),
        Some(Colisor::caixa([0.5, 1.0], SEM_GIRO)),
    ];
    let mut p = vec![[0.3, -0.2], [0.3, -0.2]];
    separate(&mut p, &c, &[1.0, 1.0], 8);
    assert!(
        (p[1][0] - p[0][0]).abs() > 0.999 && (p[1][0] - p[0][0]).abs() < 1.0001,
        "{p:?}"
    );
    assert!(
        ((p[0][0] + p[1][0]) * 0.5 - 0.3).abs() < 1e-6,
        "o meio do par ficou: {p:?}"
    );
}

/// ⭐⭐ **A PORTA da declaração** — a caixa ganha ao raio, a caixa vazia devolve a vez ao raio, e o
/// centro escala com o `size` COM sinal e gira com a peça.
#[test]
fn the_declaration_door_reads_box_first_then_radius_and_carries_the_offset() {
    let caixa = declarado(Some(0.5), Some([0.2, 0.3]), [0.0, 0.0], [2.0, -2.0], 0.0);
    assert_eq!(
        caixa,
        Some(Colisor::caixa([0.4, 0.6], SEM_GIRO)),
        "a caixa ganha"
    );

    let disco = declarado(Some(0.5), Some([0.0, 0.0]), [0.0, 0.0], [2.0, -3.0], 0.0);
    assert_eq!(
        disco,
        Some(Colisor::disco(1.5)),
        "caixa vazia: o raio, x max|size|"
    );

    assert_eq!(
        declarado(Some(0.0), Some([0.0, 0.0]), [0.0, 0.0], [1.0, 1.0], 0.0),
        None
    );
    assert_eq!(declarado(None, None, [0.0, 0.0], [1.0, 1.0], 0.0), None);

    // Centro `[1, 0]`, espelhado em x e girado 90° ⇒ `[-2, 0]` girado = `[0, -2]`.
    let girado = declarado(Some(1.0), None, [1.0, 0.0], [-2.0, 1.0], 90.0).expect("declara");
    assert!(
        girado.desvio[0].abs() < 1e-5 && (girado.desvio[1] + 2.0).abs() < 1e-5,
        "{:?}",
        girado.desvio
    );
}

/// ⭐ **Sem as colunas, a pergunta nem começa** — é o que mantém um stream sem colisor ao bit.
#[test]
fn a_stream_without_collider_columns_answers_none() {
    let s = Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]));
    assert!(colisores(&s).is_none());
    let com = s.with(
        COLLIDER_BOX_COLUMN,
        Column::Vec2(vec![[0.5, 0.5], [0.0, 0.0]]),
    );
    let c = colisores(&com).expect("declara");
    assert_eq!(c, vec![Some(Colisor::caixa([0.5, 0.5], SEM_GIRO)), None]);
}
