#![forbid(unsafe_code)]
//! `ph2d-paint-tinta` — a área **Tinta** do ADR-0197: pigmento conservado em VOLUMES, misturado em
//! Oklab pesado pelo volume, e a lei de camada que o transforma em reflectância por banda.
//!
//! - (1) mistura (decidida 31/10): estado por fatia = [`Soma`] `{v, Σv·Lab, Σv·t, Σv·m}`; cor = média
//!   pesada pelo volume em Oklab — [`mistura`], [`LeiDeMistura`].
//! - (1') óptica DERIVADA da cor misturada + transparência + volume — [`optica`].
//! - (2) Kubelka 1948, camada finita sobre `R_g` — [`camada`] (o limite `𝒮 → 0` é o mesmo ramo).
//! - (3) cobertura DERIVADA — [`cobertura`]; ecrã «exacto sobre branco» — [`ecra_sobre_branco`].
//! - (4) transferências que conservam volume — [`transfere`], [`pick_up`], [`fecha`].
//!
//! Especificação: `docs/Painter/Arquitetura/F2_grandezas_fisicas.md` §3 e
//! `F2b_mistura_de_cor_nos_apps.md`; oráculo: `docs/Painter/Arquitetura/ferramentas/lei_unica.py`;
//! mapa: `docs/Painter/Arquitetura/F4_1_tinta.md`.
//!
//! **Determinismo.** Tudo em `f32`, ordem das somas fixa, nenhum `mul_add` (o rustc nunca contrai
//! `a*b+c` em FMA sem pedido explícito), transcendentais só do `libm` pinado. Razão: o backend GPU
//! tem de dar o MESMO bit (ADR-0197 §4.3) — ele porta estas expressões na mesma ordem.

mod bandas;
mod lei;
#[expect(
    clippy::excessive_precision,
    reason = "coeficientes publicados por Ottosson VERBATIM (comparáveis à fonte e ao oráculo); \
              o literal arredonda para o mesmo f32 em CPU e GPU"
)]
pub mod oklab;
mod soma;
mod transfere;

pub use bandas::Bandas;
pub use lei::{
    Optica, R_INF_MIN, S_OPACO, S_TRANSPARENTE, camada, camada_banda, cobertura, dilui, n_veiculo,
    optica, optica_de_rinf, r_infinito, s_de_transparencia, s_fator,
};
pub use soma::{LeiDeMistura, Soma, Tinta, mistura};
pub use transfere::{decompoe, ecra_sobre_branco, fecha, optica_celula, pick_up, transfere};
