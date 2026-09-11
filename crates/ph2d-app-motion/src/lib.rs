//! **`ph2d-app-motion` — a familia MOTION fora da shell.**
//!
//! ⛔ **Esta crate NAO depende da shell, e nao PODE:** a shell e' um `bin`, logo depender dela e'
//! inexprimivel em Cargo. E' essa impossibilidade que torna o corte VERIFICAVEL em vez de
//! prometido — o que aqui entra provou que so' precisa de crates de MODULO.
//!
//! ## O que esta' aqui, e porque e' TAO pouco (medido, W2/L1 — 2026-09-11)
//!
//! A familia tem `100 352` LOC na shell e **`86 222` deles nao tocam `App` nem `gfx`** (229 de 269
//! ficheiros em `src/motion/`, 134 de 143 em `src/render_loop/`). Mesmo assim so' o
//! [`motion_leaf_images`] pode mudar-se hoje, e a razao nao e' acoplamento a shell — e' a FORMA do
//! grafo de modulos:
//!
//! - **Tudo o resto pende de `MotionState`** (a subarvore dela e' `225` ficheiros / `50 775` LOC,
//!   com `46` a fazerem `use super::*` sobre o namespace dela);
//! - **`MotionState` guarda QUATRO tipos que vivem em `src/render_loop/`** — `VecPathStore`,
//!   `PlantMemo`, `BandCache`, `TableCache`. Os quatro modulos sao PUROS (nenhum toca `App` ou
//!   `gfx`); o que os prende e' o SITIO, nao a dependencia.
//!
//! ⇒ *O bloqueador da Fase B nao e' o trait de host: e' mover `MotionState` e as quatro caches
//! JUNTAS.* Tres dos quatro modulos contem tambem a funcao de publicacao que precisa de `gfx`
//! (`publish`), entao o corte parte cada um em TIPO (vem) e PONTE (fica).
//!
//! ⚠️ **E a primeira medicao desta crate estava ERRADA por uma regua com o alcance trocado:** ela
//! perguntava *«este ficheiro so' referencia modulos da FAMILIA?»* quando a pergunta e' *«so'
//! referencia modulos DO CONJUNTO QUE MOVE?»*. Com a regua larga, `motion_demo_legend` e
//! `motion_object_bake_dims` liam-se moviveis — e o primeiro chama
//! `motion_state::demo_router::build_level` e o segundo importa de `motion_object_bake`, os dois a
//! FICAR. *Um conjunto movivel tem de ser FECHADO sob o que referencia, e «da familia» nao e'
//! «do conjunto».*

/// ⭐ **A arte, em CPU, dos quads que o Motion desenha na cena vectorial** — a memoria da terceira
/// media. O unico modulo da familia que ja' era FECHADO: zero `crate::`, zero `App`, zero `gfx`.
pub mod motion_leaf_images;

/// **O que esta família declara à shell** (`ph2d-app-registry-init`).
///
/// ⚠️⚠️ **`routers: &[]` é a verdade MEDIDA da Fase A** — `"motion"` está na catraca
/// `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` do registo, o único sítio onde *«ainda não saiu»* se
/// distingue de *«alguém esqueceu»*.
///
/// ⭐⭐ **Esta é a família que MENOS código tirou da shell (491 LOC) e a que mais ficheiros tocou
/// (436), e as duas coisas são a mesma:** a jornada foi desprender a família da `App` — os 10
/// `impl crate::App` viraram funções livres, quatro campos soltos viraram uma struct, e os ~130
/// ficheiros `motion_*.rs` viraram a pasta `src/motion/`. ⛔ *Uma leitura do diff pelo saldo de
/// linhas conclui que esta linha quase não trabalhou, e conclui ao contrário.* Os roteadores do
/// Motion (`PH2D_GPU_COOK_DEMO`, `PH2D_MOTION_OBJ_SMOKE`, …) continuam na shell porque as cenas
/// tocam a `App` — é isso, exactamente, que a Fase B corta.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "motion",
    routers: &[],
};
