//! Os gates da [`super::SourceCells`] — *que pixels o quad mostra*.

use super::SourceCells;

/// ⭐⭐ **SEM REGIÃO E SEM GRELHA, a célula é a IMAGEM INTEIRA** — o caso de omissão, e a prova de
/// que pôr esta porta no caminho de quem prende não muda uma sprite normal.
///
/// (Mutação: `hf.max(1)` → `hf` ⇒ divisão por zero / `None` numa sprite sem grelha.)
#[test]
fn a_sprite_sem_grelha_le_a_imagem_inteira() {
    let c = SourceCells::of([160, 40], None, 1, 1).expect("a imagem tem lado");
    assert_eq!(c.origin, [0.0, 0.0]);
    assert_eq!(c.cell, [160.0, 40.0]);
    assert_eq!(c.count(), 1);
    assert_eq!(c.cell_origin(0), [0.0, 0.0]);
    assert_eq!(c.cell_px(), [160, 40]);
}

/// ⭐⭐⭐ **A GRELHA DIVIDE, E A ORDEM É A DA UV** (`col = k % hframes`, linha a crescer para baixo).
///
/// ⛔⛔ **Trocar as duas põe a malha do quadro 3 sobre a arte do 1**, e o sintoma é uma animação em
/// que a silhueta e a tinta andam em quadros diferentes — *um defeito que passa em todo gate de
/// geometria*.
#[test]
fn a_grelha_divide_na_mesma_ordem_que_a_uv() {
    let c = SourceCells::of([160, 40], None, 4, 2).expect("lado");
    assert_eq!(c.cell, [40.0, 20.0]);
    assert_eq!(c.count(), 8);
    assert_eq!(c.cell_origin(0), [0.0, 0.0]);
    assert_eq!(
        c.cell_origin(1),
        [40.0, 0.0],
        "o quadro 1 e' a COLUNA seguinte"
    );
    assert_eq!(
        c.cell_origin(4),
        [0.0, 20.0],
        "o quadro 4 e' a LINHA seguinte"
    );
    assert_eq!(c.cell_origin(7), [120.0, 20.0]);
    // ⚠️ Fora da grelha satura na última célula, como a `sprite_sheet_subrect` faz com o `frame`.
    assert_eq!(c.cell_origin(99), c.cell_origin(7));
}

/// ⭐⭐ **A REGIÃO DESLOCA A ORIGEM E ENCOLHE A CÉLULA** — e as duas leis COMPÕEM, que é a ordem que
/// a emissão usa (`region_subrect` e só depois `sprite_sheet_subrect`).
#[test]
fn a_regiao_desloca_a_origem_e_a_grelha_divide_o_que_sobra() {
    let c = SourceCells::of([256, 256], Some([16.0, 8.0, 80.0, 40.0]), 2, 2).expect("lado");
    assert_eq!(c.origin, [16.0, 8.0]);
    assert_eq!(c.cell, [40.0, 20.0]);
    assert_eq!(c.cell_origin(3), [56.0, 28.0]);
}

/// ⛔ **UMA REGIÃO DEGENERADA É IGNORADA** — a mesma porta de saída da [`super::region_subrect`].
///
/// ⚠️ *Duas respostas diferentes para «esta região conta?» dariam uma célula no extract e outra no
/// bind*, e o sintoma seria a malha traçada sobre a imagem toda numa sprite com região.
#[test]
fn uma_regiao_de_lado_nulo_nao_conta() {
    let com = SourceCells::of([160, 40], Some([10.0, 10.0, 0.0, 30.0]), 1, 1).expect("lado");
    assert_eq!(com, SourceCells::of([160, 40], None, 1, 1).expect("lado"));
}

/// ⛔ **SEM FONTE NÃO HÁ CÉLULA** — e `None` é a resposta certa, porque o chamador tem um caminho
/// de sempre para onde cair.
#[test]
fn sem_dimensoes_nao_ha_celula() {
    assert!(SourceCells::of([0, 40], None, 1, 1).is_none());
    assert!(SourceCells::of([160, 0], None, 1, 1).is_none());
}

/// ⚠️ **O BUFFER ARREDONDA PARA CIMA** — um lado de `20,5 px` tem de caber em `21`, senão a última
/// coluna da arte fica de fora da malha.
#[test]
fn o_buffer_da_celula_arredonda_para_cima() {
    let c = SourceCells::of([41, 41], None, 2, 2).expect("lado");
    assert_eq!(c.cell, [20.5, 20.5]);
    assert_eq!(c.cell_px(), [21, 21]);
}
