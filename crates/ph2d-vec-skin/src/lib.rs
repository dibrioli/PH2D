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

/// ⭐⭐⭐ **Os pesos do PADRÃO-OURO para uma forma vectorial** — a 2.ª mídia a deixar a lei
/// euclidiana derivada.
pub mod pesos;

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
    aplica_com(skin, path, &[]);
}

/// ⭐⭐⭐ **[`apply`] com os pesos do PADRÃO-OURO guardados no bind** — `pesos` vazio ⇒ a lei
/// derivada, que é o caminho de sempre **ao bit**.
///
/// A tabela é achatada e a ordem é a de [`VecPath::for_each_vert_mut`]: para o vértice `k`, as três
/// casas `3k`, `3k+1`, `3k+2` são **âncora**, **alça de entrada** e **alça de saída**.
///
/// ⚠️⚠️ **As três metades continuam a pesar SEPARADAMENTE** — é o `CubicWeight` do Rive, e a razão
/// está no cabeçalho: pesar o vértice inteiro pela âncora faria uma alça que atravessa uma junta
/// rodar com o osso errado. *A lei dos pesos mudou; o que é um ponto aqui, não.*
///
/// ⛔ **Uma tabela que não fecha com o caminho é IGNORADA** (cai na lei derivada) em vez de ser lida
/// deslocada: uma tabela deslocada por um ponto entrega pesos plausíveis e arte errada.
pub fn aplica_com(skin: &Skin, path: &mut VecPath, pesos: &[f64]) {
    let mut w = skin.scratch();
    let pontos = path.verts_all().count() * 3;
    let n = pesos.len().checked_div(pontos).unwrap_or(0);
    let usa = n > 0 && pesos.len() == pontos * n;
    let mut k = 0usize;
    path.for_each_vert_mut(|v| {
        for p in [&mut v.anchor, &mut v.in_handle, &mut v.out_handle] {
            *p = if usa {
                skin.point_with(*p, &pesos[k * n..(k + 1) * n], &mut w)
            } else {
                skin.point(*p, &mut w)
            };
            k += 1;
        }
    });
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
