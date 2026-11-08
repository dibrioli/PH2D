/// **Um quadro de uma lista, como o painel o vê** (plano GEF 26): a folha e a célula, ou o vazio que uma folha
/// apagada deixou (`sheet == Self::EMPTY_SHEET`). Espelho do `ph2d_ecs::FrameRef`, que esta crate não conhece.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimFrameRef {
    pub sheet: u8,
    pub cell: u32,
}

impl AnimFrameRef {
    /// O `sheet` do quadro vazio.
    pub const EMPTY_SHEET: u8 = u8::MAX;
    /// O quadro vazio.
    pub const EMPTY: Self = Self {
        sheet: Self::EMPTY_SHEET,
        cell: 0,
    };

    /// Uma célula da folha `0`.
    #[must_use]
    pub const fn cell(cell: u32) -> Self {
        Self { sheet: 0, cell }
    }

    /// É o buraco de uma folha apagada?
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.sheet == Self::EMPTY_SHEET
    }
}
