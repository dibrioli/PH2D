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

// ─── a raiz da família (era `shells/desktop/src/motion.rs`) ───────────────────────
/// A auditoria do grupo do ciclo 2 (doc 105) — `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub mod motion_animadores_probe;
pub mod motion_autofix_smoke;
pub mod motion_autofix_smoke_appropriate;
pub mod motion_autofix_smoke_dead_branch;
/// A auditoria do grupo do ciclo 4 (os CAMPOS) — `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub mod motion_campos_probe;
/// A outra metade dele — **o PREÇO** (relógio, contagem, dispositivo). Irmão pelo tecto de
/// LOC, cortado por responsabilidade: o que um nó DECLARA fica ali, o que ele CUSTA aqui.
#[cfg(test)]
pub mod motion_ciclo_preco;
/// ⭐ **O INSTRUMENTO DE UM CICLO**, para qualquer grupo — retrato, params, cartão,
/// nomes e vocabulário. Nasceu ao abrir o ciclo 4, quando ia ser copiado do 3.
#[cfg(test)]
pub mod motion_ciclo_probe;
/// O PREÇO do mesmo grupo — irmão do acima pelo tecto de LOC, cortado por
/// responsabilidade: retratos ali, relógio e dispositivo aqui.
#[cfg(test)]
pub mod motion_deformadores_preco;
/// A auditoria do grupo do ciclo 3 (doc 106) — `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub mod motion_deformadores_probe;
pub mod motion_delay_smoke;
/// **A legenda de uma cena de smoke, no canvas** (Enio 2026-08-23) — o rótulo pousa
/// em cima do caso que ele explica, em vez de num terminal atrás da janela.
pub mod motion_demo_legend;
pub mod motion_flip_bake;
pub mod motion_fx_smoke;
pub mod motion_node_path_smoke;
pub mod motion_object_bake;
/// Irmão do acima por RESPONSABILIDADE: ele produz pixels (GPU), este diz quantos (CPU).
pub mod motion_object_bake_dims;
pub mod motion_object_smoke;
/// Irmã das duas acima: reduz um assado ao cartão do painel.
pub mod motion_object_thumb;
pub mod motion_path_smoke;
/// O estado de shell da familia — os quatro campos que saíram da `App` na W2/L1 (A2b).
/// O que uma CENA pede à shell, no vocabulário da família (W2 Fase C).
pub mod motion_scene_ctx;
/// O tile de uma forma PARAMÉTRICA (`source.shape`) — irmão do `motion_object_bake`,
/// e a metade que faz o glow alcançar as formas (bug do Enio, 2026-08-20).
pub mod motion_shape_bake;
pub mod motion_shape_smoke;
pub mod motion_shape_smoke_knobs;
pub mod motion_shell_state;
/// A auditoria do grupo do ciclo 5 (a SIMULAÇÃO) — `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub mod motion_sim_probe;
/// A sonda do custo do carimbo (report do Enio, 2026-09-06) — `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub mod motion_stamp_cost_probe;
pub mod motion_state;

// ─── as duas âncoras que eram módulos de TOPO da shell e são código desta família ─────
/// ⭐ **O gizmo dos CAMPOS na tela.** Ele lê `MotionState` e `motion_bridge::params`, logo nunca
/// foi uma folha partilhada: era código do Motion com morada errada.
pub mod field_gizmo;
/// A cena mínima que julga o picker de colunas do `value.attribute` — **só a motion a consome**.
pub mod picker_smoke;

// ─── os 17 que eram filhos DIRECTOS do `render_loop/mod.rs` ──────────────────────
// ⚠️ O LAÇO ficou na shell (ele garante a ordem dos 48 símbolos); o que saiu foram os
// CORPOS, e a shell chama-os por `ph2d_app_motion::<mod>::<fn>` no ponto certo.
/// doc 89 folha 14: a metade do shell do `source.text` — o bloco vira uma
/// instância POR CARACTERE, com a geometria de cada glifo internada no MESMO
/// store das formas (um `geometry_id` é um `geometry_id`, venha de onde vier).
pub mod motion_audio_gen;
pub mod motion_bridge;
pub mod motion_externals;
/// **A MÁSCARA DE SUJIDADE do halo**, resolvida contra a cena (doc 89 folha 11) — o nó guarda
/// um NOME, o passe de tela quer uma `TextureView`, e este é o único sítio onde a cena, o atlas
/// e as duas lojas de textura estão em mão ao mesmo tempo.
pub mod motion_glow_dirt;
/// **A CAMADA que o glow bright-passa** (bug do Enio, 2026-08-20): a lista de
/// instâncias do passe de isolamento, que é a camada MOTION inteira e não só o
/// passe de sprites — a metade vetorial viva entra pelo tile assado.
pub mod motion_glow_layer;
/// ADR-0154: the shell half of `source.shape` — build each shape's `VecPath` from
/// its node params, publish it into the cook, and draw the cooked instances as
/// live GPU vector into the shared vector scene.
pub mod motion_lsystem_gen;
pub mod motion_lsystem_leaves;
pub mod motion_lsystem_rows;
#[cfg(test)]
#[path = "motion_lsystem_testkit.rs"]
pub mod motion_lsystem_testkit;
/// A trajetória do objeto selecionado no canvas (ADR-0141, Fatia 3).
pub mod motion_path_overlay;
pub mod motion_shape_gen;
pub mod motion_table_gen;
pub mod motion_text_gen;
/// O gizmo de canvas dos deformadores de quadrilátero (Corner Pin + Bezier Warp).
pub mod warp_gizmo;
/// As FIXTURAS do gizmo de warp — montadas e **não marchadas**; ver o cabeçalho delas.
/// ⚠️ `pub` porque o portão da costura vive dentro do `motion_bridge::gpu`.
#[cfg(all(test, feature = "panel-motion-graph"))]
pub mod warp_gizmo_fixtures;
/// A sonda que diz POR QUE o gizmo do warp nao existe — ver o cabecalho dela.
#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "warp_gizmo_probe.rs"]
pub mod warp_gizmo_probe;
/// O DESENHO desse gizmo — o contorno, os braços e as alças.
pub mod warp_overlay;

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
