//! **A pré-segmentação por regiões** (`docs/Flip/09 §8`) — o que torna o Colorize uma
//! operação de clique em vez de uma de minutos.
//!
//! # Por que ela existe (o número, não a intuição)
//!
//! O corte LazyBrush sobre a grade de PIXELS foi medido (`§7.1`): **3,3 s a 4096²**, e
//! **157 s** quando dois rabiscos se contradizem sobre a mesma linha. Sobre um grafo de
//! algumas centenas de regiões, o mesmo corte é sub-milissegundo. A redução é a receita do
//! próprio paper (Zhang et al., TVCG 2009 — *trapped-ball*) e o `§8` já a mandava.
//!
//! # A lei que mantém isto CORRETO, e não só rápido
//!
//! O peso de uma aresta entre duas regiões é a **soma do `V_pq` de pixel** sobre a fronteira
//! que elas compartilham. Logo um corte no grafo de regiões custa **exatamente** o que o
//! corte de pixels correspondente custaria: a redução é uma **restrição** do mesmo problema
//! (só admite cortes que não partem uma região), nunca uma energia diferente. É isso que o
//! gate `a_region_cut_weighs_exactly_what_the_pixel_cut_weighs` afirma.
//!
//! O preço honesto dessa restrição: **o corte só pode passar por onde há costura**. A erosão
//! por bola de raio `r` costura todo vão mais estreito que `2r` — então o Colorize atravessa
//! vãos até `2·trap_px` e **não mais**. Esse é o mesmo `trap_px` que o balde já expõe no
//! painel (*"a tinta é grossa assim"*), e não um segundo knob para a mesma pergunta.
//!
//! # Sobre-segmentar é seguro; sub-segmentar é PERDA
//!
//! Uma região a mais só dá ao corte mais liberdade — ele pode reunir as duas de graça se
//! quiser. Já duas áreas que o artista quer de cores diferentes, fundidas numa região só,
//! **nunca mais podem ser separadas**. Por isso a erosão erra para o lado fino.
//!
//! # Por que os COMPONENTES são SUBDIVIDIDOS em células (o 3º smoke)
//!
//! Um componente estanque grande é UM nó, e o corte não pode partir um nó. Então quando o
//! artista rabisca **quatro cores numa área aberta sem uma linha entre elas** — o 3º smoke de
//! 2026-07-19 — os quatro rabiscos caem no mesmo nó e a primeira reivindicação leva o desenho
//! inteiro. Mas o LazyBrush *tem* resposta para isso: sem tinta a que colar, a fronteira entre
//! duas cores cai no **MEIO**. Para o corte poder produzir esse meio, todo componente é
//! ladrilhado numa grade de células (uma sobre-segmentação, e sobre-segmentar é seguro).
//!
//! A fronteira de cor numa área aberta é **arbitrária** (nada a que colar), então a resolução
//! grossa de uma célula é honesta ali. Onde a precisão importa — na tinta — as fronteiras dos
//! COMPONENTES já são pixel-tight (seguem a linha), e o corte prefere correr por elas porque
//! atravessar tinta custa `V_INK = 0`. Célula fina perto da linha seria precisão desperdiçada.

use ph2d_flip_fill::{BOUNDARY, Grid, sq_distance_to_set};

/// O pixel não pertence a região nenhuma (a grade não tem papel alcançável).
pub const NO_REGION: u32 = u32::MAX;

/// O maior número de células por eixo (a sobre-segmentação de uma área aberta).
///
/// ⚠️ Cap, e MEDIDO no que ele governa: o custo do corte cresce com o número de nós, e a
/// resolução da fronteira de cor numa área aberta cresce com ele. `160` dá células de ~2 px a
/// 380² (o smoke, liso) e ~26 px a 4096² (grosso, mas ali a fronteira é arbitrária e o custo
/// já é dominado pela EDT). O corte sobre ≤160² = 25.600 nós é sub-10 ms.
const MAX_CELLS_PER_AXIS: usize = 160;

/// A partição da grade em regiões, mais o grafo de adjacência que o corte consome.
pub struct Segmentation {
    /// `region[i]` = a **super-região** (célula) do pixel `i`, ou [`NO_REGION`]. É o nó do
    /// grafo do corte — um componente estanque subdividido em células (ver o topo do módulo).
    pub region: Vec<u32>,
    /// `component[i]` = o **componente estanque** do pixel `i` (a topologia da TINTA, antes da
    /// subdivisão em células), ou [`NO_REGION`]. É o que responde *"a linha separa estes dois
    /// pontos?"* — a pergunta que a célula, sendo uma grade arbitrária, não responde.
    pub component: Vec<u32>,
    /// Quantas super-regiões (os ids de `region` são `0..count`).
    pub count: usize,
    /// Arestas `(a, b, peso)` com `a < b`, **coalescidas**, peso > 0.
    ///
    /// Arestas de peso **zero** são omitidas de propósito, e isso é EXATO, não uma
    /// aproximação: um arco de capacidade 0 nunca é crescido (`tree_cap <= 0`), nunca é
    /// atravessado pelo `source_side` (`res > 0`) e contribui 0 para qualquer corte — e um
    /// par 0/0 não pode ganhar residual, porque empurrar fluxo por ele exigiria que ele já
    /// tivesse residual.
    pub edges: Vec<(u32, u32, i32)>,
}

/// Segmenta a grade em regiões estanques e devolve o grafo de adjacência ponderado.
///
/// `r_px` é o raio da bola (px de buffer, o `trap_px` do balde). `v_white`/`v_ink` são a
/// MESMA lei de `V_pq` do corte de pixels — passadas, não redecididas aqui.
#[must_use]
pub fn segment(grid: &Grid, r_px: f32, v_white: i32, v_ink: i32) -> Segmentation {
    let (w, h) = (grid.w, grid.h);
    let n = w * h;
    let is_ink = |i: usize| grid.flags[i] & BOUNDARY != 0;

    // 1. O campo de alcance: distância² de cada pixel à tinta. UMA vez, e é a mesma função
    //    que a C1 (o balde) usa — a lei de estanqueidade não tem duas implementações.
    let sq = sq_distance_to_set(w, h, is_ink);

    // 2. O NÚCLEO: onde a bola cabe. É a erosão que despedaça a arte nos vãos estreitos —
    //    um vão de largura < 2r não admite a bola, então os dois lados deixam de estar
    //    conectados no núcleo, e é daí que nasce a costura por onde o corte pode passar.
    let r = f64::from(r_px.max(0.0));
    let rr = r * r;
    let mut core: Vec<bool> = (0..n)
        .map(|i| !is_ink(i) && f64::from(sq[i]) >= rr)
        .collect();
    if !core.iter().any(|&c| c) {
        // Bola grande demais para esta arte: nenhum pixel a comporta. A resposta honesta
        // não é "sem regiões" (isso apagaria o Colorize em silêncio) — é a bola de raio
        // zero, ou seja o papel inteiro, que é exatamente o comportamento de antes da
        // trapped-ball existir.
        core = (0..n).map(|i| !is_ink(i)).collect();
    }

    // 3. Componentes conexas do núcleo (4-conexo) — cada uma é uma região.
    let mut region = vec![NO_REGION; n];
    let mut count = 0u32;
    let mut queue: std::collections::VecDeque<u32> = std::collections::VecDeque::new();
    for start in 0..n {
        if !core[start] || region[start] != NO_REGION {
            continue;
        }
        let id = count;
        count += 1;
        region[start] = id;
        queue.push_back(start as u32);
        while let Some(cf) = queue.pop_front() {
            let c = cf as usize;
            for q in neighbours(w, h, c) {
                if core[q] && region[q] == NO_REGION {
                    region[q] = id;
                    queue.push_back(q as u32);
                }
            }
        }
    }

    // 4a. Dilata de volta **pelo PAPEL**: cada pixel de papel vai para a região do núcleo
    //     geodesicamente mais próximo, por BFS multi-fonte (O(N), determinística: a fila
    //     nasce em ordem de índice e é FIFO).
    //
    //     ⚠️ **Só pelo papel, e isto é load-bearing.** Uma BFS que atravessasse a tinta faria
    //     uma região ENGOLIR a área do outro lado da linha sempre que essa área não tivesse
    //     núcleo próprio — e a margem da grade é exatamente esse caso quando a bola cresce.
    //     Foi o bug do smoke de 2026-07-19: uma região de 54.927 px contendo o lobo esquerdo
    //     E o lado de fora, e a cor pintando a tela inteira.
    queue.clear();
    for (i, &r) in region.iter().enumerate() {
        if r != NO_REGION {
            queue.push_back(i as u32);
        }
    }
    while let Some(cf) = queue.pop_front() {
        let c = cf as usize;
        let id = region[c];
        for q in neighbours(w, h, c) {
            if region[q] == NO_REGION && !is_ink(q) {
                region[q] = id;
                queue.push_back(q as u32);
            }
        }
    }

    // 4b. **Papel que não alcançou núcleo nenhum é uma ÁREA por direito próprio.** Uma faixa
    //     fina demais para comportar a bola continua sendo uma área separada pela linha, e a
    //     resposta honesta é dar-lhe uma região — nunca doá-la a quem está do outro lado.
    for start in 0..n {
        if is_ink(start) || region[start] != NO_REGION {
            continue;
        }
        let id = count;
        count += 1;
        region[start] = id;
        queue.push_back(start as u32);
        while let Some(cf) = queue.pop_front() {
            let c = cf as usize;
            for q in neighbours(w, h, c) {
                if region[q] == NO_REGION && !is_ink(q) {
                    region[q] = id;
                    queue.push_back(q as u32);
                }
            }
        }
    }

    // 4c. Por fim a TINTA, a partir do papel já atribuído. Ela precisa de região para que a
    //     adjacência *através da linha* exista no grafo (é a aresta barata que o corte tem de
    //     poder cortar); atribuí-la por último garante que ela nunca serve de ponte.
    queue.clear();
    for (i, &r) in region.iter().enumerate() {
        if r != NO_REGION {
            queue.push_back(i as u32);
        }
    }
    while let Some(cf) = queue.pop_front() {
        let c = cf as usize;
        let id = region[c];
        for q in neighbours(w, h, c) {
            if region[q] == NO_REGION {
                region[q] = id;
                queue.push_back(q as u32);
            }
        }
    }

    // 4d. **Subdivide cada componente em CÉLULAS** (ver o topo do módulo). O `component`
    //     guarda a topologia da tinta; o `region` (super-região) é `component × célula`, e é
    //     ele que o corte consome — assim uma área aberta é ladrilhada e o corte pode dividir
    //     duas cores pelo meio, em vez de a 1ª reivindicação levar tudo.
    let component = region.clone();
    let cell = (w.max(h)).div_ceil(MAX_CELLS_PER_AXIS).max(1);
    let cols = w.div_ceil(cell) as u64;
    let rows = h.div_ceil(cell) as u64;
    let cells_per_component = cols * rows;
    // Densifica `(componente, célula)` → id denso, em ordem de índice (determinístico).
    let mut dense: std::collections::BTreeMap<u64, u32> = std::collections::BTreeMap::new();
    let mut super_count = 0u32;
    for (i, r) in region.iter_mut().enumerate() {
        if *r == NO_REGION {
            continue;
        }
        let (x, y) = (i % w, i / w);
        let cell_id = (y / cell) as u64 * cols + (x / cell) as u64;
        let key = u64::from(*r) * cells_per_component + cell_id;
        let id = *dense.entry(key).or_insert_with(|| {
            let id = super_count;
            super_count += 1;
            id
        });
        *r = id;
    }
    count = super_count;

    // 5. O grafo: cada par 4-adjacente que atravessa uma fronteira de região contribui o
    //    `V_pq` DAQUELE par. A soma é o peso da aresta ⇒ o corte de regiões pesa exatamente
    //    o corte de pixels correspondente.
    let mut raw: Vec<(u32, u32, i32)> = Vec::new();
    for i in 0..n {
        let ri = region[i];
        if ri == NO_REGION {
            continue;
        }
        for q in [
            (i % w + 1 < w).then_some(i + 1),
            (i / w + 1 < h).then_some(i + w),
        ]
        .into_iter()
        .flatten()
        {
            let rq = region[q];
            if rq == NO_REGION || rq == ri {
                continue;
            }
            let c = if is_ink(i) || is_ink(q) {
                v_ink
            } else {
                v_white
            };
            raw.push((ri.min(rq), ri.max(rq), c));
        }
    }
    raw.sort_unstable();
    let mut edges: Vec<(u32, u32, i32)> = Vec::new();
    for (a, b, c) in raw {
        match edges.last_mut() {
            Some(last) if last.0 == a && last.1 == b => last.2 = last.2.saturating_add(c),
            _ => edges.push((a, b, c)),
        }
    }
    edges.retain(|&(_, _, c)| c > 0);

    Segmentation {
        region,
        component,
        count: count as usize,
        edges,
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
