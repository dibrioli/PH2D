//! Os gates da malha — o que a tinta diz contra o que a malha entrega.

use super::*;

/// Uma grelha `w × h` com um rectângulo cheio de tinta.
fn caixa(w: usize, h: usize, x0: usize, y0: usize, x1: usize, y1: usize) -> Vec<u8> {
    let mut a = vec![0u8; w * h];
    for y in y0..y1 {
        for x in x0..x1 {
            a[y * w + x] = 255;
        }
    }
    a
}

/// Um disco de raio `r` centrado em `(cx, cy)`, com a borda SUAVIZADA — o alfa sobe de `0` a
/// `255` ao longo de um pixel, como num desenho a sério.
fn disco(w: usize, h: usize, cx: f64, cy: f64, r: f64) -> Vec<u8> {
    let mut a = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let d = (x as f64 - cx).hypot(y as f64 - cy);
            let cobertura = (r + 0.5 - d).clamp(0.0, 1.0);
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let v = (cobertura * 255.0).round() as u8;
            a[y * w + x] = v;
        }
    }
    a
}

/// ⭐⭐ **UM QUADRADO DÁ UM QUADRADO** — a lei mais simples das três, ponta a ponta.
///
/// ⚠️ O anel de pixels de um quadrado de `10×10` tem **36** pontos (o perímetro), e a
/// simplificação tem de o levar a **4**. Se ela devolvesse os 36, a malha teria 34 triângulos
/// onde bastam 2 — e o artista pagaria isso em todo quadro, para sempre.
#[test]
fn a_square_of_paint_becomes_a_square_of_mesh() {
    let a = caixa(20, 20, 5, 5, 15, 15);
    let aneis = contour(&a, 20, 20, 1);
    assert_eq!(aneis.len(), 1, "um rectangulo e' UMA ilha");
    assert_eq!(
        aneis[0].len(),
        36,
        "o anel de pixel de um quadrado 10x10 tem o perimetro dele"
    );
    let s = simplify(&aneis[0], 1.5);
    assert_eq!(
        s.len(),
        4,
        "a simplificacao tinha de achar os quatro cantos e mais nada — deu {s:?}"
    );
    let t = triangulate(&s);
    assert_eq!(t.len(), 2, "um quadrado sao DOIS triangulos");
}

/// ⭐⭐⭐ **A BORDA SUAVIZADA NÃO É APARADA** — o limiar é `1`, não `128`.
///
/// ⛔⛔ Cortar a meio come os dois ou três pixels em que o alfa sobe, e a silhueta fica **por
/// dentro** do desenho: ao deformar, o artista vê o contorno da própria arte a ser recortado.
/// *Um limiar é uma decisão sobre o que é tinta, e a resposta certa é «tudo o que se vê».*
///
/// (Mutação: `alpha_threshold: 128` ⇒ RED — o raio medido encolhe.)
#[test]
fn a_soft_edge_is_not_shaved_off() {
    let (w, h, r) = (48usize, 48usize, 16.0);
    let a = disco(w, h, 24.0, 24.0, r);
    let raio_de = |t: u8| -> f64 {
        let aneis = contour(&a, w, h, t);
        let anel = &aneis[0];
        anel.iter()
            .map(|p| (p[0] - 24.0).hypot(p[1] - 24.0))
            .fold(0.0f64, f64::max)
    };
    let cheio = raio_de(1);
    let meio = raio_de(128);
    assert!(
        cheio >= r,
        "com o limiar em 1 a silhueta tem de conter o disco inteiro: {cheio} < {r}"
    );
    assert!(
        meio < cheio,
        "a fixtura nao produz o fenomeno — os dois limiares tinham de dar raios diferentes"
    );
    assert_eq!(
        MeshOptions::default().alpha_threshold,
        1,
        "o valor de nascimento tem de ser o que NAO apara a borda"
    );
}

/// ⭐⭐⭐ **DUAS ILHAS DÃO DUAS SUB-MALHAS, e os índices da segunda não pisam os da primeira.**
///
/// ⛔ Um desenho com duas peças soltas — dois olhos, uma corrente partida — é um caso normal, e
/// tratá-lo como uma peça só obrigaria o artista a cortar o ficheiro. ⚠️ O erro caro aqui é
/// **esquecer o deslocamento**: os triângulos da segunda ilha indexariam os vértices da primeira,
/// e a malha sairia com triângulos a atravessar o vazio entre as duas.
///
/// (Mutação: `base = 0` no `mesh_of` ⇒ RED.)
#[test]
fn two_islands_give_two_sub_meshes_and_the_indices_do_not_collide() {
    let mut a = caixa(40, 20, 2, 5, 12, 15);
    for y in 5..15 {
        for x in 26..36 {
            a[y * 40 + x] = 255;
        }
    }
    let m = mesh_of(&a, 40, 20, MeshOptions::default()).expect("ha' tinta");
    assert_eq!(m.rest.len(), 8, "dois quadrados sao oito cantos");
    assert_eq!(
        m.triangle_count(),
        4,
        "dois quadrados sao quatro triangulos"
    );
    // ⚠️ Nenhum triângulo pode misturar as duas ilhas: os índices `0..4` são a primeira.
    for t in &m.tris {
        let primeira = t.iter().filter(|&&i| i < 4).count();
        assert!(
            primeira == 3 || primeira == 0,
            "o triangulo {t:?} atravessa as duas ilhas — faltou deslocar os indices"
        );
    }
}

/// ⭐⭐ **A UV DERIVA-SE DA POSIÇÃO, e o canto da imagem é o canto da textura.**
///
/// ⚠️ Ela não é guardada de propósito ([`Mesh2d`]): uma lista paralela de UVs é o *vector
/// paralelo* que o esqueleto já proíbe por escrito nos pesos.
#[test]
fn the_uv_is_derived_from_the_rest_position() {
    let a = caixa(100, 50, 0, 0, 100, 50);
    let m = mesh_of(&a, 100, 50, MeshOptions::default()).expect("tinta");
    for (i, p) in m.rest.iter().enumerate() {
        let uv = m.uv(i).expect("a imagem tem lado");
        assert!(
            (uv[0] - p[0] / 100.0).abs() < 1e-12 && (uv[1] - p[1] / 50.0).abs() < 1e-12,
            "a uv de {p:?} saiu {uv:?}"
        );
        assert!((0.0..=1.0).contains(&uv[0]) && (0.0..=1.0).contains(&uv[1]));
    }
    // ⛔ Um índice que não existe devolve `None`, nunca um zero calado.
    assert_eq!(m.uv(m.rest.len()), None);
}

/// ⭐⭐⭐ **A DENSIDADE É O ÚNICO NÚMERO, e ela responde na direcção certa** — mais tolerância,
/// menos triângulos.
///
/// ⚠️ **A tabela é a régua, não uma promessa**: o que o gate exige é a MONOTONIA (a resposta
/// nunca sobe ao afrouxar) e que os extremos sejam de facto diferentes. O valor de nascimento é
/// decisão de produto e o smoke é quem o julga.
///
/// (Mutação: o `mesh_of` ignorar o `tolerance` ⇒ RED, os três leem igual.)
#[test]
fn the_density_knob_is_the_only_number_and_it_answers_in_the_right_direction() {
    let a = disco(64, 64, 32.0, 32.0, 26.0);
    let conta = |tol: f64| {
        mesh_of(
            &a,
            64,
            64,
            MeshOptions {
                tolerance: tol,
                ..MeshOptions::default()
            },
        )
        .expect("tinta")
        .triangle_count()
    };
    let (fino, medio, grosso) = (conta(0.3), conta(1.5), conta(6.0));
    assert!(
        fino > medio && medio > grosso,
        "a densidade nao responde: {fino} / {medio} / {grosso}"
    );
    assert!(
        grosso >= 1,
        "afrouxar ate' 6 px nao pode apagar a malha — sobra sempre um poligono"
    );
}

/// ⛔ **UMA GRELHA VAZIA NÃO DÁ MALHA NENHUMA** — e o `None` é a resposta, não uma malha vazia.
///
/// ⚠️ As duas leem-se iguais a jusante e significam coisas opostas: uma malha vazia diz *«esta
/// imagem não se move»*, e o `None` diz *«não havia o que prender»*. É o mesmo par que o repo já
/// pagou com o *«um zero de não-medido e um de perfeito são o mesmo byte»*.
#[test]
fn no_paint_is_no_mesh_and_that_is_a_none() {
    assert_eq!(
        mesh_of(&vec![0u8; 400], 20, 20, MeshOptions::default()),
        None
    );
    // Um pixel só também não: um ponto não tem interior.
    let mut a = vec![0u8; 400];
    a[210] = 255;
    assert_eq!(mesh_of(&a, 20, 20, MeshOptions::default()), None);
    // E uma grelha mal formada (bytes a menos) não estoura.
    assert!(contour(&[255, 255], 20, 20, 1).is_empty());
}

/// ⭐⭐⭐ **O ISTMO DE UM PIXEL — o critério de paragem de Jacob.**
///
/// ⛔⛔ Numa forma em halter (dois blocos ligados por uma ponte de um pixel) o rastreio passa pelo
/// pixel inicial **duas vezes**, e um critério ingénuo (*«voltei ao início»*) pára na primeira e
/// entrega **metade** da ilha. É a armadilha clássica deste algoritmo.
///
/// (Mutação: parar só na coincidência de pixel, ignorando a direcção de entrada ⇒ RED.)
#[test]
fn a_one_pixel_isthmus_does_not_cut_the_ring_in_half() {
    let mut a = vec![0u8; 40 * 12];
    for y in 3..9 {
        for x in 2..10 {
            a[y * 40 + x] = 255;
        }
        for x in 26..34 {
            a[y * 40 + x] = 255;
        }
    }
    // A ponte: uma linha de UM pixel de altura.
    for x in 10..26 {
        a[6 * 40 + x] = 255;
    }
    let aneis = contour(&a, 40, 12, 1);
    assert_eq!(aneis.len(), 1, "o halter e' UMA ilha ligada");
    let anel = &aneis[0];
    let xs: Vec<f64> = anel.iter().map(|p| p[0]).collect();
    let (menor, maior) = (
        xs.iter().copied().fold(f64::MAX, f64::min),
        xs.iter().copied().fold(f64::MIN, f64::max),
    );
    assert!(
        menor <= 2.0 && maior >= 33.0,
        "o anel foi de x={menor} a x={maior} — ele parou no istmo e perdeu o outro bloco"
    );
}

/// ⭐⭐ **A ÁREA COM SINAL é uma porta só**, e as três leis concordam por a usarem.
#[test]
fn the_signed_area_is_one_door() {
    let quadrado = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
    assert!((signed_area(&quadrado) - 16.0).abs() < 1e-12);
    let invertido: Vec<[f64; 2]> = quadrado.iter().rev().copied().collect();
    assert!((signed_area(&invertido) + 16.0).abs() < 1e-12);
    assert_eq!(signed_area(&quadrado[..2]), 0.0, "dois pontos nao tem area");
    // ⚠️ E o triangulador normaliza: as duas orientações dão o MESMO número de triângulos.
    assert_eq!(triangulate(&quadrado).len(), triangulate(&invertido).len());
}

/// ⭐⭐ **UM POLÍGONO CÔNCAVO não perde a concavidade** — o *ear-clipping* recusa a orelha que
/// contém outro vértice.
///
/// ⚠️ É o caso que separa um triangulador de um leque: um `L` triangulado por leque a partir de
/// um canto **atravessa o vazio**, e a imagem apareceria onde não há tinta.
#[test]
fn a_concave_polygon_keeps_its_concavity() {
    // Um "L".
    let l = [
        [0.0, 0.0],
        [6.0, 0.0],
        [6.0, 2.0],
        [2.0, 2.0],
        [2.0, 6.0],
        [0.0, 6.0],
    ];
    let t = triangulate(&l);
    assert_eq!(t.len(), 4, "um hexagono da' quatro triangulos");
    // Nenhum triângulo pode cobrir o canto vazio (5,5).
    let vazio = [5.0, 5.0];
    for tri in &t {
        let (a, b, c) = (l[tri[0] as usize], l[tri[1] as usize], l[tri[2] as usize]);
        let cross = |u: [f64; 2], v: [f64; 2], w: [f64; 2]| {
            (v[0] - u[0]).mul_add(w[1] - u[1], -((v[1] - u[1]) * (w[0] - u[0])))
        };
        let dentro =
            cross(a, b, vazio) > 0.0 && cross(b, c, vazio) > 0.0 && cross(c, a, vazio) > 0.0;
        assert!(!dentro, "o triangulo {tri:?} cobre o vazio do L");
    }
}

/// **A cápsula do smoke** — a arte do braço pintado, em alfa.
fn capsula(w: usize, h: usize) -> Vec<u8> {
    let (raio, ax, bx) = (
        h as f64 / 2.0 - 3.0,
        h as f64 / 2.0,
        w as f64 - h as f64 / 2.0,
    );
    let mut a = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let p = [x as f64 + 0.5, y as f64 + 0.5];
            let cx = p[0].clamp(ax, bx);
            let d = (p[0] - cx).hypot(p[1] - h as f64 / 2.0);
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let v = ((raio + 1.0 - d).clamp(0.0, 1.0) * 255.0) as u8;
            a[y * w + x] = v;
        }
    }
    a
}

/// O aspecto de um triângulo: o maior lado sobre a menor altura. `1` é equilátero.
fn aspecto(p: [[f64; 2]; 3]) -> f64 {
    let lados: Vec<f64> =
        (0..3).map(|k| (p[(k + 1) % 3][0] - p[k][0]).hypot(p[(k + 1) % 3][1] - p[k][1])).collect();
    let s = lados.iter().copied().fold(0.0f64, f64::max);
    let area = ((p[1][0] - p[0][0]) * (p[2][1] - p[0][1])
        - (p[1][1] - p[0][1]) * (p[2][0] - p[0][0]))
        .abs()
        / 2.0;
    let altura = if s > 0.0 { 2.0 * area / s } else { 0.0 };
    if altura > 0.0 { s / altura } else { f64::INFINITY }
}

/// ⭐⭐⭐ **A MALHA TEM MIOLO, E AS CÉLULAS SÃO QUADRADAS** — o report do dono de 2026-09-10, dito
/// como número.
///
/// > *«a malha criada automaticamente é de péssima qualidade»*
///
/// A malha anterior era o **contorno** triangulado por *ear-clipping*. Medida sobre esta MESMA
/// cápsula, ela dava:
///
/// | | contorno | esta grelha |
/// |---|---:|---:|
/// | vértices | `18` | > 100 |
/// | **no MIOLO** | **`0`** | a maioria |
/// | aspecto mediano | **`17,42`** | `< 3` |
/// | pior | **`53,10`** | `< 12` |
///
/// ⛔⛔ **O `0` no miolo é a causa inteira**: toda a deformação passava pela borda, e as lascas do
/// leque cisalhavam a arte.
///
/// ⚠️ **A barra do aspecto NÃO é escolhida: `2` é o CHÃO** — um quadrado partido em dois dá dois
/// triângulos rectângulos isósceles, cuja razão maior-lado/menor-altura é exactamente `2`. Pedir
/// menos seria pedir o impossível a uma grelha.
///
/// (Mutação: voltar ao `mesh_of` do contorno ⇒ RED nas quatro colunas.)
#[test]
fn the_mesh_has_a_middle_and_the_cells_are_square() {
    let (w, h) = (320usize, 96usize);
    let a = capsula(w, h);
    let focos: Vec<[f64; 2]> = (0..=3).map(|k| [48.0 + f64::from(k) * 74.6, 48.0]).collect();
    let m = grid_mesh_of(&a, w as u32, h as u32, &focos, GridOptions::default()).expect("tinta");

    // Um vértice está no MIOLO se está a mais de `4 px` da borda da cápsula.
    let (raio, ax, bx) = (h as f64 / 2.0 - 3.0, h as f64 / 2.0, w as f64 - h as f64 / 2.0);
    let miolo = m
        .rest
        .iter()
        .filter(|p| {
            let cx = p[0].clamp(ax, bx);
            (raio - (p[0] - cx).hypot(p[1] - h as f64 / 2.0)).abs() >= 4.0
        })
        .count();
    assert!(
        miolo > m.rest.len() / 2,
        "so' {miolo} de {} vertices no miolo — a deformacao volta a passar pela BORDA",
        m.rest.len()
    );
    assert!(m.rest.len() > 100, "vertices a menos: {}", m.rest.len());

    let mut asp: Vec<f64> = m
        .tris
        .iter()
        .map(|t| aspecto([
            m.rest[t[0] as usize],
            m.rest[t[1] as usize],
            m.rest[t[2] as usize],
        ]))
        .collect();
    asp.sort_by(f64::total_cmp);
    let (p50, pior) = (asp[asp.len() / 2], asp[asp.len() - 1]);
    assert!(
        p50 < 3.0,
        "aspecto mediano {p50:.2} — as celulas nao sao quadradas (o chao teorico e' 2,0)"
    );
    assert!(pior < 12.0, "pior aspecto {pior:.2} — ha' lascas na malha");
}

/// ⭐⭐⭐ **UMA ARTICULAÇÃO ADENSA A GRELHA À VOLTA DELA** — a segunda metade do pedido.
///
/// > *«deveria ser um quadmesh inteligente com maior densidade nas áreas das articulações»*
///
/// ⚠️ **A régua é o ESPAÇAMENTO, medido nos dois sítios** — perto da articulação e longe dela —, e
/// não a contagem total: uma malha uniformemente fina também teria mais vértices, e não é isso que
/// foi pedido.
///
/// (Mutação: o `passo` devolver `coarse` sempre ⇒ RED.)
#[test]
fn a_joint_makes_the_grid_denser_around_it() {
    let opts = GridOptions {
        fine: 8.0,
        coarse: 40.0,
        radius: 30.0,
        ..GridOptions::default()
    };
    let xs = axis_samples(0.0, 400.0, &[200.0], opts);
    let vao = |a: f64, b: f64| -> f64 {
        let d: Vec<f64> = xs
            .windows(2)
            .filter(|w| w[0] >= a && w[1] <= b)
            .map(|w| w[1] - w[0])
            .collect();
        d.iter().sum::<f64>() / d.len() as f64
    };
    let perto = vao(180.0, 220.0);
    let longe = vao(0.0, 100.0);
    assert!(
        perto < longe * 0.6,
        "junto da articulacao o vao e' {perto:.1} e longe {longe:.1} — nao adensou"
    );
    assert!(perto <= 12.0, "o vao junto da dobra ficou em {perto:.1}");
    // ⚠️ E o corte cai EXACTAMENTE na articulação — a marcha não a salta.
    assert!(
        xs.iter().any(|x| (x - 200.0).abs() < 1e-9),
        "a articulacao em 200 nao virou corte: {xs:?}"
    );
}

/// ⭐⭐ **A MARCHA NUNCA SALTA UMA ARTICULAÇÃO**, mesmo com o passo largo a começar antes dela.
///
/// ⛔ Sem essa guarda a dobra fica no MEIO de uma célula grande, e o adensamento existe na tabela
/// e não no sítio que interessa.
///
/// ⚠️⚠️ **A lei é «a menos de um quarto do passo fino», e não «em cima»** — e a diferença não é
/// folga: uma dobra que cai a `1 px` de um corte que já existe **está** naquele corte, e obrigar um
/// segundo corte ali produziria uma tira de `1 px` de largura, isto é, a célula de aspecto enorme
/// que esta wave inteira existe para apagar. ⚠️ A primeira redacção deste gate exigia igualdade e
/// **reprovou sobre produto correcto**.
///
/// (Mutação: tirar o `nx.min(f)` ⇒ RED — a dobra do meio fica a `5` de distância, o dobro da barra.)
#[test]
fn the_march_never_steps_over_a_joint() {
    let opts = GridOptions {
        fine: 10.0,
        coarse: 10.0,
        radius: 0.5,
        ..GridOptions::default()
    };
    let focos = [35.0, 64.0, 95.0];
    // Um passo fixo de 10 a partir de 0 passa por cima das três.
    let xs = axis_samples(0.0, 200.0, &focos, opts);
    let barra = opts.fine * 0.25;
    for f in focos {
        let d = xs.iter().map(|x| (x - f).abs()).fold(f64::INFINITY, f64::min);
        assert!(
            d <= barra + 1e-9,
            "a articulacao em {f} ficou a {d:.2} do corte mais proximo (barra {barra:.2}): {xs:?}"
        );
    }
}

/// ⛔ **UMA GRELHA MAIS GROSSA PERTO DA DOBRA É O OPOSTO DO PEDIDO**, e a porta COAGE.
///
/// ⚠️ Recusar em silêncio seria pior: o artista poria `coarse < fine` e a malha sairia ao contrário
/// sem ninguém dizer porquê.
#[test]
fn a_coarse_smaller_than_fine_is_coerced_not_obeyed() {
    let invertido = GridOptions {
        fine: 20.0,
        coarse: 5.0,
        radius: 30.0,
        ..GridOptions::default()
    };
    let xs = axis_samples(0.0, 200.0, &[100.0], invertido);
    for w in xs.windows(2) {
        let vao = w[1] - w[0];
        assert!(
            vao >= 20.0 - 1e-9 || w[1] >= 200.0 - 1e-9,
            "um vao de {vao:.2} ficou abaixo do `fine` — a coacao nao aconteceu"
        );
    }
}

/// ⛔ **CÉLULAS SEM TINTA NÃO ENTRAM** — e sem tinta nenhuma a resposta é `None`.
///
/// ⚠️ A malha **COBRE** a silhueta e não a segue (o *Expansion* do Puppet): o recorte fino é do
/// alfa da própria arte. Este gate mede a outra metade — que o vazio LONGE da tinta fica de fora.
#[test]
fn cells_without_paint_are_dropped() {
    // Tinta só no canto superior esquerdo de uma grelha grande.
    let (w, h) = (200usize, 200usize);
    let mut a = vec![0u8; w * h];
    for y in 0..30 {
        for x in 0..30 {
            a[y * w + x] = 255;
        }
    }
    let m = grid_mesh_of(&a, w as u32, h as u32, &[], GridOptions::default()).expect("tinta");
    for p in &m.rest {
        assert!(
            p[0] <= 80.0 && p[1] <= 80.0,
            "o vertice {p:?} nasceu longe de toda a tinta"
        );
    }
    assert_eq!(
        grid_mesh_of(&vec![0u8; w * h], w as u32, h as u32, &[], GridOptions::default()),
        None,
        "sem tinta a resposta e' None, nunca uma malha vazia"
    );
}
