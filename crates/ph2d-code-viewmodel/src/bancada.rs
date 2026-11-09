//! ⭐ **A Bancada** (plano 13 Onda B): quem corre o script aberto, com que valores, e o que o script
//! tem — o que a família (`ph2d-code-bancada`) lê da cena e a pintura (`ph2d-panel-code`) mostra.

/// Um objecto que corre o script. `color` = o índice na paleta das camadas (`ColorToken::layer`),
/// pela ordem estável (`StableId`) dos objectos do MESMO script: a cor dele em toda a Bancada.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Holder {
    pub bits: u64,
    pub name: String,
    pub color: usize,
}

/// Como a célula se edita: `Choice` (texto com opções) passa à seguinte; `Color` edita-se no
/// Inspector (o clique selecciona o objecto).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellKind {
    Number,
    Bool,
    Text,
    Choice,
    Vec2,
    Color,
    Object,
}

/// Uma `ph2d.property` do texto: o nome, a linha (0-based), o tipo e o valor do script, escrito.
/// `kind = None`: a cena ainda não a conhece (o texto não foi salvo).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub line: usize,
    pub kind: Option<CellKind>,
    pub default: String,
    pub options: Vec<String>,
}

/// O valor de um objecto numa propriedade, escrito; `changed` = difere do valor do script.
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub text: String,
    pub changed: bool,
    pub swatch: Option<[f32; 4]>,
}

/// Uma entrada do «In this script»: o nome e a linha (0-based).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry(pub String, pub usize);

/// O contorno de um objecto (o índice dele) no MUNDO, em anéis já achatados: o mapa da Bancada.
pub type Outline = (usize, Vec<Vec<[f32; 2]>>);

/// Tudo o que a Bancada mostra do script aberto.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Who {
    pub holders: Vec<Holder>,
    pub columns: Vec<Column>,
    /// Objecto a objecto: `cells[h * columns.len() + c]`.
    pub cells: Vec<Cell>,
    pub functions: Vec<Entry>,
    /// Os sinais que o `on_signal` compara.
    pub hears: Vec<Entry>,
    pub map: Vec<Outline>,
}

impl Who {
    #[must_use]
    pub fn cell(&self, holder: usize, column: usize) -> Option<&Cell> {
        (column < self.columns.len())
            .then(|| self.cells.get(holder * self.columns.len() + column))
            .flatten()
    }
}

/// O que está sob o rato na Bancada: a frase do pé di-lo e o pairar acende-o.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Aim {
    #[default]
    Nothing,
    Cell(usize, usize),
    /// O nome de um objecto (grelha, balão, mapa, Console).
    Holder(usize),
    Column(usize),
    /// A marca da margem (◆ ou ⚡) da linha; o «Select» do balão; a entrada do «In this script».
    Mark(usize),
    Select(usize),
    Entry(usize),
}

/// O que um clique na Bancada pede: saltar para a linha (0-based), seleccionar o objecto, ou o valor
/// novo, escrito, do objecto `.0` na propriedade `.1`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Intent {
    Jump(usize),
    Select(usize),
    Edit(usize, usize, String),
}
