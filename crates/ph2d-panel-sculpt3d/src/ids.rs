//! `NodeId`s dos widgets do painel da cena 3D.
//!
//! ⚠️ **Uma definição e ZERO re-exportações** (auditoria A5b, 2026-09-12). Os ids que só este painel
//! lê moram aqui, partidos por assunto; os que a `ph2d-editor-core` também lê (o corpo do painel,
//! que o *z-order walk* percorre) ficaram lá, e quem os usa nomeia `ph2d_editor_core::ids::X` —
//! nunca por este módulo. Ele re-exportava a fundação «para não bifurcar a fonte da verdade»: com uma
//! definição só não há o que bifurcar, e o censo de colisões (`node_id_collisions`) lê os literais da
//! workspace inteira, não uma lista na fundação.

mod inspector;
pub use inspector::*;
mod sculpt3d;
pub use sculpt3d::*;
mod sculpt3d_cloth;
pub use sculpt3d_cloth::*;
/// Os ids do pincel de POSE — ver [`sculpt3d_pose`].
mod sculpt3d_pose;
pub use sculpt3d_pose::*;
/// Os ids do pincel de CONTORNO — ver [`sculpt3d_boundary`].
mod sculpt3d_boundary;
pub use sculpt3d_boundary::*;
/// Os ids do pincel de ESFREGAR DESLOCAMENTO — ver [`sculpt3d_smear`].
mod sculpt3d_smear;
pub use sculpt3d_smear::*;
mod sculpt3d_brush;
pub use sculpt3d_brush::*;
mod sculpt3d_shading;
pub use sculpt3d_shading::*;
