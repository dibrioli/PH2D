//! ⭐⭐⭐ **AS NOVE FORMAS DO LOTE DOS SÍMBOLOS (W120), PROVADAS ANTES DE SEREM LIGADAS.**
//!
//! A régua é a do lote anterior e a mais barata que existe: **pontos cuja resposta se sabe sem a
//! fórmula** — um vértice, uma tangência, o vazio de um entalhe, o miolo de um furo. ⚠️ Nenhum deles
//! vem de correr o código e escrever o que ele deu.
//!
//! ⚠️ **Ele não substitui o censo** (`the_census_of_every_primitive`), que pergunta se o campo ainda
//! é uma distância, se a caixa contém a peça, se toda linha do painel sabe ser escrita, e se o
//! filete alcança as arestas. Estes são os que dizem se a forma **é** o que o nome promete.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::Field;

fn campo(p: Primitive) -> Field {
    Field::new(
        &FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
            NodeId(0),
        )
        .expect("a peça"),
    )
}

const NA_PELE: f64 = 3.0e-3;

#[track_caller]
fn dentro(f: &Field, p: [f64; 3], porque: &str) {
    let v = f.at(p[0], p[1], p[2]);
    assert!(
        v < -NA_PELE,
        "{porque}: {p:?} devia estar DENTRO e leu {v:.5}"
    );
}

#[track_caller]
fn fora(f: &Field, p: [f64; 3], porque: &str) {
    let v = f.at(p[0], p[1], p[2]);
    assert!(v > NA_PELE, "{porque}: {p:?} devia estar FORA e leu {v:.5}");
}

#[track_caller]
fn na_pele(f: &Field, p: [f64; 3], porque: &str) {
    let v = f.at(p[0], p[1], p[2]);
    assert!(
        v.abs() < NA_PELE,
        "{porque}: {p:?} devia estar NA SUPERFÍCIE e leu {v:.5}"
    );
}

/// ⭐ **O balão retangular tem corpo E cauda, e a cauda sai da BASE.**
#[test]
fn the_speech_balloon_has_a_body_and_a_tail_below_it() {
    let f = campo(Primitive::SpeechRect {
        half_width: 0.42,
        half_span: 0.28,
        tail: 0.20,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    });
    dentro(&f, [0.0, 0.0, 0.0], "o meio do corpo");
    na_pele(&f, [0.42, 0.0, 0.0], "o lado direito");
    na_pele(&f, [0.0, 0.28, 0.0], "o topo");
    // ⭐ **A BICA** — o ponto que distingue um balão de um retângulo, e que nenhuma amostra dentro
    // do corpo vê.
    let tx = -0.42 * 0.35;
    na_pele(&f, [tx, -0.48, 0.0], "a bica da cauda");
    dentro(&f, [tx, -0.34, 0.0], "dentro da cauda");
    fora(&f, [tx, -0.52, 0.0], "abaixo da bica");
    // ⛔ Longe da cauda, abaixo da base é vazio — se não fosse, a cauda era uma saia.
    fora(&f, [0.35, -0.34, 0.0], "abaixo da base, longe da cauda");
}

/// ⭐ **O balão oval é REDONDO** — e é isso que o separa do retangular.
#[test]
fn the_oval_balloon_is_round_where_the_rectangular_one_has_corners() {
    let (w, s) = (0.44_f64, 0.26_f64);
    let f = campo(Primitive::SpeechOval {
        half_width: w as f32,
        half_span: s as f32,
        tail: 0.20,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    });
    dentro(&f, [0.0, 0.0, 0.0], "o meio");
    na_pele(&f, [w, 0.0, 0.0], "o extremo largo");
    na_pele(&f, [0.0, s, 0.0], "o extremo alto");
    // ⛔ **O CANTO da caixa está FORA** — num retângulo ele estaria dentro, e é a única pergunta
    // que separa as duas formas.
    fora(&f, [w * 0.85, s * 0.85, 0.0], "o canto da caixa é vazio");
    dentro(&f, [w * 0.5, s * 0.5, 0.0], "e o meio do quadrante é cheio");
}

/// ⭐⭐ **A nuvem tem BOSSAS, e o balão de pensamento é ela com a fieira.**
#[test]
fn the_cloud_has_lobes_and_the_thought_balloon_adds_a_trail() {
    let nuvem = |tail: f32| Primitive::Cloud {
        lobes: 5,
        half_width: 0.45,
        half_span: 0.22,
        tail,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    };
    let f = campo(nuvem(0.0));
    dentro(&f, [0.0, 0.0, 0.0], "o meio da nuvem");
    // ⛔ **O CANTO de cima é vazio** — uma nuvem não é um retângulo, e a bossa do meio é a mais
    // alta.
    fora(&f, [0.42, 0.20, 0.0], "o canto de cima é vazio");
    dentro(&f, [0.0, 0.16, 0.0], "a bossa do meio é a mais alta");
    // ⭐ **O VALE entre duas bossas ainda existe** — é ele que faz uma nuvem parecer uma nuvem, e a
    // mistura que alisa a junta com o corpo podia enchê-lo. A bossa vizinha do meio tem o topo mais
    // baixo, então acima dela é vazio.
    fora(&f, [0.0, 0.235, 0.0], "acima da bossa mais alta");
    // ⭐ **A fieira só existe com cauda** — é a única diferença entre as duas portas.
    let pensa = campo(nuvem(0.18));
    let b = [-0.45 * 0.63, -0.22 - 0.18 * 0.85, 0.0];
    dentro(&pensa, [b[0], b[1], 0.0], "a primeira bolha do pensamento");
    fora(
        &f,
        [b[0], b[1], 0.0],
        "a nuvem sem cauda não tem bolha nenhuma",
    );
}

/// ⭐ **O raio é um ZIGUE-ZAGUE** — e o que o prova são as duas pontas laterais, uma de cada lado.
#[test]
fn the_bolt_zigzags_with_a_tip_on_each_side() {
    let (w, h) = (0.28_f64, 0.45_f64);
    let f = campo(Primitive::Bolt {
        half_width: w as f32,
        half_span: h as f32,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    });
    na_pele(&f, [0.40 * w, h, 0.0], "o vértice de cima");
    na_pele(&f, [-0.40 * w, -h, 0.0], "o vértice de baixo");
    dentro(&f, [-0.75 * w, -0.09 * h, 0.0], "a ponta da esquerda");
    dentro(&f, [0.75 * w, 0.09 * h, 0.0], "a ponta da direita");
    // ⛔ **E o cruzamento é CHEIO** — se a decomposição fosse uma partição, aqui haveria uma
    // superfície fantasma.
    dentro(&f, [0.0, 0.0, 0.0], "a banda do meio");
    // ⛔ Os dois cantos que o zigue-zague deixa vazios.
    fora(&f, [0.75 * w, -0.6 * h, 0.0], "o canto de baixo à direita");
    fora(&f, [-0.75 * w, 0.6 * h, 0.0], "o canto de cima à esquerda");
}

/// ⭐ **O escudo é largo em cima e acaba numa PONTA em baixo.**
#[test]
fn the_shield_is_wide_on_top_and_ends_in_a_point() {
    let (w, s) = (0.34_f64, 0.44_f64);
    let f = campo(Primitive::Shield {
        half_width: w as f32,
        half_span: s as f32,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    });
    na_pele(&f, [w, s, 0.0], "o canto de cima à direita");
    na_pele(&f, [-w, s, 0.0], "o canto de cima à esquerda");
    na_pele(&f, [0.0, -s, 0.0], "a ponta de baixo");
    dentro(&f, [0.0, 0.0, 0.0], "o meio");
    fora(&f, [0.0, s + 0.05, 0.0], "acima do topo");
    // ⛔ **Os lados APERTAM** — num retângulo este ponto estaria dentro.
    fora(&f, [w * 0.95, -s * 0.6, 0.0], "o lado já apertou aqui");
}

/// ⭐ **A etiqueta afila numa PONTA e tem FURO.**
#[test]
fn the_tag_tapers_to_a_point_and_has_a_hole() {
    let (w, s, hole) = (0.45_f64, 0.26_f64, 0.07_f64);
    let f = campo(Primitive::Tag {
        half_width: w as f32,
        half_span: s as f32,
        point: 0.24,
        hole: hole as f32,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    });
    na_pele(&f, [w, 0.0, 0.0], "a ponta");
    fora(&f, [w + 0.03, 0.0, 0.0], "à frente da ponta");
    // ⛔ **Os dois cantos da direita foram CORTADOS** — é isso que a torna uma etiqueta.
    fora(&f, [w - 0.02, s * 0.9, 0.0], "o canto cortado de cima");
    dentro(&f, [0.0, 0.0, 0.0], "o corpo");
    // ⭐ **O FURO** — e o centro dele é vazio.
    let cx = -w * 0.7;
    fora(&f, [cx, 0.0, 0.0], "o miolo do furo");
    na_pele(&f, [cx + hole, 0.0, 0.0], "a boca do furo");
    dentro(&f, [cx + hole * 2.0, 0.0, 0.0], "material depois do furo");
}

/// ⭐ **O visto tem DOIS braços de comprimentos diferentes, e o vértice é em baixo.**
#[test]
fn the_check_has_two_arms_of_different_length() {
    let (w, s, t) = (0.42_f64, 0.30_f64, 0.11_f64);
    let f = campo(Primitive::Check {
        half_width: w as f32,
        half_span: s as f32,
        thickness: t as f32,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    });
    let v = [-0.25 * w, -s];
    dentro(&f, [v[0], v[1], 0.0], "o vértice de baixo");
    // ⚠️ **A ponta é o FIM da faixa, logo ela está NA PELE** — a 1.ª redacção pediu-a dentro e leu
    // `0,00000`, que é exactamente o que uma ponta quadrada devolve.
    na_pele(&f, [-w, 0.15 * s, 0.0], "a ponta do braço curto");
    na_pele(&f, [w, s, 0.0], "a ponta do braço longo");
    dentro(
        &f,
        [(v[0] - w) * 0.5, (v[1] + 0.15 * s) * 0.5, 0.0],
        "o meio do braço curto",
    );
    dentro(
        &f,
        [(v[0] + w) * 0.5, (v[1] + s) * 0.5, 0.0],
        "o meio do braço longo",
    );
    // ⛔ **O meio é VAZIO** — um visto é duas faixas, não um triângulo cheio.
    fora(&f, [0.0, 0.35 * s, 0.0], "entre os dois braços é vazio");
    fora(
        &f,
        [-0.8 * w, -0.8 * s, 0.0],
        "abaixo do braço curto é vazio",
    );
}

/// ⭐ **A faixa tem um ENTALHE em cada ponta, e ele é vazio.**
#[test]
fn the_banner_is_notched_at_both_ends() {
    let (w, s, notch) = (0.45_f64, 0.22_f64, 0.14_f64);
    let f = campo(Primitive::Banner {
        half_width: w as f32,
        half_span: s as f32,
        notch: notch as f32,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    });
    dentro(&f, [0.0, 0.0, 0.0], "o meio da fita");
    // ⭐ **O vértice do entalhe** — o ponto que uma fita tem e um retângulo não.
    na_pele(&f, [w - notch, 0.0, 0.0], "o vértice do entalhe direito");
    fora(&f, [w - notch * 0.4, 0.0, 0.0], "dentro do entalhe é vazio");
    na_pele(
        &f,
        [-(w - notch), 0.0, 0.0],
        "o vértice do entalhe esquerdo",
    );
    fora(
        &f,
        [-(w - notch * 0.4), 0.0, 0.0],
        "o entalhe esquerdo é vazio",
    );
    // ⭐ E os dois dentes de cada ponta ficam.
    //
    // ⚠️ **A aresta do entalhe passa EXACTAMENTE pelos cantos** `(±w, ±s)` — é o que faz a fita
    // parecer uma fita —, então junto à ponta o dente afila até nada. A 1.ª redacção deste gate
    // media a `0,9 s` **em `x = w`**, onde o dente já não tem espessura nenhuma.
    dentro(&f, [w * 0.9, s * 0.85, 0.0], "o dente de cima");
    dentro(&f, [w * 0.9, -s * 0.85, 0.0], "o dente de baixo");
}

/// ⭐ **A chave `{` tem quatro arcos, e o miolo dela é VAZIO.**
#[test]
fn the_brace_curves_four_times_and_is_hollow() {
    let (s, t) = (0.44_f64, 0.09_f64);
    let f = campo(Primitive::Brace {
        half_span: s as f32,
        thickness: t as f32,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    });
    let r = s * 0.5;
    // As quatro pontas e o nariz, cada uma no meio da espessura.
    dentro(&f, [2.0 * r, 2.0 * r, 0.0], "a ponta de cima");
    dentro(&f, [2.0 * r, -2.0 * r, 0.0], "a ponta de baixo");
    dentro(&f, [0.0, 0.0, 0.0], "o nariz do meio");
    dentro(&f, [r, r, 0.0], "a junta de cima");
    dentro(&f, [r, -r, 0.0], "a junta de baixo");
    // ⛔ **O interior das curvas é VAZIO** — a chave é uma fita, não uma chapa.
    fora(&f, [2.0 * r - t * 3.8, r, 0.0], "o vão da curva de cima");
    fora(&f, [t * 4.0, r * 0.5, 0.0], "o vão junto ao nariz");
}
