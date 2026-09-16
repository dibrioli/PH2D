//! **O NOME QUE A INTERFACE MOSTRA** para cada verbo — irmão (`#[path]`) do
//! [`super::brush_verb`].
//!
//! ⚠️ **O corte foi por RESPONSABILIDADE e forçado pelo tecto de LOC** (o
//! ficheiro chegou a `704` com o [`crate::Verb::BoxTrim`]): ali fica *que verbos
//! existem e o que cada um FAZ*; aqui fica *como cada um se CHAMA*, que é
//! vocabulário de interface. ⛔ Nunca uma entrada no `FILE_OVERAGE_OK`.
//!
//! ⚠️ **A lista é EXAUSTIVA de propósito** (`match self` sem `_ =>`): um verbo
//! novo sem nome é **erro de compilação**, e não um chip com o rótulo do
//! vizinho.

impl crate::Verb {
    /// O nome que a UI mostra.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::BoxTrim => "Box Trim",
            // ⚠️ **«Plane», e o nome é uma decisão de PRODUTO com duas cercas:**
            // (a) o dono pediu-o pela palavra *trim*, e esta casa já tem um
            // **Box Trim** que é outra coisa inteira (uma booleana no largar) —
            // dois chips com *trim* no nome seriam duas ferramentas
            // indistinguíveis na fileira, que é o defeito que o glifo
            // obrigatório de um painel existe para impedir; (b) o que este
            // pincel faz é ajustar um PLANO e puxar para ele, e é isso que o
            // artista precisa de ler para saber que os dois tectos são as duas
            // metades desse plano.
            Self::Plane => "Plane",
            Self::Cloth => "Cloth",
            Self::Draw => "Draw",
            Self::Inflate => "Inflate",
            Self::Smooth => "Smooth",
            Self::Sharpen => "Sharpen",
            Self::Flatten => "Flatten",
            Self::Fill => "Fill",
            Self::Scrape => "Scrape",
            Self::Clay => "Clay",
            Self::Pinch => "Pinch",
            Self::Magnify => "Magnify",
            Self::Crease => "Crease",
            Self::Blob => "Blob",
            Self::Mask => "Mask",
            Self::Move => "Move / Grab",
            Self::SnakeHook => "Snake Hook",
            Self::Twist => "Twist",
            Self::LocalScale => "Local Scale",
            Self::ClayStrips => "Clay Strips",
            Self::ClayThumb => "Clay Thumb",
            Self::MultiplaneScrape => "Multiplane Scrape",
            Self::SlideRelax => "Slide Relax",
            Self::SurfaceSmooth => "Surface Smooth",
            Self::Layer => "Layer",
            Self::Pose => "Pose",
            Self::Boundary => "Boundary",
            Self::Density => "Density",
            Self::EraseMultires => "Erase Displacement",
            Self::SmearMultires => "Smear Displacement",
            Self::SceneProject => "Scene Project",
            Self::Thumb => "Thumb",
            Self::Nudge => "Nudge",
        }
    }
}
