#![forbid(unsafe_code)]
//! `ph2d-mesh-render` — **tudo que o módulo 3D fala com a GPU**.
//!
//! Espelha a `ph2d-flip-render`: o `ph2d-render` deliberadamente não depende de
//! crates de documento, então um passe wgpu dedicado mora aqui, recebe
//! `Device`/`Queue` + o alvo do shell, e rasteriza. Esta é a fronteira que o
//! `docs/3D/03.5` desenha — **a CPU esculpe, a GPU desenha** —, e ela é o que
//! torna o módulo removível: apagar esta crate apaga o 3D da tela sem tocar no
//! compositor 2D.
//!
//! Duas peças na W1/M2:
//!
//! - [`Camera3d`] — a câmera **orbital** (estado + matrizes). O gesto que a
//!   dirige mora no shell, nunca numa `Tool`: navegar não é esculpir, e o
//!   contrato congelado não é tocado (ADR-0150).
//! - [`MeshRenderer`] — o passe que rasteriza e acende a forma.
//!
//! A **W3** trocou o matcap procedural da W1 pelo **rig do artista**: as mesmas
//! quatro lâmpadas que iluminam a tinta do Painter, resolvidas pela mesma função
//! (`ph2d-light`), com o mesmo piso ambiente e o mesmo modelo RELATIVO. Sob o
//! matcap, mover a lâmpada do card não mexia na escultura — e "um documento, uma
//! iluminação" (`docs/3D/05.2`) não é uma frase sobre código compartilhado, é
//! sobre o artista afinar a arte contra a luz que ela vai ter. O empacotamento e a
//! única conversão de espaço que isso exige estão em [`lighting`].
//!
//! A W2 acrescentou o **upload incremental**: um dab move alguns milhares de
//! vértices de uma malha de milhões, e reenviar tudo faria o custo do gesto ser
//! função do DOCUMENTO em vez da PEGADA. O plano de faixas ([`upload`]) é função
//! pura e tem gate sem device; quem executa é o [`MeshRenderer`].

mod camera;
mod lighting;
mod pipeline;
pub mod upload;

pub use camera::Camera3d;
pub use lighting::{LampRaw, RigRaw};
pub use pipeline::{MeshRenderer, camera_uniform_bytes, view_proj_from_bytes};
