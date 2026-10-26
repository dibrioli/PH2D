//! **O vocabulário da secção DRAWING RULES** (W4.5, `docs/BlenderConstraints/04_W4_cad.md` §3): as regras de
//! desenho técnico cujo dono é o path escolhido ([`InspectorCadInfo`]) e o que a secção pede ([`CadFieldEdit`]).
//! Quem lê o documento e aplica é `ph2d-app-vec` (`cad_inspector.rs`); o painel só pinta e emite.

/// Quantas regras a secção pinta e edita — o tamanho das tabelas de ids do painel (`ids/inspector_cad.rs`), o
/// único recurso que a limita; o documento em si não tem tecto. Acima disto a secção diz quantas ficaram de fora.
pub const MAX_REGRAS: usize = 32;

/// A cor de uma regra — o espelho, sem a crate do solver, do `ph2d_vec_constraints::EstadoRegra`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EstadoDaRegra {
    #[default]
    Ok,
    /// REF: só mede (ou a regra perdeu o alvo) — não manda na geometria.
    Ref,
    Redundante,
    Conflito,
}

/// Uma linha da secção: uma regra do documento, pela ordem dele.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorRegraRow {
    /// O `RegraId` — o endereço durável das edições.
    pub id: u32,
    /// A chave `ph2d-i18n` do nome da lei (`app.vec.cad.lei.*`).
    pub nome: &'static str,
    pub estado: EstadoDaRegra,
    /// A medida na unidade do canal (px; GRAUS num ângulo) — `None` = a lei não tem medida.
    pub medida: Option<f32>,
    /// A medida é um ângulo.
    pub graus: bool,
    /// REF: a medida só se lê.
    pub referencia: bool,
    /// O canal da medida (`ph2d_ecs::canal_cad::chave`), como `[família, campo, i, j]`.
    pub canal: [u16; 4],
}

/// O snapshot da secção para o objecto escolhido (`None` = o path não é dono de regra nenhuma: a secção não
/// aparece).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InspectorCadInfo {
    pub entity_bits: u64,
    pub rows: Vec<InspectorRegraRow>,
}

/// O que a secção pede. A regra é endereçada pelo `RegraId`, nunca pela posição.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CadFieldEdit {
    /// A medida nova, na unidade do canal (px; graus num ângulo): a folha re-resolve a partir do desenho.
    Medida(u32, f32),
    /// Tira a regra do documento (a cura do «Conflict — delete one of: …»).
    Apagar(u32),
}
