#![forbid(unsafe_code)]
//! **O VETOR VESTE A PELE** — o 1.º cliente da [`ph2d_skeleton`] (estudo 42 item 5,
//! [doc 47](../../../docs/Vector%20Module/47_o_desenho_ganha_ossos.md)).
//!
//! A lei — *para onde vai um PONTO* — vive no módulo e não sabe o que é um caminho. O que vive
//! aqui é a única coisa que é do vetor: **o que é um ponto aqui**.
//!
//! # As três metades de um vértice skinam-se em SEPARADO
//!
//! Âncora, alça de entrada e alça de saída são **três pontos**, cada um com os pesos da posição
//! **dele**. É o `CubicWeight` do Rive, e é o que a `ph2d-vec-scene` prometia desde a ADR-0108
//! (*"os três skinados independentemente na Fase 1"*). Pesar o vértice inteiro pela âncora faria
//! uma alça que atravessa uma junta rodar com o osso errado.
//!
//! ⚠️ **Por que o LBS e não a `ph2d-vec-envelope`:** a mistura de afins ainda é um afim, então uma
//! Bézier deformada continua a ser uma Bézier — exacta e editável, sem reamostrar. O envelope paga
//! `sample + fit` porque o mapa dele não é afim.

use ph2d_skeleton::Skin;
use ph2d_vec_scene::VecPath;

/// **Deforma o caminho inteiro, em lugar** — âncora e as duas alças de todo vértice de todo
/// contorno.
///
/// ⚠️ **O `corner_radius` viaja INTACTO.** Ele é fonte (o raio que o cozimento resolve), e a
/// deformação de uma pele é localmente quase-rígida — escalá-lo pediria um factor por VÉRTICE,
/// que é a mesma conta da caneta do bug #27 (`√|det|`) mas com um afim diferente por ponto.
/// Fica **nomeado**, não esquecido.
pub fn apply(skin: &Skin, path: &mut VecPath) {
    let mut w = skin.scratch();
    path.for_each_vert_mut(|v| {
        v.anchor = skin.point(v.anchor, &mut w);
        v.in_handle = skin.point(v.in_handle, &mut w);
        v.out_handle = skin.point(v.out_handle, &mut w);
    });
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
