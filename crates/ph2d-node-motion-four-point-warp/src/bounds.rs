//! **Até onde os quatro cantos se DIGITAM** — o teto e o piso, medidos (bloco Z, doc 91).
//!
//! ⚠️ **O corte para módulo próprio foi forçado pelo HR-18** (o `lib.rs` estava a 681 de 700), e
//! a costura é por responsabilidade: o `lib.rs` responde *como o quadrilátero deforma* e isto
//! responde *que números o campo aceita*.
//!
//! ## Sete acusações numa cena só
//!
//! A sonda `what_the_corpus_authors_and_no_one_can_type` acusou **sete dos oito cantos** na cena
//! `=14` — `tr_dy = 180`, `tl_dx = 160`, `tr_dx = −120`, e mais quatro — sobre um campo que
//! digitava `[−10, 10]`. É o pior caso do catálogo por número de params, e a razão é a forma do
//! nó: um warp de quatro cantos só é interessante quando os cantos vão longe, então a cena que
//! o demonstra é obrigada a autorar exactamente o que a UI recusa.
//!
//! ## As DUAS pontas, sempre
//!
//! Um canto tem sinal, e `br_dx = −40` foi acusado ao lado de `tl_dx = 160`. Um teto generoso
//! com o piso de ontem faria o quadrilátero esticar-se para um lado e não para o outro — e um
//! gesto que só funciona num sentido lê-se como bug do nó, não como faixa de slider.
//!
//! ## De que recurso é o teto: da PRECISÃO
//!
//! `CLAUDE.md` §0.0. Um canto não satura — mais longe é mais deformado, e isso é uma resposta —,
//! então o que acaba é o `f32`: acima de `2²⁰` somar o `step` do slider (0,05) **não move o
//! número**, e dois cantos autoráveis vizinhos passam a ser o mesmo quadrilátero. O valor é
//! derivado a cada corrida pelo gate `every_precision_bound_param_types_to_the_measured_ceiling`
//! (`ph2d-node-registry-init`); escrito à mão ele envelheceria no dia em que alguém afinasse o
//! arrasto.

use ph2d_node_registry::{ParamHardMax, ParamHardMin};

/// O maior deslocamento de canto que o `step` de 0,05 ainda distingue: `2²⁰ − 1 ulp`.
const REACH: f32 = 1_048_576.0 - 0.0625;

const fn up(param: &'static str) -> ParamHardMax {
    ParamHardMax { param, max: REACH }
}

const fn down(param: &'static str) -> ParamHardMin {
    ParamHardMin { param, min: -REACH }
}

/// Os oito tetos, na mesma ordem TL,TR,BR,BL dos hints.
pub(crate) static PARAM_HARD_MAX: &[ParamHardMax] = &[
    up("tl_dx"),
    up("tl_dy"),
    up("tr_dx"),
    up("tr_dy"),
    up("br_dx"),
    up("br_dy"),
    up("bl_dx"),
    up("bl_dy"),
];

/// E os oito pisos — o simétrico de cada teto, pela razão do cabeçalho.
pub(crate) static PARAM_HARD_MIN: &[ParamHardMin] = &[
    down("tl_dx"),
    down("tl_dy"),
    down("tr_dx"),
    down("tr_dy"),
    down("br_dx"),
    down("br_dy"),
    down("bl_dx"),
    down("bl_dy"),
];
