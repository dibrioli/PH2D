//! **O ATLAS de raster do FX** — onde, na textura partilhada de um frame, cai a célula de cada
//! forma filtrada.
//!
//! # Por que existe
//!
//! O `fx_live` rasterizava cada forma filtrada num render próprio do Vello. Medido
//! (`ph2d-render/tests/fx_scene_scale_cost.rs`, RTX): um render custa **~0,12 ms antes de desenhar
//! coisa alguma**, e a MESMA área de arte que 32 renders cobrem em **4,0 ms** sai em **0,39 ms**
//! numa passagem só. Numa cena de jogo as formas filtradas ANIMAM, logo erram o memo todas, todo
//! frame — e o custo que multiplica é esse fixo, não o filtro.
//!
//! ⚠️ **A pergunta é de EMPACOTAMENTO, não de GPU**, e é por isso que mora num arquivo sozinho e
//! sem `wgpu`: ela se testa inteira na cabeça, e o modo de falha dela (duas células a sobrepor-se)
//! é exactamente o tipo de coisa que um gate headless prova e uma foto não.
//!
//! O algoritmo é o de PRATELEIRAS (o mesmo do atlas de sprites): ordena por altura decrescente e
//! enche linhas. Ele desperdiça alguma área contra um empacotador exacto — e área desperdiçada
//! aqui custa **preenchimento de textura**, que é a metade barata; o que se está a comprar é o
//! número de RENDERS, que ele leva a um.

/// A célula de uma forma: o índice dela na lista que o chamador entregou, e a origem dela na
/// textura do lote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Placement {
    pub(crate) index: usize,
    pub(crate) org: [u32; 2],
}

/// Um LOTE: uma textura, um render do Vello, e as formas que cabem nela.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Batch {
    pub(crate) w: u32,
    pub(crate) h: u32,
    pub(crate) cells: Vec<Placement>,
}

/// Empacota `sizes` em lotes de lado máximo `max_side`.
///
/// ⚠️ **A ordem de saída é determinista** (altura decrescente, desempate por largura e depois pelo
/// índice): o mesmo frame tem de dar o mesmo atlas, senão o memo do `fx_live` — que compara a
/// pilha resolvida — passaria a ver trabalho onde não há.
///
/// Uma forma maior que `max_side` num eixo ganha o lote dela e a textura sai do tamanho dela: o
/// chamador já a limitou (`MAX_FX_SIDE`), e devolver "não coube" faria a forma **desaparecer** —
/// que é o modo de falha errado para um limite de recurso.
pub(crate) fn pack(sizes: &[(u32, u32)], max_side: u32) -> Vec<Batch> {
    if sizes.is_empty() {
        return Vec::new();
    }
    let max_side = max_side.max(1);
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by_key(|&i| {
        let (w, h) = sizes[i];
        (std::cmp::Reverse(h), std::cmp::Reverse(w), i)
    });

    let mut batches: Vec<Batch> = Vec::new();
    let mut cells: Vec<Placement> = Vec::new();
    // O cursor das prateleiras: onde a linha corrente começa em Y, quanto já se andou em X, e a
    // altura dela (a da primeira forma, porque a lista vem ordenada por altura decrescente).
    let (mut shelf_y, mut x, mut shelf_h) = (0u32, 0u32, 0u32);

    for &i in &order {
        let (w, h) = (sizes[i].0.max(1), sizes[i].1.max(1));
        // ⚠️ **Uma forma maior que o teto vai SOZINHA**, e este ramo é o que impede o modo de falha
        // que o gate apanhou: sem ele ela era posta na prateleira corrente e ESPALHAVA o próprio
        // excesso pela textura do lote inteiro — os vizinhos, que cabiam, passavam a viver numa
        // textura que o device recusa. Sozinha, a textura sai do tamanho dela, que é o que o
        // chamador já limitou.
        if w > max_side || h > max_side {
            if !cells.is_empty() {
                batches.push(finish(std::mem::take(&mut cells), sizes));
            }
            batches.push(finish(
                vec![Placement {
                    index: i,
                    org: [0, 0],
                }],
                sizes,
            ));
            shelf_y = 0;
            x = 0;
            shelf_h = 0;
            continue;
        }
        // Não cabe nesta prateleira: abre a seguinte.
        if x > 0 && x.saturating_add(w) > max_side {
            shelf_y = shelf_y.saturating_add(shelf_h);
            x = 0;
            shelf_h = 0;
        }
        // Não cabe neste lote: fecha-o e abre outro. (Um lote vazio nunca é fechado — senão uma
        // forma maior que o teto entraria em ciclo.)
        if shelf_y > 0 && shelf_y.saturating_add(h) > max_side {
            batches.push(finish(std::mem::take(&mut cells), sizes));
            shelf_y = 0;
            x = 0;
            shelf_h = 0;
        }
        cells.push(Placement {
            index: i,
            org: [x, shelf_y],
        });
        x = x.saturating_add(w);
        shelf_h = shelf_h.max(h);
    }
    if !cells.is_empty() {
        batches.push(finish(cells, sizes));
    }
    batches
}

/// As dimensões de um lote são as do conteúdo — o retângulo APERTADO, não o teto: a textura é
/// alocada e rasterizada, e pagar 8192² por quatro estrelas seria pagar o teto todo frame.
fn finish(cells: Vec<Placement>, sizes: &[(u32, u32)]) -> Batch {
    let (mut w, mut h) = (1u32, 1u32);
    for c in &cells {
        let (cw, ch) = (sizes[c.index].0.max(1), sizes[c.index].1.max(1));
        w = w.max(c.org[0].saturating_add(cw));
        h = h.max(c.org[1].saturating_add(ch));
    }
    Batch { w, h, cells }
}

#[cfg(test)]
#[path = "fx_atlas_tests.rs"]
mod tests;
