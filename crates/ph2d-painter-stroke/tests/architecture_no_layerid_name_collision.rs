//! Gate anti-colisão de nomes LayerId/LayerStack (Track B, 2026-06-16).
//!
//! Havia DOIS `pub struct LayerId`/`LayerStack` no workspace com o MESMO nome:
//! o runtime `ph2d_tool_painter::layers::LayerId(u64)` (modelo canônico) e o
//! DTO de savefile `ph2d_painter_stroke::device::LayerId(u32)`. Nomes idênticos
//! em crates diferentes confundem ("qual LayerStack?") e foi a origem
//! documentada de um modelo paralelo (HANDOFF arquivado). O split runtime-vs-DTO
//! é legítimo (Cerca de Chesterton — decisão ratificada Coord 2026-05-31), então
//! NÃO colapsamos (quebraria o savefile congelado); DESAMBIGUAMOS por nome: o
//! lado savefile vira `PersistLayerId`/`PersistLayerStack`.
//!
//! Este gate trava a desambiguação: o crate do savefile NÃO pode redefinir os
//! nomes não-prefixados (senão a colisão volta). Lê o fonte (não compila).

use std::fs;

fn device_src() -> String {
    let path = format!("{}/src/device.rs", env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("não consegui ler {path}: {e}"))
}

#[test]
fn savefile_crate_uses_persist_prefixed_names() {
    let src = device_src();
    assert!(
        src.contains("pub struct PersistLayerId"),
        "device.rs deve definir PersistLayerId (DTO do savefile, u32)"
    );
    assert!(
        src.contains("pub struct PersistLayerStack"),
        "device.rs deve definir PersistLayerStack (stack do savefile)"
    );
}

#[test]
fn savefile_crate_does_not_redefine_the_unprefixed_runtime_names() {
    let src = device_src();
    // `pub struct LayerId(` e `pub struct LayerStack ` (não-prefixados) pertencem
    // AO runtime (`ph2d_tool_painter::layers`). Redefini-los aqui recria a colisão.
    assert!(
        !src.contains("pub struct LayerId("),
        "colisão: device.rs redefine `pub struct LayerId` — use PersistLayerId"
    );
    assert!(
        !src.contains("pub struct LayerStack "),
        "colisão: device.rs redefine `pub struct LayerStack` — use PersistLayerStack"
    );
}
