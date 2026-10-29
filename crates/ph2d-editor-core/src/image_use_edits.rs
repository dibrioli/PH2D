//! ⭐ **O VOCABULÁRIO DO USO DA IMAGEM** (planos GEF 13–16) — o seletor «Use» e a edição dele, num
//! módulo abaixo do [`crate::action_bus`] e do [`crate::screens`], pela MESMA razão do
//! [`crate::tags_edits`]: a catraca do DAG tolera `action_bus → screens` num tecto, e a cura que ela
//! escreve é o payload do Inspector descer para um módulo de vocabulário.

/// ⭐ O USO da imagem num objecto (plano GEF 13 D4; a lei é `ph2d_app_sprite_anim::image_use`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum InspectorImageUse {
    /// Uma imagem (ou uma célula dela) parada: o corte da imagem e a célula a mostrar.
    #[default]
    Still,
    /// A imagem repete-se (bloco, fundo): nenhum campo de quadros.
    Repeated,
    /// As células tocam: a secção Animation abre sob o seletor.
    Animated,
    /// Só uma área da imagem, escolhida no editor de Region (plano GEF 16).
    Region,
}

impl InspectorImageUse {
    /// A ordem do seletor «Use».
    pub const ALL: [Self; 4] = [Self::Still, Self::Repeated, Self::Animated, Self::Region];
}

/// ⭐ **Uma edição do USO da imagem** (planos GEF 14 D5 e 15 D3): o seletor «Use» sob a linha Image e o
/// tamanho de uma sprite que repete.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ImageUseEdit {
    /// O seletor **Still | Repeated | Animated | Region**.
    Use(InspectorImageUse),
    /// **Edit Region**: abre o editor da área (plano GEF 16 D2).
    EditRegion,
    /// O **OK** do editor de Region: a área `rect` (`[x, y, w, h]` em px da imagem) e a grelha de snap `grid`
    /// (colunas × linhas).
    SetRegion { rect: [u32; 4], grid: [u32; 2] },
    /// **Width** de uma sprite que repete, em cópias da imagem.
    Width(f32),
    /// **Height** de uma sprite que repete, em cópias da imagem.
    Height(f32),
}
