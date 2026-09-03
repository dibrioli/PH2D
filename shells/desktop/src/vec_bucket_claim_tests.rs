//! Gates da HERANÇA por ÂNCORA ([`super::donos`]).

use super::*;
use ph2d_vec_scene::{VecVertex, VertexKind};

/// Um contorno como a rede o recebe.
type Contorno = (Vec<VecVertex>, bool);

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn quadrado(r: f64) -> Contorno {
    (vec![v(-r, -r), v(r, -r), v(r, r), v(-r, r)], true)
}

/// Uma linha que atravessa o quadrado de lado a lado, de `y0` à esquerda a `y1` à direita.
fn linha(y0: f64, y1: f64) -> Contorno {
    (vec![v(-20.0, y0), v(20.0, y1)], false)
}

/// A rede, as faces limitadas e as etiquetas — cada contorno finge ser um caminho próprio.
fn rede_de(cs: &[Contorno]) -> (Rede, Vec<Face>, Vec<(u64, u16)>) {
    let r = ph2d_vec_fill::rede(cs);
    let f = r.faces().into_iter().filter(|f| f.area > 0.0).collect();
    let t = (0..cs.len()).map(|i| (i as u64, 0u16)).collect();
    (r, f, t)
}

/// Um ponto BEM DENTRO de uma face — o que a sonda precisa e o produto já não: a semente do balde
/// é o CLIQUE, e nada a re-semeia. ⚠️ Sem esta separação, o gate mediria uma lei que saiu do produto.
fn miolo(r: &Rede, f: &Face) -> [f64; 2] {
    let poly = r.contorno(f);
    let n = poly.len() as f64;
    let c = [
        poly.iter().map(|p| p[0]).sum::<f64>() / n,
        poly.iter().map(|p| p[1]).sum::<f64>() / n,
    ];
    if ph2d_vec_scene::point_in_polygon(&poly, c) {
        return c;
    }
    poly.first().copied().unwrap_or([0.0, 0.0])
}

/// A receita de quem clicou na face que contém `p`.
fn receita_em(r: &Rede, faces: &[Face], tags: &[(u64, u16)], p: [f64; 2]) -> Vec<FillAnchor> {
    let f = r.face_em(p).expect("ha' face debaixo do clique");
    let _ = faces;
    ancoras_da_face(r, tags, &f)
}

/// ⭐⭐⭐ **ARRASTAR UM NÓ NÃO TROCA A COR DE NINGUÉM** — a lei que os quatro reports de 2026-09-01/02
/// pediam, e que nenhuma heurística sobre o quadro anterior podia dar.
///
/// ⚠️ **A receita é gravada UMA vez** (no clique, sobre o desenho de partida) e **nunca reescrita**.
/// O nó move-se; os arcos continuam a ser pedaços das mesmas curvas; cada face volta ao dono dela.
#[test]
fn dragging_a_node_keeps_every_region_with_its_own_paint() {
    let antes = [quadrado(10.0), linha(0.0, 0.0)];
    let (r0, f0, t0) = rede_de(&antes);
    assert_eq!(f0.len(), 2);
    let cima = receita_em(&r0, &f0, &t0, [0.0, 5.0]);
    let baixo = receita_em(&r0, &f0, &t0, [0.0, -5.0]);

    // O nó da esquerda da linha sobe: a parede inclina-se, e as duas metades mudam de forma.
    let depois = [quadrado(10.0), linha(6.0, -4.0)];
    let (r1, f1, t1) = rede_de(&depois);
    let d = donos(
        &r1,
        &f1,
        &t1,
        &[
            Receita {
                ancoras: &cima,
                semente: [0.0, 5.0],
            },
            Receita {
                ancoras: &baixo,
                semente: [0.0, -5.0],
            },
        ],
    );

    assert_eq!(f1.len(), 2, "continuam a ser duas regioes");
    let de_cima = f1
        .iter()
        .position(|f| miolo(&r1, f)[1] > 0.0)
        .expect("ha' uma metade de cima");
    assert_eq!(d[de_cima], Some(0), "a de cima continua a ser da de cima");
    assert_eq!(
        d[1 - de_cima],
        Some(1),
        "e a de baixo da de baixo — ninguem troca"
    );
}

/// ⭐⭐⭐ **PARTIR UMA REGIÃO PINTA AS DUAS METADES — e cai de graça.**
///
/// ⚠️ Não há uma linha de código sobre partir: a região tinha **várias** âncoras, e depois do corte
/// umas cercam uma metade e outras a outra. *É a redundância da receita que faz o trabalho.*
#[test]
fn splitting_a_region_paints_both_halves() {
    let (r0, f0, t0) = rede_de(&[quadrado(10.0)]);
    assert_eq!(f0.len(), 1);
    let tudo = receita_em(&r0, &f0, &t0, [0.0, 0.0]);
    assert!(
        tudo.len() >= 2,
        "a face tem de dar VARIAS ancoras: {tudo:?}"
    );

    let (r1, f1, t1) = rede_de(&[quadrado(10.0), linha(0.0, 0.0)]);
    assert_eq!(f1.len(), 2);
    let d = donos(
        &r1,
        &f1,
        &t1,
        &[Receita {
            ancoras: &tudo,
            semente: [0.0, 0.0],
        }],
    );

    assert_eq!(d, vec![Some(0), Some(0)], "as DUAS metades sao dele");
}

/// ⭐⭐ **FUNDIR DUAS REGIÕES DÁ A FACE A QUEM TEM MAIS ÂNCORAS NELA** — e nunca ao acaso.
#[test]
fn merging_two_regions_gives_the_face_to_whoever_has_more_anchors_on_it() {
    let (r0, f0, t0) = rede_de(&[quadrado(10.0), linha(6.0, 6.0)]);
    assert_eq!(f0.len(), 2);
    // A tira fina de cima e o resto: a de baixo é cercada por muito mais lado de quadrado.
    let fina = receita_em(&r0, &f0, &t0, [0.0, 8.0]);
    let larga = receita_em(&r0, &f0, &t0, [0.0, 0.0]);
    assert!(
        larga.len() > fina.len(),
        "a larga tem de trazer MAIS ancoras (elas espalham-se pelo comprimento): {} vs {}",
        larga.len(),
        fina.len()
    );

    // A parede desaparece: fica UMA face.
    let (r1, f1, t1) = rede_de(&[quadrado(10.0)]);
    assert_eq!(f1.len(), 1);
    let d = donos(
        &r1,
        &f1,
        &t1,
        &[
            Receita {
                ancoras: &fina,
                semente: [0.0, 8.0],
            },
            Receita {
                ancoras: &larga,
                semente: [0.0, 0.0],
            },
        ],
    );

    assert_eq!(d[0], Some(1), "a face fundida e' de quem mais a cercava");
}

/// ⛔ **Uma região que ninguém pintou fica por pintar.** Sem esta metade, a lei passaria com um
/// `donos` que desse toda face ao primeiro — e o balde inventaria decisões do artista.
#[test]
fn a_region_nobody_painted_stays_unpainted() {
    let (r0, f0, t0) = rede_de(&[quadrado(10.0), linha(0.0, 0.0)]);
    let so_cima = receita_em(&r0, &f0, &t0, [0.0, 5.0]);
    let d = donos(
        &r0,
        &f0,
        &t0,
        &[Receita {
            ancoras: &so_cima,
            semente: [0.0, 5.0],
        }],
    );
    assert_eq!(
        d.iter().filter(|x| x.is_none()).count(),
        1,
        "a de baixo fica vazia"
    );
    assert_eq!(d.iter().filter(|x| **x == Some(0)).count(), 1);
}

/// ⭐ **A SEMENTE é a rede de segurança, e só entra quando TODA âncora morreu** — o que acontece
/// quando o artista refaz as linhas (uma solda nova, um corte) e os contornos que elas nomeavam
/// deixam de existir.
///
/// ⚠️ E ela **não** manda quando há âncora: sem isso seriam duas portas para a mesma pergunta, e a
/// deriva que este modelo veio matar voltava pela segunda.
#[test]
fn the_seed_is_the_net_only_when_every_anchor_died() {
    let (r, f, t) = rede_de(&[quadrado(10.0), linha(0.0, 0.0)]);
    // Âncoras que nomeiam um caminho que não existe nesta rede.
    let mortas = vec![FillAnchor {
        path: 999,
        contorno: 0,
        frac: 0.5,
        frente: true,
    }];
    let d = donos(
        &r,
        &f,
        &t,
        &[Receita {
            ancoras: &mortas,
            semente: [0.0, -5.0],
        }],
    );
    let de_baixo = f
        .iter()
        .position(|g| miolo(&r, g)[1] < 0.0)
        .expect("ha' metade de baixo");
    assert_eq!(d[de_baixo], Some(0), "a semente salva o preenchimento");
    assert!(d[1 - de_baixo].is_none(), "e so' a face dela");
}

/// ⭐⭐ **UM PREENCHIMENTO PODE GANHAR VÁRIAS FACES, e a MAIOR vem à frente.**
///
/// ⚠️ A ordem é load-bearing: a primeira face vira o contorno **primário** do caminho e as outras os
/// `subpaths`.
#[test]
fn a_fill_can_win_several_faces_and_the_largest_leads() {
    let (_, faces, _) = rede_de(&[quadrado(10.0), linha(6.0, 6.0)]);
    let (grande, pequena) = if faces[0].area > faces[1].area {
        (0, 1)
    } else {
        (1, 0)
    };
    let out = por_preenchimento(&faces, &[Some(0), Some(0)], 1);
    assert_eq!(out[0], vec![grande, pequena], "a MAIOR vem a' frente");
}

/// ⛔ **Uma face sem dono não entra em lista nenhuma.**
#[test]
fn a_face_without_an_owner_reaches_no_fill() {
    let (_, faces, _) = rede_de(&[quadrado(10.0), linha(0.0, 0.0)]);
    let out = por_preenchimento(&faces, &[Some(0), None], 2);
    assert_eq!(out[0].len(), 1);
    assert!(out[1].is_empty(), "o segundo preenchimento nao ganhou nada");
}
