//! Os gates da cena `=63` — a ordem.
//!
//! ⚠️ **A PERMUTAÇÃO não basta como oráculo, e esta cena pagou por isso.** A primeira versão
//! media só a ordem das POSIÇÕES à saída — e passou verde sobre uma cena em que as três
//! bandas saíam com a MESMA pintura, porque o `motion.tint` em gradiente lê a coluna de
//! identidade `Index`, que o `motion.sort` levava consigo. O Enio viu no smoke o que o gate
//! não via: *"a cor não corre da esquerda para a direita, mas de baixo para cima"* — de baixo
//! para cima é a ordem em que a GRELHA nasce, isto é, a ordenação não chegava ao pixel.
//! O que se afirma agora é o que o olho lê: **a COR é a ordem**.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// As posições de cada banda, na ordem em que elas saem.
fn bands() -> Vec<Vec<[f32; 2]>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_sortkey_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 3, "três bandas");
    doc.graph.validate(&reg).expect("bem-tipado");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|s| {
            match cook.cook(&doc.graph, &reg, *s, 0.0).expect("a banda coze")[0]
                .as_stream()
                .get("P")
            {
                Some(Column::Vec2(v)) => v.clone(),
                _ => Vec::new(),
            }
        })
        .collect()
}

/// Uma banda pelas duas colunas que o olho junta: a posição e a cor.
type Painted = Vec<(Vec<[f32; 2]>, Vec<[f32; 4]>)>;

/// As posições **e a tinta** de cada banda, na ordem em que elas saem.
fn painted() -> Painted {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_sortkey_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipado");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|s| {
            let v = cook.cook(&doc.graph, &reg, *s, 0.0).expect("a banda coze");
            let st = v[0].as_stream();
            let p = match st.get("P") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => Vec::new(),
            };
            let t = match st.get("tint") {
                Some(Column::Vec4(v)) => v.clone(),
                _ => Vec::new(),
            };
            (p, t)
        })
        .collect()
}

/// **A COR É A ORDEM** — em cada banda o degradê corre ao longo da ordem de saída, sem voltar
/// atrás.
///
/// ⚠️ Este é o gate que faltava. Uma banda cuja cor não é monótona na ordem de saída está a
/// pintar por OUTRA coisa (a identidade que veio de montante), e a cena mente sobre o nó
/// inteiro. O canal medido é o vermelho, que a cena manda de `0.1` a `1.0`.
#[test]
fn the_colour_runs_along_the_sorted_order_in_every_band() {
    for (i, (_, tint)) in painted().iter().enumerate() {
        assert_eq!(tint.len(), (SIDE * SIDE) as usize, "a banda {i} pinta tudo");
        let back = tint.windows(2).filter(|w| w[1][0] < w[0][0] - 1e-4).count();
        assert_eq!(
            back, 0,
            "a banda {i} tem {back} saltos de cor PARA TRÁS ao longo da ordem de saída — \
             o degradê está a seguir a identidade de montante, não a ordenação"
        );
    }
}

/// **A BANDA 1 CORRE DA ESQUERDA PARA A DIREITA** — a frase que o smoke diz, medida.
///
/// ⚠️ E o CONTROLE é a observação do Enio: a mesma banda **não pode** correr de baixo para
/// cima. Sem esta metade, uma cor que segue a ordem de nascimento da grelha (que é
/// exactamente por linhas, de baixo para cima) passaria pela primeira metade sem tremer.
#[test]
fn the_first_band_paints_left_to_right_and_not_bottom_up() {
    let b = painted();
    let (p, t) = &b[0];
    let monotone_by = |axis: usize| {
        let mut order: Vec<usize> = (0..p.len()).collect();
        order.sort_by(|&a, &c| p[a][axis].total_cmp(&p[c][axis]));
        order.windows(2).all(|w| t[w[1]][0] >= t[w[0]][0] - 1e-4)
    };
    assert!(
        monotone_by(0),
        "ordenando as peças por X a cor tem de crescer — é a definição de 'corre para a direita'"
    );
    assert!(
        !monotone_by(1),
        "…e NÃO pode crescer também com Y: uma cor que sobe nos dois eixos é a ordem de \
         nascimento da grelha, que é o defeito que o Enio viu"
    );
}

/// **AS TRÊS ORDENS SÃO TRÊS**, e o CONJUNTO de posições é o mesmo nas três.
///
/// ⚠️ As duas metades: se as ordens coincidissem a cena não mostraria nada; se o conjunto
/// mudasse, a banda estaria a mover peças em vez de as reordenar — e a mensagem do smoke
/// (*"as peças NÃO se movem entre as bandas"*) seria falsa.
#[test]
fn the_three_keys_give_three_orders_over_the_same_points() {
    let b = bands();
    let n = (SIDE * SIDE) as usize;
    for (i, band) in b.iter().enumerate() {
        assert_eq!(band.len(), n, "a banda {i} tem {n} peças");
    }
    // O MESMO conjunto: cada banda é uma permutação das outras (o `dy` do layout é comum a
    // todas dentro da banda, então comparamos o X, que ele não toca).
    let sorted_x = |v: &Vec<[f32; 2]>| {
        let mut xs: Vec<f32> = v.iter().map(|q| q[0]).collect();
        xs.sort_by(f32::total_cmp);
        xs
    };
    assert_eq!(sorted_x(&b[0]), sorted_x(&b[1]), "mesmo conjunto (1 vs 2)");
    assert_eq!(sorted_x(&b[0]), sorted_x(&b[2]), "mesmo conjunto (1 vs 3)");
    // E as três ORDENS diferem.
    let xs = |v: &Vec<[f32; 2]>| v.iter().map(|q| q[0]).collect::<Vec<_>>();
    assert_ne!(xs(&b[0]), xs(&b[1]), "X contra a diagonal");
    assert_ne!(xs(&b[0]), xs(&b[2]), "X contra o campo");
    assert_ne!(xs(&b[1]), xs(&b[2]), "a diagonal contra o campo");
}

/// **A BANDA 1 ESTÁ DE FACTO ORDENADA POR X** — a âncora que dá sentido às outras duas.
///
/// ⚠️ Sem ela, três listas diferentes provariam só que três números diferentes produzem três
/// resultados diferentes.
#[test]
fn the_first_band_really_runs_left_to_right() {
    let b = bands();
    let xs: Vec<f32> = b[0].iter().map(|q| q[0]).collect();
    assert!(
        xs.windows(2).all(|w| w[0] <= w[1]),
        "a banda 1 tem de sair em X crescente: {xs:?}"
    );
    // E o CONTROLE: a diagonal NÃO sai em X crescente (senão o ângulo não fez nada).
    let d: Vec<f32> = b[1].iter().map(|q| q[0]).collect();
    assert!(
        !d.windows(2).all(|w| w[0] <= w[1]),
        "a banda 2 não pode sair em X crescente — o ângulo tem de virar a ordem"
    );
}
