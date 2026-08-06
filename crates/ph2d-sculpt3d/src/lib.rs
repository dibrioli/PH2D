#![forbid(unsafe_code)]
//! `ph2d-sculpt3d` — **os verbos de escultura, e a lei que os governa**.
//!
//! A W1 provou que o custo de um dab é da PEGADA e não da malha. A W2 é o
//! barro: doze verbos, máscara, simetria — e, antes de tudo isso, a **lei do
//! traço** de [`stroke`], que é o que separa este módulo de um port ingênuo.
//!
//! # A entrada, em três linhas
//!
//! ```no_run
//! # use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};
//! # let mut mesh = ph2d_mesh::shapes::uv_sphere(32, 48, 1.0);
//! # let hit_point = [0.0, 0.0, 1.0];
//! let brush = Brush { verb: Verb::Draw, radius: 0.3, ..Brush::default() };
//! let mut stroke = SculptStroke::default();
//! stroke.begin(&mesh);                                        // congela o `pre`
//! let eye = [0.0, 0.0, -1.0];  // o `dir` do raio que produziu o acerto
//! stroke.dab(&mut mesh, &brush, &Dab::at(hit_point, 0.3, eye), Symmetry::MIRROR_X);
//! ```
//!
//! # Sobre a "porta única" que o `docs/3D/03.5` promete
//!
//! Aquele documento desenha `sculpt_kernel_device(vertices) -> Device`, e ela
//! **continua não construída, de propósito**. Uma porta com uma resposta só e um
//! variant inalcançável é um controle que não faz nada. Ela nasce **quando a
//! medição pedir**: se o K1 disparar num regime que o artista use, o caminho de
//! GPU e a porta chegam juntos, com o de CPU virando o oráculo de paridade dele.
//!
//! # O que a W1 mediu, e o que mudou
//!
//! A sonda `tests/measure_brush_kernel.rs` media um `apply_dab` que **não existe
//! mais** — o traço o subsumiu, e manter os dois seria a segunda porta para
//! *"aplicar um dab"*. Ela agora dirige o produto (`begin` + `dab`), que faz
//! estritamente MAIS trabalho por dab (captura + envelope + alvo), então os
//! números da W1 foram **re-medidos** em vez de herdados.

mod alpha;
mod brush;
pub mod mask_ops;
mod spacing;
mod stroke;

pub use alpha::{Alpha, DEFAULT_ALPHA_SCALE, MAX_ALPHA_SCALE, MIN_ALPHA_SCALE, recommended_scale};
pub use brush::{Amount, Brush, Falloff, Grip, REACH_FRACTION, Symmetry, Verb};
pub use spacing::{ACCUM_PER_DAB, MIN_SPACING_FRACTION, Walk, min_spacing, walk};
pub use stroke::{Dab, SculptStroke};
