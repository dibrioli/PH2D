//! O **orçamento de VRAM da cache de camadas** (por tipo de device) e o número de fatias que cabe nele,
//! irmão de `layer_compositor/mod.rs` por tecto de LOC.
//!
//! Corte mecânico: o bloco saiu inteiro, verbatim.

/// VRAM the layer texture-array cache may hold on a **shared-memory** device
/// (integrated GPU, software adapter): 2 GiB — 64 slices at 3840×2160, 32 at 4096².
/// Discrete cards get more — see [`layer_cache_budget`], which is what the compositor asks.
///
/// **The resource is shared RAM held by this cache**, and the number is measured (B42.1,
/// Radeon integrada RADV Raphael, 3840×2160, full recompose, minimum; GPU:
/// `mede_as_camadas_4k_alem_do_orcamento`, CPU: `mede_o_caminho_da_cpu_alem_do_orcamento`,
/// load ~9–12 on 32 threads):
///
/// | layers | MiB | GPU ms | CPU ms (the fallback) |
/// |---|---|---|---|
/// | 16 | 506 | 37,6 | 121 |
/// | 17 | 538 | — | 133 |
/// | 32 | 1 012 | 107 | 159 |
/// | 33 | 1 044 | 112 | 166 |
/// | 48 | 1 518 | 158 | 468 |
/// | **64** | **2 025** | **180** | **720** |
///
/// The card held all 64 with zero refusals; the old 512 MiB sent the 17th layer to a CPU path
/// 1,5–4× slower. Above the budget the allocation is fallible (`ensure_array` under
/// `sob_escopo_de_memoria`): a refusal lowers the budget to what the card gave.
pub const LAYER_CACHE_BUDGET_SHARED_BYTES: u64 = 2048 * 1024 * 1024;

/// VRAM the layer cache may hold on a **discrete** GPU: 4 GiB — 64 slices at
/// 4096², 129 at 3840×2160 (the 64-slice row below, held without complaint).
///
/// **The resource is VRAM held by this cache**, and the number is measured, not
/// chosen (`measure_how_far_the_array_actually_goes`, RTX 5060 Ti 16 GB, full
/// recompose of a 4096² canvas):
///
/// | slices | VRAM | GPU floor | CPU fallback for the same document |
/// |---|---|---|---|
/// | 8 | 512 MiB | 2,06 ms | ~149 ms |
/// | **16** | **1 GiB** | **3,86 ms** | **~254 ms** |
/// | 32 | 2 GiB | 7,66 ms | — |
/// | 48 | 3 GiB | 16,13 ms | — |
/// | 64 | 4 GiB | 14,82 ms | — |
///
/// The card held 4 GiB without complaining; past ~32 slices a full recompose no longer fits a
/// 60 Hz frame, but it stays ~17× under the CPU fallback. wgpu exposes no VRAM query
/// (`AdapterInfo` carries `device_type` only), so the bound against a smaller card is the refusal
/// itself, not a guess.
///
/// The prerequisite for going past 1 GiB was **a fallible array allocation** so a card that
/// cannot hold the ask degrades to fewer slices instead of losing the device; B42.1 made it
/// (`ensure_array` under `sob_escopo_de_memoria`), so the budget is now the measured 4 GiB row.
pub const LAYER_CACHE_BUDGET_DISCRETE_BYTES: u64 = 4096 * 1024 * 1024;

/// The whole point of the split: a discrete card gets more. Asserted at COMPILE
/// time, because it is a fact about two literals — a runtime test of it would
/// only ever tell you what the compiler already knew.
const _: () = assert!(LAYER_CACHE_BUDGET_DISCRETE_BYTES > LAYER_CACHE_BUDGET_SHARED_BYTES);

/// The cache's VRAM budget on `device_type` — the one property wgpu exposes that
/// says anything about where the memory comes from.
///
/// A discrete card has VRAM of its own; an integrated one spends the SYSTEM's
/// RAM, where a gigabyte of layer cache competes with the document, the undo
/// queue and the OS. Same cache, different resource, different number.
#[must_use]
pub fn layer_cache_budget(device_type: wgpu::DeviceType) -> u64 {
    match device_type {
        wgpu::DeviceType::DiscreteGpu => LAYER_CACHE_BUDGET_DISCRETE_BYTES,
        // Integrated / virtual / CPU / unknown all share memory with the host.
        _ => LAYER_CACHE_BUDGET_SHARED_BYTES,
    }
}

/// Largest layer stack the contract allows — spec §2.2 `HARD_CAP_LAYERS = 999`
/// (mirrors Procreate), matching `ph2d_tool_painter::layers::HARD_CAP_LAYERS`
/// and the savefile `MAX_LAYERS = 1000` (999 + overflow headroom). Independent
/// of the per-budget cap — the absolute ceiling the cache will never exceed.
pub const HARD_CAP_LAYERS: u32 = 999;

/// Maximum cached slices for a `width × height` canvas under `budget` bytes.
/// `0` for a degenerate (zero-area) canvas. The compositor additionally clamps
/// to the device's `max_texture_array_layers`.
#[must_use]
pub fn max_layers_for_budget(width: u32, height: u32, budget: u64) -> u32 {
    let per_slice = (width as u64) * (height as u64) * 4;
    if per_slice == 0 {
        return 0;
    }
    (budget / per_slice).min(HARD_CAP_LAYERS as u64) as u32
}
