/// O que o bloco de presets do painel pediu e só a ponte da app pode fazer (disco, biblioteca, a
/// política dos volumes). O tool regista; `ph2d-app-painter/src/painter_bridge_presets.rs` executa.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PedidoPreset {
    /// Escolher o preset neste caminho da biblioteca.
    Escolher(String),
    AlternarFavorito,
    GuardarComoNovo,
    GuardarComoPadrao,
    Repor,
    /// A política dos volumes, pelo índice em `Politica::TODAS`.
    Politica(u8),
    /// Ficheiros de preset para a pasta do artista (a ponte abre o seletor).
    Importar,
    /// O preset activo num ficheiro portátil (a ponte abre o seletor).
    Exportar,
    /// Apagar o preset activo — só o do artista; o painel já confirmou.
    Apagar,
    /// A ordem do navegador volta à por omissão.
    ReporPosicoes,
    /// `arrastado` passa para antes (ou `depois`) de `alvo` no navegador.
    Mover {
        arrastado: String,
        alvo: String,
        depois: bool,
    },
}
