//! Os gates do motor de layout.
//!
//! ⚠️ Os oráculos são **ARITMÉTICOS** — a posição que a régua e o gap obrigam —, e não "o que o
//! taffy devolveu". Um gate que afirmasse a saída do motor contra ela mesma passaria com qualquer
//! tradução de estilo, que é justamente a metade que este ficheiro escreve.

use super::*;

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.001
}

/// Os dois eixos com o número que o nó traz — o caso comum, e o que todo gate anterior ao abraço
/// assumia sem o dizer.
fn fixed(size: [f64; 2]) -> [Len; 2] {
    [Len::Fixed(size[0]), Len::Fixed(size[1])]
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
        size: fixed(size),
        ..Node::default()
    }
}

fn child(size: [f64; 2]) -> Node {
    Node {
        parent: Some(0),
        frame: None,
        item: ItemStyle::default(),
        size: fixed(size),
        ..Node::default()
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

/// **O vão PRINCIPAL é o que empilha, seja qual for a direção** (Enio 2026-08-02: *"Gap não
/// funciona em Column"* + *"Cross, que só aparece para Wrap, afeta Column"* — um defeito, visto
/// pelas duas pontas).
///
/// ⚠️ **A fixture `frame()` passa `[gap, gap]`** — o mesmo valor nos dois eixos —, e é por isso
/// que o defeito atravessou a wave com os gates verdes: com main == cross, trocar um pelo outro
/// não muda um número. Este gate dá valores DIFERENTES, e é o que o torna capaz de falhar.
///
/// Nasceu VERMELHO: numa coluna o `main` ia para o `width` do taffy (entre COLUNAS, inerte sem
/// wrap) e o `cross` para o `height`, que é o que de facto separa os filhos empilhados.
#[test]
fn the_main_gap_is_the_one_that_stacks_in_either_direction() {
    let column = FrameStyle {
        dir: Dir::Column,
        gap: [7.0, 100.0], // principal 7 · transversal absurdo, para o engano gritar
        ..FrameStyle::default()
    };
    let mut f = frame(Dir::Column, 0.0, [100.0, 100.0]);
    f.frame = Some(column);
    let out = solve(&[f, child([10.0, 10.0]), child([10.0, 10.0])]).expect("resolve");
    assert!(
        approx(out[2][1], 17.0),
        "o 2o filho da COLUNA devia pousar em 10 + 7 (o vão PRINCIPAL): {out:?}"
    );

    // E o controle: em LINHA o mesmo par continua certo, que é onde a versão antiga acertava.
    let mut r = frame(Dir::Row, 0.0, [100.0, 100.0]);
    r.frame = Some(FrameStyle {
        dir: Dir::Row,
        gap: [7.0, 100.0],
        ..FrameStyle::default()
    });
    let out = solve(&[r, child([10.0, 10.0]), child([10.0, 10.0])]).expect("resolve");
    assert!(
        approx(out[2][0], 17.0),
        "o 2o filho da LINHA devia pousar em 10 + 7: {out:?}"
    );
}

/// **E o vão TRANSVERSAL é o que separa as FAIXAS de um `Wrap`** — a outra metade da mesma lei.
///
/// ⚠️ **A moldura tem a altura EXACTA do conteúdo (10 + 9 + 10), e isso é da fixture, não do
/// gosto.** A primeira versão deste gate dava-lhe 100 de altura e mediu a 2ª faixa em **54,5**:
/// sobrando espaço no eixo transversal, o `taffy` DISTRIBUI as faixas por ele (o `align_content`
/// dele nasce em *stretch*), e o gate teria reprovado por uma razão que não é a que ele julga.
/// Sem folga não há o que distribuir, e o que resta é o vão.
///
/// ⚠️ **E isso é um achado sobre o MODELO, não só sobre o gate:** a moldura não expõe
/// `align_content`, então numa `Wrap` com folga as faixas espalham-se e o vão transversal
/// autorado deixa de ser a distância que se vê. É controlo a decidir (o Figma tem-no), **não
/// construído aqui** — mas nomeado, para não ser descoberto por acidente.
#[test]
fn the_cross_gap_is_the_one_that_separates_wrapped_lines() {
    let mut f = frame(Dir::RowWrap, 0.0, [25.0, 29.0]);
    f.frame = Some(FrameStyle {
        dir: Dir::RowWrap,
        gap: [0.0, 9.0], // sem vão entre irmãos; 9 entre faixas
        ..FrameStyle::default()
    });
    // Três de 10 numa moldura de 25 de largura: dois na primeira faixa, um na segunda.
    let out = solve(&[
        f,
        child([10.0, 10.0]),
        child([10.0, 10.0]),
        child([10.0, 10.0]),
    ])
    .expect("resolve");
    assert!(
        approx(out[3][1], 19.0),
        "o 3o devia abrir faixa em 10 + 9 (o vão TRANSVERSAL): {out:?}"
    );
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
        size: fixed([10.0, 20.0]),
        ..Node::default()
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
        size: fixed([40.0, 20.0]),
        ..Node::default()
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
        size: fixed([100.0, 100.0]),
        ..Node::default()
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

/// **Uma disposição FRACIONÁRIA não é arredondada para unidades inteiras.**
///
/// ⚠️ Este gate nasceu VERMELHO sobre a wave inteira, e a razão de ele ter faltado é a lição:
/// **todas as outras fixtures deste arquivo usam números inteiros** (10 de largura, vãos de 4),
/// e sob elas o arredondamento do `taffy` é invisível — as duas versões dão a mesma resposta.
///
/// Ele é o `taffy` a assumir a unidade de um NAVEGADOR (o pixel de dispositivo, onde meio pixel
/// não existe); a nossa é a unidade de MUNDO do documento. Medido na cena de smoke `=50`, com a
/// moldura em 9,0: cinco filhos de 1,1 saíam a **1,0** e o espaçador de 0,7 com `grow` saía a
/// **4,0** em vez de 3,5 — e **a soma continuava a fechar em 9,0**, que é o que torna o defeito
/// difícil de ver de olho.
#[test]
fn a_fractional_layout_is_not_rounded_to_whole_units() {
    let mut nodes = vec![Node {
        parent: None,
        frame: Some(FrameStyle::default()),
        item: ItemStyle::default(),
        size: fixed([9.0, 2.6]),
        ..Node::default()
    }];
    for i in 0..6 {
        nodes.push(Node {
            parent: Some(0),
            frame: None,
            item: if i == 3 {
                ItemStyle {
                    grow: 1.0,
                    ..ItemStyle::default()
                }
            } else {
                ItemStyle::default()
            },
            size: fixed([if i == 3 { 0.7 } else { 1.1 }, 0.84]),
            ..Node::default()
        });
    }
    let s = solve(&nodes).expect("dispoe");
    for (i, r) in s.iter().enumerate().skip(1) {
        let want = if i == 4 { 3.5 } else { 1.1 };
        assert!(
            (r[2] - want).abs() < 1e-4,
            "o filho {i} mede {:.4} em vez de {want} — a disposicao foi arredondada",
            r[2]
        );
    }
    // E o começo de cada um cai na fracção exacta, não numa grade de inteiros.
    assert!(
        (s[2][0] - 1.1).abs() < 1e-4,
        "o segundo comeca em {:.4}, e nao em 1,1",
        s[2][0]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// O ABRAÇO e os LIMITES (2026-08-10)
// ─────────────────────────────────────────────────────────────────────────────

/// **Uma moldura que abraça mede o que está DENTRO dela** — e o oráculo é a soma, não o motor.
///
/// `2 + 10 + 4 + 10 + 4 + 10 + 2 = 42` de largura; `2 + 6 + 2 = 10` de altura.
#[test]
fn a_hugging_frame_measures_what_is_inside_it() {
    let mut f = frame(Dir::Row, 4.0, [999.0, 999.0]); // o número que ele TRAZ é irrelevante
    f.size = [Len::Hug, Len::Hug];
    f.frame = Some(FrameStyle {
        dir: Dir::Row,
        gap: [4.0, 4.0],
        pad: [2.0, 2.0, 2.0, 2.0],
        ..FrameStyle::default()
    });
    let nodes = [
        f,
        child([10.0, 6.0]),
        child([10.0, 6.0]),
        child([10.0, 6.0]),
    ];
    let out = solve(&nodes).expect("resolve");
    assert!(approx(out[0][2], 42.0), "largura abracada: {:?}", out[0]);
    assert!(approx(out[0][3], 10.0), "altura abracada: {:?}", out[0]);
    // E os filhos continuam a pousar na aritmética de sempre, a partir do recuo.
    assert!(approx(out[1][0], 2.0) && approx(out[2][0], 16.0) && approx(out[3][0], 30.0));
}

/// **Os dois eixos são independentes:** a barra ocupa a largura que lhe deram e tem a altura do
/// que está dentro. É o caso de uso que mais aparece numa UI, e o que um `hug` de eixo único serve.
#[test]
fn hugging_one_axis_leaves_the_other_alone() {
    let mut f = frame(Dir::Row, 0.0, [100.0, 999.0]);
    f.size = [Len::Fixed(100.0), Len::Hug];
    f.frame = Some(FrameStyle {
        pad: [3.0, 0.0, 3.0, 0.0],
        ..FrameStyle::default()
    });
    let nodes = [f, child([10.0, 6.0])];
    let out = solve(&nodes).expect("resolve");
    assert!(
        approx(out[0][2], 100.0),
        "a largura dada manda: {:?}",
        out[0]
    );
    assert!(approx(out[0][3], 12.0), "6 + 3 + 3: {:?}", out[0]);
}

/// **Uma FOLHA que pede para abraçar é RECUSADA** — e o gate existe porque acomodar seria pior que
/// falhar: sem filhos, o conteúdo mede zero e a forma **desapareceria** sem erro nenhum.
#[test]
fn a_leaf_that_asks_to_hug_is_refused() {
    let mut kid = child([10.0, 10.0]);
    kid.size = [Len::Hug, Len::Fixed(10.0)];
    let nodes = [frame(Dir::Row, 0.0, [100.0, 100.0]), kid];
    assert_eq!(solve(&nodes), Err(LayoutError::HugWithoutFlow));
}

/// **O piso segura uma moldura que abraça** — *"cresce com o rótulo, mas nunca mais estreita que
/// isto"*, o botão que não colapsa quando o texto é uma letra só.
#[test]
fn a_minimum_holds_a_hugging_frame_open() {
    let mut f = frame(Dir::Row, 0.0, [0.0, 0.0]);
    f.size = [Len::Hug, Len::Fixed(20.0)];
    f.min = [Some(30.0), None];
    let nodes = [f, child([10.0, 10.0])];
    let out = solve(&nodes).expect("resolve");
    assert!(approx(out[0][2], 30.0), "o piso manda: {:?}", out[0]);
}

/// **O teto limita quem CRESCE** — o par que o Figma serve com *Fill + max*: o campo estica com a
/// janela até uma largura de leitura, e para.
#[test]
fn a_maximum_caps_a_growing_child() {
    let f = frame(Dir::Row, 0.0, [200.0, 20.0]);
    let mut kid = child([10.0, 10.0]);
    kid.item.grow = 1.0;
    kid.max = [Some(60.0), None];
    let out = solve(&[f, kid]).expect("resolve");
    assert!(
        approx(out[1][2], 60.0),
        "sem teto ele tomaria os 200: {:?}",
        out[1]
    );
}

/// **O neutro é byte-idêntico ao mundo de antes desta wave:** sem abraço e sem limites, a árvore
/// resolve exactamente como resolvia. É o controlo que impede a feature de mexer em quem não a
/// pediu.
#[test]
fn without_hug_or_bounds_nothing_moves() {
    let nodes = [
        frame(Dir::Row, 4.0, [100.0, 20.0]),
        child([10.0, 10.0]),
        child([20.0, 10.0]),
    ];
    let out = solve(&nodes).expect("resolve");
    assert!(approx(out[0][2], 100.0) && approx(out[1][0], 0.0) && approx(out[2][0], 14.0));
    assert!(approx(out[1][2], 10.0) && approx(out[2][2], 20.0));
}

/// **Uma moldura que abraça e QUEBRA não quebra** — e o gate existe porque a afirmação estava
/// escrita no doc de [`Len::Hug`] sem nada a segurá-la.
///
/// ⚠️ É ele que torna o espaço oferecido à raiz **load-bearing**: com `MaxContent` o motor põe os
/// três numa linha (a largura sai do conteúdo); com um espaço DEFINIDO de zero, cada filho cai
/// numa linha própria e a moldura fica alta e estreita. A mutação que troca um pelo outro
/// sobrevive a todos os outros gates desta secção — quebrar é responder *"não cabe na largura que
/// me deram"*, e quem abraça não recebeu largura nenhuma.
#[test]
fn a_hugging_frame_that_wraps_does_not_wrap() {
    let mut f = frame(Dir::RowWrap, 0.0, [0.0, 0.0]);
    f.size = [Len::Hug, Len::Hug];
    let nodes = [
        f,
        child([10.0, 6.0]),
        child([10.0, 6.0]),
        child([10.0, 6.0]),
    ];
    let out = solve(&nodes).expect("resolve");
    assert!(
        approx(out[0][2], 30.0),
        "os tres cabem numa linha so: {:?}",
        out[0]
    );
    assert!(
        approx(out[0][3], 6.0),
        "e a altura e a de UMA linha, nao de tres: {:?}",
        out[0]
    );
}
