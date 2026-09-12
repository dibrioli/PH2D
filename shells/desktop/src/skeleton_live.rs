//! A re-exportação de [`ph2d_skeleton_live::skin_live`] — o módulo mudou de crate, o nome não.
//!
//! ⭐ A lei (prender uma forma ou uma imagem aos ossos, os segmentos, a corrente) é **pura** sobre
//! o `SimWorld` e saiu para a folha [`ph2d_skeleton_live`] na Fase B (2.ª volta), porque a cena
//! `PH2D_VEC_BONE_SMOKE` não podia sair da shell enquanto ela aqui estivesse — e a catraca dos
//! roteadores é **all-or-nothing por família**.
//!
//! ⛔ **Este ficheiro FICA** para que os ficheiros da shell que escrevem `crate::skeleton_live::…`
//! continuem byte a byte iguais (HOWTO §1.2/§1.4 — a reescrita uniforme é o que evita um mapa de
//! excepções).

pub(crate) use ph2d_skeleton_live::skin_live::*;

#[cfg(test)]
#[path = "skeleton_live_tests.rs"]
mod skeleton_live_tests;
