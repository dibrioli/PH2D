//! **A pegada na placa por RÓTULO** (F36 §2, `docs/UI_New_and_Simple/F36_plano.md`): o relatório do
//! alocador do wgpu (Vulkan/DX12: `gpu-allocator`, cada alocação leva o rótulo do recurso) somado por
//! rótulo. Diagnóstico opt-in: `PH2D_GPU_PEGADA=<s>[,<s>…]` imprime a tabela aos `s` segundos do
//! arranque, chamada uma vez por quadro de [`crate::SurfaceContext::acquire_frame`].

use std::collections::BTreeMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Uma linha da pegada: o rótulo, os bytes somados e quantas alocações o levam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linha {
    pub rotulo: String,
    pub bytes: u64,
    pub alocacoes: u32,
}

/// Soma alocações `(rótulo, bytes)` por rótulo, a maior primeiro (empate: o rótulo).
#[must_use]
pub fn agrupa(alocacoes: impl IntoIterator<Item = (String, u64)>) -> Vec<Linha> {
    let mut por: BTreeMap<String, (u64, u32)> = BTreeMap::new();
    for (rotulo, bytes) in alocacoes {
        let e = por.entry(rotulo).or_default();
        e.0 += bytes;
        e.1 += 1;
    }
    let mut v: Vec<Linha> = por
        .into_iter()
        .map(|(rotulo, (bytes, alocacoes))| Linha {
            rotulo,
            bytes,
            alocacoes,
        })
        .collect();
    v.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.rotulo.cmp(&b.rotulo)));
    v
}

/// A pegada do `device` por rótulo, e os totais `(alocado, reservado)` do alocador. `None` quando o
/// backend não tem relatório (GL, Metal).
#[must_use]
pub fn por_rotulo(device: &wgpu::Device) -> Option<(u64, u64, Vec<Linha>)> {
    let r = device.generate_allocator_report()?;
    let linhas = agrupa(r.allocations.into_iter().map(|a| (a.name, a.size)));
    Some((r.total_allocated_bytes, r.total_reserved_bytes, linhas))
}

/// Imprime a pegada nos instantes pedidos por `PH2D_GPU_PEGADA` (segundos desde a 1.ª chamada).
pub fn relata_se_pedida(device: &wgpu::Device) {
    static PEDIDOS: OnceLock<(Instant, Vec<u64>)> = OnceLock::new();
    static FEITOS: AtomicUsize = AtomicUsize::new(0);
    let (origem, pedidos) = PEDIDOS.get_or_init(|| {
        let v = std::env::var("PH2D_GPU_PEGADA")
            .ok()
            .map(|s| s.split(',').filter_map(|t| t.trim().parse().ok()).collect())
            .unwrap_or_default();
        (Instant::now(), v)
    });
    let feitos = FEITOS.load(Ordering::Relaxed);
    let Some(&s) = pedidos.get(feitos) else {
        return;
    };
    if origem.elapsed().as_secs() < s {
        return;
    }
    FEITOS.store(feitos + 1, Ordering::Relaxed);
    let Some((alocado, reservado, linhas)) = por_rotulo(device) else {
        eprintln!("[gpu-pegada] {s} s: este backend não dá relatório do alocador");
        return;
    };
    const MIB: f64 = 1024.0 * 1024.0;
    eprintln!(
        "[gpu-pegada] {s} s: alocado {:.1} MiB · reservado {:.1} MiB",
        alocado as f64 / MIB,
        reservado as f64 / MIB
    );
    for l in &linhas {
        eprintln!(
            "[gpu-pegada]   {:>9.2} MiB  {:>4}×  {}",
            l.bytes as f64 / MIB,
            l.alocacoes,
            l.rotulo
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_footprint_sums_by_label_largest_first() {
        let v = agrupa([
            ("b".to_string(), 10),
            ("a".to_string(), 5),
            ("b".to_string(), 1),
            ("c".to_string(), 5),
        ]);
        let resumo: Vec<_> = v
            .iter()
            .map(|l| (l.rotulo.as_str(), l.bytes, l.alocacoes))
            .collect();
        assert_eq!(resumo, [("b", 11, 2), ("a", 5, 1), ("c", 5, 1)]);
    }
}
