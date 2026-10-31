#![forbid(unsafe_code)]
//! `ph2d-paint-tinta` — a área **Tinta** do ADR-0197: pigmento conservado em VOLUMES e a lei única
//! de camada que o transforma em reflectância por banda.
//!
//! - (1) Duncan: `𝒦 = Σ v_i K_i`, `𝒮 = s(n_v) Σ v_i S_i` — [`mistura`].
//! - (2) Kubelka 1948, camada finita sobre `R_g` — [`camada`] (o limite `𝒮 → 0` é o mesmo ramo).
//! - (3) cobertura DERIVADA — [`cobertura`].
//! - (4) transferências que conservam volume — [`transfere`], [`pick_up`], [`fecha`].
//!
//! Especificação: `docs/Painter/Arquitetura/F2_grandezas_fisicas.md` §3; oráculo:
//! `docs/Painter/Arquitetura/ferramentas/lei_unica.py`; mapa: `docs/Painter/Arquitetura/F4_1_tinta.md`.
//!
//! **Determinismo.** Tudo em `f32`, ordem das somas fixa (a ordem do slice), nenhum `mul_add`
//! (o rustc nunca contrai `a*b+c` em FMA sem pedido explícito), transcendentais só do `libm`
//! pinado. Razão: o backend GPU tem de dar o MESMO bit (ADR-0197 §4.3) — ele terá de portar estas
//! expressões na mesma ordem e proibir a contracção no shader.

mod bandas;
mod lei;
mod pigmento;
mod transfere;

pub use bandas::Bandas;
pub use lei::{
    Optica, camada, camada_banda, cobertura, dilui, mistura, n_veiculo, r_infinito, s_fator,
};
pub use pigmento::{Pigmento, R_INF_MIN, S_OPACO, S_TRANSPARENTE};
pub use transfere::{Volumes, fecha, optica_celula, pick_up, transfere};
