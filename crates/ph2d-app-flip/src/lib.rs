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
pub mod colorize_smoke;
pub mod demo;
pub mod edit_smoke;
pub mod fill_dilate;
pub mod fill_smoke;
pub mod gap_live;
pub mod multiframe;
pub mod multiplane_smoke;
pub mod peek;
pub mod pose_smoke;
pub mod segment_smoke;
pub mod selection_smoke;
pub mod self_overlap_smoke;
pub mod smooth;
pub mod strip_resolve;
pub mod strip_smoke;
pub mod tip_smoke;
pub mod tween_pairs_smoke;
pub mod tween_phase_smoke;
pub mod tween_smoke;
pub mod tween_torsion_smoke;

/// **O que esta família declara à shell** (`ph2d-app-registry-init`).
///
/// ⭐ **A `flip` é a ÚNICA das cinco famílias da Fase A que possui os próprios roteadores** — por
/// isso ela não entra na catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` do registo. Medido em
/// 2026-09-11: estes 15 nomes são lidos **aqui dentro**, e não há um único que a shell também leia.
///
/// ⚠️ **`max_level: 1` porque estes roteadores são INTERRUPTORES, não escadas** — cada um é um
/// `env::var_os(..).is_some()`, sem `match` de nível (ao contrário do `PH2D_FIELD_SMOKE` do piloto,
/// que responde por uma faixa). O maior nível que um interruptor responde é `1`, e escrever outra
/// coisa seria prometer cenas que não existem.
///
/// ⏳ **O que ainda NÃO está aqui, e é dívida nomeada:** três `PH2D_FLIP_*_SMOKE` continuam a ser
/// lidos pela shell — `PH2D_FLIP_HARDNESS_SMOKE` (o mestre), `PH2D_FLIP_PRESSURE_SMOKE` e
/// `PH2D_FLIP_RESAMPLE_SMOKE`. ⛔ *Uma lista que se lê como completa e não é vale menos que uma
/// ausência*, então eles estão escritos aqui: a Fase B traz os três e esta lista passa a 18.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "flip",
    routers: &[
        r("PH2D_FLIP_AIRBRUSH_SMOKE"),
        r("PH2D_FLIP_COLORIZE_SMOKE"),
        r("PH2D_FLIP_EDIT_SMOKE"),
        r("PH2D_FLIP_FILL_SMOKE"),
        r("PH2D_FLIP_MULTIPLANE_SMOKE"),
        r("PH2D_FLIP_POSE_SMOKE"),
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
