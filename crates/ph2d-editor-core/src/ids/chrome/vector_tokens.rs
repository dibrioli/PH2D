//! **Os ids do BINDING DE TOKEN** (plano UI/UX §4/W4) — irmão do [`super::vector`], que está a 16
//! linhas do teto de 700 LOC.
//!
//! O corte é por ASSUNTO: aqui mora *que propriedade desta forma segue um token, e qual*.

use ph2d_a11y::NodeId;

use super::super::hash_node_id;
use super::painter::fnv_node_id_runtime;

/// **O chip de token do PREENCHIMENTO** — abre a lista, e mostra o token vigente (ou `—`).
///
/// Fica ao lado da swatch de Fill, e não numa seção à parte, porque é isso que responde à pior
/// pergunta que esta feature pode gerar: *"por que a cor que eu escolhi não aparece?"*. Um valor
/// que não obedece ao que se digita e não diz por quê é a pior UI possível.
pub const VECTOR_TOKEN_FILL: NodeId = hash_node_id("vector.token.fill");

/// **O chip de token do TRAÇO** — idem, ao lado da cor do traço.
///
/// ⚠️ Só é pintado quando a seleção TEM traço: o token de traço colore o traço que existe e não
/// inventa largura (ver `VecPath::painted`), então oferecê-lo sem traço seria um controle que o
/// artista escolhe e que não muda um pixel.
pub const VECTOR_TOKEN_STROKE: NodeId = hash_node_id("vector.token.stroke");

/// **O chip de token da ESPESSURA do traço** (W4c.4) — logo abaixo do slider *Width*.
///
/// ⚠️ Oferecido sob a MESMA condição do chip de cor do traço, e pela metade que falta: um token de
/// largura numa forma sem traço teria de inventar a COR.
pub const VECTOR_TOKEN_WIDTH: NodeId = hash_node_id("vector.token.width");

/// **O chip de token do VÃO principal** do auto layout (W4c.4).
pub const VECTOR_TOKEN_GAP_MAIN: NodeId = hash_node_id("vector.token.gap.main");

/// **O chip de token do VÃO transversal** do auto layout (W4c.4).
pub const VECTOR_TOKEN_GAP_CROSS: NodeId = hash_node_id("vector.token.gap.cross");

/// **De que TABELA de tokens um slot se serve.**
///
/// ⚠️ A pergunta é de UI (*que lista o picker pinta?*) e por isso mora aqui, e não ao lado do
/// `BoundProp` no `ph2d-ecs`: o painel **não depende do ECS**, e a shell depende dos dois. Uma
/// cópia em cada lado ofereceria cores para escolher uma espessura no dia em que só uma delas
/// ganhasse um membro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenTable {
    /// `ph2d_tokens::ColorToken` — a tabela de COR.
    Colour,
    /// `ph2d_tokens::NumToken` — a escala em PIXELS (`spacing.*`/`radius.*`/`stroke.*`).
    Length,
}

impl TokenTable {
    /// Quantos tokens esta tabela lista. ⚠️ Vem da tabela, nunca de um literal: um token novo entra
    /// na macro do `ph2d-tokens` e nasce registado, pintado e clicável.
    #[must_use]
    pub fn len(self) -> usize {
        match self {
            Self::Colour => ph2d_tokens::ColorToken::ALL.len(),
            Self::Length => ph2d_tokens::NumToken::ALL.len(),
        }
    }

    /// A tabela está vazia? (Nunca está — o clippy pede o par do `len`.)
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// A CHAVE do `i`-ésimo token — a identidade que o arquivo guarda.
    #[must_use]
    pub fn key(self, i: usize) -> Option<&'static str> {
        match self {
            Self::Colour => ph2d_tokens::ColorToken::ALL.get(i).map(|t| t.key()),
            Self::Length => ph2d_tokens::NumToken::ALL.get(i).map(|t| t.key()),
        }
    }

    /// A posição da chave nesta tabela — a inversa da [`TokenTable::key`].
    #[must_use]
    pub fn position(self, key: &str) -> Option<usize> {
        (0..self.len()).find(|&i| self.key(i) == Some(key))
    }
}

/// **Um slot bindável, como a UI o vê**: o código de arquivo, o chip que o abre, e a lista que o
/// picker dele mostra.
#[derive(Clone, Copy, Debug)]
pub struct TokenSlot {
    /// ⚠️ **É o discriminante do `ph2d_ecs::BoundProp`**, e não um número paralelo — a shell o
    /// converte de volta por `BoundProp::from_code`. Dois numeradores seriam duas listas a
    /// envelhecer, e a que ficasse para trás produziria um clique que não chega a lado nenhum.
    pub code: u16,
    /// O chip que abre o popover deste slot.
    pub chip: NodeId,
    /// A tabela que o popover lista.
    pub table: TokenTable,
}

/// **A LISTA — e ela é a única.**
///
/// Cinco consumidores a percorrem: o `populate` (registar chip + opções), o `paint` (a lista), o
/// `selected_row` (que linha destacar), o roteamento de `Click` do painel, e o `token_choice` da
/// shell (id → alvo + token). ⚠️ Antes da W4c.4 cada um deles trazia o seu próprio
/// `[(0, Fill), (1, Stroke)]` escrito à mão; com uma segunda FAMÍLIA a entrar, o que um deles
/// esquecesse viraria **um chip pintado, com hit-rect, e morto sob o mouse** — o defeito que este
/// painel já pagou quatro vezes.
pub const TOKEN_SLOTS: &[TokenSlot] = &[
    TokenSlot {
        code: 0,
        chip: VECTOR_TOKEN_FILL,
        table: TokenTable::Colour,
    },
    TokenSlot {
        code: 1,
        chip: VECTOR_TOKEN_STROKE,
        table: TokenTable::Colour,
    },
    TokenSlot {
        code: 2,
        chip: VECTOR_TOKEN_WIDTH,
        table: TokenTable::Length,
    },
    TokenSlot {
        code: 3,
        chip: VECTOR_TOKEN_GAP_MAIN,
        table: TokenTable::Length,
    },
    TokenSlot {
        code: 4,
        chip: VECTOR_TOKEN_GAP_CROSS,
        table: TokenTable::Length,
    },
];

/// O slot de um código — `None` se o código não é de slot nenhum.
#[must_use]
pub fn token_slot(code: u16) -> Option<TokenSlot> {
    TOKEN_SLOTS.iter().copied().find(|s| s.code == code)
}

/// O slot de um CHIP — a busca que o painel faz, para pintar uma row sem repetir o código.
///
/// ⚠️ É esta porta que torna impossível pintar o chip de um slot com o código de outro: o par
/// viaja junto na tabela, e o sítio de pintura nomeia só o chip.
#[must_use]
pub fn token_slot_of(chip: NodeId) -> Option<TokenSlot> {
    TOKEN_SLOTS.iter().copied().find(|s| s.chip == chip)
}

/// A opção `i` no popover de tokens da propriedade `prop`.
///
/// ⚠️ **Derivado do ÍNDICE na lista `ColorToken::ALL`, e o índice é de RUNTIME** — ele nunca toca
/// o documento (lá a identidade é a CHAVE do token). Um id derivado do índice num arquivo teria o
/// mesmo defeito que a chave existe para evitar: reordenar a lista mudaria o significado.
///
/// `i == 0` é a linha **Unbind** — soltar a propriedade; as demais são `ColorToken::ALL[i - 1]`.
#[must_use]
pub fn vector_token_option_id(prop: u16, i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.token.opt.{prop}.{i}"))
}
