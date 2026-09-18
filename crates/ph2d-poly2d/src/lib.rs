//! ⭐⭐⭐ **ONDE A TINTA ACABA, E COMO ISSO VIRA TRIÂNGULOS** — a malha de uma imagem.
//!
//! Esta crate responde a **uma** pergunta: *dada a cobertura de uma imagem, que polígono a
//! contorna e em que triângulos ele se divide?* Nada aqui sabe o que é um osso, uma sprite ou uma
//! cena — é geometria pura sobre uma grelha de alfa.
//!
//! # Porque ela existe (pesquisa do estado da arte, 2026-09-09)
//!
//! O campo inteiro deixou de mandar o artista construir a malha: o *Create Hull* do Spine, a
//! *Automatic Mesh Generation* do Live2D, o *Plastic* do OpenToonz e o **Puppet** do After Effects
//! derivam-na todos do recorte da própria tinta. ⛔ **A porta aberta que temos — o Godot, MIT — é
//! o contra-exemplo medido**: `Create Internal Vertex` põe um vértice de cada vez e
//! `Paint Bone Weights` pinta o peso à mão, sem nenhum automático. Portá-la seria portar o
//! trabalho.
//!
//! # A ordem das três leis, e porque nenhuma pode trocar de sítio
//!
//! ```text
//! alfa ──▶ [contour]  anéis de PIXEL, ordenados     (a fronteira, exacta)
//!      ──▶ [simplify] menos pontos, mesma silhueta  (o orçamento do artista)
//!      ──▶ [triangulate] triângulos                 (o que a deformação move)
//! ```
//!
//! ⚠️ **Simplificar ANTES de triangular, nunca depois.** Um anel de pixels tem um vértice por
//! passo de grelha — uma sprite de `512 px` de largura entrega ~2 000 pontos, e triangulá-los dá
//! ~2 000 triângulos que o artista nunca pediu e que a deformação tem de percorrer todo o quadro.
//! Simplificar depois seria colapsar triângulos já construídos, que é o problema difícil.
//!
//! # ⚠️ O irmão em `f32` que vive atrás de uma parede de `wgpu`
//!
//! O [`triangulate`] daqui tem um gémeo: `ph2d_flip_render::fill::triangulate`, o mesmo
//! *ear-clipping* em `Vec2` de `f32`. **Uma lei em dois sítios ainda não é uma lei** — e a razão
//! de ele não ser reusado está medida: aquela crate depende de **`wgpu`**, e este leaf serve o
//! esqueleto, que tem de correr sem GPU nenhuma (é a mesma parede que fez nascer a `ph2d-affine`).
//!
//! ⇒ **A unificação é devida, e o instrumento que a decide é NOMEADO:** mover a lei para cá obriga
//! o Flip a converter `f32 → f64` na entrada, e o *ear-clipping* decide por produtos cruzados —
//! uma decisão degenerada pode virar de sinal e mudar um golden daquele módulo. *A medição que
//! autoriza a fusão é correr os goldens do Flip com a conversão posta*, e ela é de quem possui o
//! Flip. ⛔ Até lá as duas ficam, com esta nota nos dois lados.

#![forbid(unsafe_code)]

mod attr_law;
/// ⭐ **A sub-malha dentro de um rectângulo** — o corte que o 9-slice presa a ossos exige.
mod clip_rect;
mod contour;
mod grid;
mod mesh;
mod refine;
mod refine_adaptive;
mod simplify;
mod triangulate;

pub use attr_law::{AttrLaw, hermite_attrs, recover_gradients};
pub use clip_rect::submesh_in_rect;
pub use contour::contour;
pub use grid::{Cobertura, GridOptions, axis_samples, grid_mesh_com, grid_mesh_of};
pub use mesh::{Mesh2d, MeshOptions, mesh_of};
pub use refine::{
    DeformAttrs, RefineLaw, RefineOptions, RefineReport, deviation, deviation_attrs,
    deviation_with, max_split, refine_posed, refine_posed_attrs, refine_posed_uniform,
    refine_posed_with, splits_for,
};
pub use refine_adaptive::{refine_posed_adaptive, refine_rest_by_attrs};
pub use simplify::simplify;
pub use triangulate::triangulate;

/// **A área com SINAL de um anel fechado** — positiva em sentido anti-horário.
///
/// ⚠️ **Porta única**, e é por isso que é `pub`: o [`triangulate`] usa-a para normalizar a
/// orientação, o [`contour`] para ordenar os anéis por tamanho, e um gate mede-a directamente.
/// Escrita três vezes, ela divergiria no primeiro `>=` que alguém trocasse por `>`.
///
/// ⚠️ **O eixo `y` desta crate aponta PARA BAIXO** (é a convenção de uma imagem), então o que aqui
/// se chama *anti-horário* vê-se no ecrã como horário. O nome segue a matemática porque é ela que
/// o *ear-clipping* usa; o que importa é as três leis concordarem, e concordam por usarem esta.
#[must_use]
pub fn signed_area(ring: &[[f64; 2]]) -> f64 {
    let n = ring.len();
    if n < 3 {
        return 0.0;
    }
    let mut acc = 0.0;
    for i in 0..n {
        let a = ring[i];
        let b = ring[(i + 1) % n];
        acc += a[0].mul_add(b[1], -(b[0] * a[1]));
    }
    acc / 2.0
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "refine_tests.rs"]
mod refine_tests;
