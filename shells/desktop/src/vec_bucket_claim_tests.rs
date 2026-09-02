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
