//! Os gates da cena `=65` — o campo que segue o rato.
//!
//! ⚠️ **Um quadro parado não prova esta cena**, e é por isso que o gate publica o cursor em
//! DOIS sítios e compara as duas leituras. Com o rato na origem as duas bandas coincidem de
//! propósito; o que a cena afirma é que uma delas **anda** e a outra não.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Coze as duas bandas com o cursor publicado em `at`, e devolve o `size` de cada uma.
///
/// ⚠️ O `Cook` é criado UMA vez por chamada de propósito: cada leitura é um mundo próprio, e o
/// gate do memo (que o cursor invalida) vive na crate do nó, onde ele pertence.
fn sizes_with_cursor(at: [f32; 2]) -> Vec<Vec<f32>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_cursor_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 2, "duas bandas");
    doc.graph.validate(&reg).expect("bem-tipado");
    let mut cook = Cook::new();
    cook.set_external(
        ph2d_nodegraph::external::CURSOR,
        Stream::new(1).with("P", Column::Vec2(vec![at])),
    );
    sinks
        .iter()
        .map(|s| {
            match cook.cook(&doc.graph, &reg, *s, 0.0).expect("coze")[0]
                .as_stream()
                .get("size")
            {
                Some(Column::Vec2(v)) => v.iter().map(|q| q[0]).collect(),
                _ => Vec::new(),
            }
        })
        .collect()
}

/// **A BANDA 1 SEGUE O RATO E A BANDA 2 NÃO** — as duas metades, medidas no mesmo par de
/// leituras.
///
/// ⚠️ Sem a segunda metade, um gate que só visse a banda 1 mudar não distinguiria *"o cursor
/// chegou ao centro do campo"* de *"o cursor mexeu em alguma coisa qualquer no cozimento"*.
#[test]
fn only_the_driven_band_moves_when_the_cursor_does() {
    let a = sizes_with_cursor([-3.0, 0.0]);
    let b = sizes_with_cursor([3.0, 0.0]);
    let n = (SIDE * SIDE) as usize;
    for (i, band) in a.iter().enumerate() {
        assert_eq!(band.len(), n, "a banda {i} tem {n} peças");
    }
    assert_ne!(
        a[0], b[0],
        "a banda dirigida tem de mudar quando o cursor anda"
    );
    assert_eq!(
        a[1], b[1],
        "a banda de controle tem o centro AUTORADO — ela não pode mexer-se"
    );
}

/// **O CAMPO ESTÁ ONDE O RATO ESTÁ** — a peça maior é a mais próxima do cursor.
///
/// ⚠️ É a metade que separa *"alguma coisa mudou"* de *"mudou no sítio certo"*: um erro de
/// sinal no `y`, ou as duas saídas trocadas, passaria pelo gate acima sem tremer.
#[test]
fn the_biggest_piece_is_the_one_nearest_the_cursor() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_cursor_demo_document(&mut doc, &reg).expect("a cena monta");
    for at in [[2.0f32, 2.0], [-2.0, 1.5], [0.0, -2.5]] {
        let mut cook = Cook::new();
        cook.set_external(
            ph2d_nodegraph::external::CURSOR,
            Stream::new(1).with("P", Column::Vec2(vec![at])),
        );
        let st = cook.cook(&doc.graph, &reg, sinks[0], 0.0).expect("coze")[0].as_stream();
        let (Some(Column::Vec2(p)), Some(Column::Vec2(size))) = (st.get("P"), st.get("size"))
        else {
            panic!("a banda tem de trazer P e size")
        };
        // A peça maior…
        let biggest = size
            .iter()
            .enumerate()
            .max_by(|a, b| a.1[0].total_cmp(&b.1[0]))
            .expect("há peças")
            .0;
        // …e a mais próxima do cursor, **sem nenhuma correção de quadro**: é isso que este
        // gate existe para afirmar. A primeira versão da cena punha o `motion.move` DEPOIS do
        // campo, e o inchaço nascia deslocado do ponteiro pelo tamanho do deslocamento da
        // banda — o gate reprovou com a peça errada, e a cura foi na cena, não aqui.
        let d2 = |q: &[f32; 2]| (q[0] - at[0]).powi(2) + (q[1] - at[1]).powi(2);
        let nearest = p
            .iter()
            .enumerate()
            .min_by(|a, b| d2(a.1).total_cmp(&d2(b.1)))
            .expect("há peças")
            .0;
        assert_eq!(
            biggest, nearest,
            "cursor em {at:?}: a peça maior tem de ser a mais próxima dele"
        );
    }
}

/// **NEM NO PICO DO CAMPO UMA PEÇA TAPA A VIZINHA** — a lei da cena `=63`, aqui contra o
/// tamanho MÁXIMO que o campo produz.
///
/// ⚠️ Nesta cena a peça é elástica, então a régua não é o tamanho em repouso: é
/// `PIECE · GROWTH`, que é o que ela mede no centro do campo. Um gate que medisse o repouso
/// passaria e o smoke mostraria um borrão exactamente onde o olho vai.
#[test]
fn even_at_the_peak_of_the_field_no_piece_hides_its_neighbour() {
    for band in sizes_with_cursor([0.0, 0.0]) {
        let widest = band.iter().fold(0.0f32, |m, s| m.max(*s));
        assert!(
            widest <= PITCH,
            "a peça chega a {widest:.3} contra um passo de {PITCH}"
        );
        // E o controle: ela de facto CRESCEU (senão o teto é respeitado por o campo estar
        // morto).
        assert!(
            widest > PIECE * 1.5,
            "o campo tem de inchar a peça: {PIECE} -> {widest:.3}"
        );
    }
}
