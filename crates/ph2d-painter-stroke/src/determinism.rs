//! Fixed-point Q16.16 / Q8.8 helpers + det-mode replay stub.
//!
//! Por que fixed-point?
//!
//! - **HR-5 cross-OS bit-identical** (opt-in). f32 não é IEEE-bit-identical
//!   entre ARM/x86 (FMA, denormals, transcendentals). Q16.16 grava o path
//!   bruto sem ambiguidade — replay determinístico exige isso.
//! - **Reproject scaling** sem perda — multiplica por scale_factor fixed-point.
//! - **Compressão** — fixed-point comprime ~30% melhor em zstd que f32.
//!
//! Cobertura T1.8:
//! - Roundtrip `q1616 ↔ f32` ULP-zero pra valores dentro da janela útil.
//! - Roundtrip `q88 ↔ f32` ULP-zero pra valores dentro de [0, 256).
//! - `replay_stroke_det` declarado sob `#[cfg(feature = "det-painter")]`
//!   com corpo stub `todo!()` — feature arming progride em T1.9+ quando
//!   o full stamp pipeline det-mode estiver wired (HR-5 escopo W11+).

/// Converte Q16.16 (i32) para f32. Exato pra valores cujo magnitude
/// fica dentro da janela útil (~±2^15 px / coord — ADR-0046 §2.3).
#[inline]
#[must_use]
pub const fn q1616_to_f32(v: i32) -> f32 {
    v as f32 / 65536.0
}

/// Converte f32 para Q16.16 (i32) **com saturação silenciosa** fora da
/// janela útil. Clamps `f32::NAN`/`±INF` para 0; valores fora de
/// **(-32768.0, +32768.0)** vão pra `i32::{MIN,MAX}` SEM aviso (release).
///
/// **Audit T1.8 L4-H3 — naming `_saturating`:** Rust idiom estabelecido
/// (std + numerics ecosystem) é `op`/`op_checked`/`op_saturating`/
/// `op_wrapping`. O nome sem sufixo era ambíguo (caller PCA escolhe nome
/// curto e fica com gesto fora-de-canvas virando ponto no canto extremo
/// sem warning). Naming honest: `_saturating` documenta que clampa,
/// [`f32_to_q1616_checked`] é o caminho recomendado pra input não-validado
/// upstream. `debug_assert!` no path saturating ajuda detectar em testes.
///
/// Caller que NÃO valida upstream DEVE usar [`f32_to_q1616_checked`].
/// Hot path com input já validado pode usar este (pre-checked Streamline/
/// Stabilization runtime).
#[inline]
#[must_use]
pub fn f32_to_q1616_saturating(v: f32) -> i32 {
    debug_assert!(
        !v.is_finite() || v.abs() < 32_768.0,
        "f32_to_q1616_saturating input fora da janela ULP-zero (-32768, +32768): {} \
         — use f32_to_q1616_checked para validar upstream",
        v
    );
    if !v.is_finite() {
        return 0;
    }
    // **U-4 (audit T1.9):** explicit subnormal flush-to-zero pra cross-OS
    // determinism. SSE2 MXCSR.FTZ/DAZ (HPC env linking MKL/OpenBLAS) +
    // ARM FPCR.FZ (embedded ports) + wasm32 V8 motors podem flush subnormal
    // silenciosamente — multiplicação `v * 65536.0` em denormal produz
    // 0 OR subnormal-multiple OS-dependent. Pre-flush explícito = OS-
    // independent zero pra ALL subnormal/denormal inputs.
    if v.abs() < f32::MIN_POSITIVE {
        return 0;
    }
    // Clamp to representable range pra evitar overflow no `as i32`.
    let scaled = (v * 65536.0).clamp(i32::MIN as f32, i32::MAX as f32);
    scaled as i32
}

/// **Deprecated alias** para [`f32_to_q1616_saturating`]. Mantido pra
/// migração suave de consumers durante T1.9+. Remover em W11 polish.
#[inline]
#[must_use]
#[deprecated(note = "use f32_to_q1616_saturating (explicit naming, audit T1.8 L4-H3)")]
pub fn f32_to_q1616(v: f32) -> i32 {
    f32_to_q1616_saturating(v)
}

/// Variante checked de [`f32_to_q1616`] — retorna `None` se o input está
/// fora da janela útil **(-32768.0, +32768.0)** ou é não-finito.
///
/// Audit T1.8 L2-F1: caller que QUER detectar gestos fora-de-canvas
/// (replay de stroke arquivado, MCP-injected stroke com coords malformadas,
/// reproject scale-up) DEVE usar esta variante. [`f32_to_q1616`] continua
/// disponível para hot path de input onde drift incidental cross-OS é
/// aceito (Streamline, Stabilization runtime).
#[inline]
#[must_use]
pub fn f32_to_q1616_checked(v: f32) -> Option<i32> {
    if !v.is_finite() || v.abs() >= 32_768.0 {
        return None;
    }
    let scaled = v * 65536.0;
    Some(scaled as i32)
}

/// Converte Q8.8 (u16) para f32. Exato pra valores **[0, 256]** — `q=256`
/// vira `1.0` (saturation marker, audit T1.8 L2-F4).
///
/// **Pressure docstring alignment:** `RawPointerSample::pressure_q88` é
/// nominalmente `[0, 1.0]` (inclusive 1.0 = pressão máxima saturada);
/// `q=256` é a representação canônica de 1.0. Caller convertendo pressure
/// do device típicamente cap em `q=255` (255/256 ≈ 0.996) — mas
/// `f32_to_q88(1.0)` returna 256 pra preservar a equivalência 1.0 ↔ 256.
#[inline]
#[must_use]
pub const fn q88_to_f32(v: u16) -> f32 {
    v as f32 / 256.0
}

/// Converte f32 para Q8.8 (u16). Clamps para [0, u16::MAX as f32 / 256.0)
/// e descarta sinal (Q8.8 é unsigned).
#[inline]
#[must_use]
pub fn f32_to_q88(v: f32) -> u16 {
    if !v.is_finite() || v <= 0.0 {
        return 0;
    }
    // U-4: subnormal flush-to-zero pra cross-OS determinism (vide
    // f32_to_q1616_saturating).
    if v < f32::MIN_POSITIVE {
        return 0;
    }
    let scaled = (v * 256.0).clamp(0.0, u16::MAX as f32);
    scaled as u16
}

/// Replay determinístico de um stroke. Stub T1.8 — arming progride com
/// det-mode full em T1.9+.
///
/// Quando armed:
/// - Tudo CPU (zero GPU shader).
/// - Stamps gerados em ordem fixa: por seq + ordem em `points[]`.
/// - RNG: `ChaCha8Rng::seed_from_u64(stroke.rng_seed)`.
/// - Math: f64 ops via `compiler_fence` pra impedir FMA-fusion auto.
/// - Output: bit-identical cross-OS (Linux x86_64 / macOS arm64 /
///   Windows x86_64 / Web wasm32).
///
/// **Por que stub agora**: full det-mode replay precisa do CPU stamp
/// pipeline maduro (T1.5 entregou CPU MVP; T1.6 adicionou brush real;
/// W2 fecha undo + sidebar). Adiantar o gate aqui mantém o contrato
/// público sem implementação prematura.
#[cfg(feature = "det-painter")]
pub fn replay_stroke_det(
    _stroke: &crate::record::StrokeRecord,
    _layer_bytes: &mut [u8],
) -> Result<(), DetReplayError> {
    // Audit T1.8 L1-F12/L2-F3: `todo!()`/`unimplemented!()` em código
    // publico-API com feature ativa = panic em prod path. Retornamos
    // erro tipado para que caller possa detectar e cair pra GPU mode
    // (ou abortar reproject) sem panic.
    Err(DetReplayError::NotImplementedYet)
}

/// Erros possíveis do det-mode replay. T1.8 ship com apenas
/// `NotImplementedYet`; T1.9+ adiciona variants conforme o pipeline
/// completa (sample malformado, brush não encontrado, etc).
///
/// **Audit T1.8 L3-G9:** `#[non_exhaustive]` permite expansão futura
/// sem semver break em downstream.
#[cfg(feature = "det-painter")]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DetReplayError {
    /// Det-mode replay declarado mas implementação chega em T1.9+
    /// (CPU stamp pipeline com ChaCha8Rng + libm trig pinned).
    NotImplementedYet,
}

#[cfg(feature = "det-painter")]
impl std::fmt::Display for DetReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplementedYet => write!(
                f,
                "det-painter replay implementation pending (T1.9+ wires CPU stamp pipeline)"
            ),
        }
    }
}

#[cfg(feature = "det-painter")]
impl std::error::Error for DetReplayError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn q1616_zero_roundtrip() {
        assert_eq!(q1616_to_f32(0), 0.0);
        assert_eq!(f32_to_q1616_saturating(0.0), 0);
    }

    #[test]
    fn q1616_unit_roundtrip() {
        // 1.0 = 65536 em Q16.16.
        assert_eq!(f32_to_q1616_saturating(1.0), 65536);
        assert_eq!(q1616_to_f32(65536), 1.0);
    }

    #[test]
    fn q1616_negative_roundtrip() {
        assert_eq!(f32_to_q1616_saturating(-100.5), -100i32 * 65536 - 32768);
        assert_eq!(q1616_to_f32(-100i32 * 65536 - 32768), -100.5);
    }

    #[test]
    fn q1616_roundtrip_canvas_range() {
        // Janela útil: ±32768 px / coord — qualquer ponto razoável de canvas.
        // ADR-0046 §2.3 cap supports até 16384×8192 spec §2.5.
        for &v in &[
            0.0f32, 0.5, -0.5, 1.0, -1.0, 100.0, -100.0, 16383.5, -16383.5, 16384.0, -16384.0,
            // Audit T1.8 L2-F8: cobrir EXTREMOS da janela útil.
            32767.0, 32767.5, -32767.0, -32767.5,
        ] {
            let q = f32_to_q1616_saturating(v);
            let back = q1616_to_f32(q);
            assert_eq!(back, v, "Q16.16 roundtrip failed for {}", v);
        }
    }

    #[test]
    fn f32_to_q1616_checked_rejects_outside_useful_window() {
        // Audit T1.8 L2-F1: caller que precisa detectar overflow usa _checked.
        assert!(
            f32_to_q1616_checked(32_768.0).is_none(),
            "boundary +32768 inclusive rejected"
        );
        assert!(
            f32_to_q1616_checked(-32_768.0).is_none(),
            "boundary -32768 inclusive rejected"
        );
        assert!(
            f32_to_q1616_checked(50_000.0).is_none(),
            "far outside rejected"
        );
        assert!(f32_to_q1616_checked(f32::NAN).is_none());
        assert!(f32_to_q1616_checked(f32::INFINITY).is_none());
        assert!(f32_to_q1616_checked(f32::NEG_INFINITY).is_none());
        // Valores dentro da janela passam.
        assert_eq!(f32_to_q1616_checked(0.0), Some(0));
        assert_eq!(f32_to_q1616_checked(16384.0), Some(16384 * 65536));
        assert_eq!(f32_to_q1616_checked(-16384.0), Some(-16384 * 65536));
        assert_eq!(f32_to_q1616_checked(32767.5), Some(32767 * 65536 + 32768));
    }

    #[test]
    fn q1616_clamps_nan_and_infinity() {
        assert_eq!(f32_to_q1616_saturating(f32::NAN), 0);
        assert_eq!(f32_to_q1616_saturating(f32::INFINITY), 0);
        assert_eq!(f32_to_q1616_saturating(f32::NEG_INFINITY), 0);
    }

    #[test]
    fn q88_zero_and_one_roundtrip() {
        assert_eq!(q88_to_f32(0), 0.0);
        assert_eq!(f32_to_q88(0.0), 0);
        // 1.0 = 256 em Q8.8.
        assert_eq!(f32_to_q88(1.0), 256);
        assert_eq!(q88_to_f32(256), 1.0);
    }

    #[test]
    fn q88_pressure_resolution_ok() {
        // Pressure typical em [0, 1.0). Q8.8 resolução = 1/256 ≈ 0.0039.
        // Acima da JND perceptual do Pencil Pro 4096-level (~0.025).
        let q = f32_to_q88(0.5);
        assert_eq!(q, 128);
        assert_eq!(q88_to_f32(q), 0.5);
    }

    #[test]
    fn q88_clamps_negative_and_overflow() {
        assert_eq!(f32_to_q88(-1.0), 0);
        assert_eq!(f32_to_q88(f32::NAN), 0);
        assert_eq!(f32_to_q88(1000.0), u16::MAX);
    }
}
