//! **A família PAINTER da shell** — a metade de *composição* do Painter (W2 Fase D).
//!
//! ⚠️ **Isto NÃO é o motor de pintura.** O motor vive na [`ph2d-tool-painter`] (136 093 LOC, a
//! maior crate do repo) e no [`ph2d-paint-gpu`]; o que mora aqui é o que estava dentro da
//! `shells/desktop` — as cenas de smoke, as pontes de desenho do `render_loop` e os corpos dos
//! gestos de canvas. *Código de FAMÍLIA vive em `crates/ph2d-app-<família>`; a shell é
//! COMPOSIÇÃO* (`CLAUDE.md` §2, `HOWTO_partir_uma_familia_da_shell.md`).
//!
//! # ⛔ O que ficou na shell, e é DESENHO
//!
//! O **LAÇO** (`render_loop/mod.rs`) garante a ordem do quadro e não se abstrai (HOWTO §4): o que
//! sai são os **CORPOS**, e a shell chama `ph2d_app_painter::<mod>::<fn>` no ponto certo. Os
//! gestos de canvas ficam com **invólucros** `&mut App` na shell, porque os corpos deles precisam
//! do `gfx` e ⛔ **nenhuma porta do [`ph2d_app_host::AppHost`] devolve um handle** — é essa
//! proibição que segura a fronteira inteira.
//!
//! # ⚠️ A FACHADA que a régua do fecho contava como shell
//!
//! Estes seis roteadores pareciam presos à `shells/desktop` por `crate::image_import`. Aquele
//! ficheiro tem **6 linhas** e é `pub(crate) use ph2d_image_import::*` — *uma crate a usar o nome
//! da shell*. A `line/app-vec` mediu a mesma coisa na Fase C (cinco das seis âncoras dela eram
//! fachadas) e a leitura é a mesma: **a régua erra aqui no sentido conservador**, que é o oposto do
//! habitual, e por isso passa despercebida.

// ─────────────────────────────────────────────────────────────────────────
// **Os SEIS roteadores de cena.** Cada um lê a PRÓPRIA variável de ambiente
// (`enabled()`), o que é a condição do «fim da linha» desta wave — a shell
// só os chama.
// ─────────────────────────────────────────────────────────────────────────
pub mod impasto_smoke;
pub mod line_smoke;
pub mod mask_smoke;
pub mod substrate_smoke;
pub mod taper_smoke;
pub mod wetpaint_smoke;

#[cfg(test)]
mod family_tests;

/// **O que esta família declara à shell** (`ph2d_app_host::AppFamily`).
///
/// ⛔⛔ **Uma família registada DECLARA roteador.** A catraca
/// `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` morreu na Fase C, e a única ausência aceite é a da
/// família cuja cena vive numa crate irmã — que **não** é o caso desta: os seis roteadores estão
/// aqui, com a `env` lida aqui.
///
/// ⚠️ **Os `max_level` são CONTADOS, cada um no seu ficheiro** (CLAUDE.md §5.0), e a contagem tem
/// duas espécies nesta família:
///
/// | roteador | forma | `max_level` |
/// |---|---|---:|
/// | `PH2D_IMPASTO_SMOKE` | `match` com dois braços (`=2` abre 4096²) | [`impasto_smoke::NIVEIS`] = 2 |
/// | os outros **cinco** | `var_os(..).is_some()` — **presença** | `NIVEIS` = 1 |
///
/// ⛔ **`PH2D_PAINT_PERF`, `PH2D_PREVIEW_DIAG` e `PH2D_PREVIEW_DUMP` não estão aqui, e a ausência é
/// a decisão:** eles são DIAGNÓSTICO, não cenas. *Um roteador declarado diz ao dono que ele tem uma
/// cena para ver*, e mandá-lo correr um despejo de buffers seria uma promessa falsa.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "painter",
    routers: &[
        ph2d_app_host::SmokeRouter {
            env: "PH2D_IMPASTO_SMOKE",
            max_level: impasto_smoke::NIVEIS,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_WETPAINT_SMOKE",
            max_level: wetpaint_smoke::NIVEIS,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_MASK_SMOKE",
            max_level: mask_smoke::NIVEIS,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_SUBSTRATE_SMOKE",
            max_level: substrate_smoke::NIVEIS,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_TAPER_SMOKE",
            max_level: taper_smoke::NIVEIS,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_LINE_SMOKE",
            max_level: line_smoke::NIVEIS,
        },
    ],
};
