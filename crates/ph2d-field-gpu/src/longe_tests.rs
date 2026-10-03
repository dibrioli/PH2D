//! Os gates da GRADE DE LONGE e do RECORTE que não precisam de placa — ver [`super`].

use super::{CABECALHO, FOLGA, LEI, Longe, cabecalho};

fn caixa() -> Longe {
    Longe {
        lo: [-1.0, -0.5, -2.0],
        hi: [3.0, 0.5, 2.0],
        res: 16,
        perto: 1.5,
    }
}

/// ⭐⭐⭐ **O cabeçalho tem a ordem que o WGSL lê — as DUAS metades.**
///
/// ⚠️ O layout vive em dois sítios (a [`cabecalho`] escreve, a [`LEI`] lê por índice), e um
/// deslocamento de um lugar entre eles não falha a compilar: lê a aresta da célula onde devia ler um
/// canto. ⇒ a metade de cima afirma o que a escrita põe em cada índice, e a de baixo que o texto do
/// shader lê cada grandeza DESSE índice.
#[test]
fn o_cabecalho_tem_a_ordem_que_a_lei_le() {
    let l = caixa();
    let g = l.grade().expect("a grade");
    let h = cabecalho(&l, 12_345);
    assert_eq!(h.len(), CABECALHO);
    assert_eq!(h[0..3], l.lo);
    assert_eq!(h[3], g.celula);
    assert_eq!(h[4..7], l.hi);
    assert_eq!(h[7].to_bits(), 12_345, "o deslocamento viaja nos BITS");
    #[allow(clippy::cast_precision_loss)]
    let dims = g.dims.map(|d| d as f32);
    assert_eq!(h[8..11], dims);
    assert_eq!(h[11], l.perto * g.celula);
    assert_eq!(h[12..15], g.origem);

    for (leitura, o_que) in [
        ("k[h + 4u], k[h + 5u], k[h + 6u]", "o canto máximo"),
        ("let cel = k[h + 3u]", "a aresta da célula"),
        (
            "u32(k[h + 8u]), u32(k[h + 9u]), u32(k[h + 10u])",
            "os nós por eixo",
        ),
        ("k[h + 12u], k[h + 13u], k[h + 14u]", "o nó (0,0,0)"),
        ("bitcast<u32>(k[h + 7u])", "o deslocamento"),
        ("k[s.longe - 1u + 11u]", "o limite de perto"),
        ("k[s.longe - 1u + 3u] > 0.0", "a pergunta «há grade?»"),
    ] {
        assert!(
            LEI.contains(leitura),
            "a LEI deixou de ler {o_que} como `{leitura}` — o cabeçalho e o shader divergiram"
        );
    }
}

/// ⭐⭐⭐ **`res = 0` é SÓ o recorte** — nenhuma grade assada, nenhum `f32` a mais no armazém, e a
/// célula a `0`, que é o que o shader lê para não perguntar à grade em passo nenhum.
///
/// ⚠️ É o valor de OMISSÃO do produto (a grade está recusada por medição), logo esta é a metade que
/// corre em todo quadro.
#[test]
fn res_zero_e_so_o_recorte() {
    let l = Longe { res: 0, ..caixa() };
    assert!(l.grade().is_none());
    assert_eq!(l.valores(), 0);
    let h = cabecalho(&l, 0);
    assert_eq!(h[0..3], l.lo);
    assert_eq!(h[4..7], l.hi);
    assert_eq!(h[3], 0.0, "com a célula a zero o shader salta a grade");
}

/// ⭐⭐ **A grade cobre a caixa, com a folga dos DOIS lados.**
///
/// ⚠️ O salto de fora da grade é a distância à caixa da GRADE; se ela não cobrisse a da peça, um
/// ponto entre as duas saltaria pela distância a uma caixa menor que a peça.
#[test]
fn a_grade_cobre_a_caixa() {
    let l = caixa();
    let g = l.grade().expect("a grade");
    #[allow(clippy::cast_precision_loss)]
    let folga = g.celula * FOLGA as f32;
    for a in 0..3 {
        #[allow(clippy::cast_precision_loss)]
        let topo = ((g.dims[a] - 1) as f32).mul_add(g.celula, g.origem[a]);
        assert!(
            g.origem[a] <= l.lo[a] - folga * 0.999,
            "eixo {a}: a grade começa dentro da caixa"
        );
        assert!(
            topo >= l.hi[a] + folga * 0.999,
            "eixo {a}: a grade acaba dentro da caixa"
        );
    }
}

/// O texto do despacho que assa abre o armazém à escrita UMA vez — ver [`super::comum_para_assar`].
#[test]
fn o_despacho_que_assa_escreve_o_armazem() {
    let t = super::comum_para_assar();
    assert_eq!(t.matches("var<storage, read_write> grades").count(), 1);
    assert_eq!(t.matches("var<storage, read> grades").count(), 0);
}
