//! ⭐⭐ **OS MATERIAIS DA PEÇA** — um por folha, e quem diz de que folha é cada ponto.
//!
//! Sobrou do sombreamento traçado do modo Render (retirado em 03/10): é a forma em que a tabela de
//! materiais do modelador entrega as superfícies ao desenhista de jogo e às leis da curvatura.

/// ⭐⭐⭐ **OS MATERIAIS DA PEÇA, e de quem é cada pixel** (`docs/Render3d/05`).
///
/// # ⚠️ Porque são DOIS campos e não um material só
///
/// Uma peça é uma árvore de folhas, e cada folha tem o seu aspecto. O que o traçado entrega é um
/// **ponto de mundo** por pixel ([`crate::Gbuffer::point`]); quem o traduz em *«a folha nº 3»* é a lei do
/// [`ph2d_field_eval::owners`], que é a MESMA que a selecção por clique usa.
///
/// ⚠️ **`owners: None` é o caso de UM**, e não um caso especial: uma peça com um material só não
/// precisa de perguntar de quem é o pixel.
///
/// ⛔ **A ordem de [`Self::all`] é a das folhas do [`ph2d_field_eval::owners::Owners`]**, e quem as
/// constrói constrói as duas — uma lista com outra ordem pintaria cada peça com a cor da vizinha,
/// sem erro nenhum.
pub struct Surfaces<'a> {
    /// Um material por folha. **Nunca vazio**.
    pub all: &'a [ph2d_material::Surface],
    /// De quem é cada ponto. `None` ⇒ a peça inteira usa `all[0]`.
    pub owners: Option<&'a ph2d_field_eval::owners::Owners>,
}
