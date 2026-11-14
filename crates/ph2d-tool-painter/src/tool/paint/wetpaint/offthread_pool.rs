//! **N10 W2b2 — a poça pequena corre num pool PEQUENO** (filho de [`super`] por LOC).
//!
//! A 40 Hz cada estágio do passo parte uma poça minúscula pelo pool GLOBAL (32 fios): os operários acordam,
//! roubam um pedaço de microssegundos e GIRAM à espera do próximo (perf do Wet Liner parado com água viva:
//! `crossbeam_epoch` pin 27,5 % + `try_advance` 4,7 % + `steal`/`wake` ~3 %, 242–308 % de CPU para 6 % de
//! worker ocupado). O giro escala com os fios do pool, não com o trabalho. Abaixo de
//! [`MIN_CELLS_POOL_GLOBAL`] células vivas o passo corre num pool próprio de [`fios`] operários; acima, no
//! global de sempre (a poça grande precisa de todos — o B13.12 mede a escala).
//!
//! Ao bit: os passes paralelos são invariantes à partição (ADR-0145; gate
//! `o_mesmo_pincel_pinta_o_mesmo_bit_em_qualquer_largura_do_pool`), logo a largura do pool não muda nada.
//! Medição e tabela: `docs/Painter/missao_4k/medicoes/N10_cerdas_mistura.md` (W2b2).
//! `PH2D_SIM_POOL_PEQUENO=<n>` (lido uma vez): `0` = o global sempre (o de antes); omissão [`FIOS_OMISSAO`].

use std::sync::OnceLock;

/// O piso de células vivas acima do qual o passo volta ao pool global: o mesmo [`MIN_CELLS_JACOBI`] medido do
/// `par.rs` (abaixo dele cada passe é sub-milissegundo em série — o pool global não tem o que ganhar).
///
/// [`MIN_CELLS_JACOBI`]: ph2d_wet_paint::par::MIN_CELLS_JACOBI
pub(super) const MIN_CELLS_POOL_GLOBAL: u64 = ph2d_wet_paint::par::MIN_CELLS_JACOBI as u64;

/// A largura do pool pequeno por omissão — a medida (tabela no N10, W2b2).
const FIOS_OMISSAO: usize = 4;

/// Os fios do pool pequeno (`0` = desligado).
fn fios() -> usize {
    static F: OnceLock<usize> = OnceLock::new();
    *F.get_or_init(|| {
        std::env::var("PH2D_SIM_POOL_PEQUENO")
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(FIOS_OMISSAO)
    })
}

fn pool() -> Option<&'static rayon::ThreadPool> {
    static P: OnceLock<Option<rayon::ThreadPool>> = OnceLock::new();
    P.get_or_init(|| match fios() {
        0 => None,
        n => rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .thread_name(|i| format!("ph2d-wet-sim-{i}"))
            .build()
            .ok(),
    })
    .as_ref()
}

/// Corre `f` no pool que a poça pede: o pequeno abaixo do piso, o global acima (ou com a chave a `0`).
pub(super) fn no_pool_da_poca<R: Send>(celulas_vivas: u64, f: impl FnOnce() -> R + Send) -> R {
    match pool() {
        Some(p) if celulas_vivas < MIN_CELLS_POOL_GLOBAL => p.install(f),
        _ => f(),
    }
}
