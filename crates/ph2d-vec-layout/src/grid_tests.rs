//! **Os gates da GRADE** — irmão do [`super::tests`] por ASSUNTO: aqui mora o que só a grade
//! responde, e o oráculo é sempre **a aritmética que a régua e o vão obrigam**, nunca *"o que o
//! motor devolveu"*.

use super::{
    Align, Dir, FrameStyle, ItemStyle, Justify, LayoutError, Len, MAX_GRID_TRACKS, Node, solve,
};

/// Uma moldura em grade com `kids` filhos de `10 × 6`.
fn grid(size: [Len; 2], columns: u16, kids: usize, gap: [f64; 2]) -> Vec<Node> {
    kid_widths(size, Dir::Grid { columns }, &vec![10.0; kids], gap)
}

/// A mesma moldura, mas com uma largura POR FILHO e uma direção à escolha.
fn kid_widths(size: [Len; 2], dir: Dir, widths: &[f64], gap: [f64; 2]) -> Vec<Node> {
    let mut out = vec![Node {
        parent: None,
        frame: Some(FrameStyle {
            dir,
            gap,
            ..FrameStyle::default()
        }),
        size,
        ..Node::default()
    }];
    out.extend(widths.iter().map(|w| Node {
        parent: Some(0),
        size: [Len::Fixed(*w), Len::Fixed(6.0)],
        item: ItemStyle::default(),
        ..Node::default()
    }));
    out
}

/// **As células caem onde a aritmética manda.**
///
/// 3 colunas de 10 com vão 4 ⇒ `x = 0, 14, 28`; 3 linhas de 6 com vão 2 ⇒ `y = 0, 8, 16`. O sétimo
/// filho abre a terceira linha sozinho — é ele que prova que as linhas **nascem**, e que ninguém
/// as autorou.
#[test]
fn the_cells_land_where_the_column_count_and_the_gap_put_them() {
    let out = solve(&grid(
        [Len::Fixed(38.0), Len::Fixed(22.0)],
        3,
        7,
        [4.0, 2.0],
    ))
    .expect("resolve");
    let want = [
        (0.0, 0.0),
        (14.0, 0.0),
        (28.0, 0.0),
        (0.0, 8.0),
        (14.0, 8.0),
        (28.0, 8.0),
        (0.0, 16.0),
    ];
    for (i, (x, y)) in want.into_iter().enumerate() {
        let r = out[i + 1];
        assert!(
            (r[0] - x).abs() < 1e-6 && (r[1] - y).abs() < 1e-6,
            "filho {i} caiu em ({}, {}) e a aritmetica pede ({x}, {y})",
            r[0],
            r[1]
        );
    }
}

/// ⭐ **A RAZÃO DE A GRADE EXISTIR, e o `RowWrap` é o controlo.**
///
/// Com conteúdos de larguras DIFERENTES, o wrap arruma cada faixa por si — nada fica em coluna. A
/// grade dá a cada célula a mesma largura, então o item `i` de cada linha começa no mesmo `x`.
///
/// ⚠️ Sem este gate a feature inteira seria indistinguível de um wrap com sorte: as duas dispõem
/// em linhas, e num conjunto de larguras IGUAIS elas coincidem — que é exactamente a fixture em
/// que um gate ingénuo teria nascido verde.
#[test]
fn a_grid_keeps_its_columns_aligned_where_a_wrap_does_not() {
    // ⚠️ **A largura da moldura é escolhida para o wrap quebrar 3+3**, e a premissa é AFIRMADA
    // abaixo. A primeira versão deste gate usou 96 e o wrap partiu 5+1 — *um wrap quebra onde a
    // largura acaba, nunca numa contagem de colunas* —, então o índice 3 nem era o começo da
    // segunda faixa e o controlo falhou por fixture, não por produto.
    let widths = [10.0, 20.0, 10.0, 20.0, 10.0, 10.0];
    let frame = [Len::Fixed(45.0), Len::Fixed(20.0)];
    let g = solve(&kid_widths(
        frame,
        Dir::Grid { columns: 3 },
        &widths,
        [0.0, 0.0],
    ))
    .expect("grade");
    let w = solve(&kid_widths(frame, Dir::RowWrap, &widths, [0.0, 0.0])).expect("wrap");
    assert!(
        w[3][1] < 1e-6 && w[4][1] > 1e-6,
        "a PREMISSA caiu: o wrap tinha de partir 3+3 para a comparacao ser justa, e partiu noutro \
         sitio (y do 3o = {}, y do 4o = {})",
        w[3][1],
        w[4][1]
    );
    // A COLUNA 0 é o controlo do controlo: as duas alinham ali, e trivialmente — toda faixa começa
    // à esquerda. Se o gate parasse aqui ele diria *"o wrap não alinha nada"*, que é falso.
    for (name, out) in [("grade", &g), ("wrap", &w)] {
        assert!(
            (out[1][0] - out[4][0]).abs() < 1e-6,
            "{name}: a coluna 0 comeca no mesmo x nas duas linhas"
        );
    }
    // A COLUNA 1 é onde as duas divergem: na grade ela é uma pista, no wrap ela é *o que sobrou
    // depois do vizinho da esquerda*.
    assert!(
        (g[2][0] - g[5][0]).abs() < 1e-6,
        "a grade devia alinhar a coluna 1 das duas linhas, e deu {} contra {}",
        g[2][0],
        g[5][0]
    );
    assert!(
        (w[2][0] - w[5][0]).abs() > 1.0,
        "o CONTROLO caiu: se o wrap tambem alinhasse a coluna 1, a grade nao estaria a dar nada de \
         novo ({} contra {})",
        w[2][0],
        w[5][0]
    );
}

/// **O `Hug` sobrevive a colunas `1fr`** — medido, e não lido da spec.
///
/// ⚠️ O modo de falha que isto exclui é o gémeo exacto do [`LayoutError::HugWithoutFlow`]: uma
/// pista `1fr` precisa de uma largura para dividir, e o abraço oferece `MaxContent`. Se ela
/// colapsasse para zero ali, a moldura **desapareceria da tela** sem erro nenhum.
#[test]
fn a_hugging_grid_measures_its_content_instead_of_collapsing() {
    let fixed = solve(&grid(
        [Len::Fixed(38.0), Len::Fixed(22.0)],
        3,
        7,
        [4.0, 2.0],
    ))
    .expect("fixa");
    for size in [
        [Len::Hug, Len::Hug],
        [Len::Hug, Len::Fixed(22.0)],
        [Len::Fixed(38.0), Len::Hug],
    ] {
        let out = solve(&grid(size, 3, 7, [4.0, 2.0])).expect("abraco");
        assert!(
            (out[0][2] - fixed[0][2]).abs() < 1e-6 && (out[0][3] - fixed[0][3]).abs() < 1e-6,
            "a moldura que abraca mediu {:?} e a fixa mede {:?}",
            &out[0][2..],
            &fixed[0][2..]
        );
    }
}

/// **O vão PRINCIPAL separa colunas e o TRANSVERSAL separa linhas** — a mesma frase do `Row`, e por
/// isso o mesmo braço.
#[test]
fn the_main_gap_is_between_columns_and_the_cross_gap_between_rows() {
    // ⚠️ **A moldura mede exactamente `2·10 + 7`**, e não é arredondamento: só com a célula do
    // tamanho do filho é que a distância entre BORDAS de filhos *é* o vão. A primeira versão deste
    // gate usou 50, onde a célula mede 21,5 e o filho 10 — ele mediu 18,5 e acusou o produto de um
    // erro que era do oráculo.
    let out = solve(&grid(
        [Len::Fixed(27.0), Len::Fixed(15.0)],
        2,
        4,
        [7.0, 3.0],
    ))
    .expect("resolve");
    let col_gap = out[2][0] - (out[1][0] + out[1][2]);
    let row_gap = out[3][1] - (out[1][1] + out[1][3]);
    assert!(
        (col_gap - 7.0).abs() < 1e-6,
        "entre colunas o vao devia ser o PRINCIPAL (7), e mediu {col_gap}"
    );
    assert!(
        (row_gap - 3.0).abs() < 1e-6,
        "entre linhas o vao devia ser o TRANSVERSAL (3), e mediu {row_gap}"
    );
}

/// **O alinhamento diz onde o filho senta DENTRO da célula.**
///
/// A célula mede 30 e o filho 10, então o `Start` põe-no em 0, o `Center` em 10 e o `End` em 20 —
/// os três números que a aritmética obriga.
#[test]
fn the_child_sits_where_the_alignment_puts_it_inside_its_own_cell() {
    for (justify, want_x) in [
        (Justify::Start, 0.0),
        (Justify::Center, 10.0),
        (Justify::End, 20.0),
        // ⚠️ As duas DISTRIBUIÇÕES caem em `Start`: repartir sobra não é posicionar dentro de uma
        // caixa. O painel não as oferece na grade; um documento que as traga de um `Row` anterior
        // é lido assim, e o valor dele fica intacto para quando ele voltar.
        (Justify::SpaceBetween, 0.0),
        (Justify::SpaceAround, 0.0),
    ] {
        let mut tree = grid([Len::Fixed(90.0), Len::Fixed(20.0)], 3, 3, [0.0, 0.0]);
        tree[0].frame.as_mut().expect("moldura").justify = justify;
        let out = solve(&tree).expect("resolve");
        assert!(
            (out[1][0] - want_x).abs() < 1e-6,
            "{justify:?} devia pousar o filho em x={want_x} e pousou em {}",
            out[1][0]
        );
    }
    for (align, want_y) in [
        (Align::Start, 0.0),
        (Align::Center, 7.0),
        (Align::End, 14.0),
    ] {
        let mut tree = grid([Len::Fixed(90.0), Len::Fixed(20.0)], 3, 3, [0.0, 0.0]);
        let f = tree[0].frame.as_mut().expect("moldura");
        f.align = align;
        let out = solve(&tree).expect("resolve");
        assert!(
            (out[1][1] - want_y).abs() < 1e-6,
            "{align:?} devia pousar o filho em y={want_y} e pousou em {}",
            out[1][1]
        );
    }
}

/// ⭐ **O `align` governa ONDE O BLOCO DE LINHAS SENTA, e não só o filho dentro da linha.**
///
/// ⚠️ Sem esta metade a grade herdaria o CSS, onde `align-content: normal` numa grade se comporta
/// como `stretch`: as linhas seriam ESTICADAS para encher a moldura, e a mesma cena que um `Row`
/// encosta no topo sairia com as faixas espalhadas e o filho a flutuar. O gate vivo da shell
/// nasceu vermelho exactamente aí.
///
/// A moldura mede 40 de altura e o conteúdo **17** (duas linhas de 6 com vão 5), então sobram 23 —
/// e é sobre eles que os quatro valores respondem coisas diferentes.
///
/// ⚠️ Este gate nasceu vermelho por eu ter escrito `10` onde o filho mede `6` (a LARGURA dele, não
/// a altura) — a terceira aritmética de fixture errada deste arquivo, e as três acusaram o produto
/// de um erro que era do oráculo.
#[test]
fn the_alignment_also_says_where_the_block_of_rows_sits() {
    for (align, want_top_of_first_row) in [
        (Align::Start, 0.0),
        (Align::Center, 11.5),
        (Align::End, 23.0),
    ] {
        let mut tree = grid([Len::Fixed(38.0), Len::Fixed(40.0)], 3, 5, [4.0, 5.0]);
        tree[0].frame.as_mut().expect("moldura").align = align;
        let out = solve(&tree).expect("resolve");
        assert!(
            (out[1][1] - want_top_of_first_row).abs() < 1e-6,
            "{align:?}: a 1a linha devia comecar em y={want_top_of_first_row} e comecou em {}",
            out[1][1]
        );
    }
    // ⚠️ `Stretch` é o único que faz as LINHAS encherem a moldura: `(40 − 5)/2 = 17,5` cada, então
    // a segunda começa em `17,5 + 5 = 22,5` — contra os 11 que o `Start` dá.
    let mut tree = grid([Len::Fixed(38.0), Len::Fixed(40.0)], 3, 5, [4.0, 5.0]);
    tree[0].frame.as_mut().expect("moldura").align = Align::Stretch;
    let out = solve(&tree).expect("resolve");
    assert!(
        (out[4][1] - 22.5).abs() < 1e-6,
        "com Stretch a 2a linha devia comecar em 22,5 e comecou em {}",
        out[4][1]
    );
    // ⚠️ **E o FILHO não estica**, porque ele traz um tamanho explícito — toda forma deste
    // documento traz, é a bbox dela. Isto NÃO é uma divergência da grade: medido, um `Row` com
    // `Stretch` deixa o mesmo filho em 6,0 exactamente igual. O `Stretch` é vivo para um filho
    // cujo tamanho é `auto`, isto é, uma moldura aninhada que ABRAÇA — e ali ele vale nos dois.
    assert!(
        (out[4][3] - 6.0).abs() < 1e-6,
        "um filho de altura AUTORADA nao pode ser esticado; mediu {}",
        out[4][3]
    );
}

/// ⭐ **O teto de faixas RECUSA em vez de deixar o motor panicar** — e o par é o que o torna um
/// gate: um filho abaixo do teto resolve, e o seguinte é recusado.
///
/// ⚠️ O teto morde pelas LINHAS, que ninguém autora. Com **uma** coluna, `MAX_GRID_TRACKS` filhos
/// são `MAX_GRID_TRACKS` linhas — o último número que o `taffy` sabe indexar.
#[test]
fn the_track_ceiling_refuses_the_grid_that_would_panic_the_engine() {
    let n = usize::from(MAX_GRID_TRACKS);
    let ok = grid([Len::Fixed(10.0), Len::Fixed(10.0)], 1, n, [0.0, 0.0]);
    assert!(
        solve(&ok).is_ok(),
        "o CONTROLO caiu: {n} linhas sao exactamente o que o motor ainda indexa"
    );
    let over = grid([Len::Fixed(10.0), Len::Fixed(10.0)], 1, n + 1, [0.0, 0.0]);
    assert_eq!(
        solve(&over),
        Err(LayoutError::GridTooManyTracks),
        "uma linha a mais tem de ser RECUSADA, e nao entregue ao motor"
    );
    // E o mesmo teto no outro eixo, pelo lado que o artista de facto escreve.
    let cols = grid(
        [Len::Fixed(10.0), Len::Fixed(10.0)],
        MAX_GRID_TRACKS,
        1,
        [0.0, 0.0],
    );
    assert!(solve(&cols).is_ok(), "o CONTROLO do eixo das colunas caiu");
}

/// **Zero colunas é um degenerado com UMA leitura sã** — uma coluna —, e não uma recusa: recusar
/// pararia a moldura inteira por causa de um número que o painel já impede.
#[test]
fn a_grid_of_zero_columns_reads_as_one_column() {
    let out = solve(&grid(
        [Len::Fixed(10.0), Len::Fixed(30.0)],
        0,
        3,
        [0.0, 0.0],
    ))
    .expect("resolve");
    for (i, r) in out.iter().skip(1).enumerate() {
        assert!(
            r[0].abs() < 1e-6,
            "com zero colunas os tres filhos empilham numa so, e o {i} caiu em x={}",
            r[0]
        );
    }
}

/// **Nada do que já shipava se mexe** — o `Row`/`Column`/`RowWrap` continua byte a byte onde
/// estava, e é isto que faz da grade uma ADIÇÃO em vez de uma reescrita.
#[test]
fn the_three_flexbox_directions_are_untouched_by_the_grid_branch() {
    for dir in [Dir::Row, Dir::Column, Dir::RowWrap] {
        let out = solve(&kid_widths(
            [Len::Fixed(100.0), Len::Fixed(40.0)],
            dir,
            &[10.0, 10.0, 10.0],
            [4.0, 2.0],
        ))
        .expect("resolve");
        let want: [f64; 3] = match dir {
            Dir::Column => [0.0, 0.0, 0.0],
            _ => [0.0, 14.0, 28.0],
        };
        for (i, x) in want.into_iter().enumerate() {
            assert!(
                (out[i + 1][0] - x).abs() < 1e-6,
                "{dir:?}: o filho {i} devia estar em x={x} e esta em {}",
                out[i + 1][0]
            );
        }
    }
}
