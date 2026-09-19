//! **O QUE O ARTISTA AUTORA NO HALO, E O QUE CADA NÚMERO SIGNIFICA** — os params do bloom e as
//! derivações que eles alimentam (a curva do joelho, a base da tenda, o teto do bright-pass, a
//! tag da operação, a bandeira da fonte).
//!
//! ⚠️ **O corte foi FORÇADO pelo HR-18** (o `motion_fx.rs` passou de 700 ao ganhar a rampa,
//! doc 89 folha 11) **e a costura é por responsabilidade**: isto responde *o que o artista pediu
//! e o que aquilo quer dizer*, e o irmão responde *que passes o desenham*. As duas crescem por
//! razões diferentes — esta quando um knob entra, aquela quando um passe entra.

// ⭐⭐⭐ **O TIPO MUDOU-SE PARA A FOLHA [`ph2d_bloom`], e o nome fica** (2026-09-19).
//
// ⛔ Ordem do dono, depois de ver um bloom pior ao lado deste: *«nosso bloom original é muito
// melhor. retire essa implementação godot»*. A lei do halo passa a ter **um** dono — a folha —
// e o campo implícito (`docs/Render3d/12`) consome-a de lá em vez de a copiar.
//
// ⚠️ **O re-export é o que faz esta mudança não tocar em consumidor nenhum**: `MotionFx`, o nó
// `fx.glow` e a shell continuam a escrever `ph2d_render::BloomParams`.
pub use ph2d_bloom::BloomParams;

// ⚠️ **Os métodos viajaram com o TIPO** — o Rust proíbe um `impl` inerente para um tipo de outra
// crate, e isso é a linguagem a dizer a coisa certa: *a lei de um dado mora onde o dado mora*.
// Eles são hoje [`ph2d_bloom::BloomParams::upsample_basis`] e irmãos, e o `ph2d-render` chama-os
// exactamente como chamava.
pub use ph2d_bloom::COMPOSITE_OPERATIONS;
