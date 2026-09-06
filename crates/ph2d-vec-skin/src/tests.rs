//! Os gates da LEI da pele. ⚠️ Cada um mede uma coisa que, errada, produz um sintoma **visível** —
//! e o nome diz o sintoma, não a função.

use super::*;
use ph2d_vec_scene::{VecPath, VecVertex, Xform};

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

fn quadrado() -> VecPath {
    VecPath {
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([10.0, 0.0]),
            VecVertex::corner([10.0, 10.0]),
            VecVertex::corner([0.0, 10.0]),
        ],
        closed: true,
        ..VecPath::default()
    }
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
    pele.apply(&mut depois);
    let mut pior = 0.0_f64;
    for (a, b) in antes.verts_all().zip(depois.verts_all()) {
        for (p, q) in [
            (a.anchor, b.anchor),
            (a.in_handle, b.in_handle),
            (a.out_handle, b.out_handle),
        ] {
            pior = pior.max((p[0] - q[0]).abs()).max((p[1] - q[1]).abs());
        }
    }
    assert!(
        pior < 1e-12,
        "o repouso moveu a forma em {pior} - o `rest` nao esta' a ser invertido pela pose"
    );
}

/// ⭐⭐ **AS TRÊS METADES DE UM VÉRTICE RESPONDEM À POSIÇÃO DELAS** — o `CubicWeight` do Rive.
///
/// A âncora fica dentro do alcance do osso da ESQUERDA e a alça de saída dentro do da DIREITA; ao
/// mexer só o da direita, a alça anda e a âncora **não**. Pesar o vértice inteiro pela âncora
/// deixaria a alça parada, e o sintoma é a curva a rasgar-se numa junta.
#[test]
fn the_three_halves_of_a_vertex_answer_to_their_own_position() {
    let esq = osso(0.0, 4.0, 1.0);
    let dir_parado = osso(20.0, 4.0, 1.0);
    let dir_movido = osso_movido(20.0, 4.0, 1.0, [0.0, 7.0]);
    let v = VecVertex::smooth([1.0, 0.0], [-2.0, 0.0], [21.0, 0.0]);
    let forma = VecPath {
        verts: vec![v],
        ..VecPath::default()
    };

    let mut parado = forma.clone();
    Skin::new(vec![esq, dir_parado]).unwrap().apply(&mut parado);
    let mut movido = forma.clone();
    Skin::new(vec![esq, dir_movido]).unwrap().apply(&mut movido);

    let danca = |a: [f64; 2], b: [f64; 2]| (a[1] - b[1]).abs();
    assert!(
        danca(parado.verts[0].out_handle, movido.verts[0].out_handle) > 5.0,
        "a alca de saida esta' dentro do osso que se mexeu e nao o seguiu"
    );
    assert!(
        danca(parado.verts[0].anchor, movido.verts[0].anchor) < 1e-9,
        "a ancora esta' FORA do osso que se mexeu e mexeu-se na mesma - os pesos estao a sair da \
         posicao errada"
    );
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
    let mut w = vec![0.0; 2];
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
    let (mut w, mut anterior, mut maior_salto) = (vec![0.0; 2], None::<f64>, 0.0_f64);
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
    let (mut a, mut b) = (vec![0.0; 2], vec![0.0; 2]);
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
    let mut w = vec![0.0; 2];
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
    // Uma peça realista: um contorno de 200 vértices sobre um esqueleto de 12 ossos.
    let ossos: Vec<SkinBone> = (0..12)
        .map(|i| osso(f64::from(i) * 8.0, 8.0, 1.5))
        .collect();
    let pele = Skin::new(ossos).expect("12 ossos");
    let verts: Vec<VecVertex> = (0..200)
        .map(|i| {
            let t = f64::from(i) * 0.5;
            VecVertex::smooth([t, (t * 0.3).sin() * 4.0], [t - 0.2, 0.0], [t + 0.2, 0.0])
        })
        .collect();
    let forma = VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    };
    let t0 = std::time::Instant::now();
    const N: u32 = 200;
    for _ in 0..N {
        let mut p = forma.clone();
        pele.apply(&mut p);
        std::hint::black_box(&p);
    }
    let us = t0.elapsed().as_secs_f64() * 1e6 / f64::from(N);
    println!(
        "[skin] 200 vertices x 12 ossos = 600 pontos: {us:.2} us por quadro ({:.3} % de 16,7 ms)",
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
