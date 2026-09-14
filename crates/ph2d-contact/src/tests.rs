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
        salto: &mut vec![0.0; p.len()],
    };
    separate(p, &mut saida, &Pecas::novas(c, w, &zeros), 8);
}

/// Uma corrida com a rotação DESTRAVADA; devolve o giro de cada peça, em graus.
fn corre_girando(p: &mut [[f32; 2]], c: &[Option<Colisor>], w: &[f32]) -> Vec<f32> {
    let inv = inercias(c, w);
    let mut giro = vec![0.0; p.len()];
    let mut salto = vec![0.0; p.len()];
    separate(
        p,
        &mut Saida {
            giro: &mut giro,
            salto: &mut salto,
        },
        &Pecas::novas(c, w, &inv),
        8,
    );
    giro
}

/// Uma corrida com MATERIAL: o deslize medido desde `antes`, com o mesmo material para todos.
/// Devolve `(giro em graus, salto)`.
fn corre_com_atrito(
    p: &mut [[f32; 2]],
    antes: &[[f32; 2]],
    c: &[Option<Colisor>],
    w: &[f32],
    m: Material,
) -> (Vec<f32>, Vec<f32>) {
    corre_com_atrito_girando(p, antes, &vec![0.0; p.len()], c, w, m)
}

/// Idem, mas com uma rotação PRÓPRIA já feita neste passo (o `spin` integrado).
fn corre_com_atrito_girando(
    p: &mut [[f32; 2]],
    antes: &[[f32; 2]],
    girou_antes: &[f32],
    c: &[Option<Colisor>],
    w: &[f32],
    m: Material,
) -> (Vec<f32>, Vec<f32>) {
    let n = p.len();
    let inv = inercias(c, w);
    let material = vec![m; n];
    let (mut giro, mut salto) = (vec![0.0; n], vec![0.0; n]);
    separate(
        p,
        &mut Saida {
            giro: &mut giro,
            salto: &mut salto,
        },
        &Pecas {
            colisores: c,
            pesos: w,
            inv_inercia: &inv,
            deslize: Some(Deslize {
                antes,
                girou_antes,
                material: &material,
            }),
        },
        8,
    );
    (giro, salto)
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
    // ⚠️ Com MATERIAL e deslize: a igualdade ao bit tem de valer também para a metade tangencial,
    // que soma por par exactamente como a normal — e é ela a que a ordem dos vizinhos pode trocar.
    let (material, girou, antes) = (
        vec![
            Material {
                atrito: 0.7,
                salto: 0.2
            };
            p0.len()
        ],
        vec![0.0; p0.len()],
        p0.iter()
            .map(|q| [q[0] - 0.01, q[1] - 0.02])
            .collect::<Vec<_>>(),
    );
    let pecas = Pecas {
        colisores: &c,
        pesos: &w,
        inv_inercia: &inv,
        deslize: Some(Deslize {
            antes: &antes,
            girou_antes: &girou,
            material: &material,
        }),
    };
    let (mut s_grelha, mut s_todos) = (vec![0.0; p0.len()], vec![0.0; p0.len()]);
    separate(
        &mut grelha,
        &mut Saida {
            giro: &mut g_grelha,
            salto: &mut s_grelha,
        },
        &pecas,
        8,
    );
    separate_all_pairs(
        &mut todos,
        &mut Saida {
            giro: &mut g_todos,
            salto: &mut s_todos,
        },
        &pecas,
        8,
    );
    assert_eq!(s_grelha, s_todos, "o salto recolhido tem de ser o mesmo");
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

/// ⭐⭐⭐ **Uma caixa apanhada FORA DO CENTRO roda** — e o `Lock Rotation` (a inércia a zero) trava-a
/// sem lhe mudar mais nada de essencial.
///
/// A prancha é um pino largo; a caixa livre pousa na ponta dela, com metade de fora: o contacto
/// acontece a um braço do centro, e é isso que a faz tombar.
#[test]
fn a_box_caught_off_centre_turns_and_the_lock_stops_it() {
    let c = vec![
        Some(Colisor::caixa([1.0, 0.1], SEM_GIRO)),
        Some(Colisor::caixa([0.25, 0.25], SEM_GIRO)),
    ];
    let (w, p0) = ([0.0, 1.0], vec![[0.0, 0.0], [0.9, 0.25]]);

    let mut solta = p0.clone();
    let giro = corre_girando(&mut solta, &c, &w);
    assert!(
        giro[0] == 0.0,
        "o pino nao roda (inercia inversa zero): {giro:?}"
    );
    assert!(
        giro[1].abs() > 1.0,
        "a caixa apanhada na ponta tem de TOMBAR: {giro:?}"
    );

    let mut travada = p0.clone();
    corre(&mut travada, &c, &w);
    assert!(
        travada[1][1] > p0[1][1],
        "travada ela continua a ser empurrada para cima: {travada:?}"
    );
}

/// ⭐⭐⭐ **Uma caixa pousada DE CHAPA não roda** — e é isto que o recorte das faces compra: com o
/// vértice mais fundo no lugar do meio do trecho, uma pilha parada tombava sozinha.
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
    assert!((p[1][1] - 0.9).abs() < 1e-4, "e ela assenta na face: {p:?}");
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

/// Um chão imóvel (peça 0) e um disco pousado nele (peça 1), com a penetração pedida.
///
/// ⚠️ A penetração é MINÚSCULA de propósito: é assim que uma pilha assente vive (a gravidade
/// afunda `~g·dt²` por tique), e é onde o braço da tangente vale o raio inteiro.
fn chao_e_disco(raio: f32, pen: f32) -> (Vec<[f32; 2]>, Vec<Option<Colisor>>, Vec<f32>) {
    (
        vec![[0.0, -1.0], [0.0, raio - pen]],
        vec![
            Some(Colisor::caixa([10.0, 1.0], SEM_GIRO)),
            Some(Colisor::disco(raio)),
        ],
        vec![0.0, 1.0],
    )
}

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

/// ⭐⭐⭐ **UM DISCO QUE DERRAPA PASSA A RODAR** — a resposta ao report, e o SENTIDO certo: a
/// deslizar para a direita sobre um chão ele roda no sentido dos ponteiros (graus negativos).
///
/// ⚠️ **E o controlo é o material LISO**: com `atrito = 0` o mesmo deslize não roda **nada**.
#[test]
fn a_disc_that_slides_starts_to_roll_and_ice_does_not() {
    let (p0, c, w) = chao_e_disco(0.5, 1e-4);
    let antes = vec![p0[0], [p0[1][0] - 0.1, p0[1][1]]]; // o disco deslizou +0,1 em x
    let (mut p, mut gelo) = (p0.clone(), p0.clone());
    let (giro, _) = corre_com_atrito(
        &mut p,
        &antes,
        &c,
        &w,
        Material {
            atrito: 1.0,
            salto: 0.0,
        },
    );
    let (sem, _) = corre_com_atrito(&mut gelo, &antes, &c, &w, Material::LISO);
    assert_eq!(
        sem[1].to_bits(),
        0.0_f32.to_bits(),
        "gelo nao roda: {}",
        sem[1]
    );
    assert!(
        giro[1] < -0.01,
        "a deslizar para a direita o disco tem de rodar no sentido dos ponteiros, e rodou {}",
        giro[1]
    );
    assert_eq!(giro[0], 0.0, "o chao e' um obstaculo: nao roda");
}

/// ⭐⭐ **A REPARTIÇÃO É A DE MANUAL: ⅓ para mover, ⅔ para rolar.** Num disco `invI·R² = 2w`, logo
/// a massa efectiva tangencial é `3w` e o ponto de contacto recebe DUAS vezes mais da rotação do
/// que da translação — é isso que faz *«deixar de derrapar»* significar *«começar a rolar»* e não
/// *«travar»*.
#[test]
fn the_rolling_split_gives_the_spin_twice_the_slide() {
    let raio = 0.5;
    let (p0, c, w) = chao_e_disco(raio, 1e-4);
    let antes = vec![p0[0], [p0[1][0] - 0.1, p0[1][1]]];
    let mut p = p0.clone();
    let (giro, _) = corre_com_atrito(
        &mut p,
        &antes,
        &c,
        &w,
        Material {
            atrito: 1.0,
            salto: 0.0,
        },
    );
    // Quanto o ponto de contacto andou por cada uma das duas metades, ao longo da tangente.
    let da_translacao = (p[1][0] - p0[1][0]).abs();
    let da_rotacao = (giro[1] / GRAUS * raio).abs();
    assert!(
        da_translacao > 1e-9,
        "tem de haver correccao: {da_translacao}"
    );
    let razao = da_rotacao / da_translacao;
    assert!(
        (razao - 2.0).abs() < 0.02,
        "a rotacao tem de valer o DOBRO da translacao no ponto de contacto, e a razao foi {razao}"
    );
}

/// ⛔ **UM MATERIAL LISO NÃO MUDA UM BIT** — a lei de antes do §7 sobrevive termo a termo, e é
/// isso que deixa esta metade nascer sem migração nenhuma.
#[test]
fn an_icy_material_changes_nothing_to_the_bit() {
    let (p0, c, w) = nuvem(200);
    let antes: Vec<[f32; 2]> = p0.iter().map(|q| [q[0] - 0.03, q[1] + 0.02]).collect();
    let (mut com, mut sem) = (p0.clone(), p0.clone());
    let (g_com, salto) = corre_com_atrito(&mut com, &antes, &c, &w, Material::LISO);
    let g_sem = corre_girando(&mut sem, &c, &w);
    assert!(salto.iter().all(|s| *s == 0.0), "gelo morto nao salta");
    for i in 0..p0.len() {
        assert_eq!(
            (com[i][0].to_bits(), com[i][1].to_bits(), g_com[i].to_bits()),
            (sem[i][0].to_bits(), sem[i][1].to_bits(), g_sem[i].to_bits()),
            "peca {i}"
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

/// ⭐ **E o salto CHEGA a quem responde**: a peça que tocou leva o salto do par mais vivo.
#[test]
fn the_bounce_of_the_liveliest_pair_reaches_the_piece_that_touched() {
    let (p0, c, w) = chao_e_disco(0.5, 0.01);
    let antes = p0.clone();
    let mut p = p0.clone();
    let (_, salto) = corre_com_atrito(
        &mut p,
        &antes,
        &c,
        &w,
        Material {
            atrito: 0.0,
            salto: 0.8,
        },
    );
    assert!((salto[1] - 0.8).abs() < 1e-6, "o disco tocou: {salto:?}");
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

/// ⭐⭐ **UMA BOLA QUE GIRA DERRAPA CONTRA O CHÃO MESMO PARADA** — o deslize inclui a rotação
/// PRÓPRIA da peça (`ω × r`), e não só a translação dela.
///
/// ⛔ **Sem este termo o atrito lia zero** numa bola que gira no sítio, e ela aceleraria para
/// sempre: é o `spin` que o `sim.step` integra, e o `angular_damping` nasce em `1` (sem arrasto).
#[test]
fn a_spinning_disc_rubs_against_the_floor_even_standing_still() {
    let (p0, c, w) = chao_e_disco(0.5, 1e-3);
    let antes = p0.clone(); // não se moveu um bit
    let girou = vec![0.0, 30.0]; // mas rodou 30° no sentido anti-horário
    let mut p = p0.clone();
    let (giro, _) = corre_com_atrito_girando(
        &mut p,
        &antes,
        &girou,
        &c,
        &w,
        Material {
            atrito: 1.0,
            salto: 0.0,
        },
    );
    assert!(
        giro[1] < -0.01,
        "o atrito tem de OPOR o giro proprio, e devolveu {}",
        giro[1]
    );
    // ⭐ **E o SENTIDO é o da roda a patinar**: a girar no anti-horário o ponto de baixo varre
    // para `+x`, logo o chão empurra a bola para `−x` — é o que um carro faz quando a roda patina.
    // ⚠️ Bate com o gate irmão (`a_disc_that_slides_starts_to_roll…`): a deslizar para `+x` ela
    // roda no horário, logo rolar para `−x` É girar no anti-horário. *As duas metades da mesma lei.*
    assert!(
        p[1][0] < p0[1][0] - 1e-6,
        "e empurra a bola para o lado contrario ao varrimento, e ela ficou em {:?}",
        p[1]
    );
    // O CONTROLO: parada e sem girar, o atrito não tem nada a opor.
    let mut quieta = p0.clone();
    let (parada, _) = corre_com_atrito(
        &mut quieta,
        &antes,
        &c,
        &w,
        Material {
            atrito: 1.0,
            salto: 0.0,
        },
    );
    assert_eq!(parada[1].to_bits(), 0.0_f32.to_bits(), "{}", parada[1]);
}

/// ⭐⭐⭐ **O SALTO VAI AO DOBRO, E O ATRITO NÃO** — ordem do dono (2026-09-13: *«quero mais
/// capacidade de Bounciness — de zero até o dobro do máximo atual»*).
///
/// ⚠️ **O atrito é o CONTROLO desta mudança:** sem ele, alargar «o tecto do material» teria
/// alargado os dois, e o gate ficaria verde sobre uma faixa que ninguém pediu. Um coeficiente de
/// Coulomb acima de `1` não compra nada — o impulso tangencial já está limitado ao que a normal
/// aguenta.
#[test]
fn the_bounce_reaches_twice_the_old_ceiling_and_the_friction_does_not() {
    use ph2d_nodegraph::attr::{BOUNCE_MAX, FRICTION_MAX};
    assert_eq!(BOUNCE_MAX, 2.0, "o dobro do `1` de todo motor");
    assert_eq!(FRICTION_MAX, 1.0, "e o atrito fica onde estava");
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
    assert_eq!(m[0].salto, 2.0, "o dobro e' autorável");
    assert_eq!(m[1].salto, BOUNCE_MAX, "acima do tecto, COAGE");
    assert_eq!(m[1].atrito, FRICTION_MAX, "e o atrito coage no dele");
    assert_eq!(m[2].salto, 0.0, "um negativo nao e' um pedido");
    assert_eq!(m[2].atrito, 0.0, "nem um infinito");
    // E o PAR: o maior dos dois, coagido pelo mesmo tecto.
    assert_eq!(super::atrito::salto(2.0, 0.0), 2.0);
    assert_eq!(super::atrito::salto(9.0, 0.0), BOUNCE_MAX);
}
