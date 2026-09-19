//! ⭐⭐⭐ **A família `skeleton` da shell, fora da shell** — W2/L4 Fase C.
//!
//! # Porque ela é família PRÓPRIA, e não parte da `vec`
//!
//! O Esqueleto é **módulo desde o [ADR-0169]** (*«the skeleton is its own module, and each medium
//! answers only what a point is»*), com quatro crates de motor já suas — [`ph2d_affine`],
//! [`ph2d_skeleton`], [`ph2d_skeleton_ecs`], [`ph2d_skeleton_render`] — e painel próprio
//! ([`ph2d_panel_skeleton`]). Os nomes canónicos dele são **sem «vec»**, de propósito: um osso
//! deforma uma forma vectorial *e* uma imagem, e amanhã uma malha.
//!
//! ⇒ a metade-shell dele é uma família da W2 como as outras cinco, e não um apêndice da `vec`.
//! O que a `line/app-vec` tirou daqui na 2.ª volta foi a **lei pura** ([`ph2d_skeleton_live`]);
//! esta crate é a metade de **AUTORIA** — o gesto que nasce um osso, o pick, o limite de ângulo,
//! a âncora de IK e os Smart Bones.
//!
//! # ⛔⛔ O CICLO que decide de quem esta crate pode depender
//!
//! Ela **não pode** ser a [`ph2d_skeleton_ecs`], e o motivo foi medido na 2.ª volta: prender uma
//! forma aos ossos precisa da [`ph2d_vec_entities`], **que já depende** da `ph2d-skeleton-ecs`.
//! Uma família é um consumidor de motores, nunca um motor — e é isso que a mantém acíclica.
//!
//! # ⚠️ O que FICOU na shell, e porquê
//!
//! | ficheiro | razão |
//! |---|---|
//! | `bone_smart_probe.rs` · `bone_undo_probe.rs` | são `impl crate::App` — sondas de diagnóstico que leem o `gfx` inteiro |
//! | `skeleton_live.rs` · `skeleton_skin_image.rs` | fachadas de uma linha para [`ph2d_skeleton_live`], que mantêm os sítios de chamada byte a byte iguais |
//! | `skeleton_live_tests.rs` | o sujeito dele é a **sequência do smoke**, que é da shell |
//! | `skeleton_shell_seam_tests.rs` | dois gates cujo sujeito é a `ProjectState::capture` e o gizmo de grupo — *o teste segue o SUJEITO* (HOWTO §2.6) |
//!
//! # ⚠️ A família NÃO declara roteador, e a ausência é DECLARADA
//!
//! Ver [`FAMILY`]: a cena do osso é `PH2D_VEC_BONE_SMOKE` e o corpo dela vive na
//! `ph2d_app_vec::smoke_bone` — ⛔ **uma família não declara o roteador de outra**, senão o
//! registo tem duas respostas para a mesma env. Um `routers: &[]` com o nome na catraca é a forma
//! de dizer *«não tenho»* em voz alta.

pub mod bone_gesture;
pub mod bone_limit;
pub mod bone_pick;
pub mod bone_pose;
pub mod curve_tip;
pub mod goal;
/// ⭐ **A pose AUTORADA de uma corrente** — irmão do [`goal`] pelo tecto de LOC, cortado por
/// responsabilidade: *quem resolve* e *o que o artista desenhou* são duas perguntas.
mod goal_authored;
/// ⭐⭐⭐ O que cada NÚMERO do painel significa para um osso — a tradução e a aplicação.
pub mod knobs;
pub mod reveal;
pub mod skin_law;
pub mod smart;
pub mod state;

/// **A família, para o registo gerado** ([`ph2d_app_host`]).
///
/// ⛔⛔ **`routers: &[]` é uma AUSÊNCIA DECLARADA, não uma omissão.** A única cena de smoke que
/// exercita esta família é a `PH2D_VEC_BONE_SMOKE`, e o corpo dela vive na
/// `ph2d_app_vec::smoke_bone` desde a 2.ª volta da Fase B — porque a cena monta um **braço
/// vectorial** e prende-o, logo ela é da `vec` tanto quanto é do esqueleto, e quem a possui é quem
/// a constrói.
///
/// ⚠️ **Declará-la aqui TAMBÉM seria o defeito**: o registo passaria a ter duas famílias a
/// responder pela mesma variável de ambiente, e o gate `no_two_families_claim_the_same_router`
/// existe exactamente para isso. ⇒ o nome desta família entra em
/// `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL`? **Não** — ele não está na shell, está na crate irmã.
/// A catraca mede *«o roteador ainda está na shell»*, e a resposta aqui é não.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "skeleton",
    routers: &[],
};
