//! **Os dados do cartão User Data** (Pixel P12 U): a vista que o cartão mostra e o gesto que ele
//! devolve. Moram no assunto (o Pixel) e não no pintor do cartão (`panel::cards::user_data`), para o
//! `PixelPanelInfo`/`PixelRequest` não subirem ao `panel` (a catraca do DAG da fundação).

/// Uma linha da tabela, já escrita.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropRow {
    pub name: String,
    pub value: String,
    /// A CHAVE do nome do tipo no chip (`pixel.user.type.int` → `Int`, ou o do `.ase`).
    pub kind: &'static str,
    /// `false`: um tipo do `.ase` que a tela não edita; mostra-se, não se toca.
    pub editable: bool,
}

/// O que o cartão mostra.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserDataView {
    pub color: [u8; 4],
    pub text: String,
    /// O mapa do artista (`""`).
    pub rows: Vec<PropRow>,
    /// Os mapas de extensão, só de leitura, sob o nome da extensão.
    pub extensions: Vec<(String, Vec<PropRow>)>,
}

/// Um gesto do cartão em nomes de linha (o dono traduz para o documento; `Kind` roda o tipo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardEdit {
    Color([u8; 4]),
    Text(String),
    Add,
    Remove(String),
    Rename(String, String),
    Kind(String),
    Value(String, String),
}
