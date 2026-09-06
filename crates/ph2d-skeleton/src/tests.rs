//! Os gates da LEI do esqueleto. ⚠️ Cada um mede uma coisa que, errada, produz um sintoma
//! **visível** — e o nome diz o sintoma, não a função.
//!
//! ⚠️ **O que NÃO está aqui é o que uma mídia sabe.** «As três metades de um vértice pesam pela
//! posição delas» é gate do vetor e mora na `ph2d-vec-skin`: escrito aqui, ele obrigaria esta
//! crate a conhecer um `VecPath`, que é exactamente o que ela existe para não conhecer.

use super::*;

/// Um osso deitado sobre o eixo X, de `(x0,0)` a `(x0+len, 0)`, em repouso e sem pose.
fn osso(x0: f64, len: f64, strength: f64) -> SkinBone {
    SkinBone::new(
        Xform([1.0, 0.0, 0.0, 1.0, x0, 0.0]),
        len,
        strength,
        // `bone_world` = o mesmo repouso, `shape_world_inv` = identidade ⇒ pose = identidade.
        Xform([1.0, 0.0, 0.0, 1.0, x0, 0.0]),
        Xform::IDENTITY,
    )
    .expect("repouso nao-singular")
}

/// O mesmo osso, mas POSADO: transladado de `(dx, dy)` em mundo.
fn osso_movido(x0: f64, len: f64, strength: f64, d: [f64; 2]) -> SkinBone {
    SkinBone::new(
        Xform([1.0, 0.0, 0.0, 1.0, x0, 0.0]),
        len,
        strength,
        Xform([1.0, 0.0, 0.0, 1.0, x0 + d[0], d[1]]),
        Xform::IDENTITY,
    )
    .expect("repouso nao-singular")
}

/// Um quadrado de 10, como nuvem de pontos crua — a fixtura da mídia mais simples que existe.
fn quadrado() -> Vec<[f64; 2]> {
    vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
}

/// ⭐⭐⭐ **NO REPOUSO, NINGUÉM SE MEXE** — a lei da casa (todo motor novo é no-op no ponto neutro).
///
/// ⚠️ **E ela não é byte-exacta, de propósito: é exacta na ÁLGEBRA e a `f64` arredonda a mistura.**
/// `w₁·p + w₂·p + w₃·p` com pesos que somam `1` não devolve `p` ao bit quando os pesos não são
/// potências de dois — e é por isso que a barra aqui é um número MEDIDO e não um `assert_eq!`.
/// Medido nesta fixtura (3 ossos a cobrir o quadrado): pior desvio **0** de facto, e a barra fica em
/// `1e-12` porque *uma barra calibrada no melhor caso reprova na primeira fixtura menos simpática*.
#[test]
fn a_skeleton_at_rest_moves_nothing() {
    let pele = Skin::new(vec![
        osso(0.0, 6.0, 1.5),
        osso(5.0, 6.0, 1.5),
        osso(2.0, 3.0, 4.0),
    ])
    .expect("3 ossos");
    let antes = quadrado();
    let mut depois = antes.clone();
    pele.deform_points(depois.iter_mut());
    let mut pior = 0.0_f64;
    for (p, q) in antes.iter().zip(depois.iter()) {
        pior = pior.max((p[0] - q[0]).abs()).max((p[1] - q[1]).abs());
    }
    assert!(
        pior < 1e-12,
        "o repouso moveu os pontos em {pior} - o `rest` nao esta' a ser invertido pela pose"
    );
}

/// ⭐ **A PORTA DA MÍDIA DÁ O MESMO QUE O LAÇO À MÃO** — `deform_points` não pode ser um segundo
/// caminho com uma lei própria; ela é `point` num laço, e o gate diz isso ao BIT.
///
/// ⚠️ Sem ele, uma optimização futura dentro de `deform_points` (partilhar pesos entre pontos
/// vizinhos, por exemplo) passaria despercebida em toda mídia que use a porta.
#[test]
fn the_media_door_is_the_same_law_as_the_hand_written_loop() {
    let pele = Skin::new(vec![
        osso_movido(0.0, 6.0, 1.5, [1.0, -3.0]),
        osso(5.0, 6.0, 1.5),
    ])
    .expect("2 ossos");
    let mut pela_porta = quadrado();
    pele.deform_points(pela_porta.iter_mut());
    let mut w = pele.scratch();
    for (i, p) in quadrado().into_iter().enumerate() {
        assert_eq!(
            pela_porta[i],
            pele.point(p, &mut w),
            "a porta da midia divergiu do laco a mao no ponto {i}"
        );
    }
}

/// ⛔⛔ **O ÓRFÃO NUNCA FICA PARA TRÁS** — a razão de o suporte ser finito ([doc 47 §2.4]).
///
/// Um ponto fora do alcance de todo osso prende-se **rigidamente ao mais próximo**: mexer o
/// esqueleto leva-o inteiro. Com a lei global (`1/d²`) ele seguiria a MÉDIA dos ossos, e o sintoma
/// é a aba de um chapéu a atrasar-se atrás da cabeça.
#[test]
fn an_orphan_point_rides_the_nearest_bone_and_is_never_left_behind() {
    let perto = osso_movido(0.0, 2.0, 0.5, [0.0, 9.0]);
    let longe = osso(60.0, 2.0, 0.5);
    let pele = Skin::new(vec![perto, longe]).unwrap();
    let p = [1.0, 40.0]; // fora dos dois raios (raio = 1,0)
    let mut w = pele.scratch();
    assert!(
        !pele.weights_at(p, &mut w),
        "este ponto tinha de ser ORFAO - a fixtura deixou de medir o que promete"
    );
    assert_eq!(
        w,
        vec![1.0, 0.0],
        "o orfao tem de ir INTEIRO para o mais perto"
    );
    let q = pele.point(p, &mut w);
    assert!(
        (q[1] - (p[1] + 9.0)).abs() < 1e-9 && (q[0] - p[0]).abs() < 1e-9,
        "o orfao andou {q:?} em vez de acompanhar o osso mais perto (+9 em y)"
    );
}

/// ⭐ **O PESO ATRAVESSA A FRONTEIRA SEM ESTALO** — o bump é C¹ (`f(1)=0` e `f'(1)=0`).
///
/// ⚠️ **É este gate que proíbe uma poda por baixo.** Cortar pesos abaixo de um piso devolveria
/// exactamente o salto que este número mede — e mediria-se como um TREMOR na forma quando um osso
/// entra e sai de alcance.
#[test]
fn the_weight_crosses_the_edge_of_its_reach_without_a_step() {
    let pele = Skin::new(vec![osso(0.0, 10.0, 1.0), osso(0.0, 10.0, 3.0)]).unwrap();
    let (mut w, mut anterior, mut maior_salto) = (pele.scratch(), None::<f64>, 0.0_f64);
    // Varre a distância perpendicular ao eixo, atravessando o raio do 1.º osso (10) devagar.
    for i in 0..=4000 {
        let y = f64::from(i) * 0.005; // 0 .. 20
        pele.weights_at([5.0, y], &mut w);
        if let Some(a) = anterior {
            maior_salto = maior_salto.max((w[0] - a).abs());
        }
        anterior = Some(w[0]);
    }
    assert!(
        maior_salto < 5e-3,
        "o peso saltou {maior_salto} entre duas amostras a 0,005 de distancia - a lei deixou de ser C1"
    );
    pele.weights_at([5.0, 10.0], &mut w);
    assert!(
        w[0] < 1e-12,
        "na BORDA do raio o peso tem de ser zero, e foi {}",
        w[0]
    );
}

/// ⭐ **A LEI É ADIMENSIONAL** — o mesmo rig dez vezes maior pesa igual.
///
/// A razão de o raio sair do comprimento do osso (e não de um número escrito): um personagem
/// desenhado em unidades grandes e outro em pequenas têm de deformar-se do mesmo modo.
#[test]
fn the_same_rig_ten_times_bigger_weighs_exactly_the_same() {
    let pequeno = Skin::new(vec![osso(0.0, 4.0, 1.0), osso(4.0, 4.0, 1.0)]).unwrap();
    let grande = Skin::new(vec![osso(0.0, 40.0, 1.0), osso(40.0, 40.0, 1.0)]).unwrap();
    let (mut a, mut b) = (pequeno.scratch(), grande.scratch());
    for (x, y) in [(1.0, 0.5), (3.9, 2.0), (5.0, 3.5), (7.0, 0.0)] {
        pequeno.weights_at([x, y], &mut a);
        grande.weights_at([x * 10.0, y * 10.0], &mut b);
        for i in 0..2 {
            assert!(
                (a[i] - b[i]).abs() < 1e-12,
                "escala mudou o peso do osso {i} em ({x},{y}): {} contra {}",
                a[i],
                b[i]
            );
        }
    }
}

/// **Um ponto SOBRE o eixo, longe do resto, é rígido** — o caso que faz uma junta parecer uma
/// junta em vez de uma mancha.
#[test]
fn a_point_on_a_lone_bones_axis_rides_it_rigidly() {
    let pele = Skin::new(vec![
        osso_movido(0.0, 10.0, 1.0, [3.0, -2.0]),
        osso(100.0, 10.0, 1.0),
    ])
    .unwrap();
    let mut w = pele.scratch();
    let q = pele.point([5.0, 0.0], &mut w);
    assert!(
        (q[0] - 8.0).abs() < 1e-9 && (q[1] + 2.0).abs() < 1e-9,
        "o ponto no eixo devia ir para (8,-2) e foi para {q:?}"
    );
}

/// **Uma pele sem osso nenhum é a AUSÊNCIA de pele, não a identidade.** Se ela nascesse "vazia mas
/// válida", uma forma cujos ossos foram todos apagados seria passada por um mapa que soma zero
/// pesos — e colapsaria na origem, sem uma linha de erro.
#[test]
fn a_skin_with_no_bones_refuses_to_exist() {
    assert!(Skin::new(Vec::new()).is_none());
}

/// ⭐ **O PREÇO de derivar os pesos por quadro** — a medição que autoriza o §2.3 do doc 47 (*guardar
/// o bind, nunca o peso*).
///
/// ⛔ **`#[ignore]`, e não é um gate**: ele IMPRIME. Um teto de relógio aqui seria mais um membro
/// da família de flakes de recurso que o `CLAUDE.md` §5.0 lista — o que se quer deste número é a
/// ORDEM DE GRANDEZA contra um quadro de 16,7 ms, e ela decide-se uma vez.
#[test]
#[ignore = "sonda de relógio: imprime, não julga"]
fn measure_the_price_of_deriving_the_weights_every_frame() {
    // A MESMA peça de sempre: 200 vértices × 3 metades = 600 pontos, sobre 12 ossos.
    let ossos: Vec<SkinBone> = (0..12)
        .map(|i| osso(f64::from(i) * 8.0, 8.0, 1.5))
        .collect();
    let pele = Skin::new(ossos).expect("12 ossos");
    let pontos: Vec<[f64; 2]> = (0..600)
        .map(|i| {
            let t = f64::from(i) * 0.166;
            [t, (t * 0.3).sin() * 4.0]
        })
        .collect();
    let t0 = std::time::Instant::now();
    const N: u32 = 200;
    for _ in 0..N {
        let mut p = pontos.clone();
        pele.deform_points(p.iter_mut());
        std::hint::black_box(&p);
    }
    let us = t0.elapsed().as_secs_f64() * 1e6 / f64::from(N);
    println!(
        "[skeleton] 600 pontos x 12 ossos: {us:.2} us por quadro ({:.3} % de 16,7 ms)",
        us / 16_700.0 * 100.0
    );
}

/// A distância ao SEGMENTO, não à recta — um ponto além da ponta mede à ponta.
#[test]
fn the_distance_is_to_the_segment_not_to_its_infinite_line() {
    let (a, b) = ([0.0, 0.0], [10.0, 0.0]);
    assert!((dist2_to_segment([5.0, 3.0], a, b) - 9.0).abs() < 1e-12);
    assert!(
        (dist2_to_segment([14.0, 3.0], a, b) - 25.0).abs() < 1e-12,
        "alem da ponta a distancia e' ate' a PONTA (4,3 -> 25), nao a perpendicular"
    );
    assert!((dist2_to_segment([7.0, 0.0], [3.0, 3.0], [3.0, 3.0]) - 25.0).abs() < 1e-12);
}
