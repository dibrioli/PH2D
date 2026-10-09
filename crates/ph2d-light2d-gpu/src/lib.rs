//! **A luz 2D na GPU** — Radiance Cascades 2D (Sannikov) sobre a grade de luz ANCORADA NO MUNDO.
//!
//! Desenho: [`docs/Luz2D/01_plano.md`](https://github.com/dibrioli/PH2D/blob/main/docs/Luz2D/01_plano.md)
//! §2. A entrada é a [`CenaDeLuz`](ph2d_light2d_ecs::CenaDeLuz) e mais nada (a GPU nunca lê o ECS
//! por outro caminho); a saída é a FLUÊNCIA por sonda da c0 — a mesma grandeza que o gabarito
//! (`ph2d-light2d-ref::fluencia`) calcula exato, e é contra ele que o gate G3 mede cada degrau.
//!
//! Passes: **C0** captura (raster das formas: emissão · transmissão · alcance) → **C1** campo de
//! distância (JFA) → **C2** cascatas (traço + fusão por nível, de cima para baixo). A
//! [`PassList`] declara texturas e passes; dela saem a alocação e os bytes por quadro.
//!
//! O `Emissor::Ponto` (sem área) NÃO entra na C0 — é o caminho analítico C3 (W5) — e sai no
//! [`RelatorioC0`], como toda forma que não acende nenhum texel.

mod captura;
mod captura_sprite;
mod config;
mod passlist;
mod shaders;
mod solver;
mod solver_sprite;
mod tempos;

pub use captura::{ForaDaC0, Motivo, Quem, RelatorioC0, o_que_fica_fora};
pub use captura_sprite::{Amostragem, CapturaDeSprites, SpriteNaC0};
pub use config::{Config, Fusao, Grade, Precisao};
pub use passlist::{Buffer, PassList, Passe, Textura};
pub use shaders::{C0_DONO_WGSL, QUEDA_WGSL, rc as fonte_do_rc};
pub use solver::{ErroSolver, Solver};
pub use tempos::TemposMs;
