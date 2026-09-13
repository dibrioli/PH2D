//! Widget `NodeId`s for the Vector Style panel.
//!
//! ⚠️ **Uma definição e ZERO re-exportações** (auditoria A5b, 2026-09-12). Os ids que só este painel
//! lê moram aqui, partidos por assunto; os que a ferramenta também lê moram na `ph2d-tool-vector`, e
//! os que a `ph2d-editor-core` lê (o corpo do painel, o que o despacho compara, e os que a cerca da
//! `line/render-loop` cita) ficaram lá — quem os usa nomeia a crate dona, nunca este módulo. Ele
//! re-exportava a fundação «para não bifurcar a fonte da verdade»: com uma definição só não há o que
//! bifurcar, e o censo de colisões (`node_id_collisions`) lê os literais da workspace inteira.

mod vector_anchors;
pub use vector_anchors::*;
mod vector_appearance;
pub use vector_appearance::*;
mod vector_bool;
pub use vector_bool::*;
mod vector_components;
pub use vector_components::*;
mod vector_contour;
pub use vector_contour::*;
mod vector_filters;
pub use vector_filters::*;
mod vector_layout;
pub use vector_layout::*;
mod vector_morph;
pub use vector_morph::*;
mod vector_patternpath;
pub use vector_patternpath::*;
mod vector_sections;
pub use vector_sections::*;
mod vector_states;
pub use vector_states::*;
mod vector_text;
pub use vector_text::*;
mod vector_textpath;
pub use vector_textpath::*;
mod vector_texture_pattern;
pub use vector_texture_pattern::*;
mod vector_tokens;
pub use vector_tokens::*;
mod vector_widget;
pub use vector_widget::*;

mod vector;
pub use vector::*;
mod vector_cut;
pub use vector_cut::*;
mod vector_frame;
pub use vector_frame::*;
mod vector_snap;
pub use vector_snap::*;
mod vector_width;
pub use vector_width::*;
