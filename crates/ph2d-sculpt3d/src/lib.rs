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
/// **O ORÇAMENTO DE ALISAMENTO** — quantas passadas do laplaciano um
/// `auto_smooth` compra, e com que força cada uma. Ver [`auto_smooth`].
pub mod auto_smooth;
/// **A SILHUETA DE UM DAB** — ver [`footprint`].
mod footprint;
pub use footprint::{Blade, Footprint, Strip, rounded_box};

mod brush;
mod coat;
/// **A CURVA DO PINCEL** — o falloff, irmão do [`brush`]. Ver [`falloff`].
mod falloff;
mod grip;
/// **O CAMPO ELÁSTICO** — os Kelvinlets regularizados (de Goes & James 2017),
/// que são o `l-mode` da família que agarra. Ver [`kelvinlet`].
pub mod kelvinlet;
pub mod mask_ops;
mod preview;
/// **OS KERNELS DA REFERÊNCIA** — o porte 1:1 do SculptGL, `f64` na aritmética
/// e `f32` no armazenamento, gateado bit a bit contra o JS EXECUTANDO
/// (`tests/sculptgl_parity.rs`). Ver [`ref_kernels`].
pub mod ref_kernels;
mod ref_mode;
/// **OS TRÊS MODOS DE REFERÊNCIA** — de qual fonte (SculptGL · Blender ·
/// literatura) um verbo herda o que ele é. Ver [`ref_mode`] e o plano
/// `docs/3D/21_plano_modos_e_ferramentas.md`.
/// A tabela DECLARATIVA dos modos — ver o cabeçalho dela.
mod ref_profiles;
mod spacing;
mod stroke;
mod transform;

pub use alpha::{
    Alpha, AlphaFrame, AlphaImage, AlphaStencil, DEFAULT_ALPHA_SCALE, MAX_ALPHA_SCALE,
    MAX_AXIS_ELEV_DEG, MIN_ALPHA_SCALE, recommended_scale, sampled_edge,
};
pub use brush::{
    BLENDER_REACH_FRACTION, Brush, CLAY_PLANE_FRACTION, CLAY_THUMB_TILT_MAX_DEG,
    CLAY_THUMB_TILT_STEP_DEG, CREASE_FRACTION, DEFAULT_MULTIPLANE_ANGLE_DEG, FilterKind,
    LAYER_HEIGHT_HARD_MAX, LAYER_HEIGHT_UI_MAX, MAX_MASK_HARDNESS, MULTIPLANE_ANGLE_MAX_DEG,
    MULTIPLANE_ANGLE_SMOOTH, MULTIPLANE_TIP_STRETCH, PINCH_GAIN, Pass, REACH_FRACTION,
    RingOperator, STRIP_PLANE_FRACTION, Symmetry, TAUBIN_LAMBDA, TAUBIN_MU, TAUBIN_PASS_BAND, Verb,
};
pub use coat::{COAT_HEAD, coat_step};
pub use falloff::Falloff;
pub use grip::{Amount, Grip, GripLaw};
pub use kelvinlet::KELVINLET_REACH;
pub use preview::{NO_PREVIEW, preview_into, preview_verts};
pub use ref_mode::{Field, FrontFace, KernelLaw, LateralPull, PlaneReach, RefMode};
pub use ref_profiles::VerbProfile;
pub use spacing::{MIN_SPACING_FRACTION, Walk, min_spacing, walk};
pub use stroke::{
    Dab, FILTER_DRAG_PER_PX, HC_SHAPE_DEFAULT, HC_VERTEX_DEFAULT, HC_VERTEX_MIN, SculptStroke,
    sharpen_total_for_measurement,
};
pub use transform::{Gesture, MIN_SCALE_FACTOR, MaskTransform, TransformKind};
