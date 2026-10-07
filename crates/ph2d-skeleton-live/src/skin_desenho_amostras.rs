//! A amostragem do bake da forma presa — irmão de [`super`] (o desenho fiel).

/// ⭐⭐⭐ **Quantas amostras a forma INTEIRA recebe** — o orçamento que a amostragem reparte pelos
/// segmentos.
///
/// ⚠️ **Não é por segmento, e está medido:** a amostragem óptima sobre os `8` nós da barra é `64`
/// por segmento (`512` no total), e sobre os `54` nós de um ficheiro gravado entre 19 e 20 de
/// Setembro é `16` por segmento — `64` ali custaria `4×` sem ganho. O que as duas têm em comum é o
/// TOTAL, porque o que se amostra é a mesma curva. ⚠️ E amostrar MAIS piora (`128` por segmento
/// sobre `8` nós: máx `0,00456 → 0,02191`) — a amostragem passa a resolver os bicos que a malha
/// linear do campo deixa em cada aresta, em vez de os alisar. *Existe uma amostragem óptima.*
///
/// ⭐ **`1024` desde a lei do MEIO-ÂNGULO (A13, 2026-10-07)** — a tabela acima é da média em
/// CÍRCULO e de um ajuste anterior (re-medida hoje, `128` por segmento já não piora: `0,00342`
/// círculo, `0,00410` meio). Sob o meio-ângulo `512 → 1024` fecha a tampa da junta a `170°`
/// (`0,018 → 0,008`, nenhuma célula sem cor) e o salto entre poses (`0,0024 → 0,0013`); `2048` faz
/// a silhueta mudar fora do contacto (o bake resolve os bicos da malha em viragens que a bola
/// arredonda). Tabela no commit.
pub const AMOSTRAS_POR_FORMA: usize = 1024;

/// O piso e o tecto da repartição — os extremos medidos (`16`/`64` por segmento sob o círculo),
/// dobrados com o orçamento.
pub const AMOSTRAS_POR_SEGMENTO: (usize, usize) = (32, 128);

/// ⭐ **A tolerância do ajuste, em fracção da DIAGONAL da forma** — `0,03 %`, a coluna da tabela
/// que passa por baixo do chão do modelo. ⚠️ Fracção e não comprimento: a mesma forma a outra
/// escala é o mesmo desenho, e um número absoluto mediria o tamanho.
pub const TOLERANCIA_DA_DIAGONAL: f64 = 3e-4;

/// As amostras de cada segmento para um contorno de `segs` segmentos — ver [`AMOSTRAS_POR_FORMA`].
#[must_use]
pub fn amostras_por_segmento(segs: usize) -> usize {
    amostras_no_orcamento(segs, AMOSTRAS_POR_FORMA)
}

/// A mesma repartição com outro orçamento por forma (piso e tecto na mesma proporção) — a porta
/// das réguas que medem a densidade.
#[must_use]
pub fn amostras_no_orcamento(segs: usize, por_forma: usize) -> usize {
    let (piso, tecto) = AMOSTRAS_POR_SEGMENTO;
    let escala = |n: usize| (n * por_forma / AMOSTRAS_POR_FORMA).max(1);
    por_forma
        .div_ceil(segs.max(1))
        .clamp(escala(piso), escala(tecto))
}
