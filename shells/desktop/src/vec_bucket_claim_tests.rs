//! Gates da HERANÇA de uma face ([`super::donos`]).

use super::*;
use ph2d_vec_scene::{VecVertex, VertexKind, detection_polyline};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn quadrado(r: f64) -> (Vec<VecVertex>, bool) {
    (vec![v(-r, -r), v(r, -r), v(r, r), v(-r, r)], true)
}

/// Uma linha que atravessa o quadrado de lado a lado, de `y0` à esquerda a `y1` à direita.
fn linha(y0: f64, y1: f64) -> (Vec<VecVertex>, bool) {
    (vec![v(-20.0, y0), v(20.0, y1)], false)
}

/// A região de um rectângulo, como o preenchimento a teria guardado.
fn regiao(lo: [f64; 2], hi: [f64; 2], semente: [f64; 2]) -> Regiao {
    let verts = vec![
        v(lo[0], lo[1]),
        v(hi[0], lo[1]),
        v(hi[0], hi[1]),
        v(lo[0], hi[1]),
    ];
    Regiao {
        poligonos: vec![detection_polyline(&verts, true)],
        semente,
    }
}

fn faces_de(contornos: &[(Vec<VecVertex>, bool)]) -> (Rede, Vec<Face>) {
    let r = ph2d_vec_fill::rede(contornos);
    let f = r.faces().into_iter().filter(|f| f.area > 0.0).collect();
    (r, f)
}

/// ⭐⭐⭐ **PARTIR UMA REGIÃO PINTA AS DUAS METADES** — a metade do report que faz a tinta sumir.
///
/// Medido: o quadrado de área `400` vira duas faces de `200`, e com UMA semente a tinta ficava com
/// uma delas — *metade da cor desaparecia*.
#[test]
fn splitting_a_painted_region_paints_both_halves() {
    let (rede, faces) = faces_de(&[quadrado(10.0), linha(0.0, 0.0)]);
    assert_eq!(faces.len(), 2, "a fixtura tem de partir a regiao em duas");
    // O preenchimento tinha o quadrado INTEIRO.
    let regioes = vec![regiao([-10.0, -10.0], [10.0, 10.0], [0.0, 0.0])];

    let d = donos(&rede, &faces, &regioes);

    assert_eq!(
        d,
        vec![Some(0), Some(0)],
        "as DUAS metades tem de herdar a tinta de quem cobria o quadrado inteiro"
    );
}

/// ⭐⭐⭐ **FUNDIR DUAS REGIÕES DÁ A FACE À MAIOR** — a outra metade do report, em que duas tintas
/// passavam a pintar a mesma área, uma escondendo a outra.
///
/// ⚠️⚠️ **A margem desta fixtura é de 4 para 1, e isso é deliberado.** A votação amostra a face numa
/// grelha de 15×15, então a resolução dela é ~`1/15` da face: uma fila de amostras em cima da
/// parede antiga vale ~7% dos votos, e uma margem menor do que isso é decidida **pela grelha**, não
/// pela área. *A primeira redacção deste gate afirmava o vencedor de uma fusão com margem de 1%
/// (`4,17` contra `400`) — estava a medir o alinhamento da grelha e chamou-lhe lei.*
#[test]
fn merging_two_regions_gives_the_face_to_the_larger_contributor() {
    let (rede, faces) = faces_de(&[quadrado(10.0)]);
    assert_eq!(faces.len(), 1, "sem parede a regiao e' UMA face");
    // A parede desapareceu: a tira de cima cobria 80 da face, a de baixo 320.
    let cima = regiao([-10.0, 6.0], [10.0, 10.0], [0.0, 8.0]);
    let baixo = regiao([-10.0, -10.0], [10.0, 6.0], [0.0, -2.0]);

    let d = donos(&rede, &faces, &[cima, baixo]);

    assert_eq!(
        d[0],
        Some(1),
        "a face fundida e' quase toda da tira de BAIXO — e' dela"
    );
}

/// ⛔ **Uma face que ninguém tinha pintado fica por pintar.** Sem esta metade, a lei passaria com
/// um `donos` que desse toda face à primeira região — e o balde inventaria decisões do artista.
#[test]
fn a_face_nobody_had_painted_stays_unpainted() {
    let (rede, faces) = faces_de(&[quadrado(10.0), linha(0.0, 0.0)]);
    // O preenchimento so' tinha a metade de CIMA.
    let regioes = vec![regiao([-10.0, 0.0], [10.0, 10.0], [0.0, 5.0])];

    let d = donos(&rede, &faces, &regioes);

    assert_eq!(
        d.iter().filter(|x| **x == Some(0)).count(),
        1,
        "so' a de cima"
    );
    assert_eq!(
        d.iter().filter(|x| x.is_none()).count(),
        1,
        "a de baixo fica por pintar"
    );
}

/// ⚠️ **O EMPATE é resolvido pela SEMENTE**, e tem de ser resolvido: duas regiões congruentes sobre
/// a mesma face dão exactamente o mesmo número de votos. ⛔ Um desempate ao acaso faria a cor
/// piscar entre duas enquanto a mão treme.
///
/// ⚠️ **A fixtura são duas tiras DISJUNTAS e simétricas** — sem fronteira partilhada, senão a fila
/// de amostras em cima dela desfaz o empate por acidente e o gate deixa de testar o desempate.
/// A semente de uma cai **dentro** da face e a da outra **fora**, e é isso que decide.
///
/// ⛔ E a resposta não pode vir da ORDEM: as duas chamadas trocam a lista e têm de concordar.
#[test]
fn a_tie_is_broken_by_the_seed_and_never_by_chance() {
    let (rede, faces) = faces_de(&[quadrado(10.0)]);
    assert_eq!(faces.len(), 1);
    // Duas tiras congruentes (8 x 20 cada), sem se tocarem, simétricas em x.
    let esq = || regiao([-10.0, -10.0], [-2.0, 10.0], [-50.0, -50.0]); // semente FORA da face
    let dir = || regiao([2.0, -10.0], [10.0, 10.0], [6.0, 0.0]); // semente DENTRO

    let d1 = donos(&rede, &faces, &[esq(), dir()]);
    let d2 = donos(&rede, &faces, &[dir(), esq()]);

    assert_eq!(
        d1[0],
        Some(1),
        "ganha quem tem a semente na face, e nao o indice"
    );
    assert_eq!(
        d2[0],
        Some(0),
        "e a resposta e' a MESMA com a lista trocada"
    );
}

/// ⭐⭐⭐ **UM PREENCHIMENTO PODE GANHAR VÁRIAS FACES, e a MAIOR vem à frente.**
///
/// ⚠️ Sem a primeira metade, partir uma região deixava metade por pintar — o report. Sem a segunda,
/// a semente nova iria viver na LASCA de uma região que partiu, e a lasca é justamente o pedaço que
/// a edição seguinte come.
#[test]
fn a_fill_can_win_several_faces_and_the_largest_leads() {
    let (_, faces) = faces_de(&[quadrado(10.0), linha(6.0, 6.0)]);
    assert_eq!(faces.len(), 2, "a linha alta parte o quadrado em desiguais");
    let (grande, pequena) = if faces[0].area > faces[1].area {
        (0, 1)
    } else {
        (1, 0)
    };
    // As duas sao do MESMO preenchimento (a regiao dele cobria o quadrado inteiro).
    let d = vec![Some(0), Some(0)];

    let out = por_preenchimento(&faces, &d, 1);

    assert_eq!(out.len(), 1);
    assert_eq!(
        out[0],
        vec![grande, pequena],
        "as duas faces sao dele, e a MAIOR vem a' frente"
    );
}

/// ⛔ **Uma face sem dono não entra em lista nenhuma** — senão o preenchimento cresceria para
/// regiões que o artista nunca pintou.
#[test]
fn a_face_without_an_owner_reaches_no_fill() {
    let (_, faces) = faces_de(&[quadrado(10.0), linha(0.0, 0.0)]);
    let out = por_preenchimento(&faces, &[Some(0), None], 2);
    assert_eq!(out[0].len(), 1);
    assert!(out[1].is_empty(), "o segundo preenchimento nao ganhou nada");
}

/// ⭐⭐⭐ **UMA REGIÃO COMPRIDA E MAGRA CONTINUA A HERDAR A TINTA** — o report de 2026-09-02 com
/// fotos (*"comportamento bem melhor mas com inconsistências e falhas"*, com as setas a nomear a
/// cor que cada lasca devia ter).
///
/// ⚠️⚠️ **A causa não era a lei da herança: era a RÉGUA dela.** As amostras saíam de uma grelha de
/// 15×15 sobre a **caixa** da face, e uma espiga na diagonal ocupa `1,3%` da caixa dela — medido,
/// **zero** pontos da grelha caíam lá dentro. Uma face sem amostra não vota, não é votada e não
/// herda nada. ⛔ E lia-se como intermitente, porque a grelha acerta ou falha conforme o ângulo.
///
/// | a região | área | densidade na caixa | amostras com a grelha | com a varredura |
/// |---|---|---|---|---|
/// | espiga larga | `2000` | `6,7%` | `8` | `9` |
/// | espiga fina | `400` | `1,3%` | **`0`** | `4` |
/// | espiga finíssima | `100` | `0,3%` | **`0`** | `4` |
#[test]
fn a_region_that_became_a_thin_diagonal_sliver_still_inherits_its_paint() {
    // A espiga: um triângulo comprido e magro na diagonal, como o que nasce ao arrastar um nó.
    let espiga = (vec![v(0.0, 0.0), v(200.0, 150.0), v(0.0, 4.0)], true);
    let (rede, faces) = faces_de(&[espiga]);
    assert_eq!(faces.len(), 1);
    assert!(
        faces[0].area < 0.02 * 200.0 * 150.0,
        "a fixtura tem de ser MAGRA: {} de uma caixa de 30000",
        faces[0].area
    );
    assert!(
        !rede.interior_samples(&faces[0]).is_empty(),
        "uma face magra sem amostra nao vota nem e' votada — era este o defeito"
    );
    // A tinta cobria a zona inteira antes de a região emagrecer.
    let antes = regiao([-10.0, -10.0], [210.0, 160.0], [50.0, 30.0]);

    let d = donos(&rede, &faces, &[antes]);

    assert_eq!(
        d[0],
        Some(0),
        "a lasca tem de continuar com a cor que tinha"
    );
}

/// ⚠️ **E a varredura NÃO pode ter deitado fora a proporcionalidade** — é dela que sai *"a face
/// fundida fica com a tinta da maior"*. Uma face gorda continua a dar muito mais votos a quem cobre
/// muito dela do que a quem cobre pouco.
#[test]
fn the_sweep_keeps_the_vote_proportional_to_the_area() {
    let (rede, faces) = faces_de(&[quadrado(10.0)]);
    let esquerda = regiao([-10.0, -10.0], [-6.0, 10.0], [-8.0, 0.0]); // 20% da face
    let direita = regiao([-6.0, -10.0], [10.0, 10.0], [2.0, 0.0]); // 80%
    let amostras = rede.interior_samples(&faces[0]);
    assert!(
        amostras.len() > 50,
        "a face gorda tem de dar muitas amostras"
    );

    assert_eq!(
        donos(&rede, &faces, &[esquerda, direita])[0],
        Some(1),
        "80% contra 20% nao pode empatar"
    );
}

/// A forma antes e depois de um nó ser arrastado para fora, criando uma ESPIGA.
///
/// O quadrado `(0,0)-(100,0)-(100,100)-(0,100)`; o canto de cima-direita vai para `(-50, 50)`, e a
/// aresta que vem de `(100,0)` atravessa o lado esquerdo em `(0; 33,3)` — nasce um lóbulo FORA do
/// quadrado, colado ao corpo pelo pedaço de aresta que sobra.
fn espiga() -> ((Vec<VecVertex>, bool), (Vec<VecVertex>, bool)) {
    let base = ph2d_vec_scene::ellipse([0.0, 0.0], 100.0, 100.0);
    let mut movido = base.verts.clone();
    // O nó de CIMA é arrastado para muito abaixo do círculo: as duas curvas vizinhas atravessam o
    // arco de BAIXO, e entre elas nasce um lóbulo que antes era fundo.
    let alvo = [0.0, -260.0];
    let topo = movido
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.anchor[1].total_cmp(&b.1.anchor[1]))
        .map(|(i, _)| i)
        .unwrap();
    movido[topo].anchor = alvo;
    movido[topo].in_handle = alvo;
    movido[topo].out_handle = alvo;
    ((base.verts, true), (movido, true))
}

/// ⭐⭐⭐ **UMA ÁREA NOVA, VARRIDA POR UM NÓ, HERDA DA VIZINHA COM QUEM MAIS CONFINA.**
///
/// Report do Enio (2026-09-02, cinco fotos, *"várias inconsistências"*): a mesma espiga saía ora
/// verde, ora vermelha, ora **sem cor nenhuma**. ⚠️ **Medido, e NÃO é histerese**: o mesmo arrasto
/// em 1, 4, 20 e 100 passos dá a MESMA resposta — o que muda entre duas fotos é a topologia. O que
/// era constante é que o terreno **novo** nunca era reclamado, e é o que ele pediu por palavras
/// dele: *"preenchendo corretamente as áreas novas que vão surgindo"*.
#[test]
fn a_new_area_swept_by_a_node_inherits_from_the_neighbour_it_borders_most() {
    let (antes, depois) = espiga();
    let (r0, f0) = faces_de(&[antes]);
    assert_eq!(f0.len(), 1, "antes e' UMA face");
    let regioes = vec![Regiao {
        poligonos: vec![r0.contorno(&f0[0])],
        semente: r0.interior_point(&f0[0]).unwrap(),
    }];
    let (r1, f1) = faces_de(&[depois]);
    assert!(f1.len() >= 2, "a espiga tem de criar um LOBULO novo");

    let mut d = donos(&r1, &f1, &regioes);
    let sem_dono = d.iter().filter(|x| x.is_none()).count();
    assert!(sem_dono >= 1, "a votacao deixa o lobulo sem dono: {d:?}");
    let nova = terreno_novo(&r1, &f1, &r0);
    assert!(nova.iter().any(|b| *b), "o lobulo e' TERRENO NOVO");
    let areas: Vec<f64> = f1.iter().map(|f| f.area).collect();
    herda_dos_vizinhos(&r1.adjacencias(&f1), &areas, &nova, &mut d);

    assert!(
        d.iter().all(std::option::Option::is_some),
        "toda face tem de acabar com cor: {d:?} (novas {nova:?})"
    );
}

/// ⛔⛔ **A TINTA NÃO ATRAVESSA PARA UMA REGIÃO QUE O ARTISTA DEIXOU VAZIA.**
///
/// ⚠️ Sem esta metade, a herança inundaria o desenho: uma rede de duas regiões com **uma** pintada
/// veria a outra ganhar cor ao primeiro arrasto de nó. A guarda é o [`terreno_novo`] — uma face que
/// já existia antes não é nova, por mais vizinha que seja.
#[test]
fn paint_never_crosses_into_a_region_the_artist_left_empty() {
    let (rede, faces) = faces_de(&[quadrado(10.0), linha(0.0, 0.0)]);
    assert_eq!(faces.len(), 2);
    // So' a metade de CIMA esta' pintada; a rede anterior e' a MESMA (o artista nao mexeu em nada).
    let regioes = vec![regiao([-10.0, 0.0], [10.0, 10.0], [0.0, 5.0])];
    let mut d = donos(&rede, &faces, &regioes);
    let vazias = d.iter().filter(|x| x.is_none()).count();
    assert_eq!(vazias, 1);

    let nova = terreno_novo(&rede, &faces, &rede);
    assert!(
        nova.iter().all(|b| !*b),
        "nada e' novo — a rede anterior e' esta mesma"
    );
    let areas: Vec<f64> = faces.iter().map(|f| f.area).collect();
    herda_dos_vizinhos(&rede.adjacencias(&faces), &areas, &nova, &mut d);

    assert_eq!(
        d.iter().filter(|x| x.is_none()).count(),
        vazias,
        "a metade vazia tem de CONTINUAR vazia"
    );
}

/// ⚠️ **A herança escolhe pela FRONTEIRA MAIS LONGA, e não pela primeira vizinha que aparece.**
/// Contar arcos daria o mesmo peso a uma lasca e a meia volta de um círculo.
#[test]
fn the_inheritance_follows_the_longest_shared_border() {
    let adj = vec![
        vec![(1usize, 100.0)],
        vec![(0usize, 100.0), (2usize, 5.0)],
        vec![(1usize, 5.0)],
    ];
    let mut d = vec![Some(7), None, Some(9)];
    herda_dos_vizinhos(&adj, &[1.0, 1.0, 1.0], &[false, true, false], &mut d);
    assert_eq!(d[1], Some(7), "a fronteira de 100 ganha a' de 5");
}

/// ⭐⭐ **A herança PROPAGA**: uma espiga que se parte em duas leva a cor da pintada para a primeira
/// nova e daí para a segunda. ⚠️ E cada ronda decide sobre o estado do INÍCIO dela — a ordem das
/// faces na lista não pode mudar a resposta.
#[test]
fn the_inheritance_propagates_and_does_not_depend_on_the_order() {
    let adj = vec![
        vec![(1usize, 10.0)],
        vec![(0usize, 10.0), (2usize, 10.0)],
        vec![(1usize, 10.0)],
    ];
    let mut a = vec![Some(3), None, None];
    herda_dos_vizinhos(&adj, &[1.0, 1.0, 1.0], &[false, true, true], &mut a);
    assert_eq!(
        a,
        vec![Some(3), Some(3), Some(3)],
        "a cor chega ao fim da cadeia"
    );
    let mut b = vec![None, None, Some(3)];
    herda_dos_vizinhos(&adj, &[1.0, 1.0, 1.0], &[true, true, false], &mut b);
    assert_eq!(b, vec![Some(3), Some(3), Some(3)]);
}
