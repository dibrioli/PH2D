//! ⭐ **A família Flip, numa pasta só** (W2/L5, 2026-09-11).
//!
//! # Por que esta pasta existe
//!
//! A `shells/desktop` é UMA crate de 493 k linhas, a última unidade de todo build grande
//! (34–45 s sozinha no fim do gate) — e a metade de cada módulo que fala com a `App` foi ficando
//! aqui por inércia ([auditoria de 10/09](../../../../docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md)
//! §4-C2). A família Flip eram **78 ficheiros soltos na raiz de `src/`**, indistinguíveis dos das
//! outras cinco famílias a olho ou a `ls`.
//!
//! Juntá-los numa pasta faz três coisas, e a terceira é a que importa: o `main.rs` troca **49
//! linhas `mod`** por uma · a fronteira da família passa a ser legível (o que é Flip está aqui, e
//! só aqui) · e o corte da W2 Fase B passa a ser **mover UMA pasta** em vez de escolher 78
//! ficheiros um a um.
//!
//! # O que ainda NÃO está aqui, e porquê
//!
//! O **passe de render** do Flip vive em [`crate::render_loop`] (`flip_pass*`, `flip_bridge`,
//! `flip_cursor`, os overlays) — ele precisa do `AppGfx`, do dispositivo wgpu e do laço de
//! desenho, que são da shell e não da família. Sai quando o substrato da `ph2d-app-host` o
//! cobrir, não antes.
//!
//! ⚠️ E o **documento** ([`ph2d_flip::FlipDoc`]) nunca foi desta pasta: ele é partilhado desde a
//! F8 dos Componentes e vive numa crate de módulo. A ponte a não partir.

pub(crate) mod airbrush_smoke_app;
pub(crate) mod autokey;
pub(crate) mod bake;
pub(crate) mod colorize;
pub(crate) mod colorize_smoke_app;
pub(crate) mod draw_app;
pub(crate) mod edit_gesture;
pub(crate) mod edit_smoke_app;
pub(crate) mod entities;
pub(crate) mod erase;
pub(crate) mod fill;
pub(crate) mod fill_smoke_app;
pub(crate) mod fill_target;
pub(crate) mod gap_live_app;
pub(crate) mod gizmo_view;
pub(crate) mod hardness_smoke_app;
pub(crate) mod layers;
pub(crate) mod multiplane_smoke_app;
pub(crate) mod pose_gizmo;
pub(crate) mod pose_smoke_app;
pub(crate) mod pressure_smoke_app;
pub(crate) mod resample_smoke_app;
pub(crate) mod reshape;
pub(crate) mod segment_smoke_app;
pub(crate) mod select;
pub(crate) mod select_pick;
pub(crate) mod select_points;
pub(crate) mod select_segment;
pub(crate) mod selection_gizmo;
pub(crate) mod selection_smoke_app;
pub(crate) mod self_overlap_smoke_app;
pub(crate) mod state;
pub(crate) mod strip;
pub(crate) mod strip_drag;
pub(crate) mod strip_pins;
pub(crate) mod strip_smoke_app;
pub(crate) mod tip_smoke_app;
pub(crate) mod trace;
pub(crate) mod transform;
pub(crate) mod tween_correct;
pub(crate) mod tween_pairs_smoke_app;
pub(crate) mod tween_phase_smoke_app;
pub(crate) mod tween_smoke_app;
pub(crate) mod tween_torsion_smoke_app;
