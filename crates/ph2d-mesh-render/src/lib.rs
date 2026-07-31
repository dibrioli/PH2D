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
//!   contrato congelado não é tocado (ADR-0145).
//! - [`MeshRenderer`] — o passe de **matcap procedural**, que põe forma na tela
//!   sem exigir um asset.

mod camera;
mod pipeline;

pub use camera::Camera3d;
pub use pipeline::{MeshRenderer, camera_uniform_bytes, view_proj_from_bytes};
