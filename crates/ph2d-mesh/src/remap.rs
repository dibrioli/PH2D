//! **COMO OS ÍNDICES SE MOVERAM** — o canal que uma topologia que ENCOLHE deve a
//! todo mundo que guarda um índice.
//!
//! O refino só APENDA, e é por isso que ele não precisa deste módulo: um índice
//! guardado antes continua apontando para a mesma coisa depois. Um colapso
//! **apaga**, e apagar de um `Vec` sem deixar buraco é trocar com o último — o
//! que renumera exatamente uma coisa por remoção.
//!
//! ⚠️ **A alternativa foi considerada e recusada: deixar o buraco.** Um slot
//! morto de vértice envenena `is_border` (o par *nº de faces × nº de vizinhos*
//! deixa de bater), faz `vert_count`/`face_count` MENTIREM para o artista, e
//! obriga todo laço `for v in 0..n` do crate a aprender que alguns slots não são
//! nada. Trocar com o último mantém o invariante que todo consumidor já assume —
//! *todo slot é vivo* — e paga por isso com um canal explícito, que é este.

/// A renumeração que uma compactação produziu.
///
/// ⚠️ **É uma SEQUÊNCIA, não uma tabela, e aplicá-la fora de ordem dá outra
/// resposta.** Uma coisa pode mudar de casa duas vezes: com os mortos `[8, 3]`
/// numa lista de 10, a face 9 vai para 8 (que acabou de vagar) e depois de 8
/// para 3. Uma tabela `de → para` guardaria `9 → 8` e `8 → 3` como fatos
/// independentes, e quem os lesse em qualquer ordem acertaria um e erraria o
/// outro.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Remap {
    /// `(de, para)` de cada face que mudou de casa, na ordem em que mudou.
    pub face_moves: Vec<(u32, u32)>,
    /// Quantas faces sobraram.
    pub faces: usize,
    /// `(de, para)` de cada vértice que mudou de casa, na ordem em que mudou.
    pub vert_moves: Vec<(u32, u32)>,
    /// Quantos vértices sobraram.
    pub verts: usize,
}

impl Remap {
    /// **Deriva o plano de compactação** a partir do que morre e de quantos
    /// havia. É uma função PURA: ela não toca em nada, e é isso que permite ao
    /// chamador consultá-la antes de mexer numa única estrutura.
    ///
    /// ⚠️ **Os mortos são percorridos do MAIOR para o menor, e a ordem é o que
    /// torna o parceiro de troca sempre vivo:** quando `d` é processado, todo
    /// morto acima dele já foi removido, então `len - 1` não pode ser um deles.
    /// Na ordem crescente o último poderia estar condenado, e a troca instalaria
    /// um cadáver no lugar de outro.
    #[must_use]
    pub fn plan(dead_faces: &[u32], faces: usize, dead_verts: &[u32], verts: usize) -> Self {
        Self {
            face_moves: moves(dead_faces, faces),
            faces: faces - dead_faces.len(),
            vert_moves: moves(dead_verts, verts),
            verts: verts - dead_verts.len(),
        }
    }

    /// **Encadeia a compactação seguinte.** Rodadas de um mesmo dab produzem uma
    /// sequência só, e concatenar é exatamente certo porque a lista já é
    /// aplicada em ordem — a última contagem é a que vale.
    pub fn then(&mut self, next: Self) {
        self.face_moves.extend(next.face_moves);
        self.vert_moves.extend(next.vert_moves);
        self.faces = next.faces;
        self.verts = next.verts;
    }

    /// **ONDE CADA SOBREVIVENTE FOI PARAR** — os pares `(original, final)`, sem
    /// as casas intermediárias.
    ///
    /// ⚠️ **Existe porque um destino pode virar uma origem, e quem lê o CONTEÚDO
    /// da lista final não pode ver o caminho.** Com os mortos `[3, 8]` numa lista
    /// de 10 a sequência é `(9→8), (8→3)`: o slot 8 é uma casa de passagem que a
    /// lista compactada — de tamanho 8 — nem tem. Aplicar a sequência CRUA a
    /// vetores paralelos funciona (eles ainda têm o slot 8 quando a troca
    /// acontece); consultar `final[8]` não.
    ///
    /// Quem aplica em ordem sobre a lista inteira usa [`Self::face_moves`]; quem
    /// precisa do par original↔final usa isto.
    #[must_use]
    pub fn net_face_moves(&self, old_len: usize) -> Vec<(u32, u32)> {
        net(&self.face_moves, old_len, self.faces)
    }

    /// Nenhum índice mudou de casa.
    ///
    /// ⚠️ **Não é *"nada morreu"***, e a diferença morde: remover o ÚLTIMO item
    /// não move ninguém, então a contagem cai e esta função continua dizendo
    /// `true`. Quem quer saber se a lista encolheu compara a contagem.
    #[must_use]
    pub fn moves_nothing(&self) -> bool {
        self.face_moves.is_empty() && self.vert_moves.is_empty()
    }
}

/// Colapsa a sequência de trocas nos pares `(original, final)`.
///
/// O `origin[slot]` responde *"quem estava aqui no começo?"*, e ele custa um
/// `u32` por item — a mesma ordem que o `pending` do grafo de arestas e o `mark`
/// da porta que encolhe já pagam por passe.
fn net(moves: &[(u32, u32)], old_len: usize, new_len: usize) -> Vec<(u32, u32)> {
    if moves.is_empty() {
        return Vec::new();
    }
    let mut origin: Vec<u32> = (0..u32::try_from(old_len).unwrap_or(u32::MAX)).collect();
    for &(from, to) in moves {
        origin[to as usize] = origin[from as usize];
    }
    (0..new_len)
        .filter_map(|slot| {
            let orig = origin[slot];
            let slot = u32::try_from(slot).unwrap_or(u32::MAX);
            (orig != slot).then_some((orig, slot))
        })
        .collect()
}

/// A sequência de trocas que remove `dead` de uma lista de `len` itens.
///
/// ⚠️ **`dead` TEM de estar ordenado e sem repetição** — o chamador é quem sabe
/// como o conjunto foi construído, e ordenar aqui esconderia de onde ele veio.
/// Em `debug` a violação é um pânico; em `release` a lista que sai simplesmente
/// não descreve a compactação, e é por isso que o `debug_assert` existe.
fn moves(dead: &[u32], len: usize) -> Vec<(u32, u32)> {
    debug_assert!(
        dead.windows(2).all(|w| w[0] < w[1]),
        "os mortos chegam ordenados e sem repetição"
    );
    let mut out = Vec::with_capacity(dead.len());
    let mut end = len;
    for &d in dead.iter().rev() {
        end -= 1;
        let d = d as usize;
        debug_assert!(d <= end, "um morto fora da lista");
        if d != end {
            out.push((u32::try_from(end).unwrap_or(u32::MAX), d as u32));
        }
    }
    out
}

#[cfg(test)]
#[path = "remap_tests.rs"]
mod tests;
