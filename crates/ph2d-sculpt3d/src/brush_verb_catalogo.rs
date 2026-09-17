//! **A ORDEM EM QUE A UI LISTA OS VERBOS** — a lista, e só ela.
//!
//! Filho (`#[path]`) do [`super`], como o [`super::grip_por_verbo`] e o
//! [`super::dyntopo`]. O corte é de ASSUNTO: o pai diz *que verbos existem e o
//! que cada um significa*; aqui fica *a ordem em que o artista os encontra na
//! fileira* — e é essa lista que a contagem do catálogo LÊ.
//!
//! ⚠️ **A contagem não está escrita em prosa em sítio nenhum**, e é de propósito:
//! o cabeçalho do pai já disse *«dezassete»* com dezanove na lista. Quem quer o
//! número conta esta lista, que é a fonte — e os gates do painel comparam-na com
//! o array de chips, que é o que impede um verbo novo de nascer inalcançável.

use super::verb::Verb;

impl Verb {
    /// Todos, na ordem em que a UI os lista.
    pub const ALL: [Self; 35] = [
        Self::Draw,
        Self::Inflate,
        Self::Smooth,
        Self::Sharpen,
        Self::Flatten,
        Self::Fill,
        Self::Scrape,
        Self::Clay,
        Self::Pinch,
        Self::Magnify,
        Self::Crease,
        Self::Blob,
        Self::Mask,
        Self::Move,
        Self::SnakeHook,
        Self::Twist,
        Self::LocalScale,
        Self::ClayStrips,
        Self::ClayThumb,
        Self::MultiplaneScrape,
        Self::SlideRelax,
        Self::SurfaceSmooth,
        Self::Layer,
        Self::Cloth,
        Self::Thumb,
        Self::Nudge,
        Self::Pose,
        Self::Boundary,
        Self::Density,
        Self::EraseMultires,
        Self::SmearMultires,
        Self::SceneProject,
        Self::BoxTrim,
        Self::Plane,
        Self::DrawSharp,
    ];
}
