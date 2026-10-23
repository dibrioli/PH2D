//! ⭐ W6g — os recursos dos cantos da borda na lista declarada (as listas, as marcas, a tabela e os contadores;
//! W6g r9: os atlas da `V` e da `V2`) — ficheiro irmão do `passlist.rs` pelo tecto de 700 LOC.

use super::Buffer;

/// ⭐ W6g: os blocos 8×8 da captura com candidato a vértice (`c3_borda_adia`), em DUAS listas que
/// alternam por quadro (a deste e a do anterior, que o `c3_cantos_limpa` repõe) — cada uma com o
/// cabeçalho do despacho INDIRETO (`[n, 9, 1]`) e depois um `u32` por bloco.
pub(crate) const NOME_LISTA_DOS_CANTOS: &str = "luz2d/buf/lista_dos_cantos";
/// ⭐ W6g r8: a marca de cada bloco da captura já listado neste quadro (um `u32` por bloco).
pub(crate) const NOME_MARCAS_DOS_BLOCOS: &str = "luz2d/buf/marcas_dos_blocos";
/// ⭐ W6g r9: a tabela dos cantos (um `u32` por bloco da captura: 0 ou o slot da `V2` + 1), a lista dos
/// listados com cunha e os contadores (`[slots da V2, recusados sem V, recusados sem V2, _]`).
pub(crate) const NOME_TABELA_DOS_CANTOS: &str = "luz2d/buf/tabela_dos_cantos";
pub(crate) const NOME_LISTA_COM_CUNHA: &str = "luz2d/buf/lista_com_cunha";
pub(crate) const NOME_CONTADORES_DOS_CANTOS: &str = "luz2d/buf/contadores_dos_cantos";
/// Os bytes de UMA das duas listas para a captura `[w, h]` (cabem todos os blocos), alinhados a 256: o
/// deslocamento do binding da 2.ª (o teto do `min_storage_buffer_offset_alignment` no WebGPU).
pub(crate) fn metade_da_lista_dos_cantos([w, h]: [u32; 2]) -> u64 {
    (4 * (3 + u64::from(w.div_ceil(8)) * u64::from(h.div_ceil(8)))).next_multiple_of(256)
}
/// ⭐ W6g r9: o atlas da `V` e da `V2` para a captura `[w, h]` — `blocos / por_blocos` slots de 8×8 (o teto,
/// o recurso é a MEMÓRIA: handoff W6g §26), em linhas de 128 slots.
pub(crate) fn atlas_dos_cantos([w, h]: [u32; 2], por_blocos: u32) -> [u32; 2] {
    let slots = (w.div_ceil(8) * h.div_ceil(8)).div_ceil(por_blocos.max(1));
    [8 * 128, 8 * slots.div_ceil(128)]
}
/// ⭐ W6g r9: a tabela VAZIA (16 B) que liga o `cantos_tabela` do `meio` onde ele não corre: a C2, a
/// holográfica e a C3 sem os cantos.
pub(crate) const NOME_TABELA_VAZIA_C2: &str = "luz2d/buf/tabela_vazia_c2";
pub(crate) const NOME_TABELA_VAZIA_HRC: &str = "luz2d/buf/tabela_vazia_hrc";
pub(crate) const NOME_TABELA_VAZIA_C3: &str = "luz2d/buf/tabela_vazia_c3";

/// As tabelas vazias da configuração (com os cantos a C3 tem a sua; com a holográfica, a dela).
pub(crate) fn tabelas_vazias(cantos: bool, holografica: bool) -> Vec<Buffer> {
    [
        (NOME_TABELA_VAZIA_C2, true),
        (NOME_TABELA_VAZIA_HRC, holografica),
        (NOME_TABELA_VAZIA_C3, !cantos),
    ]
    .into_iter()
    .filter(|&(_, sim)| sim)
    .map(|(nome, _)| Buffer {
        nome,
        bytes: 16,
        usos: wgpu::BufferUsages::STORAGE,
    })
    .collect()
}

/// Os buffers dos cantos para a captura `[w, h]` (a da borda).
pub(crate) fn buffers_dos_cantos(captura: [u32; 2]) -> [Buffer; 5] {
    use wgpu::BufferUsages as B;
    let lista = 2 * metade_da_lista_dos_cantos(captura);
    let [w, h] = captura;
    let por_bloco = 4 * u64::from(w.div_ceil(8)) * u64::from(h.div_ceil(8));
    [
        (NOME_LISTA_DOS_CANTOS, lista, B::STORAGE | B::INDIRECT),
        (NOME_LISTA_COM_CUNHA, lista, B::STORAGE | B::INDIRECT),
        (NOME_MARCAS_DOS_BLOCOS, por_bloco, B::STORAGE),
        (NOME_TABELA_DOS_CANTOS, por_bloco, B::STORAGE | B::COPY_SRC),
        (NOME_CONTADORES_DOS_CANTOS, 16, B::STORAGE | B::COPY_SRC),
    ]
    .map(|(nome, bytes, usos)| Buffer { nome, bytes, usos })
}
