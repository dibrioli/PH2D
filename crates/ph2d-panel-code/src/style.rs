//! ⭐ A cor de cada TIPO de realce — UMA porta: quem pinta pergunta aqui, nunca escolhe um token.

use ph2d_tokens::ColorToken;

/// O tipo de um trecho de código, já sem a gramática (a família traduz o realce para isto).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Ink {
    #[default]
    Plain,
    Keyword,
    Function,
    Variable,
    Parameter,
    Property,
    Type,
    String,
    Number,
    Constant,
    Comment,
    Operator,
    Punctuation,
    Builtin,
}

impl Ink {
    pub const ALL: [Ink; 14] = [
        Ink::Plain,
        Ink::Keyword,
        Ink::Function,
        Ink::Variable,
        Ink::Parameter,
        Ink::Property,
        Ink::Type,
        Ink::String,
        Ink::Number,
        Ink::Constant,
        Ink::Comment,
        Ink::Operator,
        Ink::Punctuation,
        Ink::Builtin,
    ];

    /// O token da cor deste tipo.
    #[must_use]
    pub fn token(self) -> ColorToken {
        match self {
            Ink::Plain | Ink::Variable | Ink::Parameter => ColorToken::Text1,
            Ink::Keyword => ColorToken::CodeKeyword,
            Ink::Function => ColorToken::CodeFunction,
            Ink::Builtin => ColorToken::CodeBuiltin,
            Ink::Type => ColorToken::CodeType,
            Ink::String => ColorToken::CodeString,
            Ink::Number => ColorToken::CodeNumber,
            Ink::Constant => ColorToken::CodeConstant,
            Ink::Property => ColorToken::CodeProperty,
            Ink::Operator => ColorToken::CodeOperator,
            Ink::Comment => ColorToken::CodeComment,
            Ink::Punctuation => ColorToken::Text2,
        }
    }
}
