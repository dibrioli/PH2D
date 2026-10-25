//! **O vocabulário do cartão CONSTRAINTS** (plano `docs/BlenderConstraints/01_plano.md` §4): o que o
//! cartão mostra de cada item da pilha ([`InspectorConstraintsInfo`]) e o que ele pede
//! ([`ConstraintFieldEdit`]). Quem lê o mundo e aplica é `ph2d-app-constraints` (`inspector.rs`); o
//! painel só pinta e emite.

/// As entradas do menu **+**, na ordem em que aparecem: a chave `ph2d-i18n` do rótulo. O índice é o
/// que [`ConstraintFieldEdit::Adicionar`] leva e o que [`InspectorConstraintRow::tipo`] diz.
pub const TIPOS: [&str; 8] = [
    "panel.inspector.constraint.child_of",
    "panel.inspector.constraint.copy_location",
    "panel.inspector.constraint.copy_rotation",
    "panel.inspector.constraint.copy_scale",
    "panel.inspector.constraint.copy_transforms",
    "panel.inspector.constraint.look_at",
    "panel.inspector.constraint.limit_rotation",
    "panel.inspector.constraint.limit_distance",
];

/// Quantas linhas o cartão pinta e edita — o tamanho das tabelas de ids do painel
/// (`ids/inspector_constraint.rs`), o único recurso que o limita; a pilha em si não tem tecto. Cheio,
/// o **+** recusa com a frase `panel.inspector.constraint.full`.
pub const MAX_LINHAS: usize = 32;

/// Quantos números, escolhas e opções por escolha um item mostra debaixo da força — o tamanho das
/// tabelas de ids do painel; o tipo mais largo (Limit Rotation: Min, Max e o espaço) cabe.
pub const MAX_NUMEROS: usize = 2;
pub const MAX_ESCOLHAS: usize = 2;
pub const MAX_OPCOES: usize = 3;

/// A unidade em que um número do cartão se lê, se escreve e se anima.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unidade {
    Graus,
    Px,
}

/// Um número de um tipo (Min, Max, Distance, Offset). `campo` = o canal dele
/// (`ph2d_ecs::canal_constraint`) — é por ele que a edição chega ao item e o losango acha a curva.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AjusteNumero {
    pub campo: u16,
    /// A chave `ph2d-i18n` do nome.
    pub rotulo: &'static str,
    pub valor: f32,
    pub unidade: Unidade,
}

/// Uma escolha de um tipo (Mode, Space, Mix): `opcoes` são as chaves `ph2d-i18n`, `valor` o índice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AjusteEscolha {
    pub campo: u16,
    pub rotulo: &'static str,
    pub opcoes: &'static [&'static str],
    pub valor: u8,
}

/// O que um item faz neste quadro — o espelho, sem ECS, do `ph2d_constraints_ecs::Estado`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EstadoDaConstraint {
    #[default]
    Activa,
    Desligada,
    SemForca,
    SemAlvo,
    AlvoSiMesmo,
    Ciclo,
    CorpoDinamico,
    EspacoColapsado,
}

impl EstadoDaConstraint {
    /// A chave `ph2d-i18n` da frase que o cartão escreve por baixo do item (`None` = activa: nada a
    /// dizer).
    #[must_use]
    pub fn frase(self) -> Option<&'static str> {
        Some(match self {
            Self::Activa => return None,
            Self::Desligada => "panel.inspector.constraint.state.off",
            Self::SemForca => "panel.inspector.constraint.state.no_influence",
            Self::SemAlvo => "panel.inspector.constraint.state.no_target",
            Self::AlvoSiMesmo => "panel.inspector.constraint.state.self_target",
            Self::Ciclo => "panel.inspector.constraint.state.cycle",
            Self::CorpoDinamico => "panel.inspector.constraint.state.dynamic_body",
            Self::EspacoColapsado => "panel.inspector.constraint.state.collapsed",
        })
    }

    /// `true` ⇒ a frase é um ERRO (cor de aviso); `false` ⇒ é só o estado escolhido pelo artista.
    #[must_use]
    pub fn erro(self) -> bool {
        !matches!(self, Self::Activa | Self::Desligada | Self::SemForca)
    }
}

/// Uma linha do cartão: um item da pilha, de cima para baixo.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorConstraintRow {
    /// O `Item::id` — o endereço durável das edições.
    pub id: u32,
    /// Índice em [`TIPOS`].
    pub tipo: u8,
    /// O nome do alvo (`None` = sem alvo).
    pub alvo: Option<String>,
    /// O tipo pede um alvo (o Limit Rotation não pede).
    pub precisa_alvo: bool,
    /// A força, `0..=1`.
    pub forca: f32,
    pub ligada: bool,
    pub estado: EstadoDaConstraint,
    /// O próximo clique no palco escolhe o alvo deste item.
    pub escolhendo: bool,
    /// Os números do tipo, por esta ordem (no máximo [`MAX_NUMEROS`]).
    pub numeros: Vec<AjusteNumero>,
    /// As escolhas do tipo (no máximo [`MAX_ESCOLHAS`], cada uma com até [`MAX_OPCOES`]).
    pub escolhas: Vec<AjusteEscolha>,
    /// O tipo tem desvio que se pode zerar (o botão «Reset Offset»).
    pub zera_desvio: bool,
}

/// O snapshot do cartão para a entidade escolhida (`None` = ela não tem pilha: o cartão não aparece).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InspectorConstraintsInfo {
    pub entity_bits: u64,
    pub rows: Vec<InspectorConstraintRow>,
    pub selected_count: usize,
}

/// O que o cartão pede. O item é endereçado pelo `id`, nunca pela posição.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConstraintFieldEdit {
    /// Acrescenta no FIM um item do tipo `TIPOS[i]`.
    Adicionar(u8),
    Forca(u32, f32),
    Ligada(u32, bool),
    Apagar(u32),
    /// Sobe (`-1`) ou desce (`+1`) uma posição.
    Mover(u32, i8),
    /// Arma o clique no palco que escolhe o alvo (outro clique no mesmo botão desarma).
    EscolherAlvo(u32),
    /// Um ajuste do tipo: `(item, campo, valor)` — o número na unidade do cartão, ou o índice da
    /// opção. `campo` é o de [`AjusteNumero::campo`]/[`AjusteEscolha::campo`].
    Campo(u32, u16, f32),
    /// Zera o desvio: o dono cola-se ao alvo.
    ZerarDesvio(u32),
}
