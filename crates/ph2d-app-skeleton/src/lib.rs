//! ⭐⭐⭐ **A família `skeleton` da shell, fora da shell** — W2/L4 Fase C.
//!
//! Módulo próprio desde o [ADR-0169]; esta crate é a metade de **AUTORIA** (o gesto que nasce um
//! osso, o pick, o limite, a âncora de IK, os Smart Bones); a lei pura é a [`ph2d_skeleton_live`].
//!
//! ⛔ Ela não pode ser a [`ph2d_skeleton_ecs`]: prender uma forma precisa da [`ph2d_vec_entities`],
//! que já depende dela (ciclo medido). Uma família consome motores, nunca é um.
//!
//! O que ficou na shell (sondas `impl crate::App`, fachadas, testes do smoke e da `ProjectState`) e
//! porquê: `docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md` §2.6.
//!
//! A família NÃO declara roteador ([`FAMILY`]): a cena do osso (`PH2D_VEC_BONE_SMOKE`) é da
//! `ph2d_app_vec::smoke_bone`.

pub mod bone_bridge;
pub mod bone_gesture;
pub mod bone_limit;
pub mod bone_pick;
pub mod bone_pose;
pub mod canais_osso;
pub mod curve_tip;
/// ⭐ *Mirror Branch* — o espelho do esqueleto, sem a IK do original.
pub mod espelho;
pub mod goal;
/// ⭐⭐⭐ O que cada NÚMERO do painel significa para um osso — a tradução e a aplicação.
pub mod knobs;
pub mod loose;
pub mod object_add;
pub mod reveal;
pub mod skeleton_mode;
pub mod skin_law;
pub mod smart;
pub mod state;
/// ⭐⭐⭐ O que cada BOTÃO do painel significa para um osso — e em que grandeza o efeito dele
/// se vê. O irmão do [`knobs`], e a casa do censo que pergunta se o clique chega a um efeito.
pub mod verbos;

/// ⭐ **A sonda que mede o custo REAL de «acrescentar um ponto com a caneta»** — a 1.ª das duas
/// saídas que o dono escolheu na F26. Ela corre antes de qualquer linha de cura.
#[cfg(test)]
#[path = "sonda_do_ponto_novo_tests.rs"]
mod sonda_do_ponto_novo_tests;

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
