//! Saturating pigment-opacity table (port of `opacity.js`, SPEC §3).
//!
//! Coverage is not linear in pigment mass: each unit of mass covers 0.2% of
//! whatever paper is still showing, i.e. `alpha(m) = 1 - 0.998^m`. Precomputed
//! once as a 3001-entry table; every consumer truncates its mass to an integer
//! index and clamps into the table, so mass >= 3000 reads fully opaque.
//! Half-opacity sits near mass ~346 — a light wash (100..400) lets the paper
//! glow through, which is what makes deposits read as watercolor.
//!
//! The table is a COMPILE-TIME static, not a lazy singleton: `alpha_of_mass`
//! runs several times per cell in the drying/trail hot loops, and a OnceLock
//! acquire per lookup was measurable on the flood upper bound.

pub const OPACITY_TABLE_MAX: usize = 3000;

const fn build_table() -> [f32; OPACITY_TABLE_MAX + 1] {
    let mut t = [0.0f32; OPACITY_TABLE_MAX + 1];
    // The JS recurrence runs in f64 reading back the f32 it just stored.
    let mut m = 1;
    while m <= OPACITY_TABLE_MAX {
        t[m] = (0.002 + 0.998 * t[m - 1] as f64) as f32;
        m += 1;
    }
    t[OPACITY_TABLE_MAX] = 1.0; // force exact saturation at the top entry
    t
}

static OPACITY: [f32; OPACITY_TABLE_MAX + 1] = build_table();

/// The table read, given an index that is ALREADY the JS ToInt32 of the mass.
/// The clamping half of [`alpha_of_mass`], split out so both the door and its
/// fast path answer the tail of the question with one piece of code.
#[inline]
pub fn table_at(i: i32) -> f64 {
    if i <= 0 {
        return 0.0;
    }
    if i >= OPACITY_TABLE_MAX as i32 {
        return 1.0;
    }
    OPACITY[i as usize] as f64
}

/// The mass domain on which JS ToInt32 collapses to plain truncation: every
/// non-negative float below 2^31. There `trunc()` already lands inside
/// `[0, 2^32)`, so the `rem_euclid` of [`crate::jsmath::to_int32_wrapping`] is
/// the identity, and Rust's `as i32` truncates toward zero to the same number.
const TRUNCATION_IS_TOINT32_BELOW: f64 = 2_147_483_648.0;

/// Opacity lookup: truncate mass to int (JS `m | 0` — ToInt32 WRAPS, it
/// never saturates; port-verify finding), clamp into the table.
///
/// ⚠️ **O caminho rápido não é uma aproximação — é a MESMA resposta sem o
/// `fmod`** (doc 28 §5.43). `to_int32_wrapping` faz `trunc().rem_euclid(2^32)`,
/// e `%` em `f64` é uma **chamada à libm**, não uma instrução; medido, ela
/// custa **2,51 ns por consulta contra 0,54**, e a secagem faz **cinco**
/// consultas por célula. No domínio que uma massa de pigmento de fato ocupa
/// (`0 <= m < 2^31`) o resto é a identidade, então o índice é bit a bit o
/// mesmo — a prova está em `the_fast_path_is_the_slow_path` e o oráculo é a
/// porta ANTIGA, congelada sob `cfg(test)`.
///
/// Negativo, NaN, infinito e `m >= 2^31` caem no caminho de sempre, verbatim:
/// eles não acontecem no motor, e o que não acontece não precisa ser rápido —
/// precisa continuar **certo**.
#[inline]
pub fn alpha_of_mass(m: f64) -> f64 {
    if (0.0..TRUNCATION_IS_TOINT32_BELOW).contains(&m) {
        return table_at(m as i32);
    }
    table_at(crate::jsmath::to_int32_wrapping(m))
}

/// **A PORTA ANTIGA, congelada** — o `alpha_of_mass` que shipava antes do
/// caminho rápido, verbatim, para o gate de identidade ter um oráculo que não
/// é o código sob teste.
///
/// `#[cfg(test)]` de propósito: um `pub` sem chamador não é código morto
/// silencioso, é uma **segunda resposta** esperando alguém chamá-la (a lição do
/// `warp_axis` / do `serial_side` / do `sim_step_atomic_reference`).
#[cfg(test)]
fn alpha_of_mass_reference(m: f64) -> f64 {
    table_at(crate::jsmath::to_int32_wrapping(m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curve_saturates_like_the_spec_says() {
        assert_eq!(alpha_of_mass(0.0), 0.0);
        assert_eq!(alpha_of_mass(3000.0), 1.0);
        assert_eq!(alpha_of_mass(5000.0), 1.0);
        // Half-opacity sits near mass ~346 (SPEC §3).
        let a346 = alpha_of_mass(346.0);
        assert!((a346 - 0.5).abs() < 0.01, "alpha(346) = {a346}");
        // A light wash lets the paper show through (SPEC: 100..400 -> 0.18..0.55).
        let a100 = alpha_of_mass(100.0);
        assert!(a100 > 0.15 && a100 < 0.22, "alpha(100) = {a100}");
    }

    /// **O caminho rápido É o caminho lento** (doc 28 §5.43).
    ///
    /// O oráculo é [`alpha_of_mass_reference`] — a porta que shipava —, nunca a
    /// própria função: *um oráculo que usa a função sob teste para computar o
    /// que espera é sempre verde*.
    ///
    /// A fixture contém o fenômeno em vez de amostrá-lo perto do meio: as
    /// FRONTEIRAS do caminho rápido (`-0.0`, `0`, `1`, `2999`, `3000`, `2^31`),
    /// os valores que **não** entram nele (negativos, NaN, ±inf, subnormais) e
    /// uma varredura densa do domínio da tabela — porque o defeito que este
    /// gate existe para pegar é um índice que difere em UM.
    ///
    /// Mutação: trocar `m as i32` por `m as i32 + 1` sangra na varredura densa;
    /// afrouxar o guard para `m >= 0.0` (sem o teto) sangra em `2^31`, onde o
    /// `as i32` do Rust **satura** e o ToInt32 do JS **envolve**.
    #[test]
    fn the_fast_path_is_the_slow_path() {
        let mut cases: Vec<f64> = vec![
            -0.0,
            0.0,
            f64::MIN_POSITIVE,
            5e-324,
            0.5,
            1.0,
            1.5,
            345.9,
            346.0,
            2999.0,
            2_999.999_999,
            3000.0,
            3000.5,
            1e6,
            2_147_483_647.0,
            2_147_483_648.0,
            2_147_483_649.0,
            4_294_967_296.0,
            4_294_967_297.5,
            -1.0,
            -0.5,
            -3000.0,
            -4_294_967_296.0,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            f64::MAX,
        ];
        // Varredura densa do domínio da tabela, em passos que caem dos DOIS
        // lados de cada fronteira inteira.
        for k in 0..30_000u32 {
            cases.push(f64::from(k) * 0.1);
        }
        for m in cases {
            let fast = alpha_of_mass(m);
            let slow = alpha_of_mass_reference(m);
            assert_eq!(
                fast.to_bits(),
                slow.to_bits(),
                "alpha_of_mass({m}) divergiu: rapido {fast}, porta antiga {slow}"
            );
        }
    }

    /// A porta continua sendo UMA: `table_at` responde a metade do clamp para o
    /// caminho rápido E para o lento, então nenhum dos dois pode ganhar um teto
    /// diferente do outro.
    #[test]
    fn both_paths_clamp_through_the_same_door() {
        assert_eq!(table_at(-5), 0.0);
        assert_eq!(table_at(0), 0.0);
        assert_eq!(table_at(OPACITY_TABLE_MAX as i32), 1.0);
        assert_eq!(table_at(i32::MAX), 1.0);
        assert_eq!(table_at(1), f64::from(OPACITY[1]));
    }

    #[test]
    fn the_const_table_matches_the_runtime_recurrence() {
        // The compile-time evaluation must be the SAME arithmetic the JS
        // runs at startup — recompute at runtime and compare bit-for-bit.
        let mut prev = 0.0f32;
        for m in 1..OPACITY_TABLE_MAX {
            let v = (0.002 + 0.998 * prev as f64) as f32;
            assert_eq!(v.to_bits(), OPACITY[m].to_bits(), "table diverges at {m}");
            prev = v;
        }
    }
}
