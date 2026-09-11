//! ABI gate for the `RenderInstance` (Sprite_projeto §10.5 / §10.10 / §1.7) — a
//! field add/remove/reorder must not silently change the upload bandwidth nor desync
//! the `vertex_attr_array!` offsets from the struct.
//!
//! ⚠️ **Este ficheiro deixou de repetir o TOTAL, e a razão é uma reprovação medida
//! (2026-08-25).** Ele e o `architecture_sprite_inspector_surface` pinavam o MESMO
//! número — `184` — em dois sítios, e a `ADR-0070-amendment-9` (`+sub_order`) atualizou
//! um só: o outro ficou vermelho depois de a wave inteira estar verde nas suas próprias
//! corridas. *Uma lei escrita em dois sítios ainda não é uma lei.*
//!
//! ⇒ O **total** tem um dono, e é o
//! [`architecture_sprite_inspector_surface::render_instance_pod_size_capped`], ao lado da
//! destruturação exaustiva que obriga a contagem de campos e o tamanho a moverem-se em
//! lockstep. Aqui fica o que aquele NÃO diz: a **partição** entre a metade lida pela GPU
//! e a cauda CPU-only, e o alinhamento.

use ph2d_render::RenderInstance;

/// **A metade que a GPU lê tem de continuar contígua e do tamanho que o vertex layout
/// assume**, e o resto é cauda CPU-only.
///
/// ⚠️ Esta é a afirmação que o total não faz: um campo novo de 4 bytes inserido ANTES do
/// `texture_id` mantém o total certo se outro sair, e desloca todo `@location` — o que o
/// `vertex_attr_offsets_match_struct` apanha por offset e este apanha por TAMANHO.
///
/// O número da GPU é **derivado do próprio array de atributos** (o último offset + o
/// tamanho dele), nunca escrito ao lado: escrevê-lo seria a terceira cópia do mesmo facto.
#[test]
fn the_gpu_half_is_contiguous_and_the_rest_is_the_cpu_tail() {
    let attrs = RenderInstance::VERTEX_ATTRIBUTES;
    let last = attrs.last().expect("pelo menos um atributo");
    // `uv_xform` é `Float32x4` = 16 bytes; o fim dele é o fim da metade da GPU.
    let gpu_bytes = (last.offset + 16) as usize;
    assert_eq!(
        gpu_bytes, 164,
        "a metade lida pela GPU mudou de tamanho — o vertex layout dessincronizou"
    );
    let total = std::mem::size_of::<RenderInstance>();
    assert!(
        total > gpu_bytes,
        "a cauda CPU-only desapareceu: total {total}, metade da GPU {gpu_bytes}"
    );
    // A cauda é feita de palavras de 4 bytes (`texture_id`/`z_order`/`sampling`/
    // `clip_group`/`clip_meta`/`sub_order`), então ela é sempre um múltiplo de 4.
    assert_eq!(
        (total - gpu_bytes) % 4,
        0,
        "a cauda CPU-only tem de ser feita de palavras de 4 bytes"
    );
}

#[test]
fn render_instance_is_four_byte_aligned() {
    // The ABI is documented as 4-byte aligned with no tail padding
    // (§1.7). `bytemuck::Pod` already forbids padding bytes; this pins
    // the alignment the WGSL instance step-mode layout assumes.
    assert_eq!(
        std::mem::align_of::<RenderInstance>(),
        4,
        "RenderInstance must stay 4-byte aligned (all fields f32/u32-grained)"
    );
}
