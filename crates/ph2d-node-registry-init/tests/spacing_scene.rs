//! **A CENA DO ESPAÇAMENTO — os quatro números que a mensagem do smoke promete.**
//!
//! O smoke `PH2D_MOTION_NODE_PATH_SMOKE=2` imprime uma tabela e manda o artista julgar duas
//! coisas de OLHO: *as de cima têm o mesmo número de peças e vãos diferentes* · *as de baixo têm
//! o mesmo vão e números diferentes*. Este gate mede exactamente essas duas frases sobre o MESMO
//! grafo que a cena monta, para que a mensagem não possa envelhecer em silêncio.
//!
//! ⚠️ **A cena não é reproduzida aqui — só a LEI dela.** O que a shell acrescenta (a forma no
//! documento vetorial, a entidade, o nome, o publisher) tem gates próprios e exige uma janela;
//! o que este arquivo prova é que, ALIMENTADO com as mesmas quatro trilhas, o nó devolve os
//! números impressos. As duas metades falham por motivos diferentes, e é por isso que a cena
//! também imprime o que montou.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Graph, NodeId};

/// Uma trilha RETA de comprimento `len`, a mesma que o `straight()` da cena desenha.
fn track(len: f32) -> Stream {
    Stream::new(2).with("P", Column::Vec2(vec![[-4.0, 0.0], [-4.0 + len, 0.0]]))
}

/// Coze uma cadeia sobre a trilha nomeada e devolve as posições.
fn walk(name: &str, len: f32, set: impl FnOnce(&mut Graph, NodeId)) -> Vec<[f32; 2]> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");

    let mut g = Graph::new();
    let p = g.add_node("motion.path");
    g.set_text_param(p, "path", name);
    g.set_param(p, "align", 0.0);
    set(&mut g, p);

    let mut cook = Cook::new();
    cook.set_external(ph2d_nodegraph::external::curve_of(name), track(len));
    match cook.cook(&g, &reg, p, 0.0).expect("coza")[0]
        .as_stream()
        .get("P")
    {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// O vão entre duas peças vizinhas de uma trilha reta: a distância no eixo que ela ocupa.
fn gap(pts: &[[f32; 2]]) -> f32 {
    assert!(pts.len() >= 2, "uma trilha com menos de duas peças não tem vão");
    (pts[1][0] - pts[0][0]).abs()
}

/// **As duas de CIMA: mesma contagem, vãos diferentes.**
///
/// É o mundo que sempre shipou, e ele é o CONTROLE da cena — sem esta metade, *"as de baixo têm
/// o mesmo vão"* não diria nada (duas trilhas de comprimentos diferentes poderiam empacotar igual
/// por acaso de escolha de números).
#[test]
fn counting_by_number_gives_the_same_count_and_different_gaps() {
    let short = walk("Count Short", 4.0, |g, p| g.set_param(p, "count", 9.0));
    let long = walk("Count Long", 8.0, |g, p| g.set_param(p, "count", 9.0));

    assert_eq!(short.len(), 9, "curta: 9 peças");
    assert_eq!(long.len(), 9, "longa: 9 peças — a MESMA contagem");

    let (gs, gl) = (gap(&short), gap(&long));
    assert!(
        (gs - 4.0 / 9.0).abs() < 1e-4,
        "curta: vão 4/9 = 0,444, medido {gs}"
    );
    assert!(
        (gl - 8.0 / 9.0).abs() < 1e-4,
        "longa: vão 8/9 = 0,889, medido {gl}"
    );
    assert!(
        (gl / gs - 2.0).abs() < 1e-3,
        "o dobro do comprimento com a mesma contagem é o dobro do vão: {}",
        gl / gs
    );
}

/// **As duas de BAIXO: mesmo vão, contagens diferentes.**
///
/// A entrega da wave. Note que a igualdade dos vãos é EXACTA aqui porque 4 e 8 são múltiplos de
/// 0,5 — a lei geral (*o entregue nunca é mais apertado que o pedido*) tem gate próprio no nó, e
/// a cena escolhe múltiplos de propósito para a leitura de olho ser categórica.
#[test]
fn counting_by_spacing_gives_the_same_gap_and_different_counts() {
    let by_spacing = |name: &str, len: f32| {
        walk(name, len, |g, p| {
            g.set_param(p, "mode", 1.0);
            g.set_param(p, "spacing", 0.5);
        })
    };
    let short = by_spacing("Spacing Short", 4.0);
    let long = by_spacing("Spacing Long", 8.0);

    assert_eq!(short.len(), 8, "curta: 4 / 0,5 = 8 peças");
    assert_eq!(long.len(), 16, "longa: 8 / 0,5 = 16 peças — o dobro");

    for (tag, pts) in [("curta", &short), ("longa", &long)] {
        let g = gap(pts);
        assert!(
            (g - 0.5).abs() < 1e-4,
            "{tag}: o vão É o espaçamento pedido (0,5), medido {g}"
        );
    }
}

/// **E a cena inteira: quatro trilhas, quatro contagens, e as de baixo NÃO são as de cima.**
///
/// A mutação que este gate existe para matar é *o modo Spacing nunca chega ao nó* — nela as quatro
/// linhas caem na contagem do param `count`, que é o default 24 em três delas, e a cena desenharia
/// quatro fileiras indistinguíveis afirmando demonstrar uma diferença.
#[test]
fn the_four_rows_of_the_scene_are_not_the_same_row_four_times() {
    let counts = [
        walk("Count Short", 4.0, |g, p| g.set_param(p, "count", 9.0)).len(),
        walk("Count Long", 8.0, |g, p| g.set_param(p, "count", 9.0)).len(),
        walk("Spacing Short", 4.0, |g, p| {
            g.set_param(p, "mode", 1.0);
            g.set_param(p, "spacing", 0.5);
        })
        .len(),
        walk("Spacing Long", 8.0, |g, p| {
            g.set_param(p, "mode", 1.0);
            g.set_param(p, "spacing", 0.5);
        })
        .len(),
    ];
    assert_eq!(
        counts,
        [9, 9, 8, 16],
        "as quatro contagens que a mensagem do smoke imprime"
    );
}
