//! ⭐ **A aparência dos quadros acessórios** (decisão do dono 13, plano 14): o Stopped, os valores ao
//! vivo, o balão ◆, as opções e os Problems pintam-se como planilha (padrão), cartões ou faixa —
//! escolhida em `View ▸ Card Style`, a mesma para todos.

/// Uma das três aparências da maquete `docs/CodeEditor/maquetes/14_quadros_do_depurador.html`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CardStyle {
    /// A · planilha: Name · Value · Type, grupos dobráveis, Call stack ao lado.
    #[default]
    Sheet,
    /// B · cartões como o Inspector: um cartão por grupo, cada valor numa caixa de campo.
    Cards,
    /// C · faixa compacta: uma célula por variável, nome pequeno em cima, valor em baixo.
    Strip,
}

impl CardStyle {
    pub const ALL: [Self; 3] = [Self::Sheet, Self::Cards, Self::Strip];
}
