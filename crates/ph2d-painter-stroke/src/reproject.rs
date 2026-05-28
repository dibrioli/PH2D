//! Reproject canvas para resolução nova (W12). Stub T1.8 — declara
//! contrato + caps; implementação full é W12.
//!
//! Cap arch-gate:
//! - [`ReprojectMode`] ≤ 4 variants

use serde::{Deserialize, Serialize};

use crate::persistence::PaintProject;

/// Mode de reproject — bit-identical CPU (det) vs aproximado GPU.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ReprojectMode {
    /// CPU fallback bit-identical (HR-5 `det-painter` feature). ~5 strokes/s.
    /// Gate cross-OS replay verifica blake3 idêntico.
    #[default]
    Det,
    /// GPU stamp pipeline reaplicado. ~50 strokes/s. Aproximado (estética
    /// idêntica; pode diferir em alguns ULPs cross-backend).
    Gpu,
    // === 2 slots de headroom ===
    // - Hybrid (det pra strokes < N samples, gpu pra resto)
    // - Streaming (incremental on-demand pra preview live)
}

/// Eventos de progresso do reproject. Stub W12 expandido.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ProgressEvent {
    /// Iniciou — total de strokes a processar.
    Started { total: u64 },
    /// Stroke N completou.
    StrokeDone { seq: u64 },
    /// Finalizado.
    Finished,
}

/// Erros possíveis do reproject. ≤ 8 variants (não-gateado mas reservado).
///
/// **Audit T1.8 L3-G9:** `#[non_exhaustive]` permite W12+ adicionar
/// variants (`PartialFailure { successful_strokes, total }`,
/// `OutOfMemory`, etc.) sem semver break em downstream.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ReprojectError {
    /// Target dimension fora do range suportado (1..16384 × 1..16384).
    InvalidTargetSize { width: u32, height: u32 },
    /// `brush_handle` em algum stroke não tem entrada correspondente em
    /// `brush_snapshots`. Cache corrompido OU canon foi salvo sem dedup.
    MissingBrushSnapshot { brush_handle: u32 },
    /// Falha I/O ao escrever resultado offload-disk (W12 streaming).
    Io,
    /// Reproject GPU indisponível neste tier (caller deve cair pra `Det`).
    GpuUnavailable,
    /// Reproject foi cancelado pelo caller via `progress` callback.
    Cancelled,
    // === 3 slots de headroom ===
}

/// Reproject canvas — stub T1.8.
///
/// Implementação real (W12):
/// 1. Cria canvas novo com `target_size` + mesmo color profile.
/// 2. Para cada `stroke` em `history` (ordem `seq` crescente):
///    - Resolve `brush_params_hash` → `Brush` (lookup em `brush_snapshots`).
///    - Escala `points[].x_q1616 / y_q1616` linearmente pro `target_size`
///      (multiplicação fixed-point exata — sem perda de bit).
///    - Re-pinta via stamp pipeline (Det = CPU bit-identical; Gpu = pipeline normal).
/// 3. Aplica adjustment layers + masks no compositor final.
///
/// Hoje retorna `Err(InvalidTargetSize)` se target zero/excedido; senão
/// devolve o canon source clonado (placeholder até W12 wire o pipeline real).
pub fn reproject_canvas(
    src: &PaintProject,
    target_size: (u32, u32),
    _mode: ReprojectMode,
    _progress: &mut dyn FnMut(ProgressEvent),
) -> Result<PaintProject, ReprojectError> {
    let (w, h) = target_size;
    if w == 0 || h == 0 || w > 16_384 || h > 16_384 {
        return Err(ReprojectError::InvalidTargetSize {
            width: w,
            height: h,
        });
    }
    // STUB W12: mode é IGNORADO em T1.8 (Det/Gpu produzem output idêntico).
    // Quando W12 wirar pipeline real, ambos os modos passam a ter
    // comportamento distinto — caller que assumiu "mode é no-op" PRECISA
    // ser revisitado. Audit T1.8 L2-F9 + test
    // `reproject_stub_treats_modes_identically_in_t18` documenta o
    // comportamento atual como SENTINEL pra regressão.
    let mut out = src.clone();
    out.canvas.width = w;
    out.canvas.height = h;
    out.recompute_checksum();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::CanvasInfo;

    #[test]
    fn reproject_rejects_zero_dim() {
        let src = PaintProject::new(CanvasInfo::default());
        let mut progress = |_| {};
        let err = reproject_canvas(&src, (0, 100), ReprojectMode::Det, &mut progress).unwrap_err();
        assert!(matches!(err, ReprojectError::InvalidTargetSize { .. }));
    }

    #[test]
    fn reproject_rejects_oversized_dim() {
        let src = PaintProject::new(CanvasInfo::default());
        let mut progress = |_| {};
        let err =
            reproject_canvas(&src, (20_000, 100), ReprojectMode::Det, &mut progress).unwrap_err();
        assert!(matches!(err, ReprojectError::InvalidTargetSize { .. }));
    }

    #[test]
    fn reproject_stub_recomputes_checksum() {
        let src = PaintProject::new(CanvasInfo::default());
        let mut progress = |_| {};
        let out = reproject_canvas(&src, (2048, 2048), ReprojectMode::Det, &mut progress).unwrap();
        assert_eq!(out.canvas.width, 2048);
        assert_eq!(out.canvas.height, 2048);
        assert!(out.verify_checksum(), "reproject output must self-verify");
    }

    #[test]
    fn reproject_stub_treats_modes_identically_in_t18() {
        // Audit T1.8 L2-F9 — sentinel pra regressão. Quando W12 wirar
        // pipeline real, ESTE test DEVE quebrar — atualizar pra cobrir
        // a diferença Det vs Gpu (ex: blake3 do output difere por <ULP).
        let src = PaintProject::new(CanvasInfo::default());
        let mut p_det = |_| {};
        let mut p_gpu = |_| {};
        let out_det = reproject_canvas(&src, (1024, 1024), ReprojectMode::Det, &mut p_det).unwrap();
        let out_gpu = reproject_canvas(&src, (1024, 1024), ReprojectMode::Gpu, &mut p_gpu).unwrap();
        assert_eq!(
            out_det.checksum, out_gpu.checksum,
            "T1.8 stub treats Det/Gpu identically — when W12 wires real pipeline, update this test"
        );
    }
}
