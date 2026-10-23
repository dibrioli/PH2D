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
    /// O n-ésimo ColorDrop do C&F (a cor solta na tela), a contar de 1 — tem cor, não tamanho.
    Preenchimento(usize),
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
    /// Uma gravação está armada (o botão é Parar), com ou sem família aberta (W18: o objecto Macro arma
    /// vazio).
    pub armado: bool,
    /// O nome do objecto Macro que o Gravar arma — ou em que se grava.
    pub objeto_macro: Option<String>,
    /// **A biblioteca do utilizador** (W18), por nome.
    pub biblioteca: Vec<MacroDaBibliotecaView>,
    /// Quantos ficheiros da pasta não se leram (nunca em silêncio).
    pub biblioteca_ilegiveis: usize,
    /// O objecto escolhido tem uma gravação que se pode guardar (o «Guardar na biblioteca»).
    pub pode_guardar: bool,
    /// O nome do objecto escolhido por cima do qual «Aplicar por cima» toca; `None` = o botão não se
    /// pinta e [`Self::por_cima_recusa`] diz porquê.
    pub por_cima_de: Option<String>,
    /// Os bits desse objecto — o `alvo` de [`OndeAplicar::PorCima`].
    pub por_cima_alvo: Option<u64>,
    pub por_cima_recusa: Option<&'static str>,
    /// A frase do último gesto da biblioteca (guardou, exportou para…, recusou porque…): a chave e o
    /// detalhe que ela cita.
    pub aviso: Option<ErroDaMacroView>,
}

/// Uma macro da biblioteca como o painel a lista.
#[derive(Clone, Debug, PartialEq)]
pub struct MacroDaBibliotecaView {
    pub nome: String,
    /// As famílias das trilhas, pela ordem.
    pub familias: Vec<FamiliaDaMacro>,
    pub duracao_s: f64,
    /// O tamanho da imagem que a macro cria (a 1.ª trilha do Painter).
    pub tela: Option<(u32, u32)>,
}

/// Onde «Aplicar» põe a macro (decisão 3 do dono: «escolho na hora»).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OndeAplicar {
    /// Nasce o objecto Macro e o que ele cria; `em` = o ponto do mundo (`None` = o centro da vista).
    Novo { em: Option<[f32; 2]> },
    /// O objecto Macro nasce por cima do objecto `alvo` (bits) e toca nele.
    PorCima { alvo: u64 },
}

/// Uma propriedade de uma etapa.
#[derive(Clone, Debug, PartialEq)]
pub enum CampoDaEtapa {
    Cor([u8; 3]),
    Tamanho(f32),
    Valor(f64),
    /// A espera antes da etapa, em segundos (absoluta): ela e as seguintes andam.
    Espera(f64),
    /// **O instante do fim da etapa** na trilha que toca, em segundos (absoluto; o losango da linha
    /// «Macro steps» largado, W16): a etapa anda SOZINHA até lá — as outras ficam.
    Instante(f64),
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
    VaiPara {
        entity_bits: u64,
        t_s: f64,
    },
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
    MostraMe {
        entity_bits: u64,
    },
    /// O tutorial «fazer»: o objecto volta ao estado inicial (o balão, «Recomeçar»).
    Recomeca {
        entity_bits: u64,
    },
    /// **Biblioteca** (W18): guarda a gravação do objecto Macro `entity_bits`.
    Guarda {
        entity_bits: u64,
    },
    /// Aplica a macro `nome` da biblioteca.
    Aplica {
        nome: String,
        onde: OndeAplicar,
    },
    Renomeia {
        de: String,
        para: String,
    },
    ApagaDaBiblioteca {
        nome: String,
    },
    /// Exportar…: o diálogo de gravar escolhe o ficheiro.
    Exporta {
        nome: String,
    },
    /// Importar…: o diálogo de abrir escolhe o ficheiro.
    Importa,
    /// Guarda a gravação do objecto Macro `entity_bits` como asset do projecto (receita).
    GuardaComoAsset {
        entity_bits: u64,
    },
}

/// Gravar: o Painter não tem tela.
pub const MACRO_RECUSA_SEM_TELA: &str = "panel.macros.refusal.no_canvas";
/// Gravar: já se está a gravar.
pub const MACRO_RECUSA_JA_GRAVA: &str = "panel.macros.refusal.already_recording";
/// Gravar: nenhuma família grava aqui (nem o Painter em Wet Paint nem um objecto Motion em edição).
pub const MACRO_RECUSA_SEM_FAMILIA: &str = "panel.macros.refusal.nothing_to_record";
/// Gravar no objecto Macro: a família trabalha num objecto que não está dentro dele (W18).
pub const MACRO_RECUSA_FORA_DO_OBJETO: &str = "panel.macros.refusal.outside_the_macro";
/// Gravar no objecto Macro: a imagem (ou o Motion) está dentro de outro objecto dentro dele.
pub const MACRO_RECUSA_FUNDO_DEMAIS: &str = "panel.macros.refusal.too_deep";
/// Gravar no objecto Macro: esta gravação já leva uma imagem (ou um Motion) — uma de cada família.
pub const MACRO_RECUSA_UM_DE_CADA: &str = "panel.macros.refusal.one_per_family";
/// Parar sem que nada tenha sido gravado.
pub const MACRO_RECUSA_NADA_GRAVADO: &str = "panel.macros.refusal.nothing_recorded";

// ─── A biblioteca e o aplicar (W18, `docs/Macros/16_W18_plano.md` §2.5–2.7): as frases do `aviso` ───

/// A família da trilha não sabe levá-la a outro documento.
pub const MACRO_NAO_TRANSPOE: &str = "app.macros.lib.family_cannot_transpose";
/// Guardou na biblioteca (`{detail}` = o nome).
pub const MACRO_LIB_GUARDADA: &str = "app.macros.lib.saved";
/// O objecto escolhido não tem gravação.
pub const MACRO_LIB_SEM_GRAVACAO: &str = "app.macros.lib.nothing_to_save";
/// O disco recusou (`{detail}` = o caminho e o porquê).
pub const MACRO_LIB_DISCO: &str = "app.macros.lib.disk";
/// Já há uma macro com esse nome (`{detail}`).
pub const MACRO_LIB_JA_EXISTE: &str = "app.macros.lib.name_taken";
/// O nome, limpo, ficou vazio.
pub const MACRO_LIB_NOME_VAZIO: &str = "app.macros.lib.empty_name";
/// O ficheiro não é uma macro que este build toque.
pub const MACRO_LIB_ILEGIVEL: &str = "app.macros.lib.unreadable";
/// Não há pasta da biblioteca (nem `HOME`).
pub const MACRO_LIB_SEM_PASTA: &str = "app.macros.lib.no_folder";
/// A macro `{detail}` já não está na biblioteca.
pub const MACRO_LIB_NAO_HA: &str = "app.macros.lib.missing";
/// Exportou para `{detail}`.
pub const MACRO_LIB_EXPORTADA: &str = "app.macros.lib.exported";
/// Importou como `{detail}`.
pub const MACRO_LIB_IMPORTADA: &str = "app.macros.lib.imported";
pub const MACRO_LIB_RENOMEADA: &str = "app.macros.lib.renamed";
pub const MACRO_LIB_APAGADA: &str = "app.macros.lib.deleted";
/// A macro não toca até ao fim (`{detail}` = onde parou): não se exporta nem se aplica.
pub const MACRO_LIB_NAO_TOCA: &str = "app.macros.lib.does_not_play";
/// Aplicou `{detail}`.
pub const MACRO_APLICADA: &str = "app.macros.apply.applied";
/// Por cima: a macro tem trilhas de duas famílias e o alvo é um objecto só.
pub const MACRO_APLICAR_MISTO: &str = "app.macros.apply.over_mixed";
/// Por cima: o objecto escolhido não recebe esta macro (nem imagem nem objecto Motion).
pub const MACRO_APLICAR_INCOMPATIVEL: &str = "app.macros.apply.over_incompatible";
/// Por cima: nada escolhido — escolha uma imagem ou um objecto Motion.
pub const MACRO_POR_CIMA_ESCOLHA: &str = "panel.macros.lib.apply_over_needs_target";
/// O objecto alvo já não existe.
pub const MACRO_APLICAR_SEM_ALVO: &str = "app.macros.apply.no_target";
/// O objecto que a macro cria não nasceu (`{detail}`).
pub const MACRO_APLICAR_NAO_NASCE: &str = "app.macros.apply.not_born";
/// Guardou a receita nos assets do projecto (`{detail}` = o nome).
pub const MACRO_ASSET_GUARDADO: &str = "app.macros.asset.saved";
