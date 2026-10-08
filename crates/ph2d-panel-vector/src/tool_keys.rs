//! ⭐⭐ **As TECLAS do vetor — o padrão ILLUSTRATOR** (D10 decisão 3, Enio 07/10: *«vamos com o mais
//! bem aceito»*). Fonte da tabela: `docs/UI_New_and_Simple/vetor_nova_ui/02_estudo_os_apps_amados.md`
//! §5 — o perfil `adobe-illustrator-cs2.xml` que o Inkscape instalado traz, e o hongkiat para as
//! que ele não cobre.
//!
//! ⚠️ **A ÚNICA fonte**: a shell traduz a tecla física para um [`Chord`] e pergunta aqui
//! ([`action_for`]); a dica de cada botão da coluna ① lê a mesma tabela ([`chord_for_mode`]). Uma
//! segunda tabela na shell envelheceria em silêncio ao lado desta.
//!
//! ⛔ `Fillet`, `Chamfer`, `Trim` e `Connect` **não têm tecla**: o Illustrator não as tem, e não se
//! inventa tecla fora do idioma (D10). ⛔ `Tab` continua a ser o MODO (Object ↔ Edit, 05/10).

use ph2d_tool_vector::DrawMode;

/// Uma tecla de letra com os modificadores que contam.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chord {
    /// A letra, minúscula (`'v'`, `'m'`…).
    pub letter: char,
    pub shift: bool,
    /// `Ctrl+Alt` — o acorde das booleanas (o idioma do Penpot; o Illustrator não tem tecla de
    /// fábrica para o Pathfinder).
    pub ctrl_alt: bool,
}

impl Chord {
    const fn key(letter: char) -> Self {
        Self {
            letter,
            shift: false,
            ctrl_alt: false,
        }
    }
    const fn shift(letter: char) -> Self {
        Self {
            letter,
            shift: true,
            ctrl_alt: false,
        }
    }
    const fn ctrl_alt(letter: char) -> Self {
        Self {
            letter,
            shift: false,
            ctrl_alt: true,
        }
    }

    /// Como a tecla se escreve numa dica: `P`, `Shift+M`, `Ctrl+Alt+U`.
    #[must_use]
    pub fn label(self) -> String {
        let l = self.letter.to_ascii_uppercase().to_string();
        match (self.shift, self.ctrl_alt) {
            (true, _) => ph2d_i18n::tr_with("panel.vector.key.shift", &[("l", &l)]),
            (_, true) => ph2d_i18n::tr_with("panel.vector.key.ctrl_alt", &[("l", &l)]),
            _ => l,
        }
    }
}

/// O que uma tecla faz.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    /// Pega numa ferramenta (a coluna ①).
    Mode(DrawMode),
    /// `M`: a Forma com o RECTÂNGULO do catálogo já escolhido.
    ShapeRect,
    /// `L`: a Forma com a ELIPSE do catálogo já escolhida.
    ShapeEllipse,
    /// `X`: o foco da cor passa de Preenchimento a Traço (e volta).
    ToggleFillStroke,
    /// `Shift+X`: troca as cores do Preenchimento e do Traço.
    SwapFillStroke,
    /// `D`: repõe as cores de fábrica.
    DefaultColors,
    /// `Ctrl+Alt+U/I/D/X` — as booleanas das duas últimas formas.
    Union,
    Intersect,
    Subtract,
    Exclude,
}

/// ⭐ **A tabela** (estudo §5).
pub const KEYS: [(Chord, KeyAction); 21] = [
    (Chord::key('v'), KeyAction::Mode(DrawMode::Select)),
    (Chord::key('a'), KeyAction::Mode(DrawMode::Node)),
    (Chord::key('p'), KeyAction::Mode(DrawMode::Pen)),
    (Chord::key('n'), KeyAction::Mode(DrawMode::Pencil)),
    (Chord::key('m'), KeyAction::ShapeRect),
    (Chord::key('l'), KeyAction::ShapeEllipse),
    (Chord::key('t'), KeyAction::Mode(DrawMode::Text)),
    (Chord::shift('m'), KeyAction::Mode(DrawMode::Build)),
    (Chord::shift('w'), KeyAction::Mode(DrawMode::Width)),
    (Chord::key('k'), KeyAction::Mode(DrawMode::Bucket)),
    (Chord::key('w'), KeyAction::Mode(DrawMode::PickBlend)),
    (Chord::key('c'), KeyAction::Mode(DrawMode::Cut)),
    (Chord::shift('o'), KeyAction::Mode(DrawMode::Frame)),
    (Chord::key('i'), KeyAction::Mode(DrawMode::Eyedropper)),
    (Chord::key('x'), KeyAction::ToggleFillStroke),
    (Chord::shift('x'), KeyAction::SwapFillStroke),
    (Chord::key('d'), KeyAction::DefaultColors),
    (Chord::ctrl_alt('u'), KeyAction::Union),
    (Chord::ctrl_alt('i'), KeyAction::Intersect),
    (Chord::ctrl_alt('d'), KeyAction::Subtract),
    (Chord::ctrl_alt('x'), KeyAction::Exclude),
];

/// A ação de uma tecla, ou `None` se ela não é do vetor.
#[must_use]
pub fn action_for(chord: Chord) -> Option<KeyAction> {
    KEYS.iter().find(|(c, _)| *c == chord).map(|(_, a)| *a)
}

/// A tecla de uma ferramenta, para a dica do botão dela — `None` = sem tecla (por decisão).
#[must_use]
pub fn chord_for_mode(mode: DrawMode) -> Option<Chord> {
    // A Forma não tem tecla própria no Illustrator: abre-se pelas das formas (`M` o rectângulo).
    let wanted = if mode == DrawMode::Shape {
        KeyAction::ShapeRect
    } else {
        KeyAction::Mode(mode)
    };
    KEYS.iter().find(|(_, a)| *a == wanted).map(|(c, _)| *c)
}

/// As ferramentas que ficam SEM tecla, por decisão (D10 — o Illustrator não as tem).
pub const KEYLESS: [DrawMode; 4] = [
    DrawMode::Fillet,
    DrawMode::Chamfer,
    DrawMode::Trim,
    DrawMode::Connect,
];

#[cfg(test)]
#[path = "tool_keys_tests.rs"]
mod tests;
