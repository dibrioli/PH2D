//! Os gates do contacto entre peças (doc 109 W2, §5 e §6).

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

/// A inércia inversa DERIVADA de cada peça — o que o produto usa com a rotação destravada.
fn inercias(c: &[Option<Colisor>], w: &[f32]) -> Vec<f32> {
    c.iter()
        .zip(w)
        .map(|(c, w)| c.map_or(0.0, |c| c.inv_inercia(*w)))
        .collect()
}

/// Uma corrida com a rotação TRAVADA — a lei de posição pura, que é o que os gates de geometria
/// afirmam.
fn corre(p: &mut [[f32; 2]], c: &[Option<Colisor>], w: &[f32]) {
    let zeros = vec![0.0; p.len()];
    let mut saida = Saida {
        giro: &mut vec![0.0; p.len()],
    };
    separate(p, &mut saida, &Pecas::novas(c, w, &zeros), 8);
}

/// Uma corrida com a rotação DESTRAVADA; devolve o giro de cada peça, em graus.
fn corre_girando(p: &mut [[f32; 2]], c: &[Option<Colisor>], w: &[f32]) -> Vec<f32> {
    let inv = inercias(c, w);
    let mut giro = vec![0.0; p.len()];
    separate(
        p,
        &mut Saida { giro: &mut giro },
        &Pecas::novas(c, w, &inv),
        8,
    );
    giro
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

/// ⭐⭐⭐ **A grelha dá OS MESMOS BITS que todos-os-pares** — com discos, caixas E rotação.
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
    let inv = inercias(&c, &w);
    let (mut grelha, mut todos) = (p0.clone(), p0.clone());
    let (mut g_grelha, mut g_todos) = (vec![0.0; p0.len()], vec![0.0; p0.len()]);
    let pecas = Pecas::novas(&c, &w, &inv);
    separate(
        &mut grelha,
        &mut Saida {
            giro: &mut g_grelha,
        },
        &pecas,
        8,
    );
    separate_all_pairs(&mut todos, &mut Saida { giro: &mut g_todos }, &pecas, 8);
    // Os controlos: a nuvem mexeu-se, e alguém RODOU (senão a igualdade do giro seria de dois zeros).
    let mexeu = (0..p0.len()).filter(|&i| p0[i] != todos[i]).count();
    assert!(
        mexeu > 200,
        "a nuvem tinha de estar apertada: so' {mexeu} pecas se mexeram"
    );
    assert!(
        g_todos.iter().filter(|g| g.abs() > 1e-3).count() > 20,
        "com a rotação destravada as caixas tinham de rodar: {:?}",
        &g_todos[..8]
    );
    for i in 0..p0.len() {
        assert_eq!(
            (
                grelha[i][0].to_bits(),
                grelha[i][1].to_bits(),
                g_grelha[i].to_bits()
            ),
            (
                todos[i][0].to_bits(),
                todos[i][1].to_bits(),
                g_todos[i].to_bits()
            ),
            "peca {i}: grelha {:?}/{} contra todos-os-pares {:?}/{}",
            grelha[i],
            g_grelha[i],
            todos[i],
            g_todos[i]
        );
    }
}

/// ⭐ **Duas peças sobrepostas assentam à SOMA dos raios**, e o ponto médio do par não se mexe.
#[test]
fn two_overlapping_pieces_settle_at_the_sum_of_their_radii() {
    let mut p = vec![[-0.1, 0.0], [0.1, 0.0]];
    corre(&mut p, &discos(&[0.3, 0.6]), &[1.0, 1.0]);
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
    corre(&mut p, &discos(&[0.5, 0.5]), &[1.0, 1.0]);
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
    corre(&mut p, &discos(&[0.5, 0.0]), &[1.0, 1.0]);
    assert_eq!(p, p0, "nenhuma das duas se mexe");
}

/// ⭐ **Um pino é OBSTÁCULO**: não se move, e a outra peça sai inteira do caminho.
#[test]
fn a_pinned_piece_does_not_move_and_the_other_goes_around_it() {
    let mut p = vec![[0.0, 0.0], [0.2, 0.0]];
    corre(&mut p, &discos(&[0.5, 0.5]), &[0.0, 1.0]);
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
    corre(&mut p, &discos(&[0.4, 0.4]), &[1.0, 1.0]);
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
    corre(&mut p, &c, &[1.0, 1.0]);
    assert!((p[1][0] - p[0][0] - 0.8).abs() < 1e-4, "{p:?}");
    assert_eq!((p[0][1], p[1][1]), (0.0, 0.0), "e nenhuma saiu na vertical");
    // O CONTROLO: os círculos à volta das duas afastá-las-iam muito mais.
    let circulos = c[0].map_or(0.0, |c| c.alcance()) + c[1].map_or(0.0, |c| c.alcance());
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
    corre(&mut p, &c, &[0.0, 1.0]);
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
    corre(&mut p, &c, &[1.0, 1.0]);
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
    corre(&mut p, &c, &[0.0, 1.0]);
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
    corre(&mut p, &c, &[0.0, 1.0]);
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
    corre(&mut p, &c, &[1.0, 1.0]);
    let vao = (p[1][0] - p[0][0]).abs();
    assert!((vao - 1.0).abs() < 1e-3, "{p:?}");
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

// ── A ROTAÇÃO (doc 109 §6) ───────────────────────────────────────────────────────────

/// ⭐⭐⭐ **Uma caixa cujo CENTRO passa da beira tomba** — e o `Lock Rotation` (a inércia a zero)
/// trava-a sem lhe mudar mais nada de essencial.
///
/// A prancha é um pino largo que acaba em `x = 1,0`; a caixa livre pousa com o **centro em `1,1`**,
/// para lá da beira, e tomba.
///
/// ⚠️⚠️ **A 1.ª redacção deste gate punha o centro em `0,9` — DENTRO do apoio — e exigia que ela
/// tombasse.** Ele passava porque o contacto de UM ponto dá binário a uma caixa que está apoiada,
/// que é exactamente o defeito que o encosto de dois pontos veio curar (doc 111 §5.10): *o gate
/// tinha o defeito escrito dentro dele, e por isso defendia-o*. A varredura que o apanhou:
///
/// ```text
///   centro |  apoiado? |    giro
///     0,60 |       SIM |   0,000
///     0,80 |       SIM |   0,010
///     0,90 |       SIM |  −0,208   ← a fixtura velha, a exigir |giro| > 1
///     1,05 |       NAO | −10,544
///     1,10 |       NAO | −13,998
/// ```
#[test]
fn a_box_whose_centre_clears_the_edge_topples_and_the_lock_stops_it() {
    let c = vec![
        Some(Colisor::caixa([1.0, 0.1], SEM_GIRO)),
        Some(Colisor::caixa([0.25, 0.25], SEM_GIRO)),
    ];
    let (w, p0) = ([0.0, 1.0], vec![[0.0, 0.0], [1.1, 0.25]]);

    let mut solta = p0.clone();
    let giro = corre_girando(&mut solta, &c, &w);
    assert!(
        giro[0] == 0.0,
        "o pino nao roda (inercia inversa zero): {giro:?}"
    );
    assert!(
        giro[1].abs() > 1.0,
        "com o centro FORA do apoio ela tem de TOMBAR: {giro:?}"
    );

    let mut travada = p0.clone();
    corre(&mut travada, &c, &w);
    assert!(
        travada[1][1] > p0[1][1],
        "travada ela continua a ser empurrada para cima: {travada:?}"
    );
}

/// ⭐⭐⭐ **E a METADE QUE O ENCOSTO DE DOIS PONTOS COMPRA: uma caixa APOIADA PELO CENTRO não tomba.**
///
/// É a cura do 5.º report do dono (doc 111 §5.10) escrita como propriedade: com um contacto
/// pontual o empurrão age no meio do trecho, que **não** está debaixo do centro de massa, e a caixa
/// ganha binário sem ninguém lhe tocar — na `=114` isso crescia `×1,4` por tique até ela tombar
/// nos `45°`.
///
/// ⚠️ **A barra é o CONTRASTE, não um número escolhido:** a mesma caixa com o centro fora da beira
/// (`1,1`) roda `−14,0°`, e com ele dentro (`0,9`) tem de ficar **duas ordens de grandeza** abaixo.
#[test]
fn a_box_supported_under_its_centre_does_not_topple() {
    let c = vec![
        Some(Colisor::caixa([1.0, 0.1], SEM_GIRO)),
        Some(Colisor::caixa([0.25, 0.25], SEM_GIRO)),
    ];
    let mut apoiada = vec![[0.0, 0.0], [0.9, 0.25]];
    let dentro = corre_girando(&mut apoiada, &c, &[0.0, 1.0])[1].abs();

    let mut fora = vec![[0.0, 0.0], [1.1, 0.25]];
    let alem = corre_girando(&mut fora, &c, &[0.0, 1.0])[1].abs();

    assert!(
        dentro < 1.0,
        "apoiada pelo centro ela NAO tomba: {dentro:.3}°"
    );
    assert!(
        alem > 10.0 * dentro,
        "e o contraste com a que passa a beira e' de uma ordem de grandeza: {dentro:.3}° contra {alem:.3}°"
    );
}

/// ⭐⭐⭐ **Uma caixa pousada DE CHAPA não roda** — e é isto que o recorte das faces compra: com o
/// vértice mais fundo no lugar do meio do trecho, uma pilha parada tombava sozinha.
///
/// ⚠️⚠️ **E a separação CONVERGE, não enviesa** — a distinção que o encosto de dois pontos obrigou
/// a medir. Com dois pontos, cada um carrega o termo de rotação na massa efectiva dele
/// (`k = w + invI·b²`, e `b` nas pontas é maior que no meio), logo cada varredura corrige MENOS e
/// são precisas mais. Medido:
///
/// ```text
///   varreduras |        y | residuo
///            8 | 0,899160 | 8,4e-4
///           16 | 0,899986 | 1,4e-5
///           32 | 0,900000 | 0,0      ← exacto
/// ```
///
/// ⛔ *Um resíduo que ENCOLHE com as varreduras é convergência; um que fica é viés* — e por isso o
/// gate mede as DUAS pontas, em vez de afrouxar a tolerância até a de `8` passar.
#[test]
fn a_box_resting_flat_on_another_does_not_turn() {
    let c = vec![
        Some(Colisor::caixa([1.0, 0.5], SEM_GIRO)),
        Some(Colisor::caixa([0.4, 0.4], SEM_GIRO)),
    ];
    let mut p = vec![[0.0, 0.0], [0.0, 0.85]];
    let giro = corre_girando(&mut p, &c, &[0.0, 1.0]);
    assert!(
        giro[1].abs() < 1e-3,
        "de chapa o binario e' zero: {giro:?} · {p:?}"
    );
    // Às `8` varreduras do produto: já lá quase, e o resíduo é o MEDIDO acima.
    assert!((p[1][1] - 0.9).abs() < 1e-3, "e ela assenta na face: {p:?}");
    // ⭐ E com varreduras a chegar ela assenta EXACTAMENTE — a prova de que não há viés.
    let mut convergida = vec![[0.0, 0.0], [0.0, 0.85]];
    let inv = inercias(&c, &[0.0, 1.0]);
    let mut g = vec![0.0; 2];
    separate(
        &mut convergida,
        &mut Saida { giro: &mut g },
        &Pecas::novas(&c, &[0.0, 1.0], &inv),
        32,
    );
    // ⚠️⚠️ **A BARRA MUDOU DE DONO, e a mudança é o preço do `REPOUSO_VISIVEL`** (report do dono de
    // 18/09). Ela era `1e-6` — escrita quando o laço varria SEMPRE o tecto inteiro — e hoje o
    // produto pára quando ninguém mais se mexe de forma visível, deixando um resíduo da ordem do
    // limiar. ⇒ a barra é **DERIVADA da constante**, nunca um número novo: o resíduo tem de caber
    // em poucos limiares, e um viés real (que não encolhe) estoura-a por ordens de grandeza.
    let residuo = (convergida[1][1] - 0.9).abs();
    let teto = 8.0 * REPOUSO_VISIVEL * c[1].expect("a caixa de cima tem colisor").alcance();
    assert!(
        residuo < teto,
        "o residuo ({residuo:.3e}) tem de caber no repouso visivel ({teto:.3e}): {convergida:?}"
    );
    // ⭐ **E o CONTROLO do viés, no caminho que NUNCA pára cedo:** é a referência que prova que a
    // lei assenta exactamente — sem esta metade, um viés sistemático esconder-se-ia atrás da barra
    // nova.
    let mut sem_atalho = vec![[0.0, 0.0], [0.0, 0.85]];
    let mut g_ref = vec![0.0; 2];
    separate_all_pairs(
        &mut sem_atalho,
        &mut Saida { giro: &mut g_ref },
        &Pecas::novas(&c, &[0.0, 1.0], &inv),
        256,
    );
    assert!(
        (sem_atalho[1][1] - 0.9).abs() < 1e-6,
        "sem atalho nenhum a caixa assenta EXACTAMENTE: {sem_atalho:?}"
    );
    assert!(g[1].abs() < 1e-3, "e o binario continua zero: {g:?}");
}

/// ⭐⭐ **A inércia inversa sai da FORMA e do peso** — e um pino (ou um colisor degenerado) nunca roda.
#[test]
fn the_inverse_inertia_comes_from_the_shape_and_a_pin_never_turns() {
    // Caixa de meias `0,5`: `I = m(hx² + hy²)/3` ⇒ `invI = 3w / 0,5 = 6w`.
    let caixa = Colisor::caixa([0.5, 0.5], SEM_GIRO);
    assert!((caixa.inv_inercia(1.0) - 6.0).abs() < 1e-4);
    assert!((caixa.inv_inercia(0.5) - 3.0).abs() < 1e-4);
    // Disco de raio `0,5`: `I = m r²/2` ⇒ `invI = 2w / 0,25 = 8w`.
    assert!((Colisor::disco(0.5).inv_inercia(1.0) - 8.0).abs() < 1e-4);
    // Um pino não roda, aconteça o que acontecer.
    assert_eq!(caixa.inv_inercia(0.0), 0.0);
    assert_eq!(caixa.inv_inercia(f32::NAN), 0.0);
}

/// ⭐⭐ **A COLUNA manda: `0` trava, ausente DERIVA** — a porta que o cartão da forma usa para o
/// botão `Lock Rotation`.
#[test]
fn the_inertia_column_locks_and_the_absent_one_derives() {
    let base = Stream::new(2).with(
        COLLIDER_BOX_COLUMN,
        Column::Vec2(vec![[0.5, 0.5], [0.5, 0.5]]),
    );
    let c = colisores(&base).expect("declara");
    let pesos = [1.0, 1.0];
    let derivada = inv_inercias(&base, &c, &pesos);
    assert!(
        (derivada[0] - 6.0).abs() < 1e-4 && (derivada[1] - 6.0).abs() < 1e-4,
        "{derivada:?}"
    );
    let travado = base.with(INV_INERTIA_COLUMN, Column::Scalar(vec![0.0, 0.0]));
    assert_eq!(inv_inercias(&travado, &c, &pesos), vec![0.0, 0.0]);
}

// ───────────────────────── §7 · O MATERIAL E O ATRITO ─────────────────────────

/// ⭐⭐⭐ **A CONTA QUE EXPLICA O REPORT** (doc 109 §7 — *«os círculos não rotacionam com a
/// colisão»*): num disco a alavanca da NORMAL é **exactamente zero** e a da TANGENTE é o raio
/// inteiro. Não «pequena»: `0.0` ao bit, em todas as três rotas que produzem um contacto de disco.
///
/// ⇒ *nenhuma lei que só empurre ao longo da normal pode rodar um círculo, com que número for.*
#[test]
fn a_disc_has_no_lever_on_the_normal_and_all_of_it_on_the_tangent() {
    let d = Colisor::disco(0.5);
    let casos: [(&str, Contacto, [f32; 2]); 3] = [
        // disco × disco
        (
            "disco x disco",
            contato(&d, [0.0, 0.0], &d, [0.95, 0.0], false).expect("tocam"),
            [0.0, 0.0],
        ),
        // disco × caixa, por fora
        (
            "disco x caixa",
            contato(
                &Colisor::caixa([1.0, 1.0], SEM_GIRO),
                [0.0, 0.0],
                &d,
                [0.0, 1.45],
                false,
            )
            .expect("tocam"),
            [0.0, 1.45],
        ),
        // o ponto de suporte contra um plano
        (
            "disco x plano",
            Contacto {
                normal: [0.0, 1.0],
                penetracao: 0.1,
                ponto: d.ponto_de_suporte([0.0, 0.4], [0.0, 1.0]),
            },
            [0.0, 0.4],
        ),
    ];
    for (nome, c, centro) in casos {
        assert_eq!(
            c.braco(centro).to_bits(),
            0.0_f32.to_bits(),
            "{nome}: a alavanca da normal num disco tem de ser ZERO ao bit, e leu {}",
            c.braco(centro)
        );
        // ⚠️ **E LONGE DA ORIGEM ela é ruído, não zero ao bit:** o braço é `ponto − centro`, e a
        // `x = 300` isso subtrai dois números mil vezes maiores do que a diferença. O que fica é
        // cancelamento de `f32` — medido abaixo contra o RAIO, que é a grandeza com que ele
        // compete. *É por isso que os gates de cena usam uma barra de ruído e não um zero.*
        let longe = [centro[0] + 300.0, centro[1] - 120.0];
        let c_longe = Contacto {
            ponto: [c.ponto[0] + 300.0, c.ponto[1] - 120.0],
            ..c
        };
        assert!(
            c_longe.braco(longe).abs() < 1e-4,
            "{nome}: longe da origem a alavanca tem de ser RUIDO, e leu {}",
            c_longe.braco(longe)
        );
        // ⚠️ `~o raio`, e não o raio exacto: a alavanca é a distância do centro ao PONTO, que
        // dentro da penetração fica a `R` menos o quanto se afundou. As três fixturas afundam
        // `0,05` de um raio de `0,5`, logo a barra é `0,44` e não `0,5`.
        assert!(
            c.braco_tangente(centro).abs() > 0.44,
            "{nome}: a alavanca da tangente tem de ser ~o raio, e leu {}",
            c.braco_tangente(centro)
        );
    }
}

/// ⭐ **O PAR combina-se pelas leis do Box2D**: o atrito pela média GEOMÉTRICA (uma peça de gelo
/// desliza contra tudo) e o salto pelo MAIOR (uma bola saltitante salta contra uma parede morta).
#[test]
fn the_pair_takes_the_geometric_mean_of_friction_and_the_livelier_bounce() {
    assert_eq!(super::atrito::mu(0.0, 1.0), 0.0, "gelo contra lixa e' gelo");
    assert!((super::atrito::mu(0.25, 0.64) - 0.4).abs() < 1e-6);
    assert_eq!(super::atrito::salto(0.0, 0.9), 0.9, "a mais viva manda");
    // ⚠️ O tecto passou de `1` para `2` em 2026-09-13 (§7.8) — e é lido da coluna, nunca escrito.
    assert_eq!(
        super::atrito::salto(9.0, 0.1),
        ph2d_nodegraph::attr::BOUNCE_MAX,
        "e nunca passa do tecto da coluna"
    );
}

/// ⭐⭐ **A ausência das colunas é o material LISO** — a porta que o `sim.step` e o `sim.collide`
/// perguntam, e a razão de nenhuma cena de hoje mudar.
#[test]
fn a_stream_without_material_columns_is_ice() {
    let vazio = Stream::new(3);
    assert!(
        materiais(&vazio).is_none(),
        "ninguem declarou: `None`, e nao `Some(LISO)` — ver o doc da porta"
    );
    let s = Stream::new(2)
        .with(
            ph2d_nodegraph::attr::FRICTION_COLUMN,
            Column::Scalar(vec![0.5, -1.0]),
        )
        .with(
            ph2d_nodegraph::attr::BOUNCE_COLUMN,
            Column::Scalar(vec![f32::NAN, 3.0]),
        );
    let m = materiais(&s).expect("declarou");
    assert_eq!(m[0].atrito, 0.5);
    assert_eq!(m[0].salto, 0.0, "um NaN nao e' um pedido");
    assert_eq!(m[1].atrito, 0.0, "um negativo nao e' um pedido");
    // ⚠️ **`2` e não `1` desde a ordem do dono de 2026-09-13** (§7.8): a faixa do salto é o DOBRO
    // da de todo motor, e o tecto vive na coluna (`BOUNCE_MAX`).
    assert_eq!(
        m[1].salto,
        ph2d_nodegraph::attr::BOUNCE_MAX,
        "e a faixa e' a da COLUNA"
    );
}

/// ⭐⭐⭐ **O SALTO E O ATRITO PARAM AMBOS EM `1`** — ordem do dono (2026-09-15: *«Limite Bounciness
/// para máximo de 1»*), que REVERTE a dele própria de 2026-09-13 (*«quero mais capacidade de
/// Bounciness — de zero até o dobro do máximo atual»*).
///
/// ⚠️ **A medição que abriu a faixa para `2` continua válida e está no
/// [`ph2d_nodegraph::attr::BOUNCE_MAX`], intacta** — ela respondia *«o que acontece acima de 1?»* e
/// a resposta não mudou. *O que mudou foi o veredito de PRODUTO, e ele não precisa de desmentir a
/// medição para valer.*
///
/// ⚠️ **O atrito continua a ser o CONTROLO:** os dois tectos são agora o mesmo número, e é o gate
/// que impede que alguém os leia como uma coisa só — eles param em `1` por motivos diferentes
/// (o salto por decisão do dono, o atrito porque acima de `1` Coulomb não compra nada).
#[test]
fn the_bounce_and_the_friction_both_stop_at_one() {
    use ph2d_nodegraph::attr::{BOUNCE_MAX, FRICTION_MAX};
    assert_eq!(BOUNCE_MAX, 1.0, "o tecto do salto, por ordem do dono");
    assert_eq!(FRICTION_MAX, 1.0, "e o atrito fica onde sempre esteve");
    let s = Stream::new(3)
        .with(
            ph2d_nodegraph::attr::FRICTION_COLUMN,
            Column::Scalar(vec![0.5, 2.0, f32::INFINITY]),
        )
        .with(
            ph2d_nodegraph::attr::BOUNCE_COLUMN,
            Column::Scalar(vec![2.0, 5.0, -1.0]),
        );
    let m = materiais(&s).expect("declarou");
    // ⚠️ O `2,0` autorado COAGE agora para `1,0` — era o tecto de ontem e é o dobro do de hoje.
    assert_eq!(
        m[0].salto, BOUNCE_MAX,
        "o que era o tecto de ontem COAGE hoje"
    );
    assert_eq!(m[1].salto, BOUNCE_MAX, "acima do tecto, COAGE");
    assert_eq!(m[1].atrito, FRICTION_MAX, "e o atrito coage no dele");
    assert_eq!(m[2].salto, 0.0, "um negativo nao e' um pedido");
    assert_eq!(m[2].atrito, 0.0, "nem um infinito");
    // E o PAR: o maior dos dois, coagido pelo mesmo tecto.
    assert_eq!(super::atrito::salto(0.5, 0.0), 0.5);
    assert_eq!(super::atrito::salto(9.0, 0.0), BOUNCE_MAX);
}

/// ⭐⭐⭐ **O ATALHO DO PONTO FIXO NÃO MUDA UM BIT** — a cura do report de 2026-09-18.
///
/// O `separate` pára quando uma varredura não mexe um bit, porque a seguinte leria a MESMA entrada
/// (a mesma foto, os mesmos ângulos, a mesma grelha) e devolveria o mesmo. ⚠️ Isto é uma indução,
/// não uma heurística — e este gate é o que a torna observável: o [`separate_all_pairs`] **varre
/// sempre até ao fim**, logo a igualdade ao bit a `1024` varreduras prova que as varreduras que o
/// atalho saltou não faziam nada.
///
/// ⛔ **A METADE QUE FAZ O GATE VALER É O CONTROLO:** a nuvem tem de CHEGAR ao ponto fixo dentro
/// das `1024` — senão o atalho nunca dispara e a igualdade não afirma nada sobre ele.
///
/// ⚠️⚠️ **E foi o controlo que escolheu a fixtura, não eu.** A 1.ª redacção usava a [`nuvem`] com a
/// inércia DERIVADA e reprovou: varrida a escala de `1,0` a `3,0`, ela **nunca** assenta em 1024
/// varreduras — com a rotação solta duas caixas continuam a acertar-se por um ULP para sempre.
/// ⇒ a fixtura é a mesma nuvem com a **rotação travada** (`inv_inercia = 0`, que é o que a coluna
/// `inv_inertia` de um cartão escreve), onde ela assenta.
///
/// ⚠️ *Isto diz também o que o atalho NÃO compra:* numa cena de caixas a rodar ele quase nunca
/// arma, e o que segura o relógio ali é a grelha e o paralelo. O número está na sonda
/// [`custo_probe::a_escada_das_varreduras_contra_a_populacao`].
#[test]
fn o_atalho_do_ponto_fixo_nao_muda_um_bit() {
    let (p0, c, w) = nuvem(200);
    // ⚠️ **A fixtura é MEDIDA, nas duas grandezas** (calibração no cabeçalho): rotação travada, e a
    // nuvem ESPALHADA ao dobro. Apertada ela é uma PILHA e não assenta — `56` varreduras aqui,
    // `nunca` à densidade original —, e ao triplo mexem-se só `21` peças, o que é pouca cena para
    // uma igualdade dizer alguma coisa. A `2,0`: assenta em `56` com `64` peças a mexer-se.
    let p0: Vec<[f32; 2]> = p0.iter().map(|q| [q[0] * 2.0, q[1] * 2.0]).collect();
    let inv = vec![0.0; p0.len()];
    let pecas = Pecas {
        colisores: &c,
        pesos: &w,
        inv_inercia: &inv,
    };
    const VARREDURAS: usize = 1024;
    let (mut atalho, mut sempre) = (p0.clone(), p0.clone());
    let (mut g_atalho, mut g_sempre) = (vec![0.0; p0.len()], vec![0.0; p0.len()]);
    // ⚠️ **Com o repouso visível DESLIGADO (`0.0`)**: este gate afirma a lei do ponto fixo AO BIT,
    // e ela é uma indução. A outra paragem — parar quando nada mais se VÊ — não é bit-idêntica de
    // propósito e tem gate próprio (`parar_no_repouso_visivel_nao_muda_o_que_se_ve`). *Uma régua
    // que só visse a soma das duas não podia afirmar nada sobre nenhuma.*
    separate_com(
        &mut atalho,
        &mut Saida {
            giro: &mut g_atalho,
        },
        &pecas,
        VARREDURAS,
        false,
        0.0,
    );
    separate_all_pairs(
        &mut sempre,
        &mut Saida {
            giro: &mut g_sempre,
        },
        &pecas,
        VARREDURAS,
    );
    // O CONTROLO: quantas varreduras esta nuvem de facto precisou. Se ela nunca assentasse, o
    // `break` do produto não teria corrido e o gate mediria o caminho de sempre.
    let mut passo = p0.clone();
    let mut g_passo = vec![0.0; p0.len()];
    let mut parou = None;
    for v in 1..=VARREDURAS {
        let (antes_p, antes_g) = (passo.clone(), g_passo.clone());
        separate(&mut passo, &mut Saida { giro: &mut g_passo }, &pecas, 1);
        if passo == antes_p && g_passo == antes_g {
            parou = Some(v);
            break;
        }
    }
    let parou = parou.expect("controlo: esta nuvem tem de assentar dentro das 1024 varreduras");
    assert!(
        parou < VARREDURAS,
        "controlo: a nuvem assentou em {parou}, que nao deixa varredura nenhuma para o atalho saltar"
    );
    for i in 0..p0.len() {
        assert_eq!(
            (
                atalho[i][0].to_bits(),
                atalho[i][1].to_bits(),
                g_atalho[i].to_bits()
            ),
            (
                sempre[i][0].to_bits(),
                sempre[i][1].to_bits(),
                g_sempre[i].to_bits()
            ),
            "a peca {i} divergiu: o atalho do ponto fixo (parou em {parou}) saltou trabalho a serio"
        );
    }
}

/// ⭐⭐⭐ **UMA PEÇA LARGADA LONGE NÃO MUDA UM BIT** — a cerca [`grelha::CELULAS_MAX`].
///
/// A grelha densa cobre a caixa das peças activas; uma peça a um milhão de unidades pediria mais
/// células do que a memória do tecto permite, e então o **lado DOBRA** até caber. A malha fica
/// grosseira, cada célula recebe mais candidatos — e quem não toca é descartado pelo `manifesto`.
///
/// ⇒ o resultado tem de ser o mesmo **ao bit**, e é isso que este gate mede: *o preço de uma cena
/// esticada é relógio, nunca resposta errada*.
#[test]
fn uma_peca_largada_longe_nao_muda_um_bit() {
    let (p0, c, w) = nuvem(200);
    let inv = inercias(&c, &w);
    let mut longe = p0.clone();
    // ⚠️ Longe o bastante para a grelha FINA pedir mais de 2^16 células — é essa a cerca a exercitar.
    longe[7] = [4.0e6, -2.5e6];
    let pecas = Pecas {
        colisores: &c,
        pesos: &w,
        inv_inercia: &inv,
    };
    let (mut grelha, mut todos) = (longe.clone(), longe.clone());
    let (mut g_grelha, mut g_todos) = (vec![0.0; p0.len()], vec![0.0; p0.len()]);
    separate(
        &mut grelha,
        &mut Saida {
            giro: &mut g_grelha,
        },
        &pecas,
        8,
    );
    separate_all_pairs(&mut todos, &mut Saida { giro: &mut g_todos }, &pecas, 8);
    // O CONTROLO: a nuvem que ficou continua a resolver-se (senão isto compararia dois nada).
    assert!(
        (0..p0.len()).filter(|&i| longe[i] != todos[i]).count() > 50,
        "controlo: a nuvem tinha de continuar a separar-se com a peca distante la'"
    );
    for i in 0..p0.len() {
        assert_eq!(
            (grelha[i][0].to_bits(), grelha[i][1].to_bits()),
            (todos[i][0].to_bits(), todos[i][1].to_bits()),
            "a peca {i} divergiu com a grelha ENGROSSADA pela cerca de memoria"
        );
    }
}

/// ⭐⭐⭐ **O PARALELO DÁ OS MESMOS BITS QUE O SÉRIE** — a lei que torna a
/// [`PECAS_PARA_PARALELIZAR`] um número de RELÓGIO e nunca de resposta.
///
/// Cada peça é calculada a partir da FOTO da varredura anterior (Jacobi), logo os elementos são
/// independentes e o `collect` indexado repõe a ordem. ⇒ mudar o limiar — ou a máquina ter outro
/// número de núcleos — não pode mover um bit.
///
/// ⚠️ **A fixtura é a do produto**: rotação solta, caixas com o centro fora de `P` e pinos, que é
/// onde a ordem das somas por peça mais poderia divergir.
#[test]
fn o_paralelo_da_os_mesmos_bits_que_o_serie() {
    let (p0, c, w) = nuvem(400);
    let inv = inercias(&c, &w);
    let pecas = Pecas {
        colisores: &c,
        pesos: &w,
        inv_inercia: &inv,
    };
    let (mut serie, mut paralelo) = (p0.clone(), p0.clone());
    let (mut g_serie, mut g_par) = (vec![0.0; p0.len()], vec![0.0; p0.len()]);
    separate_com(
        &mut serie,
        &mut Saida { giro: &mut g_serie },
        &pecas,
        16,
        false,
        REPOUSO_VISIVEL,
    );
    separate_com(
        &mut paralelo,
        &mut Saida { giro: &mut g_par },
        &pecas,
        16,
        true,
        REPOUSO_VISIVEL,
    );
    // O CONTROLO: a nuvem mexeu-se de facto (senão isto compara dois nada).
    assert!(
        (0..p0.len()).filter(|&i| p0[i] != serie[i]).count() > 200,
        "controlo: a nuvem tinha de se separar"
    );
    for i in 0..p0.len() {
        assert_eq!(
            (
                serie[i][0].to_bits(),
                serie[i][1].to_bits(),
                g_serie[i].to_bits()
            ),
            (
                paralelo[i][0].to_bits(),
                paralelo[i][1].to_bits(),
                g_par[i].to_bits()
            ),
            "a peca {i} divergiu entre o serie e o paralelo"
        );
    }
}

/// ⭐⭐⭐ **PARAR NO REPOUSO VISÍVEL NÃO MUDA O QUE SE VÊ** — a segunda paragem do [`separate`], e a
/// que **não** é bit-idêntica de propósito (report do dono de 2026-09-18: *«centenas a milhares de
/// objectos em runtime»*).
///
/// ⛔⛔ **A régua NÃO é a posição, é o que o artista vê:** quantos pares continuam **visivelmente**
/// sobrepostos. Uma barra de posição sozinha ou seria tão apertada que proibiria a paragem, ou tão
/// frouxa que deixaria passar uma cena por separar.
///
/// ⚠️ **E o CONTROLO é o que separa isto de «aceita e mente»:** a paragem tem de ter ARMADO (menos
/// varreduras do que o tecto) — senão o gate estaria a comparar duas corridas completas e a
/// aprovar-se a si mesmo.
#[test]
fn parar_no_repouso_visivel_nao_muda_o_que_se_ve() {
    const TECTO: usize = 1024;
    const VISIVEL: f32 = 0.02;
    // ⚠️ **A fixtura é uma CENA e não uma PILHA, e foi o CONTROLO que a escolheu:** à densidade
    // original a `nuvem` é um monte compacto que **nunca** chega ao repouso visível (gasta as 1024
    // e o controlo reprova). Espalhada ao dobro ela assenta — que é a cena de que o report fala.
    let (p0, c, w) = nuvem(300);
    let p0: Vec<[f32; 2]> = p0.iter().map(|q| [q[0] * 2.0, q[1] * 2.0]).collect();
    let inv = inercias(&c, &w);
    let pecas = Pecas {
        colisores: &c,
        pesos: &w,
        inv_inercia: &inv,
    };
    let conta = |p: &[[f32; 2]]| {
        let mut k = 0;
        for i in 0..p.len() {
            for j in (i + 1)..p.len() {
                if let (Some(a), Some(b)) = (c[i].as_ref(), c[j].as_ref())
                    && contato(a, p[i], b, p[j], false).is_some_and(|t| t.penetracao > VISIVEL)
                {
                    k += 1;
                }
            }
        }
        k
    };
    let (mut cedo, mut g_cedo) = (p0.clone(), vec![0.0; p0.len()]);
    let usadas = separate(&mut cedo, &mut Saida { giro: &mut g_cedo }, &pecas, TECTO);
    let (mut cheio, mut g_cheio) = (p0.clone(), vec![0.0; p0.len()]);
    separate_com(
        &mut cheio,
        &mut Saida { giro: &mut g_cheio },
        &pecas,
        TECTO,
        false,
        0.0,
    );
    assert!(
        usadas < TECTO,
        "CONTROLO: a paragem tem de ter armado, senao este gate compara duas corridas iguais \
         (usou {usadas} de {TECTO})"
    );
    assert_eq!(
        conta(&cedo),
        conta(&cheio),
        "parar cedo mudou o que se VE': {} pares sobrepostos contra {} (usou {usadas} varreduras)",
        conta(&cedo),
        conta(&cheio)
    );
    let alcance = c
        .iter()
        .flatten()
        .map(|x| x.alcance())
        .fold(0.0f32, f32::max);
    let desvio = (0..p0.len())
        .map(|i| (cedo[i][0] - cheio[i][0]).hypot(cedo[i][1] - cheio[i][1]))
        .fold(0.0f32, f32::max)
        / alcance;
    let desvio_giro = (0..p0.len())
        .map(|i| (g_cedo[i] - g_cheio[i]).abs())
        .fold(0.0f32, f32::max);
    // ⚠️⚠️ **AS BARRAS SÃO ABSOLUTAS E MEDIDAS — e a 1.ª redacção derivava-as do
    // `REPOUSO_VISIVEL`.** Isso é uma régua que não pode testar o que mede: uma mutação que sobe o
    // limiar `1000×` sobe o desvio **e a barra** na mesma proporção, e o gate fica verde sobre uma
    // paragem que já muda o desenho (medido: ela SOBREVIVEU). ⇒ os números vêm da corrida, com
    // margem: nesta fixtura o produto lê `5,6e-5` da peça e `3,4e-3` graus, em `63` de `1024`
    // varreduras.
    assert!(
        desvio < 1e-3,
        "a cauda que a paragem deixou ({desvio:.3e} da peca) passou a ser visivel · {usadas} varreduras"
    );
    assert!(
        desvio_giro < 0.05,
        "a paragem deixou a peca a {desvio_giro:.3e} graus de onde ela assentaria"
    );
}

/// ⭐⭐⭐ **UMA PEÇA QUE SÓ RODA NÃO É LIDA COMO PARADA** — a metade da paragem que nenhuma outra
/// fixtura alcança.
///
/// ⚠️⚠️ **Ela existe por uma MUTAÇÃO SOBREVIVENTE:** tirar o termo da rotação de `aplica` passava
/// todos os gates, porque nas cenas normais quem roda também **transladas** — e é a translação que
/// mantém o laço vivo. *A régua só vê a rotação onde a translação é impossível.*
///
/// ⇒ a fixtura são duas caixas **travadas em translação** (`inv_mass = 0`) e **livres para rodar**
/// (`inv_inertia > 0`), que é o que um cartão exprime. Sem o termo, `andou` lê `0` na primeira
/// varredura, o atalho do ponto fixo arma, e as peças ficam **por rodar**.
#[test]
fn uma_peca_que_so_roda_nao_e_lida_como_parada() {
    let c = vec![
        Some(Colisor::caixa([0.8, 0.2], SEM_GIRO)),
        Some(Colisor::caixa([0.8, 0.2], SEM_GIRO)),
    ];
    // Sobrepostas e DESCENTRADAS: o contacto tem braço, logo pede binário.
    let p0 = vec![[0.0, 0.0], [0.9, 0.25]];
    let pesos = vec![0.0, 0.0]; // travadas em translação
    let inv_i = vec![1.0, 1.0]; // livres para rodar
    let pecas = Pecas::novas(&c, &pesos, &inv_i);
    let (mut p, mut g) = (p0.clone(), vec![0.0; 2]);
    let usadas = separate(&mut p, &mut Saida { giro: &mut g }, &pecas, 512);
    assert_eq!(
        p, p0,
        "a fixtura tem de ser SO' rotação: nada pode transladar"
    );
    // O CONTROLO: elas de facto rodaram — senão o gate não contém o fenómeno.
    let rodou = g[0].abs().max(g[1].abs());
    assert!(
        rodou > 1.0,
        "controlo: as caixas tinham de rodar de forma visivel, e rodaram {rodou:.3e} graus"
    );
    // E a lei: a paragem não pode ter armado na primeira varredura, que é o que acontece quando
    // `aplica` não conta a rotação.
    assert!(
        usadas > 4,
        "a paragem leu uma peca que SO' roda como parada: {usadas} varredura(s), {g:?} graus"
    );
}

/// ⭐⭐⭐ **O PISO DA TAREFA TEM DE DEIXAR TRABALHO PARA TODOS OS NÚCLEOS** — a lei que as minhas
/// DUAS primeiras escolhas violaram.
///
/// ⛔⛔ A 1.ª foi um número redondo (`64` peças por tarefa): com `1000` peças isso são **16 tarefas**
/// numa máquina de 32 núcleos. A 2.ª derivava o número de PEDAÇOS do `n` e dos núcleos — e isso
/// piorou o app do dono de `18,6` para `30,4 ms`, porque *fixar os pedaços tira ao rayon a decisão
/// de partir só quando há quem trabalhe*.
///
/// ⇒ hoje é um **PISO** pequeno e medido, e o que este gate afirma é a propriedade que sobra: com
/// uma cena real, o piso deixa **pelo menos uma tarefa por núcleo**.
#[test]
fn o_piso_da_tarefa_deixa_trabalho_para_todos_os_nucleos() {
    let nucleos = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    // ⚠️ `const _` e não `assert!`: os dois lados são constantes, logo o compilador dobra-o — e
    // assim ele passa a ser **erro de compilação**, que é mais forte do que um teste.
    const _: () = assert!(
        PISO_DA_TAREFA >= 1,
        "um piso de zero deixaria o rayon partir ate' um elemento por tarefa"
    );
    // ⚠️⚠️ **A GUARDA da 1.ª redacção deste gate DESLIGAVA a lei exactamente quando ela era
    // violada:** ela dizia `if n >= nucleos * PISO`, e com um piso enorme essa condição é FALSA
    // para toda a cena — o gate passava por vácuo. Uma mutação que punha o piso em `4096`
    // **sobreviveu**. ⇒ a cerca é sobre a CENA (a população que o dono nomeou), nunca sobre o
    // número que está a ser testado.
    for n in [1000usize, 4000, 100_000] {
        let tarefas = n / PISO_DA_TAREFA;
        assert!(
            tarefas >= nucleos.min(n / 8),
            "com {n} pecas o piso {PISO_DA_TAREFA} da' {tarefas} tarefas para {nucleos} nucleos"
        );
    }
}

/// Os candidatos de **UMA camada** — o CONTROLO de todo gate do corte. Ele não é uma segunda
/// resposta: é o plano de antes desta wave, alcançável por uma porta própria
/// ([`crate::grelha::Grelha::planeia_numa_camada`]).
fn candidatos_numa_camada(c: &[Option<Colisor>], p: &[[f32; 2]]) -> usize {
    let vivo: Vec<bool> = (0..p.len()).map(|i| ativo(p[i], c[i].as_ref())).collect();
    let alcance_max = crate::grelha::alcances_de(c, &vivo)
        .iter()
        .fold(0.0_f32, |a, b| a.max(*b));
    let mut g = crate::grelha::Grelha::default();
    g.planeia_numa_camada(&vivo, 2.0 * alcance_max);
    g.constroi(p, &vivo);
    let (mut viz, mut soma) = (Vec::new(), 0usize);
    for k in 0..p.len() {
        g.vizinhos_de(k, &mut viz);
        soma += viz.len();
    }
    soma
}

/// A [`nuvem`] com UMA peça `8 ×` maior que a maior das outras — a forma da cena do dono, onde o
/// perfilador dele leu `132`–`156` vizinhos por peça contra os `12` da fixtura uniforme.
fn nuvem_com_uma_grande(n: u32) -> (Vec<[f32; 2]>, Vec<Option<Colisor>>, Vec<f32>) {
    let (p, mut c, w) = nuvem(n);
    let maior = c
        .iter()
        .flatten()
        .map(Colisor::alcance)
        .fold(0.0_f32, f32::max);
    c[1] = Some(Colisor::disco(8.0 * maior));
    (p, c, w)
}

/// ⭐⭐⭐ **AS DUAS CAMADAS DÃO OS MESMOS BITS QUE TODOS-OS-PARES.**
///
/// ⚠️⚠️ **O CONTROLO POSITIVO é metade do gate:** sem ele, uma fixtura sem dispersão de tamanhos
/// nunca arma o corte e este teste ficaria verde a medir o caminho de UMA camada — que é
/// exactamente o que a [`nuvem`] uniforme do gate irmão faz. *Um gate cuja fixtura não contém o
/// fenómeno não afirma nada sobre ele.*
#[test]
fn a_grelha_em_duas_camadas_da_os_mesmos_bits() {
    let (p0, c, w) = nuvem_com_uma_grande(400);
    let grandes = grandes_do_plano_medido(&c, &p0);
    let _ = &w;
    assert!(
        grandes > 0,
        "o plano NAO partiu em duas camadas: este gate nao esta' a medir o caminho novo"
    );
    let inv = inercias(&c, &w);
    let pecas = Pecas::novas(&c, &w, &inv);
    let (mut grelha, mut todos) = (p0.clone(), p0.clone());
    let (mut g_grelha, mut g_todos) = (vec![0.0; p0.len()], vec![0.0; p0.len()]);
    // ⚠️⚠️ **Pela porta das CERCAS e não pela do produto:** o corte shipa DESLIGADO (doc 115 §31),
    // e um `separate` aqui mediria o caminho de UMA camada — *verde a afirmar nada sobre a lei que
    // este gate nomeia*. Duas mutações sobreviveram exactamente assim no dia da inversão.
    crate::separate_com_cercas(
        &mut grelha,
        &mut Saida {
            giro: &mut g_grelha,
        },
        &pecas,
        8,
        &crate::Cercas {
            paralelo: p0.len() >= PECAS_PARA_PARALELIZAR,
            repouso: REPOUSO_VISIVEL,
            grao: PISO_DA_TAREFA,
            duas_camadas: true,
        },
    );
    separate_all_pairs(&mut todos, &mut Saida { giro: &mut g_todos }, &pecas, 8);
    let mexeu = (0..p0.len()).filter(|&i| p0[i] != todos[i]).count();
    assert!(mexeu > 200, "a nuvem tinha de mexer-se: so' {mexeu} pecas");
    for i in 0..p0.len() {
        assert_eq!(
            (
                grelha[i][0].to_bits(),
                grelha[i][1].to_bits(),
                g_grelha[i].to_bits()
            ),
            (
                todos[i][0].to_bits(),
                todos[i][1].to_bits(),
                g_todos[i].to_bits()
            ),
            "peca {i}: duas camadas {:?}/{} contra todos-os-pares {:?}/{}",
            grelha[i],
            g_grelha[i],
            todos[i],
            g_todos[i]
        );
    }
}

/// ⭐⭐⭐ **UMA PEÇA GRANDE DEIXOU DE INFLAR A GRELHA DE TODAS** — e a régua é a CONTAGEM, que a
/// carga da máquina não estraga.
///
/// ⚠️ A grandeza é o CANDIDATO, que é o multiplicador do custo de uma varredura e o número que a
/// [`crate::Relatorio`] publica. A barra sai da medição
/// ([`crate::custo_probe::contagens::onde_o_corte_dos_grandes_paga`]: `11,3 ×` a `1000` discos com
/// dispersão `4 ×`) e é deliberadamente **metade** dela — o que se afirma é a CLASSE, não o número.
#[test]
fn uma_peca_grande_deixou_de_inflar_a_grelha_de_todas() {
    let (p, c, _w) = nuvem_com_uma_grande(400);
    let uma = candidatos_numa_camada(&c, &p);
    let (duas, grandes) = candidatos_do_plano_medido(&c, &p);
    assert_eq!(grandes, 1, "só a peça 1 é grande");
    assert!(
        uma >= 4 * duas,
        "o corte comprou pouco: uma camada {uma}, duas camadas {duas}"
    );
}

/// ⭐⭐⭐ **SEM DISPERSÃO O PLANO NÃO PARTE, E O CAMINHO É O DE SEMPRE — ao candidato.**
///
/// ⚠️⚠️ **Esta é a metade NEGATIVA, e ela vale tanto como a positiva:** promover uma peça não é de
/// graça (ela passa a ver a nuvem inteira), logo um plano que partisse sempre PIORARIA toda cena de
/// tamanho uniforme. A tabela da [`crate::MARGEM_DO_CORTE`] mede a piora: `0,91 ×` a uma dispersão
/// de `1,25 ×`.
#[test]
fn sem_dispersao_o_plano_nao_parte() {
    // Discos todos iguais: não há nada a promover.
    let (p, _, w) = nuvem(400);
    let c: Vec<Option<Colisor>> = (0..p.len()).map(|_| Some(Colisor::disco(0.2))).collect();
    let _ = &w;
    let (duas, grandes) = candidatos_do_plano_medido(&c, &p);
    assert_eq!(grandes, 0, "uma nuvem uniforme nao tem peca grande");
    assert_eq!(
        duas,
        candidatos_numa_camada(&c, &p),
        "sem corte o plano tem de dar a grelha de sempre, candidato a candidato"
    );
    // E a nuvem MISTA de sempre (dispersão ~3 ×, mas espalhada) também não parte: o termo `g · m`
    // come o ganho muito antes de a escada descer o suficiente.
    let (p2, c2, _w2) = nuvem(400);
    let g2 = grandes_do_plano_medido(&c2, &p2);
    assert_eq!(g2, 0, "a nuvem mista nao devia partir");
}

/// ⭐⭐⭐ **UMA DISPERSÃO PEQUENA NÃO PAGA O CORTE** — o gate da [`crate::MARGEM_DO_CORTE`].
///
/// ⚠️⚠️ **Ele nasceu de uma MUTAÇÃO SOBREVIVENTE, e o que ela expôs foi um gate meu a prometer
/// mais do que media:** o `sem_dispersao_o_plano_nao_parte` fica verde com a margem a `0`, porque
/// numa nuvem UNIFORME o minimizador acha o mínimo em `g = 0` e **a margem nunca é consultada**.
/// *A margem só decide onde o modelo vê um ganho pequeno — e é ali que ela tem de ser medida.*
///
/// A fixtura é esse regime: quatro peças `1,25 ×` maiores que as outras, onde a tabela medida lê
/// **`0,91 ×`** — uma PIORA de `9 %` que o modelo, sozinho, adoptaria.
#[test]
fn uma_dispersao_pequena_nao_paga_o_corte() {
    let (p, _, _w) = nuvem(400);
    let c: Vec<Option<Colisor>> = (0..p.len())
        .map(|i| Some(Colisor::disco(if i < 4 { 0.25 } else { 0.20 })))
        .collect();
    // (a) O plano MEDIDO recusa o corte.
    let grandes = grandes_do_plano_medido(&c, &p);
    assert_eq!(grandes, 0, "uma dispersao de 1,25x nao paga o corte");
    // (b) O CONTROLO, dentro do gate: com a margem desarmada o modelo PARTIRIA — logo existe um
    //     corte a recusar, e esta fixtura está no regime que a margem existe para julgar.
    let vivo: Vec<bool> = (0..p.len()).map(|i| ativo(p[i], c[i].as_ref())).collect();
    let alcances = crate::grelha::alcances_de(&c, &vivo);
    let mut g = crate::grelha::Grelha::default();
    g.planeia_com_margem(&p, &vivo, &alcances, 1.0);
    g.constroi(&p, &vivo);
    assert!(
        g.grandes() > 0,
        "sem margem o modelo tinha de partir: esta fixtura ja' nao mede a margem"
    );
    // (c) E a RAZÃO da recusa, em CANDIDATOS medidos: esse corte é uma PIORA. ⭐ O modelo sobrestima
    //     a coluna de uma camada (o bloco 3x3 está cortado nas bordas da nuvem), e é exactamente
    //     essa diferença que a margem cobre.
    let (mut viz, mut duas) = (Vec::new(), 0usize);
    for k in 0..p.len() {
        g.vizinhos_de(k, &mut viz);
        duas += viz.len();
    }
    let uma = candidatos_numa_camada(&c, &p);
    assert!(
        duas >= uma,
        "o corte teria comprado alguma coisa ({uma} contra {duas}) — a recusa seria errada"
    );
}

/// ⭐⭐⭐ **A RÉGUA DO PLANO CONTA EXACTAMENTE O QUE A GRELHA ENTREGA.**
///
/// ⚠️⚠️ **Ela é a metade que torna a decisão do plano uma MEDIÇÃO e não uma previsão** — o
/// [`crate::grelha::Grelha::planeia`] constrói as duas hipóteses, conta-as por
/// `candidatos_previstos` e fica com a que ganhou. *Se essa contagem discordar do que o
/// `vizinhos_de` devolve, a decisão passa a ser sobre um número que não existe.*
///
/// ⛔ Por isso ela é dobrada à mão, nas TRÊS configurações: uma camada, duas camadas, e a nuvem
/// mista de sempre.
#[test]
fn a_regua_do_plano_conta_o_que_a_grelha_entrega() {
    for (nome, (p, c, _w)) in [
        ("uniforme", nuvem(400)),
        ("uma grande", nuvem_com_uma_grande(400)),
    ] {
        let vivo: Vec<bool> = (0..p.len()).map(|i| ativo(p[i], c[i].as_ref())).collect();
        let alcances = crate::grelha::alcances_de(&c, &vivo);
        let alcance_max = alcances.iter().fold(0.0_f32, |a, b| a.max(*b));
        for camadas in [1usize, 2] {
            let mut g = crate::grelha::Grelha::default();
            if camadas == 1 {
                g.planeia_numa_camada(&vivo, 2.0 * alcance_max);
                g.constroi(&p, &vivo);
            } else {
                g.planeia_medindo(&p, &vivo, &alcances);
            }
            let (mut viz, mut mao) = (Vec::new(), 0usize);
            for k in 0..p.len() {
                g.vizinhos_de(k, &mut viz);
                mao += viz.len();
            }
            assert_eq!(
                g.candidatos_previstos(),
                mao,
                "{nome}, {camadas} camada(s): a régua do plano discorda da grelha"
            );
        }
    }
}

/// ⭐ **A PORTA DE BISSECÇÃO lê-se como o dono a escreve** — e a armadilha que esta casa já pagou é
/// a do meio: `env PH2D_CONTACT_DUAS_CAMADAS=` **define** a variável, vazia.
#[test]
fn a_porta_de_bisseccao_le_o_que_o_dono_escreve() {
    use crate::grelha::ordem_de;
    assert!(ordem_de(Some("1")), "=1 desliga o corte");
    assert!(ordem_de(Some("sim")), "qualquer valor nao-vazio desliga");
    assert!(!ordem_de(None), "ausente é o caminho de omissão");
    assert!(!ordem_de(Some("0")), "=0 é o caminho de omissão, explícito");
    assert!(!ordem_de(Some("")), "=<vazio> NÃO é uma ordem");
}

/// ⭐⭐⭐ **A DECISÃO DO PLANO VÊ AS CÉLULAS, E UMA RÉGUA DE CANDIDATOS É CEGA ALI.**
///
/// ⚠️⚠️ **Este gate nasceu de um report do dono** (*«motor anterior mais rápido»*, 19/09): a
/// decisão contava CANDIDATOS, e a malha fina paga também `O(células)` **por varredura**. A sonda
/// [`crate::custo_probe::contagens::quanto_custa_uma_celula_contra_um_candidato`] mede o regime
/// onde isso decide: a `1 000` candidatos PARADOS, quadruplicar as células leva uma varredura de
/// `25,7` para `56,8 µs`.
///
/// ⭐ O CONTROLO está dentro do gate: a régua de candidatos **empata** onde a medida separa.
#[test]
fn a_decisao_do_plano_ve_as_celulas() {
    let (p, _, _) = nuvem(400);
    let c: Vec<Option<Colisor>> = (0..p.len()).map(|_| Some(Colisor::disco(0.2))).collect();
    let vivo: Vec<bool> = (0..p.len()).map(|i| ativo(p[i], c[i].as_ref())).collect();
    let mede = |lado: f32| {
        let mut g = crate::grelha::Grelha::default();
        g.planeia_numa_camada(&vivo, lado);
        g.constroi(&p, &vivo);
        (g.candidatos_previstos(), g.celulas(), g.custo_medido())
    };
    // Duas malhas já tão finas que refiná-las quase não tira candidatos — e QUADRUPLICA as células.
    let (cand_a, cel_a, custo_a) = mede(0.05);
    let (cand_b, cel_b, custo_b) = mede(0.025);
    assert!(
        cand_b < cand_a,
        "o CONTROLO: uma régua de candidatos preferiria a malha FINA ({cand_a} contra {cand_b})"
    );
    assert!(
        cel_b > cel_a * 3,
        "a malha fina tinha de ter muito mais células: {cel_a} contra {cel_b}"
    );
    assert!(
        custo_b > custo_a,
        "a régua do plano é cega às células: {custo_a} contra {custo_b}"
    );
    // E a composição é a que a cerca declara — nem mais um termo, nem menos.
    assert_eq!(custo_a, cand_a + cel_a / crate::CELULAS_POR_CANDIDATO);
}

/// Os candidatos e as GRANDES do plano de duas camadas **medido**, pela porta que não lê o ambiente.
///
/// ⚠️⚠️ **Ela existe porque o corte shipa DESLIGADO** (doc 115 §31): um gate que entrasse pela porta
/// do produto mediria o caminho de uma camada e ficaria **verde a afirmar nada**.
fn candidatos_do_plano_medido(c: &[Option<Colisor>], p: &[[f32; 2]]) -> (usize, usize) {
    let vivo: Vec<bool> = (0..p.len()).map(|i| ativo(p[i], c[i].as_ref())).collect();
    let alcances = crate::grelha::alcances_de(c, &vivo);
    let mut g = crate::grelha::Grelha::default();
    g.planeia_medindo(p, &vivo, &alcances);
    g.constroi(p, &vivo);
    let (mut viz, mut soma) = (Vec::new(), 0usize);
    for k in 0..p.len() {
        g.vizinhos_de(k, &mut viz);
        soma += viz.len();
    }
    (soma, g.grandes())
}

/// Quantas peças o plano medido promoveria — a metade da porta acima que os gates da recusa usam.
fn grandes_do_plano_medido(c: &[Option<Colisor>], p: &[[f32; 2]]) -> usize {
    candidatos_do_plano_medido(c, p).1
}

/// ⛔⛔⛔ **O CORTE EM DUAS CAMADAS SHIPA DESLIGADO** — e isto é um gate, não um comentário.
///
/// ⚠️⚠️ **Ele nasceu de uma AUDITORIA pedida pelo dono** (doc 115 §31), depois de três reports
/// seguidos de quadros perdidos. Eu liguei o corte por omissão com **todas** as medições tiradas
/// numa forma de cena que a dele não tem (`12` vizinhos por peça contra `124`), e nunca consegui
/// reproduzir o que ele mediu.
///
/// ⛔ *Quando a minha medição e o report do dono discordam e eu não fecho a distância, o caminho de
/// omissão é o que ele APROVOU.* O ónus da prova é de quem mudou, e o default é onde isso se
/// escreve.
#[test]
fn o_corte_em_duas_camadas_shipa_desligado() {
    assert!(
        !crate::grelha::duas_camadas_activas(),
        "o corte voltou a ligar-se por omissão sem o dono o ter aprovado"
    );
}
