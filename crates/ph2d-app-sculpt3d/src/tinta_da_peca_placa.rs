//! ⭐⭐⭐ **O TECTO DA PLACA da tinta fina** — quantas amostras o dispositivo
//! aceita num plano, e o maior degrau que cabe nesse orçamento. Filho de
//! `tinta_da_peca.rs`, que é a lei do plano; a porta que as junta é a
//! [`super::garante_no_orcamento`].

use ph2d_mesh::Mesh;

/// ⭐⭐⭐ **Quantas amostras a PLACA aceita num plano** — o tecto real da tinta
/// fina, e ele é do DEVICE e não um nível escolhido (report do dono,
/// 2026-09-24: *«a resolução de 16x não chega para o painter»*).
///
/// O plano vive num ÚNICO buffer de armazenamento, `12` bytes por amostra
/// (`f32 × 3`, sem folga — o `tinta_gpu` aloca à medida), logo o tecto é o
/// menor de `max_storage_buffer_binding_size` e `max_buffer_size`, que a
/// `ph2d-gpu` sobe ao que o adaptador anuncia. ⭐ Nesta máquina (RTX 5060 Ti,
/// `16 GB`) são `4 GiB` ⇒ `357 M` amostras: na peça da lição o `256x` cabe
/// (`48 M`), na peça de fábrica o `32x` cabe (`101 M`) e o `64x` não (`403 M`).
#[must_use]
pub(crate) fn orcamento_da_placa(lim: &wgpu::Limits) -> u64 {
    let bytes = lim.max_storage_buffer_binding_size.min(lim.max_buffer_size);
    bytes / size_of::<[f32; 3]>() as u64
}

/// ⭐⭐ **O degrau que CABE** — o maior `≤ k` cujo plano, NESTA malha, tem no
/// máximo `orcamento` amostras. `0` quer dizer *«nem o `2x` cabe»*.
///
/// ⚠️ Ela conta sem alocar ([`ph2d_mesh_colors::Topologia::amostras_ao_nivel`]).
/// ⚠️⚠️ **E reaproveita a topologia do plano que a peça JÁ tem** (`ja`, se ele
/// ainda descreve a malha): numa peça que não está activa e ficou num degrau
/// descido, o pedido da cena continua acima do dela e esta conta corre em TODO
/// quadro — com a topologia reaproveitada ela é um laço sobre as faces, e sem
/// ela seria o mapa das arestas a ser refeito a `60 Hz`.
#[must_use]
pub(crate) fn degrau_que_cabe(
    mesh: &Mesh,
    ja: Option<&ph2d_mesh_colors::Topologia>,
    k: u8,
    orcamento: u64,
) -> u8 {
    let feita;
    let topo = match ja {
        Some(t) => t,
        None => {
            let faces = mesh.faces().iter().map(ph2d_mesh::Face::verts);
            feita = ph2d_mesh_colors::Topologia::nova(mesh.vert_count(), faces, 0);
            &feita
        }
    };
    (1..=k)
        .rev()
        .find(|&j| topo.amostras_ao_nivel(j) <= orcamento)
        .unwrap_or(0)
}
