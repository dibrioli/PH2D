//! **O que da família `physics` ficou na shell** (W2/L2 Fase C, 2026-09-12).
//!
//! ⚠️⚠️ **Nenhum destes ficou por causa da `App` como TIPO** — e essa é a resposta que esta linha
//! deve às outras. As 5 portas do [`ph2d_app_host::AppHost`] cobriram todos os gestos; o que
//! prende o que sobra são **duas coisas da COMPOSIÇÃO**, e nenhuma delas é curável por uma porta:
//!
//! | prende | ficheiros | o que é |
//! |---|---:|---|
//! | `init::build_component_registry` + `component_attach::attach_by_name` | 9 | a **porta de produção** do `+` do Inspector (ADR-0166 / F3) |
//! | `undo::ProjectState` + `project_library::LibraryDoc` | 1 | a unidade de undo desta shell |
//! | o prólogo (`impl crate::App`) | 1 | rebobinar · armar o toggle · abrir a timeline · play/pause |
//!
//! ⭐ **O `build_component_registry` regista os componentes de CINCO crates irmãs**
//! (`ph2d_render`, `ph2d_physics_ecs`, `ph2d_field_ecs`, `ph2d_skeleton_ecs` + os do `ph2d-ecs`):
//! ele é **composição**, pela mesma lei que mantém o `render_loop` aqui. E os gates que o
//! atravessam fazem-no de propósito — *«um atalho de teste que constrói o componente por outro
//! caminho é a segunda porta que diverge»* —, logo eles moram com o que EXERCITAM (HOWTO §2.6),
//! e não com o sujeito. ⛔ Isto não é dívida: é a metade-shell de gates que medem a shell.
//!
//! ⚠️ **O que mudou desde a Fase B, e é a lei do `CLAUDE.md` §0.0 a funcionar:** aquela tabela
//! nomeava `render_loop::inspector_ordering`, `preview_drive` e `name_unique` como as três folhas
//! que prendiam 19 ficheiros. A `line/shell-folhas` fez-nas crates em 12/09 e as três
//! **dissolveram** — medido, `0` citações de código a partir daqui. *Quem move o número que
//! tornava algo inalcançável tem de reconferir a nota.*
//!
//! ⇒ Os outros 48 ficheiros desta pasta vivem hoje em [`ph2d_app_physics`], mais o
//! `render_loop/point_gizmo{,_tests}.rs`, que era física com nome genérico.

// ─────────────────────────────────────────────────────────────────────────
// **O PRÓLOGO.** `impl crate::App`: ele mexe no `Playhead`, nas `flags` da
// timeline e no `HeroScreen` — três coisas da COMPOSIÇÃO, não da família.
// *O que sai são os CORPOS; o que decide a ordem do quadro fica* (HOWTO §4).
// ─────────────────────────────────────────────────────────────────────────
pub(crate) mod physics_smoke;

// ─────────────────────────────────────────────────────────────────────────
// **Os INVÓLUCROS dos gestos de junta** — a costura `&mut App` → a assinatura
// da crate. ⚠️ Padrão de CHEGADA, não dívida (`ESTADO_W2` §2).
// ─────────────────────────────────────────────────────────────────────────
pub(crate) mod joint_gestures_app;

// ─────────────────────────────────────────────────────────────────────────
// **Os gates da §14 do Inspector do player.** O sujeito é o painel, que vive
// na [`ph2d_app_physics::inspector::player`]; o que eles exercitam é a PORTA
// DE PRODUÇÃO desta shell (`component_attach::attach_by_name` sobre o registo
// do `init`). ⚠️ Um `#[cfg(test)]` é invisível do outro lado da fronteira de
// crate (HOWTO §2.5), e o `attach_player` — o helper que os 29 gates
// partilham — atravessa a porta real de propósito.
// ─────────────────────────────────────────────────────────────────────────
#[cfg(test)]
#[path = "inspector_player_tests.rs"]
pub mod inspector_player_tests;

// ─────────────────────────────────────────────────────────────────────────
// **Os gestos COMPOSTOS da §11** — uma sequência de cliques produz uma coisa
// que funciona? Mesma porta de produção, mesmo motivo para ficarem.
// ─────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod physics_gesture_tests;
#[cfg(test)]
mod physics_gesture_zone_tests;

// ─────────────────────────────────────────────────────────────────────────
// **A ponte do bake contra o ARQUIVO.** Ele mede o `crate::undo::ProjectState`
// e o `crate::project_library::LibraryDoc` — a captura desta shell —, que é o
// terceiro caso da tabela do HOWTO §2.6: *o gate que mede a `App`, o
// `ProjectState` ou o arnês do ponteiro fica na shell.*
// ⚠️ O `bridge.rs` que ele exercita MUDOU-SE (é hoje
// [`ph2d_app_physics::bridge::dispatch`]): o sujeito e o arnês separam-se, e
// isso é a espécie normal desta wave, não um defeito.
// ─────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod bridge_tests;
