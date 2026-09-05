//! ⭐⭐⭐ **AS SEIS FORMAS DO LOTE DA SETA (W119), PROVADAS ANTES DE SEREM LIGADAS.**
//!
//! # ⚠️ Por que este arquivo existe, e o que ele NÃO substitui
//!
//! Ligar uma primitiva custa ~14 sítios (o enum · o `kind` · a `ALL` · as dimensões do painel · a
//! escrita delas · o limite do filete · o tamanho característico · o raio delimitador · as
//! meias-extensões · a escala · a validação · o despacho · o rótulo · a linha do catálogo · o nome
//! na Hierarquia). ⚠️ **Nenhum deles diz se a FÓRMULA está certa** — e a W106 pagou esta lição com
//! três defeitos que só um ponto conhecido apanhou (um sinal trocado, um cone virado ao contrário,
//! um cone infinito).
//!
//! ⭐ A régua aqui é a mais barata que existe: **pontos cuja resposta se sabe sem a fórmula** — um
//! vértice, uma tangência, o meio de uma face, o vazio de uma banda. Nenhum deles vem de correr o
//! código e escrever o que ele deu.
//!
//! ⚠️ O censo (`the_census_of_every_primitive`) e a sonda de arestas (`measure_sharp_edges`)
//! respondem às **outras** perguntas — se o campo ainda é uma distância, se a caixa contém a peça,
//! se o filete deixa peça. As seis entram lá derivadas de [`PrimitiveKind::ALL`].

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

/// A folga de uma amostra «na superfície». ⚠️ Ela é do **instrumento**: um ponto escrito com quatro
/// casas não cai exactamente na superfície de nada.
const NA_PELE: f64 = 2.0e-3;

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

fn uma_seta(heads: u32) -> Primitive {
    Primitive::Arrow {
        heads,
        half_length: 0.45,
        shaft: 0.10,
        head: 0.22,
        head_length: 0.25,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    }
}

/// ⭐ **A seta aponta, tem farpa, e a haste não espreita por fora da ponta.**
///
/// ⚠️ **A farpa é o ponto que uma sonda de silhueta no eixo NÃO vê** — foi assim que a gota da W106
/// escondeu um cone infinito. Aqui ela é medida onde ela está: em `x = tip − head_length`.
#[test]
fn the_arrow_has_a_tip_a_barb_and_a_shaft_that_never_pokes_out() {
    let f = campo(uma_seta(1));
    na_pele(&f, [0.45, 0.0, 0.0], "o bico");
    dentro(&f, [0.40, 0.0, 0.0], "atrás do bico");
    fora(&f, [0.47, 0.0, 0.0], "à frente do bico");
    na_pele(&f, [-0.45, 0.0, 0.0], "a cauda");
    dentro(&f, [-0.40, 0.08, 0.0], "dentro da haste");
    // ⚠️ **Acima da haste e ATRÁS da ponta é vazio** — é isto que distingue uma seta de um triângulo
    // com um rabo, e nenhuma amostra no eixo o vê.
    fora(&f, [-0.40, 0.16, 0.0], "acima da haste");
    dentro(&f, [0.21, 0.19, 0.0], "dentro da farpa");
    fora(&f, [0.21, 0.24, 0.0], "fora da farpa");
    // ⭐ **A LEI DA SOBREPOSIÇÃO, medida:** a haste entra na ponta e as quinas dela pousam sobre o
    // flanco. Em `x` logo à frente de onde a ponta vale `shaft`, o material tem de ser da PONTA — se
    // a haste espreitasse, este ponto lia dentro.
    let frente = 0.45 - 0.25 * (0.10 / 0.22);
    fora(&f, [frente + 0.02, 0.101, 0.0], "a haste não espreita");
    fora(&f, [0.0, 0.0, 0.12], "a chapa acaba em Z");
}

/// ⭐⭐ **A seta DUPLA é a mesma forma dobrada por `|x|`** — as duas pontas, e nenhuma cauda.
#[test]
fn the_double_arrow_has_two_tips_and_no_tail() {
    let f = campo(uma_seta(2));
    na_pele(&f, [0.45, 0.0, 0.0], "o bico da direita");
    na_pele(&f, [-0.45, 0.0, 0.0], "o bico da esquerda");
    dentro(&f, [-0.25, 0.15, 0.0], "dentro da farpa da esquerda");
    dentro(&f, [0.25, 0.15, 0.0], "dentro da farpa da direita");
    // ⛔ Uma seta simples lia este ponto FORA (a cauda é um rectângulo de meia-largura `shaft`).
    let simples = campo(uma_seta(1));
    fora(
        &simples,
        [-0.25, 0.15, 0.0],
        "a seta simples não tem farpa atrás",
    );
}

fn um_chevron() -> Primitive {
    Primitive::Chevron {
        half_length: 0.40,
        half_span: 0.30,
        thickness: 0.08,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    }
}

/// ⭐⭐ **O chevron é uma BANDA, e o miolo dele é VAZIO** — é isso que o separa de uma seta sem
/// haste, e é a única pergunta que distingue as duas formas.
#[test]
fn the_chevron_is_a_band_and_its_middle_is_empty() {
    let f = campo(um_chevron());
    na_pele(&f, [0.40, 0.0, 0.0], "o bico exterior");
    dentro(&f, [0.35, 0.0, 0.0], "dentro da banda, atrás do bico");
    // ⭐ **O MIOLO** — se ele lesse dentro, isto era uma cunha cheia e não um chevron.
    fora(&f, [0.0, 0.0, 0.0], "o miolo é vazio");
    fora(&f, [-0.20, 0.0, 0.0], "o miolo continua vazio mais atrás");
    dentro(&f, [-0.30, 0.24, 0.0], "dentro do braço de cima");
    dentro(&f, [-0.30, -0.24, 0.0], "dentro do braço de baixo");
    fora(&f, [-0.30, 0.33, 0.0], "fora do braço de cima");
    na_pele(&f, [-0.40, 0.28, 0.0], "a ponta do braço, na face de trás");
    fora(&f, [-0.45, 0.28, 0.0], "atrás da face de trás");
}

fn uma_seta_dobrada() -> Primitive {
    Primitive::BentArrow {
        run: 0.40,
        rise: 0.40,
        shaft: 0.08,
        head: 0.18,
        head_length: 0.22,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    }
}

/// ⭐ **A seta dobrada tem os DOIS braços, o cotovelo cheio e o canto de dentro vazio.**
#[test]
fn the_bent_arrow_turns_a_corner_and_leaves_the_inside_empty() {
    let f = campo(uma_seta_dobrada());
    dentro(&f, [-0.35, -0.32, 0.0], "dentro do braço deitado");
    dentro(&f, [0.32, -0.32, 0.0], "dentro do cotovelo");
    dentro(&f, [0.32, 0.0, 0.0], "dentro do braço de pé");
    na_pele(&f, [0.32, 0.40, 0.0], "o bico");
    fora(&f, [0.32, 0.43, 0.0], "à frente do bico");
    dentro(&f, [0.32, 0.36, 0.0], "atrás do bico");
    // ⭐ **O CANTO DE DENTRO do «L»** — se ele lesse dentro, a peça era um quadrado.
    fora(&f, [0.0, 0.0, 0.0], "o canto de dentro é vazio");
    fora(&f, [-0.35, 0.20, 0.0], "acima do braço deitado é vazio");
    // ⚠️ **A farpa está na BASE da ponta** (`y = rise − head_length = 0,18`), onde ela abre `head`
    // para cada lado do braço de pé — e não a meio da ponta, onde já afunilou. *A 1.ª redacção
    // deste gate mediu a `y = 0,30`, onde a meia-largura é `0,082` e o ponto está mesmo fora.*
    dentro(&f, [0.16, 0.19, 0.0], "dentro da farpa da esquerda");
    fora(&f, [0.10, 0.19, 0.0], "fora da farpa");
    dentro(&f, [0.46, 0.19, 0.0], "dentro da farpa da direita");
}

/// ⭐ **O losango tem as DUAS diagonais**, e é isso que o separa do prisma de quatro lados.
///
/// ⚠️ **O ponto mais fundo dele é o raio INSCRITO** (`a·b/√(a²+b²)`), e não a menor meia-diagonal —
/// é a mesma conta que o [`ph2d_field::round_limit`] dele usa, e este gate mede-a pelo campo.
#[test]
fn the_rhombus_has_two_different_diagonals_and_an_inradius() {
    let (a, b) = (0.40_f64, 0.25_f64);
    let f = campo(Primitive::Rhombus {
        half_width: a as f32,
        half_span: b as f32,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    });
    na_pele(&f, [a, 0.0, 0.0], "o vértice largo");
    na_pele(&f, [0.0, b, 0.0], "o vértice alto");
    na_pele(&f, [a * 0.5, b * 0.5, 0.0], "o meio de uma aresta");
    // ⛔ O canto da caixa envolvente está FORA — um prisma de 4 lados com o mesmo circunraio
    // continha-o.
    fora(&f, [a * 0.8, b * 0.8, 0.0], "o canto da caixa é vazio");
    // ⭐⭐ **O raio inscrito mede-se numa chapa MAIS GROSSA que ele**, senão o que a sonda lê no
    // centro é a TAMPA: a `half_height = 0,10` a resposta é `−0,10`, e ela está certa — a face mais
    // próxima do centro é a de cima. *Uma régua posta no sítio errado mede a peça errada.*
    let inscrito = a * b / (a * a + b * b).sqrt();
    let grossa = campo(Primitive::Rhombus {
        half_width: a as f32,
        half_span: b as f32,
        half_height: 0.35,
        round: 0.0,
        chamfer: 0.0,
    });
    let centro = grossa.at(0.0, 0.0, 0.0);
    assert!(
        (centro + inscrito).abs() < NA_PELE,
        "o centro devia ler −{inscrito:.5} (o raio inscrito) e leu {centro:.5}"
    );
}

/// ⭐⭐ **O tubo tem FURO, e em `π` o anel FECHA** — as duas perguntas que a `sd_pie` não responde.
///
/// ⚠️⚠️ **A segunda é a que tem armadilha:** em `π` os dois semiplanos do sector são opostos e a
/// união deles vale `−|x|`, que é **zero sobre todo o eixo** — uma fenda fantasma a partir o anel ao
/// meio. Por isso o sector **sai da árvore** em vez de degenerar, e é isso que este gate mede.
#[test]
fn the_tube_has_a_hole_and_closes_into_a_ring_at_pi() {
    let f = campo(Primitive::Tube {
        outer: 0.40,
        inner: 0.25,
        angle: std::f32::consts::PI,
        half_height: 0.12,
        round: 0.0,
        chamfer: 0.0,
    });
    na_pele(&f, [0.40, 0.0, 0.0], "o bordo de fora");
    na_pele(&f, [0.25, 0.0, 0.0], "o bordo do furo");
    dentro(&f, [0.325, 0.0, 0.0], "a meia-parede");
    fora(&f, [0.0, 0.0, 0.0], "o furo é vazio");
    fora(&f, [0.45, 0.0, 0.0], "fora do bordo");
    // ⭐⭐ **O ANEL FECHA:** os quatro quadrantes têm de ler o mesmo, e o eixo `x = 0` (onde a fenda
    // fantasma nasceria) é material como qualquer outro sítio da parede.
    for (x, y) in [(0.325, 0.0), (-0.325, 0.0), (0.0, 0.325), (0.0, -0.325)] {
        dentro(&f, [x, y, 0.0], "a parede, à volta toda");
    }
}

/// ⭐ **O arco de anel é o mesmo tubo com o sector**, e a bissectriz dele é `+Y`.
#[test]
fn the_ring_arc_keeps_the_hole_and_cuts_one_side() {
    let f = campo(Primitive::Tube {
        outer: 0.40,
        inner: 0.25,
        angle: 0.8,
        half_height: 0.12,
        round: 0.0,
        chamfer: 0.0,
    });
    dentro(&f, [0.0, 0.325, 0.0], "dentro do sector");
    fora(&f, [0.0, -0.325, 0.0], "do lado que o sector corta");
    fora(&f, [0.0, 0.0, 0.0], "o furo continua vazio");
}

/// ⭐ **O segmento de círculo é cortado por uma CORDA**, e não por dois raios nem por um disco.
///
/// ⚠️ **A corda negativa é o caso que a porta de escrita recusava até esta wave** — ver a guarda do
/// [`ph2d_field::Span::Free`] em `dims_write.rs`.
#[test]
fn the_circle_segment_is_cut_by_a_chord() {
    let f = campo(Primitive::CircleSegment {
        radius: 0.40,
        cut: 0.10,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    });
    na_pele(&f, [0.0, 0.40, 0.0], "o topo do arco");
    na_pele(&f, [0.0, 0.10, 0.0], "a corda");
    dentro(&f, [0.0, 0.25, 0.0], "entre a corda e o arco");
    fora(&f, [0.0, 0.0, 0.0], "abaixo da corda");
    // ⛔ Uma FATIA (`Pie`) conteria este ponto: ela converge para a origem, o segmento não.
    fora(&f, [0.0, 0.05, 0.0], "logo abaixo da corda");
    // As duas quinas onde a corda encontra o arco.
    let meia_corda = (0.40_f64 * 0.40 - 0.10 * 0.10).sqrt();
    na_pele(&f, [meia_corda, 0.10, 0.0], "a quina da direita");
    na_pele(&f, [-meia_corda, 0.10, 0.0], "a quina da esquerda");
    // ⭐ Com a corda NEGATIVA sobra mais de meio disco.
    let baixo = campo(Primitive::CircleSegment {
        radius: 0.40,
        cut: -0.20,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    });
    dentro(
        &baixo,
        [0.0, -0.10, 0.0],
        "abaixo do centro, com a corda negativa",
    );
    fora(&baixo, [0.0, -0.30, 0.0], "abaixo da corda negativa");
}
