//! **A partição trapped-ball** (`docs/Flip/09 §8`) — o que torna o Colorize uma operação de
//! clique em vez de uma de minutos.
//!
//! # Por que ela existe (o número, não a intuição)
//!
//! O corte LazyBrush sobre a grade de PIXELS foi medido (`§7.1`): **3,3 s a 4096²**, e
//! **157 s** quando dois rabiscos se contradizem sobre a mesma linha. A partição responde a
//! pergunta cara — *"a linha separa estes dois pontos?"* — UMA vez, por flood, e o resto do
//! pipeline decide componente a componente (preencher, ou dividir por Voronoi — `solve`).
//!
//! # A lei
//!
//! Um **componente** é a topologia da TINTA: o flood de papel não atravessa tinta, então dois
//! pontos caem no mesmo componente se e só se há um caminho de papel entre eles. A **bola** de
//! raio `trap_px` (a erosão do trapped-ball, Zhang et al. TVCG 2009) costura todo vão mais
//! estreito que o diâmetro dela — o Colorize atravessa vãos até `2·trap_px` e não mais. É o
//! mesmo `trap_px` que o balde já expõe no painel, não um segundo knob para a mesma pergunta.
//!
//! # Sobre-segmentar é seguro; sub-segmentar é PERDA
//!
//! Um componente a mais só dá liberdade ao solver. Já duas áreas que o artista quer de cores
//! diferentes, fundidas num componente só, **nunca mais podem ser separadas**. Por isso a
//! erosão erra para o lado fino.
//!
//! # ⚠️ As CÉLULAS e o grafo de arestas MORRERAM (4º smoke, 2026-07-20)
//!
//! A versão anterior subdividia cada componente numa grade de células e devolvia um grafo
//! ponderado por `V_pq` para o solver cortar. Dois defeitos, ambos visíveis no smoke:
//! uma célula (> 1 px) **cavalgava a linha** — um nó só com papel dos dois lados da tinta, e
//! a cor vazava por ele — e o `V_pq` como métrica de DISTÂNCIA tem a direção **errada**
//! (`V_pq` é custo de CORTE: tinta barata; numa métrica isso faz a linha ficar invisível
//! para a frente, e a fronteira caía no meio dos rabiscos em vez de na linha). O componente
//! contestado agora é dividido por Voronoi geodésico **por pixel** (`voronoi.rs`), que não
//! consome grafo nenhum — só esta partição e a EDT.

use ph2d_flip_fill::{BOUNDARY, Grid, sq_distance_to_set};

/// O pixel não pertence a componente nenhum (a grade não tem papel alcançável).
pub const NO_REGION: u32 = u32::MAX;

/// A partição da grade em componentes estanques.
pub struct Segmentation {
    /// `component[i]` = o componente estanque do pixel `i` (a topologia da TINTA), ou
    /// [`NO_REGION`]. Papel, papel fino e a própria tinta são todos atribuídos.
    pub component: Vec<u32>,
    /// Quantos componentes (os ids são `0..count`).
    pub count: usize,
    /// Distância² de cada pixel à tinta — a EDT da fase 1, exposta porque o Voronoi cobra
    /// **pedágio de aperto** por ela (`voronoi.rs`); recomputá-la lá seria uma 2ª porta
    /// para a mesma pergunta.
    pub ink_dist2: Vec<u32>,
}

/// Segmenta a grade em componentes estanques.
///
/// `r_px` é o raio da bola (px de buffer, o `trap_px` do balde).
#[must_use]
pub fn segment(grid: &Grid, r_px: f32) -> Segmentation {
    let (w, h) = (grid.w, grid.h);
    let n = w * h;
    let is_ink = |i: usize| grid.flags[i] & BOUNDARY != 0;

    // 1. O campo de alcance: distância² de cada pixel à tinta. UMA vez, e é a mesma função
    //    que a C1 (o balde) usa — a lei de estanqueidade não tem duas implementações.
    let sq = sq_distance_to_set(w, h, is_ink);

    // 2. O NÚCLEO: onde a bola cabe. É a erosão que despedaça a arte nos vãos estreitos —
    //    um vão de largura < 2r não admite a bola, então os dois lados deixam de estar
    //    conectados no núcleo, e é daí que nasce a costura.
    let r = f64::from(r_px.max(0.0));
    let rr = r * r;
    let mut core: Vec<bool> = (0..n)
        .map(|i| !is_ink(i) && f64::from(sq[i]) >= rr)
        .collect();
    if !core.iter().any(|&c| c) {
        // Bola grande demais para esta arte: nenhum pixel a comporta. A resposta honesta
        // não é "sem componentes" (isso apagaria o Colorize em silêncio) — é a bola de raio
        // zero, ou seja o papel inteiro, que é exatamente o comportamento de antes da
        // trapped-ball existir.
        core = (0..n).map(|i| !is_ink(i)).collect();
    }

    // 3. Componentes conexas do núcleo (4-conexo).
    let mut component = vec![NO_REGION; n];
    let mut count = 0u32;
    let mut queue: std::collections::VecDeque<u32> = std::collections::VecDeque::new();
    for start in 0..n {
        if !core[start] || component[start] != NO_REGION {
            continue;
        }
        let id = count;
        count += 1;
        component[start] = id;
        queue.push_back(start as u32);
        while let Some(cf) = queue.pop_front() {
            let c = cf as usize;
            for q in neighbours(w, h, c) {
                if core[q] && component[q] == NO_REGION {
                    component[q] = id;
                    queue.push_back(q as u32);
                }
            }
        }
    }

    // 4a. Dilata de volta **pelo PAPEL**: cada pixel de papel vai para o componente do núcleo
    //     geodesicamente mais próximo, por BFS multi-fonte (O(N), determinística: a fila
    //     nasce em ordem de índice e é FIFO).
    //
    //     ⚠️ **Só pelo papel, e isto é load-bearing.** Uma BFS que atravessasse a tinta faria
    //     um componente ENGOLIR a área do outro lado da linha sempre que essa área não tivesse
    //     núcleo próprio — e a margem da grade é exatamente esse caso quando a bola cresce.
    //     Foi o bug do smoke de 2026-07-19: uma região de 54.927 px contendo o lobo esquerdo
    //     E o lado de fora, e a cor pintando a tela inteira.
    queue.clear();
    for (i, &r) in component.iter().enumerate() {
        if r != NO_REGION {
            queue.push_back(i as u32);
        }
    }
    while let Some(cf) = queue.pop_front() {
        let c = cf as usize;
        let id = component[c];
        for q in neighbours(w, h, c) {
            if component[q] == NO_REGION && !is_ink(q) {
                component[q] = id;
                queue.push_back(q as u32);
            }
        }
    }

    // 4b. **Papel que não alcançou núcleo nenhum é uma ÁREA por direito próprio.** Uma faixa
    //     fina demais para comportar a bola continua sendo uma área separada pela linha, e a
    //     resposta honesta é dar-lhe um componente — nunca doá-la a quem está do outro lado.
    for start in 0..n {
        if is_ink(start) || component[start] != NO_REGION {
            continue;
        }
        let id = count;
        count += 1;
        component[start] = id;
        queue.push_back(start as u32);
        while let Some(cf) = queue.pop_front() {
            let c = cf as usize;
            for q in neighbours(w, h, c) {
                if component[q] == NO_REGION && !is_ink(q) {
                    component[q] = id;
                    queue.push_back(q as u32);
                }
            }
        }
    }

    // 4c. Por fim a TINTA, a partir do papel já atribuído. Ela precisa de componente para o
    //     preenchimento de um componente de uma cor só cobrir a arte até a linha; atribuí-la
    //     por último garante que ela nunca serve de ponte entre dois papéis.
    queue.clear();
    for (i, &r) in component.iter().enumerate() {
        if r != NO_REGION {
            queue.push_back(i as u32);
        }
    }
    while let Some(cf) = queue.pop_front() {
        let c = cf as usize;
        let id = component[c];
        for q in neighbours(w, h, c) {
            if component[q] == NO_REGION {
                component[q] = id;
                queue.push_back(q as u32);
            }
        }
    }

    Segmentation {
        component,
        count: count as usize,
        ink_dist2: sq,
    }
}

/// Os até 4 vizinhos de `i`, em ordem fixa (determinismo: HR-5).
#[inline]
fn neighbours(w: usize, h: usize, i: usize) -> impl Iterator<Item = usize> {
    let (x, y) = (i % w, i / w);
    [
        (x + 1 < w).then_some(i + 1),
        (x > 0).then(|| i - 1),
        (y + 1 < h).then_some(i + w),
        (y > 0).then(|| i - w),
    ]
    .into_iter()
    .flatten()
}

#[cfg(test)]
#[path = "segment_tests.rs"]
mod tests;
