//! ⭐ **A metade de APLICAÇÃO do módulo Flip** (W2/L5, 2026-09-11).
//!
//! # O que está aqui
//!
//! As leis do editor de Flip que **não precisam da shell para existir**: a reamostragem e o
//! ajuste do traço autorado ([`smooth`]), a dilatação que fecha o vão do balde
//! ([`fill_dilate`]), a resolução de alvo da tira de quadros ([`strip_resolve`]), a política de
//! teclado do *peek* ([`peek`]), a composição multi-quadro ([`multiframe`]) e a cena de
//! demonstração ([`demo`]).
//!
//! # ⛔ A regra que faz isto funcionar
//!
//! **Nada aqui depende da `shells/desktop`.** Ela é um binário — uma dependência de volta seria
//! um ciclo — e é por isso que o corte é acíclico por construção: a shell chama aqui, e o que
//! precisa de `App` (janela, `AppGfx`, painéis, captura de undo) fica lá, nomeado.
//!
//! # ⚠️ O documento não é daqui
//!
//! [`ph2d_flip::FlipDoc`] é **partilhado** — a F8 dos Componentes fechou-o em 10/09 — e vive na
//! crate de módulo. Ele é o que se grava e o que o undo fotografa. O que sai da shell para aqui é
//! a metade de *aplicação*, nunca o modelo.

pub mod airbrush_smoke;
pub mod autokey;
pub mod bake;
pub mod bridge;
pub mod colorize;
pub mod colorize_smoke;
pub mod ctx;
pub mod cursor;
pub mod demo;
pub mod draw;
pub mod edit_gesture;
pub mod edit_smoke;
pub mod erase;
pub mod fill;
pub mod fill_dilate;
pub mod fill_smoke;
pub mod fill_target;
pub mod gap_live;
pub mod gap_overlay;
pub mod gizmo_view;
pub mod hardness_smoke;
pub mod layers;
pub mod multiframe;
pub mod multiplane_smoke;
pub mod pass;
pub mod pass_cache;
pub mod pass_ghosts;
pub mod pass_stage;
pub mod peek;
pub mod pose_gizmo;
pub mod pose_smoke;
pub mod pressure_smoke;
pub mod resample_smoke;
pub mod reshape;
pub mod segment_smoke;
pub mod select;
pub mod select_pick;
pub mod select_points;
pub mod select_segment;
pub mod selection_gizmo;
pub mod selection_overlay;
pub mod selection_smoke;
pub mod self_overlap_smoke;
pub mod smooth;
pub mod state;
pub mod strip;
pub mod strip_drag;
pub mod strip_pins;
pub mod strip_resolve;
pub mod strip_smoke;
pub mod tip_smoke;
pub mod trace;
pub mod tween_correct;
pub mod tween_overlay;
pub mod tween_pairs_smoke;
pub mod tween_phase_smoke;
pub mod tween_smoke;
pub mod tween_torsion_smoke;

/// **O que esta família declara à shell** (`ph2d-app-registry-init`).
///
/// ⭐ **A `flip` é a ÚNICA das cinco famílias da Fase A que possuía os próprios roteadores** — por
/// isso ela nunca entrou na catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` do registo.
///
/// ⚠️ **`max_level: 1` porque estes roteadores são INTERRUPTORES, não escadas** — cada um é um
/// `env::var_os(..).is_some()`, sem `match` de nível (ao contrário do `PH2D_FIELD_SMOKE` do piloto,
/// que responde por uma faixa). O maior nível que um interruptor responde é `1`, e escrever outra
/// coisa seria prometer cenas que não existem. ⚠️ Os três que entraram na Fase B foram **contados
/// no roteador**, não escritos de memória: os três são `var_os(..).is_some()`.
///
/// ✅ **A dívida dos três FECHOU na Fase B** (11/09): `PH2D_FLIP_HARDNESS_SMOKE` (o mestre),
/// `PH2D_FLIP_PRESSURE_SMOKE` e `PH2D_FLIP_RESAMPLE_SMOKE` são lidos **aqui dentro**, e a lista
/// passou de 15 para **18**. ⭐ O que os destravou não foi um refactor grande: os três precisavam de
/// **UMA** função do `draw` (`stroke_from_samples`), e a única do `draw` presa atrás do
/// `vec_transform` era o `bake_stroke` — separá-la libertou a lei inteira do traço.
///
/// ⛔ **`PH2D_FLIP_FILL_DEBUG` e `PH2D_FLIP_SELECT_DEBUG` NÃO entram, e a ausência é a decisão:**
/// são **diagnóstico** (ligam um `eprintln!`), não roteadores de cena. Um registo que os aceitasse
/// prometeria ao dono uma cena que não existe.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "flip",
    routers: &[
        r("PH2D_FLIP_AIRBRUSH_SMOKE"),
        r("PH2D_FLIP_COLORIZE_SMOKE"),
        r("PH2D_FLIP_EDIT_SMOKE"),
        r("PH2D_FLIP_FILL_SMOKE"),
        r("PH2D_FLIP_HARDNESS_SMOKE"),
        r("PH2D_FLIP_MULTIPLANE_SMOKE"),
        r("PH2D_FLIP_POSE_SMOKE"),
        r("PH2D_FLIP_PRESSURE_SMOKE"),
        r("PH2D_FLIP_RESAMPLE_SMOKE"),
        r("PH2D_FLIP_SEGMENT_SMOKE"),
        r("PH2D_FLIP_SELF_OVERLAP_SMOKE"),
        r("PH2D_FLIP_STRIP_SMOKE"),
        r("PH2D_FLIP_TIP_SMOKE"),
        r("PH2D_FLIP_TWEEN_PAIRS_SMOKE"),
        r("PH2D_FLIP_TWEEN_PHASE_SMOKE"),
        r("PH2D_FLIP_TWEEN_SMOKE"),
        r("PH2D_FLIP_TWEEN_TORSION_SMOKE"),
        r("PH2D_FLIP_XFORM_SMOKE"),
    ],
};

/// Um roteador-interruptor desta família — ver a nota do [`FAMILY`] sobre o `max_level`.
const fn r(env: &'static str) -> ph2d_app_host::SmokeRouter {
    ph2d_app_host::SmokeRouter { env, max_level: 1 }
}
