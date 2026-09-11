//! **A familia MOTION da shell** — as 269 raizes e folhas que viviam soltas em `src/`.
//!
//! ⚠️ **Porque existe (W2/L1, 2026-09-11):** agrupar a familia numa pasta faz o CORTE da Fase B
//! ser mover UMA pasta, em vez de escolher 269 ficheiros entre 1 784. A shell e' a ultima unidade
//! de todo build grande (`16,69 s` a frio das `51,38 s` do workspace, medido a `load 7`), e esta
//! familia e' a maior que la' vive: `100 352` LOC de `493 252`.
//!
//! ⚠️ **Os 243 `#[path]` internos NAO precisaram de UMA edicao:** sao nomes de ficheiro IRMAO, e um
//! `#[path]` resolve relativo ao directorio do ficheiro que o contem — mover o conjunto INTEIRO
//! preserva-os por construcao. Foi medido ANTES de mover: 243 resolvem dentro do conjunto, `0`
//! problemas, `0` apontadores de fora para dentro.
//!
//! ⚠️⚠️ **E as OITO raizes `#[cfg(test)]` sao load-bearing.** A 1.a tentativa desta wave colapsou as
//! 27 declaracoes numa so' e pos `mod motion;` debaixo do `#[cfg(test)]` da PRIMEIRA delas — a
//! familia inteira virou test-only e `15` ficheiros deixaram de a encontrar. *Um `mod` colapsado
//! herda o atributo do vizinho de cima, e o compilador diz «could not find `motion` in the crate
//! root», que nao aponta para a causa.*

/// A auditoria do grupo do ciclo 2 (doc 105) — `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub(crate) mod motion_animadores_probe;
pub(crate) mod motion_autofix_smoke;
pub(crate) mod motion_autofix_smoke_appropriate;
pub(crate) mod motion_autofix_smoke_dead_branch;
/// A auditoria do grupo do ciclo 4 (os CAMPOS) — `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub(crate) mod motion_campos_probe;
/// A outra metade dele — **o PREÇO** (relógio, contagem, dispositivo). Irmão pelo tecto de
/// LOC, cortado por responsabilidade: o que um nó DECLARA fica ali, o que ele CUSTA aqui.
#[cfg(test)]
pub(crate) mod motion_ciclo_preco;
/// ⭐ **O INSTRUMENTO DE UM CICLO**, para qualquer grupo — retrato, params, cartão,
/// nomes e vocabulário. Nasceu ao abrir o ciclo 4, quando ia ser copiado do 3.
#[cfg(test)]
pub(crate) mod motion_ciclo_probe;
/// O PREÇO do mesmo grupo — irmão do acima pelo tecto de LOC, cortado por
/// responsabilidade: retratos ali, relógio e dispositivo aqui.
#[cfg(test)]
pub(crate) mod motion_deformadores_preco;
/// A auditoria do grupo do ciclo 3 (doc 106) — `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub(crate) mod motion_deformadores_probe;
pub(crate) mod motion_delay_smoke;
/// **A legenda de uma cena de smoke, no canvas** (Enio 2026-08-23) — o rótulo pousa
/// em cima do caso que ele explica, em vez de num terminal atrás da janela.
pub(crate) mod motion_demo_legend;
pub(crate) mod motion_flip_bake;
pub(crate) mod motion_fx_smoke;
pub(crate) mod motion_leaf_images;
pub(crate) mod motion_node_path_smoke;
pub(crate) mod motion_object_bake;
/// Irmão do acima por RESPONSABILIDADE: ele produz pixels (GPU), este diz quantos (CPU).
pub(crate) mod motion_object_bake_dims;
pub(crate) mod motion_object_smoke;
/// Irmã das duas acima: reduz um assado ao cartão do painel.
pub(crate) mod motion_object_thumb;
pub(crate) mod motion_path_smoke;
/// O tile de uma forma PARAMÉTRICA (`source.shape`) — irmão do `motion_object_bake`,
/// e a metade que faz o glow alcançar as formas (bug do Enio, 2026-08-20).
pub(crate) mod motion_shape_bake;
pub(crate) mod motion_shape_smoke;
pub(crate) mod motion_shape_smoke_knobs;
/// O estado de shell da familia — os quatro campos que saíram da `App` na W2/L1 (A2b).
pub(crate) mod motion_shell_state;
/// A auditoria do grupo do ciclo 5 (a SIMULAÇÃO) — `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub(crate) mod motion_sim_probe;
/// A sonda do custo do carimbo (report do Enio, 2026-09-06) — `#[cfg(test)]`, não entra no bin.
#[cfg(test)]
pub(crate) mod motion_stamp_cost_probe;
pub(crate) mod motion_state;
