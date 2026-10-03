//! ⭐ **A LEITURA DE VOLTA** — os bytes que a placa devolveu, no vocabulário do G-buffer.
//!
//! ⚠️ **Ela saiu do [`crate::trace`] em 2026-09-15 por TETO DE LINHAS**, e a fronteira é real: aqui
//! não há decisão nenhuma, só desempacotamento. *O que decide o que é lido fica lá; o que sabe a
//! forma dos bytes fica aqui.*

use crate::trace::DeviceEdge;

/// O que uma leitura devolve: `t`, a normal e as bordas.
pub(super) type Lido = (Vec<f32>, Vec<[f32; 3]>, Vec<DeviceEdge>);

/// O desempacotamento, na ordem em que a marcha escreveu.
pub(super) fn lida(d_centro: &[u8], usadas: u64, d_borda: Option<&[u8]>) -> Lido {
    let f4 = |q: &[u8; 16], o: usize| f32::from_le_bytes([q[o], q[o + 1], q[o + 2], q[o + 3]]);
    let mut t = Vec::with_capacity(d_centro.len() / 16);
    let mut normal = Vec::with_capacity(t.capacity());
    for q in d_centro.as_chunks::<16>().0 {
        t.push(f4(q, 0));
        normal.push([f4(q, 4), f4(q, 8), f4(q, 12)]);
    }
    // ⭐ O passo do `luz` é `1 + n_lamps + 3`: o céu à frente, as lâmpadas a seguir, e os TRÊS do
    // RICOCHETE no fim (`docs/Render3d/08`).
    //
    // ⚠️⚠️ **As lâmpadas CONTAM-SE tirando os três do fim, e não são `passo - 1`** — era assim
    // antes do ricochete, e uma leitura que não descontasse leria os canais dele como três
    // lâmpadas fantasma, **em silêncio**.
    let vazio: [u8; 0] = [];
    let quads = d_borda.unwrap_or(&vazio).as_chunks::<16>().0;
    #[allow(clippy::cast_possible_truncation)]
    let usadas = usadas as usize;
    let mut edges = Vec::with_capacity(usadas);
    for slot in 0..usadas.min(quads.len() / 5) {
        let cabeca = &quads[slot * 5];
        let pixel = u32::from_le_bytes([cabeca[0], cabeca[1], cabeca[2], cabeca[3]]);
        let mut hit = [false; 4];
        let mut nrm = [[0.0f32; 3]; 4];
        for j in 0..4 {
            let q = &quads[slot * 5 + 1 + j];
            hit[j] = f4(q, 0) >= 0.0;
            nrm[j] = [f4(q, 4), f4(q, 8), f4(q, 12)];
        }
        edges.push(DeviceEdge {
            pixel,
            hit,
            normal: nrm,
        });
    }
    // ⚠️ **Ordenada por pixel**, como a `Gbuffer::edges` da CPU promete — a ordem da lista aqui é
    // a de chegada dos workgroups, que é arbitrária.
    edges.sort_by_key(|e| e.pixel);
    (t, normal, edges)
}
