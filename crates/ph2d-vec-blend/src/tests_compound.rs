//! Os gates do **compound path** (a rosquinha) — arquivo irmão pelo teto de LOC.
//!
//! O que eles medem é o que o **olho** vê. E aqui o olho tem uma pergunta só, que nenhum contador
//! de contorno responde: **o buraco está lá?** — isto é, o centro da rosquinha está FORA da forma?
//!
//! O oráculo é [`ph2d_vec_scene::contains_point`], que aplica a [`FillRule`] do path exatamente
//! como o renderer. Ele não sabe nada sobre `subpaths` ser um `Vec` de tamanho 1: ele responde
//! sobre a **aparência**. Um gate que contasse contornos ficaria VERDE com um buraco de área zero,
//! ou com um buraco pendurado FORA da forma, ou com a regra de preenchimento errada — todas
//! rosquinhas que o artista veria como disco. [[reference_topic_oracle_discipline]]

use super::*;
use ph2d_vec_scene::{Contour, FillRule, ShapeKind, contains_point, cook};

/// Uma forma do catálogo, centrada em `c`.
fn shape(kind: ShapeKind, c: [f64; 2], half: [f64; 2]) -> VecPath {
    cook(
        kind,
        [c[0] - half[0], c[1] - half[1]],
        [c[0] + half[0], c[1] + half[1]],
        &[],
    )
}

fn circle(c: [f64; 2], r: f64) -> VecPath {
    shape(ShapeKind::Ellipse, c, [r, r])
}

/// Uma **rosquinha**: disco externo de raio `r_out`, buraco de raio `r_in`, centrados em `c`.
///
/// Montada como a booleana monta (`compound_from`): o contorno de fora é o primário, o de dentro é
/// um subpath, e a regra é **EvenOdd** — que é o que faz o aninhado virar buraco independente de
/// como foi orientado.
fn donut(c: [f64; 2], r_out: f64, r_in: f64) -> VecPath {
    let outer = circle(c, r_out);
    let inner = circle(c, r_in);
    VecPath {
        verts: outer.verts,
        closed: true,
        subpaths: vec![Contour::new_closed(inner.verts)],
        fill_rule: FillRule::EvenOdd,
        ..VecPath::default()
    }
}

/// **O BURACO SOBREVIVE AO MORPH.**
///
/// Duas rosquinhas; no meio do caminho tem de haver uma rosquinha. Hoje sai um DISCO: `Outline::of`
/// lê só `cooked.verts` (o contorno de fora) e `path_from` reconstrói com `..VecPath::default()`
/// (`subpaths` vazio, `fill_rule` de volta a `NonZero`) — os dois lados perdem o buraco, cada um
/// por conta própria.
///
/// A rosquinha é a saída TÍPICA da booleana e do Shape Builder: este não é um caso exótico.
#[test]
fn the_hole_survives_the_morph() {
    let a = donut([0.0, 0.0], 2.0, 1.0);
    let b = donut([6.0, 0.0], 2.0, 1.0);

    // As pontas são rosquinhas — se isto falhar, o fixture é que está errado.
    assert!(
        !contains_point(&a, [0.0, 0.0]),
        "o fixture A não tem buraco"
    );
    assert!(
        !contains_point(&b, [6.0, 0.0]),
        "o fixture B não tem buraco"
    );

    let mid = morph(&a, &b, 0.5).expect("duas rosquinhas válidas têm morph");

    // No meio: o anel está lá (um ponto na parede é DENTRO)...
    assert!(
        contains_point(&mid, [3.0 + 1.5, 0.0]),
        "a parede do anel sumiu no meio do caminho"
    );
    // ...e o buraco também (o centro é FORA).
    assert!(
        !contains_point(&mid, [3.0, 0.0]),
        "O BURACO SUMIU: a rosquinha virou disco no meio do caminho"
    );
}

/// **As pontas de um morph de compound são as formas originais.**
///
/// O gate irmão de `the_ends_of_the_morph_are_the_shapes_themselves`, para quem tem buraco: se
/// `t=0` já não devolve a rosquinha A, nada no meio importa.
#[test]
fn the_ends_of_a_compound_morph_are_the_shapes_themselves() {
    let a = donut([0.0, 0.0], 2.0, 1.0);
    let b = donut([6.0, 0.0], 2.0, 0.5);

    let at0 = morph(&a, &b, 0.0).expect("morph válido");
    let at1 = morph(&a, &b, 1.0).expect("morph válido");

    for (t, path, c) in [(0.0, &at0, [0.0, 0.0]), (1.0, &at1, [6.0, 0.0])] {
        assert!(
            !contains_point(path, c),
            "t={t}: a ponta perdeu o buraco (devia ser a forma original)"
        );
        assert_eq!(
            path.contour_count(),
            2,
            "t={t}: a ponta devia ter os 2 contornos da forma original"
        );
    }
}

/// **O CONTORNO DE FORA NUNCA CASA COM O BURACO — nem quando a distância MANDA que case.**
///
/// O papel de um contorno (a profundidade de aninhamento) vem ANTES da distância. Este gate arma o
/// conflito: o buraco de B é **descentrado**, de modo que o centroide do buraco de B fica em cima
/// do centroide do contorno de FORA de A (distância 0), enquanto o par certo — de fora com de fora
/// — custa 1,0. Pelo critério de viagem sozinho, o pareamento errado **ganha**.
///
/// # A 1ª versão deste gate era inútil, e a lição vale o parágrafo
///
/// Ela usava duas rosquinhas CONCÊNTRICAS e idênticas, apostando que as quatro viagens seriam zero
/// e que o desempate cairia do lado errado. Não caíam: contornos idênticos dão centroides
/// **exatamente** iguais (viagem `0.0`), e contornos diferentes dão `1e-16` de ruído de arredondamento
/// — então a distância, por acidente de `f64`, já fazia o trabalho do filtro de profundidade. O
/// gate ficava verde COM e SEM o filtro: [[reference_topic_fixture_discipline]] — um fixture só
/// prova o que ele contém, e aquele não continha conflito nenhum.
#[test]
fn the_outer_contour_never_marries_a_hole() {
    // A: rosquinha concêntrica em [0,0].
    let a = donut([0.0, 0.0], 3.0, 1.5);
    // B: o MESMO buraco em [0,0], mas o contorno de fora deslocado para [1,0] — a rosquinha de
    // parede grossa de um lado. (1 + 1,5 < 3: o buraco está estritamente dentro, sem tangenciar.)
    let b = {
        let outer = circle([1.0, 0.0], 3.0);
        let inner = circle([0.0, 0.0], 1.5);
        VecPath {
            verts: outer.verts,
            closed: true,
            subpaths: vec![Contour::new_closed(inner.verts)],
            fill_rule: FillRule::EvenOdd,
            ..VecPath::default()
        }
    };
    for (name, p, c) in [("a", &a, [0.0, 0.0]), ("b", &b, [0.0, 0.0])] {
        assert!(!contains_point(p, c), "{name}: o fixture não tem buraco");
    }

    let mid = morph(&a, &b, 0.5).expect("morph válido");
    // O par certo dá a rosquinha do meio: de fora r=3 em [0,5, 0], buraco r=1,5 em [0,0]. O par
    // errado dá dois círculos de r=2,25 (em [0,0] e [0,5, 0]) — e nenhum deles alcança x=3.
    for x in [3.0, 3.4] {
        assert!(
            contains_point(&mid, [x, 0.0]),
            "a parede não chega em x={x}: o contorno de fora casou com o BURACO (a distância \
             preferia esse par) e a forma encolheu para o tamanho do buraco"
        );
    }
    assert!(!contains_point(&mid, [0.0, 0.0]), "o buraco sumiu");
}

/// **UMA ROSQUINHA VIRANDO DISCO FECHA O BURACO — dentro da forma.**
///
/// A mudança de TOPOLOGIA: B não tem buraco para oferecer, então o de A não tem par e **colapsa
/// num ponto**. A pergunta que este gate faz não é "o buraco fecha?" (fechar é fácil: bastaria
/// deixá-lo cair) — é **ONDE** ele fecha.
///
/// O ponto de colapso é o centroide do lado OPOSTO, e é isso que o gate mede: em `t=0,5` o buraco
/// já encolheu mas ainda está **dentro da parede**, viajando junto com a forma. Colapsando-o no
/// centroide dele mesmo, ele ficaria parado em `x=0` enquanto a forma anda para `x=3` — e o
/// "buraco" apareceria fora da própria forma.
#[test]
fn a_donut_becoming_a_disc_closes_the_hole_inside_itself() {
    let a = donut([0.0, 0.0], 2.0, 1.0);
    let b = circle([6.0, 0.0], 2.0);

    let mid = morph(&a, &b, 0.5).expect("morph válido");
    // O meio do caminho é uma rosquinha a meia altura: o de fora em [3,0] r≈2, o buraco em [3,0].
    assert!(
        contains_point(&mid, [3.0, 1.5]),
        "a parede sumiu no meio do caminho"
    );
    assert!(
        !contains_point(&mid, [3.0, 0.0]),
        "o buraco não está no meio da forma: ou foi DESCARTADO, ou colapsou no lugar errado \
         (ficou para trás em vez de viajar com a forma)"
    );

    // E na ponta ele fechou de vez: o disco é SÓLIDO.
    let at1 = morph(&a, &b, 1.0).expect("morph válido");
    assert!(
        contains_point(&at1, [6.0, 0.2]),
        "t=1: o disco devia ser sólido — o buraco não fechou"
    );
}

/// **A ROSQUINHA DE VERDADE — a que sai da booleana.**
///
/// Todos os gates acima montam o compound à mão, e um fixture só prova o que ele contém: o meu
/// contém a MINHA ideia de rosquinha (dois círculos do catálogo, o de fora primeiro, contagens de
/// vértice iguais). A do artista sai de um **Subtract** — e é ela que tem a ordem de contornos, a
/// contagem de vértices e a regra de preenchimento que a booleana de fato escolhe.
/// [[reference_topic_fixture_discipline]]
///
/// É o mesmo caminho do Shape Builder. Se este gate cair, o blend está errado no caso que o
/// handoff chamou de "a saída típica da booleana" — que é o motivo desta wave existir.
#[test]
fn the_donut_the_boolean_actually_makes_blends_with_its_hole() {
    let cut = |c: [f64; 2]| {
        let (outer, inner) = (circle(c, 2.0), circle(c, 1.0));
        let mut made =
            ph2d_vec_boolean::apply_many(&[&outer, &inner], ph2d_vec_boolean::BoolOp::Subtract);
        assert_eq!(
            made.len(),
            1,
            "um Subtract concêntrico devia dar UM compound"
        );
        made.remove(0)
    };
    let (a, b) = (cut([0.0, 0.0]), cut([6.0, 0.0]));

    // A booleana produziu mesmo uma rosquinha? (Se não, o gate abaixo não prova nada.)
    for (name, p, c) in [("a", &a, [0.0, 0.0]), ("b", &b, [6.0, 0.0])] {
        assert!(
            p.is_compound(),
            "{name}: a booleana não devolveu um compound"
        );
        assert!(
            !contains_point(p, c),
            "{name}: a booleana não deixou buraco"
        );
    }

    let mid = morph(&a, &b, 0.5).expect("duas rosquinhas válidas têm morph");
    assert!(contains_point(&mid, [3.0, 1.5]), "a parede sumiu no meio");
    assert!(
        !contains_point(&mid, [3.0, 0.0]),
        "O BURACO SUMIU numa rosquinha vinda da BOOLEANA — o caso real"
    );
}

/// **O PAREAMENTO DE BURACOS IRMÃOS É O DE MENOR CUSTO TOTAL, não o guloso.**
///
/// O erro clássico do guloso: a melhor escolha LOCAL do primeiro força uma péssima para o segundo.
/// Aqui, dois buracos de A (em x=0 e x=6) e dois de B (em x=5 e x=12). O guloso, na ordem do
/// documento, dá ao 1º buraco o seu vizinho mais próximo (x=5, custo 25) e sobra x=12 para o 2º
/// (custo 36): total **61**. O ótimo é 0→5 e 6→12 … que é o MESMO. Então o fixture inverte a ordem
/// dos buracos de A (x=6 primeiro): aí o guloso dá x=5 ao de x=6 (custo 1) e força o de x=0 a
/// atravessar até x=12 (custo 144) — total **145**, contra o ótimo **61** (0→5, 6→12).
///
/// O que o olho vê: no pareamento guloso um buraco **cruza a forma inteira** e passa por cima do
/// outro; no ótimo cada um vai para o vizinho dele. O gate mede pelo lugar onde os buracos estão
/// no meio do caminho.
#[test]
fn sibling_holes_take_the_cheapest_pairing_overall_not_the_greedy_one() {
    // Uma placa larga com dois buracos. A ordem dos buracos no documento é adversária de propósito.
    let plate = |holes: &[[f64; 2]]| VecPath {
        verts: shape(ShapeKind::Rectangle, [6.0, 0.0], [10.0, 3.0]).verts,
        closed: true,
        subpaths: holes
            .iter()
            .map(|&c| Contour::new_closed(circle(c, 0.8).verts))
            .collect(),
        fill_rule: FillRule::EvenOdd,
        ..VecPath::default()
    };
    let a = plate(&[[6.0, 0.0], [0.0, 0.0]]); // o de x=6 PRIMEIRO — a isca do guloso
    let b = plate(&[[5.0, 0.0], [12.0, 0.0]]);

    let mid = morph(&a, &b, 0.5).expect("morph válido");
    // Ótimo (0→5, 6→12): os buracos do meio ficam em x=2,5 e x=9.
    // Guloso (6→5, 0→12): ficam em x=5,5 e x=6 — colados, e um deles atravessou a placa toda.
    for x in [2.5, 9.0] {
        assert!(
            !contains_point(&mid, [x, 0.0]),
            "não há buraco em x={x}: o pareamento não foi o de menor custo total — um buraco \
             atravessou a placa em vez de ir para o vizinho dele"
        );
    }
}

/// **A regra de preenchimento viaja com a forma.**
///
/// `path_from` reconstrói o passo com `..VecPath::default()`, e o default de [`FillRule`] é
/// `NonZero` — então mesmo um passo que carregasse os dois contornos renderizaria SÓLIDO se os
/// contornos tiverem o mesmo winding (que é o caso: os dois círculos do catálogo nascem no mesmo
/// sentido). O buraco não é feito de contornos: é feito de contornos **mais a regra**.
#[test]
fn the_fill_rule_travels_with_the_shape() {
    let a = donut([0.0, 0.0], 2.0, 1.0);
    let b = donut([6.0, 0.0], 2.0, 1.0);
    let mid = morph(&a, &b, 0.5).expect("morph válido");
    assert_eq!(
        mid.fill_rule,
        FillRule::EvenOdd,
        "o passo esqueceu a regra de preenchimento e voltou ao default NonZero"
    );
}
