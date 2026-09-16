//! ⭐⭐ **O ESQUELETO VIVO NO DOCUMENTO** — prender uma forma (ou uma imagem) aos ossos, e
//! responder o que a corrente deles é.
//!
//! # Por que esta folha existe, e por que ela NÃO é a `ph2d-skeleton-ecs`
//!
//! O esqueleto é módulo próprio desde [ADR-0169], com a lei em `ph2d-skeleton` e os componentes em
//! `ph2d-skeleton-ecs`. O que vivia na shell era o degrau do meio: as operações **sobre o mundo**
//! — `bind`, `bind_image`, `bone_segments`, `chain_to` — e elas são **puras** (`SimWorld` mais
//! tipos de crates; zero `App`, zero `gfx`).
//!
//! ⛔⛔ **Elas não podem morar na `ph2d-skeleton-ecs`, e o motivo é um CICLO:** o `bind` precisa do
//! mapa `caminho ⟺ entidade` da [`ph2d_vec_entities`], e essa crate **já depende** da
//! `ph2d-skeleton-ecs` (o `settle_origins` dela salta uma forma presa a um osso). Pôr a lei lá
//! fecharia `skeleton-ecs → vec-entities → skeleton-ecs`, que o cargo recusa. *Uma folha nova não
//! é escolha de gosto quando a alternativa é um ciclo.*
//!
//! # O que a tirou da shell
//!
//! A cena `PH2D_VEC_BONE_SMOKE` é um dos **cinco** roteadores da família `vec`, e a catraca
//! `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` é **all-or-nothing por família**: enquanto uma cena
//! precisasse de um módulo da shell, **nenhum** dos cinco podia ser declarado. Esta crate é o que
//! fecha essa conta.
//!
//! ⚠️ **Os módulos da shell NÃO foram apagados** — `skeleton_live` e `skeleton_skin_image` ficaram
//! como re-exportação de uma linha, e os **21** ficheiros que os nomeiam continuam byte a byte
//! iguais ([HOWTO §1.2]).
//!
//! [ADR-0169]: ../../../docs/architecture/decisions/0169-the-skeleton-is-its-own-module-and-each-medium-answers-only-what-a-point-is.md
//! [HOWTO §1.2]: ../../../docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md

/// ⭐ A CURVATURA de um osso, viva — irmão do `skin_live` pelo tecto de LOC, cortado por assunto.
pub mod bend_live;
pub mod bone;
pub mod goal;
/// ⭐ O ORÇAMENTO de peças do quadro — irmão do `skin_image` pelo tecto de LOC, cortado por assunto.
pub mod skin_budget;
pub mod skin_image;
pub mod skin_live;
pub mod skin_refine;
pub mod skinned_mesh;

/// ⚠️ **Os auxiliares que ATRAVESSAM a fronteira, e nada mais** (HOWTO §2.5).
///
/// O gate `probe_the_smoke_sequence` ficou na SHELL porque o sujeito dele (a sequência do smoke,
/// que inclui uma POSE) ficou lá — e ele usa dois auxiliares que eram `#[cfg(test)]` desta crate.
/// Um `cfg(test)` é **falso** quando a crate é dependência, logo eles tinham de atravessar ou ser
/// copiados. ⛔ Copiar seriam duas definições da mesma régua, e a que envelhece é a de fora.
#[cfg(any(test, feature = "test-support"))]
pub mod test_support {
    use ph2d_ecs::SimWorld;
    use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

    /// Re-cozinha a cena e devolve o caminho `id` como ele ficou.
    pub fn quadro(sim: &SimWorld, scene: &mut VecScene, id: VecPathId) -> VecPath {
        crate::skin_live::recook(sim, scene);
        scene
            .paths()
            .iter()
            .find(|p| p.id == id)
            .expect("o path")
            .clone()
    }

    /// O maior desvio, em qualquer eixo, entre os vértices de dois estados do mesmo caminho.
    #[must_use]
    pub fn pior_desvio(a: &VecPath, b: &VecPath) -> f64 {
        a.verts_all()
            .zip(b.verts_all())
            .flat_map(|(x, y)| {
                [
                    (x.anchor, y.anchor),
                    (x.in_handle, y.in_handle),
                    (x.out_handle, y.out_handle),
                ]
            })
            .fold(0.0_f64, |m, (p, q)| {
                m.max((p[0] - q[0]).abs()).max((p[1] - q[1]).abs())
            })
    }
}
