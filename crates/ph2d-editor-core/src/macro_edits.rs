//! **O vocabulário da secção MACRO** (ADR-0186) — a gravação de um objecto.
//!
//! O painel não vê a `ph2d-ecs` nem a `ph2d-macro` (ADR-0029): o modo viaja como o ÍNDICE de
//! `ModoDaMacro`, e a gravação como o que a secção mostra dela. A tradução é da
//! `ph2d-app-components`, com gate de ida-e-volta.

/// Quantos modos há — o comprimento de `ModoDaMacro` (gate na `macro_inspector`).
pub const MACRO_MODOS: usize = 3;

/// Índice do modo `Fiel` (o que o artista viu).
pub const MODO_FIEL: u8 = 0;
/// Índice do modo `Explicativo` (o efeito de cada nó).
pub const MODO_EXPLICATIVO: u8 = 1;
/// Índice do modo `Fazer` (o tutorial: o aluno faz cada etapa).
pub const MODO_FAZER: u8 = 2;

/// Uma edição da secção.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MacroFieldEdit {
    /// O modo, pelo índice de `ModoDaMacro`.
    Modo(u8),
}

/// A gravação, lida do componente.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MacroGravacao {
    /// Nada gravado.
    Vazia,
    /// Cada trilha (a família e quantos passos), e a duração em microssegundos.
    Lida {
        trilhas: Vec<(String, usize)>,
        duracao_us: u64,
    },
    /// O ficheiro não se lê neste build: a chave da frase.
    Ilegivel(&'static str),
}

/// Gravada por uma versão mais nova (outra major do formato).
pub const MACRO_DE_VERSAO_NOVA: &str = "panel.inspector.macro.newer_version";
/// Os bytes não são uma macro, ou estão cortados.
pub const MACRO_ESTRAGADA: &str = "panel.inspector.macro.damaged";

/// O instantâneo da secção.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorMacroInfo {
    /// Os bits da entidade — o endereço que toda edição carrega.
    pub entity_bits: u64,
    /// O modo, pelo índice de `ModoDaMacro`.
    pub modo: u8,
    pub gravacao: MacroGravacao,
    /// Quantos objectos estão escolhidos (a edição aplica-se ao activo).
    pub selected_count: usize,
    /// Quantas etapas a gravação tem (as do painel MACROS).
    pub etapas: usize,
    /// A 1.ª etapa com edições, a contar de 0 (o cartão diz «editada desde a etapa N+1»).
    pub editada_desde: Option<usize>,
    /// O erro que chega ao artista: o da reprodução em cena vence o da leitura.
    pub erro: Option<ErroDaMacroView>,
}

// ─── O painel MACROS e a linha de etapas (W5d, `docs/Macros/02_plano_W5.md` §5) ───────────────────

/// A família de uma trilha.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FamiliaDaMacro {
    Painter,
    Motion,
}

/// O que uma etapa fez — o painel dá-lhe o nome pelo i18n (os tipos de nó e os params chegam pelo
/// nome canónico, `node.<tipo>.name`).
#[derive(Clone, Debug, PartialEq)]
pub enum TipoDaEtapaView {
    /// A n-ésima pincelada (a contar de 1).
    Pincelada(usize),
    Acrescenta(Vec<String>),
    Liga {
        de: Option<String>,
        para: Option<String>,
    },
    Desliga {
        para: Option<String>,
    },
    Apaga(usize),
    Move,
    Renomeia(String),
    Param {
        no: Option<String>,
        param: String,
        de: Option<f32>,
        para: f64,
    },
    Grafo,
    Direta,
    /// Uma porta que a macro não reproduz: o porquê (chave do i18n ou nome da porta).
    Recusada(String),
}

/// As propriedades editáveis de uma etapa, além da espera e do nome.
#[derive(Clone, Debug, PartialEq)]
pub enum PropsDaEtapa {
    Pincelada { cor: [u8; 3], tamanho_px: f32 },
    Param { valor: f64 },
    Nenhuma,
}

/// Uma etapa como o painel e a linha da Timeline a mostram.
#[derive(Clone, Debug, PartialEq)]
pub struct EtapaView {
    /// A trilha no `MacroDoc` e a etapa dentro dela — o endereço de toda edição.
    pub trilha: usize,
    pub k: usize,
    pub familia: FamiliaDaMacro,
    pub tipo: TipoDaEtapaView,
    /// O nome que o artista escreveu.
    pub nome: Option<String>,
    /// O texto do balão do tutorial que o autor escreveu.
    pub dica: Option<String>,
    /// Início e fim na trilha QUE TOCA, em segundos de macro.
    pub t0_s: f64,
    pub t1_s: f64,
    /// A espera editada antes da etapa, em segundos (+ abre, − encurta).
    pub espera_s: f64,
    pub props: PropsDaEtapa,
}

/// Um erro que chega ao artista (o plano 01 §5.6): a chave da frase e o detalhe que ela cita.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErroDaMacroView {
    pub chave: &'static str,
    pub detalhe: Option<String>,
}

/// O objecto escolhido que tem uma `Macro`.
#[derive(Clone, Debug, PartialEq)]
pub struct MacroObjetoView {
    pub entity_bits: u64,
    pub duracao_s: f64,
    pub etapas: Vec<EtapaView>,
    /// A 1.ª etapa com edições (o cartão diz «editada desde a etapa N»).
    pub editada_desde: Option<usize>,
    pub erro: Option<ErroDaMacroView>,
}

/// O instantâneo do painel MACROS.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MacroPanelInfo {
    /// A família que está a gravar agora.
    pub gravando: Option<FamiliaDaMacro>,
    /// A família que o botão Gravar armaria agora (`None` = o botão diz porquê, em `recusa`).
    pub pode_gravar: Option<FamiliaDaMacro>,
    /// A chave da frase: porque não se pode gravar, ou porque a última tentativa recusou.
    pub recusa: Option<&'static str>,
    pub objeto: Option<MacroObjetoView>,
}

/// Uma propriedade de uma etapa.
#[derive(Clone, Debug, PartialEq)]
pub enum CampoDaEtapa {
    Cor([u8; 3]),
    Tamanho(f32),
    Valor(f64),
    /// A espera antes da etapa, em segundos (absoluta).
    Espera(f64),
    /// `None` = volta ao nome derivado.
    Nome(Option<String>),
    /// O texto do balão do tutorial «fazer» nesta etapa; `None` = só a frase derivada.
    Dica(Option<String>),
}

/// O que o painel MACROS e a linha de etapas da Timeline pedem.
#[derive(Clone, Debug, PartialEq)]
pub enum MacroRequest {
    /// Arma a gravação da família que [`MacroPanelInfo::pode_gravar`] diz.
    Grava,
    /// Pára: o objecto fica com a `Macro` e o clipe na Timeline.
    Para,
    /// Leva a cabeça de leitura ao fim da etapa.
    VaiPara { entity_bits: u64, t_s: f64 },
    Edita {
        entity_bits: u64,
        trilha: usize,
        k: usize,
        campo: CampoDaEtapa,
    },
    /// Apaga a etapa: o tempo dela fecha-se.
    Apaga {
        entity_bits: u64,
        trilha: usize,
        k: usize,
    },
    /// O tutorial «fazer»: a etapa corrente feita pelo aluno (o balão, «Mostra-me»).
    MostraMe { entity_bits: u64 },
    /// O tutorial «fazer»: o objecto volta ao estado inicial (o balão, «Recomeçar»).
    Recomeca { entity_bits: u64 },
}

/// Gravar: só o Wet Paint grava nesta versão.
pub const MACRO_RECUSA_MEIO: &str = "panel.macros.refusal.wet_paint_only";
/// Gravar: o Painter não tem tela.
pub const MACRO_RECUSA_SEM_TELA: &str = "panel.macros.refusal.no_canvas";
/// Gravar: já se está a gravar.
pub const MACRO_RECUSA_JA_GRAVA: &str = "panel.macros.refusal.already_recording";
/// Gravar: nenhuma família grava aqui (nem o Painter em Wet Paint nem um objecto Motion em edição).
pub const MACRO_RECUSA_SEM_FAMILIA: &str = "panel.macros.refusal.nothing_to_record";
