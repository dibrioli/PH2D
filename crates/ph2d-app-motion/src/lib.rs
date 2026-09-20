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

/// ⭐⭐⭐ **A LEI DA APARÊNCIA, do lado do dispositivo** — uma corrente que não veio de uma forma
/// não vira pixel. A lei mora no avaliador; aqui está a metade que pergunta ao `MotionState`.
pub mod lei_da_aparencia;
pub mod ponto_gizmo;
pub mod ponto_gizmo_overlay;

/// ⭐ **A arte, em CPU, dos quads que o Motion desenha na cena vectorial** — a memoria da terceira
/// media. O unico modulo da familia que ja' era FECHADO: zero `crate::`, zero `App`, zero `gfx`.
pub mod motion_leaf_images;

// ─── a raiz da família (era `shells/desktop/src/motion.rs`) ───────────────────────
/// A auditoria do grupo do ciclo 2 (doc 105) — `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub mod motion_animadores_probe;
/// A auditoria do grupo do ciclo 7 (APARÊNCIA) — `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub mod motion_aparencia_probe;
pub mod motion_autofix_smoke;
pub mod motion_autofix_smoke_appropriate;
pub mod motion_autofix_smoke_dead_branch;
/// ⭐ **A VISTA chega ao tamanho** — o gate que prova que o `source.camera` serve (ciclo 8, W3).
#[cfg(test)]
#[path = "motion_camera_source_tests.rs"]
mod motion_camera_source_tests;
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
/// ⭐⭐ **A CAIXA DE CORREIO DO CARTÃO** — as intenções de param que sobem para a ponte. Ela veio
/// da crate do painel lateral quando ele saiu (doc 114 §13); era ela que tornava a remoção
/// impossível, porque o TIPO e a FILA moravam lá e quem os enchia era o CARTÃO.
pub mod motion_param_intent;
pub use motion_param_intent::{MotionParamIntent, drain_param_intents, push_param_intent};
/// ⭐ **O VOCABULÁRIO de uma row de param** — o que uma row É (`ParamRow` e as structs dela) e o
/// `ParamsSnapshot` que o `build_params_snapshot` produz. ⚠️ **Dado puro, zero imports**, e é por
/// isso que ele se moveu limpo: o painel PINTAVA-o, não o definia.
pub mod motion_param_rows;
pub use motion_param_rows::*;
/// ⭐ O COLISOR DECLARADO contra os DUPLICADORES — a ordem do dono de 2026-09-17 (doc 114 §12).
#[cfg(test)]
pub mod motion_colisor_duplicador_probe;
/// ⭐ O que um COZIMENTO de centenas de objectos custa na CPU (doc 115 §10.4) — medição, não gate.
#[cfg(test)]
#[path = "motion_cozimento_cpu_probe.rs"]
pub mod motion_cozimento_cpu_probe;
/// O CUSTO DO QUADRO da cena do dono (Boids + Shape + colisão) — doc 115 §20.
#[cfg(test)]
pub mod motion_custo_do_quadro_probe;
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
/// A auditoria do grupo do ciclo 8 (as FONTES) — `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub mod motion_fontes_probe;
pub mod motion_fx_smoke;
pub mod motion_node_path_smoke;
pub mod motion_object_bake;
/// Irmão do acima por RESPONSABILIDADE: ele produz pixels (GPU), este diz quantos (CPU).
pub mod motion_object_bake_dims;
pub mod motion_object_smoke;
/// Irmã das duas acima: reduz um assado ao cartão do painel.
pub mod motion_object_thumb;
/// ⭐ **O censo do ALCANCE** — um param que nenhuma combinação de gates revela (ciclo 6 W3).
/// `#[cfg(test)]`: é um instrumento de auditoria, não código de produto.
#[cfg(test)]
pub mod motion_param_reach;
pub mod motion_path_smoke;
/// ⭐ A COLISÃO no grupo do ciclo 9 — o report do dono de 2026-09-17, medido (doc 114 §11).
#[cfg(test)]
pub mod motion_rig_colisao_probe;
/// A auditoria do grupo do RIG e dos CORPOS MOLES (ciclo 9, passo 2 — doc 114) —
/// `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub mod motion_rig_probe;
/// O PREÇO do mesmo grupo — irmão do acima pelo tecto de LOC (ciclo 9, W4/W5).
#[cfg(test)]
pub mod motion_rig_relogio;
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
/// A auditoria do grupo do ciclo 6 (VALOR & PULSO) — `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub mod motion_valor_probe;

// ─── as duas âncoras que eram módulos de TOPO da shell e são código desta família ─────
/// ⭐ **O gizmo dos CAMPOS na tela.** Ele lê `MotionState` e `motion_bridge::params`, logo nunca
/// foi uma folha partilhada: era código do Motion com morada errada.
pub mod field_gizmo;
/// A cena mínima que julga o picker de colunas do `value.attribute` — **só a motion a consome**.
pub mod picker_smoke;

// ─── os 17 que eram filhos DIRECTOS do `render_loop/mod.rs` ──────────────────────
// ⚠️ O LAÇO ficou na shell (ele garante a ordem dos 48 símbolos); o que saiu foram os
// CORPOS, e a shell chama-os por `crate::<mod>::<fn>` no ponto certo.
/// ⭐⭐ O gizmo do COLISOR da forma (doc 109 §5) — os contornos e as alças no canvas.
pub mod collider_gizmo;
/// O DESENHO desse gizmo.
pub mod collider_gizmo_overlay;
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
/// ⭐⭐ O gizmo do PIVÔ da forma (ordem do dono, 2026-09-19) — o alvo que acende enquanto a mão
/// arrasta o `Pivot X`/`Pivot Y` no cartão.
pub mod pivot_gizmo;
/// O DESENHO desse gizmo.
pub mod pivot_gizmo_overlay;
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
/// ⭐⭐⭐ **`"motion"` SAIU da catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` em 2026-09-12, e
/// era a ÚLTIMA** — a lista fica VAZIA, e com ela o bloco e a metade `if` do gate.
///
/// # ⚠️ Os 38 roteadores, e porque eram 38 e não 9
///
/// O censo por **prefixo de ficheiro** via nove. Os outros **29** vivem em ficheiros sem o
/// prefixo `motion_`, soltos em `src/` — `value_*_smoke`, `splice_smoke`, `gradient_smoke`,
/// `lens_smoke`… ⛔ E a catraca é **all-or-nothing por família**, logo os 29 invisíveis
/// bloqueavam os 9 visíveis. *É exactamente o que a `vec` pagou em 12/09 («cinco, não os quatro
/// que o briefing listava»), e o instrumento que os achou foi o mesmo que quase os perdeu:*
/// ⚠️⚠️ **um censo de nomes de ENV tem de ler o código COM as strings** — um `env::var("PH2D_X")`
/// tem o nome DENTRO de uma string, e um stripper que as branqueia devolve **zero**.
///
/// # ⚠️ Cada `max_level` foi CONTADO no `match`, e três não eram o que pareciam
///
/// | roteador | lê-se | é | porquê |
/// |---|---:|---:|---|
/// | `PH2D_AUTOFIX_SMOKE` | 6 | **8** | o `_ =>` delega aos irmãos, e os modos 7 e 8 vivem lá |
/// | `PH2D_MOTION_NODE_PATH_SMOKE` | 6 | **4** | os `3 =>`/`6 =>` são números de **FRAME** |
/// | `PH2D_SHAPE_SMOKE` | 2 | **3** | o irmão `_knobs` responde ao 3 |
///
/// *Um censo que conta braços de `match` sem saber sobre O QUÊ se casa erra nos dois sentidos.*
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "motion",
    routers: &[
        ph2d_app_host::SmokeRouter {
            env: "PH2D_GPU_COOK_DEMO",
            max_level: motion_state::demo_router::MAX_DEMO_LEVEL,
        }, // ⭐ DERIVADO da constante, não escrito
        ph2d_app_host::SmokeRouter {
            env: "PH2D_MOTION_OBJ_SMOKE",
            max_level: 12,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_AUTOFIX_SMOKE",
            max_level: 8,
        }, // ⚠️ 8 e não 6: o `_ =>` delega aos IRMÃOS
        ph2d_app_host::SmokeRouter {
            env: "PH2D_MOTION_NODE_PATH_SMOKE",
            max_level: 4,
        }, // ⚠️ 4: o `mode()` mapeia {0,2,3,4}, o resto → 1
        ph2d_app_host::SmokeRouter {
            env: "PH2D_SHAPE_SMOKE",
            max_level: 3,
        }, // ⚠️ 3: o irmão `_knobs` responde ao 3
        ph2d_app_host::SmokeRouter {
            env: "PH2D_PATH_SMOKE",
            max_level: 2,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_ADAPTER_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_ATTR_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_DRIVEN_ROW_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_ECHO_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_EMITTER_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_GLOW_DIRT_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_GRADIENT_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_LENS_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_MOTION_DELAY_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_MOTION_FX_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_OSC_RULER_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_PICKER_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_SPLICE_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_TRANSFORM_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_UNITS_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_CURVE_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_GAIN_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_MEDIAN_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_MIX_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_NOISE_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_NORMALIZE_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_PATTERN_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_PERCENTILE_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_QUANTIZE_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_REDUCE_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_SLOPE_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_SMOOTH_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_STEP_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_TIME_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_UNARY_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_WAVE_SMOKE",
            max_level: 1,
        },
        ph2d_app_host::SmokeRouter {
            env: "PH2D_VALUE_WRAP_SMOKE",
            max_level: 1,
        },
    ],
};

// ─── o SEGUNDO cluster de cenas: 31 ficheiros que o censo por PREFIXO nunca viu ──────
// ⚠️ Elas não têm prefixo `motion_` e vivem soltas em `src/` — e é por elas que
// `"motion"` não podia sair da catraca: ela é **all-or-nothing por família**.
pub mod adapter_smoke;
pub mod attribute_demo_smoke;
pub mod driven_row_smoke;
pub mod echo_family_smoke;
pub mod emitter_smoke;
pub mod glow_dirt_smoke;
pub mod gradient_smoke;
pub mod lens_smoke;
/// ⭐ A porta única de ARRUMAR o documento, com cada cartão medido — ver o módulo.
mod motion_arrumar;
pub mod osc_ruler_smoke;
pub mod smoke_layout;
pub mod splice_smoke;
pub mod transform_family_smoke;
pub mod units_smoke;
pub mod value_curve_smoke;
pub mod value_gain_smoke;
pub mod value_median_smoke;
pub mod value_mix_smoke;
pub mod value_noise_smoke;
pub mod value_normalize_smoke;
pub mod value_pattern_smoke;
pub mod value_percentile_smoke;
pub mod value_quantize_smoke;
pub mod value_reduce_smoke;
pub mod value_slope_smoke;
pub mod value_smooth_smoke;
pub mod value_step_smoke;
pub mod value_time_smoke;
pub mod value_unary_smoke;
pub mod value_wave_smoke;
pub mod value_wrap_smoke;
