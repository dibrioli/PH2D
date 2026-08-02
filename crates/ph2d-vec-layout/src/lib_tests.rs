//! Os gates do motor de layout.
//!
//! ⚠️ Os oráculos são **ARITMÉTICOS** — a posição que a régua e o gap obrigam —, e não "o que o
//! taffy devolveu". Um gate que afirmasse a saída do motor contra ela mesma passaria com qualquer
//! tradução de estilo, que é justamente a metade que este ficheiro escreve.

use super::*;

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.001
}

fn frame(dir: Dir, gap: f64, size: [f64; 2]) -> Node {
    Node {
        parent: None,
        frame: Some(FrameStyle {
            dir,
            gap: [gap, gap],
            ..FrameStyle::default()
        }),
        item: ItemStyle::default(),
        size,
    }
}

fn child(size: [f64; 2]) -> Node {
    Node {
        parent: Some(0),
        frame: None,
        item: ItemStyle::default(),
        size,
    }
}

/// **Três filhos numa linha com gap conhecido pousam na aritmética exacta.**
#[test]
fn a_row_puts_three_children_at_the_arithmetic_positions() {
    let nodes = [
        frame(Dir::Row, 4.0, [100.0, 20.0]),
        child([10.0, 10.0]),
        child([20.0, 10.0]),
        child([30.0, 10.0]),
    ];
    let out = solve(&nodes).expect("resolve");
    assert!(approx(out[1][0], 0.0), "1o: {out:?}");
    assert!(approx(out[2][0], 14.0), "2o = 10 + 4: {out:?}");
    assert!(approx(out[3][0], 38.0), "3o = 10+4+20+4: {out:?}");
    // Sem crescimento, cada um fica com a largura autorada.
    assert!(approx(out[1][2], 10.0) && approx(out[2][2], 20.0) && approx(out[3][2], 30.0));
}

/// **O recuo desloca TUDO, e a coluna empilha no outro eixo.**
#[test]
fn padding_offsets_everything_and_a_column_stacks_down() {
    let mut f = frame(Dir::Column, 6.0, [100.0, 100.0]);
    f.frame = Some(FrameStyle {
        dir: Dir::Column,
        gap: [6.0, 6.0],
        pad: [8.0, 0.0, 0.0, 5.0], // topo, direita, base, esquerda
        ..FrameStyle::default()
    });
    let nodes = [f, child([10.0, 10.0]), child([10.0, 20.0])];
    let out = solve(&nodes).expect("resolve");
    assert!(approx(out[1][0], 5.0) && approx(out[1][1], 8.0), "{out:?}");
    // 8 (topo) + 10 (altura do 1o) + 6 (gap)
    assert!(approx(out[2][1], 24.0), "{out:?}");
}

/// **`grow` reparte a SOBRA, e reparte-a na proporção pedida.**
///
/// ⚠️ O oráculo é a conta: 100 de largura, dois filhos de 10, sobra 80; com `grow` 1 e 3 o segundo
/// leva três quartos. Um gate que só exigisse *"o segundo ficou maior"* passaria com a repartição
/// errada.
#[test]
fn grow_splits_the_leftover_in_the_ratio_asked() {
    let mut a = child([10.0, 10.0]);
    let mut b = child([10.0, 10.0]);
    a.item.grow = 1.0;
    b.item.grow = 3.0;
    let nodes = [frame(Dir::Row, 0.0, [100.0, 20.0]), a, b];
    let out = solve(&nodes).expect("resolve");
    assert!(approx(out[1][2], 30.0), "10 + 80/4: {out:?}");
    assert!(approx(out[2][2], 70.0), "10 + 3*80/4: {out:?}");
    assert!(
        approx(out[2][0], 30.0),
        "o 2o comeca onde o 1o acaba: {out:?}"
    );
}

/// **O espaçador com `grow` empurra o resto para o fim** — o caso da barra de ferramentas.
#[test]
fn a_grow_spacer_pushes_the_rest_to_the_end() {
    let mut spacer = child([0.0, 10.0]);
    spacer.item.grow = 1.0;
    let nodes = [
        frame(Dir::Row, 0.0, [100.0, 20.0]),
        child([10.0, 10.0]),
        spacer,
        child([20.0, 10.0]),
    ];
    let out = solve(&nodes).expect("resolve");
    assert!(approx(out[3][0], 80.0), "o ultimo encosta no fim: {out:?}");
}

/// **Uma moldura ANINHADA resolve os filhos dela no tamanho que ela recebeu.**
///
/// É o caso que separa "uma lista de filhos" de "uma árvore": a moldura interna cresce por `grow`,
/// e o que está dentro dela tem de ver a largura NOVA, não a autorada.
#[test]
fn a_nested_frame_lays_its_own_children_at_the_size_it_got() {
    let mut inner = Node {
        parent: Some(0),
        frame: Some(FrameStyle {
            dir: Dir::Row,
            gap: [0.0, 0.0],
            ..FrameStyle::default()
        }),
        item: ItemStyle::default(),
        size: [10.0, 20.0],
    };
    inner.item.grow = 1.0;
    let mut grand = child([10.0, 10.0]);
    grand.parent = Some(1);
    grand.item.grow = 1.0;
    let nodes = [frame(Dir::Row, 0.0, [100.0, 20.0]), inner, grand];
    let out = solve(&nodes).expect("resolve");
    assert!(approx(out[1][2], 100.0), "a interna cresceu: {out:?}");
    assert!(
        approx(out[2][2], 100.0),
        "o neto tem de ver a largura NOVA da interna: {out:?}"
    );
    // E a posição do neto é ABSOLUTA (relativa à raiz), não relativa ao pai.
    assert!(approx(out[2][0], 0.0), "{out:?}");
}

/// **A posição devolvida é relativa à RAIZ.**
///
/// Com a moldura interna deslocada por um irmão antes dela, o neto tem de sair no x do pai + o
/// dele. Devolver a posição relativa ao pai (o que o motor dá) desenharia a sub-árvore inteira no
/// canto da moldura.
#[test]
fn positions_are_absolute_to_the_root() {
    let inner = Node {
        parent: Some(0),
        frame: Some(FrameStyle::default()),
        item: ItemStyle::default(),
        size: [40.0, 20.0],
    };
    let mut grand = child([10.0, 10.0]);
    grand.parent = Some(2);
    let nodes = [
        frame(Dir::Row, 0.0, [100.0, 20.0]),
        child([25.0, 10.0]),
        inner,
        grand,
    ];
    let out = solve(&nodes).expect("resolve");
    assert!(approx(out[2][0], 25.0), "a interna: {out:?}");
    assert!(approx(out[3][0], 25.0), "o neto herda o x do pai: {out:?}");
}

/// **Uma fatia mal ordenada é RECUSADA** — um layout errado não falha, ele desenha.
#[test]
fn a_badly_ordered_slice_is_refused() {
    assert_eq!(solve(&[]), Err(LayoutError::Empty));
    let orphan = [child([1.0, 1.0])];
    assert_eq!(solve(&orphan), Err(LayoutError::NotRooted));
    let two_roots = [
        frame(Dir::Row, 0.0, [10.0, 10.0]),
        frame(Dir::Row, 0.0, [1.0, 1.0]),
    ];
    assert_eq!(solve(&two_roots), Err(LayoutError::NotRooted));
    let mut late_parent = child([1.0, 1.0]);
    late_parent.parent = Some(2); // aponta para a frente
    let bad = [
        frame(Dir::Row, 0.0, [10.0, 10.0]),
        late_parent,
        child([1.0, 1.0]),
    ];
    assert_eq!(solve(&bad), Err(LayoutError::OutOfOrder));
}

/// **A mesma entrada dá a mesma saída** — o passe roda por frame, e uma pose que oscila é uma forma
/// que treme.
#[test]
fn the_same_tree_solves_to_the_same_numbers() {
    let nodes = [
        frame(Dir::RowWrap, 3.0, [50.0, 40.0]),
        child([20.0, 10.0]),
        child([20.0, 10.0]),
        child([20.0, 10.0]),
    ];
    let a = solve(&nodes).expect("resolve");
    let b = solve(&nodes).expect("resolve");
    assert_eq!(a, b);
    // E a quebra aconteceu: o terceiro caiu para a segunda linha.
    assert!(a[3][1] > a[1][1], "a linha tem de quebrar: {a:?}");
}

/// **Um pai que NÃO flui é recusado** — e o gate existe porque o contrário passou.
///
/// ⚠️ O default do motor é `Display::Flex`: um nó sem estilo de moldura **dispõe os filhos na
/// mesma**. A primeira versão desta crate confiava nisso ao contrário e teria empilhado, em
/// silêncio, os filhos de um grupo comum que por acaso caísse dentro da fatia — o artista veria as
/// formas dele saltarem para um canto. Quem monta a fatia recolhe só sub-árvores que fluem.
#[test]
fn a_parent_that_does_not_flow_is_refused() {
    let plain = Node {
        parent: None,
        frame: None,
        item: ItemStyle::default(),
        size: [100.0, 100.0],
    };
    assert_eq!(
        solve(&[plain, child([10.0, 10.0])]),
        Err(LayoutError::ParentDoesNotFlow)
    );
    // Sozinho ele é legítimo: uma moldura sem filhos não dispõe nada.
    assert!(solve(&[plain]).is_ok());
    // E a recusa alcança um nível fundo, não só a raiz.
    let mut group = child([50.0, 50.0]);
    group.frame = None;
    let mut grand = child([10.0, 10.0]);
    grand.parent = Some(1);
    assert_eq!(
        solve(&[frame(Dir::Row, 0.0, [100.0, 100.0]), group, grand]),
        Err(LayoutError::ParentDoesNotFlow)
    );
}
